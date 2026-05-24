//! Full-text search endpoint.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::auth::OptionalAuthContext;
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
    Workspace,
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
        if mime_prefix.is_some() && matches!(kind, SearchKind::Note | SearchKind::Workspace) {
            return Err("type filter can only be used with file or all search".to_owned());
        }
        let visibility = self
            .visibility
            .as_deref()
            .map(Visibility::parse)
            .transpose()
            .map_err(|error| error.to_string())?;
        if visibility.is_some() && matches!(kind, SearchKind::Note | SearchKind::Workspace) {
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
        "workspace" | "workspaces" => Ok(SearchKind::Workspace),
        _ => Err("kind must be one of all, file, note, or workspace".to_owned()),
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
    /// Matching saved workspace.
    Workspace {
        /// Workspace id.
        id: String,
        /// Workspace owner.
        owner_id: String,
        /// Workspace name.
        name: String,
        /// Workspace language/type metadata.
        language: String,
        /// Last update timestamp.
        updated_at: i64,
        /// Lightweight body excerpt.
        snippet: String,
    },
}

/// Response for search endpoint.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SearchResponse {
    /// Stable response schema version.
    pub schema_version: u8,
    /// Applied result limit.
    pub limit: u64,
    /// Total number of results returned.
    pub result_count: usize,
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
    OptionalAuthContext(auth): OptionalAuthContext,
    Query(params): Query<SearchQuery>,
) -> Response {
    if params.q.trim().is_empty() {
        return invalid_request("query parameter 'q' must not be empty".to_owned());
    }
    let filters = match params.to_filters() {
        Ok(value) => value,
        Err(message) => return invalid_request(message),
    };

    let search_provider = state.search_provider.clone();
    let query = params.q.clone();

    match tokio::task::spawn_blocking(move || search_provider.search(&query)).await {
        Ok(Ok(hits)) => {
            let results =
                match build_search_results(&state, auth.as_ref(), &params.q, &filters, hits) {
                    Ok(results) => results,
                    Err(error) => return search_error("search_failed", error),
                };
            let result_count = results.len();
            let response = SearchResponse {
                schema_version: 1,
                limit: filters.limit,
                result_count,
                results,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(Err(error)) => search_error("search_failed", error),
        Err(error) => search_error("internal_error", format!("search worker failed: {error}")),
    }
}

fn invalid_request(message: String) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "invalid_request",
                message,
            },
        }),
    )
        .into_response()
}

fn search_error(code: &'static str, message: String) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: ErrorBody { code, message },
        }),
    )
        .into_response()
}

fn build_search_results(
    state: &HttpState,
    auth: Option<&crate::auth::AuthContext>,
    query: &str,
    filters: &SearchFilters,
    hits: Vec<SearchHit>,
) -> Result<Vec<SearchResultItem>, String> {
    let limit = usize::try_from(filters.limit).unwrap_or(usize::MAX);
    let mut results: Vec<SearchResultItem> = hits
        .into_iter()
        .filter(|hit| hit_matches_filters(hit, filters))
        .take(limit)
        .map(|hit| match &hit {
            SearchHit::File(file) => SearchResultItem::File {
                record: FileRecordResponse::from_record(file),
            },
            SearchHit::Note(note) => SearchResultItem::Note {
                record: NoteRecordResponse::from_record(note),
            },
        })
        .collect();
    if results.len() < limit {
        let mut workspace_results =
            search_workspaces(state, auth, query, filters, limit - results.len())?;
        results.append(&mut workspace_results);
    }
    Ok(results)
}

fn search_workspaces(
    state: &HttpState,
    auth: Option<&crate::auth::AuthContext>,
    query: &str,
    filters: &SearchFilters,
    limit: usize,
) -> Result<Vec<SearchResultItem>, String> {
    if !workspace_filters_allow_results(filters) {
        return Ok(Vec::new());
    }
    let Some(auth) = auth else {
        return Ok(Vec::new());
    };
    let Some(store) = state.workspaces.as_deref() else {
        return Ok(Vec::new());
    };
    let owner = if auth.is_admin() {
        None
    } else {
        Some(auth.user_id.as_str())
    };
    let matches = store
        .search(query, owner, u64::try_from(limit).unwrap_or(filters.limit))
        .map_err(|error| error.to_string())?;
    Ok(matches
        .into_iter()
        .map(|workspace| SearchResultItem::Workspace {
            id: workspace.id,
            owner_id: workspace.owner_id,
            name: workspace.name,
            language: workspace.language,
            updated_at: workspace.updated_at,
            snippet: workspace.snippet,
        })
        .take(limit)
        .collect())
}

fn workspace_filters_allow_results(filters: &SearchFilters) -> bool {
    matches!(filters.kind, SearchKind::All | SearchKind::Workspace)
        && filters.tag.is_none()
        && filters.mime_prefix.is_none()
        && filters.visibility.is_none()
        && !filters.pinned
}

fn hit_matches_filters(hit: &SearchHit, filters: &SearchFilters) -> bool {
    match hit {
        SearchHit::File(file) => {
            if matches!(filters.kind, SearchKind::Note | SearchKind::Workspace) {
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
            if matches!(filters.kind, SearchKind::File | SearchKind::Workspace)
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
#[path = "search_tests.rs"]
mod tests;
