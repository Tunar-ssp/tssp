//! Single file metadata delivery.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
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

/// Query params for thumbnail size selection.
#[derive(Debug, Deserialize)]
pub(crate) struct ThumbnailQuery {
    size: Option<String>,
}

/// GET /api/v1/files/{id}/thumbnail?size={small|medium|large}
///
/// Returns 404 for non-image files.
/// Returns 202 Accepted when the thumbnail is not yet generated (lazy mode).
pub(crate) async fn get_file_thumbnail(
    State(state): State<HttpState>,
    Path(id): Path<String>,
    Query(query): Query<ThumbnailQuery>,
) -> Response {
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

    let size = query.size.as_deref().unwrap_or("medium");
    if !matches!(size, "small" | "medium" | "large") {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "invalid_size",
                    message: "size must be small, medium, or large".to_owned(),
                },
            }),
        )
            .into_response();
    }

    let metadata = state.stats_provider.clone();
    let record = match tokio::task::spawn_blocking(move || metadata.find_file(&file_id)).await {
        Ok(Ok(Some(r))) => r,
        Ok(Ok(None)) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "file_not_found",
                        message: "file was not found".to_owned(),
                    },
                }),
            )
                .into_response();
        }
        _ => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "metadata_unavailable",
                        message: "could not look up file metadata".to_owned(),
                    },
                }),
            )
                .into_response();
        }
    };

    // Only images can have thumbnails
    if !record.mime_type.as_str().starts_with("image/") {
        return (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "not_an_image",
                    message: "thumbnails are only available for image files".to_owned(),
                },
            }),
        )
            .into_response();
    }

    // Lazy mode: thumbnail generation is deferred. Return 202 with retry hint.
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "status": "pending",
            "message": "thumbnail is being generated",
            "retry_after_seconds": 5,
            "file_id": record.id.as_str(),
            "size": size,
        })),
    )
        .into_response()
}
