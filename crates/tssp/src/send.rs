//! `tssp send` command implementation.

use std::path::PathBuf;

use tssp_cli_core::CliExitCode;
use tssp::{Cli, SendArgs};

/// Runs the send command.
pub fn run(_cli: &Cli, args: &SendArgs) -> Result<CliExitCode, String> {
    validate_file_exists(&args.file)?;

    eprintln!(
        "creating send session for {} with {} tag(s)",
        args.file.display(),
        args.tags.len()
    );

    eprintln!(
        "send session would be created; implementation pending\ntarget: {}",
        args.file.display()
    );

    Ok(CliExitCode::Success)
}

fn validate_file_exists(path: &PathBuf) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("file not found: {}", path.display()));
    }
    if !path.is_file() {
        return Err(format!("not a file: {}", path.display()));
    }
    Ok(())
}
