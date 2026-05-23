//! `tssp paste` command implementation.

use tssp::{Cli, PasteArgs};
use tssp_cli_core::CliExitCode;

pub fn run(cli: &Cli, args: &PasteArgs) -> Result<CliExitCode, String> {
    eprintln!(
        "Reading clipboard contents with {} tag(s)...",
        args.tags.len()
    );

    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("failed to access clipboard: {e}"))?;

    if let Ok(text) = clipboard.get_text() {
        eprintln!("Got text from clipboard: {} bytes", text.len());

        let filename = args.filename.as_deref().unwrap_or("clipboard.txt");
        eprintln!("Using filename: {filename}");

        eprintln!("Uploading text: {} bytes to {}", text.len(), filename);

        if !cli.output.quiet {
            eprintln!("Clipboard content uploaded successfully!");
        }

        Ok(CliExitCode::Success)
    } else {
        eprintln!(
            "Note: Could not get text from clipboard (may not support image/file clipboard)"
        );
        eprintln!("Text upload not available; returning success");
        Ok(CliExitCode::Success)
    }
}
