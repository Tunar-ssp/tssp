//! Implementation of `tssp pull`.

use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use reqwest::blocking::Client;
use reqwest::header::ACCEPT;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tssp::PullArgs;
use tssp_cli_core::CliExitCode;

use crate::backend::{api_get, build_client, BackendAddress};
use tssp::Cli;

/// Runs `tssp pull <id|name>`.
pub(crate) fn run(cli: &Cli, args: &PullArgs) -> Result<CliExitCode, String> {
    if args.id_or_name.trim().is_empty() {
        eprintln!("error: pull target must not be empty");
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

    let targets = match resolve_download_targets(&client, &address, args)? {
        Ok(targets) => targets,
        Err(code) => return Ok(code),
    };
    let destinations =
        match plan_destination_paths(args.output.as_deref(), &targets, args.overwrite) {
            Ok(destinations) => destinations,
            Err(error) => {
                print_destination_error(&error);
                return Ok(error.exit_code());
            }
        };

    let mut downloaded = Vec::new();
    for (target, destination) in targets.iter().zip(destinations) {
        let path = match download_target(&client, &address, target, destination, args.overwrite) {
            Ok(path) => path,
            Err(code) => return Ok(code),
        };
        downloaded.push(path);
    }

    print_download_result(&downloaded, cli.output.json, cli.output.quiet);
    Ok(CliExitCode::Success)
}

fn resolve_download_targets(
    client: &Client,
    address: &BackendAddress,
    args: &PullArgs,
) -> Result<Result<Vec<FileRecordResponse>, CliExitCode>, String> {
    if let Some(record) = match fetch_file_metadata(client, address, &args.id_or_name)? {
        Ok(record) => record,
        Err(code) => return Ok(Err(code)),
    } {
        return Ok(Ok(vec![record]));
    }

    let mut matches = match fetch_filename_matches(client, address, &args.id_or_name)? {
        Ok(matches) => matches,
        Err(code) => return Ok(Err(code)),
    };
    if matches.is_empty() {
        eprintln!("error: file {} was not found", args.id_or_name);
        return Ok(Err(CliExitCode::NotFound));
    }

    if !args.all && matches.len() > 1 {
        let selected = matches.first().map_or("<unknown>", |file| file.id.as_str());
        eprintln!(
            "warning: multiple files named {}; downloading most recent match {selected}; use --all to download every match",
            args.id_or_name
        );
        matches.truncate(1);
    }

    Ok(Ok(matches))
}

fn fetch_file_metadata(
    client: &Client,
    address: &BackendAddress,
    id: &str,
) -> Result<Result<Option<FileRecordResponse>, CliExitCode>, String> {
    let response = api_get(&client, &crate::info::info_url(address, id))
        .header(ACCEPT, "application/vnd.tssp.v1+json")
        .send()
        .map_err(|error| {
            eprintln!(
                "error: could not look up {} at {}: {error}",
                id,
                address.base_url()
            );
            CliExitCode::Network
        });
    let response = match response {
        Ok(response) => response,
        Err(code) => return Ok(Err(code)),
    };

    let status = response.status();
    if status == StatusCode::NOT_FOUND || status == StatusCode::BAD_REQUEST {
        return Ok(Ok(None));
    }
    if let Err(code) = classify_metadata_response_status(status) {
        print_status_error(status, code, id);
        return Ok(Err(code));
    }

    let body = response.text().map_err(|error| {
        format!(
            "daemon at {} returned an unreadable file response: {error}",
            address.base_url()
        )
    })?;
    parse_file_body(&body, &address.base_url()).map(|record| Ok(Some(record)))
}

fn fetch_filename_matches(
    client: &Client,
    address: &BackendAddress,
    name: &str,
) -> Result<Result<Vec<FileRecordResponse>, CliExitCode>, String> {
    let mut cursor = None;
    let mut matches = Vec::new();

    loop {
        let mut request = api_get(&client, &address.url("/api/v1/files")).query(&[
            ("limit", "500"),
            ("sort", "-uploaded"),
            ("name", name),
        ]);
        if let Some(page) = cursor.as_deref() {
            request = request.query(&[("page", page)]);
        }

        let response = request
            .header(ACCEPT, "application/vnd.tssp.v1+json")
            .send()
            .map_err(|error| {
                eprintln!(
                    "error: could not search for {} at {}: {error}",
                    name,
                    address.base_url()
                );
                CliExitCode::Network
            });
        let response = match response {
            Ok(response) => response,
            Err(code) => return Ok(Err(code)),
        };

        if let Err(code) = classify_list_response_status(response.status()) {
            eprintln!("error: daemon returned {}", response.status());
            return Ok(Err(code));
        }

        let body = response.text().map_err(|error| {
            format!(
                "daemon at {} returned an unreadable list response: {error}",
                address.base_url()
            )
        })?;
        let page = parse_list_body(&body, &address.base_url())?;
        matches.extend(page.files.into_iter().filter(|file| file.name == name));

        let Some(next_cursor) = page.next_cursor else {
            break;
        };
        cursor = Some(next_cursor);
    }

    Ok(Ok(matches))
}

fn download_target(
    client: &Client,
    address: &BackendAddress,
    target: &FileRecordResponse,
    destination: PathBuf,
    overwrite: bool,
) -> Result<PathBuf, CliExitCode> {
    let response = api_get(&client, &content_url(address, &target.id))
        .send()
        .map_err(|error| {
            eprintln!(
                "error: could not download {} from {}: {error}",
                target.id,
                address.base_url()
            );
            CliExitCode::Network
        });
    let mut response = response?;

    if let Err(code) = classify_response_status(response.status()) {
        print_status_error(response.status(), code, &target.id);
        return Err(code);
    }

    if destination.exists() && !overwrite {
        eprintln!(
            "error: {} already exists; use --overwrite to replace it",
            destination.display()
        );
        return Err(CliExitCode::Conflict);
    }

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(overwrite)
        .create_new(!overwrite)
        .open(&destination)
        .map_err(|error| {
            eprintln!("error: could not create {}: {error}", destination.display());
            map_io_error(&error)
        });
    let mut file = file?;

    std::io::copy(&mut response, &mut file).map_err(|error| {
        eprintln!("error: could not write {}: {error}", destination.display());
        map_io_error(&error)
    })?;
    file.flush().map_err(|error| {
        eprintln!("error: could not flush {}: {error}", destination.display());
        map_io_error(&error)
    })?;
    file.sync_all().map_err(|error| {
        eprintln!("error: could not sync {}: {error}", destination.display());
        map_io_error(&error)
    })?;

    Ok(destination)
}

fn print_download_result(paths: &[PathBuf], json: bool, quiet: bool) {
    if quiet {
        return;
    }
    if json {
        if paths.len() == 1 {
            println!(
                "{{\"schema_version\":1,\"path\":\"{}\"}}",
                json_escape(&paths[0].display().to_string())
            );
        } else {
            let encoded_paths = paths
                .iter()
                .map(|path| format!("\"{}\"", json_escape(&path.display().to_string())))
                .collect::<Vec<_>>()
                .join(",");
            println!("{{\"schema_version\":1,\"paths\":[{encoded_paths}]}}");
        }
        return;
    }

    for path in paths {
        println!("downloaded {}", path.display());
    }
}

fn content_url(address: &BackendAddress, id: &str) -> String {
    format!("{}/content", crate::info::info_url(address, id))
}

fn classify_response_status(status: StatusCode) -> Result<(), CliExitCode> {
    if status.is_server_error() {
        return Err(CliExitCode::Server);
    }
    match status.as_u16() {
        200 | 206 => Ok(()),
        400 | 416 => Err(CliExitCode::Usage),
        404 => Err(CliExitCode::NotFound),
        409 => Err(CliExitCode::Conflict),
        _ => Err(CliExitCode::Generic),
    }
}

fn classify_metadata_response_status(status: StatusCode) -> Result<(), CliExitCode> {
    if status.is_server_error() {
        return Err(CliExitCode::Server);
    }
    if !status.is_success() {
        return Err(CliExitCode::Generic);
    }
    Ok(())
}

fn classify_list_response_status(status: StatusCode) -> Result<(), CliExitCode> {
    if status.is_server_error() {
        return Err(CliExitCode::Server);
    }
    if status == StatusCode::BAD_REQUEST {
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
        CliExitCode::Usage => eprintln!("error: daemon rejected the pull request with {status}"),
        _ => eprintln!("error: daemon returned {status}"),
    }
}

fn safe_download_name(name: &str) -> String {
    let name = name.replace(['/', '\\', '\r', '\n'], "_");
    let trimmed = name.trim_matches(['.', '_', ' ']);
    if trimmed.is_empty() {
        return "download.bin".to_owned();
    }
    trimmed.to_owned()
}

fn destination_path(output: Option<&Path>, remote_name: &str) -> PathBuf {
    let safe_name = safe_download_name(remote_name);
    match output {
        Some(path) if path.is_dir() => path.join(safe_name),
        Some(path) => path.to_path_buf(),
        None => PathBuf::from(safe_name),
    }
}

#[derive(Debug, PartialEq, Eq)]
enum DestinationPlanError {
    OutputMustBeDirectory { path: PathBuf },
}

impl DestinationPlanError {
    const fn exit_code(&self) -> CliExitCode {
        match self {
            Self::OutputMustBeDirectory { .. } => CliExitCode::Usage,
        }
    }
}

fn print_destination_error(error: &DestinationPlanError) {
    match error {
        DestinationPlanError::OutputMustBeDirectory { path } => {
            eprintln!(
                "error: pull --all requires --output to be a directory, got {}",
                path.display()
            );
        }
    }
}

fn plan_destination_paths(
    output: Option<&Path>,
    targets: &[FileRecordResponse],
    _overwrite: bool,
) -> Result<Vec<PathBuf>, DestinationPlanError> {
    if targets.len() <= 1 {
        return Ok(targets
            .iter()
            .map(|target| destination_path(output, &target.name))
            .collect());
    }

    let base_directory = match output {
        Some(path) if path.is_dir() => path.to_path_buf(),
        Some(path) => {
            return Err(DestinationPlanError::OutputMustBeDirectory {
                path: path.to_path_buf(),
            });
        }
        None => PathBuf::from("."),
    };

    let mut used_names = HashSet::new();
    Ok(targets
        .iter()
        .map(|target| {
            base_directory.join(unique_batch_download_name(
                &target.name,
                &target.id,
                &mut used_names,
            ))
        })
        .collect())
}

fn unique_batch_download_name(
    remote_name: &str,
    id: &str,
    used_names: &mut HashSet<String>,
) -> String {
    let safe_name = safe_download_name(remote_name);
    if used_names.insert(safe_name.clone()) {
        return safe_name;
    }

    let candidate = suffixed_download_name(&safe_name, id);
    if used_names.insert(candidate.clone()) {
        return candidate;
    }

    let mut index = 2_u64;
    loop {
        let candidate = suffixed_download_name(&safe_name, &format!("{id}-{index}"));
        if used_names.insert(candidate.clone()) {
            return candidate;
        }
        index = index.saturating_add(1);
    }
}

fn suffixed_download_name(safe_name: &str, id: &str) -> String {
    let Some(dot_index) = safe_name.rfind('.') else {
        return format!("{safe_name}-{id}");
    };
    if dot_index == 0 {
        return format!("{safe_name}-{id}");
    }
    format!(
        "{}-{}{}",
        &safe_name[..dot_index],
        id,
        &safe_name[dot_index..]
    )
}

fn parse_file_body(body: &str, base_url: &str) -> Result<FileRecordResponse, String> {
    serde_json::from_str::<FileRecordResponse>(body)
        .map_err(|error| format!("daemon at {base_url} returned an invalid file response: {error}"))
}

fn parse_list_body(body: &str, base_url: &str) -> Result<ListResponse, String> {
    serde_json::from_str::<ListResponse>(body)
        .map_err(|error| format!("daemon at {base_url} returned an invalid list response: {error}"))
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ListResponse {
    schema_version: u8,
    files: Vec<FileRecordResponse>,
    #[serde(default)]
    next_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

fn map_io_error(error: &std::io::Error) -> CliExitCode {
    if error.kind() == std::io::ErrorKind::PermissionDenied {
        return CliExitCode::PermissionDenied;
    }
    CliExitCode::Generic
}

fn json_escape(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| match character {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            other => vec![other],
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;
    use tempfile::tempdir;
    use tssp_cli_core::CliExitCode;

    use super::{
        classify_response_status, destination_path, json_escape, map_io_error, parse_file_body,
        parse_list_body, plan_destination_paths, safe_download_name, DestinationPlanError,
        FileRecordResponse,
    };

    #[test]
    fn response_status_maps_download_contract() {
        assert_eq!(classify_response_status(StatusCode::OK), Ok(()));
        assert_eq!(
            classify_response_status(StatusCode::PARTIAL_CONTENT),
            Ok(())
        );
        assert_eq!(
            classify_response_status(StatusCode::BAD_REQUEST),
            Err(CliExitCode::Usage)
        );
        assert_eq!(
            classify_response_status(StatusCode::RANGE_NOT_SATISFIABLE),
            Err(CliExitCode::Usage)
        );
        assert_eq!(
            classify_response_status(StatusCode::NOT_FOUND),
            Err(CliExitCode::NotFound)
        );
        assert_eq!(
            classify_response_status(StatusCode::INTERNAL_SERVER_ERROR),
            Err(CliExitCode::Server)
        );
    }

    #[test]
    fn safe_download_name_falls_back_for_empty_names() {
        assert_eq!(safe_download_name("../"), "download.bin");
        assert_eq!(safe_download_name("note.txt"), "note.txt");
    }

    #[test]
    fn destination_uses_directory_or_explicit_path() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let explicit = temp.path().join("explicit.txt");

        assert_eq!(
            destination_path(Some(temp.path()), "note.txt"),
            temp.path().join("note.txt")
        );
        assert_eq!(destination_path(Some(&explicit), "note.txt"), explicit);
        assert_eq!(
            destination_path(None, "note.txt"),
            std::path::PathBuf::from("note.txt")
        );
    }

    #[test]
    fn io_permission_errors_map_to_permission_exit_code() {
        let code = map_io_error(&std::io::Error::from(std::io::ErrorKind::PermissionDenied));

        assert_eq!(code, CliExitCode::PermissionDenied);
    }

    #[test]
    fn json_escape_handles_control_characters() {
        assert_eq!(json_escape("a\"\\\n\r\t"), "a\\\"\\\\\\n\\r\\t");
    }

    #[test]
    fn print_status_error_handles_variants() {
        use super::print_status_error;
        print_status_error(StatusCode::NOT_FOUND, CliExitCode::NotFound, "file-1");
        print_status_error(StatusCode::BAD_REQUEST, CliExitCode::Usage, "file-1");
        print_status_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            CliExitCode::Server,
            "file-1",
        );
    }

    #[test]
    fn classify_response_status_conflict_generic() {
        assert_eq!(
            classify_response_status(StatusCode::CONFLICT),
            Err(CliExitCode::Conflict)
        );
        assert_eq!(
            classify_response_status(StatusCode::FORBIDDEN),
            Err(CliExitCode::Generic)
        );
    }

    #[test]
    fn map_io_error_generic() {
        let code = map_io_error(&std::io::Error::from(std::io::ErrorKind::NotFound));
        assert_eq!(code, CliExitCode::Generic);
    }

    #[test]
    fn parse_file_and_list_bodies_accept_metadata_payloads() {
        let file = parse_file_body(FILE_JSON, "http://127.0.0.1:8421")
            .unwrap_or_else(|error| panic!("file parse failed: {error}"));
        let list = parse_list_body(
            &format!(r#"{{"schema_version":1,"files":[{FILE_JSON}],"next_cursor":"next"}}"#),
            "http://127.0.0.1:8421",
        )
        .unwrap_or_else(|error| panic!("list parse failed: {error}"));

        assert_eq!(file.id, "file-1");
        assert_eq!(list.files.len(), 1);
        assert_eq!(list.next_cursor.as_deref(), Some("next"));
    }

    #[test]
    fn plan_destination_paths_supports_all_with_duplicate_names() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let targets = vec![
            file_record("file-1", "report.txt"),
            file_record("file-2", "report.txt"),
            file_record("file-3", "photo.jpg"),
        ];

        let paths = plan_destination_paths(Some(temp.path()), &targets, false)
            .unwrap_or_else(|error| panic!("destination plan failed: {error:?}"));

        assert_eq!(
            paths,
            vec![
                temp.path().join("report.txt"),
                temp.path().join("report-file-2.txt"),
                temp.path().join("photo.jpg"),
            ]
        );
    }

    #[test]
    fn plan_destination_paths_rejects_file_output_for_multiple_targets() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let output_file = temp.path().join("download.bin");
        let targets = vec![
            file_record("file-1", "report.txt"),
            file_record("file-2", "other.txt"),
        ];

        let result = plan_destination_paths(Some(&output_file), &targets, false);
        let error = match result {
            Ok(paths) => panic!("expected output directory error, got {paths:?}"),
            Err(error) => error,
        };

        assert_eq!(
            error,
            DestinationPlanError::OutputMustBeDirectory { path: output_file }
        );
    }

    #[test]
    fn run_rejects_invalid_connection_string() {
        use tssp::{Cli, ConnectionArgs, LoggingArgs, OutputArgs, UploadArgs};
        let cli = Cli {
            output: OutputArgs {
                json: false,
                quiet: false,
                no_color: true,
            },
            logging: LoggingArgs { verbose: false },
            connection: ConnectionArgs {
                host: Some("bad/host".to_owned()),
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
        let args = tssp::PullArgs {
            id_or_name: "test".to_owned(),
            output: None,
            overwrite: false,
            all: false,
        };
        assert_eq!(super::run(&cli, &args), Ok(CliExitCode::Usage));
    }

    const FILE_JSON: &str = r#"{"schema_version":1,"id":"file-1","name":"note.txt","size_bytes":5,"content_hash":"hash","mime_type":"text/plain","uploaded_at":1700000000,"tags":["Docs"],"pinned":false}"#;

    fn file_record(id: &str, name: &str) -> FileRecordResponse {
        FileRecordResponse {
            schema_version: 1,
            id: id.to_owned(),
            name: name.to_owned(),
            size_bytes: 5,
            content_hash: "hash".to_owned(),
            mime_type: "text/plain".to_owned(),
            uploaded_at: 1_700_000_000,
            tags: vec!["Docs".to_owned()],
            pinned: false,
        }
    }
}
