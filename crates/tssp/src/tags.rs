//! Implementation of `tssp tag` and `tssp untag`.

use reqwest::{header::ACCEPT, StatusCode};
use serde::{Deserialize, Serialize};
use tssp::{Cli, TagArgs};
use tssp_cli_core::CliExitCode;

use crate::backend::{api_delete, api_post, build_client, BackendAddress};

/// Runs `tssp tag <id> <tag>...`.
pub(crate) fn run_tag(cli: &Cli, args: &TagArgs) -> Result<CliExitCode, String> {
    if args.tags.is_empty() {
        eprintln!("error: tag requires at least one tag name");
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
    let response = api_post(&client, &tags_url(&address, &args.id))
        .header(ACCEPT, "application/vnd.tssp.v1+json")
        .json(&args.tags)
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
    let mutation = match handle_mutation_response(response, &address, &args.id)? {
        Ok(value) => value,
        Err(code) => return Ok(code),
    };
    print_tag_result("tagged", &args.id, mutation.changed_count, cli)?;
    Ok(CliExitCode::Success)
}

/// Runs `tssp untag <id> <tag>...`.
pub(crate) fn run_untag(cli: &Cli, args: &TagArgs) -> Result<CliExitCode, String> {
    if args.tags.is_empty() {
        eprintln!("error: untag requires at least one tag name");
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
    let mut changed_count = 0_u64;
    for tag in &args.tags {
        let response = api_delete(&client, &tag_url(&address, &args.id, tag))
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
        let mutation = match handle_mutation_response(response, &address, &args.id)? {
            Ok(value) => value,
            Err(code) => return Ok(code),
        };
        changed_count = changed_count
            .checked_add(mutation.changed_count)
            .ok_or_else(|| "tag mutation count overflow".to_owned())?;
    }
    print_tag_result("untagged", &args.id, changed_count, cli)?;
    Ok(CliExitCode::Success)
}

fn handle_mutation_response(
    response: reqwest::blocking::Response,
    address: &BackendAddress,
    id: &str,
) -> Result<Result<TagMutationResponse, CliExitCode>, String> {
    if let Err(code) = classify_response_status(response.status()) {
        print_status_error(response.status(), code, id);
        return Ok(Err(code));
    }
    let body = response.text().map_err(|error| {
        format!(
            "daemon at {} returned an unreadable tag response: {error}",
            address.base_url()
        )
    })?;
    parse_tag_body(&body, &address.base_url()).map(Ok)
}

fn tags_url(address: &BackendAddress, id: &str) -> String {
    format!("{}/tags", crate::info::info_url(address, id))
}

fn tag_url(address: &BackendAddress, id: &str, tag: &str) -> String {
    format!(
        "{}/{}",
        tags_url(address, id),
        crate::info::path_segment(tag)
    )
}

fn classify_response_status(status: StatusCode) -> Result<(), CliExitCode> {
    if status.is_server_error() {
        return Err(CliExitCode::Server);
    }
    match status.as_u16() {
        200 => Ok(()),
        400 => Err(CliExitCode::Usage),
        404 => Err(CliExitCode::NotFound),
        409 => Err(CliExitCode::Conflict),
        _ => Err(CliExitCode::Generic),
    }
}

fn print_status_error(status: StatusCode, code: CliExitCode, id: &str) {
    match code {
        CliExitCode::NotFound => eprintln!("error: file {id} was not found"),
        CliExitCode::Usage => eprintln!("error: tag request is invalid"),
        _ => eprintln!("error: daemon returned {status}"),
    }
}

fn parse_tag_body(body: &str, base_url: &str) -> Result<TagMutationResponse, String> {
    let parsed = serde_json::from_str::<TagMutationResponse>(body).map_err(|error| {
        format!("daemon at {base_url} returned an invalid tag response: {error}")
    })?;
    if parsed.schema_version != 1 {
        return Err(format!(
            "daemon at {base_url} returned unsupported tag schema version {}",
            parsed.schema_version
        ));
    }
    Ok(parsed)
}

fn print_tag_result(action: &str, id: &str, changed_count: u64, cli: &Cli) -> Result<(), String> {
    if cli.output.quiet {
        return Ok(());
    }
    if cli.output.json {
        let output = TagOutput {
            schema_version: 1,
            id,
            changed_count,
        };
        let encoded = serde_json::to_string(&output)
            .map_err(|error| format!("could not encode tag JSON: {error}"))?;
        println!("{encoded}");
        return Ok(());
    }
    println!("{action} {id}: {changed_count} changed");
    Ok(())
}

#[derive(Debug, Deserialize)]
struct TagMutationResponse {
    schema_version: u8,
    changed_count: u64,
}

#[derive(Debug, Serialize)]
struct TagOutput<'a> {
    schema_version: u8,
    id: &'a str,
    changed_count: u64,
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;
    use tssp::{Cli, ConnectionArgs, LoggingArgs, OutputArgs, UploadArgs};
    use tssp_cli_core::CliExitCode;

    use crate::backend::BackendAddress;

    use super::{classify_response_status, parse_tag_body, print_tag_result, tag_url, tags_url};

    #[test]
    fn tag_urls_percent_encode_path_segments() {
        let address = BackendAddress::from_connection_args(&ConnectionArgs {
            host: Some("127.0.0.1".to_owned()),
            port: Some(8421),
        })
        .unwrap_or_else(|error| panic!("address failed: {error}"));

        assert_eq!(
            tags_url(&address, "file 1"),
            "http://127.0.0.1:8421/api/v1/files/file%201/tags"
        );
        assert_eq!(
            tag_url(&address, "file-1", "Family Photos"),
            "http://127.0.0.1:8421/api/v1/files/file-1/tags/Family%20Photos"
        );
    }

    #[test]
    fn response_status_maps_tag_contract() {
        assert_eq!(classify_response_status(StatusCode::OK), Ok(()));
        assert_eq!(
            classify_response_status(StatusCode::BAD_REQUEST),
            Err(CliExitCode::Usage)
        );
        assert_eq!(
            classify_response_status(StatusCode::NOT_FOUND),
            Err(CliExitCode::NotFound)
        );
        assert_eq!(
            classify_response_status(StatusCode::CONFLICT),
            Err(CliExitCode::Conflict)
        );
        assert_eq!(
            classify_response_status(StatusCode::INTERNAL_SERVER_ERROR),
            Err(CliExitCode::Server)
        );
    }

    #[test]
    fn parse_tag_body_accepts_mutation_payload() {
        let parsed = parse_tag_body(
            r#"{"schema_version":1,"changed_count":2}"#,
            "http://127.0.0.1:8421",
        )
        .unwrap_or_else(|error| panic!("parse failed: {error}"));

        assert_eq!(parsed.schema_version, 1);
        assert_eq!(parsed.changed_count, 2);
    }

    #[test]
    fn parse_tag_body_rejects_invalid_payload() {
        let parsed = parse_tag_body(r#"{"schema_version":1}"#, "http://127.0.0.1:8421");

        assert!(matches!(parsed, Err(message) if message.contains("invalid tag response")));
    }

    #[test]
    fn print_tag_result_supports_quiet_json_and_human_output() {
        assert_eq!(
            print_tag_result("tagged", "file-1", 1, &cli(true, false)),
            Ok(())
        );
        assert_eq!(
            print_tag_result("tagged", "file-1", 1, &cli(false, true)),
            Ok(())
        );
        assert_eq!(
            print_tag_result("tagged", "file-1", 1, &cli(false, false)),
            Ok(())
        );
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
        }
    }
}
