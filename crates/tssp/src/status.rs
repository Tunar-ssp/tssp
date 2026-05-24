//! Implementation of `tssp status`.

use reqwest::{header::ACCEPT, StatusCode};
use serde::{Deserialize, Serialize};
use tssp_cli_core::CliExitCode;

use crate::backend::{api_get, build_client, BackendAddress};
use tssp::Cli;

pub(crate) fn run(cli: &Cli) -> Result<CliExitCode, String> {
    let address = match BackendAddress::from_connection_args(&cli.connection) {
        Ok(value) => value,
        Err(message) => {
            eprintln!("error: {message}");
            return Ok(CliExitCode::Usage);
        }
    };
    let client = build_client()?;
    let response = api_get(&client, &address.url("/api/v1/status"))
        .header(ACCEPT, "application/vnd.tssp.v1+json")
        .send()
        .map_err(|error| {
            eprintln!(
                "error: could not reach daemon at {}: {error}",
                address.base_url()
            );
            CliExitCode::Network
        });
    let response = match response {
        Ok(value) => value,
        Err(code) => return Ok(code),
    };

    if let Err(code) = classify_response_status(response.status()) {
        eprintln!("error: daemon returned {}", response.status());
        return Ok(code);
    }

    let body = response.text().map_err(|error| {
        format!(
            "daemon at {} returned an unreadable status response: {error}",
            address.base_url()
        )
    })?;
    let status = parse_status_body(&body, &address.base_url())?;
    print_status(&status, cli.output.json)?;
    Ok(CliExitCode::Success)
}

fn classify_response_status(status: StatusCode) -> Result<(), CliExitCode> {
    if status.is_server_error() {
        return Err(CliExitCode::Server);
    }
    if !status.is_success() {
        return Err(CliExitCode::Generic);
    }
    Ok(())
}

fn parse_status_body(body: &str, base_url: &str) -> Result<DaemonStatus, String> {
    serde_json::from_str::<DaemonStatus>(body).map_err(|error| {
        format!("daemon at {base_url} returned an invalid status response: {error}")
    })
}

#[derive(Debug, Deserialize, Serialize)]
struct DaemonStatus {
    schema_version: u8,
    version: String,
    status: String,
    uptime_seconds: u64,
    file_count: u64,
    #[serde(default)]
    note_count: u64,
    tag_count: u64,
    pinned_count: u64,
    recent_upload_count_24h: u64,
    #[serde(default)]
    storage_bytes_used: u64,
    #[serde(default)]
    corrupt_file_count: u64,
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    let s = secs % 60;
    if days > 0 {
        format!("{days}d {hours}h {mins}m {s}s")
    } else if hours > 0 {
        format!("{hours}h {mins}m {s}s")
    } else if mins > 0 {
        format!("{mins}m {s}s")
    } else {
        format!("{s}s")
    }
}

fn format_bytes(b: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    #[allow(clippy::cast_precision_loss)]
    if b >= GB {
        format!("{:.1} GiB", b as f64 / GB as f64)
    } else if b >= MB {
        format!("{:.1} MiB", b as f64 / MB as f64)
    } else if b >= KB {
        format!("{:.1} KiB", b as f64 / KB as f64)
    } else {
        format!("{b} B")
    }
}

fn print_status(status: &DaemonStatus, json: bool) -> Result<(), String> {
    if json {
        let encoded = serde_json::to_string(status)
            .map_err(|error| format!("could not encode status JSON: {error}"))?;
        println!("{encoded}");
        return Ok(());
    }

    println!("tsspd {}  [{}]", status.version, status.status);
    println!("Uptime:    {}", format_uptime(status.uptime_seconds));
    println!("Storage:   {}", format_bytes(status.storage_bytes_used));
    println!(
        "Files:     {}  (pinned: {}, uploads/24h: {})",
        status.file_count, status.pinned_count, status.recent_upload_count_24h
    );
    println!("Notes:     {}", status.note_count);
    println!("Tags:      {}", status.tag_count);
    if status.corrupt_file_count > 0 {
        eprintln!(
            "warning: {} file(s) with missing blobs",
            status.corrupt_file_count
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{classify_response_status, parse_status_body, print_status, DaemonStatus};
    use reqwest::StatusCode;
    use tssp_cli_core::CliExitCode;

    #[test]
    fn daemon_status_shape_is_stable() {
        let status = DaemonStatus {
            schema_version: 1,
            version: "0.1.0".to_owned(),
            status: "ok".to_owned(),
            uptime_seconds: 1,
            file_count: 2,
            note_count: 0,
            tag_count: 3,
            pinned_count: 4,
            recent_upload_count_24h: 5,
            storage_bytes_used: 0,
            corrupt_file_count: 0,
        };

        assert_eq!(status.schema_version, 1);
        assert_eq!(status.file_count, 2);
    }

    #[test]
    fn response_status_maps_server_errors() {
        let result = classify_response_status(StatusCode::INTERNAL_SERVER_ERROR);

        assert_eq!(result, Err(CliExitCode::Server));
    }

    #[test]
    fn response_status_maps_client_errors() {
        let result = classify_response_status(StatusCode::NOT_FOUND);

        assert_eq!(result, Err(CliExitCode::Generic));
    }

    #[test]
    fn response_status_allows_success() {
        assert_eq!(classify_response_status(StatusCode::OK), Ok(()));
    }

    #[test]
    fn parse_status_body_accepts_valid_payload() {
        let status = parse_status_body(
            r#"{"schema_version":1,"version":"0.1.0","status":"ok","uptime_seconds":1,"file_count":2,"note_count":1,"tag_count":3,"pinned_count":4,"recent_upload_count_24h":5,"storage_bytes_used":1024,"corrupt_file_count":0}"#,
            "http://127.0.0.1:8421",
        )
        .unwrap_or_else(|error| panic!("parse failed: {error}"));

        assert_eq!(status.version, "0.1.0");
        assert_eq!(status.recent_upload_count_24h, 5);
    }

    #[test]
    fn parse_status_body_rejects_invalid_payload() {
        let result = parse_status_body(r#"{"schema_version":1}"#, "http://127.0.0.1:8421");

        assert!(matches!(result, Err(message) if message.contains("invalid status response")));
    }

    #[test]
    fn print_status_supports_json_and_human_output() {
        let status = DaemonStatus {
            schema_version: 1,
            version: "0.1.0".to_owned(),
            status: "ok".to_owned(),
            uptime_seconds: 1,
            file_count: 2,
            note_count: 0,
            tag_count: 3,
            pinned_count: 4,
            recent_upload_count_24h: 5,
            storage_bytes_used: 0,
            corrupt_file_count: 0,
        };

        assert_eq!(print_status(&status, true), Ok(()));
        assert_eq!(print_status(&status, false), Ok(()));
    }
}
