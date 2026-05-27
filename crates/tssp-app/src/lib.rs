//! Application services for backend use cases.
//!
//! Services in this crate orchestrate domain values and ports. They own use-case
//! ordering and cleanup rules but do not know whether persistence is `SQLite`,
//! files are local, or delivery is HTTP.

pub mod audit;
mod delete;
mod folders;
/// Git service.
pub mod git;
/// LSP service.
pub mod lsp;
mod notes;
mod pins;
mod sessions;
mod tags;
/// Terminal service.
pub mod terminal;
#[cfg(test)]
mod terminal_tests;
#[cfg(test)]
mod git_tests;
mod upload;
pub mod workspace_files;

pub use audit::{log_audit_event, AuditAction};
pub use delete::{
    DeleteFileError, DeleteFileResult, DeleteFileService, PurgeDeletedFilesService, PurgeError,
    RestoreFileError, RestoreFileService,
};
pub use folders::{normalize_folder_path, validate_folder_path, FolderError, FolderService};
pub use git::GitService;
pub use lsp::LspService;
pub use notes::{CreateNoteRequest, NoteError, NoteService, UpdateNoteRequest};
pub use pins::{PinError, PinService};
pub use sessions::SessionService;
pub use tags::{TagError, TagService};
pub use terminal::TerminalService;
pub use upload::{UploadError, UploadRequest, UploadResult, UploadService};
pub use workspace_files::WorkspaceFileService;
