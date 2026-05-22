//! `tssp` binary entry point.

use std::io::{self, Write};
use std::process::ExitCode;

use clap::Parser;
use tssp::{generate_completion, Cli, Command};
use tssp_cli_core::CliExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => ExitCode::from(code.as_u8()),
        Err(message) => {
            let _ = writeln!(io::stderr(), "error: {message}");
            ExitCode::from(CliExitCode::Generic.as_u8())
        }
    }
}

fn run(cli: Cli) -> Result<CliExitCode, String> {
    if let Some(Command::Completions(args)) = cli.command {
        let script = generate_completion(args.shell);
        io::stdout()
            .write_all(&script)
            .map_err(|error| format!("could not write completion script: {error}"))?;
        return Ok(CliExitCode::Success);
    }

    println!("tssp command surface is available; backend command execution is not wired yet");
    Ok(CliExitCode::Generic)
}
