//! Single file metadata delivery.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use tssp_domain::FileId;

use crate::upload::FileRecordResponse;
use crate::{ErrorBody, ErrorResponse, HttpState};

pub(crate) async fn get_file(State(state): State<HttpState>, Path(id): Path<String>) -> Response {
    let file_id = match FileId::new(id) {
        Ok(value) => value,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "invalid_file_id",
                        message: error.to_string(),
                    },
                }),
            )
                .into_response();
        }
    };

    let metadata = state.stats_provider.clone();
    match tokio::task::spawn_blocking(move || metadata.find_file(&file_id)).await {
        Ok(Ok(Some(record))) => (
            StatusCode::OK,
            Json(FileRecordResponse::from_record(&record)),
        )
            .into_response(),
        Ok(Ok(None)) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "file_not_found",
                    message: "file was not found".to_owned(),
                },
            }),
        )
            .into_response(),
        Ok(Err(error)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "metadata_unavailable",
                    message: error,
                },
            }),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "internal_error",
                    message: format!("metadata worker failed: {error}"),
                },
            }),
        )
            .into_response(),
    }
}
