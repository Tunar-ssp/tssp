//! Shared HTTP backend client helpers for CLI commands.

use std::time::Duration;

use reqwest::blocking::Client;
use tssp::ConnectionArgs;

const DEFAULT_HOST: &str = "127.0.0.1";
pub(crate) const DEFAULT_PORT: u16 = 8421;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const READ_TIMEOUT: Duration = Duration::from_secs(60);

/// Parsed daemon address used by command handlers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackendAddress {
    host: String,
    port: u16,
}

impl BackendAddress {
    /// Builds an address from global connection flags.
    pub(crate) fn from_connection_args(args: &ConnectionArgs) -> Result<Self, String> {
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

    /// Base HTTP URL without a trailing slash.
    pub(crate) fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    /// Absolute URL for an API path beginning with `/`.
    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url(), path)
    }
}

/// Builds the blocking HTTP client used by synchronous CLI commands.
pub(crate) fn build_client() -> Result<Client, String> {
    Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(READ_TIMEOUT)
        .build()
        .map_err(|error| format!("could not build HTTP client: {error}"))
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

        assert_eq!(
            address.url("/api/v1/status"),
            "http://127.0.0.1:8421/api/v1/status"
        );
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

    #[test]
    fn host_must_not_be_empty_after_trimming() {
        let address = BackendAddress::from_connection_args(&ConnectionArgs {
            host: Some("  ".to_owned()),
            port: Some(DEFAULT_PORT),
        });

        assert!(matches!(address, Err(message) if message.contains("empty")));
    }

    #[test]
    fn build_client_uses_valid_timeout_configuration() {
        let client = super::build_client();

        assert!(client.is_ok());
    }
}
