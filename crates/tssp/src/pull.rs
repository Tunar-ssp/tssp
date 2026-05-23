//! Implementation of `tssp pull`.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use reqwest::header::CONTENT_DISPOSITION;
use reqwest::StatusCode;
use tssp::PullArgs;
use tssp_cli_core::CliExitCode;

use crate::backend::{build_client, BackendAddress};
use tssp::Cli;

/// Runs `tssp pull <id>`.
pub(crate) fn run(cli: &Cli, args: &PullArgs) -> Result<CliExitCode, String> {
    if args.all {
        eprintln!("error: pull --all is not wired yet");
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
    let response = client
        .get(content_url(&address, &args.id_or_name))
        .send()
        .map_err(|error| {
            eprintln!(
                "error: could not download {} from {}: {error}",
                args.id_or_name,
                address.base_url()
            );
            CliExitCode::Network
        });
    let mut response = match response {
        Ok(value) => value,
        Err(code) => return Ok(code),
    };

    if let Err(code) = classify_response_status(response.status()) {
        print_status_error(response.status(), code, &args.id_or_name);
        return Ok(code);
    }

    let remote_name = response
        .headers()
        .get(CONTENT_DISPOSITION)
        .and_then(|value| value.to_str().ok())
        .and_then(filename_from_content_disposition)
        .unwrap_or_else(|| args.id_or_name.clone());
    let destination = destination_path(args.output.as_deref(), &remote_name);
    if destination.exists() && !args.overwrite {
        eprintln!(
            "error: {} already exists; use --overwrite to replace it",
            destination.display()
        );
        return Ok(CliExitCode::Conflict);
    }

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(args.overwrite)
        .create_new(!args.overwrite)
        .open(&destination)
        .map_err(|error| {
            eprintln!("error: could not create {}: {error}", destination.display());
            map_io_error(&error)
        });
    let mut file = match file {
        Ok(value) => value,
        Err(code) => return Ok(code),
    };

    if let Err(code) = std::io::copy(&mut response, &mut file).map_err(|error| {
        eprintln!("error: could not write {}: {error}", destination.display());
        map_io_error(&error)
    }) {
        return Ok(code);
    }
    if let Err(code) = file.flush().map_err(|error| {
        eprintln!("error: could not flush {}: {error}", destination.display());
        map_io_error(&error)
    }) {
        return Ok(code);
    }
    if let Err(code) = file.sync_all().map_err(|error| {
        eprintln!("error: could not sync {}: {error}", destination.display());
        map_io_error(&error)
    }) {
        return Ok(code);
    }

    if !cli.output.quiet {
        if cli.output.json {
            println!(
                "{{\"schema_version\":1,\"path\":\"{}\"}}",
                json_escape(&destination.display().to_string())
            );
        } else {
            println!("downloaded {}", destination.display());
        }
    }
    Ok(CliExitCode::Success)
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

fn print_status_error(status: StatusCode, code: CliExitCode, id: &str) {
    match code {
        CliExitCode::NotFound => eprintln!("error: file {id} was not found"),
        CliExitCode::Usage => eprintln!("error: daemon rejected the pull request with {status}"),
        _ => eprintln!("error: daemon returned {status}"),
    }
}

fn filename_from_content_disposition(value: &str) -> Option<String> {
    value.split(';').skip(1).find_map(|part| {
        let (name, value) = part.trim().split_once('=')?;
        if !name.eq_ignore_ascii_case("filename") {
            return None;
        }
        Some(safe_download_name(unquote_header_value(value.trim())))
    })
}

fn unquote_header_value(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
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
        classify_response_status, destination_path, filename_from_content_disposition, json_escape,
        map_io_error, safe_download_name,
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
    fn content_disposition_filename_is_sanitized() {
        let filename =
            filename_from_content_disposition("attachment; filename=\"../report\\evil.txt\"");

        assert_eq!(filename.as_deref(), Some("report_evil.txt"));
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
        assert_eq!(json_escape("a\"\\\n"), "a\\\"\\\\\\n");
    }
}
