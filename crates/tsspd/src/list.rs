//! List files delivery with optional filtering and pagination.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::upload::FileRecordResponse;
use crate::{ErrorBody, ErrorResponse, HttpState};

/// Query parameters for listing files.
#[derive(Debug, Deserialize)]
pub(crate) struct ListQuery {
    /// Maximum number of files to return (default 50, max 500).
    #[serde(default = "default_limit")]
    limit: u64,
    /// Optional tag to filter by.
    tag: Option<String>,
}

fn default_limit() -> u64 {
    50
}

impl ListQuery {
    fn validate(&self) -> Result<(), String> {
        if self.limit == 0 {
            return Err("limit must be greater than 0".to_owned());
        }
        if self.limit > 500 {
            return Err("limit must not exceed 500".to_owned());
        }
        Ok(())
    }
}

/// Response for list endpoint containing an array of files.
#[derive(Debug, Serialize)]
pub(crate) struct ListResponse {
    /// Stable response schema version.
    pub schema_version: u8,
    /// Array of file records.
    pub files: Vec<FileRecordResponse>,
}

pub(crate) async fn list_files(
    State(state): State<HttpState>,
    Query(params): Query<ListQuery>,
) -> Response {
    if let Err(error) = params.validate() {
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

    let stats_provider = state.stats_provider.clone();
    let limit = params.limit;

    let fetch_result = match params.tag {
        Some(tag) => match tssp_domain::TagKey::new(&tag) {
            Ok(tag_key) => {
                tokio::task::spawn_blocking(move || {
                    stats_provider.list_files_by_tag(&tag_key, limit)
                })
                .await
            }
            Err(error) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: ErrorBody {
                            code: "invalid_tag",
                            message: error.to_string(),
                        },
                    }),
                )
                    .into_response();
            }
        },
        None => tokio::task::spawn_blocking(move || stats_provider.list_files_recent(limit)).await,
    };

    match fetch_result {
        Ok(Ok(files)) => {
            let response = ListResponse {
                schema_version: 1,
                files: files.iter().map(FileRecordResponse::from_record).collect(),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
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
    use super::{default_limit, ListQuery};

    #[test]
    fn default_limit_is_50() {
        assert_eq!(default_limit(), 50);
    }

    #[test]
    fn validate_rejects_zero_limit() {
        let query = ListQuery {
            limit: 0,
            tag: None,
        };
        let result = query.validate();
        assert!(matches!(result, Err(message) if message.contains("greater than 0")));
    }

    #[test]
    fn validate_rejects_over_500() {
        let query = ListQuery {
            limit: 501,
            tag: None,
        };
        let result = query.validate();
        assert!(matches!(result, Err(message) if message.contains("500")));
    }

    #[test]
    fn validate_accepts_valid_limits() {
        for limit in [1, 50, 100, 500] {
            let query = ListQuery { limit, tag: None };
            assert!(query.validate().is_ok(), "limit {limit} should be valid");
        }
    }

    #[test]
    fn validate_accepts_with_tag() {
        let query = ListQuery {
            limit: 50,
            tag: Some("Docs".to_owned()),
        };
        assert!(query.validate().is_ok());
    }
}
