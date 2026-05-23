//! Health and metadata status delivery.

use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use tssp_domain::{FileId, FileRecord, UnixTimestamp};
use tssp_ports::{Clock, FileRepository, RepositoryStats};

use crate::{ErrorBody, ErrorResponse, HttpState};

/// Supplies metadata counts to the status endpoint.
pub trait MetadataStatsProvider: Send + Sync {
    /// Returns the latest metadata stats.
    ///
    /// # Errors
    ///
    /// Returns a short diagnostic when counts cannot be read.
    fn stats(&self) -> Result<RepositoryStats, String>;

    /// Returns recent files.
    ///
    /// # Errors
    ///
    /// Returns a short diagnostic when files cannot be listed.
    fn list_files_recent(&self, limit: u64) -> Result<Vec<FileRecord>, String>;

    /// Returns one file by id.
    ///
    /// # Errors
    ///
    /// Returns a short diagnostic when metadata lookup fails.
    fn find_file(&self, id: &FileId) -> Result<Option<FileRecord>, String>;
}

#[derive(Debug)]
pub(crate) struct StaticMetadataStatsProvider;

impl MetadataStatsProvider for StaticMetadataStatsProvider {
    fn stats(&self) -> Result<RepositoryStats, String> {
        Ok(RepositoryStats {
            file_count: 0,
            tag_count: 0,
            pinned_count: 0,
            recent_upload_count: 0,
        })
    }

    fn list_files_recent(&self, _limit: u64) -> Result<Vec<FileRecord>, String> {
        Ok(Vec::new())
    }

    fn find_file(&self, _id: &FileId) -> Result<Option<FileRecord>, String> {
        Ok(None)
    }
}

/// Metadata stats provider backed by a repository and clock.
#[derive(Debug)]
pub struct RepositoryMetadataStatsProvider<R, C> {
    repository: R,
    clock: C,
}

impl<R, C> RepositoryMetadataStatsProvider<R, C> {
    /// Creates a repository-backed stats provider.
    #[must_use]
    pub const fn new(repository: R, clock: C) -> Self {
        Self { repository, clock }
    }
}

impl<R, C> MetadataStatsProvider for RepositoryMetadataStatsProvider<R, C>
where
    R: FileRepository + Send + Sync,
    C: Clock + Send + Sync,
{
    fn stats(&self) -> Result<RepositoryStats, String> {
        let now = self.clock.now();
        let cutoff = now.seconds().saturating_sub(86_400);
        let recent_since = UnixTimestamp::new(cutoff).map_err(|error| error.to_string())?;
        self.repository
            .stats_since(recent_since)
            .map_err(|error| error.to_string())
    }

    fn list_files_recent(&self, limit: u64) -> Result<Vec<FileRecord>, String> {
        self.repository
            .list_files_recent(limit)
            .map_err(|error| error.to_string())
    }

    fn find_file(&self, id: &FileId) -> Result<Option<FileRecord>, String> {
        self.repository
            .find_file(id)
            .map_err(|error| error.to_string())
    }
}

/// Minimal status response consumed by `tssp status`.
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    /// Stable response schema version.
    pub schema_version: u8,
    /// Daemon version.
    pub version: &'static str,
    /// Human-readable health state.
    pub status: &'static str,
    /// Seconds since process startup.
    pub uptime_seconds: u64,
    /// Indexed file count.
    pub file_count: u64,
    /// Known tag count.
    pub tag_count: u64,
    /// Pinned file count.
    pub pinned_count: u64,
    /// Uploads in the last 24 hours.
    pub recent_upload_count_24h: u64,
}

pub(crate) async fn healthz() -> impl IntoResponse {
    ([(CONTENT_TYPE, "text/plain; charset=utf-8")], "ok")
}

pub(crate) async fn readyz() -> impl IntoResponse {
    ([(CONTENT_TYPE, "text/plain; charset=utf-8")], "ready")
}

pub(crate) async fn status(State(state): State<HttpState>) -> Response {
    match state.stats_provider.stats() {
        Ok(repository_stats) => Json(StatusResponse {
            schema_version: 1,
            version: env!("CARGO_PKG_VERSION"),
            status: "ok",
            uptime_seconds: state.started_at.elapsed().as_secs(),
            file_count: repository_stats.file_count,
            tag_count: repository_stats.tag_count,
            pinned_count: repository_stats.pinned_count,
            recent_upload_count_24h: repository_stats.recent_upload_count,
        })
        .into_response(),
        Err(message) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "metadata_unavailable",
                    message,
                },
            }),
        )
            .into_response(),
    }
}
