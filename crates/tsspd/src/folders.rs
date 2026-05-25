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
use serde::{Deserialize, Serialize};

use crate::auth::AuthContext;
use crate::{ErrorBody, ErrorResponse, HttpState};

#[derive(Debug, Deserialize)]
pub struct FolderMoveBody {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Deserialize)]
pub struct FolderCreateBody {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct FolderDeleteBody {
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct FolderEntry {
    pub path: String,
    pub file_count: u64,
}

/// `POST /api/v1/folders` — create a new folder.
pub async fn create_folder(
    auth: AuthContext,
    Json(body): Json<FolderCreateBody>,
) -> Response {
    // Folders are namespaced by the user's files, so all authenticated users can create folders
    let _ = auth;
    let path = normalize_folder_path(&body.path);

    if path.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "invalid_request",
                    message: "folder path cannot be empty".to_owned(),
                },
            }),
        )
            .into_response();
    }

    if let Err(message) = validate_folder_path(&path) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "invalid_request",
                    message: format!("invalid folder path: {message}"),
                },
            }),
        )
            .into_response();
    }

    (
        StatusCode::CREATED,
        Json(FolderEntry {
            path,
            file_count: 0,
        }),
    )
        .into_response()
}

/// `GET /api/v1/folders` — list virtual folder counts for the drive browser.
pub async fn list_folders(
    State(state): State<HttpState>,
    auth: AuthContext,
) -> Response {
    let owner_id = if auth.is_admin() {
        None
    } else {
        Some(&auth.user_id)
    };

    match state.stats_provider.list_folder_counts(owner_id) {
        Ok(folders) => {
            let entries: Vec<FolderEntry> = folders
                .into_iter()
                .map(|(path, file_count)| FolderEntry { path, file_count })
                .collect();

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "schema_version": 1,
                    "folders": entries,
                })),
            )
                .into_response()
        }
        Err(message) => internal(message),
    }
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

    let from = normalize_folder_path(&body.from);
    let to = normalize_folder_path(&body.to);

    if let Err(message) = validate_folder_path(&from) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "invalid_request",
                    message: format!("invalid 'from' path: {message}"),
                },
            }),
        )
            .into_response();
    }

    if let Err(message) = validate_folder_path(&to) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "invalid_request",
                    message: format!("invalid 'to' path: {message}"),
                },
            }),
        )
            .into_response();
    }

    match state.stats_provider.update_folder_path_prefix(&from, &to) {
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

/// `POST /api/v1/folders/delete` — move all files out of a folder (admin).
///
/// Files directly in `path` move to the bucket root. Nested paths under `path`
/// are rewritten so their parent prefix is removed (e.g. `photos/2024` → `2024`).
pub async fn delete_folder(
    State(state): State<HttpState>,
    auth: AuthContext,
    Json(body): Json<FolderDeleteBody>,
) -> Response {
    if !auth.is_admin() {
        return forbidden();
    }

    let path = normalize_folder_path(&body.path);
    if path.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "invalid_request",
                    message: "cannot delete the bucket root".to_owned(),
                },
            }),
        )
            .into_response();
    }

    if let Err(message) = validate_folder_path(&path) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "invalid_request",
                    message: format!("invalid folder path: {message}"),
                },
            }),
        )
            .into_response();
    }

    match state.stats_provider.update_folder_path_prefix(&path, "") {
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

#[cfg(test)]
mod tests {
    use super::{normalize_folder_path, validate_folder_path};

    #[test]
    fn normalize_strips_slashes() {
        assert_eq!(
            normalize_folder_path("/photos/vacation/"),
            "photos/vacation"
        );
    }

    #[test]
    fn validate_rejects_traversal() {
        assert!(validate_folder_path("photos/../secret").is_err());
    }

    #[test]
    fn validate_accepts_nested_paths() {
        assert!(validate_folder_path("archive/2024/q1").is_ok());
    }
}
