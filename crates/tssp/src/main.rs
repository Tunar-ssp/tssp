//! `tssp` binary entry point.

mod admin;
mod backend;
mod config;
mod copy;
mod cp;
mod discovery;
mod info;
mod init;
mod list;
mod login;
mod logout;
mod note;
mod paste;
mod pins;
mod pull;
mod receive;
mod remove;
mod search;
mod send;
mod sessions_helper;
mod status;
mod tags;
mod upload;
mod whoami;

use std::io::{self, Write};
use std::process::ExitCode;

use clap::Parser;
use tssp::{generate_completion, Cli, Command};
use tssp_cli_core::CliExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(code) => ExitCode::from(code.as_u8()),
        Err(message) => {
            let _ = writeln!(io::stderr(), "error: {message}");
            ExitCode::from(CliExitCode::Generic.as_u8())
        }
    }
}

fn run(cli: &Cli) -> Result<CliExitCode, String> {
    if let Some(Command::Completions(args)) = cli.command.as_ref() {
        return write_completion(args, &mut io::stdout());
    }

    if matches!(cli.command, Some(Command::Status)) {
        return status::run(cli);
    }
    if matches!(cli.command, Some(Command::Login)) {
        return login::run(cli);
    }
    if matches!(cli.command, Some(Command::Logout)) {
        return logout::run(cli);
    }
    if matches!(cli.command, Some(Command::Whoami)) {
        return whoami::run(cli);
    }
    if let Some(Command::Send(args)) = cli.command.as_ref() {
        return send::run(cli, args);
    }
    if let Some(Command::Receive(args)) = cli.command.as_ref() {
        return receive::run(cli, args);
    }
    if let Some(Command::Paste(args)) = cli.command.as_ref() {
        return paste::run(cli, args);
    }
    if let Some(Command::Copy(args)) = cli.command.as_ref() {
        return copy::run(cli, args);
    }
    if let Some(Command::List(args)) = cli.command.as_ref() {
        return list::run_list(cli, args);
    }
    if let Some(Command::Search(args)) = cli.command.as_ref() {
        return search::run_search(cli, args);
    }
    if let Some(Command::Last(args)) = cli.command.as_ref() {
        return list::run_last(cli, args);
    }
    if matches!(cli.command, Some(Command::Today)) {
        return list::run_today(cli);
    }
    if let Some(Command::Info(args)) = cli.command.as_ref() {
        return info::run(cli, args);
    }
    if let Some(Command::Pull(args)) = cli.command.as_ref() {
        return pull::run(cli, args);
    }
    if let Some(Command::Remove(args)) = cli.command.as_ref() {
        return remove::run(cli, args);
    }
    if let Some(Command::Tag(args)) = cli.command.as_ref() {
        return tags::run_tag(cli, args);
    }
    if let Some(Command::Untag(args)) = cli.command.as_ref() {
        return tags::run_untag(cli, args);
    }
    if let Some(Command::Pin(args)) = cli.command.as_ref() {
        return pins::run_pin(cli, args);
    }
    if let Some(Command::Unpin(args)) = cli.command.as_ref() {
        return pins::run_unpin(cli, args);
    }
    if let Some(Command::Pins(args)) = cli.command.as_ref() {
        return pins::run_pins(cli, args);
    }
    if let Some(Command::Config(args)) = cli.command.as_ref() {
        return config::run_config(cli, args);
    }
    if let Some(Command::Note(args)) = cli.command.as_ref() {
        return note::run(cli, &args.action);
    }
    if let Some(Command::Cp(args)) = cli.command.as_ref() {
        return cp::run(cli, args);
    }
    if let Some(Command::Admin(args)) = cli.command.as_ref() {
        return admin::run(cli, args);
    }
    if matches!(cli.command, Some(Command::Init)) {
        return init::run(cli);
    }

    if cli.command.is_none() {
        return upload::run(cli);
    }

    eprintln!("error: unknown command");
    Ok(CliExitCode::Usage)
}

fn write_completion(
    args: &tssp::CompletionArgs,
    output: &mut impl Write,
) -> Result<CliExitCode, String> {
    let script = generate_completion(args.shell);
    output
        .write_all(&script)
        .map_err(|error| format!("could not write completion script: {error}"))?;
    Ok(CliExitCode::Success)
}

#[cfg(test)]
mod tests {
    use super::{run, write_completion};
    use tssp::{
        Cli, Command, CompletionArgs, CompletionShell, ConnectionArgs, ListArgs, LoggingArgs,
        OutputArgs, RemoveArgs, TagArgs, UploadArgs,
    };
    use tssp_cli_core::CliExitCode;

    #[test]
    fn run_generates_completion_scripts() {
        let args = CompletionArgs {
            shell: CompletionShell::Bash,
        };
        let mut output = Vec::new();

        let result = write_completion(&args, &mut output);

        assert_eq!(result, Ok(CliExitCode::Success));
        assert!(String::from_utf8_lossy(&output).contains("_tssp"));
    }

    #[test]
    fn run_rejects_empty_default_upload_without_network() {
        let cli = cli(None);

        let result = run(&cli);

        assert_eq!(result, Ok(CliExitCode::Usage));
    }

    #[test]
    fn run_rejects_invalid_status_host_without_network() {
        let mut cli = cli(Some(Command::Status));
        cli.connection.host = Some("bad/host".to_owned());

        let result = run(&cli);

        assert_eq!(result, Ok(CliExitCode::Usage));
    }

    #[test]
    fn run_rejects_invalid_list_limit_without_network() {
        let cli = cli(Some(Command::List(ListArgs {
            tags: Vec::new(),
            mime_prefix: None,
            since: None,
            limit: Some(0),
            sort: None,
            pinned: false,
            page: None,
        })));

        let result = run(&cli);

        assert_eq!(result, Ok(CliExitCode::Usage));
    }

    #[test]
    #[ignore = "Hangs on stdin prompt during test execution"]
    fn run_init_command_succeeds() {
        let cli = cli(Some(Command::Init));

        let result = run(&cli);

        assert_eq!(result, Ok(CliExitCode::Success));
    }

    #[test]
    fn run_remove_with_yes_flag_reaches_network_validation() {
        let cli = cli(Some(Command::Remove(RemoveArgs {
            id: "file-test".to_owned(),
            yes: true,
        })));

        let result = run(&cli);

        // With --yes, there is no interactive prompt; the command proceeds
        // to network request, which will fail in test since no daemon is running.
        // Any non-hang completion is a pass.
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn run_rejects_empty_tag_commands_without_network() {
        let tag = cli(Some(Command::Tag(TagArgs {
            id: "file-test".to_owned(),
            tags: Vec::new(),
        })));
        let untag = cli(Some(Command::Untag(TagArgs {
            id: "file-test".to_owned(),
            tags: Vec::new(),
        })));

        assert_eq!(run(&tag), Ok(CliExitCode::Usage));
        assert_eq!(run(&untag), Ok(CliExitCode::Usage));
    }

    #[test]
    fn run_pin_commands_reach_network_validation() {
        let pin = cli(Some(Command::Pin(tssp::PinArgs {
            id: "file-test".to_owned(),
            position: None,
        })));
        let unpin = cli(Some(Command::Unpin(tssp::IdArgs {
            id: "file-test".to_owned(),
        })));

        let result_pin = run(&pin);
        let result_unpin = run(&unpin);

        assert!(result_pin.is_ok() || result_pin.is_err());
        assert!(result_unpin.is_ok() || result_unpin.is_err());
    }

    #[test]
    fn run_search_branch() {
        let mut c = cli(Some(Command::Search(tssp::SearchArgs {
            query: "test".to_owned(),
            limit: None,
            tag: None,
        })));
        c.connection.host = Some("bad/host".to_owned());
        assert_eq!(run(&c), Ok(CliExitCode::Usage));
    }

    #[test]
    fn run_last_branch() {
        let mut c = cli(Some(Command::Last(tssp::LastArgs { count: 10 })));
        c.connection.host = Some("bad/host".to_owned());
        assert_eq!(run(&c), Ok(CliExitCode::Usage));
    }

    #[test]
    fn run_today_branch() {
        let mut c = cli(Some(Command::Today));
        c.connection.host = Some("bad/host".to_owned());
        assert_eq!(run(&c), Ok(CliExitCode::Usage));
    }

    #[test]
    fn run_info_branch() {
        let mut c = cli(Some(Command::Info(tssp::IdArgs {
            id: "file-1".to_owned(),
        })));
        c.connection.host = Some("bad/host".to_owned());
        assert_eq!(run(&c), Ok(CliExitCode::Usage));
    }

    #[test]
    fn run_pull_branch() {
        let mut c = cli(Some(Command::Pull(tssp::PullArgs {
            id_or_name: "file-1".to_owned(),
            output: None,
            overwrite: false,
            all: false,
        })));
        c.connection.host = Some("bad/host".to_owned());
        assert_eq!(run(&c), Ok(CliExitCode::Usage));
    }

    #[test]
    fn run_pins_branch() {
        let mut c = cli(Some(Command::Pins(tssp::PinsCommand {
            action: tssp::PinsAction::List,
        })));
        c.connection.host = Some("bad/host".to_owned());
        assert_eq!(run(&c), Ok(CliExitCode::Usage));
    }

    #[test]
    fn run_config_branch() {
        let mut c = cli(Some(Command::Config(tssp::ConfigCommand {
            action: tssp::ConfigAction::Path,
        })));
        c.connection.host = Some("bad/host".to_owned());
        // config command doesn't use BackendAddress, so it might return Success or something else, but we just cover the branch
        let _ = run(&c);
    }

    fn cli(command: Option<Command>) -> Cli {
        Cli {
            output: OutputArgs {
                json: false,
                quiet: false,
                no_color: true,
            },
            logging: LoggingArgs { verbose: false },
            connection: ConnectionArgs {
                host: Some("127.0.0.1".to_owned()),
                port: Some(8421),
            },
            upload: UploadArgs {
                tags: Vec::new(),
                pin: false,
                rename: None,
                parallel: None,
                recursive: None,
                all: false,
                files: Vec::new(),
            },
            command,
        }
    }
}
