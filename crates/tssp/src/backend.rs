//! Shared HTTP backend client helpers for CLI commands.

use std::time::Duration;

use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::header::AUTHORIZATION;
use tssp::ConnectionArgs;
use tssp_cli_core::CliExitCode;

const DEFAULT_HOST: &str = "127.0.0.1";
pub(crate) const DEFAULT_PORT: u16 = 8421;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const READ_TIMEOUT: Duration = Duration::from_secs(60);

/// Maximum number of retries for transient failures.
#[allow(dead_code)]
const MAX_RETRIES: u32 = 3;

/// Initial backoff delay before the first retry.
#[allow(dead_code)]
const INITIAL_BACKOFF: Duration = Duration::from_millis(250);

/// Parsed daemon address used by command handlers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackendAddress {
    base: String,
}

impl BackendAddress {
    /// Builds an address from global connection flags and config file.
    pub(crate) fn from_connection_args(args: &ConnectionArgs) -> Result<Self, String> {
        let config = crate::config::load_config().unwrap_or_default();

        if let Some(url) = config.url.as_deref() {
            return Self::from_base_url(url.trim());
        }

        let discovery_enabled = config.discovery.unwrap_or(true);
        let mut host = args
            .host
            .as_deref()
            .or(config.host.as_deref())
            .unwrap_or(DEFAULT_HOST)
            .trim()
            .to_owned();

        if host.is_empty() {
            return Err("host must not be empty".to_owned());
        }
        if host.contains('/') {
            return Err(
                "host must not contain a URL path; use config url for full URLs".to_owned(),
            );
        }

        if host == DEFAULT_HOST && args.host.is_none() && config.host.is_none() && discovery_enabled
        {
            if let Some(discovered) = crate::discovery::discover_daemon(true) {
                host = discovered.host;
            }
        }

        let port = args.port.or(config.port).unwrap_or(DEFAULT_PORT);
        let scheme = config.scheme.as_deref().unwrap_or("http");
        let base = format!("{scheme}://{host}:{port}");
        Self::from_base_url(&base)
    }

    fn from_base_url(url: &str) -> Result<Self, String> {
        let trimmed = url.trim_end_matches('/');
        if trimmed.is_empty() {
            return Err("base URL must not be empty".to_owned());
        }
        if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
            return Err("base URL must start with http:// or https://".to_owned());
        }
        Ok(Self {
            base: trimmed.to_owned(),
        })
    }

    /// Base HTTP URL without a trailing slash.
    pub(crate) fn base_url(&self) -> String {
        self.base.clone()
    }

    /// Absolute URL for an API path beginning with `/`.
    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}{}", self.base, path)
    }

    /// Configured host name (parsed from base URL).
    pub(crate) fn host(&self) -> &str {
        self.base
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split(':')
            .next()
            .unwrap_or(DEFAULT_HOST)
    }

    /// Configured port (parsed from base URL).
    pub(crate) fn port(&self) -> u16 {
        self.base
            .rsplit(':')
            .next()
            .and_then(|p| p.parse().ok())
            .unwrap_or(DEFAULT_PORT)
    }
}

/// Starts an authorized GET request.
pub(crate) fn api_get(client: &Client, url: &str) -> RequestBuilder {
    authorize(client.get(url))
}

/// Starts an authorized POST request.
pub(crate) fn api_post(client: &Client, url: &str) -> RequestBuilder {
    authorize(client.post(url))
}

/// Starts an authorized PUT request.
pub(crate) fn api_put(client: &Client, url: &str) -> RequestBuilder {
    authorize(client.put(url))
}

/// Starts an authorized DELETE request.
pub(crate) fn api_delete(client: &Client, url: &str) -> RequestBuilder {
    authorize(client.delete(url))
}

/// Attaches a stored bearer token when configured.
pub(crate) fn authorize(request: RequestBuilder) -> RequestBuilder {
    let token = crate::config::load_config()
        .ok()
        .and_then(|config| config.token)
        .filter(|value| !value.trim().is_empty());
    if let Some(token) = token {
        return request.header(AUTHORIZATION, format!("Bearer {token}"));
    }
    request
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
#[allow(dead_code)]
pub(crate) fn send_with_retry<F>(make_request: F, operation: &str) -> Result<Response, CliExitCode>
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
                if status.is_client_error() || status.as_u16() == 507 {
                    return Ok(response);
                }
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
    use super::BackendAddress;
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
}
