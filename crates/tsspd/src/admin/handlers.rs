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

/// Consolidated admin status for the operations dashboard.
#[derive(Debug, Serialize)]
pub struct AdminStatusResponse {
    pub schema_version: u8,
    pub status: &'static str,
    pub version: &'static str,
    pub uptime_seconds: u64,
    pub uptime_hours: u64,
    pub last_restart: String,
    pub disk_used: u64,
    pub disk_total: u64,
    pub memory_used: u64,
    pub memory_total: u64,
    pub cpu_percent: f64,
    pub load_average: f64,
    pub total_files: u64,
    pub total_size: u64,
    pub db_size: u64,
    pub db_status: &'static str,
}

/// `GET /api/v1/admin/status`
pub async fn admin_status(State(state): State<HttpState>) -> impl IntoResponse {
    let repository_stats = state.stats_provider.stats().unwrap_or_default();
    let system = collect_system_snapshot(&state.settings().data_dir).ok();

    let uptime_seconds = state.started_at.elapsed().as_secs();
    let uptime_hours = uptime_seconds / 3600;

    let (disk_total, disk_free) = system
        .as_ref()
        .map_or((0, 0), |s| (s.data_dir_total_bytes, s.data_dir_free_bytes));
    let disk_used = disk_total.saturating_sub(disk_free);

    let (memory_total, memory_available) = system
        .as_ref()
        .map_or((0, 0), |s| (s.total_memory_bytes, s.available_memory_bytes));
    let memory_used = memory_total.saturating_sub(memory_available);

    let db_size = std::fs::metadata(state.settings().data_dir.join("metadata.sqlite3"))
        .map(|m| m.len())
        .unwrap_or(0);

    Json(AdminStatusResponse {
        schema_version: 1,
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        uptime_seconds,
        uptime_hours,
        last_restart: format!("{uptime_seconds} seconds ago"),
        disk_used,
        disk_total,
        memory_used,
        memory_total,
        cpu_percent: 0.0, // Placeholder for now
        load_average: system.as_ref().map_or(0.0, |s| s.load_average_1m),
        total_files: repository_stats.file_count,
        total_size: repository_stats.storage_bytes_used,
        db_size,
        db_status: "ok",
    })
}

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
            corrupt_file_count: state
                .corrupt_file_count
                .load(std::sync::atomic::Ordering::Relaxed),
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
    let corrupt_file_count = state
        .corrupt_file_count
        .load(std::sync::atomic::Ordering::Relaxed);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "schema_version": 1,
            "corrupt_file_count": corrupt_file_count,
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

    // Also cleanup expired chunked upload sessions
    let expired_sessions = state
        .upload_session_manager
        .cleanup_expired(std::time::Duration::from_secs(24 * 60 * 60))
        .await;

    for session_id in expired_sessions {
        let session_dir = state.upload_temp_dir.join(format!(".{}", session_id.as_str()));
        if session_dir.exists() {
            let _ = std::fs::remove_dir_all(session_dir);
        }
    }

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
    match state.stats_provider.list_folder_counts(None) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_status_response_has_schema_version_one() {
        let response = AdminStatusResponse {
            schema_version: 1,
            status: "ok",
            version: "1.0.0",
            uptime_seconds: 3600,
            uptime_hours: 1,
            last_restart: "3600 seconds ago".to_owned(),
            disk_used: 1_000_000,
            disk_total: 10_000_000,
            memory_used: 500_000,
            memory_total: 4_000_000,
            cpu_percent: 25.5,
            load_average: 0.5,
            total_files: 100,
            total_size: 500_0000,
            db_size: 100_000,
            db_status: "ok",
        };
        assert_eq!(response.schema_version, 1);
        assert_eq!(response.status, "ok");
    }

    #[test]
    fn admin_status_response_serializes_to_json() {
        let response = AdminStatusResponse {
            schema_version: 1,
            status: "ok",
            version: "1.0.0",
            uptime_seconds: 3600,
            uptime_hours: 1,
            last_restart: "3600 seconds ago".to_owned(),
            disk_used: 1_000_000,
            disk_total: 10_000_000,
            memory_used: 500_000,
            memory_total: 4_000_000,
            cpu_percent: 25.5,
            load_average: 0.5,
            total_files: 100,
            total_size: 500_0000,
            db_size: 100_000,
            db_status: "ok",
        };
        let json = serde_json::to_string(&response).expect("serialization failed");
        assert!(json.contains("\"schema_version\":1"));
        assert!(json.contains("\"status\":\"ok\""));
    }

    #[test]
    fn admin_overview_response_schema_version() {
        let response = AdminOverviewResponse {
            schema_version: 1,
            version: "1.0.0",
            uptime_seconds: 3600,
            file_count: 50,
            note_count: 20,
            tag_count: 15,
            pinned_count: 5,
            workspace_count: 3,
            corrupt_file_count: 0,
            storage_bytes_used: 1_000_000,
            public_url: Some("https://example.com".to_owned()),
        };
        assert_eq!(response.schema_version, 1);
        assert_eq!(response.file_count, 50);
    }

    #[test]
    fn admin_overview_response_serializes() {
        let response = AdminOverviewResponse {
            schema_version: 1,
            version: "1.0.0",
            uptime_seconds: 3600,
            file_count: 50,
            note_count: 20,
            tag_count: 15,
            pinned_count: 5,
            workspace_count: 3,
            corrupt_file_count: 0,
            storage_bytes_used: 1_000_000,
            public_url: Some("https://example.com".to_owned()),
        };
        let json = serde_json::to_string(&response).expect("serialization failed");
        assert!(json.contains("\"file_count\":50"));
        assert!(json.contains("\"note_count\":20"));
    }

    #[test]
    fn admin_activity_item_with_all_fields() {
        let item = AdminActivityItem {
            kind: "file".to_owned(),
            id: "file123".to_owned(),
            title: "document.pdf".to_owned(),
            detail: "/documents".to_owned(),
            occurred_at: 1_609_459_200,
            visibility: Some("public".to_owned()),
            size_bytes: Some(1_024_000),
            language: None,
        };
        assert_eq!(item.kind, "file");
        assert_eq!(item.size_bytes, Some(1_024_000));
    }

    #[test]
    fn admin_activity_item_serializes_visibility() {
        let item = AdminActivityItem {
            kind: "file".to_owned(),
            id: "file123".to_owned(),
            title: "document.pdf".to_owned(),
            detail: "/documents".to_owned(),
            occurred_at: 1_609_459_200,
            visibility: Some("public".to_owned()),
            size_bytes: Some(1_024_000),
            language: None,
        };
        let json = serde_json::to_string(&item).expect("serialization failed");
        assert!(json.contains("\"visibility\":\"public\""));
    }

    #[test]
    fn admin_activity_item_skips_none_visibility() {
        let item = AdminActivityItem {
            kind: "note".to_owned(),
            id: "note456".to_owned(),
            title: "My Note".to_owned(),
            detail: "my-tag, other-tag".to_owned(),
            occurred_at: 1_609_459_200,
            visibility: None,
            size_bytes: None,
            language: None,
        };
        let json = serde_json::to_string(&item).expect("serialization failed");
        assert!(!json.contains("\"visibility\""));
    }

    #[test]
    fn admin_activity_response_contains_items() {
        let response = AdminActivityResponse {
            schema_version: 1,
            items: vec![
                AdminActivityItem {
                    kind: "file".to_owned(),
                    id: "id1".to_owned(),
                    title: "file1".to_owned(),
                    detail: "detail1".to_owned(),
                    occurred_at: 100,
                    visibility: None,
                    size_bytes: None,
                    language: None,
                },
                AdminActivityItem {
                    kind: "note".to_owned(),
                    id: "id2".to_owned(),
                    title: "note1".to_owned(),
                    detail: "detail2".to_owned(),
                    occurred_at: 200,
                    visibility: None,
                    size_bytes: None,
                    language: None,
                },
            ],
        };
        assert_eq!(response.items.len(), 2);
        assert_eq!(response.schema_version, 1);
    }

    #[test]
    fn folder_entry_serializes() {
        let entry = FolderEntry {
            path: "/documents".to_owned(),
            file_count: 42,
        };
        let json = serde_json::to_string(&entry).expect("serialization failed");
        assert!(json.contains("/documents"));
        assert!(json.contains("42"));
    }

    #[test]
    fn admin_files_query_default_limit() {
        let json = r"{}";
        let query: AdminFilesQuery = serde_json::from_str(json).expect("deserialize");
        assert_eq!(query.limit, 100);
    }

    #[test]
    fn admin_files_query_custom_limit() {
        let json = r#"{"limit": 50}"#;
        let query: AdminFilesQuery = serde_json::from_str(json).expect("deserialize");
        assert_eq!(query.limit, 50);
    }

    #[test]
    fn admin_files_query_with_folder_and_type() {
        let json = r#"{"limit": 25, "folder": "/docs", "type": "image"}"#;
        let query: AdminFilesQuery = serde_json::from_str(json).expect("deserialize");
        assert_eq!(query.limit, 25);
        assert_eq!(query.folder, Some("/docs".to_owned()));
        assert_eq!(query.mime_prefix, Some("image".to_owned()));
    }

    #[test]
    fn admin_files_query_folder_optional() {
        let json = r#"{"limit": 100}"#;
        let query: AdminFilesQuery = serde_json::from_str(json).expect("deserialize");
        assert_eq!(query.folder, None);
    }

    #[test]
    fn admin_activity_query_default_limit() {
        let json = r"{}";
        let query: AdminActivityQuery = serde_json::from_str(json).expect("deserialize");
        assert_eq!(query.limit, 100);
    }

    #[test]
    fn admin_activity_query_custom_limit() {
        let json = r#"{"limit": 75}"#;
        let query: AdminActivityQuery = serde_json::from_str(json).expect("deserialize");
        assert_eq!(query.limit, 75);
    }

    #[test]
    fn default_limit_returns_100() {
        assert_eq!(default_limit(), 100);
    }

    #[test]
    fn admin_activity_error_response_status_code() {
        let response = admin_activity_error("test_error", "test message".to_owned());
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn admin_activity_error_includes_code_and_message() {
        let error_response = ErrorResponse {
            error: ErrorBody {
                code: "test_error",
                message: "test message".to_owned(),
            },
        };
        assert_eq!(error_response.error.code, "test_error");
        assert_eq!(error_response.error.message, "test message");
    }

    #[test]
    fn folder_entry_path_and_count() {
        let entry = FolderEntry {
            path: "/my/folder".to_owned(),
            file_count: 123,
        };
        assert_eq!(entry.path, "/my/folder");
        assert_eq!(entry.file_count, 123);
    }

    #[test]
    fn admin_status_response_uptime_hours_calculation() {
        let response = AdminStatusResponse {
            schema_version: 1,
            status: "ok",
            version: "1.0.0",
            uptime_seconds: 7200,
            uptime_hours: 2,
            last_restart: "7200 seconds ago".to_owned(),
            disk_used: 0,
            disk_total: 0,
            memory_used: 0,
            memory_total: 0,
            cpu_percent: 0.0,
            load_average: 0.0,
            total_files: 0,
            total_size: 0,
            db_size: 0,
            db_status: "ok",
        };
        assert_eq!(response.uptime_hours, 7200 / 3600);
    }

    #[test]
    fn admin_activity_item_with_language_field() {
        let item = AdminActivityItem {
            kind: "workspace".to_owned(),
            id: "ws1".to_owned(),
            title: "Python Workspace".to_owned(),
            detail: "python".to_owned(),
            occurred_at: 1_609_459_200,
            visibility: None,
            size_bytes: None,
            language: Some("python".to_owned()),
        };
        assert_eq!(item.language, Some("python".to_owned()));
    }

    #[test]
    fn admin_activity_response_serializes_items() {
        let response = AdminActivityResponse {
            schema_version: 1,
            items: vec![
                AdminActivityItem {
                    kind: "file".to_owned(),
                    id: "id1".to_owned(),
                    title: "title1".to_owned(),
                    detail: "detail1".to_owned(),
                    occurred_at: 100,
                    visibility: Some("public".to_owned()),
                    size_bytes: Some(500),
                    language: None,
                },
            ],
        };
        let json = serde_json::to_string(&response).expect("serialization failed");
        assert!(json.contains("\"schema_version\":1"));
        assert!(json.contains("\"items\""));
    }

    #[test]
    fn admin_files_query_zero_limit_allowed() {
        let json = r#"{"limit": 0}"#;
        let query: AdminFilesQuery = serde_json::from_str(json).expect("deserialize");
        assert_eq!(query.limit, 0);
    }

    #[test]
    fn admin_activity_query_zero_limit_allowed() {
        let json = r#"{"limit": 0}"#;
        let query: AdminActivityQuery = serde_json::from_str(json).expect("deserialize");
        assert_eq!(query.limit, 0);
    }

    #[test]
    fn admin_status_response_cpu_and_load() {
        let response = AdminStatusResponse {
            schema_version: 1,
            status: "ok",
            version: "1.0.0",
            uptime_seconds: 3600,
            uptime_hours: 1,
            last_restart: "3600 seconds ago".to_owned(),
            disk_used: 0,
            disk_total: 0,
            memory_used: 0,
            memory_total: 0,
            cpu_percent: 45.25,
            load_average: 1.5,
            total_files: 0,
            total_size: 0,
            db_size: 0,
            db_status: "ok",
        };
        assert!(response.cpu_percent > 0.0);
        assert!(response.load_average > 0.0);
    }

    #[test]
    fn admin_overview_public_url_optional() {
        let response_with_url = AdminOverviewResponse {
            schema_version: 1,
            version: "1.0.0",
            uptime_seconds: 3600,
            file_count: 0,
            note_count: 0,
            tag_count: 0,
            pinned_count: 0,
            workspace_count: 0,
            corrupt_file_count: 0,
            storage_bytes_used: 0,
            public_url: Some("https://example.com".to_owned()),
        };
        assert!(response_with_url.public_url.is_some());

        let response_without_url = AdminOverviewResponse {
            schema_version: 1,
            version: "1.0.0",
            uptime_seconds: 3600,
            file_count: 0,
            note_count: 0,
            tag_count: 0,
            pinned_count: 0,
            workspace_count: 0,
            corrupt_file_count: 0,
            storage_bytes_used: 0,
            public_url: None,
        };
        assert!(response_without_url.public_url.is_none());
    }

    #[test]
    fn admin_activity_item_occurred_at_timestamp() {
        let item = AdminActivityItem {
            kind: "file".to_owned(),
            id: "test".to_owned(),
            title: "test".to_owned(),
            detail: "test".to_owned(),
            occurred_at: 1_609_459_200,
            visibility: None,
            size_bytes: None,
            language: None,
        };
        assert_eq!(item.occurred_at, 1_609_459_200);
    }

    #[test]
    fn folder_entry_empty_path() {
        let entry = FolderEntry {
            path: String::new(),
            file_count: 0,
        };
        assert_eq!(entry.path, "");
        assert_eq!(entry.file_count, 0);
    }

    #[test]
    fn admin_files_query_deserialize_image_type() {
        let json = r#"{"type": "image"}"#;
        let query: AdminFilesQuery = serde_json::from_str(json).expect("deserialize");
        assert_eq!(query.mime_prefix, Some("image".to_owned()));
    }

    #[test]
    fn admin_status_response_large_values() {
        let response = AdminStatusResponse {
            schema_version: 1,
            status: "ok",
            version: "1.0.0",
            uptime_seconds: u64::MAX - 1,
            uptime_hours: (u64::MAX - 1) / 3600,
            last_restart: "very long ago".to_owned(),
            disk_used: u64::MAX / 2,
            disk_total: u64::MAX,
            memory_used: u64::MAX / 4,
            memory_total: u64::MAX / 2,
            cpu_percent: 100.0,
            load_average: 10.0,
            total_files: u64::MAX / 8,
            total_size: u64::MAX / 3,
            db_size: u64::MAX / 100,
            db_status: "ok",
        };
        assert_eq!(response.uptime_seconds, u64::MAX - 1);
        assert!(response.disk_total > response.disk_used);
    }
}
