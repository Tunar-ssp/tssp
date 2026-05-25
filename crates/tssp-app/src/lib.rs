//! Application services for backend use cases.
//!
//! Services in this crate orchestrate domain values and ports. They own use-case
//! ordering and cleanup rules but do not know whether persistence is `SQLite`,
//! files are local, or delivery is HTTP.

pub mod audit;
mod delete;
mod folders;
mod notes;
mod pins;
mod sessions;
mod tags;
mod upload;

pub use audit::{log_audit_event, AuditAction};
pub use delete::{
    DeleteFileError, DeleteFileResult, DeleteFileService, PurgeDeletedFilesService, PurgeError,
    RestoreFileError, RestoreFileService,
};
pub use folders::{normalize_folder_path, validate_folder_path, FolderError, FolderService};
pub use notes::{CreateNoteRequest, NoteError, NoteService, UpdateNoteRequest};
pub use pins::{PinError, PinService};
pub use sessions::SessionService;
pub use tags::{TagError, TagService};
pub use upload::{UploadError, UploadRequest, UploadResult, UploadService};
