//! `tssp receive` command implementation.

use std::time::Duration;

use serde_json::json;
use tssp::{Cli, ReceiveArgs};
use tssp_cli_core::CliExitCode;

use crate::backend::{build_client, BackendAddress};
use crate::sessions_helper::generate_qr_code;

pub fn run(cli: &Cli, args: &ReceiveArgs) -> Result<CliExitCode, String> {
    let timeout = parse_timeout(&args.timeout)?;

    eprintln!(
        "Creating receive session with timeout: {}s...",
        timeout.as_secs()
    );

    let address = BackendAddress::from_connection_args(&cli.connection)
        .map_err(|e| format!("invalid backend address: {e}"))?;
    let client = build_client()?;

    let ttl = timeout.as_secs() as u64;
    let req = json!({
        "ttl_seconds": ttl
    });

    let response = client
        .post(address.url("/api/v1/sessions/receive"))
        .json(&req)
        .send()
        .map_err(|e| format!("failed to create receive session: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "failed to create receive session: {}",
            response.status()
        ));
    }

    let body = response
        .json::<serde_json::Value>()
        .map_err(|e| format!("failed to parse session response: {e}"))?;

    if let Some(token) = body.get("token").and_then(|v| v.as_str()) {
        let receive_url = format!("{}/u/{}", address.base_url(), token);

        if cli.output.json {
            let output = serde_json::json!({
                "success": true,
                "token": token,
                "receive_url": receive_url,
                "timeout_seconds": ttl,
            });
            let json_str = serde_json::to_string_pretty(&output)
                .map_err(|e| format!("failed to serialize JSON: {e}"))?;
            println!("{}", json_str);
        } else {
            let qr = generate_qr_code(&receive_url, 200)?;
            println!("{}", qr);

            println!("\nReceive URL:");
            println!("{}\n", receive_url);
            println!("Session token: {}", token);
            println!("Expires in: {} seconds", ttl);

            if !cli.output.quiet {
                eprintln!("Receive session created successfully!");
            }
        }

        Ok(CliExitCode::Success)
    } else {
        Err("failed to extract token from session response".to_string())
    }
}

fn parse_timeout(timeout_str: &Option<String>) -> Result<Duration, String> {
    match timeout_str {
        None => Ok(Duration::from_secs(300)),
        Some(s) => {
            if let Ok(secs) = s.parse::<u64>() {
                Ok(Duration::from_secs(secs))
            } else {
                Err(format!("invalid timeout duration: {}", s))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_timeout;
    use std::time::Duration;

    #[test]
    fn parse_timeout_defaults_to_300_seconds() {
        let result = parse_timeout(&None);
        assert_eq!(result, Ok(Duration::from_secs(300)));
    }

    #[test]
    fn parse_timeout_parses_valid_duration() {
        let result = parse_timeout(&Some("600".to_string()));
        assert_eq!(result, Ok(Duration::from_secs(600)));
    }

    #[test]
    fn parse_timeout_rejects_invalid_duration() {
        let result = parse_timeout(&Some("invalid".to_string()));
        assert!(matches!(result, Err(e) if e.contains("invalid timeout")));
    }

    #[test]
    fn parse_timeout_rejects_negative_duration() {
        let result = parse_timeout(&Some("-100".to_string()));
        assert!(matches!(result, Err(e) if e.contains("invalid timeout")));
    }
}
