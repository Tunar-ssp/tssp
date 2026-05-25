//! Pin use case orchestration.

use thiserror::Error;
use tssp_domain::{DomainError, FileId, FileRecord};
use tssp_ports::{FileRepository, PinOutcome, RepositoryError};

/// Coordinates pin listing, pin/unpin mutations, and reordering.
pub struct PinService<R> {
    repository: R,
}

impl<R> PinService<R> {
    /// Creates a pin service from an explicit metadata repository port.
    #[must_use]
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> PinService<R>
where
    R: FileRepository,
{
    /// Lists pinned files in pin order.
    ///
    /// # Errors
    ///
    /// Returns [`PinError`] when metadata listing fails.
    pub fn list_pins(&self) -> Result<Vec<FileRecord>, PinError> {
        self.repository
            .list_pinned_files()
            .map_err(PinError::Repository)
    }

    /// Pins a file, optionally at a specific position.
    ///
    /// # Errors
    ///
    /// Returns [`PinError`] when the file id is invalid or metadata mutation fails.
    pub fn pin(&self, id: &str, position: Option<u32>) -> Result<PinOutcome, PinError> {
        let file_id = FileId::new(id)?;
        self.repository
            .pin_file(&file_id, position)
            .map_err(PinError::Repository)
    }

    /// Unpins a file.
    ///
    /// # Errors
    ///
    /// Returns [`PinError`] when the file id is invalid or metadata mutation fails.
    pub fn unpin(&self, id: &str) -> Result<PinOutcome, PinError> {
        let file_id = FileId::new(id)?;
        self.repository
            .unpin_file(&file_id)
            .map_err(PinError::Repository)
    }

    /// Reorders pins according to the provided list of file IDs.
    ///
    /// # Errors
    ///
    /// Returns [`PinError`] when any id is invalid or the reorder fails.
    pub fn reorder(&self, ids: &[&str]) -> Result<(), PinError> {
        let file_ids: Vec<FileId> = ids.iter().map(FileId::new).collect::<Result<Vec<_>, _>>()?;
        self.repository
            .reorder_pins(&file_ids)
            .map_err(PinError::Repository)
    }
}

/// Pin use-case failure.
#[derive(Debug, Error)]
pub enum PinError {
    /// Invalid file id text.
    #[error(transparent)]
    InvalidRequest(#[from] DomainError),

    /// Metadata repository failure.
    #[error(transparent)]
    Repository(RepositoryError),
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use tssp_domain::{
        ContentHash, FileId, FileName, FileRecord, FileSize, MimeType, StorageHandle, Tag, TagKey,
        UnixTimestamp,
    };
    use tssp_ports::{
        DeletedFileRecord, FileRepository, NewFileRecord, PinOutcome, RepositoryError,
        RepositoryStats, TagMutationOutcome, TagSummary,
    };

    use super::{PinError, PinService};

    struct FakeRepository {
        pinned: RefCell<Vec<FileRecord>>,
        failure: Option<RepositoryError>,
    }

    impl FileRepository for FakeRepository {
        fn insert_file(&self, _new_file: NewFileRecord) -> Result<FileRecord, RepositoryError> {
            Err(RepositoryError::OperationFailed {
                message: "not used by pin tests".to_owned(),
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
            if let Some(error) = &self.failure {
                return Err(clone_repository_error(error));
            }
            Ok(PinOutcome {
                existed: true,
                changed: true,
            })
        }

        fn unpin_file(&self, _id: &FileId) -> Result<PinOutcome, RepositoryError> {
            if let Some(error) = &self.failure {
                return Err(clone_repository_error(error));
            }
            Ok(PinOutcome {
                existed: true,
                changed: true,
            })
        }

        fn list_pinned_files(&self) -> Result<Vec<FileRecord>, RepositoryError> {
            if let Some(error) = &self.failure {
                return Err(clone_repository_error(error));
            }
            Ok(self.pinned.borrow().clone())
        }

        fn reorder_pins(&self, _ordered_ids: &[FileId]) -> Result<(), RepositoryError> {
            if let Some(error) = &self.failure {
                return Err(clone_repository_error(error));
            }
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
    fn list_pins_returns_repository_results() {
        let service = PinService::new(FakeRepository {
            pinned: RefCell::new(vec![test_record("pinned-1")]),
            failure: None,
        });

        let pins = service
            .list_pins()
            .unwrap_or_else(|error| panic!("list pins failed: {error}"));

        assert_eq!(pins.len(), 1);
        assert_eq!(pins[0].id.as_str(), "pinned-1");
    }

    #[test]
    fn pin_file_delegates_to_repository() {
        let service = PinService::new(FakeRepository {
            pinned: RefCell::new(Vec::new()),
            failure: None,
        });

        let outcome = service
            .pin("file-1", None)
            .unwrap_or_else(|error| panic!("pin failed: {error}"));

        assert!(outcome.changed);
        assert!(outcome.existed);
    }

    #[test]
    fn unpin_file_delegates_to_repository() {
        let service = PinService::new(FakeRepository {
            pinned: RefCell::new(Vec::new()),
            failure: None,
        });

        let outcome = service
            .unpin("file-1")
            .unwrap_or_else(|error| panic!("unpin failed: {error}"));

        assert!(outcome.changed);
    }

    #[test]
    fn pin_service_rejects_invalid_file_id() {
        let service = PinService::new(FakeRepository {
            pinned: RefCell::new(Vec::new()),
            failure: None,
        });

        let pin_result = service.pin("", None);
        let unpin_result = service.unpin("");

        assert!(matches!(pin_result, Err(PinError::InvalidRequest(_))));
        assert!(matches!(unpin_result, Err(PinError::InvalidRequest(_))));
    }

    #[test]
    fn pin_service_preserves_repository_errors() {
        let service = PinService::new(FakeRepository {
            pinned: RefCell::new(Vec::new()),
            failure: Some(RepositoryError::NotFound),
        });

        let result = service.pin("file-1", None);

        assert!(matches!(
            result,
            Err(PinError::Repository(RepositoryError::NotFound))
        ));
    }

    #[test]
    fn reorder_delegates_to_repository() {
        let service = PinService::new(FakeRepository {
            pinned: RefCell::new(Vec::new()),
            failure: None,
        });

        let result = service.reorder(&["file-1", "file-2"]);

        assert!(result.is_ok());
    }

    #[test]
    fn reorder_rejects_invalid_ids() {
        let service = PinService::new(FakeRepository {
            pinned: RefCell::new(Vec::new()),
            failure: None,
        });

        let result = service.reorder(&[""]);

        assert!(matches!(result, Err(PinError::InvalidRequest(_))));
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

    fn test_record(id: &str) -> FileRecord {
        FileRecord {
            id: FileId::new(id).unwrap_or_else(|error| panic!("invalid file id: {error}")),
            name: FileName::new("note.txt")
                .unwrap_or_else(|error| panic!("invalid filename: {error}")),
            size: FileSize::new(5),
            content_hash: ContentHash::new(
                "abcdefabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123",
            )
            .unwrap_or_else(|error| panic!("invalid hash: {error}")),
            mime_type: MimeType::new("text/plain")
                .unwrap_or_else(|error| panic!("invalid mime: {error}")),
            storage_handle: StorageHandle::new("blobs/ab/cd/abcdef")
                .unwrap_or_else(|error| panic!("invalid handle: {error}")),
            uploaded_at: UnixTimestamp::new(1_700_000_000)
                .unwrap_or_else(|error| panic!("invalid timestamp: {error}")),
            tags: Vec::new(),
            pinned_at: Some(1),
            folder_path: String::new(),
            owner_id: None,
            visibility: tssp_domain::Visibility::Private,
            public_token: None,
        }
    }
}
