//! `tssp init` first-run configuration command.

use std::io::{self, Write};

use tssp::Cli;
use tssp_cli_core::CliExitCode;

/// Runs the init command for first-time setup.
pub fn run(_cli: &Cli) -> Result<CliExitCode, String> {
    eprintln!("tssp first-run setup wizard");
    eprintln!("");
    eprintln!("No configuration file found. Let's set up your daemon connection.");
    eprintln!("");

    let hostname = prompt_for_hostname()?;
    let port = prompt_for_port()?;

    eprintln!("");
    eprintln!("Configuration would be saved for {}:{}", hostname, port);
    eprintln!("init setup; implementation pending");
    eprintln!("");
    eprintln!("Configuration saved. You can now use tssp commands.");

    Ok(CliExitCode::Success)
}

fn prompt_for_hostname() -> Result<String, String> {
    eprint!("Daemon hostname [localhost]: ");
    let _ = io::stderr().flush();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("failed to read input: {e}"))?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok("localhost".to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

fn prompt_for_port() -> Result<u16, String> {
    eprint!("Daemon port [8421]: ");
    let _ = io::stderr().flush();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("failed to read input: {e}"))?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(8421)
    } else {
        trimmed
            .parse()
            .map_err(|_| format!("invalid port: {}", trimmed))
    }
}
