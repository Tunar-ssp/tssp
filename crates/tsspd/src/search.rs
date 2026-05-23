//! Full-text search endpoint.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::upload::FileRecordResponse;
use crate::{ErrorBody, ErrorResponse, HttpState};
use tssp_domain::FileRecord;
use tssp_ports::FileRepository;

/// Query parameters for searching files.
#[derive(Debug, Deserialize)]
pub(crate) struct SearchQuery {
    /// The search string.
    pub q: String,
}

/// Response for search endpoint.
#[derive(Debug, Serialize)]
pub(crate) struct SearchResponse {
    /// Stable response schema version.
    pub schema_version: u8,
    /// Array of matching file records.
    pub files: Vec<FileRecordResponse>,
}

/// Provides file search functionality.
pub trait FileSearchProvider: Send + Sync {
    /// Searches files matching the query.
    ///
    /// # Errors
    ///
    /// Returns a short diagnostic when the query fails.
    fn search_files(&self, query: &str) -> Result<Vec<FileRecord>, String>;
}

#[derive(Debug)]
pub(crate) struct StaticFileSearchProvider;

impl FileSearchProvider for StaticFileSearchProvider {
    fn search_files(&self, _query: &str) -> Result<Vec<FileRecord>, String> {
        Ok(Vec::new())
    }
}

/// Search provider backed by a repository.
#[derive(Debug)]
pub struct RepositoryFileSearchProvider<R> {
    repository: R,
}

impl<R> RepositoryFileSearchProvider<R> {
    /// Creates a repository-backed search provider.
    #[must_use]
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> FileSearchProvider for RepositoryFileSearchProvider<R>
where
    R: FileRepository + Send + Sync,
{
    fn search_files(&self, query: &str) -> Result<Vec<FileRecord>, String> {
        self.repository
            .search_files(query)
            .map_err(|error| error.to_string())
    }
}

pub(crate) async fn search_files(
    State(state): State<HttpState>,
    Query(params): Query<SearchQuery>,
) -> Response {
    if params.q.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "invalid_request",
                    message: "query parameter 'q' must not be empty".to_owned(),
                },
            }),
        )
            .into_response();
    }

    let search_provider = state.search_provider.clone();
    let query = params.q;

    match tokio::task::spawn_blocking(move || search_provider.search_files(&query)).await {
        Ok(Ok(files)) => {
            let response = SearchResponse {
                schema_version: 1,
                files: files.iter().map(FileRecordResponse::from_record).collect(),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(Err(error)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "search_failed",
                    message: error,
                },
            }),
        )
            .into_response(),
        Err(error) => {
            let message = format!("search worker failed: {error}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "internal_error",
                        message,
                    },
                }),
            )
                .into_response()
        }
    }
}
