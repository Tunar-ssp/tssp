//! Tag use case orchestration.

use thiserror::Error;
use tssp_domain::{DomainError, FileId, Tag, TagKey};
use tssp_ports::{FileRepository, RepositoryError, TagMutationOutcome, TagSummary};

/// Coordinates tag listing and idempotent file tag mutations.
pub struct TagService<R> {
    repository: R,
}

impl<R> TagService<R> {
    /// Creates a tag service from an explicit metadata repository port.
    #[must_use]
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> TagService<R>
where
    R: FileRepository,
{
    /// Lists all tags with file counts.
    ///
    /// # Errors
    ///
    /// Returns [`TagError`] when metadata listing fails.
    pub fn list_tags(&self) -> Result<Vec<TagSummary>, TagError> {
        self.repository.list_tags().map_err(TagError::Repository)
    }

    /// Adds tags to a file idempotently.
    ///
    /// # Errors
    ///
    /// Returns [`TagError`] when a tag is invalid or metadata mutation fails.
    pub fn add_tags(&self, id: &FileId, tags: &[&str]) -> Result<TagMutationOutcome, TagError> {
        let tags = normalize_tags(tags)?;
        self.repository
            .add_tags_to_file(id, &tags)
            .map_err(TagError::Repository)
    }

    /// Removes one tag from a file idempotently.
    ///
    /// # Errors
    ///
    /// Returns [`TagError`] when the tag is invalid or metadata mutation fails.
    pub fn remove_tag(&self, id: &FileId, tag: &str) -> Result<TagMutationOutcome, TagError> {
        let key = TagKey::new(tag)?;
        self.repository
            .remove_tag_from_file(id, &key)
            .map_err(TagError::Repository)
    }
}

/// Tag use-case failure.
#[derive(Debug, Error)]
pub enum TagError {
    /// Invalid tag text.
    #[error(transparent)]
    InvalidRequest(#[from] DomainError),

    /// Metadata repository failure.
    #[error(transparent)]
    Repository(RepositoryError),
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

    use tssp_domain::{ContentHash, FileId, FileName, FileRecord, Tag, TagKey, UnixTimestamp};
    use tssp_ports::{
        DeletedFileRecord, FileRepository, NewFileRecord, PinOutcome, RepositoryError,
        RepositoryStats, TagMutationOutcome, TagSummary,
    };

    use super::{TagError, TagService};

    struct FakeRepository {
        tags: RefCell<Vec<TagSummary>>,
        failure: Option<RepositoryError>,
    }

    impl FileRepository for FakeRepository {
        fn insert_file(&self, _new_file: NewFileRecord) -> Result<FileRecord, RepositoryError> {
            Err(RepositoryError::OperationFailed {
                message: "not used by tag tests".to_owned(),
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
            match &self.failure {
                Some(error) => Err(clone_repository_error(error)),
                None => Ok(self.tags.borrow().clone()),
            }
        }

        fn add_tags_to_file(
            &self,
            _id: &FileId,
            tags: &[Tag],
        ) -> Result<TagMutationOutcome, RepositoryError> {
            if let Some(error) = &self.failure {
                return Err(clone_repository_error(error));
            }
            Ok(TagMutationOutcome {
                changed_count: u64::try_from(tags.len()).unwrap_or(u64::MAX),
            })
        }

        fn remove_tag_from_file(
            &self,
            _id: &FileId,
            _tag: &TagKey,
        ) -> Result<TagMutationOutcome, RepositoryError> {
            if let Some(error) = &self.failure {
                return Err(clone_repository_error(error));
            }
            Ok(TagMutationOutcome { changed_count: 1 })
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

        fn list_folder_counts(&self) -> Result<Vec<(String, u64)>, RepositoryError> {
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

    #[test]
    fn list_tags_returns_repository_summaries() {
        let service = TagService::new(FakeRepository {
            tags: RefCell::new(vec![TagSummary {
                tag: tag_value("Docs"),
                file_count: 2,
            }]),
            failure: None,
        });

        let tags = service
            .list_tags()
            .unwrap_or_else(|error| panic!("list tags failed: {error}"));

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].file_count, 2);
    }

    #[test]
    fn add_tags_normalizes_and_deduplicates_input() {
        let service = TagService::new(FakeRepository {
            tags: RefCell::new(Vec::new()),
            failure: None,
        });

        let outcome = service
            .add_tags(&file_id("file-1"), &["Docs", " docs "])
            .unwrap_or_else(|error| panic!("add tags failed: {error}"));

        assert_eq!(outcome.changed_count, 1);
    }

    #[test]
    fn tag_service_rejects_invalid_tag_text() {
        let service = TagService::new(FakeRepository {
            tags: RefCell::new(Vec::new()),
            failure: None,
        });

        let add = service.add_tags(&file_id("file-1"), &["bad/tag"]);
        let remove = service.remove_tag(&file_id("file-1"), "bad/tag");

        assert!(matches!(add, Err(TagError::InvalidRequest(_))));
        assert!(matches!(remove, Err(TagError::InvalidRequest(_))));
    }

    #[test]
    fn tag_service_preserves_repository_errors() {
        let service = TagService::new(FakeRepository {
            tags: RefCell::new(Vec::new()),
            failure: Some(RepositoryError::NotFound),
        });

        let add = service.add_tags(&file_id("file-1"), &["Docs"]);
        let remove = service.remove_tag(&file_id("file-1"), "Docs");

        assert!(matches!(
            add,
            Err(TagError::Repository(RepositoryError::NotFound))
        ));
        assert!(matches!(
            remove,
            Err(TagError::Repository(RepositoryError::NotFound))
        ));
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

    fn tag_value(value: &str) -> Tag {
        Tag::new(value).unwrap_or_else(|error| panic!("invalid tag: {error}"))
    }
}
