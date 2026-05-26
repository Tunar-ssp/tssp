//! HTTP daemon foundation for `tsspd`.
//!
//! The current server exposes lifecycle, status, upload, and web shell routes.
//! HTTP handlers stay thin and delegate storage behavior to application
//! services.

mod admin;
pub mod auth;
mod chunked_upload;
#[cfg(test)]
mod concurrency_tests;
mod config;
mod content;
mod delete;
mod edge_cases_tests;
pub mod error_handler;
mod error_handling_tests;
mod file;
pub mod folders;
mod garbage_collection;
mod http_error;
mod integrity;
mod list;
mod mdns;
mod metrics;
mod move_file;
mod notes;
#[cfg(test)]
mod performance_tests;
mod pins;
mod public_api;
#[cfg(test)]
mod public_link_tests;
mod public_sessions;
mod qr;
mod rate_limit;
mod rename;
mod router;
mod search;
mod sessions;
mod settings;
mod share;
mod startup;
mod state;
mod stats_cache;
mod status;
mod tags;
pub mod temp_cleanup;
pub mod trash_cleanup;
mod upload;
mod urls;
pub mod validators;
mod visibility;
mod web;
pub mod workspaces;
mod workspace_fs;
mod workspace_features;

#[cfg(test)]
mod http_tests;

pub use config::bind_error_message;
pub use delete::{
    ApplicationFileDeleteProvider, ApplicationFileRestoreProvider, FileDeleteProvider,
    FileRestoreProvider, HttpDeleteError, HttpDeleteOutcome, HttpRestoreError, HttpRestoreOutcome,
};
pub use folders::{ApplicationFolderProvider, FolderProvider, HttpFolderError};
pub use garbage_collection::collect_garbage;
pub use integrity::{run_startup_integrity_scan, spawn_startup_integrity_scan};
pub use mdns::spawn_advertisement;
pub use notes::{ApplicationNoteProvider, NoteProvider};
pub use pins::{ApplicationFilePinProvider, FilePinProvider, HttpPinError, HttpPinMutation};
pub use router::build_router;
pub use search::{FileSearchProvider, RepositoryFileSearchProvider};
pub use sessions::{ApplicationSessionProvider, SessionProvider, SessionResponse};
pub use settings::{CliOverrides, DaemonSettings};
pub use startup::StartupService;
pub use state::HttpState;
pub use status::{MetadataStatsProvider, RepositoryMetadataStatsProvider, StatusResponse};
pub use tags::{ApplicationFileTagProvider, FileTagProvider, HttpTagError, HttpTagMutation};
pub use upload::{
    ApplicationFileUploadProvider, FileRecordResponse, FileUploadProvider, HttpUploadError,
    HttpUploadOutcome, HttpUploadRequest,
};
pub use urls::PublicUrlBuilder;

pub(crate) use http_error::{ErrorBody, ErrorResponse};
