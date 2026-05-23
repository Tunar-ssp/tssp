//! `tssp paste` command implementation.

use tssp::{Cli, PasteArgs};
use tssp_cli_core::CliExitCode;

/// Runs the paste command.
pub fn run(_cli: &Cli, args: &PasteArgs) -> Result<CliExitCode, String> {
    eprintln!(
        "uploading clipboard contents with {} tag(s)",
        args.tags.len()
    );

    if let Some(filename) = &args.filename {
        eprintln!("using custom filename: {}", filename);
    }

    eprintln!("clipboard content upload; implementation pending");

    Ok(CliExitCode::Success)
}
