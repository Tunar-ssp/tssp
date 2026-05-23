//! Shared HTTP backend client helpers for CLI commands.

use std::time::Duration;

use reqwest::blocking::{Client, RequestBuilder, Response};
use tssp::ConnectionArgs;
use tssp_cli_core::CliExitCode;

const DEFAULT_HOST: &str = "127.0.0.1";
pub(crate) const DEFAULT_PORT: u16 = 8421;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const READ_TIMEOUT: Duration = Duration::from_secs(60);

/// Maximum number of retries for transient failures.
const MAX_RETRIES: u32 = 3;

/// Initial backoff delay before the first retry.
const INITIAL_BACKOFF: Duration = Duration::from_millis(250);

/// Parsed daemon address used by command handlers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackendAddress {
    host: String,
    port: u16,
}

impl BackendAddress {
    /// Builds an address from global connection flags.
    pub(crate) fn from_connection_args(args: &ConnectionArgs) -> Result<Self, String> {
        let config = crate::config::load_config().unwrap_or_default();

        let host_override = args
            .host
            .as_deref()
            .or(config.host.as_deref())
            .unwrap_or(DEFAULT_HOST)
            .trim();

        if host_override.is_empty() {
            return Err("host must not be empty".to_owned());
        }
        if host_override.contains('/') {
            return Err("host must not contain a URL path".to_owned());
        }

        let port_override = args.port.or(config.port).unwrap_or(DEFAULT_PORT);

        Ok(Self {
            host: host_override.to_owned(),
            port: port_override,
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

/// Sends a request with exponential-backoff retry on transient network errors.
///
/// Only retries on connection errors and server-side 5xx responses (except 507).
/// Client errors (4xx) and 507 are returned immediately without retry.
pub(crate) fn send_with_retry<F>(
    make_request: F,
    operation: &str,
) -> Result<Response, CliExitCode>
where
    F: Fn() -> RequestBuilder,
{
    let mut delay = INITIAL_BACKOFF;
    let mut last_err = String::new();

    for attempt in 0..=MAX_RETRIES {
        let result = make_request().send();
        match result {
            Ok(response) => {
                let status = response.status();
                // Don't retry client errors or storage-full
                if status.is_client_error() || status.as_u16() == 507 {
                    return Ok(response);
                }
                // Retry on 5xx
                if status.is_server_error() {
                    last_err = format!("server returned {status}");
                } else {
                    return Ok(response);
                }
            }
            Err(error) if error.is_connect() || error.is_timeout() => {
                last_err = format!("{error}");
            }
            Err(error) => {
                eprintln!("error: {operation}: {error}");
                return Err(CliExitCode::Network);
            }
        }

        if attempt < MAX_RETRIES {
            eprintln!(
                "warning: {operation} failed (attempt {}/{}), retrying in {:.1}s: {last_err}",
                attempt + 1,
                MAX_RETRIES + 1,
                delay.as_secs_f32()
            );
            std::thread::sleep(delay);
            delay = delay.saturating_mul(2);
        }
    }

    eprintln!("error: {operation}: {last_err} (all {MAX_RETRIES} retries exhausted)");
    Err(CliExitCode::Network)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
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
