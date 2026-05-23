//! Implementation of `tssp list` and `tssp last`.

use reqwest::{header::ACCEPT, StatusCode};
use serde::{Deserialize, Serialize};
use tssp_cli_core::CliExitCode;

use crate::backend::{build_client, BackendAddress};
use tssp::{Cli, LastArgs, ListArgs};

const LIST_ENDPOINT: &str = "/api/v1/files";
const DEFAULT_LIMIT: u16 = 50;
const MAX_LIMIT: u16 = 500;

/// Runs `tssp list`.
pub(crate) fn run_list(cli: &Cli, args: &ListArgs) -> Result<CliExitCode, String> {
    if let Some(message) = unsupported_filter(args) {
        eprintln!("error: {message}");
        return Ok(CliExitCode::Usage);
    }
    let limit = args.limit.unwrap_or(DEFAULT_LIMIT);

    // Convert first tag to query parameter for now, as backend supports one tag
    let tag = if args.tags.is_empty() {
        None
    } else {
        if args.tags.len() > 1 {
            return Err("list tag filter currently supports only one tag".to_owned());
        }
        Some(args.tags[0].clone())
    };

    run_recent_list(cli, limit, tag.as_deref())
}

/// Runs `tssp last`.
pub(crate) fn run_last(cli: &Cli, args: &LastArgs) -> Result<CliExitCode, String> {
    run_recent_list(cli, args.count, None)
}

fn run_recent_list(cli: &Cli, limit: u16, tag: Option<&str>) -> Result<CliExitCode, String> {
    if !(1..=MAX_LIMIT).contains(&limit) {
        eprintln!("error: limit must be between 1 and {MAX_LIMIT}");
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
    let mut request = client
        .get(address.url(LIST_ENDPOINT))
        .query(&[("limit", limit)]);
    if let Some(t) = tag {
        request = request.query(&[("tag", t)]);
    }

    let response = request
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
            "daemon at {} returned an unreadable list response: {error}",
            address.base_url()
        )
    })?;
    let listed = parse_list_body(&body, &address.base_url())?;
    print_list(&listed, cli.output.json, cli.output.quiet)?;
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

fn parse_list_body(body: &str, base_url: &str) -> Result<ListResponse, String> {
    serde_json::from_str::<ListResponse>(body)
        .map_err(|error| format!("daemon at {base_url} returned an invalid list response: {error}"))
}

fn unsupported_filter(args: &ListArgs) -> Option<&'static str> {
    if args.mime_prefix.is_some() {
        return Some("list MIME filters are not wired yet");
    }
    if args.since.is_some() {
        return Some("list time filters are not wired yet");
    }
    if args.sort.is_some() {
        return Some("list sorting is not wired yet");
    }
    if args.pinned {
        return Some("list pinned filtering is not wired yet");
    }
    if args.page.is_some() {
        return Some("list cursor pagination is not wired yet");
    }
    None
}

fn print_list(response: &ListResponse, json: bool, quiet: bool) -> Result<(), String> {
    if quiet {
        return Ok(());
    }
    if json {
        let encoded = serde_json::to_string(response)
            .map_err(|error| format!("could not encode list JSON: {error}"))?;
        println!("{encoded}");
        return Ok(());
    }

    if response.files.is_empty() {
        println!("no files");
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
struct ListResponse {
    schema_version: u8,
    files: Vec<FileRecordResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FileRecordResponse {
    schema_version: u8,
    id: String,
    name: String,
    size_bytes: u64,
    content_hash: String,
    mime_type: String,
    uploaded_at: i64,
    tags: Vec<String>,
    pinned: bool,
}

#[cfg(test)]
mod tests {
    use super::{
        classify_response_status, parse_list_body, print_list, unsupported_filter,
        FileRecordResponse, ListResponse,
    };
    use reqwest::StatusCode;
    use tssp::ListArgs;
    use tssp_cli_core::CliExitCode;

    #[test]
    fn unsupported_filter_rejects_each_unwired_filter() {
        let mut args = list_args();
        args.mime_prefix = Some("text/".to_owned());
        assert!(matches!(unsupported_filter(&args), Some(message) if message.contains("MIME")));

        let mut args = list_args();
        args.since = Some("1h".to_owned());
        assert!(matches!(unsupported_filter(&args), Some(message) if message.contains("time")));

        let mut args = list_args();
        args.sort = Some("name".to_owned());
        assert!(matches!(unsupported_filter(&args), Some(message) if message.contains("sorting")));

        let mut args = list_args();
        args.pinned = true;
        assert!(matches!(unsupported_filter(&args), Some(message) if message.contains("pinned")));

        let mut args = list_args();
        args.page = Some("cursor".to_owned());
        assert!(
            matches!(unsupported_filter(&args), Some(message) if message.contains("pagination"))
        );

        assert_eq!(unsupported_filter(&list_args()), None);
    }

    #[test]
    fn response_status_maps_server_errors() {
        let result = classify_response_status(StatusCode::SERVICE_UNAVAILABLE);

        assert_eq!(result, Err(CliExitCode::Server));
    }

    #[test]
    fn response_status_maps_non_success_to_generic() {
        let result = classify_response_status(StatusCode::UNAUTHORIZED);

        assert_eq!(result, Err(CliExitCode::Generic));
    }

    #[test]
    fn response_status_allows_success() {
        assert_eq!(classify_response_status(StatusCode::OK), Ok(()));
    }

    #[test]
    fn parse_list_body_accepts_files() {
        let response = parse_list_body(
            r#"{"schema_version":1,"files":[{"schema_version":1,"id":"file-1","name":"note.txt","size_bytes":5,"content_hash":"hash","mime_type":"text/plain","uploaded_at":1700000000,"tags":["Docs"],"pinned":false}]}"#,
            "http://127.0.0.1:8421",
        )
        .unwrap_or_else(|error| panic!("parse failed: {error}"));

        assert_eq!(response.files.len(), 1);
        assert_eq!(response.files[0].name, "note.txt");
    }

    #[test]
    fn parse_list_body_rejects_invalid_payload() {
        let result = parse_list_body(r#"{"schema_version":1}"#, "http://127.0.0.1:8421");

        assert!(matches!(result, Err(message) if message.contains("invalid list response")));
    }

    #[test]
    fn print_list_supports_quiet_json_empty_and_human_output() {
        let empty = ListResponse {
            schema_version: 1,
            files: Vec::new(),
        };
        let populated = ListResponse {
            schema_version: 1,
            files: vec![file_record()],
        };

        assert_eq!(print_list(&populated, false, true), Ok(()));
        assert_eq!(print_list(&populated, true, false), Ok(()));
        assert_eq!(print_list(&empty, false, false), Ok(()));
        assert_eq!(print_list(&populated, false, false), Ok(()));
    }

    fn list_args() -> ListArgs {
        ListArgs {
            tags: Vec::new(),
            mime_prefix: None,
            since: None,
            limit: None,
            sort: None,
            pinned: false,
            page: None,
        }
    }

    fn file_record() -> FileRecordResponse {
        FileRecordResponse {
            schema_version: 1,
            id: "file-1".to_owned(),
            name: "note.txt".to_owned(),
            size_bytes: 5,
            content_hash: "hash".to_owned(),
            mime_type: "text/plain".to_owned(),
            uploaded_at: 1_700_000_000,
            tags: vec!["Docs".to_owned()],
            pinned: false,
        }
    }
}
