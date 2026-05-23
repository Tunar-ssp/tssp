//! Full-text search endpoint.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::notes::NoteRecordResponse;
use crate::upload::FileRecordResponse;
use crate::{ErrorBody, ErrorResponse, HttpState};
use tssp_domain::{TagKey, Visibility};
use tssp_ports::{NoteRepository, SearchHit};

const DEFAULT_SEARCH_LIMIT: u64 = 50;
const MAX_SEARCH_LIMIT: u64 = 100;

/// Query parameters for searching files.
#[derive(Debug, Deserialize)]
pub(crate) struct SearchQuery {
    /// The search string.
    pub q: String,
    /// Maximum number of results to return.
    #[serde(default)]
    pub limit: Option<u64>,
    /// Optional result kind (`file`, `note`, or `all`).
    #[serde(default)]
    pub kind: Option<String>,
    /// Optional tag filter applied to files and notes.
    #[serde(default)]
    pub tag: Option<String>,
    /// Optional MIME prefix filter for files.
    #[serde(default, rename = "type")]
    pub mime_prefix: Option<String>,
    /// Only return pinned files/notes.
    #[serde(default)]
    pub pinned: bool,
    /// Optional file visibility filter (`public` or `private`).
    #[serde(default)]
    pub visibility: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchKind {
    All,
    File,
    Note,
}

#[derive(Debug)]
struct SearchFilters {
    limit: u64,
    kind: SearchKind,
    tag: Option<TagKey>,
    mime_prefix: Option<String>,
    pinned: bool,
    visibility: Option<Visibility>,
}

impl SearchQuery {
    fn to_filters(&self) -> Result<SearchFilters, String> {
        let limit = self.limit.unwrap_or(DEFAULT_SEARCH_LIMIT);
        if limit == 0 {
            return Err("limit must be greater than 0".to_owned());
        }
        if limit > MAX_SEARCH_LIMIT {
            return Err(format!("limit must not exceed {MAX_SEARCH_LIMIT}"));
        }

        let kind = parse_kind(self.kind.as_deref())?;
        let tag = self
            .tag
            .as_deref()
            .map(TagKey::new)
            .transpose()
            .map_err(|error| error.to_string())?;
        let mime_prefix = self
            .mime_prefix
            .as_deref()
            .map(validate_mime_prefix)
            .transpose()?;
        if mime_prefix.is_some() && kind == SearchKind::Note {
            return Err("type filter can only be used with file or all search".to_owned());
        }
        let visibility = self
            .visibility
            .as_deref()
            .map(Visibility::parse)
            .transpose()
            .map_err(|error| error.to_string())?;
        if visibility.is_some() && kind == SearchKind::Note {
            return Err("visibility filter can only be used with file or all search".to_owned());
        }

        Ok(SearchFilters {
            limit,
            kind,
            tag,
            mime_prefix,
            pinned: self.pinned,
            visibility,
        })
    }
}

fn parse_kind(value: Option<&str>) -> Result<SearchKind, String> {
    match value.unwrap_or("all").trim().to_ascii_lowercase().as_str() {
        "" | "all" => Ok(SearchKind::All),
        "file" | "files" => Ok(SearchKind::File),
        "note" | "notes" => Ok(SearchKind::Note),
        _ => Err("kind must be one of all, file, or note".to_owned()),
    }
}

fn validate_mime_prefix(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("type filter must not be empty".to_owned());
    }
    if trimmed.chars().any(|character| {
        !(character.is_ascii_alphanumeric() || matches!(character, '/' | '+' | '-' | '.'))
    }) {
        return Err("type filter contains invalid characters".to_owned());
    }
    Ok(trimmed.to_ascii_lowercase())
}

/// One unified search result.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub(crate) enum SearchResultItem {
    /// Matching file.
    File {
        /// File metadata payload.
        #[serde(flatten)]
        record: FileRecordResponse,
    },
    /// Matching note.
    Note {
        /// Note metadata payload.
        #[serde(flatten)]
        record: NoteRecordResponse,
    },
}

/// Response for search endpoint.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SearchResponse {
    /// Stable response schema version.
    pub schema_version: u8,
    /// Applied result limit.
    pub limit: u64,
    /// Ranked matches across files and notes.
    pub results: Vec<SearchResultItem>,
}

/// Provides unified search functionality.
pub trait FileSearchProvider: Send + Sync {
    /// Searches files and notes matching the query.
    ///
    /// # Errors
    ///
    /// Returns a short diagnostic when the query fails.
    fn search(&self, query: &str) -> Result<Vec<SearchHit>, String>;
}

#[derive(Debug)]
pub(crate) struct StaticFileSearchProvider;

impl FileSearchProvider for StaticFileSearchProvider {
    fn search(&self, _query: &str) -> Result<Vec<SearchHit>, String> {
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
    R: NoteRepository + Send + Sync,
{
    fn search(&self, query: &str) -> Result<Vec<SearchHit>, String> {
        self.repository
            .search_all(query)
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
    let filters = match params.to_filters() {
        Ok(value) => value,
        Err(message) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "invalid_request",
                        message,
                    },
                }),
            )
                .into_response();
        }
    };

    let search_provider = state.search_provider.clone();
    let query = params.q.clone();

    match tokio::task::spawn_blocking(move || search_provider.search(&query)).await {
        Ok(Ok(hits)) => {
            let results = hits
                .into_iter()
                .filter(|hit| hit_matches_filters(hit, &filters))
                .take(usize::try_from(filters.limit).unwrap_or(usize::MAX))
                .map(|hit| match &hit {
                    SearchHit::File(file) => SearchResultItem::File {
                        record: FileRecordResponse::from_record(file),
                    },
                    SearchHit::Note(note) => SearchResultItem::Note {
                        record: NoteRecordResponse::from_record(note),
                    },
                })
                .collect();
            let response = SearchResponse {
                schema_version: 1,
                limit: filters.limit,
                results,
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

fn hit_matches_filters(hit: &SearchHit, filters: &SearchFilters) -> bool {
    match hit {
        SearchHit::File(file) => {
            if filters.kind == SearchKind::Note {
                return false;
            }
            if filters.pinned && !file.is_pinned() {
                return false;
            }
            if let Some(tag) = &filters.tag {
                if !file.tags.iter().any(|item| item.key() == tag) {
                    return false;
                }
            }
            if let Some(prefix) = &filters.mime_prefix {
                if !file.mime_type.as_str().starts_with(prefix) {
                    return false;
                }
            }
            if let Some(visibility) = filters.visibility {
                if file.visibility != visibility {
                    return false;
                }
            }
            true
        }
        SearchHit::Note(note) => {
            if filters.kind == SearchKind::File
                || filters.mime_prefix.is_some()
                || filters.visibility.is_some()
            {
                return false;
            }
            if filters.pinned && note.pinned_at.is_none() {
                return false;
            }
            if let Some(tag) = &filters.tag {
                return note.tags.iter().any(|item| item.key() == tag);
            }
            true
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
    use tower::ServiceExt;
    use tssp_ports::FileRepository;

    fn build_test_router(provider: Arc<dyn FileSearchProvider>) -> Router {
        let mut state = HttpState::test_http_state(std::path::PathBuf::from("/tmp"));
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
    async fn search_rejects_invalid_limit() {
        let provider = Arc::new(StaticFileSearchProvider);
        let router = build_test_router(provider);

        let request = Request::builder()
            .uri("/search?q=test&limit=0")
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
        assert_eq!(result.limit, DEFAULT_SEARCH_LIMIT);
        assert!(result.results.is_empty());
    }

    #[test]
    fn search_query_accepts_supported_filters() {
        let query = SearchQuery {
            q: "report".to_owned(),
            limit: Some(10),
            kind: Some("file".to_owned()),
            tag: Some("Finance".to_owned()),
            mime_prefix: Some("application/pdf".to_owned()),
            pinned: true,
            visibility: Some("public".to_owned()),
        };

        let filters = query
            .to_filters()
            .unwrap_or_else(|error| panic!("filters failed: {error}"));

        assert_eq!(filters.limit, 10);
        assert_eq!(filters.kind, SearchKind::File);
        assert!(filters.tag.is_some());
        assert_eq!(filters.mime_prefix.as_deref(), Some("application/pdf"));
        assert_eq!(filters.visibility, Some(Visibility::Public));
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

        fn rename_file(
            &self,
            _id: &tssp_domain::FileId,
            _new_name: &tssp_domain::FileName,
        ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }

        fn list_folder_counts(&self) -> Result<Vec<(String, u64)>, tssp_ports::RepositoryError> {
            Ok(Vec::new())
        }

        fn set_file_visibility(
            &self,
            _id: &tssp_domain::FileId,
            _visibility: tssp_domain::Visibility,
            _public_token: Option<&str>,
        ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }

        fn find_file_by_public_token(
            &self,
            _token: &str,
        ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }

        fn update_folder_path_prefix(
            &self,
            _from_prefix: &str,
            _to_prefix: &str,
        ) -> Result<u64, tssp_ports::RepositoryError> {
            unimplemented!()
        }

        fn set_file_folder_path(
            &self,
            _id: &tssp_domain::FileId,
            _folder_path: &str,
        ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }
    }

    impl NoteRepository for MockRepo {
        fn insert_note(
            &self,
            _new_note: tssp_ports::NewNoteRecord,
        ) -> Result<tssp_domain::NoteRecord, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn find_note(
            &self,
            _id: &tssp_domain::NoteId,
        ) -> Result<Option<tssp_domain::NoteRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn update_note(
            &self,
            _id: &tssp_domain::NoteId,
            _title: &tssp_domain::NoteTitle,
            _body: &tssp_domain::NoteBody,
            _updated_at: tssp_domain::UnixTimestamp,
        ) -> Result<tssp_domain::NoteRecord, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn delete_note(
            &self,
            _id: &tssp_domain::NoteId,
        ) -> Result<bool, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn list_notes(
            &self,
            _query: &tssp_ports::NoteListQuery,
        ) -> Result<tssp_ports::PagedNotes, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn add_tags_to_note(
            &self,
            _id: &tssp_domain::NoteId,
            _tags: &[tssp_domain::Tag],
        ) -> Result<tssp_ports::TagMutationOutcome, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn remove_tag_from_note(
            &self,
            _id: &tssp_domain::NoteId,
            _tag: &tssp_domain::TagKey,
        ) -> Result<tssp_ports::TagMutationOutcome, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn pin_note(
            &self,
            _id: &tssp_domain::NoteId,
            _position: Option<u32>,
        ) -> Result<tssp_ports::PinOutcome, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn unpin_note(
            &self,
            _id: &tssp_domain::NoteId,
        ) -> Result<tssp_ports::PinOutcome, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn search_notes(
            &self,
            _query: &str,
        ) -> Result<Vec<tssp_domain::NoteRecord>, tssp_ports::RepositoryError> {
            Ok(vec![])
        }
        fn search_all(
            &self,
            _query: &str,
        ) -> Result<Vec<tssp_ports::SearchHit>, tssp_ports::RepositoryError> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn repository_search_provider_delegates_to_repo() {
        let provider = RepositoryFileSearchProvider::new(MockRepo);
        let result = provider
            .search("test")
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

        fn rename_file(
            &self,
            _id: &tssp_domain::FileId,
            _new_name: &tssp_domain::FileName,
        ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }

        fn list_folder_counts(&self) -> Result<Vec<(String, u64)>, tssp_ports::RepositoryError> {
            Ok(Vec::new())
        }

        fn set_file_visibility(
            &self,
            _id: &tssp_domain::FileId,
            _visibility: tssp_domain::Visibility,
            _public_token: Option<&str>,
        ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }

        fn find_file_by_public_token(
            &self,
            _token: &str,
        ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }

        fn update_folder_path_prefix(
            &self,
            _from_prefix: &str,
            _to_prefix: &str,
        ) -> Result<u64, tssp_ports::RepositoryError> {
            unimplemented!()
        }

        fn set_file_folder_path(
            &self,
            _id: &tssp_domain::FileId,
            _folder_path: &str,
        ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }
    }

    impl NoteRepository for FailingMockRepo {
        fn insert_note(
            &self,
            _new_note: tssp_ports::NewNoteRecord,
        ) -> Result<tssp_domain::NoteRecord, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn find_note(
            &self,
            _id: &tssp_domain::NoteId,
        ) -> Result<Option<tssp_domain::NoteRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn update_note(
            &self,
            _id: &tssp_domain::NoteId,
            _title: &tssp_domain::NoteTitle,
            _body: &tssp_domain::NoteBody,
            _updated_at: tssp_domain::UnixTimestamp,
        ) -> Result<tssp_domain::NoteRecord, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn delete_note(
            &self,
            _id: &tssp_domain::NoteId,
        ) -> Result<bool, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn list_notes(
            &self,
            _query: &tssp_ports::NoteListQuery,
        ) -> Result<tssp_ports::PagedNotes, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn add_tags_to_note(
            &self,
            _id: &tssp_domain::NoteId,
            _tags: &[tssp_domain::Tag],
        ) -> Result<tssp_ports::TagMutationOutcome, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn remove_tag_from_note(
            &self,
            _id: &tssp_domain::NoteId,
            _tag: &tssp_domain::TagKey,
        ) -> Result<tssp_ports::TagMutationOutcome, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn pin_note(
            &self,
            _id: &tssp_domain::NoteId,
            _position: Option<u32>,
        ) -> Result<tssp_ports::PinOutcome, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn unpin_note(
            &self,
            _id: &tssp_domain::NoteId,
        ) -> Result<tssp_ports::PinOutcome, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn search_notes(
            &self,
            _query: &str,
        ) -> Result<Vec<tssp_domain::NoteRecord>, tssp_ports::RepositoryError> {
            unimplemented!()
        }
        fn search_all(
            &self,
            _query: &str,
        ) -> Result<Vec<tssp_ports::SearchHit>, tssp_ports::RepositoryError> {
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
