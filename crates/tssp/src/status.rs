//! Implementation of `tssp status`.

use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};
use tssp_cli_core::CliExitCode;

use tssp::{Cli, ConnectionArgs};

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 8421;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const READ_TIMEOUT: Duration = Duration::from_secs(60);

pub(crate) fn run(cli: &Cli) -> Result<CliExitCode, String> {
    let address = BackendAddress::from_connection_args(&cli.connection)?;
    let client = Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(READ_TIMEOUT)
        .build()
        .map_err(|error| format!("could not build HTTP client: {error}"))?;
    let response = client
        .get(address.status_url())
        .header(ACCEPT, "application/vnd.tssp.v1+json")
        .send()
        .map_err(|error| {
            eprintln!(
                "error: could not reach daemon at {}: {error}",
                address.base_url()
            );
            CliExitCode::Network
        });
    let response = match response {
        Ok(value) => value,
        Err(code) => return Ok(code),
    };

    if response.status().is_server_error() {
        eprintln!("error: daemon returned {}", response.status());
        return Ok(CliExitCode::Server);
    }

    if !response.status().is_success() {
        eprintln!("error: daemon returned {}", response.status());
        return Ok(CliExitCode::Generic);
    }

    let status = response.json::<DaemonStatus>().map_err(|error| {
        format!(
            "daemon at {} returned an invalid status response: {error}",
            address.base_url()
        )
    })?;
    print_status(&status, cli.output.json)?;
    Ok(CliExitCode::Success)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BackendAddress {
    host: String,
    port: u16,
}

impl BackendAddress {
    fn from_connection_args(args: &ConnectionArgs) -> Result<Self, String> {
        let host = args.host.as_deref().unwrap_or(DEFAULT_HOST).trim();
        if host.is_empty() {
            return Err("host must not be empty".to_owned());
        }
        if host.contains('/') {
            return Err("host must not contain a URL path".to_owned());
        }

        Ok(Self {
            host: host.to_owned(),
            port: args.port.unwrap_or(DEFAULT_PORT),
        })
    }

    fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    fn status_url(&self) -> String {
        format!("{}/api/v1/status", self.base_url())
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct DaemonStatus {
    schema_version: u8,
    version: String,
    status: String,
    uptime_seconds: u64,
    file_count: u64,
    tag_count: u64,
    pinned_count: u64,
    recent_upload_count_24h: u64,
}

fn print_status(status: &DaemonStatus, json: bool) -> Result<(), String> {
    if json {
        let encoded = serde_json::to_string(status)
            .map_err(|error| format!("could not encode status JSON: {error}"))?;
        println!("{encoded}");
        return Ok(());
    }

    println!("OK tsspd {}", status.version);
    println!("Status: {}", status.status);
    println!("Uptime: {}s", status.uptime_seconds);
    println!("Files: {}", status.file_count);
    println!("Tags: {}", status.tag_count);
    println!("Pinned: {}", status.pinned_count);
    println!("Uploaded in last 24h: {}", status.recent_upload_count_24h);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{BackendAddress, DEFAULT_PORT};
    use tssp::ConnectionArgs;

    #[test]
    fn default_status_url_uses_local_daemon() {
        let address = BackendAddress::from_connection_args(&ConnectionArgs {
            host: None,
            port: None,
        })
        .unwrap_or_else(|error| panic!("address failed: {error}"));

        assert_eq!(address.status_url(), "http://127.0.0.1:8421/api/v1/status");
    }

    #[test]
    fn explicit_host_and_port_override_defaults() {
        let address = BackendAddress::from_connection_args(&ConnectionArgs {
            host: Some("tsspd.local".to_owned()),
            port: Some(9000),
        })
        .unwrap_or_else(|error| panic!("address failed: {error}"));

        assert_eq!(address.base_url(), "http://tsspd.local:9000");
    }

    #[test]
    fn host_must_not_include_path() {
        let address = BackendAddress::from_connection_args(&ConnectionArgs {
            host: Some("127.0.0.1/api".to_owned()),
            port: Some(DEFAULT_PORT),
        });

        assert!(address.is_err());
    }
}
