//! `tssp admin` — storage and host management commands.

use reqwest::header::ACCEPT;
use serde::Deserialize;
use tssp_cli_core::CliExitCode;

use crate::backend::{api_delete, api_get, api_post, build_client, BackendAddress};
use tssp::{AdminAction, AdminCommand, AdminFilesArgs, Cli};

pub(crate) fn run(cli: &Cli, command: &AdminCommand) -> Result<CliExitCode, String> {
    match &command.action {
        AdminAction::Overview => get_json(cli, "/api/v1/admin/overview", "overview"),
        AdminAction::System => get_json(cli, "/api/v1/admin/system", "system"),
        AdminAction::Files(args) => admin_files(cli, args),
        AdminAction::Folders => get_json(cli, "/api/v1/admin/folders", "folders"),
        AdminAction::Delete(args) => admin_delete(cli, &args.id),
        AdminAction::Corrupt => get_json(cli, "/api/v1/admin/corrupt", "corrupt"),
        AdminAction::CleanupTemp => post_json(cli, "/api/v1/admin/cleanup/temp", "cleanup"),
        AdminAction::CleanupSessions => post_json(cli, "/api/v1/admin/cleanup/sessions", "cleanup"),
    }
}

fn get_json(cli: &Cli, path: &str, label: &str) -> Result<CliExitCode, String> {
    let body = fetch(
        cli,
        |client, url| api_get(client, url).header(ACCEPT, "application/json"),
        path,
    )?;
    print_body(cli, &body, label)
}

fn post_json(cli: &Cli, path: &str, label: &str) -> Result<CliExitCode, String> {
    let body = fetch(
        cli,
        |client, url| api_post(client, url).header(ACCEPT, "application/json"),
        path,
    )?;
    print_body(cli, &body, label)
}

fn admin_files(cli: &Cli, args: &AdminFilesArgs) -> Result<CliExitCode, String> {
    let mut path = format!("/api/v1/admin/files?limit={}", args.limit);
    if let Some(folder) = &args.folder {
        path.push_str("&folder=");
        path.push_str(&encode_query_component(folder));
    }
    if let Some(mime) = &args.mime_type {
        path.push_str("&type=");
        path.push_str(&encode_query_component(mime));
    }
    let body = fetch(
        cli,
        |client, url| api_get(client, url).header(ACCEPT, "application/json"),
        &path,
    )?;
    if cli.output.json {
        println!("{body}");
        return Ok(CliExitCode::Success);
    }
    let parsed: AdminFilesResponse =
        serde_json::from_str(&body).map_err(|e| format!("invalid admin files response: {e}"))?;
    if parsed.files.is_empty() {
        if !cli.output.quiet {
            println!("No files.");
        }
        return Ok(CliExitCode::Success);
    }
    for file in &parsed.files {
        let folder = if file.folder_path.is_empty() {
            String::new()
        } else {
            format!("  folder: {}", file.folder_path)
        };
        let pin = if file.pinned { " pinned" } else { "" };
        println!(
            "{}  {}  {} bytes  {}{}",
            file.id, file.name, file.size_bytes, file.mime_type, pin
        );
        if !folder.is_empty() && !cli.output.quiet {
            println!("{folder}");
        }
    }
    Ok(CliExitCode::Success)
}

fn admin_delete(cli: &Cli, id: &str) -> Result<CliExitCode, String> {
    let path = format!("/api/v1/admin/files/{id}");
    let body = fetch(
        cli,
        |client, url| api_delete(client, url).header(ACCEPT, "application/json"),
        &path,
    )?;
    print_body(cli, &body, "delete")
}

fn fetch<F>(cli: &Cli, build: F, path: &str) -> Result<String, String>
where
    F: FnOnce(&reqwest::blocking::Client, &str) -> reqwest::blocking::RequestBuilder,
{
    let address = BackendAddress::from_connection_args(&cli.connection)?;
    let client = build_client()?;
    let url = address.url(path);
    let response = build(&client, &url)
        .send()
        .map_err(|e| format!("could not reach daemon at {}: {e}", address.base_url()))?;
    if !response.status().is_success() {
        return Err(format!(
            "daemon returned {} for {}",
            response.status(),
            path
        ));
    }
    response
        .text()
        .map_err(|e| format!("unreadable response from {}: {e}", address.base_url()))
}

fn print_body(cli: &Cli, body: &str, _label: &str) -> Result<CliExitCode, String> {
    if cli.output.json {
        println!("{body}");
    } else if !cli.output.quiet {
        let value: serde_json::Value = serde_json::from_str(body)
            .unwrap_or_else(|_| serde_json::Value::String(body.to_owned()));
        println!(
            "{}",
            serde_json::to_string_pretty(&value).unwrap_or_else(|_| body.to_owned())
        );
    }
    Ok(CliExitCode::Success)
}

#[derive(Debug, Deserialize)]
struct AdminFilesResponse {
    files: Vec<AdminFileRow>,
}

#[derive(Debug, Deserialize)]
struct AdminFileRow {
    id: String,
    name: String,
    size_bytes: u64,
    mime_type: String,
    #[serde(default)]
    folder_path: String,
    #[serde(default)]
    pinned: bool,
}

fn encode_query_component(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'/' => {
                char::from(byte).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}
