//! `tssp note` subcommands.

use reqwest::{header::ACCEPT, StatusCode};
use serde::{Deserialize, Serialize};
use tssp::{
    Cli, NoteCreateArgs, NoteDeleteArgs, NoteEditArgs, NoteExportArgs, NoteListArgs, NoteShowArgs,
    NoteSubcommand,
};
use tssp_cli_core::CliExitCode;

use crate::backend::{api_delete, api_get, api_post, api_put, build_client, BackendAddress};

const NOTES_ENDPOINT: &str = "/api/v1/notes";

/// Runs a `tssp note` subcommand.
pub fn run(cli: &Cli, command: &NoteSubcommand) -> Result<CliExitCode, String> {
    match command {
        NoteSubcommand::Create(args) => run_create(cli, args),
        NoteSubcommand::List(args) => run_list(cli, args),
        NoteSubcommand::Show(args) => run_show(cli, args),
        NoteSubcommand::Edit(args) => run_edit(cli, args),
        NoteSubcommand::Delete(args) => run_delete(cli, args),
        NoteSubcommand::Export(args) => run_export(cli, args),
    }
}

fn run_create(cli: &Cli, args: &NoteCreateArgs) -> Result<CliExitCode, String> {
    let body = read_note_body(args)?;
    if body.trim().is_empty() {
        eprintln!("error: note body must not be empty");
        return Ok(CliExitCode::Usage);
    }

    let address = BackendAddress::from_connection_args(&cli.connection)
        .map_err(|message| format!("invalid backend address: {message}"))?;
    let payload = CreateNoteRequest {
        title: args.title.clone(),
        body,
        tags: args.tags.clone(),
        pin: args.pin,
    };

    let client = build_client()?;
    let response = api_post(&client, &address.url(NOTES_ENDPOINT))
        .header(ACCEPT, "application/vnd.tssp.v1+json")
        .json(&payload)
        .send()
        .map_err(|error| format!("could not reach daemon: {error}"))?;

    handle_note_response(response, cli, "created")
}

fn run_list(cli: &Cli, args: &NoteListArgs) -> Result<CliExitCode, String> {
    let address = BackendAddress::from_connection_args(&cli.connection)
        .map_err(|message| format!("invalid backend address: {message}"))?;
    let client = build_client()?;
    let mut query = vec![("limit", args.limit.unwrap_or(50).to_string())];
    for tag in &args.tags {
        query.push(("tag", tag.clone()));
    }
    if args.pinned {
        query.push(("pinned", "true".to_owned()));
    }
    if let Some(sort) = &args.sort {
        query.push(("sort", sort.clone()));
    }

    let response = api_get(&client, &address.url(NOTES_ENDPOINT))
        .query(&query)
        .header(ACCEPT, "application/vnd.tssp.v1+json")
        .send()
        .map_err(|error| format!("could not reach daemon: {error}"))?;

    if !response.status().is_success() {
        eprintln!("error: daemon returned {}", response.status());
        return Ok(classify_status(response.status()));
    }

    let body = response
        .text()
        .map_err(|error| format!("could not read note list: {error}"))?;
    let page: NoteListResponse = serde_json::from_str(&body)
        .map_err(|error| format!("invalid note list response: {error}"))?;

    if cli.output.json {
        println!("{body}");
        return Ok(CliExitCode::Success);
    }
    if cli.output.quiet {
        for note in &page.notes {
            println!("{}\t{}", note.id, note.title);
        }
        return Ok(CliExitCode::Success);
    }

    for note in &page.notes {
        let pin = if note.pinned { "📌 " } else { "   " };
        let tags = if note.tags.is_empty() {
            String::new()
        } else {
            format!("  [{}]", note.tags.join(", "))
        };
        println!("{pin}{:<36}  {}{}", note.id, note.title, tags);
    }
    Ok(CliExitCode::Success)
}

fn run_show(cli: &Cli, args: &NoteShowArgs) -> Result<CliExitCode, String> {
    let address = BackendAddress::from_connection_args(&cli.connection)
        .map_err(|message| format!("invalid backend address: {message}"))?;
    let client = build_client()?;
    let response = api_get(
        &client,
        &address.url(&format!("{NOTES_ENDPOINT}/{}", args.id)),
    )
    .header(ACCEPT, "application/vnd.tssp.v1+json")
    .send()
    .map_err(|error| format!("could not reach daemon: {error}"))?;

    if !response.status().is_success() {
        eprintln!("error: daemon returned {}", response.status());
        return Ok(classify_status(response.status()));
    }

    let note: NoteRecordResponse = response
        .json()
        .map_err(|error| format!("invalid note response: {error}"))?;

    if cli.output.json {
        println!(
            "{}",
            serde_json::to_string(&note).map_err(|error| error.to_string())?
        );
        return Ok(CliExitCode::Success);
    }

    println!("# {}{}", note.title, if note.pinned { "  📌" } else { "" });
    if !note.tags.is_empty() {
        println!("tags: {}", note.tags.join(", "));
    }
    println!();
    println!("{}", note.body);
    Ok(CliExitCode::Success)
}

fn run_edit(cli: &Cli, args: &NoteEditArgs) -> Result<CliExitCode, String> {
    let address = BackendAddress::from_connection_args(&cli.connection)
        .map_err(|message| format!("invalid backend address: {message}"))?;
    let client = build_client()?;

    // Handle pin/unpin as separate requests first.
    if args.pin || args.unpin {
        let pin_url = address.url(&format!("{NOTES_ENDPOINT}/{}/pin", args.id));
        let res = if args.pin {
            api_put(&client, &pin_url)
                .header(ACCEPT, "application/vnd.tssp.v1+json")
                .send()
        } else {
            api_delete(&client, &pin_url)
                .header(ACCEPT, "application/vnd.tssp.v1+json")
                .send()
        }
        .map_err(|error| format!("could not reach daemon: {error}"))?;
        if !res.status().is_success() && res.status() != StatusCode::NO_CONTENT {
            eprintln!("error: pin update failed: {}", res.status());
            return Ok(classify_status(res.status()));
        }
        if !cli.output.quiet && args.body.is_none() && args.title.is_none() && args.tags.is_empty() {
            println!("{} note {}", if args.pin { "pinned" } else { "unpinned" }, args.id);
        }
        if args.body.is_none() && args.title.is_none() && args.tags.is_empty() {
            return Ok(CliExitCode::Success);
        }
    }

    let body = if let Some(body) = &args.body {
        body.clone()
    } else {
        read_note_body_from_editor_or_stdin(args)?
    };
    if body.trim().is_empty() {
        eprintln!("error: note body must not be empty");
        return Ok(CliExitCode::Usage);
    }

    let tags = if args.tags.is_empty() { None } else { Some(args.tags.clone()) };
    let payload = UpdateNoteRequest {
        title: args.title.clone(),
        body,
        tags,
    };
    let response = api_put(
        &client,
        &address.url(&format!("{NOTES_ENDPOINT}/{}", args.id)),
    )
    .header(ACCEPT, "application/vnd.tssp.v1+json")
    .json(&payload)
    .send()
    .map_err(|error| format!("could not reach daemon: {error}"))?;

    handle_note_response(response, cli, "updated")
}

fn run_export(cli: &Cli, args: &NoteExportArgs) -> Result<CliExitCode, String> {
    let address = BackendAddress::from_connection_args(&cli.connection)
        .map_err(|message| format!("invalid backend address: {message}"))?;
    let client = build_client()?;
    let response = api_get(&client, &address.url("/api/v1/notes/export"))
        .send()
        .map_err(|error| format!("could not reach daemon: {error}"))?;

    if !response.status().is_success() {
        eprintln!("error: daemon returned {}", response.status());
        return Ok(classify_status(response.status()));
    }

    let body = response
        .text()
        .map_err(|error| format!("could not read export: {error}"))?;

    if let Some(path) = &args.output {
        std::fs::write(path, &body)
            .map_err(|error| format!("could not write {}: {error}", path.display()))?;
        if !cli.output.quiet {
            println!("exported to {}", path.display());
        }
    } else {
        print!("{body}");
    }
    Ok(CliExitCode::Success)
}

fn run_delete(cli: &Cli, args: &NoteDeleteArgs) -> Result<CliExitCode, String> {
    let address = BackendAddress::from_connection_args(&cli.connection)
        .map_err(|message| format!("invalid backend address: {message}"))?;
    let client = build_client()?;
    let response = api_delete(
        &client,
        &address.url(&format!("{NOTES_ENDPOINT}/{}", args.id)),
    )
    .header(ACCEPT, "application/vnd.tssp.v1+json")
    .send()
    .map_err(|error| format!("could not reach daemon: {error}"))?;

    if response.status() == StatusCode::NO_CONTENT || response.status().is_success() {
        if !cli.output.quiet {
            println!("deleted note {}", args.id);
        }
        return Ok(CliExitCode::Success);
    }
    eprintln!("error: daemon returned {}", response.status());
    Ok(classify_status(response.status()))
}

fn handle_note_response(
    response: reqwest::blocking::Response,
    cli: &Cli,
    verb: &str,
) -> Result<CliExitCode, String> {
    if !response.status().is_success() {
        eprintln!("error: daemon returned {}", response.status());
        return Ok(classify_status(response.status()));
    }
    let note: NoteRecordResponse = response
        .json()
        .map_err(|error| format!("invalid note response: {error}"))?;
    if cli.output.json {
        println!(
            "{}",
            serde_json::to_string(&note).map_err(|error| error.to_string())?
        );
    } else if !cli.output.quiet {
        println!("{verb} note {} ({})", note.id, note.title);
    }
    Ok(CliExitCode::Success)
}

fn classify_status(status: StatusCode) -> CliExitCode {
    if status == StatusCode::NOT_FOUND {
        CliExitCode::NotFound
    } else if status.is_server_error() {
        CliExitCode::Server
    } else {
        CliExitCode::Generic
    }
}

fn read_note_body(args: &NoteCreateArgs) -> Result<String, String> {
    if let Some(body) = &args.body {
        return Ok(body.clone());
    }
    read_note_body_from_stdin_or_editor()
}

fn read_note_body_from_editor_or_stdin(_args: &NoteEditArgs) -> Result<String, String> {
    read_note_body_from_stdin_or_editor()
}

fn read_note_body_from_stdin_or_editor() -> Result<String, String> {
    use std::io::{self, IsTerminal, Read};
    if !io::stdin().is_terminal() {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|error| format!("could not read note body from stdin: {error}"))?;
        return Ok(buffer);
    }
    Err("interactive editor support is not configured; pass --body or pipe stdin".to_owned())
}

#[derive(Debug, Serialize)]
struct CreateNoteRequest {
    title: Option<String>,
    body: String,
    tags: Vec<String>,
    pin: bool,
}

#[derive(Debug, Serialize)]
struct UpdateNoteRequest {
    title: Option<String>,
    body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct NoteListResponse {
    notes: Vec<NoteRecordResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
struct NoteRecordResponse {
    id: String,
    title: String,
    body: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    pinned: bool,
    updated_at: i64,
}
