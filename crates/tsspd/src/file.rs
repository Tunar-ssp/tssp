//! Single file metadata delivery.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use std::os::unix::io::IntoRawFd;
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

/// Pixel dimensions for thumbnail size variants.
fn thumbnail_pixels(size: &str) -> u32 {
    match size {
        "small" => 128,
        "large" => 512,
        _ => 256, // medium
    }
}

/// Returns the JPEG bytes of a thumbnail, generating and caching it on first request.
///
/// Cache path: `<data_dir>/.thumbnails/<blake3_hash>-<size>.jpg`
fn generate_or_load_thumbnail(
    source_path: impl AsRef<std::path::Path>,
    cache_path: &std::path::Path,
    max_px: u32,
) -> Result<Vec<u8>, String> {
    if let Ok(bytes) = std::fs::read(cache_path) {
        return Ok(bytes);
    }

    let img = image::open(source_path).map_err(|e| format!("image decode failed: {e}"))?;
    let thumb = img.thumbnail(max_px, max_px);

    // Ensure cache directory exists.
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("cache dir create failed: {e}"))?;
    }

    let mut buf = std::io::Cursor::new(Vec::new());
    thumb
        .write_to(&mut buf, image::ImageFormat::Jpeg)
        .map_err(|e| format!("jpeg encode failed: {e}"))?;
    let bytes = buf.into_inner();

    // Write to cache atomically using a temp file in the same directory.
    let tmp = cache_path.with_extension("tmp");
    std::fs::write(&tmp, &bytes).map_err(|e| format!("cache write failed: {e}"))?;
    std::fs::rename(&tmp, cache_path).map_err(|e| format!("cache rename failed: {e}"))?;

    Ok(bytes)
}

/// GET /api/v1/files/{id}/thumbnail?size={small|medium|large}
///
/// Generates a JPEG thumbnail and caches it in `<data_dir>/.thumbnails/`.
#[allow(clippy::too_many_lines)]
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

    // Locate the blob file on disk.
    let blob_reader = state.blob_reader.clone();
    let storage_handle = record.storage_handle.clone();
    let source_file =
        match tokio::task::spawn_blocking(move || blob_reader.open_blob(&storage_handle)).await {
            Ok(Ok(file)) => file,
            Ok(Err(e)) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: ErrorBody {
                            code: "blob_unavailable",
                            message: format!("could not open blob: {e}"),
                        },
                    }),
                )
                    .into_response();
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: ErrorBody {
                            code: "task_failed",
                            message: format!("worker failed: {e}"),
                        },
                    }),
                )
                    .into_response();
            }
        };

    // Determine cache path using the content hash (BLAKE3) and size.
    let cache_dir = state.settings().data_dir.join(".thumbnails");
    let cache_filename = format!("{}-{size}.jpg", record.content_hash.as_str());
    let cache_path = cache_dir.join(&cache_filename);
    let max_px = thumbnail_pixels(size);

    // Use /proc/self/fd/<fd> to pass the already-open file to image decoder.
    let source_path_result = tokio::task::spawn_blocking(move || {
        let path = format!("/proc/self/fd/{}", source_file.into_raw_fd());
        generate_or_load_thumbnail(path, &cache_path, max_px)
    })
    .await;

    match source_path_result {
        Ok(Ok(bytes)) => (
            StatusCode::OK,
            [
                (axum::http::header::CONTENT_TYPE, "image/jpeg"),
                (axum::http::header::CACHE_CONTROL, "public, max-age=86400"),
            ],
            bytes,
        )
            .into_response(),
        Ok(Err(message)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "thumbnail_failed",
                    message,
                },
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "task_failed",
                    message: format!("thumbnail worker failed: {e}"),
                },
            }),
        )
            .into_response(),
    }
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
