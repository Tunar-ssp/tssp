//! Host system metrics for the admin panel.

use serde::Serialize;
use std::path::Path;

/// Host-level system snapshot.
#[derive(Debug, Serialize)]
pub struct SystemSnapshot {
    pub schema_version: u8,
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub load_average_1m: f64,
    pub total_memory_bytes: u64,
    pub available_memory_bytes: u64,
    pub data_dir_free_bytes: u64,
    pub data_dir_total_bytes: u64,
}

/// Collects system metrics for admin overview.
///
/// # Errors
///
/// Returns an error when disk stats cannot be read.
pub fn collect_system_snapshot(data_dir: &Path) -> Result<SystemSnapshot, String> {
    let (total_memory_bytes, available_memory_bytes) = memory_info();
    let (data_dir_total_bytes, data_dir_free_bytes) = disk_info(data_dir)?;
    Ok(SystemSnapshot {
        schema_version: 1,
        hostname: hostname(),
        os: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
        load_average_1m: load_average(),
        total_memory_bytes,
        available_memory_bytes,
        data_dir_free_bytes,
        data_dir_total_bytes,
    })
}

fn hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_owned())
        .unwrap_or_else(|_| "localhost".to_owned())
}

fn load_average() -> f64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(line) = std::fs::read_to_string("/proc/loadavg") {
            if let Some(one) = line.split_whitespace().next() {
                if let Ok(value) = one.parse::<f64>() {
                    return value;
                }
            }
        }
    }
    0.0
}

fn memory_info() -> (u64, u64) {
    #[cfg(target_os = "linux")]
    {
        if let Ok(contents) = std::fs::read_to_string("/proc/meminfo") {
            let mut total = 0_u64;
            let mut available = 0_u64;
            for line in contents.lines() {
                if let Some(kb) = line.strip_prefix("MemTotal:") {
                    total = parse_kb(kb);
                } else if let Some(kb) = line.strip_prefix("MemAvailable:") {
                    available = parse_kb(kb);
                }
            }
            if total > 0 {
                return (total, available);
            }
        }
    }
    (0, 0)
}

#[cfg(target_os = "linux")]
fn parse_kb(value: &str) -> u64 {
    value
        .trim()
        .strip_suffix(" kB")
        .or_else(|| value.trim().strip_suffix(" kB"))
        .and_then(|v| v.trim().parse::<u64>().ok())
        .map(|kb| kb * 1024)
        .unwrap_or(0)
}

fn disk_info(path: &Path) -> Result<(u64, u64), String> {
    let path = path.to_path_buf();
    std::thread::spawn(move || {
        let stat = nix_statvfs(&path)?;
        Ok((stat.total_bytes, stat.free_bytes))
    })
    .join()
    .map_err(|_| "disk info thread failed".to_owned())?
}

struct StatVfs {
    total_bytes: u64,
    free_bytes: u64,
}

fn nix_statvfs(path: &Path) -> Result<StatVfs, String> {
    let check_path = if path.exists() {
        path.to_path_buf()
    } else {
        path.parent()
            .filter(|p| p.exists())
            .unwrap_or(std::path::Path::new("/tmp"))
            .to_path_buf()
    };

    let stat = rustix::fs::statvfs(&check_path)
        .map_err(|error| format!("statvfs on {} failed: {error}", check_path.display()))?;
    let block_size = stat.f_frsize;
    Ok(StatVfs {
        total_bytes: stat.f_blocks * block_size,
        free_bytes: stat.f_bavail * block_size,
    })
}
