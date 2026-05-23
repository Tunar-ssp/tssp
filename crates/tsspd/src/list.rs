//! List files delivery with optional filtering and pagination.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use tssp_domain::{Cursor, TagKey, UnixTimestamp};
use tssp_ports::{ListQuery as RepositoryListQuery, ListSort, PagedFiles};

use crate::upload::FileRecordResponse;
use crate::{ErrorBody, ErrorResponse, HttpState};

/// Query parameters for listing files.
#[derive(Debug, Deserialize)]
pub(crate) struct ListQueryParams {
    /// Maximum number of files to return (default 50, max 500).
    #[serde(default = "default_limit")]
    limit: u64,
    /// Tag filters with AND semantics.
    #[serde(default, rename = "tag")]
    tags: Vec<String>,
    /// Optional MIME type prefix.
    #[serde(default, rename = "type")]
    mime_prefix: Option<String>,
    /// Optional filename substring filter.
    #[serde(default, rename = "name")]
    name_substring: Option<String>,
    /// Optional minimum upload timestamp.
    #[serde(default)]
    since: Option<i64>,
    /// Optional maximum upload timestamp.
    #[serde(default)]
    until: Option<i64>,
    /// Only return pinned files when true.
    #[serde(default, rename = "pinned")]
    pinned_only: bool,
    /// Optional sort field.
    #[serde(default)]
    sort: Option<String>,
    /// Cursor for the next page.
    #[serde(default, rename = "page")]
    page: Option<String>,
    /// Filter by virtual folder prefix (e.g. `photos` or `photos/vacation`).
    #[serde(default)]
    folder: Option<String>,
}

fn default_limit() -> u64 {
    50
}

impl ListQueryParams {
    fn into_repository_query(self) -> Result<RepositoryListQuery, String> {
        if self.limit == 0 {
            return Err("limit must be greater than 0".to_owned());
        }
        if self.limit > 500 {
            return Err("limit must not exceed 500".to_owned());
        }

        let tags = self
            .tags
            .into_iter()
            .map(|tag| TagKey::new(tag).map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?;

        let mime_prefix = self
            .mime_prefix
            .map(|value| validate_mime_prefix(&value))
            .transpose()?;
        let name_substring = self
            .name_substring
            .map(validate_name_substring)
            .transpose()?;

        let since = self
            .since
            .map(UnixTimestamp::new)
            .transpose()
            .map_err(|error| error.to_string())?;
        let until = self
            .until
            .map(UnixTimestamp::new)
            .transpose()
            .map_err(|error| error.to_string())?;

        if let (Some(since), Some(until)) = (since, until) {
            if since.seconds() > until.seconds() {
                return Err("since must not be later than until".to_owned());
            }
        }

        let sort = self
            .sort
            .as_deref()
            .map(|value| {
                ListSort::parse(value).ok_or_else(|| {
                    "sort must be one of uploaded, -uploaded, name, -name, size, -size".to_owned()
                })
            })
            .transpose()?
            .unwrap_or_default();

        let after_cursor = self
            .page
            .as_deref()
            .map(Cursor::new)
            .transpose()
            .map_err(|error| error.to_string())?;

        let folder_prefix = self
            .folder
            .map(|value| crate::folders::normalize_folder_path(&value));

        Ok(RepositoryListQuery {
            limit: self.limit,
            tags,
            mime_prefix,
            name_substring,
            since,
            until,
            pinned_only: self.pinned_only,
            sort,
            after_cursor,
            folder_prefix,
            visibility: None,
            owner_id: None,
        })
    }
}

fn validate_mime_prefix(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("MIME prefix must not be empty".to_owned());
    }
    if trimmed.chars().any(|character| {
        !(character.is_ascii_alphanumeric() || matches!(character, '/' | '+' | '-' | '.'))
    }) {
        return Err("MIME prefix contains invalid characters".to_owned());
    }
    Ok(trimmed.to_ascii_lowercase())
}

fn validate_name_substring(value: String) -> Result<String, String> {
    if value.trim().is_empty() {
        return Err("name filter must not be empty".to_owned());
    }
    Ok(value)
}

/// Response for list endpoint containing an array of files.
#[derive(Debug, Serialize)]
pub(crate) struct ListResponse {
    /// Stable response schema version.
    pub schema_version: u8,
    /// Array of file records.
    pub files: Vec<FileRecordResponse>,
    /// Cursor for the next page, if another page exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl ListResponse {
    fn from_paged_files(page: PagedFiles) -> Self {
        Self {
            schema_version: 1,
            files: page
                .files
                .iter()
                .map(FileRecordResponse::from_record)
                .collect(),
            next_cursor: page.next_cursor.map(|cursor| cursor.as_str().to_owned()),
        }
    }
}

pub(crate) async fn list_files(
    State(state): State<HttpState>,
    Query(params): Query<ListQueryParams>,
) -> Response {
    let query = match params.into_repository_query() {
        Ok(query) => query,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "invalid_request",
                        message: error,
                    },
                }),
            )
                .into_response();
        }
    };

    let stats_provider = state.stats_provider.clone();
    let fetch_result = tokio::task::spawn_blocking(move || stats_provider.list_files(&query)).await;

    match fetch_result {
        Ok(Ok(page)) => {
            (StatusCode::OK, Json(ListResponse::from_paged_files(page))).into_response()
        }
        Ok(Err(error)) if error.starts_with("invalid cursor:") => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "invalid_cursor",
                    message: error,
                },
            }),
        )
            .into_response(),
        Ok(Err(error)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "list_failed",
                    message: error,
                },
            }),
        )
            .into_response(),
        Err(error) => {
            let message = format!("list worker failed: {error}");
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
    use super::{default_limit, validate_mime_prefix, ListQueryParams};
    use tssp_domain::Cursor;
    use tssp_ports::ListSort;

    #[test]
    fn default_limit_is_50() {
        assert_eq!(default_limit(), 50);
    }

    #[test]
    fn into_repository_query_rejects_zero_limit() {
        let query = ListQueryParams {
            limit: 0,
            tags: Vec::new(),
            mime_prefix: None,
            name_substring: None,
            since: None,
            until: None,
            pinned_only: false,
            sort: None,
            page: None,
            folder: None,
        };
        let result = query.into_repository_query();
        assert!(matches!(result, Err(message) if message.contains("greater than 0")));
    }

    #[test]
    fn into_repository_query_rejects_over_500() {
        let query = ListQueryParams {
            limit: 501,
            tags: Vec::new(),
            mime_prefix: None,
            name_substring: None,
            since: None,
            until: None,
            pinned_only: false,
            sort: None,
            page: None,
            folder: None,
        };
        let result = query.into_repository_query();
        assert!(matches!(result, Err(message) if message.contains("500")));
    }

    #[test]
    fn into_repository_query_accepts_valid_limits() {
        for limit in [1, 50, 100, 500] {
            let query = ListQueryParams {
                limit,
                tags: Vec::new(),
                mime_prefix: None,
                name_substring: None,
                since: None,
                until: None,
                pinned_only: false,
                sort: None,
                page: None,
                folder: None,
            };
            assert!(
                query.into_repository_query().is_ok(),
                "limit {limit} should be valid"
            );
        }
    }

    #[test]
    fn into_repository_query_accepts_full_filter_set() {
        let query = ListQueryParams {
            limit: 50,
            tags: vec!["Docs".to_owned(), "Family".to_owned()],
            mime_prefix: Some("image".to_owned()),
            name_substring: Some("report".to_owned()),
            since: Some(1_700_000_000),
            until: Some(1_700_000_100),
            pinned_only: true,
            sort: Some("-name".to_owned()),
            page: Some("cursor-1".to_owned()),
            folder: None,
        };
        let result = query
            .into_repository_query()
            .unwrap_or_else(|error| panic!("query parse failed: {error}"));

        assert_eq!(result.tags.len(), 2);
        assert_eq!(result.mime_prefix.as_deref(), Some("image"));
        assert_eq!(result.name_substring.as_deref(), Some("report"));
        assert!(result.pinned_only);
        assert_eq!(result.sort, ListSort::NameDesc);
        assert_eq!(
            result.after_cursor.as_ref().map(Cursor::as_str),
            Some("cursor-1")
        );
    }

    #[test]
    fn into_repository_query_rejects_invalid_sort() {
        let query = ListQueryParams {
            limit: 50,
            tags: Vec::new(),
            mime_prefix: None,
            name_substring: None,
            since: None,
            until: None,
            pinned_only: false,
            sort: Some("bad".to_owned()),
            page: None,
            folder: None,
        };

        let result = query.into_repository_query();
        assert!(matches!(result, Err(message) if message.contains("sort must be one of")));
    }

    #[test]
    fn into_repository_query_rejects_inverted_time_bounds() {
        let query = ListQueryParams {
            limit: 50,
            tags: Vec::new(),
            mime_prefix: None,
            name_substring: None,
            since: Some(20),
            until: Some(10),
            pinned_only: false,
            sort: None,
            page: None,
            folder: None,
        };

        let result = query.into_repository_query();
        assert!(matches!(result, Err(message) if message.contains("since must not be later")));
    }

    #[test]
    fn validate_mime_prefix_rejects_invalid_values() {
        assert!(matches!(
            validate_mime_prefix(" "),
            Err(message) if message.contains("must not be empty")
        ));
        assert!(matches!(
            validate_mime_prefix("image*"),
            Err(message) if message.contains("invalid characters")
        ));
    }

    #[test]
    fn into_repository_query_rejects_blank_name_filter() {
        let query = ListQueryParams {
            limit: 50,
            tags: Vec::new(),
            mime_prefix: None,
            name_substring: Some("   ".to_owned()),
            since: None,
            until: None,
            pinned_only: false,
            sort: None,
            page: None,
            folder: None,
        };

        let result = query.into_repository_query();
        assert!(matches!(result, Err(message) if message.contains("name filter")));
    }
}
