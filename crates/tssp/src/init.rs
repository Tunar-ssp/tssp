//! `tssp init` first-run configuration command.

use std::io::{self, Write};

use tssp::Cli;
use tssp_cli_core::CliExitCode;

use crate::config::{resolve_config_path, save_config, ConfigFile};

/// Runs the init command for first-time setup.
pub fn run(_cli: &Cli) -> Result<CliExitCode, String> {
    eprintln!("tssp first-run setup wizard");
    eprintln!();
    eprintln!("No configuration file found. Let's set up your daemon connection.");
    eprintln!();

    let hostname = prompt_for_hostname()?;
    let port = prompt_for_port()?;

    let config = ConfigFile {
        host: if hostname == "localhost" {
            None
        } else {
            Some(hostname.clone())
        },
        port: if port == 8421 { None } else { Some(port) },
    };

    save_config(&config)?;

    let config_path = resolve_config_path()?;
    eprintln!();
    eprintln!("Configuration saved to: {}", config_path.display());
    eprintln!("Daemon: {hostname}:{port}");
    eprintln!();
    eprintln!("You can now use tssp commands.");

    Ok(CliExitCode::Success)
}

fn prompt_for_hostname() -> Result<String, String> {
    eprint!("Daemon hostname [localhost]: ");
    io::stderr()
        .flush()
        .map_err(|e| format!("failed to flush stderr: {e}"))?;

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
    io::stderr()
        .flush()
        .map_err(|e| format!("failed to flush stderr: {e}"))?;

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
            .map_err(|_| format!("invalid port: {trimmed}"))
    }
}
