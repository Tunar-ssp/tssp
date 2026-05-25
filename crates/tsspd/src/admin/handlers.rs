//! HTTP handlers for `/api/v1/admin/*`.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use tssp_adapter_system::SystemClock;
use tssp_ports::Clock;
use tssp_ports::{ListQuery, NoteListQuery};

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
    pub workspace_count: u64,
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

#[derive(Debug, Deserialize)]
pub struct AdminActivityQuery {
    #[serde(default = "default_limit")]
    pub limit: u64,
}

#[derive(Debug, Serialize)]
pub struct AdminActivityResponse {
    pub schema_version: u8,
    pub items: Vec<AdminActivityItem>,
}

#[derive(Debug, Serialize)]
pub struct AdminActivityItem {
    pub kind: String,
    pub id: String,
    pub title: String,
    pub detail: String,
    pub occurred_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

fn admin_activity_error(code: &'static str, message: String) -> axum::response::Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: ErrorBody { code, message },
        }),
    )
        .into_response()
}

fn collect_file_activity(
    state: &HttpState,
    limit: u64,
) -> Result<Vec<AdminActivityItem>, (&'static str, String)> {
    state
        .stats_provider
        .list_files(&ListQuery {
            limit,
            ..ListQuery::default()
        })
        .map(|page| {
            page.files
                .into_iter()
                .map(|file| AdminActivityItem {
                    kind: "file".to_owned(),
                    id: file.id.as_str().to_owned(),
                    title: file.name.original().to_owned(),
                    detail: if file.folder_path.is_empty() {
                        "Bucket root".to_owned()
                    } else {
                        file.folder_path.clone()
                    },
                    occurred_at: file.uploaded_at.seconds(),
                    visibility: Some(file.visibility.as_str().to_owned()),
                    size_bytes: Some(file.size.bytes()),
                    language: None,
                })
                .collect()
        })
        .map_err(|message| ("admin_activity_files_failed", message))
}

fn collect_note_activity(
    state: &HttpState,
    limit: u64,
) -> Result<Vec<AdminActivityItem>, (&'static str, String)> {
    state
        .note_provider
        .list_notes(NoteListQuery {
            limit,
            ..NoteListQuery::default()
        })
        .map(|page| {
            page.notes
                .into_iter()
                .map(|note| AdminActivityItem {
                    kind: "note".to_owned(),
                    id: note.id.as_str().to_owned(),
                    title: note.title.as_str().to_owned(),
                    detail: if note.tags.is_empty() {
                        "Updated note".to_owned()
                    } else {
                        note.tags
                            .iter()
                            .map(|tag| tag.display().to_owned())
                            .collect::<Vec<_>>()
                            .join(", ")
                    },
                    occurred_at: note.updated_at.seconds(),
                    visibility: None,
                    size_bytes: None,
                    language: None,
                })
                .collect()
        })
        .map_err(|error| {
            (
                "admin_activity_notes_failed",
                format!("note activity query failed: {error:?}"),
            )
        })
}

fn collect_workspace_activity(state: &HttpState) -> Vec<AdminActivityItem> {
    state
        .workspaces
        .as_deref()
        .and_then(|store| store.list_all().ok())
        .unwrap_or_default()
        .into_iter()
        .map(|workspace| AdminActivityItem {
            kind: "workspace".to_owned(),
            id: workspace.id.clone(),
            title: workspace.name.clone(),
            detail: workspace.language.clone(),
            occurred_at: workspace.updated_at,
            visibility: None,
            size_bytes: None,
            language: Some(workspace.language),
        })
        .collect()
}

fn default_limit() -> u64 {
    100
}

/// `GET /api/v1/admin/overview`
pub async fn admin_overview(State(state): State<HttpState>) -> impl IntoResponse {
    let repository_stats = match state.stats_provider.stats() {
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

    let workspace_count = state
        .workspaces
        .as_deref()
        .and_then(|store| store.list_all().ok())
        .map_or(0, |ws| ws.len() as u64);

    (
        StatusCode::OK,
        Json(AdminOverviewResponse {
            schema_version: 1,
            version: env!("CARGO_PKG_VERSION"),
            uptime_seconds: state.started_at.elapsed().as_secs(),
            file_count: repository_stats.file_count,
            note_count: repository_stats.note_count,
            tag_count: repository_stats.tag_count,
            pinned_count: repository_stats.pinned_count,
            workspace_count,
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

/// `GET /api/v1/admin/activity`
pub async fn admin_activity(
    State(state): State<HttpState>,
    Query(params): Query<AdminActivityQuery>,
) -> impl IntoResponse {
    let limit = params.limit.clamp(1, 100);
    let mut items = match collect_file_activity(&state, limit) {
        Ok(items) => items,
        Err((code, message)) => return admin_activity_error(code, message),
    };
    let note_items = match collect_note_activity(&state, limit) {
        Ok(items) => items,
        Err((code, message)) => return admin_activity_error(code, message),
    };
    items.extend(note_items);
    items.extend(collect_workspace_activity(&state));

    items.sort_by(|left, right| {
        right
            .occurred_at
            .cmp(&left.occurred_at)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.id.cmp(&right.id))
    });
    items.truncate(limit as usize);

    (
        StatusCode::OK,
        Json(AdminActivityResponse {
            schema_version: 1,
            items,
        }),
    )
        .into_response()
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
    let report = tokio::task::spawn_blocking(move || cleanup_dir_files(&dir))
        .await
        .unwrap_or_default();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "schema_version": 1,
            "removed": report.total_removed(),
            "files_removed": report.files_removed,
            "directories_removed": report.directories_removed,
            "errors": report.errors,
            "minimum_age_seconds": 2 * 60 * 60,
        })),
    )
        .into_response()
}

/// `POST /api/v1/admin/cleanup/sessions`
pub async fn admin_cleanup_sessions(State(state): State<HttpState>) -> impl IntoResponse {
    match state.auth.cleanup_expired(SystemClock.now().seconds()) {
        Ok((sessions_removed, devices_removed)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "schema_version": 1,
                "sessions_removed": sessions_removed,
                "devices_removed": devices_removed,
                "message": format!(
                    "removed {sessions_removed} expired session(s) and {devices_removed} expired device trust record(s)"
                ),
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "cleanup_sessions_failed",
                    message: error.to_string(),
                },
            }),
        )
            .into_response(),
    }
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

fn cleanup_dir_files(dir: &std::path::Path) -> crate::temp_cleanup::TempCleanupReport {
    crate::temp_cleanup::cleanup_temp_upload_dir(
        dir,
        Some(std::time::Duration::from_secs(2 * 60 * 60)),
    )
}
