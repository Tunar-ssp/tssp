//! `tssp logout` — clear saved credentials.

use tssp::Cli;
use tssp_cli_core::CliExitCode;

use crate::config::{load_config, save_config};

/// Runs `tssp logout`.
pub(crate) fn run(cli: &Cli) -> Result<CliExitCode, String> {
    let mut config = load_config()?;
    let had_token = config.token.take().is_some();
    save_config(&config)?;
    if !cli.output.quiet {
        if had_token {
            println!("Logged out. Token removed from config.");
        } else {
            println!("No saved token.");
        }
    }
    Ok(CliExitCode::Success)
}
