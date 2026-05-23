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
