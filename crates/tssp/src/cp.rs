//! `tssp cp` quick text save to the cloud.

use std::io::{self, IsTerminal, Read};

use reqwest::header::ACCEPT;
use tssp::{Cli, CpArgs};
use tssp_cli_core::CliExitCode;

use crate::backend::{api_post, build_client, BackendAddress};

/// Runs `tssp cp`.
pub fn run(cli: &Cli, args: &CpArgs) -> Result<CliExitCode, String> {
    let text = read_input(args)?;
    if text.trim().is_empty() {
        eprintln!("error: no text to save");
        return Ok(CliExitCode::Usage);
    }

    let address = BackendAddress::from_connection_args(&cli.connection)
        .map_err(|message| format!("invalid backend address: {message}"))?;
    let payload = serde_json::json!({
        "title": args.title,
        "body": text,
        "tags": args.tags,
        "pin": false,
    });

    let client = build_client()?;
    let response = api_post(&client, &address.url("/api/v1/notes"))
        .header(ACCEPT, "application/vnd.tssp.v1+json")
        .json(&payload)
        .send()
        .map_err(|error| format!("could not reach daemon: {error}"))?;

    if !response.status().is_success() {
        eprintln!("error: daemon returned {}", response.status());
        return Ok(if response.status().is_server_error() {
            CliExitCode::Server
        } else {
            CliExitCode::Generic
        });
    }

    let note: serde_json::Value = response
        .json()
        .map_err(|error| format!("invalid note response: {error}"))?;
    if cli.output.json {
        println!("{note}");
    } else if !cli.output.quiet {
        let id = note
            .get("id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        let title = note
            .get("title")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("Untitled");
        println!("saved note {id} ({title})");
    }
    Ok(CliExitCode::Success)
}

fn read_input(args: &CpArgs) -> Result<String, String> {
    if !io::stdin().is_terminal() {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|error| format!("could not read stdin: {error}"))?;
        return Ok(buffer);
    }

    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        if let Ok(text) = clipboard.get_text() {
            if !text.trim().is_empty() {
                return Ok(text);
            }
        }
    }

    if let Some(path) = looks_like_path_from_stdin_hint(args) {
        return Err(format!(
            "input looks like a file path ({path}); use `tssp` to upload files"
        ));
    }

    Err("no piped stdin and clipboard is empty or unavailable".to_owned())
}

fn looks_like_path_from_stdin_hint(_args: &CpArgs) -> Option<String> {
    None
}
