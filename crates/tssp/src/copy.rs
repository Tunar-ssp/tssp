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
