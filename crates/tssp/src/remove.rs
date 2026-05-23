//! Implementation of `tssp remove`.

use std::io::{self, IsTerminal, Write};

use reqwest::{header::ACCEPT, StatusCode};
use tssp::RemoveArgs;
use tssp_cli_core::CliExitCode;

use crate::backend::{api_delete, build_client, BackendAddress};
use tssp::Cli;

/// Runs `tssp remove <id>`.
pub(crate) fn run(cli: &Cli, args: &RemoveArgs) -> Result<CliExitCode, String> {
    match confirmation_decision(args.yes, io::stdin().is_terminal()) {
        ConfirmationDecision::Proceed => {}
        ConfirmationDecision::RequireYes => {
            eprintln!("error: remove requires --yes when stdin is not a terminal");
            return Ok(CliExitCode::Usage);
        }
        ConfirmationDecision::Prompt => {
            if !confirm_delete(&args.id)? {
                eprintln!("cancelled");
                return Ok(CliExitCode::Cancelled);
            }
        }
    }

    let address = match BackendAddress::from_connection_args(&cli.connection) {
        Ok(value) => value,
        Err(message) => {
            eprintln!("error: {message}");
            return Ok(CliExitCode::Usage);
        }
    };
    let client = build_client()?;
    let response = api_delete(&client, &remove_url(&address, &args.id))
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

    let already_gone = header_is_true(response.headers(), "x-tssp-already-gone");
    print_remove_result(&args.id, already_gone, cli.output.json, cli.output.quiet)?;
    Ok(CliExitCode::Success)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ConfirmationDecision {
    Proceed,
    Prompt,
    RequireYes,
}

fn confirmation_decision(yes: bool, stdin_is_terminal: bool) -> ConfirmationDecision {
    if yes {
        return ConfirmationDecision::Proceed;
    }
    if stdin_is_terminal {
        return ConfirmationDecision::Prompt;
    }
    ConfirmationDecision::RequireYes
}

fn confirm_delete(id: &str) -> Result<bool, String> {
    eprint!("Delete {id}? Type yes to continue: ");
    io::stderr()
        .flush()
        .map_err(|error| format!("could not flush prompt: {error}"))?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|error| format!("could not read confirmation: {error}"))?;
    Ok(answer.trim().eq_ignore_ascii_case("yes"))
}

fn remove_url(address: &BackendAddress, id: &str) -> String {
    crate::info::info_url(address, id)
}

fn classify_response_status(status: StatusCode) -> Result<(), CliExitCode> {
    if status.is_server_error() {
        return Err(CliExitCode::Server);
    }
    match status.as_u16() {
        200 | 202 | 204 => Ok(()),
        400 => Err(CliExitCode::Usage),
        404 => Err(CliExitCode::NotFound),
        409 => Err(CliExitCode::Conflict),
        _ => Err(CliExitCode::Generic),
    }
}

fn print_status_error(status: StatusCode, code: CliExitCode, id: &str) {
    match code {
        CliExitCode::NotFound => eprintln!("error: file {id} was not found"),
        CliExitCode::Usage => eprintln!("error: file id is invalid"),
        _ => eprintln!("error: daemon returned {status}"),
    }
}

fn header_is_true(headers: &reqwest::header::HeaderMap, name: &str) -> bool {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

fn print_remove_result(
    id: &str,
    already_gone: bool,
    json: bool,
    quiet: bool,
) -> Result<(), String> {
    if quiet {
        return Ok(());
    }
    if json {
        let output = RemoveOutput {
            schema_version: 1,
            id,
            already_gone,
        };
        let encoded = serde_json::to_string(&output)
            .map_err(|error| format!("could not encode remove JSON: {error}"))?;
        println!("{encoded}");
        return Ok(());
    }

    if already_gone {
        println!("file {id} was already gone");
    } else {
        println!("removed {id}");
    }
    Ok(())
}

#[derive(serde::Serialize)]
struct RemoveOutput<'a> {
    schema_version: u8,
    id: &'a str,
    already_gone: bool,
}

#[cfg(test)]
mod tests {
    use reqwest::header::{HeaderMap, HeaderValue};
    use reqwest::StatusCode;
    use tssp::ConnectionArgs;
    use tssp_cli_core::CliExitCode;

    use crate::backend::BackendAddress;

    use super::{
        classify_response_status, confirmation_decision, header_is_true, print_remove_result,
        remove_url, ConfirmationDecision,
    };

    #[test]
    fn confirmation_decision_prompts_only_for_interactive_without_yes() {
        assert_eq!(
            confirmation_decision(true, false),
            ConfirmationDecision::Proceed
        );
        assert_eq!(
            confirmation_decision(false, true),
            ConfirmationDecision::Prompt
        );
        assert_eq!(
            confirmation_decision(false, false),
            ConfirmationDecision::RequireYes
        );
    }

    #[test]
    fn remove_url_percent_encodes_file_id() {
        let address = BackendAddress::from_connection_args(&ConnectionArgs {
            host: Some("127.0.0.1".to_owned()),
            port: Some(8421),
        })
        .unwrap_or_else(|error| panic!("address failed: {error}"));

        assert_eq!(
            remove_url(&address, "bad id"),
            "http://127.0.0.1:8421/api/v1/files/bad%20id"
        );
    }

    #[test]
    fn response_status_maps_delete_contract() {
        assert_eq!(classify_response_status(StatusCode::NO_CONTENT), Ok(()));
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
            classify_response_status(StatusCode::SERVICE_UNAVAILABLE),
            Err(CliExitCode::Server)
        );
    }

    #[test]
    fn already_gone_header_is_case_insensitive() {
        let mut headers = HeaderMap::new();
        headers.insert("x-tssp-already-gone", HeaderValue::from_static("TRUE"));

        assert!(header_is_true(&headers, "x-tssp-already-gone"));
        assert!(!header_is_true(&headers, "x-tssp-missing"));
    }

    #[test]
    fn print_remove_result_supports_output_modes() {
        assert_eq!(print_remove_result("file-1", false, false, true), Ok(()));
        assert_eq!(print_remove_result("file-1", true, true, false), Ok(()));
        assert_eq!(print_remove_result("file-1", false, false, false), Ok(()));
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
}
