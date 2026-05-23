//! Health and metadata status delivery.

use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use tssp_domain::{FileId, FileRecord, UnixTimestamp};
use tssp_ports::{Clock, FileRepository, ListQuery, PagedFiles, RepositoryStats};

use crate::{ErrorBody, ErrorResponse, HttpState};

/// Supplies metadata counts to the status endpoint.
pub trait MetadataStatsProvider: Send + Sync {
    /// Returns the latest metadata stats.
    ///
    /// # Errors
    ///
    /// Returns a short diagnostic when counts cannot be read.
    fn stats(&self) -> Result<RepositoryStats, String>;

    /// Returns filtered and paginated files.
    ///
    /// # Errors
    ///
    /// Returns a short diagnostic when files cannot be listed.
    fn list_files(&self, query: &ListQuery) -> Result<PagedFiles, String>;

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

    /// Returns recent files filtered by a tag.
    ///
    /// # Errors
    ///
    /// Returns a short diagnostic when files cannot be listed.
    fn list_files_by_tag(
        &self,
        tag: &tssp_domain::TagKey,
        limit: u64,
    ) -> Result<Vec<FileRecord>, String>;

    /// Renames a file.
    ///
    /// # Errors
    ///
    /// Returns a short diagnostic when the rename fails.
    fn rename_file(
        &self,
        id: &FileId,
        new_name: &tssp_domain::FileName,
    ) -> Result<Option<FileRecord>, String>;
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

    fn list_files(&self, query: &ListQuery) -> Result<PagedFiles, String> {
        Ok(PagedFiles {
            files: self.list_files_recent(query.limit)?,
            next_cursor: None,
        })
    }

    fn find_file(&self, _id: &FileId) -> Result<Option<FileRecord>, String> {
        Ok(None)
    }

    fn list_files_by_tag(
        &self,
        _tag: &tssp_domain::TagKey,
        _limit: u64,
    ) -> Result<Vec<FileRecord>, String> {
        Ok(Vec::new())
    }

    fn rename_file(
        &self,
        _id: &FileId,
        _new_name: &tssp_domain::FileName,
    ) -> Result<Option<FileRecord>, String> {
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

    fn list_files(&self, query: &ListQuery) -> Result<PagedFiles, String> {
        self.repository
            .list_files(query)
            .map_err(|error| error.to_string())
    }

    fn find_file(&self, id: &FileId) -> Result<Option<FileRecord>, String> {
        self.repository
            .find_file(id)
            .map_err(|error| error.to_string())
    }

    fn list_files_by_tag(
        &self,
        tag: &tssp_domain::TagKey,
        limit: u64,
    ) -> Result<Vec<FileRecord>, String> {
        self.repository
            .list_files_by_tag(tag, limit)
            .map_err(|error| error.to_string())
    }

    fn rename_file(
        &self,
        id: &FileId,
        new_name: &tssp_domain::FileName,
    ) -> Result<Option<FileRecord>, String> {
        self.repository
            .rename_file(id, new_name)
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

#[cfg(test)]
mod tests {
    use super::{healthz, readyz, status, MetadataStatsProvider, StaticMetadataStatsProvider};
    use crate::HttpState;
    use axum::body::to_bytes;
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use std::sync::Arc;
    use tssp_domain::{FileId, FileRecord};
    use tssp_ports::{ListQuery, PagedFiles, RepositoryStats};

    #[tokio::test]
    async fn healthz_returns_ok() {
        let response = healthz().await.into_response();
        let body = to_bytes(response.into_body(), 1024)
            .await
            .unwrap_or_else(|error| panic!("body read failed: {error}"));
        assert_eq!(&body[..], b"ok");
    }

    #[tokio::test]
    async fn readyz_returns_ready() {
        let response = readyz().await.into_response();
        let body = to_bytes(response.into_body(), 1024)
            .await
            .unwrap_or_else(|error| panic!("body read failed: {error}"));
        assert_eq!(&body[..], b"ready");
    }

    #[test]
    fn static_provider_returns_empty_stats() {
        let provider = StaticMetadataStatsProvider;
        let stats = provider
            .stats()
            .unwrap_or_else(|error| panic!("stats failed: {error}"));
        assert_eq!(stats.file_count, 0);
        assert_eq!(stats.pinned_count, 0);

        let files = provider
            .list_files_recent(10)
            .unwrap_or_else(|error| panic!("list recent failed: {error}"));
        assert!(files.is_empty());

        let id = FileId::new("file-1").unwrap_or_else(|error| panic!("id parse failed: {error}"));
        assert!(provider
            .find_file(&id)
            .unwrap_or_else(|error| panic!("find file failed: {error}"))
            .is_none());

        let tag = tssp_domain::TagKey::new("docs")
            .unwrap_or_else(|error| panic!("tag parse failed: {error}"));
        assert!(provider
            .list_files_by_tag(&tag, 10)
            .unwrap_or_else(|error| panic!("list by tag failed: {error}"))
            .is_empty());

        let name = tssp_domain::FileName::new("newname.txt")
            .unwrap_or_else(|error| panic!("filename parse failed: {error}"));
        assert!(provider
            .rename_file(&id, &name)
            .unwrap_or_else(|error| panic!("rename failed: {error}"))
            .is_none());
    }

    #[tokio::test]
    async fn status_endpoint_returns_ok_with_stats() {
        let state = HttpState::new(std::time::Instant::now(), std::path::PathBuf::from("/tmp"))
            .with_stats_provider(Arc::new(StaticMetadataStatsProvider));

        let response = status(State(state)).await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn status_endpoint_returns_unavailable_on_stats_error() {
        struct ErrorStatsProvider;

        impl MetadataStatsProvider for ErrorStatsProvider {
            fn stats(&self) -> Result<RepositoryStats, String> {
                Err("database error".to_owned())
            }

            fn list_files_recent(&self, _limit: u64) -> Result<Vec<FileRecord>, String> {
                Ok(vec![])
            }

            fn list_files(&self, _query: &ListQuery) -> Result<PagedFiles, String> {
                Ok(PagedFiles {
                    files: vec![],
                    next_cursor: None,
                })
            }

            fn find_file(&self, _id: &FileId) -> Result<Option<FileRecord>, String> {
                Ok(None)
            }

            fn list_files_by_tag(
                &self,
                _tag: &tssp_domain::TagKey,
                _limit: u64,
            ) -> Result<Vec<FileRecord>, String> {
                Ok(vec![])
            }

            fn rename_file(
                &self,
                _id: &FileId,
                _new_name: &tssp_domain::FileName,
            ) -> Result<Option<FileRecord>, String> {
                Ok(None)
            }
        }

        let state = HttpState::new(std::time::Instant::now(), std::path::PathBuf::from("/tmp"))
            .with_stats_provider(Arc::new(ErrorStatsProvider));

        let response = status(State(state)).await;

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn static_provider_list_files_returns_empty() {
        use tssp_ports::ListQuery as RepositoryListQuery;
        let provider = StaticMetadataStatsProvider;
        let query = RepositoryListQuery {
            limit: 10,
            tags: vec![],
            mime_prefix: None,
            name_substring: None,
            since: None,
            until: None,
            pinned_only: false,
            sort: Default::default(),
            after_cursor: None,
        };
        let result = provider
            .list_files(&query)
            .unwrap_or_else(|error| panic!("list failed: {error}"));
        assert!(result.files.is_empty());
        assert!(result.next_cursor.is_none());
    }

    #[test]
    fn static_provider_find_file_returns_none() {
        let provider = StaticMetadataStatsProvider;
        let id = FileId::new("test-file").expect("create file id failed");
        let result = provider
            .find_file(&id)
            .unwrap_or_else(|error| panic!("find failed: {error}"));
        assert!(result.is_none());
    }

    #[test]
    fn static_provider_list_files_by_tag_returns_empty() {
        let provider = StaticMetadataStatsProvider;
        let tag = tssp_domain::TagKey::new("test-tag").expect("create tag failed");
        let result = provider
            .list_files_by_tag(&tag, 10)
            .unwrap_or_else(|error| panic!("list by tag failed: {error}"));
        assert!(result.is_empty());
    }

    #[test]
    fn static_provider_rename_file_returns_none() {
        let provider = StaticMetadataStatsProvider;
        let id = FileId::new("test-file").expect("create file id failed");
        let name = tssp_domain::FileName::new("new-name.txt").expect("create name failed");
        let result = provider
            .rename_file(&id, &name)
            .unwrap_or_else(|error| panic!("rename failed: {error}"));
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn status_response_includes_uptime() {
        let state = HttpState::new(std::time::Instant::now(), std::path::PathBuf::from("/tmp"))
            .with_stats_provider(Arc::new(StaticMetadataStatsProvider));
        let response = status(State(state)).await;

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 4096)
            .await
            .unwrap_or_else(|error| panic!("body read failed: {error}"));
        let json: serde_json::Value = serde_json::from_slice(&body)
            .unwrap_or_else(|error| panic!("json parse failed: {error}"));

        assert!(json.get("uptime_seconds").is_some());
        assert!(json.get("file_count").is_some());
    }
}
