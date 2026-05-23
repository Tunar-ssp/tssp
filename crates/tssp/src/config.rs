//! Implementation of configuration management for the `tssp` CLI.

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use tssp::{Cli, ConfigAction, ConfigCommand};
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
        ConfigAction::Get(args) => {
            let config = load_config()?;
            if let Some(key) = &args.key {
                match key.as_str() {
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
                    other => {
                        eprintln!("error: unknown configuration key '{other}'");
                        return Ok(CliExitCode::Usage);
                    }
                }
            } else if cli.output.json {
                let serialized = serde_json::to_string(&config)
                    .map_err(|error| format!("failed to serialize config: {error}"))?;
                println!("{serialized}");
            } else {
                if let Some(host) = config.host {
                    println!("host = {host}");
                }
                if let Some(port) = config.port {
                    println!("port = {port}");
                }
            }
            Ok(CliExitCode::Success)
        }
        ConfigAction::Set(args) => {
            let mut config = load_config()?;
            match args.key.as_str() {
                "host" => {
                    let host = args.value.trim();
                    if host.is_empty() {
                        return Err("host cannot be empty".to_owned());
                    }
                    config.host = Some(host.to_owned());
                }
                "port" => {
                    let port = args
                        .value
                        .parse::<u16>()
                        .map_err(|_| "port must be a valid 16-bit unsigned integer".to_owned())?;
                    config.port = Some(port);
                }
                other => {
                    eprintln!("error: unknown configuration key '{other}'");
                    return Ok(CliExitCode::Usage);
                }
            }
            save_config(&config)?;
            if !cli.output.quiet {
                println!("set {} = {}", args.key, args.value);
            }
            Ok(CliExitCode::Success)
        }
        ConfigAction::Unset(args) => {
            let mut config = load_config()?;
            let mut found = false;
            match args.key.as_str() {
                "host" => {
                    if config.host.is_some() {
                        config.host = None;
                        found = true;
                    }
                }
                "port" => {
                    if config.port.is_some() {
                        config.port = None;
                        found = true;
                    }
                }
                other => {
                    eprintln!("error: unknown configuration key '{other}'");
                    return Ok(CliExitCode::Usage);
                }
            }
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
        };
        let serialized = serde_json::to_string(&config)?;
        let deserialized: ConfigFile = serde_json::from_str(&serialized)?;
        assert_eq!(config, deserialized);
        Ok(())
    }
}
