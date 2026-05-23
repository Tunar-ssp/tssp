//! `tssp receive` command implementation.

use std::time::Duration;

use tssp_cli_core::CliExitCode;
use tssp::{Cli, ReceiveArgs};

/// Runs the receive command.
pub fn run(_cli: &Cli, args: &ReceiveArgs) -> Result<CliExitCode, String> {

    let timeout = parse_timeout(&args.timeout)?;

    eprintln!(
        "waiting for receive session (timeout: {}s)",
        timeout.as_secs()
    );
    eprintln!("receive session listening; implementation pending");

    Ok(CliExitCode::Success)
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
