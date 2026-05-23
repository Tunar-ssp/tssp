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
#[derive(Debug, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::Request;
    use axum::routing::get;
    use axum::Router;
    use std::sync::Arc;
    use std::time::Instant;
    use tower::ServiceExt;

    fn build_test_router(provider: Arc<dyn FileSearchProvider>) -> Router {
        let mut state = HttpState::new(Instant::now(), std::path::PathBuf::from("/tmp"));
        state = state.with_search_provider(provider);
        Router::new()
            .route("/search", get(search_files))
            .with_state(state)
    }

    #[tokio::test]
    async fn search_returns_bad_request_on_empty_query() {
        let provider = Arc::new(StaticFileSearchProvider);
        let router = build_test_router(provider);

        let request = Request::builder()
            .uri("/search?q=")
            .body(axum::body::Body::empty())
            .unwrap_or_else(|error| panic!("request build failed: {error}"));

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn search_returns_ok_on_valid_query() {
        let provider = Arc::new(StaticFileSearchProvider);
        let router = build_test_router(provider);

        let request = Request::builder()
            .uri("/search?q=test")
            .body(axum::body::Body::empty())
            .unwrap_or_else(|error| panic!("request build failed: {error}"));

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap_or_else(|error| panic!("body read failed: {error}"));
        let result: SearchResponse = serde_json::from_slice(&body)
            .unwrap_or_else(|error| panic!("json parse failed: {error}"));
        assert_eq!(result.schema_version, 1);
        assert!(result.files.is_empty());
    }

    struct MockRepo;

    impl FileRepository for MockRepo {
        fn insert_file(
            &self,
            _new_file: tssp_ports::NewFileRecord,
        ) -> Result<tssp_domain::FileRecord, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn find_file(
            &self,
            _id: &tssp_domain::FileId,
        ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn find_file_by_content_hash(
            &self,
            _content_hash: &tssp_domain::ContentHash,
        ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn delete_file(
            &self,
            _id: &tssp_domain::FileId,
        ) -> Result<Option<tssp_ports::DeletedFileRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn list_files(
            &self,
            _query: &tssp_ports::ListQuery,
        ) -> Result<tssp_ports::PagedFiles, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn list_files_recent(
            &self,
            _limit: u64,
        ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn list_files_by_tag(
            &self,
            _tag: &tssp_domain::TagKey,
            _limit: u64,
        ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn list_tags(&self) -> Result<Vec<tssp_ports::TagSummary>, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn add_tags_to_file(
            &self,
            _id: &tssp_domain::FileId,
            _tags: &[tssp_domain::Tag],
        ) -> Result<tssp_ports::TagMutationOutcome, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn remove_tag_from_file(
            &self,
            _id: &tssp_domain::FileId,
            _tag: &tssp_domain::TagKey,
        ) -> Result<tssp_ports::TagMutationOutcome, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn stats_since(
            &self,
            _recent_since: tssp_domain::UnixTimestamp,
        ) -> Result<tssp_ports::RepositoryStats, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn pin_file(
            &self,
            _id: &tssp_domain::FileId,
            _position: Option<u32>,
        ) -> Result<tssp_ports::PinOutcome, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn unpin_file(
            &self,
            _id: &tssp_domain::FileId,
        ) -> Result<tssp_ports::PinOutcome, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn list_pinned_files(
            &self,
        ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn reorder_pins(
            &self,
            _ordered_ids: &[tssp_domain::FileId],
        ) -> Result<(), tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn search_files(
            &self,
            _query: &str,
        ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn repository_search_provider_delegates_to_repo() {
        let repo = MockRepo;
        let provider = RepositoryFileSearchProvider::new(repo);

        let result = provider
            .search_files("test")
            .unwrap_or_else(|error| panic!("search failed: {error}"));
        assert!(result.is_empty());
    }

    struct FailingMockRepo;

    impl FileRepository for FailingMockRepo {
        fn insert_file(
            &self,
            _new_file: tssp_ports::NewFileRecord,
        ) -> Result<tssp_domain::FileRecord, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn find_file(
            &self,
            _id: &tssp_domain::FileId,
        ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn find_file_by_content_hash(
            &self,
            _content_hash: &tssp_domain::ContentHash,
        ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn delete_file(
            &self,
            _id: &tssp_domain::FileId,
        ) -> Result<Option<tssp_ports::DeletedFileRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn list_files(
            &self,
            _query: &tssp_ports::ListQuery,
        ) -> Result<tssp_ports::PagedFiles, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn list_files_recent(
            &self,
            _limit: u64,
        ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn list_files_by_tag(
            &self,
            _tag: &tssp_domain::TagKey,
            _limit: u64,
        ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn list_tags(&self) -> Result<Vec<tssp_ports::TagSummary>, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn add_tags_to_file(
            &self,
            _id: &tssp_domain::FileId,
            _tags: &[tssp_domain::Tag],
        ) -> Result<tssp_ports::TagMutationOutcome, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn remove_tag_from_file(
            &self,
            _id: &tssp_domain::FileId,
            _tag: &tssp_domain::TagKey,
        ) -> Result<tssp_ports::TagMutationOutcome, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn stats_since(
            &self,
            _recent_since: tssp_domain::UnixTimestamp,
        ) -> Result<tssp_ports::RepositoryStats, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn pin_file(
            &self,
            _id: &tssp_domain::FileId,
            _position: Option<u32>,
        ) -> Result<tssp_ports::PinOutcome, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn unpin_file(
            &self,
            _id: &tssp_domain::FileId,
        ) -> Result<tssp_ports::PinOutcome, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn list_pinned_files(
            &self,
        ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn reorder_pins(
            &self,
            _ordered_ids: &[tssp_domain::FileId],
        ) -> Result<(), tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn search_files(
            &self,
            _query: &str,
        ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
            Err(tssp_ports::RepositoryError::OperationFailed {
                message: "db error".into(),
            })
        }
    }

    #[tokio::test]
    async fn search_returns_internal_error_on_failure() {
        let repo = FailingMockRepo;
        let provider = Arc::new(RepositoryFileSearchProvider::new(repo));
        let router = build_test_router(provider);

        let request = Request::builder()
            .uri("/search?q=test")
            .body(axum::body::Body::empty())
            .unwrap_or_else(|error| panic!("request build failed: {error}"));

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
