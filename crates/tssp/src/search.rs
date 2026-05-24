//! Implementation of `tssp search`.

use reqwest::{header::ACCEPT, StatusCode};
use serde::{Deserialize, Serialize};
use tssp_cli_core::CliExitCode;

use crate::backend::{api_get, build_client, BackendAddress};
use tssp::{Cli, SearchArgs};

const SEARCH_ENDPOINT: &str = "/api/v1/search";

/// Runs `tssp search`.
pub(crate) fn run_search(cli: &Cli, args: &SearchArgs) -> Result<CliExitCode, String> {
    if let Some(message) = validate_args(args) {
        eprintln!("error: {message}");
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
    let mut query_params = vec![("q", args.query.clone())];
    if let Some(limit) = args.limit {
        query_params.push(("limit", limit.to_string()));
    }
    if let Some(tag) = &args.tag {
        query_params.push(("tag", tag.clone()));
    }

    let response = api_get(&client, &address.url(SEARCH_ENDPOINT))
        .query(&query_params)
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

    let search_result = apply_filters(parse_search_body(&body, &address.base_url())?, args);
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

fn validate_args(args: &SearchArgs) -> Option<&'static str> {
    if args.query.trim().is_empty() {
        return Some("search query must not be empty");
    }
    if matches!(args.limit, Some(0)) {
        return Some("search limit must be at least 1");
    }
    args.tag.as_deref().and_then(|tag| {
        normalize_tag(tag)
            .is_empty()
            .then_some("search tag filter must not be empty")
    })
}

fn apply_filters(mut response: SearchResponse, args: &SearchArgs) -> SearchResponse {
    if let Some(tag_filter) = args.tag.as_deref() {
        let normalized_filter = normalize_tag(tag_filter);
        response.results.retain(|item| match item {
            SearchResultItem::File { record: file } => file
                .tags
                .iter()
                .any(|tag| normalize_tag(tag) == normalized_filter),
            SearchResultItem::Note { record: note } => note
                .tags
                .iter()
                .any(|tag| normalize_tag(tag) == normalized_filter),
            SearchResultItem::Workspace { .. } => false,
        });
    }

    if let Some(limit) = args.limit {
        response.results.truncate(usize::from(limit));
    }

    response
}

fn normalize_tag(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
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

    let count = response.result_count.unwrap_or(response.results.len());
    if response.results.is_empty() {
        println!("no matches");
        return Ok(());
    }

    println!("{count} result{}:", if count == 1 { "" } else { "s" });
    for item in &response.results {
        match item {
            SearchResultItem::File { record: file } => {
                let tags = if file.tags.is_empty() {
                    String::new()
                } else {
                    format!("  [{}]", file.tags.join(", "))
                };
                println!("  file      {}  {}{}", file.id, file.name, tags);
            }
            SearchResultItem::Note { record: note } => {
                let tags = if note.tags.is_empty() {
                    String::new()
                } else {
                    format!("  [{}]", note.tags.join(", "))
                };
                println!("  note      {}  {}{}", note.id, note.title, tags);
                if !note.snippet.is_empty() {
                    println!(
                        "            {}",
                        note.snippet.lines().next().unwrap_or_default()
                    );
                }
            }
            SearchResultItem::Workspace { record: workspace } => {
                println!(
                    "  workspace {}  {}  ({})",
                    workspace.id, workspace.name, workspace.language
                );
                if !workspace.snippet.is_empty() {
                    println!(
                        "            {}",
                        workspace.snippet.lines().next().unwrap_or_default()
                    );
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct SearchResponse {
    schema_version: u8,
    #[serde(default)]
    result_count: Option<usize>,
    results: Vec<SearchResultItem>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum SearchResultItem {
    File {
        #[serde(flatten)]
        record: FileRecordResponse,
    },
    Note {
        #[serde(flatten)]
        record: NoteRecordResponse,
    },
    Workspace {
        #[serde(flatten)]
        record: WorkspaceRecordResponse,
    },
}

#[derive(Debug, Deserialize, Serialize)]
struct FileRecordResponse {
    id: String,
    name: String,
    size_bytes: u64,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct NoteRecordResponse {
    id: String,
    title: String,
    updated_at: i64,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    snippet: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct WorkspaceRecordResponse {
    id: String,
    name: String,
    language: String,
    updated_at: i64,
    snippet: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_response_status_identifies_success() {
        assert!(classify_response_status(StatusCode::OK).is_ok());
    }

    #[test]
    fn classify_response_status_identifies_server_error() {
        assert_eq!(
            classify_response_status(StatusCode::INTERNAL_SERVER_ERROR),
            Err(CliExitCode::Server)
        );
    }

    #[test]
    fn classify_response_status_identifies_generic_error() {
        assert_eq!(
            classify_response_status(StatusCode::BAD_REQUEST),
            Err(CliExitCode::Generic)
        );
    }

    #[test]
    fn parse_search_body_handles_valid_json() {
        let json = r#"{
            "schema_version": 1,
            "results": [
                {
                    "type": "file",
                    "id": "file-1",
                    "name": "report.pdf",
                    "size_bytes": 1024,
                    "tags": ["Docs"]
                }
            ]
        }"#;

        let result = parse_search_body(json, "http://localhost")
            .unwrap_or_else(|error| panic!("parse failed: {error}"));
        assert_eq!(result.schema_version, 1);
        assert_eq!(result.results.len(), 1);
        match &result.results[0] {
            SearchResultItem::File { record: file } => assert_eq!(file.id, "file-1"),
            SearchResultItem::Note { .. } | SearchResultItem::Workspace { .. } => {
                panic!("expected file result")
            }
        }
    }

    #[test]
    fn parse_search_body_handles_workspace_results() {
        let json = r#"{
            "schema_version": 1,
            "results": [
                {
                    "type": "workspace",
                    "id": "ws-1",
                    "owner_id": "user-tunar",
                    "name": "Ops",
                    "language": "markdown",
                    "updated_at": 1000,
                    "snippet": "backup notes"
                }
            ]
        }"#;

        let result = parse_search_body(json, "http://localhost")
            .unwrap_or_else(|error| panic!("parse failed: {error}"));

        match &result.results[0] {
            SearchResultItem::Workspace { record } => assert_eq!(record.name, "Ops"),
            SearchResultItem::File { .. } | SearchResultItem::Note { .. } => {
                panic!("expected workspace result")
            }
        }
    }

    #[test]
    fn parse_search_body_rejects_invalid_json() {
        let result = parse_search_body("invalid", "http://localhost");
        assert!(result.is_err());
    }

    #[test]
    fn validate_args_rejects_empty_query() {
        let args = SearchArgs {
            query: "   ".to_string(),
            limit: None,
            tag: None,
        };
        assert_eq!(validate_args(&args), Some("search query must not be empty"));
    }

    #[test]
    fn validate_args_rejects_zero_limit() {
        let args = SearchArgs {
            query: "test".to_string(),
            limit: Some(0),
            tag: None,
        };
        assert_eq!(
            validate_args(&args),
            Some("search limit must be at least 1")
        );
    }

    #[test]
    fn validate_args_rejects_empty_tag_filter() {
        let args = SearchArgs {
            query: "test".to_string(),
            limit: None,
            tag: Some(" \t ".to_string()),
        };
        assert_eq!(
            validate_args(&args),
            Some("search tag filter must not be empty")
        );
    }

    #[test]
    fn validate_args_allows_supported_filters() {
        let args = SearchArgs {
            query: "test".to_string(),
            limit: Some(10),
            tag: Some("Docs".to_string()),
        };
        assert_eq!(validate_args(&args), None);
    }

    #[test]
    fn apply_filters_limits_results() {
        let response = SearchResponse {
            schema_version: 1,
            result_count: None,
            results: vec![
                file_hit("id-1", "first.txt", &["Docs"]),
                file_hit("id-2", "second.txt", &["Notes"]),
            ],
        };

        let filtered = apply_filters(
            response,
            &SearchArgs {
                query: "test".to_string(),
                limit: Some(1),
                tag: None,
            },
        );

        assert_eq!(filtered.results.len(), 1);
        match &filtered.results[0] {
            SearchResultItem::File { record: file } => assert_eq!(file.id, "id-1"),
            SearchResultItem::Note { .. } | SearchResultItem::Workspace { .. } => {
                panic!("expected file")
            }
        }
    }

    #[test]
    fn apply_filters_matches_tags_case_insensitively() {
        let response = SearchResponse {
            schema_version: 1,
            result_count: None,
            results: vec![
                file_hit("id-1", "first.txt", &["Family Photos"]),
                file_hit("id-2", "second.txt", &["Docs"]),
            ],
        };

        let filtered = apply_filters(
            response,
            &SearchArgs {
                query: "family".to_string(),
                limit: None,
                tag: Some("  FAMILY   photos ".to_string()),
            },
        );

        assert_eq!(filtered.results.len(), 1);
        match &filtered.results[0] {
            SearchResultItem::File { record: file } => assert_eq!(file.id, "id-1"),
            SearchResultItem::Note { .. } | SearchResultItem::Workspace { .. } => {
                panic!("expected file")
            }
        }
    }

    #[test]
    fn normalize_tag_collapses_whitespace_and_case() {
        assert_eq!(normalize_tag("  FAMILY   photos "), "family photos");
    }

    #[test]
    fn print_search_results_quiet() {
        let response = SearchResponse {
            schema_version: 1,
            result_count: None,
            results: vec![],
        };
        assert!(print_search_results(&response, false, true).is_ok());
    }

    #[test]
    fn print_search_results_json() {
        let response = SearchResponse {
            schema_version: 1,
            result_count: None,
            results: vec![],
        };
        assert!(print_search_results(&response, true, false).is_ok());
    }

    #[test]
    fn print_search_results_empty() {
        let response = SearchResponse {
            schema_version: 1,
            result_count: None,
            results: vec![],
        };
        assert!(print_search_results(&response, false, false).is_ok());
    }

    #[test]
    fn print_search_results_with_files() {
        let response = SearchResponse {
            schema_version: 1,
            result_count: None,
            results: vec![file_hit("id1", "test.txt", &["Docs"])],
        };
        assert!(print_search_results(&response, false, false).is_ok());
    }

    #[test]
    fn print_search_results_with_workspace() {
        let response = SearchResponse {
            schema_version: 1,
            result_count: None,
            results: vec![workspace_hit("ws-1", "Ops")],
        };
        assert!(print_search_results(&response, false, false).is_ok());
    }

    #[test]
    fn run_search_rejects_zero_limit() {
        use tssp::{Cli, ConnectionArgs, LoggingArgs, OutputArgs, UploadArgs};
        let cli = Cli {
            output: OutputArgs {
                json: false,
                quiet: false,
                no_color: true,
            },
            logging: LoggingArgs { verbose: false },
            connection: ConnectionArgs {
                host: None,
                port: None,
            },
            upload: UploadArgs {
                tags: Vec::new(),
                pin: false,
                rename: None,
                parallel: None,
                recursive: None,
                all: false,
                files: Vec::new(),
            },
            command: None,
        };
        let args = SearchArgs {
            query: "test".to_string(),
            limit: Some(0),
            tag: None,
        };
        assert_eq!(super::run_search(&cli, &args), Ok(CliExitCode::Usage));
    }

    #[test]
    fn run_search_rejects_empty_query() {
        use tssp::{Cli, ConnectionArgs, LoggingArgs, OutputArgs, UploadArgs};
        let cli = Cli {
            output: OutputArgs {
                json: false,
                quiet: false,
                no_color: true,
            },
            logging: LoggingArgs { verbose: false },
            connection: ConnectionArgs {
                host: None,
                port: None,
            },
            upload: UploadArgs {
                tags: Vec::new(),
                pin: false,
                rename: None,
                parallel: None,
                recursive: None,
                all: false,
                files: Vec::new(),
            },
            command: None,
        };
        let args = SearchArgs {
            query: "   ".to_string(),
            limit: None,
            tag: None,
        };
        assert_eq!(super::run_search(&cli, &args), Ok(CliExitCode::Usage));
    }

    #[test]
    fn run_search_rejects_invalid_connection_args() {
        use tssp::{Cli, ConnectionArgs, LoggingArgs, OutputArgs, UploadArgs};
        let cli = Cli {
            output: OutputArgs {
                json: false,
                quiet: false,
                no_color: true,
            },
            logging: LoggingArgs { verbose: false },
            connection: ConnectionArgs {
                host: Some("bad/host".to_string()),
                port: None,
            },
            upload: UploadArgs {
                tags: Vec::new(),
                pin: false,
                rename: None,
                parallel: None,
                recursive: None,
                all: false,
                files: Vec::new(),
            },
            command: None,
        };
        let args = SearchArgs {
            query: "test".to_string(),
            limit: None,
            tag: None,
        };
        assert_eq!(super::run_search(&cli, &args), Ok(CliExitCode::Usage));
    }

    fn file_hit(id: &str, name: &str, tags: &[&str]) -> SearchResultItem {
        SearchResultItem::File {
            record: FileRecordResponse {
                id: id.to_string(),
                name: name.to_string(),
                size_bytes: 123,
                tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
            },
        }
    }

    fn workspace_hit(id: &str, name: &str) -> SearchResultItem {
        SearchResultItem::Workspace {
            record: WorkspaceRecordResponse {
                id: id.to_string(),
                name: name.to_string(),
                language: "text".to_string(),
                updated_at: 1000,
                snippet: "saved text".to_string(),
            },
        }
    }
}
