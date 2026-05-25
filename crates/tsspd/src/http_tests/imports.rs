//! Shared imports for HTTP integration test modules.

pub use std::sync::Arc;

pub use axum::body::{to_bytes, Body};
pub use axum::http::header::CONTENT_TYPE;
pub use axum::http::{Request, StatusCode};
pub use tempfile::tempdir;
pub use tower::ServiceExt;
pub use tssp_adapter_fs::FilesystemBlobStore;
pub use tssp_adapter_sqlite::SqliteFileRepository;
pub use tssp_adapter_system::{SystemClock, UuidV7FileIdGenerator};
pub use tssp_app::{DeleteFileService, PinService, TagService, UploadService};

pub use crate::{
    bind_error_message, build_router, ApplicationFileDeleteProvider, ApplicationFilePinProvider,
    ApplicationFileTagProvider, ApplicationFileUploadProvider, DaemonSettings, HttpState,
    RepositoryMetadataStatsProvider,
};
