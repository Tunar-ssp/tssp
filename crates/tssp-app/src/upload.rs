//! Upload use case orchestration.

use std::io::Read;

use thiserror::Error;
use tssp_domain::{DomainError, FileName, FileRecord, MimeType, Tag};
use tssp_ports::{
    BlobStore, BlobStoreError, Clock, FileRepository, IdGenerationError, IdGenerator,
    NewFileRecord, RepositoryError,
};

/// Coordinates streaming blob storage and metadata insertion for one upload.
pub struct UploadService<B, R, I, C> {
    blob_store: B,
    repository: R,
    id_generator: I,
    clock: C,
}

impl<B, R, I, C> UploadService<B, R, I, C> {
    /// Creates an upload service from explicit infrastructure ports.
    #[must_use]
    pub const fn new(blob_store: B, repository: R, id_generator: I, clock: C) -> Self {
        Self {
            blob_store,
            repository,
            id_generator,
            clock,
        }
    }
}

impl<B, R, I, C> UploadService<B, R, I, C>
where
    B: BlobStore,
    R: FileRepository,
    I: IdGenerator,
    C: Clock,
{
    /// Streams an upload into storage, creates metadata, and cleans up on failure.
    ///
    /// # Errors
    ///
    /// Returns [`UploadError`] when request metadata is invalid, id generation
    /// fails, blob storage fails, or metadata commit fails.
    pub fn upload(&self, request: &mut UploadRequest<'_>) -> Result<UploadResult, UploadError> {
        let name = FileName::new(request.filename)?;
        let mime_type = request
            .mime_type
            .map(MimeType::new)
            .transpose()?
            .unwrap_or_else(MimeType::octet_stream);
        let tags = normalize_tags(request.tags)?;
        let blob = self.blob_store.put_stream(request.source)?;

        let new_file = NewFileRecord {
            id: self.id_generator.new_file_id()?,
            name,
            size: blob.size,
            content_hash: blob.content_hash,
            mime_type,
            storage_handle: blob.handle.clone(),
            uploaded_at: self.clock.now(),
            tags,
            pinned_at: request.pinned_at,
        };

        match self.repository.insert_file(new_file) {
            Ok(record) => Ok(UploadResult {
                record,
                deduplicated: blob.deduplicated,
            }),
            Err(error) => {
                let cleanup = self.blob_store.cleanup_unreferenced(&blob.handle);
                Err(UploadError::commit_failed(error, cleanup.err()))
            }
        }
    }
}

/// Input for a single uploaded file.
pub struct UploadRequest<'a> {
    /// User-facing filename supplied by the client.
    pub filename: &'a str,
    /// Optional MIME type supplied or detected by the delivery layer.
    pub mime_type: Option<&'a str>,
    /// Initial tags supplied by the client.
    pub tags: &'a [&'a str],
    /// Optional initial pin order.
    pub pinned_at: Option<u32>,
    /// Streaming content source.
    pub source: &'a mut dyn Read,
}

/// Successful upload result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadResult {
    /// Created logical file record.
    pub record: FileRecord,
    /// True when blob bytes were already present.
    pub deduplicated: bool,
}

/// Upload use-case failure.
#[derive(Debug, Error)]
pub enum UploadError {
    /// Invalid request metadata.
    #[error(transparent)]
    InvalidRequest(#[from] DomainError),

    /// File id generation failed.
    #[error(transparent)]
    IdGeneration(#[from] IdGenerationError),

    /// Blob storage failed before metadata commit.
    #[error(transparent)]
    BlobStore(#[from] BlobStoreError),

    /// Metadata commit failed after blob storage succeeded.
    #[error("metadata commit failed after blob write")]
    CommitFailed {
        /// Repository error that caused the failed commit.
        repository: RepositoryError,
        /// Cleanup error, if cleanup also failed.
        cleanup: Option<BlobStoreError>,
    },
}

impl UploadError {
    fn commit_failed(repository: RepositoryError, cleanup: Option<BlobStoreError>) -> Self {
        Self::CommitFailed {
            repository,
            cleanup,
        }
    }
}

fn normalize_tags(tags: &[&str]) -> Result<Vec<Tag>, DomainError> {
    let mut normalized = Vec::with_capacity(tags.len());
    for tag in tags {
        let parsed = Tag::new(tag)?;
        if !normalized
            .iter()
            .any(|existing: &Tag| existing.key() == parsed.key())
        {
            normalized.push(parsed);
        }
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::io::{Cursor, Read};

    use tssp_domain::{
        ContentHash, FileId, FileName, FileRecord, FileSize, MimeType, StorageHandle, UnixTimestamp,
    };
    use tssp_ports::{
        BlobStore, BlobStoreError, BlobWriteOutcome, Clock, FileRepository, IdGenerationError,
        IdGenerator, NewFileRecord, RepositoryError, RepositoryStats,
    };

    use super::{UploadError, UploadRequest, UploadService};

    struct FixedClock;

    impl Clock for FixedClock {
        fn now(&self) -> UnixTimestamp {
            match UnixTimestamp::new(1_700_000_000) {
                Ok(value) => value,
                Err(error) => panic!("invalid fixed timestamp: {error}"),
            }
        }
    }

    struct QueueIds {
        ids: RefCell<VecDeque<FileId>>,
    }

    impl QueueIds {
        fn new(ids: Vec<FileId>) -> Self {
            Self {
                ids: RefCell::new(VecDeque::from(ids)),
            }
        }
    }

    impl IdGenerator for QueueIds {
        fn new_file_id(&self) -> Result<FileId, IdGenerationError> {
            match self.ids.borrow_mut().pop_front() {
                Some(value) => Ok(value),
                None => Err(IdGenerationError {
                    message: "id queue is empty".to_owned(),
                }),
            }
        }
    }

    struct FakeBlobStore {
        outcome: BlobWriteOutcome,
        cleanup_calls: RefCell<Vec<StorageHandle>>,
    }

    impl BlobStore for FakeBlobStore {
        fn put_stream(&self, source: &mut dyn Read) -> Result<BlobWriteOutcome, BlobStoreError> {
            let mut buffer = [0_u8; 8];
            loop {
                match source.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(error) => {
                        return Err(BlobStoreError::ReadFailed {
                            message: error.to_string(),
                        });
                    }
                }
            }
            Ok(self.outcome.clone())
        }

        fn cleanup_unreferenced(&self, handle: &StorageHandle) -> Result<(), BlobStoreError> {
            self.cleanup_calls.borrow_mut().push(handle.clone());
            Ok(())
        }
    }

    struct FakeRepository {
        failure: Option<RepositoryError>,
    }

    impl FileRepository for FakeRepository {
        fn insert_file(&self, new_file: NewFileRecord) -> Result<FileRecord, RepositoryError> {
            if let Some(error) = &self.failure {
                return Err(match error {
                    RepositoryError::Busy => RepositoryError::Busy,
                    RepositoryError::Conflict { message } => RepositoryError::Conflict {
                        message: message.clone(),
                    },
                    RepositoryError::NotFound => RepositoryError::NotFound,
                    RepositoryError::OperationFailed { message } => {
                        RepositoryError::OperationFailed {
                            message: message.clone(),
                        }
                    }
                });
            }

            Ok(FileRecord {
                id: new_file.id,
                name: new_file.name,
                size: new_file.size,
                content_hash: new_file.content_hash,
                mime_type: new_file.mime_type,
                storage_handle: new_file.storage_handle,
                uploaded_at: new_file.uploaded_at,
                tags: new_file.tags,
                pinned_at: new_file.pinned_at,
            })
        }

        fn find_file(&self, _id: &FileId) -> Result<Option<FileRecord>, RepositoryError> {
            Ok(None)
        }

        fn stats_since(
            &self,
            _recent_since: UnixTimestamp,
        ) -> Result<RepositoryStats, RepositoryError> {
            Ok(RepositoryStats {
                file_count: 0,
                tag_count: 0,
                pinned_count: 0,
                recent_upload_count: 0,
            })
        }
    }

    fn valid_hash() -> ContentHash {
        match ContentHash::new("abcdefabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123") {
            Ok(value) => value,
            Err(error) => panic!("invalid test hash: {error}"),
        }
    }

    fn file_id(value: &str) -> FileId {
        match FileId::new(value) {
            Ok(id) => id,
            Err(error) => panic!("invalid test file id: {error}"),
        }
    }

    fn handle() -> StorageHandle {
        match StorageHandle::new("blobs/ab/cd/hash") {
            Ok(value) => value,
            Err(error) => panic!("invalid test storage handle: {error}"),
        }
    }

    fn filename(value: &str) -> FileName {
        match FileName::new(value) {
            Ok(name) => name,
            Err(error) => panic!("invalid test filename: {error}"),
        }
    }

    fn mime_type(value: &str) -> MimeType {
        match MimeType::new(value) {
            Ok(mime_type) => mime_type,
            Err(error) => panic!("invalid test MIME type: {error}"),
        }
    }

    fn blob_store(deduplicated: bool) -> FakeBlobStore {
        FakeBlobStore {
            outcome: BlobWriteOutcome {
                content_hash: valid_hash(),
                handle: handle(),
                size: FileSize::new(11),
                deduplicated,
            },
            cleanup_calls: RefCell::new(Vec::new()),
        }
    }

    #[test]
    fn upload_streams_blob_and_inserts_metadata() {
        let store = blob_store(false);
        let service = UploadService::new(
            store,
            FakeRepository { failure: None },
            QueueIds::new(vec![file_id("file-1")]),
            FixedClock,
        );
        let mut source = Cursor::new(b"hello world".as_slice());
        let mut request = UploadRequest {
            filename: "photo.jpg",
            mime_type: Some("IMAGE/JPEG"),
            tags: &["Family", " family "],
            pinned_at: Some(1),
            source: &mut source,
        };

        let result = service.upload(&mut request);

        assert!(matches!(
            result,
            Ok(value) if value.record.id.as_str() == "file-1"
                && value.record.name == filename("photo.jpg")
                && value.record.mime_type == mime_type("image/jpeg")
                && value.record.tags.len() == 1
                && value.record.is_pinned()
                && !value.deduplicated
        ));
    }

    #[test]
    fn upload_cleans_blob_when_metadata_commit_fails() {
        let store = blob_store(true);
        let expected_handle = store.outcome.handle.clone();
        let service = UploadService::new(
            store,
            FakeRepository {
                failure: Some(RepositoryError::Busy),
            },
            QueueIds::new(vec![file_id("file-1")]),
            FixedClock,
        );
        let mut source = Cursor::new(b"hello world".as_slice());
        let mut request = UploadRequest {
            filename: "photo.jpg",
            mime_type: None,
            tags: &[],
            pinned_at: None,
            source: &mut source,
        };

        let result = service.upload(&mut request);

        assert!(matches!(
            result,
            Err(UploadError::CommitFailed {
                repository: RepositoryError::Busy,
                cleanup: None,
            })
        ));
        assert_eq!(
            service.blob_store.cleanup_calls.borrow().as_slice(),
            &[expected_handle]
        );
    }
}
