//! `tssp whoami` — show current session identity.

use reqwest::header::ACCEPT;
use serde::Deserialize;
use tssp::Cli;
use tssp_cli_core::CliExitCode;

use crate::backend::{api_get, build_client, BackendAddress};
use crate::config::load_config;

#[derive(Debug, Deserialize)]
struct AuthMeResponse {
    authenticated: bool,
    name: Option<String>,
    role: Option<String>,
}

/// Runs `tssp whoami`.
pub(crate) fn run(cli: &Cli) -> Result<CliExitCode, String> {
    let address = BackendAddress::from_connection_args(&cli.connection)?;
    let config = load_config()?;
    let Some(token) = config.token.as_deref() else {
        eprintln!("Not logged in (no token in config). Run `tssp login` first.");
        return Ok(CliExitCode::Generic);
    };
    let client = build_client()?;
    let response = api_get(&client, &address.url("/api/v1/auth/me"))
        .header(ACCEPT, "application/vnd.tssp.v1+json")
        .bearer_auth(token)
        .send()
        .map_err(|error| format!("could not reach daemon at {}: {error}", address.base_url()))?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        eprintln!("Session expired or invalid. Run `tssp login` again.");
        return Ok(CliExitCode::Generic);
    }
    if !response.status().is_success() {
        return Err(format!("daemon returned {}", response.status()));
    }

    let body: AuthMeResponse = response
        .json()
        .map_err(|error| format!("could not parse response: {error}"))?;

    if !cli.output.quiet {
        if body.authenticated {
            if let (Some(name), Some(role)) = (body.name, body.role) {
                println!("{name} ({role}) @ {}", address.base_url());
            } else {
                println!("Authenticated (open local mode) @ {}", address.base_url());
            }
        } else {
            println!("Not authenticated @ {}", address.base_url());
        }
    }
    Ok(CliExitCode::Success)
}
