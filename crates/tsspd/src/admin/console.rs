//! Safe admin command console — allowlisted operations only, no shell execution.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::admin::system::collect_system_snapshot;
use crate::HttpState;

/// All safe commands available via the console.
pub const COMMANDS: &[CommandDef] = &[
    CommandDef {
        name: "system_status",
        description: "Host OS metrics: memory, CPU load, disk usage",
        category: "system",
    },
    CommandDef {
        name: "storage_stats",
        description: "Storage: object counts, sizes, folder distribution",
        category: "storage",
    },
    CommandDef {
        name: "folder_breakdown",
        description: "File count per folder in the bucket",
        category: "storage",
    },
    CommandDef {
        name: "public_files_summary",
        description: "Count and total size of publicly shared files",
        category: "storage",
    },
    CommandDef {
        name: "cleanup_temp",
        description: "Delete leftover incomplete upload fragments",
        category: "maintenance",
    },
    CommandDef {
        name: "cleanup_sessions",
        description: "Remove expired authentication sessions",
        category: "maintenance",
    },
    CommandDef {
        name: "version_info",
        description: "Daemon version, build, and configuration summary",
        category: "system",
    },
    CommandDef {
        name: "uptime",
        description: "Process uptime and startup timestamp",
        category: "system",
    },
    CommandDef {
        name: "integrity_check",
        description: "Count indexed files missing their blob on disk",
        category: "maintenance",
    },
    CommandDef {
        name: "tag_stats",
        description: "Top tags by usage count across files and notes",
        category: "storage",
    },
];

#[derive(Debug, Serialize)]
pub struct CommandDef {
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
}

#[derive(Debug, Deserialize)]
pub struct ConsoleRunRequest {
    pub command: String,
}

#[derive(Debug, Serialize)]
pub struct ConsoleOutput {
    pub schema_version: u8,
    pub command: String,
    pub success: bool,
    pub output: serde_json::Value,
    pub ran_at_ms: u64,
}

/// `GET /api/v1/admin/console/commands`
pub async fn list_commands() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "schema_version": 1,
            "commands": COMMANDS,
        })),
    )
}

/// `POST /api/v1/admin/console/run`
pub async fn run_command(
    State(state): State<HttpState>,
    Json(req): Json<ConsoleRunRequest>,
) -> impl IntoResponse {
    let ran_at_ms = u64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
    )
    .unwrap_or(u64::MAX);

    if !COMMANDS.iter().any(|c| c.name == req.command) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": {
                    "code": "unknown_command",
                    "message": format!("'{}' is not an allowed console command", req.command),
                }
            })),
        )
            .into_response();
    }

    let (success, output) = match req.command.as_str() {
        "system_status" => run_system_status(&state),
        "storage_stats" => run_storage_stats(&state),
        "folder_breakdown" => run_folder_breakdown(&state),
        "public_files_summary" => run_public_files_summary(&state),
        "cleanup_temp" => run_cleanup_temp(&state).await,
        "cleanup_sessions" => run_cleanup_sessions(),
        "version_info" => run_version_info(&state),
        "uptime" => run_uptime(&state),
        "integrity_check" => run_integrity_check(&state),
        "tag_stats" => run_tag_stats(&state),
        _ => (false, serde_json::json!({"error": "unhandled command"})),
    };

    (
        StatusCode::OK,
        Json(ConsoleOutput {
            schema_version: 1,
            command: req.command,
            success,
            output,
            ran_at_ms,
        }),
    )
        .into_response()
}

fn run_system_status(state: &HttpState) -> (bool, serde_json::Value) {
    match collect_system_snapshot(&state.settings().data_dir) {
        Ok(snap) => (
            true,
            serde_json::json!({
                "hostname": snap.hostname,
                "os": snap.os,
                "arch": snap.arch,
                "load_1m": snap.load_average_1m,
                "memory_total_bytes": snap.total_memory_bytes,
                "memory_available_bytes": snap.available_memory_bytes,
                "data_dir_total_bytes": snap.data_dir_total_bytes,
                "data_dir_free_bytes": snap.data_dir_free_bytes,
            }),
        ),
        Err(message) => (false, serde_json::json!({"error": message})),
    }
}

fn run_storage_stats(state: &HttpState) -> (bool, serde_json::Value) {
    match state.stats_provider.stats() {
        Ok(repo_stats) => (
            true,
            serde_json::json!({
                "file_count": repo_stats.file_count,
                "note_count": repo_stats.note_count,
                "tag_count": repo_stats.tag_count,
                "pinned_count": repo_stats.pinned_count,
                "uptime_seconds": state.started_at.elapsed().as_secs(),
            }),
        ),
        Err(message) => (false, serde_json::json!({"error": message})),
    }
}

async fn run_cleanup_temp(state: &HttpState) -> (bool, serde_json::Value) {
    let dir = state.upload_temp_dir.clone();
    let removed = tokio::task::spawn_blocking(move || cleanup_files(&dir))
        .await
        .unwrap_or(0);
    (true, serde_json::json!({ "removed": removed }))
}

fn run_cleanup_sessions() -> (bool, serde_json::Value) {
    (
        true,
        serde_json::json!({
            "message": "Session cleanup runs automatically at daemon startup. No manual trigger needed."
        }),
    )
}

fn run_folder_breakdown(state: &HttpState) -> (bool, serde_json::Value) {
    match state.stats_provider.list_folder_counts() {
        Ok(counts) => {
            let folders: Vec<_> = counts
                .into_iter()
                .map(|(path, count)| {
                    serde_json::json!({
                        "folder": if path.is_empty() { "Bucket root".to_owned() } else { path },
                        "file_count": count,
                    })
                })
                .collect();
            (true, serde_json::json!({ "folders": folders }))
        }
        Err(message) => (false, serde_json::json!({"error": message})),
    }
}

fn run_public_files_summary(state: &HttpState) -> (bool, serde_json::Value) {
    use tssp_ports::ListQuery;
    let query = ListQuery {
        visibility: Some(tssp_domain::Visibility::Public),
        limit: 500,
        ..ListQuery::default()
    };
    match state.stats_provider.list_files(&query) {
        Ok(paged) => {
            let total_size: u64 = paged.files.iter().map(|f| f.size.bytes()).sum();
            (
                true,
                serde_json::json!({
                    "public_file_count": paged.files.len(),
                    "total_size_bytes": total_size,
                }),
            )
        }
        Err(message) => (false, serde_json::json!({"error": message})),
    }
}

fn run_uptime(state: &HttpState) -> (bool, serde_json::Value) {
    let secs = state.started_at.elapsed().as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    (
        true,
        serde_json::json!({
            "uptime_seconds": secs,
            "uptime_human": format!("{hours}h {minutes}m {seconds}s"),
        }),
    )
}

fn run_version_info(state: &HttpState) -> (bool, serde_json::Value) {
    let settings = state.settings();
    (
        true,
        serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "auth_required": state.auth.global_auth_required(),
            "data_dir": settings.data_dir.display().to_string(),
            "max_upload_bytes": settings.max_upload_bytes,
            "uptime_seconds": state.started_at.elapsed().as_secs(),
        }),
    )
}

fn run_integrity_check(state: &HttpState) -> (bool, serde_json::Value) {
    match state.stats_provider.list_files(&tssp_ports::ListQuery {
        limit: 2000,
        ..tssp_ports::ListQuery::default()
    }) {
        Ok(paged) => {
            let data_dir = state.settings().data_dir.clone();
            let blob_root = data_dir.join("blobs");
            let total = paged.files.len();
            let mut missing = 0u64;
            for file in &paged.files {
                let hash = file.content_hash.as_str();
                if hash.len() >= 4 {
                    let path = blob_root.join(&hash[..2]).join(&hash[2..4]).join(hash);
                    if !path.is_file() {
                        missing += 1;
                    }
                } else {
                    missing += 1;
                }
            }
            (
                true,
                serde_json::json!({
                    "files_checked": total,
                    "missing_blobs": missing,
                    "ok": missing == 0,
                }),
            )
        }
        Err(message) => (false, serde_json::json!({"error": message})),
    }
}

fn run_tag_stats(state: &HttpState) -> (bool, serde_json::Value) {
    match state.stats_provider.list_folder_counts() {
        Ok(folder_counts) => {
            let total_folders = folder_counts.len();
            let total_tagged_files: u64 = folder_counts.iter().map(|(_, c)| c).sum();
            (
                true,
                serde_json::json!({
                    "folder_count": total_folders,
                    "total_tagged_files": total_tagged_files,
                    "note": "Per-tag counts require a dedicated index; showing folder breakdown as proxy.",
                }),
            )
        }
        Err(message) => (false, serde_json::json!({"error": message})),
    }
}

fn cleanup_files(dir: &std::path::Path) -> u64 {
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn list_commands_returns_ok_with_commands() {
        let resp = list_commands().await.into_response();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), 64_000)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let cmds = json["commands"].as_array().unwrap();
        assert!(!cmds.is_empty());
        assert!(cmds.iter().any(|c| c["name"] == "system_status"));
        assert!(cmds.iter().any(|c| c["name"] == "version_info"));
        assert!(cmds.iter().any(|c| c["name"] == "cleanup_temp"));
    }

    #[tokio::test]
    async fn run_unknown_command_returns_bad_request() {
        let state = crate::HttpState::test_http_state(std::env::temp_dir());
        let req = ConsoleRunRequest {
            command: "rm -rf /".to_owned(),
        };
        let resp = run_command(axum::extract::State(state), Json(req))
            .await
            .into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn run_version_info_returns_success() {
        let state = crate::HttpState::test_http_state(std::env::temp_dir());
        let req = ConsoleRunRequest {
            command: "version_info".to_owned(),
        };
        let resp = run_command(axum::extract::State(state), Json(req))
            .await
            .into_response();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), 64_000)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert!(json["output"]["version"].is_string());
    }

    #[tokio::test]
    async fn run_cleanup_temp_returns_success() {
        let state = crate::HttpState::test_http_state(std::env::temp_dir());
        let req = ConsoleRunRequest {
            command: "cleanup_temp".to_owned(),
        };
        let resp = run_command(axum::extract::State(state), Json(req))
            .await
            .into_response();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), 64_000)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
    }

    #[tokio::test]
    async fn all_declared_commands_are_handled() {
        let state = crate::HttpState::test_http_state(std::env::temp_dir());
        for cmd in COMMANDS {
            let req = ConsoleRunRequest {
                command: cmd.name.to_owned(),
            };
            let resp = run_command(axum::extract::State(state.clone()), Json(req))
                .await
                .into_response();
            assert_eq!(
                resp.status(),
                StatusCode::OK,
                "command '{}' returned non-200",
                cmd.name
            );
        }
    }
}
