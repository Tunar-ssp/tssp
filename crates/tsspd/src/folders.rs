//! Folder management delivery.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use tssp_app::{FolderError, FolderService};
use tssp_ports::FileRepository;

use crate::auth::AuthContext;
use crate::{ErrorBody, ErrorResponse, HttpState};

/// Handles folder operations through the application layer.
pub trait FolderProvider: Send + Sync {
    /// Renames or moves a virtual folder.
    ///
    /// # Errors
    ///
    /// Returns [`HttpFolderError`] when the path is invalid or the operation fails.
    fn move_folder(&self, from: &str, to: &str) -> Result<u64, HttpFolderError>;
    /// Moves all files out of a virtual folder.
    ///
    /// # Errors
    ///
    /// Returns [`HttpFolderError`] when the path is invalid or the operation fails.
    fn delete_folder(&self, path: &str) -> Result<u64, HttpFolderError>;
    /// Lists virtual folders and their file counts.
    ///
    /// # Errors
    ///
    /// Returns [`HttpFolderError`] when the query fails.
    fn list_folders(
        &self,
        owner_id: Option<&tssp_domain::UserId>,
    ) -> Result<Vec<(String, u64)>, HttpFolderError>;
}

/// Folder provider backed by the core application folder service.
pub struct ApplicationFolderProvider<R> {
    service: FolderService<R>,
}

impl<R> ApplicationFolderProvider<R> {
    /// Creates a new folder provider.
    pub const fn new(service: FolderService<R>) -> Self {
        Self { service }
    }
}

impl<R> FolderProvider for ApplicationFolderProvider<R>
where
    R: FileRepository + Send + Sync,
{
    fn move_folder(&self, from: &str, to: &str) -> Result<u64, HttpFolderError> {
        self.service.move_folder(from, to).map_err(map_folder_error)
    }

    fn delete_folder(&self, path: &str) -> Result<u64, HttpFolderError> {
        self.service.delete_folder(path).map_err(map_folder_error)
    }

    fn list_folders(
        &self,
        owner_id: Option<&tssp_domain::UserId>,
    ) -> Result<Vec<(String, u64)>, HttpFolderError> {
        self.service
            .list_folders(owner_id)
            .map_err(map_folder_error)
    }
}

/// Request body for moving a virtual folder.
#[derive(Debug, Deserialize)]
pub struct FolderMoveBody {
    /// Original folder path.
    pub from: String,
    /// New folder path.
    pub to: String,
}

/// Request body for creating a new virtual folder.
#[derive(Debug, Deserialize)]
pub struct FolderCreateBody {
    /// Virtual folder path.
    pub path: String,
}

/// Request body for deleting a virtual folder.
#[derive(Debug, Deserialize)]
pub struct FolderDeleteBody {
    /// Virtual folder path to remove.
    pub path: String,
}

/// Logical folder entry with file count.
#[derive(Debug, Serialize)]
pub struct FolderEntry {
    /// Normalized folder path.
    pub path: String,
    /// Number of files within the folder.
    pub file_count: u64,
}

/// `POST /api/v1/folders` — create a new folder.
pub async fn create_folder(auth: AuthContext, Json(body): Json<FolderCreateBody>) -> Response {
    let _ = auth;
    let path = tssp_app::normalize_folder_path(&body.path);

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

    if let Err(message) = tssp_app::validate_folder_path(&path) {
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
pub async fn list_folders(State(state): State<HttpState>, auth: AuthContext) -> Response {
    let owner_id = if auth.is_admin() {
        None
    } else {
        Some(&auth.user_id)
    };

    match state.folder_provider.list_folders(owner_id) {
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
        Err(error) => error.response(),
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

    match state.folder_provider.move_folder(&body.from, &body.to) {
        Ok(count) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "schema_version": 1,
                "files_updated": count,
            })),
        )
            .into_response(),
        Err(error) => error.response(),
    }
}

/// `POST /api/v1/folders/delete` — move all files out of a folder (admin).
pub async fn delete_folder(
    State(state): State<HttpState>,
    auth: AuthContext,
    Json(body): Json<FolderDeleteBody>,
) -> Response {
    if !auth.is_admin() {
        return forbidden();
    }

    match state.folder_provider.delete_folder(&body.path) {
        Ok(count) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "schema_version": 1,
                "files_updated": count,
            })),
        )
            .into_response(),
        Err(error) => error.response(),
    }
}

/// Errors returned by the folder provider mapped to HTTP.
#[derive(Debug)]
pub enum HttpFolderError {
    /// Provided folder path was invalid.
    InvalidPath(String),
    /// Internal server error occurred.
    Internal(String),
    /// User does not have permission.
    Forbidden,
}

impl HttpFolderError {
    fn response(&self) -> Response {
        match self {
            Self::InvalidPath(message) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "invalid_request",
                        message: message.clone(),
                    },
                }),
            )
                .into_response(),
            Self::Internal(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "internal_error",
                        message: message.clone(),
                    },
                }),
            )
                .into_response(),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "forbidden",
                        message: "admin role required".to_owned(),
                    },
                }),
            )
                .into_response(),
        }
    }
}

fn map_folder_error(error: FolderError) -> HttpFolderError {
    match error {
        FolderError::InvalidPath(message) => HttpFolderError::InvalidPath(message.to_owned()),
        FolderError::Repository(error) => HttpFolderError::Internal(error.to_string()),
        FolderError::Forbidden => HttpFolderError::Forbidden,
    }
}

fn forbidden() -> Response {
    HttpFolderError::Forbidden.response()
}
