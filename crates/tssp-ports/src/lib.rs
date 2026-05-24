//! Boundary traits between application services and infrastructure.
//!
//! Implementations live in adapter crates. Keeping these traits small and typed
//! lets tests replace `SQLite`, filesystem storage, clocks, and ID generation with
//! deterministic in-memory doubles.
//!
//! # Module layout
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`blob`] | [`BlobStore`], [`BlobReader`] port traits |
//! | [`clock`] | [`Clock`] trait |
//! | [`errors`] | Error types returned by all port traits |
//! | [`id`] | [`IdGenerator`], [`SessionTokenGenerator`] traits |
//! | [`query`] | Query parameters and result types |
//! | [`record`] | Input/output record types for write operations |
//! | [`repository`] | [`FileRepository`], [`NoteRepository`] port traits |
//! | [`session`] | [`SessionRepository`] port trait |

pub mod blob;
pub mod clock;
pub mod errors;
pub mod id;
pub mod query;
pub mod record;
pub mod repository;
pub mod session;

// Flat re-exports for backward compatibility.
pub use blob::{BlobReader, BlobStore};
pub use clock::Clock;
pub use errors::{BlobReadError, BlobStoreError, IdGenerationError, RepositoryError};
pub use id::{IdGenerator, SessionTokenGenerator};
pub use query::{
    ListQuery, ListSort, NoteListQuery, NoteListSort, PagedFiles, PagedNotes, PinOutcome,
    RepositoryStats, SearchHit, TagMutationOutcome, TagSummary,
};
pub use record::{BlobWriteOutcome, DeletedFileRecord, NewFileRecord, NewNoteRecord};
pub use repository::{FileRepository, NoteRepository};
pub use session::SessionRepository;

#[cfg(test)]
mod tests {
    use tssp_domain::{
        ContentHash, FileId, FileRecord, FileSize, FileName, MimeType, StorageHandle,
        UnixTimestamp,
    };

    use super::*;

    #[test]
    fn repository_error_display_messages() {
        assert_eq!(RepositoryError::Busy.to_string(), "metadata store is busy");
        assert_eq!(
            RepositoryError::NotFound.to_string(),
            "metadata record was not found"
        );
        assert_eq!(
            RepositoryError::Conflict {
                message: "dup".to_owned()
            }
            .to_string(),
            "metadata conflict: dup"
        );
        assert_eq!(
            RepositoryError::OperationFailed {
                message: "io fail".to_owned()
            }
            .to_string(),
            "metadata operation failed: io fail"
        );
    }

    #[test]
    fn blob_store_error_display_messages() {
        assert!(BlobStoreError::InsufficientStorage {
            required_bytes: 100,
            available_bytes: 10,
        }
        .to_string()
        .contains("100"));

        assert!(BlobStoreError::ReadFailed {
            message: "broken pipe".to_owned()
        }
        .to_string()
        .contains("broken pipe"));

        assert!(BlobStoreError::WriteFailed {
            message: "disk error".to_owned()
        }
        .to_string()
        .contains("disk error"));
    }

    #[test]
    fn blob_read_error_display_messages() {
        let handle = StorageHandle::new("blobs/ab/cd/test").unwrap_or_else(|e| panic!("{e}"));
        assert!(BlobReadError::Missing { handle }
            .to_string()
            .contains("missing"));

        assert!(BlobReadError::ReadFailed {
            message: "io error".to_owned()
        }
        .to_string()
        .contains("io error"));
    }

    #[test]
    fn id_generation_error_display() {
        let err = IdGenerationError {
            message: "uuid failure".to_owned(),
        };
        assert!(err.to_string().contains("uuid failure"));
    }

    #[test]
    fn repository_stats_is_constructable() {
        let stats = RepositoryStats {
            file_count: 10,
            note_count: 4,
            tag_count: 3,
            pinned_count: 2,
            recent_upload_count: 5,
            recent_note_count: 1,
        };
        assert_eq!(stats.file_count, 10);
        assert_eq!(stats.note_count, 4);
        assert_eq!(stats.pinned_count, 2);
    }

    #[test]
    fn tag_mutation_outcome_tracks_changes() {
        let outcome = TagMutationOutcome { changed_count: 3 };
        assert_eq!(outcome.changed_count, 3);
    }

    #[test]
    fn pin_outcome_tracks_state() {
        let outcome = PinOutcome {
            existed: true,
            changed: false,
        };
        assert!(outcome.existed);
        assert!(!outcome.changed);
    }

    #[test]
    fn deleted_file_record_tracks_references() {
        let id = FileId::new("del-1").unwrap_or_else(|e| panic!("{e}"));
        let name = FileName::new("gone.txt").unwrap_or_else(|e| panic!("{e}"));
        let hash =
            ContentHash::new("abcdefabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123")
                .unwrap_or_else(|e| panic!("{e}"));
        let mime = MimeType::new("text/plain").unwrap_or_else(|e| panic!("{e}"));
        let handle = StorageHandle::new("blobs/ab/cd/test").unwrap_or_else(|e| panic!("{e}"));
        let ts = UnixTimestamp::new(1_700_000_000).unwrap_or_else(|e| panic!("{e}"));

        let deleted = DeletedFileRecord {
            record: FileRecord {
                id,
                name,
                size: FileSize::new(42),
                content_hash: hash,
                mime_type: mime,
                storage_handle: handle,
                uploaded_at: ts,
                tags: vec![],
                pinned_at: None,
                folder_path: String::new(),
                owner_id: None,
                visibility: tssp_domain::Visibility::Private,
                public_token: None,
            },
            remaining_content_references: 0,
        };
        assert_eq!(deleted.remaining_content_references, 0);
        assert_eq!(deleted.record.size.bytes(), 42);
    }
}
