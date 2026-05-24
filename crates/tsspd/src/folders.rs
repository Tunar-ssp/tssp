//! Logical folder rename/move/delete.

const MAX_FOLDER_PATH_BYTES: usize = 1024;

/// Normalizes a folder path for storage (no leading/trailing slashes).
#[must_use]
pub fn normalize_folder_path(value: &str) -> String {
    value.trim().trim_matches('/').replace('\\', "/")
}

/// Validates a normalized folder path.
///
/// Returns `Err` with a human-readable message when the path contains null bytes,
/// `..` traversal components, or exceeds the length limit.
///
/// # Errors
///
/// Returns `Err(&'static str)` when the path fails validation.
pub fn validate_folder_path(path: &str) -> Result<(), &'static str> {
    if path.contains('\0') {
        return Err("folder path must not contain null bytes");
    }
    if path.len() > MAX_FOLDER_PATH_BYTES {
        return Err("folder path is too long (max 1024 bytes)");
    }
    // Reject any component that is exactly ".." after splitting on "/"
    for component in path.split('/') {
        if component == ".." {
            return Err("folder path must not contain '..' components");
        }
    }
    Ok(())
}

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use crate::auth::AuthContext;
use crate::{ErrorBody, ErrorResponse, HttpState};

#[derive(Debug, Deserialize)]
pub struct FolderMoveBody {
    pub from: String,
    pub to: String,
}

/// `POST /api/v1/folders/move` — rewrite `folder_path` prefixes (admin).
pub async fn move_folder(
    State(state): State<HttpState>,
    auth: AuthContext,
    Json(body): Json<FolderMoveBody>,
) -> Response {
    if !auth.is_admin() {
        return forbidden();
    }
    match state
        .stats_provider
        .update_folder_path_prefix(&body.from, &body.to)
    {
        Ok(count) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "schema_version": 1,
                "files_updated": count,
            })),
        )
            .into_response(),
        Err(message) => internal(message),
    }
}

fn forbidden() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "forbidden",
                message: "admin role required".to_owned(),
            },
        }),
    )
        .into_response()
}

fn internal(message: String) -> Response {
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
