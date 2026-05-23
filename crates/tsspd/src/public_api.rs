//! Public file listing and download (unauthenticated when visibility is public).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use tssp_domain::Visibility;
use tssp_ports::ListQuery;

use crate::content::{self, DispositionMode};
use crate::upload::FileRecordResponse;
use crate::{ErrorBody, ErrorResponse, HttpState};

/// `GET /api/v1/public/files`
pub async fn list_public_files(State(state): State<HttpState>) -> impl IntoResponse {
    let query = ListQuery {
        limit: 200,
        visibility: Some(Visibility::Public),
        ..ListQuery::default()
    };
    match state.stats_provider.list_files(&query) {
        Ok(page) => {
            let files: Vec<FileRecordResponse> = page
                .files
                .iter()
                .map(FileRecordResponse::from_record)
                .collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "schema_version": 1,
                    "files": files,
                })),
            )
                .into_response()
        }
        Err(message) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "public_list_failed",
                    message,
                },
            }),
        )
            .into_response(),
    }
}

/// `GET /p/{token}` — stream public file content.
pub async fn public_download(
    State(state): State<HttpState>,
    Path(token): Path<String>,
) -> Response {
    let file = match state.stats_provider.find_file_by_public_token(&token) {
        Ok(Some(file)) => file,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let blob = match content::open_blob(state, file.storage_handle.clone()).await {
        Ok(value) => value,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "blob_unavailable",
                        message: error.to_string(),
                    },
                }),
            )
                .into_response();
        }
    };
    content::stream_blob_response(&file, blob, None, DispositionMode::Attachment)
}
