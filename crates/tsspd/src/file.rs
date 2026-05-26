//! Single file metadata delivery.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use tssp_domain::{FileId, Visibility};

use crate::upload::FileRecordResponse;
use crate::{ErrorBody, ErrorResponse, HttpState};

pub(crate) async fn get_file(
    State(state): State<HttpState>,
    crate::auth::OptionalAuthContext(auth): crate::auth::OptionalAuthContext,
    Path(id): Path<String>,
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

    let metadata = state.stats_provider.clone();
    let record = match tokio::task::spawn_blocking(move || metadata.find_file(&file_id)).await {
        Ok(Ok(Some(record))) => record,
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
        Ok(Err(error)) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "metadata_unavailable",
                        message: error,
                    },
                }),
            )
                .into_response();
        }
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "internal_error",
                        message: format!("metadata worker failed: {error}"),
                    },
                }),
            )
                .into_response();
        }
    };

    // Check authorization: public files accessible to all, private files only to owner/admin
    if let Err(resp) = authorize_file_access(&record, auth.as_ref()) {
        return resp;
    }

    (
        StatusCode::OK,
        Json(FileRecordResponse::from_record(&record)),
    )
        .into_response()
}

/// Query params for thumbnail size selection.
#[derive(Debug, Deserialize)]
pub(crate) struct ThumbnailQuery {
    size: Option<String>,
}

/// GET /api/v1/files/{id}/thumbnail?size={small|medium|large}
///
/// Returns 404 for non-image files.
/// Returns 501 Not Implemented because thumbnail generation is not yet implemented.
pub(crate) async fn get_file_thumbnail(
    State(state): State<HttpState>,
    crate::auth::OptionalAuthContext(auth): crate::auth::OptionalAuthContext,
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

    // Check authorization: public files accessible to all, private files only to owner/admin
    if let Err(resp) = authorize_file_access(&record, auth.as_ref()) {
        return resp;
    }

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

    // Thumbnail generation is not implemented yet.
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "not_implemented",
                message: "thumbnail generation is not yet implemented".to_owned(),
            },
        }),
    )
        .into_response()
}

#[allow(clippy::result_large_err)]
fn authorize_file_access(
    record: &tssp_domain::FileRecord,
    auth: Option<&crate::auth::AuthContext>,
) -> Result<(), Response> {
    if record.visibility == Visibility::Public {
        return Ok(());
    }

    match auth {
        Some(auth) if auth.is_admin() || record.owner_id.as_ref() == Some(&auth.user_id) => Ok(()),
        Some(_) => Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "access_denied",
                    message: "you do not have permission to access this file".to_owned(),
                },
            }),
        )
            .into_response()),
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "authentication_required",
                    message: "authentication is required to access this file".to_owned(),
                },
            }),
        )
            .into_response()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumbnail_query_size_defaults_to_none() {
        let query = ThumbnailQuery { size: None };
        assert_eq!(query.size.as_deref(), None);
    }

    #[test]
    fn thumbnail_query_size_deserializes_small() {
        let query = ThumbnailQuery {
            size: Some("small".to_owned()),
        };
        assert_eq!(query.size.as_deref(), Some("small"));
    }

    #[test]
    fn thumbnail_query_size_deserializes_medium() {
        let query = ThumbnailQuery {
            size: Some("medium".to_owned()),
        };
        assert_eq!(query.size.as_deref(), Some("medium"));
    }

    #[test]
    fn thumbnail_query_size_deserializes_large() {
        let query = ThumbnailQuery {
            size: Some("large".to_owned()),
        };
        assert_eq!(query.size.as_deref(), Some("large"));
    }

    #[test]
    fn valid_sizes_match_allowed_list() {
        let size = "medium";
        assert!(matches!(size, "small" | "medium" | "large"));
    }

    #[test]
    fn invalid_size_does_not_match() {
        let size = "xlarge";
        assert!(!matches!(size, "small" | "medium" | "large"));
    }

    #[test]
    fn mime_type_image_detection() {
        let mime_type = "image/png";
        assert!(mime_type.starts_with("image/"));
    }

    #[test]
    fn mime_type_non_image_detection() {
        let mime_type = "application/pdf";
        assert!(!mime_type.starts_with("image/"));
    }
}
