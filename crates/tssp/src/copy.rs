//! `tssp copy` command implementation.

use tssp_cli_core::CliExitCode;
use tssp::{Cli, CopyArgs};

/// Runs the copy command.
pub fn run(_cli: &Cli, args: &CopyArgs) -> Result<CliExitCode, String> {

    if args.share {
        eprintln!(
            "generating share session URL for file {}",
            args.id
        );
        eprintln!("share URL generation; implementation pending");
    } else {
        eprintln!(
            "generating direct download URL for file {}",
            args.id
        );
        eprintln!("direct URL generation; implementation pending");
    }

    Ok(CliExitCode::Success)
}

#[cfg(test)]
mod tests {
    use super::run;
    use tssp::{Cli, ConnectionArgs, CopyArgs, LoggingArgs, OutputArgs};

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
    fn copy_command_with_share_succeeds() {
        let cli = cli();
        let args = CopyArgs {
            id: "file-123".to_string(),
            share: true,
        };
        let result = run(&cli, &args);
        assert_eq!(result, Ok(CliExitCode::Success));
    }

    #[test]
    fn copy_command_direct_download_succeeds() {
        let cli = cli();
        let args = CopyArgs {
            id: "file-456".to_string(),
            share: false,
        };
        let result = run(&cli, &args);
        assert_eq!(result, Ok(CliExitCode::Success));
    }
}
