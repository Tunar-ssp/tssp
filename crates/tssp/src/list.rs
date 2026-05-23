//! Implementation of `tssp list` and `tssp last`.

use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::{header::ACCEPT, StatusCode};
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tssp_cli_core::CliExitCode;

use crate::backend::{build_client, BackendAddress};
use tssp::{Cli, LastArgs, ListArgs};

const LIST_ENDPOINT: &str = "/api/v1/files";
const DEFAULT_LIMIT: u16 = 50;
const MAX_LIMIT: u16 = 500;

/// Runs `tssp list`.
pub(crate) fn run_list(cli: &Cli, args: &ListArgs) -> Result<CliExitCode, String> {
    let query = match build_list_query(args) {
        Ok(query) => query,
        Err(message) => {
            eprintln!("error: {message}");
            return Ok(CliExitCode::Usage);
        }
    };

    run_list_request(cli, &query)
}

/// Runs `tssp last`.
pub(crate) fn run_last(cli: &Cli, args: &LastArgs) -> Result<CliExitCode, String> {
    let query = match build_last_query(args) {
        Ok(query) => query,
        Err(message) => {
            eprintln!("error: {message}");
            return Ok(CliExitCode::Usage);
        }
    };

    run_list_request(cli, &query)
}

/// Runs `tssp today`.
pub(crate) fn run_today(cli: &Cli) -> Result<CliExitCode, String> {
    let query = match build_today_query() {
        Ok(query) => query,
        Err(message) => {
            eprintln!("error: {message}");
            return Ok(CliExitCode::Usage);
        }
    };

    run_list_request(cli, &query)
}

fn run_list_request(cli: &Cli, query: &[(String, String)]) -> Result<CliExitCode, String> {
    let address = match BackendAddress::from_connection_args(&cli.connection) {
        Ok(value) => value,
        Err(message) => {
            eprintln!("error: {message}");
            return Ok(CliExitCode::Usage);
        }
    };

    let client = build_client()?;
    let mut request = client.get(address.url(LIST_ENDPOINT));
    for (name, value) in query {
        request = request.query(&[(name.as_str(), value.as_str())]);
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

fn build_list_query(args: &ListArgs) -> Result<Vec<(String, String)>, String> {
    let limit = args.limit.unwrap_or(DEFAULT_LIMIT);
    validate_limit(limit)?;

    let mut query = vec![("limit".to_owned(), limit.to_string())];
    for tag in &args.tags {
        query.push((
            "tag".to_owned(),
            validate_non_empty(tag, "list tag filter")?,
        ));
    }

    if let Some(mime_prefix) = args.mime_prefix.as_deref() {
        query.push((
            "type".to_owned(),
            validate_non_empty(mime_prefix, "list MIME filter")?,
        ));
    }
    if let Some(since) = args.since.as_deref() {
        query.push(("since".to_owned(), parse_since_filter(since)?.to_string()));
    }
    if let Some(sort) = args.sort.as_deref() {
        query.push(("sort".to_owned(), validate_sort(sort)?));
    }
    if args.pinned {
        query.push(("pinned".to_owned(), "true".to_owned()));
    }
    if let Some(page) = args.page.as_deref() {
        query.push(("page".to_owned(), validate_non_empty(page, "list cursor")?));
    }

    Ok(query)
}

fn build_last_query(args: &LastArgs) -> Result<Vec<(String, String)>, String> {
    validate_limit(args.count)?;
    Ok(vec![
        ("limit".to_owned(), args.count.to_string()),
        ("sort".to_owned(), "-uploaded".to_owned()),
    ])
}

fn build_today_query() -> Result<Vec<(String, String)>, String> {
    let now = OffsetDateTime::now_local()
        .map_err(|error| format!("could not determine local time for `tssp today`: {error}"))?;
    build_today_query_at(now)
}

fn build_today_query_at(now: OffsetDateTime) -> Result<Vec<(String, String)>, String> {
    Ok(vec![
        ("limit".to_owned(), DEFAULT_LIMIT.to_string()),
        (
            "since".to_owned(),
            start_of_local_day_timestamp(now)?.to_string(),
        ),
    ])
}

fn validate_limit(limit: u16) -> Result<(), String> {
    if !(1..=MAX_LIMIT).contains(&limit) {
        return Err(format!("limit must be between 1 and {MAX_LIMIT}"));
    }
    Ok(())
}

fn validate_non_empty(value: &str, field: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    Ok(trimmed.to_owned())
}

fn validate_sort(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    match trimmed {
        "uploaded" | "-uploaded" | "name" | "-name" | "size" | "-size" => Ok(trimmed.to_owned()),
        _ => {
            Err("list sort must be one of uploaded, -uploaded, name, -name, size, -size".to_owned())
        }
    }
}

fn parse_since_filter(value: &str) -> Result<i64, String> {
    parse_since_filter_at(value, unix_now_seconds()?)
}

fn parse_since_filter_at(value: &str, now: i64) -> Result<i64, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("list since filter must not be empty".to_owned());
    }

    if let Ok(timestamp) = trimmed.parse::<i64>() {
        return validate_since_timestamp(timestamp);
    }

    if let Some(duration_seconds) = parse_relative_duration_seconds(trimmed)? {
        return now
            .checked_sub(duration_seconds)
            .ok_or_else(|| "list since duration reaches before the Unix epoch".to_owned());
    }

    let timestamp = OffsetDateTime::parse(trimmed, &Rfc3339)
        .map_err(|_| {
            "list since filter must be a relative duration, UNIX timestamp, or RFC3339 timestamp"
                .to_owned()
        })?
        .unix_timestamp();
    validate_since_timestamp(timestamp)
}

fn validate_since_timestamp(timestamp: i64) -> Result<i64, String> {
    if timestamp < 0 {
        return Err("list since filter must not be before the Unix epoch".to_owned());
    }
    Ok(timestamp)
}

fn parse_relative_duration_seconds(value: &str) -> Result<Option<i64>, String> {
    if value.len() < 2 {
        return Ok(None);
    }

    let (amount_text, unit_text) = value.split_at(value.len() - 1);
    let unit = unit_text
        .chars()
        .next()
        .map(|character| character.to_ascii_lowercase());
    let multiplier = match unit {
        Some('s') => 1_u64,
        Some('m') => 60_u64,
        Some('h') => 3_600_u64,
        Some('d') => 86_400_u64,
        Some('w') => 604_800_u64,
        _ => return Ok(None),
    };

    let amount = amount_text.parse::<u64>().map_err(|error| {
        format!("list since duration has an invalid amount `{amount_text}`: {error}")
    })?;
    let seconds = amount
        .checked_mul(multiplier)
        .ok_or_else(|| "list since duration is too large".to_owned())?;
    i64::try_from(seconds)
        .map(Some)
        .map_err(|error| format!("list since duration is too large: {error}"))
}

fn unix_now_seconds() -> Result<i64, String> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock is before the Unix epoch: {error}"))?;
    i64::try_from(duration.as_secs())
        .map_err(|error| format!("system clock is too large for list filtering: {error}"))
}

fn start_of_local_day_timestamp(now: OffsetDateTime) -> Result<i64, String> {
    let start = now
        .date()
        .midnight()
        .assume_offset(now.offset())
        .unix_timestamp();
    validate_since_timestamp(start)
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
            "{}\t{}\t{}\t{}\t{}",
            file.id,
            file.name,
            file.size_bytes,
            file.uploaded_at,
            file.tags.join(",")
        );
    }
    if let Some(cursor) = &response.next_cursor {
        println!("next page: {cursor}");
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct ListResponse {
    schema_version: u8,
    files: Vec<FileRecordResponse>,
    #[serde(default)]
    next_cursor: Option<String>,
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
        build_last_query, build_list_query, build_today_query_at, classify_response_status,
        parse_list_body, parse_since_filter_at, print_list, FileRecordResponse, ListResponse,
    };
    use reqwest::StatusCode;
    use time::format_description::well_known::Rfc3339;
    use time::OffsetDateTime;
    use tssp::ListArgs;
    use tssp_cli_core::CliExitCode;

    #[test]
    fn build_list_query_supports_full_filter_set() {
        let args = ListArgs {
            tags: vec!["Docs".to_owned(), "Family".to_owned()],
            mime_prefix: Some("image".to_owned()),
            since: Some("1970-01-02T00:00:00Z".to_owned()),
            limit: Some(25),
            sort: Some("-name".to_owned()),
            pinned: true,
            page: Some("cursor-1".to_owned()),
        };

        let query = build_list_query(&args).unwrap_or_else(|error| panic!("build failed: {error}"));

        assert_eq!(
            query,
            vec![
                ("limit".to_owned(), "25".to_owned()),
                ("tag".to_owned(), "Docs".to_owned()),
                ("tag".to_owned(), "Family".to_owned()),
                ("type".to_owned(), "image".to_owned()),
                ("since".to_owned(), "86400".to_owned()),
                ("sort".to_owned(), "-name".to_owned()),
                ("pinned".to_owned(), "true".to_owned()),
                ("page".to_owned(), "cursor-1".to_owned()),
            ]
        );
    }

    #[test]
    fn build_list_query_rejects_invalid_values() {
        let mut args = list_args();
        args.tags = vec!["   ".to_owned()];
        assert!(matches!(
            build_list_query(&args),
            Err(message) if message.contains("tag filter")
        ));

        let mut args = list_args();
        args.sort = Some("rank".to_owned());
        assert!(matches!(
            build_list_query(&args),
            Err(message) if message.contains("sort")
        ));

        let mut args = list_args();
        args.page = Some("   ".to_owned());
        assert!(matches!(
            build_list_query(&args),
            Err(message) if message.contains("cursor")
        ));
    }

    #[test]
    fn build_last_query_uses_descending_uploaded_sort() {
        let query = build_last_query(&tssp::LastArgs { count: 7 })
            .unwrap_or_else(|error| panic!("build failed: {error}"));

        assert_eq!(
            query,
            vec![
                ("limit".to_owned(), "7".to_owned()),
                ("sort".to_owned(), "-uploaded".to_owned()),
            ]
        );
    }

    #[test]
    fn build_today_query_uses_local_start_of_day() {
        let now = OffsetDateTime::parse("2026-05-23T13:45:00+04:00", &Rfc3339)
            .unwrap_or_else(|error| panic!("parse failed: {error}"));
        let expected_since = OffsetDateTime::parse("2026-05-23T00:00:00+04:00", &Rfc3339)
            .unwrap_or_else(|error| panic!("parse failed: {error}"))
            .unix_timestamp()
            .to_string();

        let query =
            build_today_query_at(now).unwrap_or_else(|error| panic!("build failed: {error}"));

        assert_eq!(
            query,
            vec![
                ("limit".to_owned(), "50".to_owned()),
                ("since".to_owned(), expected_since),
            ]
        );
    }

    #[test]
    fn parse_since_filter_at_supports_relative_rfc3339_and_unix_values() {
        assert_eq!(parse_since_filter_at("2h", 10_000), Ok(2_800));
        assert_eq!(
            parse_since_filter_at("1970-01-02T00:00:00Z", 10_000),
            Ok(86_400)
        );
        assert_eq!(parse_since_filter_at("123", 10_000), Ok(123));
    }

    #[test]
    fn parse_since_filter_at_rejects_invalid_values() {
        assert!(matches!(
            parse_since_filter_at("  ", 10_000),
            Err(message) if message.contains("must not be empty")
        ));
        assert!(matches!(
            parse_since_filter_at("3x", 10_000),
            Err(message) if message.contains("RFC3339")
        ));
        assert!(matches!(
            parse_since_filter_at("-1", 10_000),
            Err(message) if message.contains("Unix epoch")
        ));
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
    fn parse_list_body_accepts_files_and_next_cursor() {
        let response = parse_list_body(
            r#"{"schema_version":1,"files":[{"schema_version":1,"id":"file-1","name":"note.txt","size_bytes":5,"content_hash":"hash","mime_type":"text/plain","uploaded_at":1700000000,"tags":["Docs"],"pinned":false}],"next_cursor":"cursor-1"}"#,
            "http://127.0.0.1:8421",
        )
        .unwrap_or_else(|error| panic!("parse failed: {error}"));

        assert_eq!(response.files.len(), 1);
        assert_eq!(response.files[0].name, "note.txt");
        assert_eq!(response.next_cursor.as_deref(), Some("cursor-1"));
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
            next_cursor: None,
        };
        let populated = ListResponse {
            schema_version: 1,
            files: vec![file_record()],
            next_cursor: Some("cursor-1".to_owned()),
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
