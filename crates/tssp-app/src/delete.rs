//! Delete use case orchestration.
//!
//! File deletion is metadata-first: once the index record is gone the file is no
//! longer visible to clients. Blob cleanup only runs when the delete transaction
//! proves no remaining logical file references the same content hash.

use thiserror::Error;
use tssp_domain::FileId;
use tssp_ports::{BlobStore, BlobStoreError, FileRepository, RepositoryError};

/// Coordinates metadata deletion and safe content-addressed blob cleanup.
pub struct DeleteFileService<B, R> {
    blob_store: B,
    repository: R,
}

impl<B, R> DeleteFileService<B, R> {
    /// Creates a delete service from explicit infrastructure ports.
    #[must_use]
    pub const fn new(blob_store: B, repository: R) -> Self {
        Self {
            blob_store,
            repository,
        }
    }
}

impl<B, R> DeleteFileService<B, R>
where
    B: BlobStore,
    R: FileRepository,
{
    /// Deletes one logical file id.
    ///
    /// # Errors
    ///
    /// Returns [`DeleteFileError`] when metadata deletion fails or last-reference
    /// blob cleanup cannot complete.
    pub fn delete(&self, id: &FileId) -> Result<DeleteFileResult, DeleteFileError> {
        let Some(deleted) = self.repository.delete_file(id)? else {
            return Ok(DeleteFileResult {
                existed: false,
                blob_cleaned: false,
            });
        };

        if deleted.remaining_content_references > 0 {
            return Ok(DeleteFileResult {
                existed: true,
                blob_cleaned: false,
            });
        }

        self.blob_store
            .cleanup_unreferenced(&deleted.record.storage_handle)?;
        Ok(DeleteFileResult {
            existed: true,
            blob_cleaned: true,
        })
    }
}

/// Successful delete outcome.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DeleteFileResult {
    /// True when a metadata record existed and was removed.
    pub existed: bool,
    /// True when the last blob reference was removed from storage.
    pub blob_cleaned: bool,
}

/// Delete use-case failure.
#[derive(Debug, Error)]
pub enum DeleteFileError {
    /// Metadata delete transaction failed.
    #[error(transparent)]
    Repository(#[from] RepositoryError),

    /// Last-reference blob cleanup failed after metadata deletion.
    #[error(transparent)]
    BlobCleanup(#[from] BlobStoreError),
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::io::Read;
    use std::path::Path;

    use tssp_domain::{
        ContentHash, FileId, FileName, FileRecord, FileSize, MimeType, StorageHandle, Tag, TagKey,
        UnixTimestamp,
    };
    use tssp_ports::{
        BlobStore, BlobStoreError, BlobWriteOutcome, DeletedFileRecord, FileRepository,
        NewFileRecord, PinOutcome, RepositoryError, RepositoryStats, TagMutationOutcome,
        TagSummary,
    };

    use super::{DeleteFileError, DeleteFileService};

    struct FakeBlobStore {
        cleanup_calls: RefCell<Vec<StorageHandle>>,
        cleanup_error: Option<BlobStoreError>,
    }

    impl BlobStore for FakeBlobStore {
        fn put_stream(&self, _source: &mut dyn Read) -> Result<BlobWriteOutcome, BlobStoreError> {
            Err(BlobStoreError::WriteFailed {
                message: "not used by delete tests".to_owned(),
            })
        }

        fn put_staged(
            &self,
            _temp_path: &Path,
            _content_hash: &ContentHash,
            _size: FileSize,
        ) -> Result<BlobWriteOutcome, BlobStoreError> {
            Err(BlobStoreError::WriteFailed {
                message: "not used by delete tests".to_owned(),
            })
        }

        fn cleanup_unreferenced(&self, handle: &StorageHandle) -> Result<(), BlobStoreError> {
            self.cleanup_calls.borrow_mut().push(handle.clone());
            match &self.cleanup_error {
                Some(error) => Err(clone_blob_error(error)),
                None => Ok(()),
            }
        }
    }

    struct FakeRepository {
        deleted: Option<DeletedFileRecord>,
        error: Option<RepositoryError>,
    }

    impl FileRepository for FakeRepository {
        fn insert_file(&self, _new_file: NewFileRecord) -> Result<FileRecord, RepositoryError> {
            Err(RepositoryError::OperationFailed {
                message: "not used by delete tests".to_owned(),
            })
        }

        fn find_file(&self, _id: &FileId) -> Result<Option<FileRecord>, RepositoryError> {
            Ok(None)
        }

        fn find_file_by_content_hash(
            &self,
            _content_hash: &ContentHash,
        ) -> Result<Option<FileRecord>, RepositoryError> {
            Ok(None)
        }

        fn delete_file(&self, _id: &FileId) -> Result<Option<DeletedFileRecord>, RepositoryError> {
            match &self.error {
                Some(error) => Err(clone_repository_error(error)),
                None => Ok(self.deleted.clone()),
            }
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

        fn insert_audit_event(
            &self,
            _id: &str,
            _timestamp: i64,
            _user_id: Option<&str>,
            _action: &str,
            _resource: Option<&str>,
            _resource_id: Option<&str>,
            _status: &str,
            _details: Option<&str>,
        ) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    #[test]
    fn delete_missing_file_is_idempotent_and_skips_blob_cleanup() {
        let store = fake_store(None);
        let service = DeleteFileService::new(
            store,
            FakeRepository {
                deleted: None,
                error: None,
            },
        );

        let result = service
            .delete(&file_id("file-1"))
            .unwrap_or_else(|error| panic!("delete failed: {error}"));

        assert!(!result.existed);
        assert!(!result.blob_cleaned);
        assert!(service.blob_store.cleanup_calls.borrow().is_empty());
    }

    #[test]
    fn delete_shared_blob_keeps_storage_until_last_reference() {
        let store = fake_store(None);
        let service = DeleteFileService::new(
            store,
            FakeRepository {
                deleted: Some(deleted_record(1)),
                error: None,
            },
        );

        let result = service
            .delete(&file_id("file-1"))
            .unwrap_or_else(|error| panic!("delete failed: {error}"));

        assert!(result.existed);
        assert!(!result.blob_cleaned);
        assert!(service.blob_store.cleanup_calls.borrow().is_empty());
    }

    #[test]
    fn delete_last_reference_cleans_blob() {
        let store = fake_store(None);
        let expected_handle = storage_handle();
        let service = DeleteFileService::new(
            store,
            FakeRepository {
                deleted: Some(deleted_record(0)),
                error: None,
            },
        );

        let result = service
            .delete(&file_id("file-1"))
            .unwrap_or_else(|error| panic!("delete failed: {error}"));

        assert!(result.existed);
        assert!(result.blob_cleaned);
        assert_eq!(
            service.blob_store.cleanup_calls.borrow().as_slice(),
            &[expected_handle]
        );
    }

    #[test]
    fn delete_reports_repository_failure() {
        let service = DeleteFileService::new(
            fake_store(None),
            FakeRepository {
                deleted: None,
                error: Some(RepositoryError::Busy),
            },
        );

        let result = service.delete(&file_id("file-1"));

        assert!(matches!(
            result,
            Err(DeleteFileError::Repository(RepositoryError::Busy))
        ));
    }

    #[test]
    fn delete_reports_cleanup_failure_after_metadata_delete() {
        let service = DeleteFileService::new(
            fake_store(Some(BlobStoreError::CleanupFailed {
                handle: storage_handle(),
                message: "permission denied".to_owned(),
            })),
            FakeRepository {
                deleted: Some(deleted_record(0)),
                error: None,
            },
        );

        let result = service.delete(&file_id("file-1"));

        assert!(matches!(result, Err(DeleteFileError::BlobCleanup(_))));
    }

    fn fake_store(cleanup_error: Option<BlobStoreError>) -> FakeBlobStore {
        FakeBlobStore {
            cleanup_calls: RefCell::new(Vec::new()),
            cleanup_error,
        }
    }

    fn deleted_record(remaining_content_references: u64) -> DeletedFileRecord {
        DeletedFileRecord {
            record: file_record(),
            remaining_content_references,
        }
    }

    fn file_record() -> FileRecord {
        FileRecord {
            id: file_id("file-1"),
            name: filename("note.txt"),
            size: FileSize::new(5),
            content_hash: content_hash(),
            mime_type: mime_type("text/plain"),
            storage_handle: storage_handle(),
            uploaded_at: timestamp(1_700_000_000),
            tags: Vec::new(),
            pinned_at: None,
            folder_path: String::new(),
            owner_id: None,
            visibility: tssp_domain::Visibility::Private,
            public_token: None,
        }
    }

    fn clone_blob_error(error: &BlobStoreError) -> BlobStoreError {
        match error {
            BlobStoreError::InsufficientStorage {
                required_bytes,
                available_bytes,
            } => BlobStoreError::InsufficientStorage {
                required_bytes: *required_bytes,
                available_bytes: *available_bytes,
            },
            BlobStoreError::ReadFailed { message } => BlobStoreError::ReadFailed {
                message: message.clone(),
            },
            BlobStoreError::WriteFailed { message } => BlobStoreError::WriteFailed {
                message: message.clone(),
            },
            BlobStoreError::CleanupFailed { handle, message } => BlobStoreError::CleanupFailed {
                handle: handle.clone(),
                message: message.clone(),
            },
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

    fn file_id(value: &str) -> FileId {
        FileId::new(value).unwrap_or_else(|error| panic!("invalid file id: {error}"))
    }

    fn filename(value: &str) -> FileName {
        FileName::new(value).unwrap_or_else(|error| panic!("invalid filename: {error}"))
    }

    fn content_hash() -> ContentHash {
        ContentHash::new("abcdefabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123")
            .unwrap_or_else(|error| panic!("invalid hash: {error}"))
    }

    fn mime_type(value: &str) -> MimeType {
        MimeType::new(value).unwrap_or_else(|error| panic!("invalid MIME type: {error}"))
    }

    fn storage_handle() -> StorageHandle {
        StorageHandle::new("blobs/ab/cd/abcdef")
            .unwrap_or_else(|error| panic!("invalid storage handle: {error}"))
    }

    fn timestamp(seconds: i64) -> UnixTimestamp {
        UnixTimestamp::new(seconds).unwrap_or_else(|error| panic!("invalid timestamp: {error}"))
    }
}
