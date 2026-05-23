//! Implementation of `tssp pin`, `tssp unpin`, and `tssp pins`.

use reqwest::{header::ACCEPT, StatusCode};
use tssp::{IdArgs, PinArgs, PinsAction, PinsCommand};
use tssp_cli_core::CliExitCode;

use crate::backend::{build_client, BackendAddress};
use tssp::Cli;

/// Runs `tssp pin <id> [--position <n>]`.
pub(crate) fn run_pin(cli: &Cli, args: &PinArgs) -> Result<CliExitCode, String> {
    let address = match BackendAddress::from_connection_args(&cli.connection) {
        Ok(value) => value,
        Err(message) => {
            eprintln!("error: {message}");
            return Ok(CliExitCode::Usage);
        }
    };
    let client = build_client()?;
    let url = format!("{}/api/v1/files/{}/pin", address.base_url(), args.id);

    let mut request = client.put(url).header(ACCEPT, "application/vnd.tssp.v1+json");
    if let Some(pos) = args.position {
        request = request.json(&serde_json::json!({ "position": pos }));
    }

    let response = request.send().map_err(|error| {
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

    if !cli.output.quiet {
        println!("pinned {}", args.id);
    }
    Ok(CliExitCode::Success)
}

/// Runs `tssp unpin <id>`.
pub(crate) fn run_unpin(cli: &Cli, args: &IdArgs) -> Result<CliExitCode, String> {
    let address = match BackendAddress::from_connection_args(&cli.connection) {
        Ok(value) => value,
        Err(message) => {
            eprintln!("error: {message}");
            return Ok(CliExitCode::Usage);
        }
    };
    let client = build_client()?;
    let url = format!("{}/api/v1/files/{}/pin", address.base_url(), args.id);

    let response = client
        .delete(url)
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

    if !cli.output.quiet {
        println!("unpinned {}", args.id);
    }
    Ok(CliExitCode::Success)
}

/// Runs `tssp pins`.
pub(crate) fn run_pins(cli: &Cli, command: &PinsCommand) -> Result<CliExitCode, String> {
    match &command.action {
        PinsAction::List => run_pins_list(cli),
        PinsAction::Reorder(args) => run_pins_reorder(cli, args),
    }
}

fn run_pins_list(cli: &Cli) -> Result<CliExitCode, String> {
    let address = match BackendAddress::from_connection_args(&cli.connection) {
        Ok(value) => value,
        Err(message) => {
            eprintln!("error: {message}");
            return Ok(CliExitCode::Usage);
        }
    };
    let client = build_client()?;
    let url = format!("{}/api/v1/pins", address.base_url());

    let response = client
        .get(url)
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

    if response.status().is_server_error() {
        eprintln!("error: daemon returned {}", response.status());
        return Ok(CliExitCode::Server);
    }
    if !response.status().is_success() {
        eprintln!("error: daemon returned {}", response.status());
        return Ok(CliExitCode::Generic);
    }

    let text = response
        .text()
        .map_err(|error| format!("failed to read response: {error}"))?;
    
    if cli.output.json {
        println!("{text}");
        return Ok(CliExitCode::Success);
    }

    let parsed: serde_json::Value =
        serde_json::from_str(&text).map_err(|error| format!("invalid json: {error}"))?;
    
    if let Some(files) = parsed["files"].as_array() {
        for file in files {
            if let (Some(id), Some(name)) = (file["id"].as_str(), file["name"].as_str()) {
                println!("{id}\t{name}");
            }
        }
    }

    Ok(CliExitCode::Success)
}

fn run_pins_reorder(cli: &Cli, args: &tssp::ReorderArgs) -> Result<CliExitCode, String> {
    let address = match BackendAddress::from_connection_args(&cli.connection) {
        Ok(value) => value,
        Err(message) => {
            eprintln!("error: {message}");
            return Ok(CliExitCode::Usage);
        }
    };
    let client = build_client()?;
    let url = format!("{}/api/v1/pins/reorder", address.base_url());

    let payload = serde_json::json!({
        "ids": args.ids
    });

    let response = client
        .post(url)
        .header(ACCEPT, "application/vnd.tssp.v1+json")
        .json(&payload)
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

    if !cli.output.quiet {
        println!("pins reordered successfully");
    }
    Ok(CliExitCode::Success)
}

fn classify_response_status(status: StatusCode) -> Result<(), CliExitCode> {
    if status.is_server_error() {
        return Err(CliExitCode::Server);
    }
    match status.as_u16() {
        200 | 204 => Ok(()),
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

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;
    use tssp_cli_core::CliExitCode;
    use super::classify_response_status;

    #[test]
    fn response_status_maps_pin_contract() {
        assert_eq!(classify_response_status(StatusCode::OK), Ok(()));
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
            classify_response_status(StatusCode::INTERNAL_SERVER_ERROR),
            Err(CliExitCode::Server)
        );
    }
}
