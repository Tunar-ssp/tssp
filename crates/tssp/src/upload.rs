//! Implementation of the default `tssp <file>` upload action.

use std::collections::VecDeque;
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use reqwest::blocking::Body;
use reqwest::header::{CONTENT_TYPE, LOCATION};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tssp_cli_core::CliExitCode;

use crate::backend::{build_client, BackendAddress};
use tssp::{Cli, UploadArgs};

const MULTIPART_BOUNDARY_PREFIX: &str = "tssp-upload";
const UPLOAD_ENDPOINT: &str = "/api/v1/files";

/// Runs the default upload action.
pub(crate) fn run(cli: &Cli) -> Result<CliExitCode, String> {
    let plan = match UploadPlan::from_args(&cli.upload) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("error: {}", error.message);
            return Ok(error.code);
        }
    };
    let address = match BackendAddress::from_connection_args(&cli.connection) {
        Ok(value) => value,
        Err(message) => {
            eprintln!("error: {message}");
            return Ok(CliExitCode::Usage);
        }
    };
    let client = build_client()?;

    if plan.items.len() > 1 {
        upload_batch(&client, &address, &plan.items, cli)
    } else {
        Ok(upload_single(&client, &address, &plan.items, cli))
    }
}

fn upload_single(
    client: &reqwest::blocking::Client,
    address: &BackendAddress,
    items: &[UploadItem],
    cli: &Cli,
) -> CliExitCode {
    let mut successes = 0_usize;
    let mut last_error = CliExitCode::Success;

    for item in items {
        match upload_one(client, address, item, cli) {
            Ok(()) => successes = successes.saturating_add(1),
            Err(code) => last_error = code,
        }
    }

    if successes == items.len() {
        return CliExitCode::Success;
    }
    if successes > 0 {
        return CliExitCode::PartialSuccess;
    }
    last_error
}

fn upload_batch(
    client: &reqwest::blocking::Client,
    address: &BackendAddress,
    items: &[UploadItem],
    cli: &Cli,
) -> Result<CliExitCode, String> {
    let started_at = Instant::now();
    let boundary = multipart_boundary();

    let mut form_data = Vec::new();
    for item in items {
        for tag in &item.tags {
            form_data.extend(text_field(&boundary, "tag", tag));
        }
        if item.pinned {
            form_data.extend(text_field(&boundary, "pin", "true"));
        }
        form_data.extend(file_header(&boundary, &item.filename));

        let mut file = File::open(&item.path)
            .map_err(|error| format!("could not open {}: {error}", item.path.display()))?;
        file.read_to_end(&mut form_data)
            .map_err(|error| format!("could not read {}: {error}", item.path.display()))?;
        form_data.extend(b"\r\n");
    }
    form_data.extend(format!("--{boundary}--\r\n").as_bytes());

    let response = client
        .post(address.url("/api/v1/files/batch"))
        .header(
            CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(form_data))
        .send()
        .map_err(|error| format!("could not upload batch to {}: {error}", address.base_url()))?;

    handle_batch_response(response, items, cli, started_at)
}

struct UploadPlan {
    items: Vec<UploadItem>,
}

impl UploadPlan {
    fn from_args(args: &UploadArgs) -> Result<Self, UploadInputError> {
        if args.all {
            return Self::from_all_files(args);
        }
        if let Some(root) = &args.recursive {
            return Self::from_recursive(root, args);
        }
        if args.files.is_empty() {
            return Err(UploadInputError::usage("no files specified for upload"));
        }
        if args.rename.is_some() && args.files.len() > 1 {
            return Err(UploadInputError::usage(
                "--rename can only be used with one file",
            ));
        }

        let items = args
            .files
            .iter()
            .map(|path| UploadItem::from_path(path, args))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { items })
    }

    fn from_all_files(args: &UploadArgs) -> Result<Self, UploadInputError> {
        let current_dir = std::env::current_dir().map_err(|error| {
            UploadInputError::from_io_error(
                format!("could not read current directory: {error}"),
                &error,
            )
        })?;

        let mut items = Vec::new();
        for entry in std::fs::read_dir(&current_dir).map_err(|error| {
            UploadInputError::from_io_error(
                format!("could not list current directory: {error}"),
                &error,
            )
        })? {
            let entry = entry.map_err(|error| {
                UploadInputError::from_io_error(
                    format!("could not read directory entry: {error}"),
                    &error,
                )
            })?;
            let path = entry.path();
            if path.is_file() && !is_hidden(&path) {
                items.push(UploadItem::from_path(&path, args)?);
            }
        }

        if items.is_empty() {
            return Err(UploadInputError::usage(
                "no non-hidden files found in current directory",
            ));
        }

        Ok(Self { items })
    }

    fn from_recursive(root: &Path, args: &UploadArgs) -> Result<Self, UploadInputError> {
        if !root.is_dir() {
            return Err(UploadInputError::usage(format!(
                "{} is not a directory",
                root.display()
            )));
        }

        let mut items = Vec::new();
        let mut queue = VecDeque::from([root.to_path_buf()]);

        while let Some(dir) = queue.pop_front() {
            for entry in std::fs::read_dir(&dir).map_err(|error| {
                UploadInputError::from_io_error(
                    format!("could not list {}: {error}", dir.display()),
                    &error,
                )
            })? {
                let entry = entry.map_err(|error| {
                    UploadInputError::from_io_error(
                        format!("could not read directory entry: {error}"),
                        &error,
                    )
                })?;
                let path = entry.path();
                if !is_hidden(&path) {
                    if path.is_dir() {
                        queue.push_back(path);
                    } else if path.is_file() {
                        items.push(UploadItem::from_path(&path, args)?);
                    }
                }
            }
        }

        if items.is_empty() {
            return Err(UploadInputError::usage(
                "no files found in the specified directory tree",
            ));
        }

        Ok(Self { items })
    }
}

#[derive(Clone)]
struct UploadItem {
    path: PathBuf,
    filename: String,
    size_bytes: u64,
    tags: Vec<String>,
    pinned: bool,
}

impl UploadItem {
    fn from_path(path: &Path, args: &UploadArgs) -> Result<Self, UploadInputError> {
        let metadata = std::fs::symlink_metadata(path).map_err(|error| {
            UploadInputError::from_io_error(
                format!("could not read {}: {error}", path.display()),
                &error,
            )
        })?;
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            return Err(UploadInputError::usage(format!(
                "{} is a symlink; symlinks are not followed",
                path.display()
            )));
        }
        if file_type.is_dir() {
            return Err(UploadInputError::usage(format!(
                "{} is a directory; use -r to upload folders",
                path.display()
            )));
        }
        if !file_type.is_file() {
            return Err(UploadInputError::usage(format!(
                "{} is not a regular file",
                path.display()
            )));
        }

        Ok(Self {
            path: path.to_path_buf(),
            filename: upload_filename(path, args).map_err(UploadInputError::usage)?,
            size_bytes: metadata.len(),
            tags: args.tags.clone(),
            pinned: args.pin,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UploadInputError {
    code: CliExitCode,
    message: String,
}

impl UploadInputError {
    fn usage(message: impl Into<String>) -> Self {
        Self {
            code: CliExitCode::Usage,
            message: message.into(),
        }
    }

    fn from_io_error(message: String, error: &std::io::Error) -> Self {
        let code = match error.kind() {
            std::io::ErrorKind::NotFound => CliExitCode::NotFound,
            std::io::ErrorKind::PermissionDenied => CliExitCode::PermissionDenied,
            _ => CliExitCode::Generic,
        };
        Self { code, message }
    }
}

fn upload_one(
    client: &reqwest::blocking::Client,
    address: &BackendAddress,
    item: &UploadItem,
    cli: &Cli,
) -> Result<(), CliExitCode> {
    let started_at = Instant::now();
    let file = File::open(&item.path).map_err(|error| {
        eprintln!("error: could not open {}: {error}", item.path.display());
        map_io_error(&error)
    })?;
    let boundary = multipart_boundary();
    let body = MultipartUploadBody::new(&boundary, item, file);
    let response = client
        .post(address.url(UPLOAD_ENDPOINT))
        .header(
            CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::new(body))
        .send()
        .map_err(|error| {
            eprintln!(
                "error: could not upload {} to {}: {error}",
                item.path.display(),
                address.base_url()
            );
            CliExitCode::Network
        })?;

    handle_upload_response(response, item, cli, started_at)
}

fn handle_upload_response(
    response: reqwest::blocking::Response,
    item: &UploadItem,
    cli: &Cli,
    started_at: Instant,
) -> Result<(), CliExitCode> {
    let status = response.status();
    if let Err(code) = classify_upload_status(status) {
        eprintln!(
            "error: daemon returned {status} while uploading {}",
            item.path.display()
        );
        return Err(code);
    }

    let deduplicated = parse_deduplicated_header(
        response
            .headers()
            .get("x-tssp-deduplicated")
            .and_then(|value| value.to_str().ok()),
    );
    let location = response
        .headers()
        .get(LOCATION)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let body = response.text().map_err(|error| {
        eprintln!("error: daemon returned unreadable upload response: {error}");
        CliExitCode::Server
    })?;
    let record = parse_upload_response_body(&body)?;

    print_upload_result(&record, item, deduplicated, location, started_at, cli)
}

fn classify_upload_status(status: StatusCode) -> Result<(), CliExitCode> {
    if status.is_server_error() {
        return Err(CliExitCode::Server);
    }
    if !status.is_success() {
        return Err(map_client_status(status));
    }
    Ok(())
}

fn parse_deduplicated_header(value: Option<&str>) -> bool {
    value.is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

fn parse_upload_response_body(body: &str) -> Result<FileRecordResponse, CliExitCode> {
    serde_json::from_str::<FileRecordResponse>(body).map_err(|error| {
        eprintln!("error: daemon returned invalid upload response: {error}");
        CliExitCode::Server
    })
}

fn print_upload_result(
    record: &FileRecordResponse,
    item: &UploadItem,
    deduplicated: bool,
    location: Option<String>,
    started_at: Instant,
    cli: &Cli,
) -> Result<(), CliExitCode> {
    if cli.output.quiet {
        return Ok(());
    }

    let duration = started_at.elapsed();
    let throughput = throughput_bytes_per_second(item.size_bytes, duration);
    if cli.output.json {
        let output = UploadOutput::new(record, deduplicated, duration.as_millis(), throughput);
        let encoded = serde_json::to_string(&output).map_err(|error| {
            eprintln!("error: could not encode upload JSON: {error}");
            CliExitCode::Generic
        })?;
        println!("{encoded}");
        return Ok(());
    }

    let duplicate = if deduplicated { " deduplicated" } else { "" };
    let location = location.unwrap_or_else(|| format!("/api/v1/files/{}", record.id));
    println!(
        "uploaded {} id={} size={}{} duration_ms={} throughput_bps={} location={}",
        record.name,
        record.id,
        record.size_bytes,
        duplicate,
        duration.as_millis(),
        throughput,
        location
    );
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

#[derive(Debug, Serialize)]
struct UploadOutput<'a> {
    schema_version: u8,
    id: &'a str,
    name: &'a str,
    size_bytes: u64,
    content_hash: &'a str,
    deduplicated: bool,
    duration_ms: u128,
    throughput_bytes_per_second: u64,
}

impl<'a> UploadOutput<'a> {
    fn new(
        record: &'a FileRecordResponse,
        deduplicated: bool,
        duration_ms: u128,
        throughput_bytes_per_second: u64,
    ) -> Self {
        Self {
            schema_version: 1,
            id: &record.id,
            name: &record.name,
            size_bytes: record.size_bytes,
            content_hash: &record.content_hash,
            deduplicated,
            duration_ms,
            throughput_bytes_per_second,
        }
    }
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
}

fn upload_filename(path: &Path, args: &UploadArgs) -> Result<String, String> {
    if let Some(rename) = &args.rename {
        return Ok(rename.clone());
    }
    let filename = path
        .file_name()
        .ok_or_else(|| format!("{} does not have a file name", path.display()))?;
    Ok(filename.to_string_lossy().into_owned())
}

fn multipart_boundary() -> String {
    format!(
        "{}-{}-{}",
        MULTIPART_BOUNDARY_PREFIX,
        std::process::id(),
        chrono_like_timestamp()
    )
}

fn chrono_like_timestamp() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos())
}

fn map_io_error(error: &std::io::Error) -> CliExitCode {
    if error.kind() == std::io::ErrorKind::PermissionDenied {
        return CliExitCode::PermissionDenied;
    }
    CliExitCode::Generic
}

fn handle_batch_response(
    response: reqwest::blocking::Response,
    items: &[UploadItem],
    cli: &Cli,
    started_at: Instant,
) -> Result<CliExitCode, String> {
    let status = response.status();
    if status.is_server_error() {
        return Err(format!("daemon returned {status} during batch upload",));
    }
    if !status.is_success() {
        return Err(format!("daemon returned {status} during batch upload",));
    }

    let body = response
        .text()
        .map_err(|error| format!("daemon returned unreadable batch response: {error}"))?;
    let batch_result: BatchUploadResponse = serde_json::from_str(&body)
        .map_err(|error| format!("daemon returned invalid batch response: {error}"))?;

    print_batch_results(&batch_result, items, cli, started_at)
}

fn print_batch_results(
    result: &BatchUploadResponse,
    items: &[UploadItem],
    cli: &Cli,
    started_at: Instant,
) -> Result<CliExitCode, String> {
    if cli.output.quiet {
        return Ok(if result.failed_count == 0 {
            CliExitCode::Success
        } else if result.created_count > 0 || result.deduplicated_count > 0 {
            CliExitCode::PartialSuccess
        } else {
            CliExitCode::Generic
        });
    }

    let duration = started_at.elapsed();
    if cli.output.json {
        for (idx, item_result) in result.results.iter().enumerate() {
            if let Some(file) = &item_result.file {
                let record = FileRecordResponse {
                    schema_version: 1,
                    id: file.id.clone(),
                    name: file.name.clone(),
                    size_bytes: file.size_bytes,
                    content_hash: file.content_hash.clone(),
                    mime_type: file.mime_type.clone(),
                    uploaded_at: file.uploaded_at,
                    tags: file.tags.clone(),
                    pinned: file.pinned,
                };
                let output = UploadOutput::new(
                    &record,
                    item_result.outcome == "deduplicated",
                    duration.as_millis() / items.len() as u128,
                    0,
                );
                let encoded = serde_json::to_string(&output)
                    .map_err(|e| format!("could not encode JSON: {e}"))?;
                println!("{encoded}");
            } else {
                eprintln!(
                    "error: batch item {} failed: {}",
                    idx,
                    item_result
                        .error
                        .as_ref()
                        .map_or(&"unknown error".to_string(), |e| &e.message)
                );
            }
        }
    } else {
        println!(
            "batch upload completed: {} created, {} deduplicated, {} failed",
            result.created_count, result.deduplicated_count, result.failed_count
        );
        for (idx, item_result) in result.results.iter().enumerate() {
            if item_result.outcome == "failed" {
                eprintln!(
                    "  [{}] {}: {}",
                    idx,
                    items
                        .get(idx)
                        .map_or(&"?".to_string(), |i| &i.filename),
                    item_result
                        .error
                        .as_ref()
                        .map_or(&"unknown error".to_string(), |e| &e.message)
                );
            } else {
                println!(
                    "  [{}] {} ({})",
                    idx,
                    items
                        .get(idx)
                        .map_or(&"?".to_string(), |i| &i.filename),
                    item_result.outcome
                );
            }
        }
    }

    Ok(if result.failed_count == 0 {
        CliExitCode::Success
    } else if result.created_count > 0 || result.deduplicated_count > 0 {
        CliExitCode::PartialSuccess
    } else {
        CliExitCode::Generic
    })
}

#[derive(Debug, Deserialize)]
struct BatchUploadResponse {
    #[allow(dead_code)]
    schema_version: u8,
    created_count: u64,
    deduplicated_count: u64,
    failed_count: u64,
    results: Vec<BatchUploadResult>,
}

#[derive(Debug, Deserialize)]
struct BatchUploadResult {
    #[allow(dead_code)]
    name: String,
    outcome: String,
    #[allow(dead_code)]
    http_status: u16,
    file: Option<FileRecordResponse>,
    error: Option<ErrorMessage>,
}

#[derive(Debug, Deserialize)]
struct ErrorMessage {
    #[allow(dead_code)]
    code: String,
    message: String,
}

fn map_client_status(status: reqwest::StatusCode) -> CliExitCode {
    match status.as_u16() {
        400 | 413 => CliExitCode::Usage,
        404 => CliExitCode::NotFound,
        409 => CliExitCode::Conflict,
        _ => CliExitCode::Generic,
    }
}

fn throughput_bytes_per_second(bytes: u64, duration: Duration) -> u64 {
    let nanos = duration.as_nanos();
    if nanos == 0 {
        return bytes;
    }

    let bytes_per_second = u128::from(bytes)
        .saturating_mul(1_000_000_000)
        .saturating_add(nanos / 2)
        / nanos;
    u64::try_from(bytes_per_second).unwrap_or(u64::MAX)
}

struct MultipartUploadBody {
    segments: VecDeque<MultipartSegment>,
}

impl MultipartUploadBody {
    fn new(boundary: &str, item: &UploadItem, file: File) -> Self {
        let mut segments = VecDeque::new();
        for tag in &item.tags {
            segments.push_back(MultipartSegment::bytes(text_field(boundary, "tag", tag)));
        }
        if item.pinned {
            segments.push_back(MultipartSegment::bytes(text_field(boundary, "pin", "true")));
        }
        segments.push_back(MultipartSegment::bytes(file_header(
            boundary,
            &item.filename,
        )));
        segments.push_back(MultipartSegment::File(file));
        segments.push_back(MultipartSegment::bytes(file_footer(boundary)));
        Self { segments }
    }
}

impl Read for MultipartUploadBody {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        while let Some(segment) = self.segments.front_mut() {
            let read = segment.read(buffer)?;
            if read > 0 {
                return Ok(read);
            }
            let _finished = self.segments.pop_front();
        }
        Ok(0)
    }
}

enum MultipartSegment {
    Bytes(Cursor<Vec<u8>>),
    File(File),
}

impl MultipartSegment {
    fn bytes(bytes: Vec<u8>) -> Self {
        Self::Bytes(Cursor::new(bytes))
    }
}

impl Read for MultipartSegment {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Bytes(cursor) => cursor.read(buffer),
            Self::File(file) => file.read(buffer),
        }
    }
}

fn text_field(boundary: &str, name: &str, value: &str) -> Vec<u8> {
    format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"{}\"\r\n\r\n{}\r\n",
        header_value(name),
        value
    )
    .into_bytes()
}

fn file_header(boundary: &str, filename: &str) -> Vec<u8> {
    format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\nContent-Type: application/octet-stream\r\n\r\n",
        header_value(filename)
    )
    .into_bytes()
}

fn file_footer(boundary: &str) -> Vec<u8> {
    format!("\r\n--{boundary}--\r\n").into_bytes()
}

fn header_value(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            '"' => "\\\"".to_owned(),
            '\\' => "\\\\".to_owned(),
            '\r' | '\n' => "_".to_owned(),
            control if control.is_control() => "_".to_owned(),
            other => other.to_string(),
        })
        .collect()
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use std::io::Read;

    use reqwest::StatusCode;
    use tempfile::tempdir;
    use tssp::{Cli, Command, ConnectionArgs, LoggingArgs, OutputArgs, UploadArgs};
    use tssp_cli_core::CliExitCode;

    use super::{
        classify_upload_status, header_value, map_client_status, map_io_error,
        parse_deduplicated_header, parse_upload_response_body, print_upload_result,
        throughput_bytes_per_second, upload_filename, FileRecordResponse, MultipartUploadBody,
        UploadInputError, UploadItem, UploadOutput, UploadPlan,
    };

    #[test]
    fn upload_plan_rejects_directory_without_recursive() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let args = upload_args(vec![temp.path().to_path_buf()]);

        let plan = UploadPlan::from_args(&args);

        assert!(matches!(plan, Err(error) if error.message.contains("use -r")));
    }

    #[test]
    fn upload_plan_rejects_empty_input() {
        let args = upload_args(Vec::new());

        let plan = UploadPlan::from_args(&args);

        assert!(matches!(plan, Err(error) if error.message.contains("no files")));
    }

    #[test]
    fn upload_plan_rejects_all_flag_when_no_files_in_directory() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let current = std::env::current_dir().unwrap_or_else(|error| panic!("cwd failed: {error}"));
        std::env::set_current_dir(temp.path())
            .unwrap_or_else(|error| panic!("chdir failed: {error}"));
        let mut args = upload_args(Vec::new());
        args.all = true;
        args.files = Vec::new(); // Ensure no files are specified

        let plan = UploadPlan::from_args(&args);
        std::env::set_current_dir(&current).ok();

        assert!(
            matches!(plan, Err(error) if error.message.contains("no files") || error.message.contains("no non-hidden"))
        );
    }

    #[test]
    fn upload_plan_accepts_all_flag_with_files_in_directory() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let file1 = temp.path().join("file1.txt");
        std::fs::write(&file1, b"data").unwrap_or_else(|error| panic!("write failed: {error}"));
        let current = std::env::current_dir().unwrap_or_else(|error| panic!("cwd failed: {error}"));
        std::env::set_current_dir(temp.path())
            .unwrap_or_else(|error| panic!("chdir failed: {error}"));
        let mut args = upload_args(Vec::new());
        args.all = true;

        let plan = UploadPlan::from_args(&args);
        std::env::set_current_dir(&current).ok();

        assert!(plan.is_ok());
        assert_eq!(plan.expect("plan should be ok").items.len(), 1);
    }

    #[test]
    fn upload_plan_rejects_recursive_flag_when_directory_is_empty() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let mut args = upload_args(Vec::new());
        args.recursive = Some(temp.path().to_path_buf());

        let plan = UploadPlan::from_args(&args);

        assert!(matches!(plan, Err(error) if error.message.contains("no files")));
    }

    #[test]
    fn upload_plan_accepts_recursive_flag_with_files() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let file1 = temp.path().join("file1.txt");
        std::fs::write(&file1, b"data").unwrap_or_else(|error| panic!("write failed: {error}"));
        let mut args = upload_args(Vec::new());
        args.recursive = Some(temp.path().to_path_buf());

        let plan = UploadPlan::from_args(&args);

        assert!(plan.is_ok());
        assert_eq!(plan.expect("plan should be ok").items.len(), 1);
    }

    #[test]
    fn upload_plan_rejects_rename_with_multiple_files() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let one = temp.path().join("one.txt");
        let two = temp.path().join("two.txt");
        std::fs::write(&one, b"one").unwrap_or_else(|error| panic!("write failed: {error}"));
        std::fs::write(&two, b"two").unwrap_or_else(|error| panic!("write failed: {error}"));
        let mut args = upload_args(vec![one, two]);
        args.rename = Some("renamed.txt".to_owned());

        let plan = UploadPlan::from_args(&args);

        assert!(matches!(plan, Err(error) if error.message.contains("--rename")));
    }

    #[test]
    fn upload_plan_accepts_single_file_with_rename() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let path = temp.path().join("one.txt");
        std::fs::write(&path, b"one").unwrap_or_else(|error| panic!("write failed: {error}"));
        let mut args = upload_args(vec![path.clone()]);
        args.rename = Some("renamed.txt".to_owned());

        let plan = UploadPlan::from_args(&args).unwrap_or_else(|error| panic!("{}", error.message));

        assert_eq!(plan.items.len(), 1);
        assert_eq!(plan.items[0].path, path);
        assert_eq!(plan.items[0].filename, "renamed.txt");
        assert_eq!(plan.items[0].tags, vec!["Docs"]);
        assert!(plan.items[0].pinned);
    }

    #[test]
    fn multipart_body_streams_fields_and_file_content() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let path = temp.path().join("note.txt");
        std::fs::write(&path, b"hello").unwrap_or_else(|error| panic!("write failed: {error}"));
        let args = upload_args(vec![path.clone()]);
        let item =
            UploadItem::from_path(&path, &args).unwrap_or_else(|error| panic!("{}", error.message));
        let file = std::fs::File::open(&path).unwrap_or_else(|error| panic!("{error}"));
        let mut body = MultipartUploadBody::new("boundary", &item, file);
        let mut text = String::new();

        body.read_to_string(&mut text)
            .unwrap_or_else(|error| panic!("read failed: {error}"));

        assert!(text.contains("name=\"tag\""));
        assert!(text.contains("Docs"));
        assert!(text.contains("filename=\"note.txt\""));
        assert!(text.contains("hello"));
        assert!(text.ends_with("--boundary--\r\n"));
    }

    #[test]
    fn header_value_removes_control_characters() {
        assert_eq!(header_value("a\"\r\nb"), "a\\\"__b");
    }

    #[test]
    fn upload_filename_uses_path_basename_without_rename() {
        let args = upload_args(Vec::new());

        let filename = upload_filename(std::path::Path::new("/tmp/report.txt"), &args)
            .unwrap_or_else(|error| panic!("filename failed: {error}"));

        assert_eq!(filename, "report.txt");
    }

    #[test]
    fn upload_filename_uses_explicit_rename() {
        let mut args = upload_args(Vec::new());
        args.rename = Some("renamed.txt".to_owned());

        let filename = upload_filename(std::path::Path::new("/tmp/report.txt"), &args)
            .unwrap_or_else(|error| panic!("filename failed: {error}"));

        assert_eq!(filename, "renamed.txt");
    }

    #[test]
    fn upload_input_error_maps_not_found() {
        let error = UploadInputError::from_io_error(
            "missing".to_owned(),
            &std::io::Error::from(std::io::ErrorKind::NotFound),
        );

        assert_eq!(error.code, CliExitCode::NotFound);
        assert_eq!(error.message, "missing");
    }

    #[test]
    fn map_io_error_maps_permission_denied() {
        let code = map_io_error(&std::io::Error::from(std::io::ErrorKind::PermissionDenied));

        assert_eq!(code, CliExitCode::PermissionDenied);
    }

    #[test]
    fn map_client_status_uses_cli_error_contract() {
        assert_eq!(
            map_client_status(StatusCode::BAD_REQUEST),
            CliExitCode::Usage
        );
        assert_eq!(
            map_client_status(StatusCode::PAYLOAD_TOO_LARGE),
            CliExitCode::Usage
        );
        assert_eq!(
            map_client_status(StatusCode::NOT_FOUND),
            CliExitCode::NotFound
        );
        assert_eq!(
            map_client_status(StatusCode::CONFLICT),
            CliExitCode::Conflict
        );
        assert_eq!(
            map_client_status(StatusCode::UNAUTHORIZED),
            CliExitCode::Generic
        );
    }

    #[test]
    fn classify_upload_status_maps_success_and_failures() {
        assert_eq!(classify_upload_status(StatusCode::CREATED), Ok(()));
        assert_eq!(
            classify_upload_status(StatusCode::INTERNAL_SERVER_ERROR),
            Err(CliExitCode::Server)
        );
        assert_eq!(
            classify_upload_status(StatusCode::CONFLICT),
            Err(CliExitCode::Conflict)
        );
    }

    #[test]
    fn deduplicated_header_is_case_insensitive() {
        assert!(parse_deduplicated_header(Some("TrUe")));
        assert!(!parse_deduplicated_header(Some("false")));
        assert!(!parse_deduplicated_header(None));
    }

    #[test]
    fn parse_upload_response_body_accepts_valid_record() {
        let record = parse_upload_response_body(FILE_RECORD_JSON)
            .unwrap_or_else(|code| panic!("parse failed with {code:?}"));

        assert_eq!(record.id, "file-1");
        assert_eq!(record.name, "note.txt");
    }

    #[test]
    fn parse_upload_response_body_rejects_invalid_record() {
        let result = parse_upload_response_body(r#"{"schema_version":1}"#);

        assert!(matches!(result, Err(CliExitCode::Server)));
    }

    #[test]
    fn upload_output_uses_stable_schema() {
        let record = file_record();

        let output = UploadOutput::new(&record, true, 25, 100);

        assert_eq!(output.schema_version, 1);
        assert_eq!(output.id, "file-1");
        assert!(output.deduplicated);
        assert_eq!(output.duration_ms, 25);
        assert_eq!(output.throughput_bytes_per_second, 100);
    }

    #[test]
    fn print_upload_result_supports_quiet_json_and_human_modes() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let path = temp.path().join("note.txt");
        std::fs::write(&path, b"hello").unwrap_or_else(|error| panic!("write failed: {error}"));
        let args = upload_args(vec![path.clone()]);
        let item =
            UploadItem::from_path(&path, &args).unwrap_or_else(|error| panic!("{}", error.message));
        let record = file_record();

        assert_eq!(
            print_upload_result(
                &record,
                &item,
                false,
                None,
                std::time::Instant::now(),
                &cli(false, true)
            ),
            Ok(())
        );
        assert_eq!(
            print_upload_result(
                &record,
                &item,
                true,
                None,
                std::time::Instant::now(),
                &cli(true, false)
            ),
            Ok(())
        );
        assert_eq!(
            print_upload_result(
                &record,
                &item,
                false,
                Some("/api/v1/files/file-1".to_owned()),
                std::time::Instant::now(),
                &cli(false, false),
            ),
            Ok(())
        );
    }

    #[test]
    fn throughput_handles_zero_duration() {
        assert_eq!(
            throughput_bytes_per_second(12, std::time::Duration::ZERO),
            12
        );
    }

    #[test]
    fn throughput_rounds_to_nearest_byte_per_second() {
        assert_eq!(
            throughput_bytes_per_second(10, std::time::Duration::from_secs(2)),
            5
        );
    }

    #[test]
    fn run_rejects_invalid_connection_string() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let file_path = temp.path().join("test.txt");
        std::fs::write(&file_path, "test").unwrap_or_else(|error| panic!("write failed: {error}"));

        let mut cli_args = cli(false, false);
        cli_args.connection.host = Some("bad/host".to_owned());
        cli_args.upload.files.push(file_path);

        assert_eq!(super::run(&cli_args), Ok(CliExitCode::Usage));
    }

    #[test]
    fn run_returns_usage_on_plan_error() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let current = std::env::current_dir().unwrap_or_else(|error| panic!("cwd failed: {error}"));
        std::env::set_current_dir(temp.path())
            .unwrap_or_else(|error| panic!("chdir failed: {error}"));
        let mut cli_args = cli(false, false);
        cli_args.upload.all = true;

        let result = super::run(&cli_args);
        std::env::set_current_dir(&current).ok();

        assert_eq!(result, Ok(CliExitCode::Usage));
    }

    fn file_record() -> FileRecordResponse {
        serde_json::from_str(FILE_RECORD_JSON)
            .unwrap_or_else(|error| panic!("record parse failed: {error}"))
    }

    fn cli(json: bool, quiet: bool) -> Cli {
        Cli {
            output: OutputArgs {
                json,
                quiet,
                no_color: true,
            },
            logging: LoggingArgs { verbose: false },
            connection: ConnectionArgs {
                host: Some("127.0.0.1".to_owned()),
                port: Some(8421),
            },
            upload: upload_args(Vec::new()),
            command: Some(Command::Status),
        }
    }

    fn upload_args(files: Vec<std::path::PathBuf>) -> UploadArgs {
        UploadArgs {
            tags: vec!["Docs".to_owned()],
            pin: true,
            rename: None,
            parallel: None,
            recursive: None,
            all: false,
            files,
        }
    }

    const FILE_RECORD_JSON: &str = r#"{"schema_version":1,"id":"file-1","name":"note.txt","size_bytes":5,"content_hash":"hash","mime_type":"text/plain","uploaded_at":1700000000,"tags":["Docs"],"pinned":false}"#;
}
