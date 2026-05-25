use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::{Cursor, Read};
use std::path::Path;

use tssp_domain::{
    ContentHash, FileId, FileName, FileRecord, FileSize, MimeType, StorageHandle, Tag, TagKey,
    UnixTimestamp,
};
use tssp_ports::{
    BlobStore, BlobStoreError, BlobWriteOutcome, Clock, DeletedFileRecord, FileRepository,
    IdGenerationError, IdGenerator, NewFileRecord, PinOutcome, RepositoryError, RepositoryStats,
    TagMutationOutcome, TagSummary,
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

    fn put_staged(
        &self,
        _temp_path: &Path,
        _content_hash: &ContentHash,
        _size: FileSize,
    ) -> Result<BlobWriteOutcome, BlobStoreError> {
        Ok(self.outcome.clone())
    }

    fn cleanup_unreferenced(&self, handle: &StorageHandle) -> Result<(), BlobStoreError> {
        self.cleanup_calls.borrow_mut().push(handle.clone());
        Ok(())
    }
}

struct FakeRepository {
    failure: Option<RepositoryError>,
    existing_by_hash: Option<FileRecord>,
    insert_calls: RefCell<usize>,
}

impl FakeRepository {
    fn ok() -> Self {
        Self {
            failure: None,
            existing_by_hash: None,
            insert_calls: RefCell::new(0),
        }
    }

    fn failing(error: RepositoryError) -> Self {
        Self {
            failure: Some(error),
            existing_by_hash: None,
            insert_calls: RefCell::new(0),
        }
    }

    fn with_existing(existing: FileRecord) -> Self {
        Self {
            failure: None,
            existing_by_hash: Some(existing),
            insert_calls: RefCell::new(0),
        }
    }
}

impl FileRepository for FakeRepository {
    fn insert_file(&self, new_file: NewFileRecord) -> Result<FileRecord, RepositoryError> {
        *self.insert_calls.borrow_mut() += 1;
        if let Some(error) = &self.failure {
            return Err(clone_repository_error(error));
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
            folder_path: String::new(),
            owner_id: None,
            visibility: tssp_domain::Visibility::Private,
            public_token: None,
        })
    }

    fn find_file(&self, _id: &FileId) -> Result<Option<FileRecord>, RepositoryError> {
        Ok(None)
    }

    fn find_file_by_content_hash(
        &self,
        _content_hash: &ContentHash,
    ) -> Result<Option<FileRecord>, RepositoryError> {
        if let Some(error) = &self.failure {
            return Err(clone_repository_error(error));
        }

        Ok(self.existing_by_hash.clone())
    }

    fn delete_file(&self, _id: &FileId) -> Result<Option<DeletedFileRecord>, RepositoryError> {
        Ok(None)
    }

    fn list_files(
        &self,
        _query: &tssp_ports::ListQuery,
    ) -> Result<tssp_ports::PagedFiles, RepositoryError> {
        Ok(tssp_ports::PagedFiles {
            files: Vec::new(),
            next_cursor: None,
        })
    }

    fn list_files_recent(&self, _limit: u64) -> Result<Vec<FileRecord>, RepositoryError> {
        Ok(Vec::new())
    }

    fn list_tags(&self) -> Result<Vec<TagSummary>, RepositoryError> {
        Ok(Vec::new())
    }

    fn add_tags_to_file(
        &self,
        _id: &FileId,
        _tags: &[Tag],
    ) -> Result<TagMutationOutcome, RepositoryError> {
        Ok(TagMutationOutcome { changed_count: 0 })
    }

    fn remove_tag_from_file(
        &self,
        _id: &FileId,
        _tag: &TagKey,
    ) -> Result<TagMutationOutcome, RepositoryError> {
        Ok(TagMutationOutcome { changed_count: 0 })
    }

    fn stats_since(
        &self,
        _recent_since: UnixTimestamp,
    ) -> Result<RepositoryStats, RepositoryError> {
        Ok(RepositoryStats {
            file_count: 0,
            note_count: 0,
            tag_count: 0,
            pinned_count: 0,
            recent_upload_count: 0,
            recent_note_count: 0,
                storage_bytes_used: 0,
        })
    }

    fn pin_file(
        &self,
        _id: &FileId,
        _position: Option<u32>,
    ) -> Result<PinOutcome, RepositoryError> {
        Ok(PinOutcome {
            existed: true,
            changed: false,
        })
    }

    fn unpin_file(&self, _id: &FileId) -> Result<PinOutcome, RepositoryError> {
        Ok(PinOutcome {
            existed: true,
            changed: false,
        })
    }

    fn list_pinned_files(&self) -> Result<Vec<FileRecord>, RepositoryError> {
        Ok(Vec::new())
    }

    fn reorder_pins(&self, _ordered_ids: &[FileId]) -> Result<(), RepositoryError> {
        Ok(())
    }

    fn search_files(&self, _query: &str) -> Result<Vec<FileRecord>, RepositoryError> {
        Ok(Vec::new())
    }

    fn list_files_by_tag(
        &self,
        _tag: &tssp_domain::TagKey,
        _limit: u64,
    ) -> Result<Vec<FileRecord>, RepositoryError> {
        Ok(Vec::new())
    }

    fn rename_file(
        &self,
        _id: &FileId,
        _new_name: &FileName,
    ) -> Result<Option<FileRecord>, RepositoryError> {
        Ok(None)
    }

    fn list_folder_counts(
        &self,
        _owner_id: Option<&tssp_domain::UserId>,
    ) -> Result<Vec<(String, u64)>, RepositoryError> {
        Ok(Vec::new())
    }

    fn set_file_visibility(
        &self,
        _id: &FileId,
        _visibility: tssp_domain::Visibility,
        _public_token: Option<&str>,
    ) -> Result<Option<FileRecord>, RepositoryError> {
        Ok(None)
    }

    fn find_file_by_public_token(
        &self,
        _token: &str,
    ) -> Result<Option<FileRecord>, RepositoryError> {
        Ok(None)
    }

    fn update_folder_path_prefix(
        &self,
        _from_prefix: &str,
        _to_prefix: &str,
    ) -> Result<u64, RepositoryError> {
        Ok(0)
    }

    fn set_file_folder_path(
        &self,
        _id: &FileId,
        _folder_path: &str,
    ) -> Result<Option<FileRecord>, RepositoryError> {
        Ok(None)
    }
}

fn clone_repository_error(error: &RepositoryError) -> RepositoryError {
    match error {
        RepositoryError::Busy => RepositoryError::Busy,
        RepositoryError::Conflict { message } => RepositoryError::Conflict {
            message: message.clone(),
        },
        RepositoryError::NotFound => RepositoryError::NotFound,
        RepositoryError::OperationFailed { message } => RepositoryError::OperationFailed {
            message: message.clone(),
        },
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
        FakeRepository::ok(),
        QueueIds::new(vec![file_id("file-1")]),
        FixedClock,
    );
    let mut source = Cursor::new(b"hello world".as_slice());
    let mut request = UploadRequest {
        filename: "photo.jpg",
        mime_type: Some("IMAGE/JPEG"),
        tags: &["Family", " family "],
        pinned_at: Some(1),
        folder_path: "",
        owner_id: None,
        visibility: tssp_domain::Visibility::Private,
        public_token: None,
        source: &mut source,
        staged_path: None,
        content_hash: None,
        size: None,
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
fn deduplicated_upload_returns_existing_record_without_insert() {
    let existing = existing_record();
    let service = UploadService::new(
        blob_store(true),
        FakeRepository::with_existing(existing),
        QueueIds::new(Vec::new()),
        FixedClock,
    );
    let mut source = Cursor::new(b"hello world".as_slice());
    let mut request = UploadRequest {
        filename: "duplicate.jpg",
        mime_type: Some("image/jpeg"),
        tags: &["ignored"],
        pinned_at: Some(1),
        folder_path: "",
        owner_id: None,
        visibility: tssp_domain::Visibility::Private,
        public_token: None,
        source: &mut source,
        staged_path: None,
        content_hash: None,
        size: None,
    };

    let result = service.upload(&mut request);

    assert!(matches!(
        result,
        Ok(value) if value.record.id.as_str() == "file-existing" && value.deduplicated
    ));
    assert_eq!(*service.repository.insert_calls.borrow(), 0);
}

#[test]
fn upload_cleans_blob_when_metadata_commit_fails() {
    let store = blob_store(false);
    let expected_handle = store.outcome.handle.clone();
    let service = UploadService::new(
        store,
        FakeRepository::failing(RepositoryError::Busy),
        QueueIds::new(vec![file_id("file-1")]),
        FixedClock,
    );
    let mut source = Cursor::new(b"hello world".as_slice());
    let mut request = UploadRequest {
        filename: "photo.jpg",
        mime_type: None,
        tags: &[],
        pinned_at: None,
        folder_path: "",
        owner_id: None,
        visibility: tssp_domain::Visibility::Private,
        public_token: None,
        source: &mut source,
        staged_path: None,
        content_hash: None,
        size: None,
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

#[test]
fn upload_rejects_invalid_request_metadata_before_storage() {
    let service = UploadService::new(
        blob_store(false),
        FakeRepository::ok(),
        QueueIds::new(vec![file_id("file-1")]),
        FixedClock,
    );
    let mut source = Cursor::new(b"hello world".as_slice());
    let mut request = UploadRequest {
        filename: "",
        mime_type: Some("text/plain"),
        tags: &[],
        pinned_at: None,
        folder_path: "",
        owner_id: None,
        visibility: tssp_domain::Visibility::Private,
        public_token: None,
        source: &mut source,
        staged_path: None,
        content_hash: None,
        size: None,
    };

    let result = service.upload(&mut request);

    assert!(matches!(result, Err(UploadError::InvalidRequest(_))));
    assert!(service.blob_store.cleanup_calls.borrow().is_empty());
}

#[test]
fn upload_reports_id_generation_failure_after_blob_write() {
    let service = UploadService::new(
        blob_store(false),
        FakeRepository::ok(),
        QueueIds::new(Vec::new()),
        FixedClock,
    );
    let mut source = Cursor::new(b"hello world".as_slice());
    let mut request = UploadRequest {
        filename: "photo.jpg",
        mime_type: None,
        tags: &[],
        pinned_at: None,
        folder_path: "",
        owner_id: None,
        visibility: tssp_domain::Visibility::Private,
        public_token: None,
        source: &mut source,
        staged_path: None,
        content_hash: None,
        size: None,
    };

    let result = service.upload(&mut request);

    assert!(matches!(result, Err(UploadError::IdGeneration(_))));
}

#[test]
fn upload_reports_blob_read_failure() {
    let service = UploadService::new(
        blob_store(false),
        FakeRepository::ok(),
        QueueIds::new(vec![file_id("file-1")]),
        FixedClock,
    );
    let mut source = FailingReader;
    let mut request = UploadRequest {
        filename: "photo.jpg",
        mime_type: None,
        tags: &[],
        pinned_at: None,
        folder_path: "",
        owner_id: None,
        visibility: tssp_domain::Visibility::Private,
        public_token: None,
        source: &mut source,
        staged_path: None,
        content_hash: None,
        size: None,
    };

    let result = service.upload(&mut request);

    assert!(matches!(
        result,
        Err(UploadError::BlobStore(BlobStoreError::ReadFailed { .. }))
    ));
}

#[test]
fn deduplicated_upload_reports_lookup_failure() {
    let service = UploadService::new(
        blob_store(true),
        FakeRepository::failing(RepositoryError::OperationFailed {
            message: "lookup failed".to_owned(),
        }),
        QueueIds::new(Vec::new()),
        FixedClock,
    );
    let mut source = Cursor::new(b"hello world".as_slice());
    let mut request = UploadRequest {
        filename: "duplicate.jpg",
        mime_type: Some("image/jpeg"),
        tags: &[],
        pinned_at: None,
        folder_path: "",
        owner_id: None,
        visibility: tssp_domain::Visibility::Private,
        public_token: None,
        source: &mut source,
        staged_path: None,
        content_hash: None,
        size: None,
    };

    let result = service.upload(&mut request);

    assert!(matches!(
        result,
        Err(UploadError::DedupLookup(
            RepositoryError::OperationFailed { .. }
        ))
    ));
}

fn existing_record() -> FileRecord {
    FileRecord {
        id: file_id("file-existing"),
        name: filename("original.jpg"),
        size: FileSize::new(11),
        content_hash: valid_hash(),
        mime_type: mime_type("image/jpeg"),
        storage_handle: handle(),
        uploaded_at: FixedClock.now(),
        tags: Vec::new(),
        pinned_at: None,
        folder_path: String::new(),
        owner_id: None,
        visibility: tssp_domain::Visibility::Private,
        public_token: None,
    }
}

struct FailingReader;

impl Read for FailingReader {
    fn read(&mut self, _buffer: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("read failed"))
    }
}
