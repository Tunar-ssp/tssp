//! `tssp login` — obtain a remote API bearer token.

use std::io::{self, Write};

use reqwest::header::ACCEPT;
use serde::Deserialize;
use tssp::Cli;
use tssp_cli_core::CliExitCode;

use crate::backend::{api_post, build_client, BackendAddress};
use crate::config::{load_config, save_config};

#[derive(Debug, Deserialize)]
struct TokenResponse {
    token: String,
}

/// Runs `tssp login`.
pub(crate) fn run(cli: &Cli) -> Result<CliExitCode, String> {
    let address = BackendAddress::from_connection_args(&cli.connection)?;
    let password = read_password("Password: ")?;
    let client = build_client()?;
    let response = api_post(&client, &address.url("/api/v1/auth/token"))
        .header(ACCEPT, "application/vnd.tssp.v1+json")
        .json(&serde_json::json!({ "password": password }))
        .send()
    .map_err(|error| format!("could not reach daemon at {}: {error}", address.base_url()))?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        eprintln!("error: invalid password");
        return Ok(CliExitCode::Generic);
    }
    if !response.status().is_success() {
        return Err(format!(
            "daemon returned {} while exchanging credentials",
            response.status()
        ));
    }

    let body: TokenResponse = response
        .json()
        .map_err(|error| format!("could not parse token response: {error}"))?;

    let mut config = load_config()?;
    config.host = Some(address.host().to_owned());
    config.port = Some(address.port());
    config.token = Some(body.token);
    save_config(&config)?;

    if !cli.output.quiet {
        println!("Logged in to {}", address.base_url());
        println!("Token saved to {}", crate::config::resolve_config_path()?.display());
    }
    Ok(CliExitCode::Success)
}

fn read_password(prompt: &str) -> Result<String, String> {
    if let Ok(password) = std::env::var("TSSP_LOGIN_PASSWORD") {
        let password = password.trim().to_owned();
        if !password.is_empty() {
            return Ok(password);
        }
    }
    eprint!("{prompt}");
    io::stdout()
        .flush()
        .map_err(|error| format!("stdout flush failed: {error}"))?;
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|error| format!("could not read password: {error}"))?;
    let password = line.trim().to_owned();
    if password.is_empty() {
        return Err("password must not be empty".to_owned());
    }
    Ok(password)
}

#[cfg(test)]
mod tests {
    use super::TokenResponse;

    #[test]
    fn token_response_deserializes() {
        let body: TokenResponse =
            serde_json::from_str(r#"{"token":"abc123"}"#).expect("deserialize");
        assert_eq!(body.token, "abc123");
    }
}
