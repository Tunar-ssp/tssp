//! Workspace status commands.

use serde::Deserialize;
use serde_json::json;

use crate::backend::{api_get, build_client, BackendAddress};
use crate::Cli;
use tssp_cli_core::CliExitCode;

#[derive(Debug, Deserialize, serde::Serialize)]
struct TerminalStatusResponse {
    schema_version: u8,
    available: bool,
    reason: Option<String>,
}

#[derive(Debug, Deserialize, serde::Serialize)]
struct LspStatusResponse {
    schema_version: u8,
    status: String,
    available_languages: Vec<String>,
}

#[derive(Debug, Deserialize, serde::Serialize)]
struct GitStatusResponse {
    schema_version: u8,
    is_repo: bool,
    branch: Option<String>,
    changed: u32,
    staged: u32,
    untracked: u32,
}

/// Run workspace terminal-status command.
pub fn terminal_status(cli: &Cli, workspace_id: &str) -> Result<CliExitCode, String> {
    let address = BackendAddress::from_connection_args(&cli.connection)?;
    let client = build_client()?;
    let url = address.url(&format!("/api/v1/workspaces/{workspace_id}/terminal"));

    let response = api_get(&client, &url)
        .send()
        .map_err(|error| format!("could not reach daemon: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("daemon error: {}", response.status()));
    }

    let data: TerminalStatusResponse = response
        .json()
        .map_err(|error| format!("invalid response: {error}"))?;

    if cli.output.json {
        println!("{}", json!(data));
    } else if data.available {
        println!("terminal: available");
    } else {
        println!(
            "terminal: unavailable ({})",
            data.reason.as_deref().unwrap_or("unknown reason")
        );
    }
    Ok(CliExitCode::Success)
}

/// Run workspace lsp-status command.
pub fn lsp_status(cli: &Cli, workspace_id: &str) -> Result<CliExitCode, String> {
    let address = BackendAddress::from_connection_args(&cli.connection)?;
    let client = build_client()?;
    let url = address.url(&format!("/api/v1/workspaces/{workspace_id}/lsp"));

    let response = api_get(&client, &url)
        .send()
        .map_err(|error| format!("could not reach daemon: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("daemon error: {}", response.status()));
    }

    let data: LspStatusResponse = response
        .json()
        .map_err(|error| format!("invalid response: {error}"))?;

    if cli.output.json {
        println!("{}", json!(data));
    } else {
        println!("lsp: {}", data.status);
        if !data.available_languages.is_empty() {
            println!(
                "  available languages: {}",
                data.available_languages.join(", ")
            );
        }
    }
    Ok(CliExitCode::Success)
}

/// Run workspace git-status command.
pub fn git_status(cli: &Cli, workspace_id: &str) -> Result<CliExitCode, String> {
    let address = BackendAddress::from_connection_args(&cli.connection)?;
    let client = build_client()?;
    let url = address.url(&format!("/api/v1/workspaces/{workspace_id}/git"));

    let response = api_get(&client, &url)
        .send()
        .map_err(|error| format!("could not reach daemon: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("daemon error: {}", response.status()));
    }

    let data: GitStatusResponse = response
        .json()
        .map_err(|error| format!("invalid response: {error}"))?;

    if cli.output.json {
        println!("{}", json!(data));
    } else if data.is_repo {
        println!("git: repository");
        if let Some(branch) = &data.branch {
            println!("  branch: {branch}");
        }
        println!("  changed: {}", data.changed);
        println!("  staged: {}", data.staged);
        println!("  untracked: {}", data.untracked);
    } else {
        println!("git: not a git repository");
    }
    Ok(CliExitCode::Success)
}
