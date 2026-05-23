//! Implementation of configuration management for the `tssp` CLI.

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use tssp::{Cli, ConfigAction, ConfigCommand, ConfigGetArgs, ConfigSetArgs, ConfigUnsetArgs};
use tssp_cli_core::CliExitCode;

const CONFIG_DIR_NAME: &str = "tssp";
const CONFIG_FILE_NAME: &str = "config.json";

/// Config file structure.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigFile {
    /// Host override.
    pub host: Option<String>,
    /// Port override.
    pub port: Option<u16>,
    /// Bearer token for remote daemon access.
    pub token: Option<String>,
    /// Full base URL override (`https://cloud.example.com`).
    pub url: Option<String>,
    /// URL scheme when not using `url`.
    pub scheme: Option<String>,
    /// Discover daemon via mDNS when host is not configured.
    pub discovery: Option<bool>,
}

/// Resolves the standard configuration file path for the current OS.
pub fn resolve_config_path() -> Result<PathBuf, String> {
    let home =
        std::env::var("HOME").map_err(|_| "HOME environment variable is not defined".to_owned())?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join(CONFIG_DIR_NAME)
        .join(CONFIG_FILE_NAME))
}

/// Loads config file if it exists, otherwise returns a default empty configuration.
pub fn load_config() -> Result<ConfigFile, String> {
    let path = resolve_config_path()?;
    if !path.exists() {
        return Ok(ConfigFile::default());
    }

    let mut file =
        File::open(&path).map_err(|error| format!("failed to open config file: {error}"))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|error| format!("failed to read config file: {error}"))?;

    if content.trim().is_empty() {
        return Ok(ConfigFile::default());
    }

    serde_json::from_str(&content).map_err(|error| format!("failed to parse config JSON: {error}"))
}

/// Saves the configuration to disk.
pub fn save_config(config: &ConfigFile) -> Result<(), String> {
    let path = resolve_config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create config directory: {error}"))?;
    }

    let content = serde_json::to_string_pretty(config)
        .map_err(|error| format!("failed to serialize config: {error}"))?;

    let mut file =
        File::create(&path).map_err(|error| format!("failed to create config file: {error}"))?;
    file.write_all(content.as_bytes())
        .map_err(|error| format!("failed to write config file: {error}"))?;

    Ok(())
}

/// Runs `tssp config` subcommand.
pub(crate) fn run_config(cli: &Cli, command: &ConfigCommand) -> Result<CliExitCode, String> {
    match &command.action {
        ConfigAction::Path => {
            let path = resolve_config_path()?;
            println!("{}", path.display());
            Ok(CliExitCode::Success)
        }
        ConfigAction::Get(args) => run_get(cli, args),
        ConfigAction::Set(args) => run_set(cli, args),
        ConfigAction::Unset(args) => run_unset(cli, args),
    }
}

fn run_get(cli: &Cli, args: &ConfigGetArgs) -> Result<CliExitCode, String> {
    let config = load_config()?;
    if let Some(key) = &args.key {
        return Ok(print_config_key(config, key));
    }
    if cli.output.json {
        let serialized = serde_json::to_string(&config)
            .map_err(|error| format!("failed to serialize config: {error}"))?;
        println!("{serialized}");
    } else {
        print_config_human(config);
    }
    Ok(CliExitCode::Success)
}

fn print_config_key(config: ConfigFile, key: &str) -> CliExitCode {
    match key {
        "host" => {
            if let Some(host) = config.host {
                println!("{host}");
            }
        }
        "port" => {
            if let Some(port) = config.port {
                println!("{port}");
            }
        }
        "token" => {
            if let Some(token) = config.token {
                println!("{token}");
            }
        }
        other => {
            eprintln!("error: unknown configuration key '{other}'");
            return CliExitCode::Usage;
        }
    }
    CliExitCode::Success
}

fn print_config_human(config: ConfigFile) {
    if let Some(host) = config.host {
        println!("host = {host}");
    }
    if let Some(port) = config.port {
        println!("port = {port}");
    }
    if config.token.is_some() {
        println!("token = (set)");
    }
}

fn run_set(cli: &Cli, args: &ConfigSetArgs) -> Result<CliExitCode, String> {
    let mut config = load_config()?;
    if !set_config_value(&mut config, &args.key, &args.value)? {
        return Ok(CliExitCode::Usage);
    }
    save_config(&config)?;
    if !cli.output.quiet {
        println!("set {} = {}", args.key, args.value);
    }
    Ok(CliExitCode::Success)
}

fn set_config_value(config: &mut ConfigFile, key: &str, value: &str) -> Result<bool, String> {
    match key {
        "host" => {
            let host = value.trim();
            if host.is_empty() {
                return Err("host cannot be empty".to_owned());
            }
            config.host = Some(host.to_owned());
        }
        "port" => {
            let port = value
                .parse::<u16>()
                .map_err(|_| "port must be a valid 16-bit unsigned integer".to_owned())?;
            config.port = Some(port);
        }
        "token" => {
            let token = value.trim();
            if token.is_empty() {
                return Err("token cannot be empty".to_owned());
            }
            config.token = Some(token.to_owned());
        }
        other => {
            eprintln!("error: unknown configuration key '{other}'");
            return Ok(false);
        }
    }
    Ok(true)
}

fn run_unset(cli: &Cli, args: &ConfigUnsetArgs) -> Result<CliExitCode, String> {
    let mut config = load_config()?;
    let Some(found) = unset_config_value(&mut config, &args.key) else {
        return Ok(CliExitCode::Usage);
    };
    if found {
        save_config(&config)?;
        if !cli.output.quiet {
            println!("unset {}", args.key);
        }
    } else if !cli.output.quiet {
        println!("key {} was already not set", args.key);
    }
    Ok(CliExitCode::Success)
}

fn unset_config_value(config: &mut ConfigFile, key: &str) -> Option<bool> {
    match key {
        "host" => Some(config.host.take().is_some()),
        "port" => Some(config.port.take().is_some()),
        "token" => Some(config.token.take().is_some()),
        other => {
            eprintln!("error: unknown configuration key '{other}'");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_file_serialization() -> Result<(), serde_json::Error> {
        let config = ConfigFile {
            host: Some("192.168.1.5".to_owned()),
            port: Some(9999),
            token: Some("secret-token".to_owned()),
            url: None,
            scheme: None,
            discovery: Some(true),
        };
        let serialized = serde_json::to_string(&config)?;
        let deserialized: ConfigFile = serde_json::from_str(&serialized)?;
        assert_eq!(config, deserialized);
        Ok(())
    }
}
