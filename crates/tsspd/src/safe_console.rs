//! Safe console — runs only pre-approved diagnostic commands
//! No arbitrary shell execution, no sudo, no dangerous commands

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

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

fn get_disk_usage() -> Result<String, String> {
    // Read /proc/mounts and stat each filesystem
    std::fs::read_to_string("/proc/mounts")
        .map_err(|e| e.to_string())
        .and_then(|mounts| {
            let mut output = String::new();
            for line in mounts.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 {
                    let path = parts[1];
                    if let Ok(stat) = rustix::fs::statvfs(path) {
                        let block_size = stat.f_bsize as u64;
                        let total_blocks = stat.f_blocks as u64;
                        let free_blocks = stat.f_bfree as u64;

                        let total = total_blocks * block_size / (1024 * 1024 * 1024);
                        let used = (total_blocks - free_blocks) * block_size / (1024 * 1024 * 1024);
                        let percent = if total > 0 { (used * 100) / total } else { 0 };

                        output.push_str(&format!("{:<20} {:>8}G {:>8}G {:>3}% {}\n",
                            format!("{}:", path), total, used, percent, path));
                    }
                }
            }
            Ok(output)
        })
}

fn get_memory_usage() -> Result<String, String> {
    std::fs::read_to_string("/proc/meminfo")
        .map_err(|e| e.to_string())
        .map(|content| {
            let mut mem_total = 0u64;
            let mut mem_available = 0u64;

            for line in content.lines() {
                if let Some(value_str) = line.strip_prefix("MemTotal:") {
                    mem_total = value_str.trim().split_whitespace().next()
                        .and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                } else if let Some(value_str) = line.strip_prefix("MemAvailable:") {
                    mem_available = value_str.trim().split_whitespace().next()
                        .and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                }
            }

            let used = (mem_total - mem_available) / 1024;
            let total = mem_total / 1024;
            let percent = if total > 0 { (used * 100) / total } else { 0 };

            format!("Mem:  {:>8}M {:>8}M {:>3}%\n", total, used, percent)
        })
}

fn get_uptime() -> Result<String, String> {
    std::fs::read_to_string("/proc/uptime")
        .map_err(|e| e.to_string())
        .and_then(|content| {
            content.split_whitespace().next()
                .and_then(|s| s.parse::<f64>().ok())
                .map(|seconds| {
                    let days = seconds as u64 / 86400;
                    let hours = (seconds as u64 % 86400) / 3600;
                    let minutes = (seconds as u64 % 3600) / 60;

                    format!("up {:.0}d {:.0}h {:.0}m\n", days, hours, minutes)
                })
                .ok_or_else(|| "Could not parse uptime".to_string())
        })
}

fn get_top_processes() -> Result<String, String> {
    // Simple process listing from /proc
    let mut processes = Vec::new();

    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            if let Ok(filename) = entry.file_name().into_string() {
                if filename.chars().all(|c| c.is_ascii_digit()) {
                    let stat_path = format!("/proc/{}/stat", filename);
                    if let Ok(stat_content) = std::fs::read_to_string(&stat_path) {
                        // Basic parsing: get process name and CPU info
                        if let Some(open_paren) = stat_content.find('(') {
                            if let Some(close_paren) = stat_content.find(')') {
                                let name = &stat_content[open_paren + 1..close_paren];
                                processes.push((filename, name.to_string()));
                            }
                        }
                    }
                }
            }
        }
    }

    processes.sort_by(|a, b| a.0.cmp(&b.0));
    let mut output = String::from("PID     COMMAND\n");
    for (pid, name) in processes.iter().take(10) {
        output.push_str(&format!("{:<6} {}\n", pid, name));
    }
    Ok(output)
}

fn get_network_info() -> Result<String, String> {
    let mut output = String::new();

    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            if let Ok(filename) = entry.file_name().into_string() {
                output.push_str(&format!("{}\n", filename));
            }
        }
    }

    Ok(output)
}

fn get_temperature() -> Result<String, String> {
    std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
        .map_err(|e| e.to_string())
        .map(|temp_str| {
            if let Ok(temp_millidegrees) = temp_str.trim().parse::<f64>() {
                let temp_celsius = temp_millidegrees / 1000.0;
                format!("CPU Temp: {:.1}°C\n", temp_celsius)
            } else {
                "N/A\n".to_string()
            }
        })
}

fn get_disk_io() -> Result<String, String> {
    std::fs::read_to_string("/proc/diskstats")
        .map_err(|e| e.to_string())
        .map(|content| {
            let mut output = String::from("Device            Read(MB)     Write(MB)\n");
            for line in content.lines().take(5) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 10 {
                    let device = parts[2];
                    let read_sectors = parts[5].parse::<u64>().unwrap_or(0);
                    let write_sectors = parts[9].parse::<u64>().unwrap_or(0);
                    let read_mb = (read_sectors * 512) / (1024 * 1024);
                    let write_mb = (write_sectors * 512) / (1024 * 1024);
                    output.push_str(&format!("{:<15} {:>10} {:>10}\n", device, read_mb, write_mb));
                }
            }
            output
        })
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

    let output_str = match req.command.as_str() {
        "disk_usage" => {
            // Use /proc/mounts and statfs for disk usage
            match get_disk_usage() {
                Ok(s) => s,
                Err(e) => format!("Error: {}", e),
            }
        },
        "memory_usage" => {
            // Read /proc/meminfo for memory stats
            match get_memory_usage() {
                Ok(s) => s,
                Err(e) => format!("Error: {}", e),
            }
        },
        "uptime" => {
            // Read /proc/uptime for uptime
            match get_uptime() {
                Ok(s) => s,
                Err(e) => format!("Error: {}", e),
            }
        },
        "processes" => {
            // Read /proc files for process info
            match get_top_processes() {
                Ok(s) => s,
                Err(e) => format!("Error: {}", e),
            }
        },
        "network" => {
            // Read /sys/class/net for network interfaces
            match get_network_info() {
                Ok(s) => s,
                Err(e) => format!("Error: {}", e),
            }
        },
        "temperature" => {
            // Read /sys/class/thermal for temp
            match get_temperature() {
                Ok(s) => s,
                Err(e) => format!("N/A"),
            }
        },
        "disk_io" => {
            // Read /proc/diskstats for I/O info
            match get_disk_io() {
                Ok(s) => s,
                Err(e) => format!("iostat not available"),
            }
        },
        "file_count" => {
            // This requires directory walking - just return a message for now
            "File counting requires bulk operation. Use admin API for accurate count.".to_string()
        },
        _ => "Unknown command".to_string(),
    };

    let duration = start.elapsed();

    (
        StatusCode::OK,
        Json(CommandOutput {
            command: req.command,
            output: output_str,
            exit_code: 0,
            duration_ms: duration.as_millis(),
        }),
    )
        .into_response()
}
}
