//! HTTP daemon foundation for `tsspd`.
//!
//! The current server exposes lifecycle, status, upload, and web shell routes.
//! HTTP handlers stay thin and delegate storage behavior to application
//! services.

mod admin;
pub mod auth;
mod config;
mod content;
mod delete;
mod file;
mod folders;
mod http_error;
#[allow(dead_code)]
mod integrity;
mod list;
#[allow(dead_code)]
mod mdns;
mod metrics;
mod notes;
mod pins;
mod public_api;
mod public_sessions;
mod qr;
mod rename;
mod router;
mod search;
mod sessions;
mod settings;
mod share;
mod startup;
mod state;
mod status;
mod tags;
mod upload;
mod urls;
mod visibility;
mod web;
pub mod workspaces;

#[cfg(test)]
mod http_tests;

pub use config::bind_error_message;
pub use delete::{
    ApplicationFileDeleteProvider, FileDeleteProvider, HttpDeleteError, HttpDeleteOutcome,
};
pub use integrity::run_startup_integrity_scan;
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
