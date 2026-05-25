//! Application services for backend use cases.
//!
//! Services in this crate orchestrate domain values and ports. They own use-case
//! ordering and cleanup rules but do not know whether persistence is `SQLite`,
//! files are local, or delivery is HTTP.

mod delete;
mod notes;
mod pins;
mod sessions;
mod tags;
mod upload;
pub mod audit;

pub use delete::{
    DeleteFileError, DeleteFileResult, DeleteFileService, RestoreFileError, RestoreFileService,
    PurgeError, PurgeDeletedFilesService,
};
pub use notes::{CreateNoteRequest, NoteError, NoteService, UpdateNoteRequest};
pub use pins::{PinError, PinService};
pub use sessions::SessionService;
pub use tags::{TagError, TagService};
pub use upload::{UploadError, UploadRequest, UploadResult, UploadService};
pub use audit::{AuditAction, log_audit_event};
