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

    match tokio::task::spawn_blocking(move || stats_provider.list_files_recent(params.limit)).await
    {
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
