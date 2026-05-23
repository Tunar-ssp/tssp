//! Implementation of `tssp info`.

use std::fmt::Write as _;

use reqwest::{header::ACCEPT, StatusCode};
use serde::{Deserialize, Serialize};
use tssp_cli_core::CliExitCode;

use crate::backend::{api_get, build_client, BackendAddress};
use tssp::{Cli, IdArgs};

/// Runs `tssp info <id>`.
pub(crate) fn run(cli: &Cli, args: &IdArgs) -> Result<CliExitCode, String> {
    let address = match BackendAddress::from_connection_args(&cli.connection) {
        Ok(value) => value,
        Err(message) => {
            eprintln!("error: {message}");
            return Ok(CliExitCode::Usage);
        }
    };
    let client = build_client()?;
    let response = api_get(&client, &info_url(&address, &args.id))
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
        print_status_error(response.status(), code, &args.id);
        return Ok(code);
    }

    let body = response.text().map_err(|error| {
        format!(
            "daemon at {} returned an unreadable file response: {error}",
            address.base_url()
        )
    })?;
    let file = parse_file_body(&body, &address.base_url())?;
    print_file(&file, cli.output.json, cli.output.quiet)?;
    Ok(CliExitCode::Success)
}

fn classify_response_status(status: StatusCode) -> Result<(), CliExitCode> {
    if status.is_server_error() {
        return Err(CliExitCode::Server);
    }
    if status.as_u16() == 404 {
        return Err(CliExitCode::NotFound);
    }
    if status.as_u16() == 400 {
        return Err(CliExitCode::Usage);
    }
    if !status.is_success() {
        return Err(CliExitCode::Generic);
    }
    Ok(())
}

fn print_status_error(status: StatusCode, code: CliExitCode, id: &str) {
    match code {
        CliExitCode::NotFound => eprintln!("error: file {id} was not found"),
        CliExitCode::Usage => eprintln!("error: file id is invalid"),
        _ => eprintln!("error: daemon returned {status}"),
    }
}

fn parse_file_body(body: &str, base_url: &str) -> Result<FileRecordResponse, String> {
    serde_json::from_str::<FileRecordResponse>(body)
        .map_err(|error| format!("daemon at {base_url} returned an invalid file response: {error}"))
}

pub(crate) fn info_url(address: &BackendAddress, id: &str) -> String {
    format!("{}/{}", address.url("/api/v1/files"), path_segment(id))
}

pub(crate) fn path_segment(id: &str) -> String {
    let mut encoded = String::new();
    for byte in id.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' => {
                encoded.push(char::from(byte));
            }
            other => {
                let _written = write!(encoded, "%{other:02X}");
            }
        }
    }
    encoded
}

fn print_file(file: &FileRecordResponse, json: bool, quiet: bool) -> Result<(), String> {
    if quiet {
        return Ok(());
    }
    if json {
        let encoded = serde_json::to_string(file)
            .map_err(|error| format!("could not encode file JSON: {error}"))?;
        println!("{encoded}");
        return Ok(());
    }

    println!("Id: {}", file.id);
    println!("Name: {}", file.name);
    println!("Size: {}", file.size_bytes);
    println!("Hash: {}", file.content_hash);
    println!("MIME: {}", file.mime_type);
    println!("Uploaded: {}", file.uploaded_at);
    println!("Tags: {}", file.tags.join(","));
    println!("Pinned: {}", file.pinned);
    Ok(())
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
        classify_response_status, info_url, parse_file_body, path_segment, print_file,
        FileRecordResponse,
    };
    use crate::backend::BackendAddress;
    use reqwest::StatusCode;
    use tssp::ConnectionArgs;
    use tssp_cli_core::CliExitCode;

    #[test]
    fn info_url_percent_encodes_unsafe_id_bytes() {
        let address = BackendAddress::from_connection_args(&ConnectionArgs {
            host: Some("127.0.0.1".to_owned()),
            port: Some(8421),
        })
        .unwrap_or_else(|error| panic!("address failed: {error}"));

        assert_eq!(
            info_url(&address, "bad id"),
            "http://127.0.0.1:8421/api/v1/files/bad%20id"
        );
    }

    #[test]
    fn path_segment_keeps_file_id_characters() {
        assert_eq!(path_segment("file-ABC_123"), "file-ABC_123");
    }

    #[test]
    fn response_status_maps_server_errors() {
        let result = classify_response_status(StatusCode::INTERNAL_SERVER_ERROR);

        assert_eq!(result, Err(CliExitCode::Server));
    }

    #[test]
    fn response_status_maps_not_found() {
        let result = classify_response_status(StatusCode::NOT_FOUND);

        assert_eq!(result, Err(CliExitCode::NotFound));
    }

    #[test]
    fn response_status_maps_invalid_ids() {
        let result = classify_response_status(StatusCode::BAD_REQUEST);

        assert_eq!(result, Err(CliExitCode::Usage));
    }

    #[test]
    fn response_status_maps_other_failures_to_generic() {
        let result = classify_response_status(StatusCode::CONFLICT);

        assert_eq!(result, Err(CliExitCode::Generic));
    }

    #[test]
    fn response_status_allows_success() {
        assert_eq!(classify_response_status(StatusCode::OK), Ok(()));
    }

    #[test]
    fn parse_file_body_accepts_file_payload() {
        let file = parse_file_body(
            r#"{"schema_version":1,"id":"file-1","name":"note.txt","size_bytes":5,"content_hash":"hash","mime_type":"text/plain","uploaded_at":1700000000,"tags":["Docs"],"pinned":true}"#,
            "http://127.0.0.1:8421",
        )
        .unwrap_or_else(|error| panic!("parse failed: {error}"));

        assert_eq!(file.id, "file-1");
        assert!(file.pinned);
    }

    #[test]
    fn parse_file_body_rejects_invalid_payload() {
        let result = parse_file_body(r#"{"schema_version":1}"#, "http://127.0.0.1:8421");

        assert!(matches!(result, Err(message) if message.contains("invalid file response")));
    }

    #[test]
    fn print_file_supports_quiet_json_and_human_output() {
        let file = file_record();

        assert_eq!(print_file(&file, false, true), Ok(()));
        assert_eq!(print_file(&file, true, false), Ok(()));
        assert_eq!(print_file(&file, false, false), Ok(()));
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
            pinned: true,
        }
    }
}
