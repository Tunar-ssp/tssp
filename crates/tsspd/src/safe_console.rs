//! Safe console — runs only pre-approved diagnostic commands
//! No arbitrary shell execution, no sudo, no dangerous commands

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleCommand {
    pub id: String,
    pub label: String,
    pub description: String,
    pub command: String,
}

#[derive(Debug, Deserialize)]
pub struct RunCommandRequest {
    pub command: String, // ID of the whitelisted command
}

#[derive(Debug, Serialize)]
pub struct CommandOutput {
    pub command: String,
    pub output: String,
    pub exit_code: i32,
    pub duration_ms: u128,
}

/// Get list of available safe commands
pub async fn list_commands() -> impl IntoResponse {
    let commands = vec![
        ConsoleCommand {
            id: "disk_usage".to_string(),
            label: "Disk Usage".to_string(),
            description: "Show disk usage (df -h)".to_string(),
            command: "df -h".to_string(),
        },
        ConsoleCommand {
            id: "memory_usage".to_string(),
            label: "Memory Usage".to_string(),
            description: "Show memory usage (free -h)".to_string(),
            command: "free -h".to_string(),
        },
        ConsoleCommand {
            id: "uptime".to_string(),
            label: "Uptime".to_string(),
            description: "Show system uptime".to_string(),
            command: "uptime".to_string(),
        },
        ConsoleCommand {
            id: "processes".to_string(),
            label: "Top Processes".to_string(),
            description: "Show top CPU-consuming processes".to_string(),
            command: "ps aux --sort=-%cpu | head -10".to_string(),
        },
        ConsoleCommand {
            id: "network".to_string(),
            label: "Network Stats".to_string(),
            description: "Show network configuration".to_string(),
            command: "ip addr show".to_string(),
        },
        ConsoleCommand {
            id: "temperature".to_string(),
            label: "System Temperature".to_string(),
            description: "Show CPU temperature (if available)".to_string(),
            command: "cat /sys/class/thermal/thermal_zone0/temp 2>/dev/null || echo 'N/A'".to_string(),
        },
        ConsoleCommand {
            id: "disk_io".to_string(),
            label: "Disk I/O".to_string(),
            description: "Show disk I/O stats (iostat summary)".to_string(),
            command: "iostat -x 1 2 2>/dev/null | tail -10 || echo 'iostat not available'".to_string(),
        },
        ConsoleCommand {
            id: "file_count".to_string(),
            label: "File Count".to_string(),
            description: "Count total files in storage".to_string(),
            command: "find . -type f | wc -l".to_string(),
        },
    ];

    Json(commands)
}

/// Run a whitelisted console command (admin only)
pub async fn run_command(
    State(_state): State<()>,
    Json(req): Json<RunCommandRequest>,
) -> impl IntoResponse {
    // Whitelist of safe command IDs
    let commands = vec![
        "disk_usage",
        "memory_usage",
        "uptime",
        "processes",
        "network",
        "temperature",
        "disk_io",
        "file_count",
    ];

    if !commands.contains(&req.command.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Command not whitelisted"
            })),
        )
            .into_response();
    }

    let cmd_str = match req.command.as_str() {
        "disk_usage" => "df -h",
        "memory_usage" => "free -h",
        "uptime" => "uptime",
        "processes" => "ps aux --sort=-%cpu | head -10",
        "network" => "ip addr show",
        "temperature" => "cat /sys/class/thermal/thermal_zone0/temp 2>/dev/null || echo 'N/A'",
        "disk_io" => "iostat -x 1 2 2>/dev/null | tail -10 || echo 'iostat not available'",
        "file_count" => "find . -type f | wc -l",
        _ => return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Unknown command"})),
        )
            .into_response(),
    };

    let start = std::time::Instant::now();
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd_str)
        .output();

    let duration = start.elapsed();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let code = output.status.code().unwrap_or(-1);

            (
                StatusCode::OK,
                Json(CommandOutput {
                    command: req.command,
                    output: stdout,
                    exit_code: code,
                    duration_ms: duration.as_millis(),
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to execute: {}", e)
            })),
        )
            .into_response(),
    }
}
