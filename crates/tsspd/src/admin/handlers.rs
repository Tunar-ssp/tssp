//! HTTP handlers for `/api/v1/admin/*`.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use tssp_ports::ListQuery;

use crate::admin::system::collect_system_snapshot;
use crate::upload::FileRecordResponse;
use crate::{ErrorBody, ErrorResponse, HttpState};

/// Admin dashboard overview.
#[derive(Debug, Serialize)]
pub struct AdminOverviewResponse {
    pub schema_version: u8,
    pub version: &'static str,
    pub uptime_seconds: u64,
    pub file_count: u64,
    pub note_count: u64,
    pub tag_count: u64,
    pub pinned_count: u64,
    pub corrupt_file_count: u64,
    pub storage_bytes_used: u64,
    pub public_url: Option<String>,
}

/// Folder entry for virtual directory browser.
#[derive(Debug, Serialize)]
pub struct FolderEntry {
    pub path: String,
    pub file_count: u64,
}

#[derive(Debug, Deserialize)]
pub struct AdminFilesQuery {
    #[serde(default = "default_limit")]
    pub limit: u64,
    #[serde(default)]
    pub folder: Option<String>,
    #[serde(default, rename = "type")]
    pub mime_prefix: Option<String>,
}

fn default_limit() -> u64 {
    100
}

/// `GET /api/v1/admin/overview`
pub async fn admin_overview(State(state): State<HttpState>) -> impl IntoResponse {
    let stats = match state.stats_provider.stats() {
        Ok(value) => value,
        Err(message) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "admin_stats_failed",
                        message,
                    },
                }),
            )
                .into_response();
        }
    };
    let storage_bytes_used = tokio::task::spawn_blocking({
        let dir = state.upload_temp_dir.clone();
        move || crate::status::calculate_directory_size(dir.parent().unwrap_or(&dir))
    })
    .await
    .unwrap_or(0);

    (
        StatusCode::OK,
        Json(AdminOverviewResponse {
            schema_version: 1,
            version: env!("CARGO_PKG_VERSION"),
            uptime_seconds: state.started_at.elapsed().as_secs(),
            file_count: stats.file_count,
            note_count: stats.note_count,
            tag_count: stats.tag_count,
            pinned_count: stats.pinned_count,
            corrupt_file_count: state.corrupt_file_count,
            storage_bytes_used,
            public_url: Some(state.public_urls().base().to_owned()),
        }),
    )
        .into_response()
}

/// `GET /api/v1/admin/system`
pub async fn admin_system(State(state): State<HttpState>) -> impl IntoResponse {
    match collect_system_snapshot(&state.settings().data_dir) {
        Ok(snapshot) => (StatusCode::OK, Json(snapshot)).into_response(),
        Err(message) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "system_snapshot_failed",
                    message,
                },
            }),
        )
            .into_response(),
    }
}

/// `GET /api/v1/admin/files`
pub async fn admin_list_files(
    State(state): State<HttpState>,
    Query(params): Query<AdminFilesQuery>,
) -> impl IntoResponse {
    let limit = params.limit.clamp(1, 500);
    let mut query = ListQuery {
        limit,
        mime_prefix: params.mime_prefix.clone(),
        folder_prefix: params.folder.clone(),
        ..ListQuery::default()
    };
    if params.mime_prefix.as_deref() == Some("image") {
        query.mime_prefix = Some("image/".to_owned());
    }
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
                    "next_cursor": page.next_cursor.map(|c| c.as_str().to_owned()),
                })),
            )
                .into_response()
        }
        Err(message) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "admin_list_failed",
                    message,
                },
            }),
        )
            .into_response(),
    }
}

/// `DELETE /api/v1/admin/files/{id}`
pub async fn admin_delete_file(
    State(state): State<HttpState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let file_id = match tssp_domain::FileId::new(&id) {
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
    match state.delete_provider.delete(file_id) {
        Ok(outcome) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "schema_version": 1,
                "existed": outcome.existed,
                "blob_cleaned": outcome.blob_cleaned,
            })),
        )
            .into_response(),
        Err(error) => error.response(),
    }
}

/// `GET /api/v1/admin/corrupt`
pub async fn admin_corrupt_files(State(state): State<HttpState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "schema_version": 1,
            "corrupt_file_count": state.corrupt_file_count,
        })),
    )
        .into_response()
}

/// `POST /api/v1/admin/cleanup/temp`
pub async fn admin_cleanup_temp(State(state): State<HttpState>) -> impl IntoResponse {
    let dir = state.upload_temp_dir.clone();
    let removed = tokio::task::spawn_blocking(move || cleanup_dir_files(&dir))
        .await
        .unwrap_or(0);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "schema_version": 1,
            "removed": removed,
        })),
    )
        .into_response()
}

/// `POST /api/v1/admin/cleanup/sessions`
pub async fn admin_cleanup_sessions(State(_state): State<HttpState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "schema_version": 1,
            "message": "session cleanup runs on daemon startup",
        })),
    )
        .into_response()
}

/// `GET /api/v1/admin/folders`
pub async fn admin_folders(State(state): State<HttpState>) -> impl IntoResponse {
    match state.stats_provider.list_folder_counts() {
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
        }
        .into_response(),
        Err(message) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "folders_failed",
                    message,
                },
            }),
        )
            .into_response(),
    }
}

fn cleanup_dir_files(dir: &std::path::Path) -> u64 {
    let mut removed = 0_u64;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().is_file() && std::fs::remove_file(entry.path()).is_ok() {
                removed += 1;
            }
        }
    }
    removed
}
