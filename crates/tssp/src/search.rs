//! Implementation of `tssp search`.

use reqwest::{header::ACCEPT, StatusCode};
use serde::{Deserialize, Serialize};
use tssp_cli_core::CliExitCode;

use crate::backend::{build_client, BackendAddress};
use tssp::{Cli, SearchArgs};

const SEARCH_ENDPOINT: &str = "/api/v1/search";

/// Runs `tssp search`.
pub(crate) fn run_search(cli: &Cli, args: &SearchArgs) -> Result<CliExitCode, String> {
    if let Some(message) = unsupported_filter(args) {
        eprintln!("error: {message}");
        return Ok(CliExitCode::Usage);
    }

    if args.query.trim().is_empty() {
        eprintln!("error: search query must not be empty");
        return Ok(CliExitCode::Usage);
    }

    let address = match BackendAddress::from_connection_args(&cli.connection) {
        Ok(value) => value,
        Err(message) => {
            eprintln!("error: {message}");
            return Ok(CliExitCode::Usage);
        }
    };

    let client = build_client()?;
    // URL encoding the query manually if urlencoding crate is available.
    // Assuming reqwest can handle it if we construct the URL properly.
    // To avoid adding a dependency, we will just pass it to `reqwest` via query params.

    let response = client
        .get(address.url(SEARCH_ENDPOINT))
        .query(&[("q", &args.query)])
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
            "daemon at {} returned an unreadable search response: {error}",
            address.base_url()
        )
    })?;

    let search_result = parse_search_body(&body, &address.base_url())?;
    print_search_results(&search_result, cli.output.json, cli.output.quiet)?;
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

fn parse_search_body(body: &str, base_url: &str) -> Result<SearchResponse, String> {
    serde_json::from_str::<SearchResponse>(body).map_err(|error| {
        format!("daemon at {base_url} returned an invalid search response: {error}")
    })
}

fn unsupported_filter(args: &SearchArgs) -> Option<&'static str> {
    if args.limit.is_some() {
        return Some("search limit filtering is not wired yet");
    }
    if args.tag.is_some() {
        return Some("search tag filters are not wired yet");
    }
    None
}

fn print_search_results(response: &SearchResponse, json: bool, quiet: bool) -> Result<(), String> {
    if quiet {
        return Ok(());
    }
    if json {
        let encoded = serde_json::to_string(response)
            .map_err(|error| format!("could not encode search JSON: {error}"))?;
        println!("{encoded}");
        return Ok(());
    }

    if response.files.is_empty() {
        println!("no files matched");
        return Ok(());
    }

    for file in &response.files {
        println!(
            "{}\t{}\t{}\t{}",
            file.id,
            file.name,
            file.size_bytes,
            file.tags.join(",")
        );
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct SearchResponse {
    schema_version: u8,
    files: Vec<FileRecordResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FileRecordResponse {
    id: String,
    name: String,
    size_bytes: u64,
    tags: Vec<String>,
}
