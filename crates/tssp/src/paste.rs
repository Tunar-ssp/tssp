//! `tssp paste` command implementation.

use tssp_cli_core::CliExitCode;
use tssp::{Cli, PasteArgs};

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

#[cfg(test)]
mod tests {
    use super::run;
    use tssp::{Cli, ConnectionArgs, PasteArgs, LoggingArgs, OutputArgs};

    fn cli() -> Cli {
        Cli {
            command: None,
            connection: ConnectionArgs {
                host: "127.0.0.1".to_string(),
                port: 8421,
            },
            logging: LoggingArgs {
                verbose: false,
                json: false,
                quiet: false,
                no_color: false,
            },
            output: OutputArgs {
                json: false,
                quiet: false,
            },
            upload: Default::default(),
        }
    }

    #[test]
    fn paste_command_succeeds() {
        let cli = cli();
        let args = PasteArgs {
            tags: vec![],
            filename: None,
        };
        let result = run(&cli, &args);
        assert_eq!(result, Ok(CliExitCode::Success));
    }

    #[test]
    fn paste_command_with_filename_succeeds() {
        let cli = cli();
        let args = PasteArgs {
            tags: vec!["Docs".to_string()],
            filename: Some("custom.txt".to_string()),
        };
        let result = run(&cli, &args);
        assert_eq!(result, Ok(CliExitCode::Success));
    }
}
