//! HTTP daemon foundation for `tsspd`.
//!
//! The current server exposes lifecycle and status endpoints plus an embedded
//! placeholder web shell. Feature routes will be added through application
//! services instead of placing business logic in handlers.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Instant;

use axum::body::Body;
use axum::http::header::{CONTENT_SECURITY_POLICY, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use tssp_domain::UnixTimestamp;
use tssp_ports::{Clock, FileRepository, RepositoryStats};

/// Server configuration required to bind the daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonConfig {
    /// IP address to bind.
    pub bind: IpAddr,
    /// TCP port to listen on.
    pub port: u16,
}

impl DaemonConfig {
    /// Returns the socket address represented by the config.
    #[must_use]
    pub const fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.bind, self.port)
    }
}

/// Shared HTTP state.
#[derive(Clone)]
pub struct HttpState {
    started_at: Instant,
    stats_provider: Arc<dyn MetadataStatsProvider>,
}

impl HttpState {
    /// Creates HTTP state using the current process start instant.
    #[must_use]
    pub fn new(started_at: Instant) -> Self {
        Self {
            started_at,
            stats_provider: Arc::new(StaticMetadataStatsProvider),
        }
    }

    /// Creates HTTP state with a real metadata stats provider.
    #[must_use]
    pub fn with_stats_provider(
        started_at: Instant,
        stats_provider: Arc<dyn MetadataStatsProvider>,
    ) -> Self {
        Self {
            started_at,
            stats_provider,
        }
    }
}

/// Supplies metadata counts to the status endpoint.
pub trait MetadataStatsProvider: Send + Sync {
    /// Returns the latest metadata stats.
    ///
    /// # Errors
    ///
    /// Returns a short diagnostic when counts cannot be read.
    fn stats(&self) -> Result<RepositoryStats, String>;
}

#[derive(Debug)]
struct StaticMetadataStatsProvider;

impl MetadataStatsProvider for StaticMetadataStatsProvider {
    fn stats(&self) -> Result<RepositoryStats, String> {
        Ok(RepositoryStats {
            file_count: 0,
            tag_count: 0,
            pinned_count: 0,
            recent_upload_count: 0,
        })
    }
}

/// Metadata stats provider backed by a repository and clock.
#[derive(Debug)]
pub struct RepositoryMetadataStatsProvider<R, C> {
    repository: R,
    clock: C,
}

impl<R, C> RepositoryMetadataStatsProvider<R, C> {
    /// Creates a repository-backed stats provider.
    #[must_use]
    pub const fn new(repository: R, clock: C) -> Self {
        Self { repository, clock }
    }
}

impl<R, C> MetadataStatsProvider for RepositoryMetadataStatsProvider<R, C>
where
    R: FileRepository + Send + Sync,
    C: Clock + Send + Sync,
{
    fn stats(&self) -> Result<RepositoryStats, String> {
        let now = self.clock.now();
        let cutoff = now.seconds().saturating_sub(86_400);
        let recent_since = UnixTimestamp::new(cutoff).map_err(|error| error.to_string())?;
        self.repository
            .stats_since(recent_since)
            .map_err(|error| error.to_string())
    }
}

/// Builds the daemon router.
pub fn build_router(state: HttpState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/api/v1/status", get(status))
        .fallback(web_fallback)
        .with_state(state)
}

async fn healthz() -> impl IntoResponse {
    ([(CONTENT_TYPE, "text/plain; charset=utf-8")], "ok")
}

async fn readyz() -> impl IntoResponse {
    ([(CONTENT_TYPE, "text/plain; charset=utf-8")], "ready")
}

async fn status(axum::extract::State(state): axum::extract::State<HttpState>) -> Response {
    match state.stats_provider.stats() {
        Ok(repository_stats) => Json(StatusResponse {
            schema_version: 1,
            version: env!("CARGO_PKG_VERSION"),
            status: "ok",
            uptime_seconds: state.started_at.elapsed().as_secs(),
            file_count: repository_stats.file_count,
            tag_count: repository_stats.tag_count,
            pinned_count: repository_stats.pinned_count,
            recent_upload_count_24h: repository_stats.recent_upload_count,
        })
        .into_response(),
        Err(message) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "metadata_unavailable",
                    message,
                },
            }),
        )
            .into_response(),
    }
}

async fn web_fallback() -> Response<Body> {
    let mut response = Html(WEB_PLACEHOLDER).into_response();
    let headers = response.headers_mut();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(
        CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; connect-src 'self'; style-src 'self' 'unsafe-inline'",
        ),
    );
    response
}

/// Minimal status response consumed by `tssp status`.
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    /// Stable response schema version.
    pub schema_version: u8,
    /// Daemon version.
    pub version: &'static str,
    /// Human-readable health state.
    pub status: &'static str,
    /// Seconds since process startup.
    pub uptime_seconds: u64,
    /// Indexed file count.
    pub file_count: u64,
    /// Known tag count.
    pub tag_count: u64,
    /// Pinned file count.
    pub pinned_count: u64,
    /// Uploads in the last 24 hours.
    pub recent_upload_count_24h: u64,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

const WEB_PLACEHOLDER: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>TSSP</title>
  <style>
    :root { color-scheme: light dark; font-family: system-ui, sans-serif; }
    body { margin: 0; min-height: 100vh; display: grid; place-items: center; }
    main { max-width: 42rem; padding: 2rem; }
    h1 { font-size: clamp(2rem, 8vw, 4rem); margin: 0 0 1rem; }
    p { line-height: 1.6; }
  </style>
</head>
<body>
  <main>
    <h1>TSSP</h1>
    <p>The embedded web shell is available. API connectivity starts at <code>/api/v1/status</code>.</p>
  </main>
</body>
</html>"#;

/// Maps startup bind failures to concise user-facing messages.
#[must_use]
pub fn bind_error_message(address: SocketAddr, error: &std::io::Error) -> String {
    if error.kind() == std::io::ErrorKind::AddrInUse {
        return format!(
            "port {} is already in use; choose another port with --port",
            address.port()
        );
    }

    format!("could not bind {address}: {error}")
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use std::sync::Arc;
    use std::time::Instant;

    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    use super::{bind_error_message, build_router, DaemonConfig, HttpState, MetadataStatsProvider};
    use tssp_ports::RepositoryStats;

    #[test]
    fn config_builds_socket_address() {
        let config = DaemonConfig {
            bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 8421,
        };

        assert_eq!(config.socket_addr().to_string(), "127.0.0.1:8421");
    }

    #[tokio::test]
    async fn health_endpoint_returns_plain_ok() {
        let app = build_router(HttpState::new(Instant::now()));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap_or_else(|error| panic!("request build failed: {error}")),
            )
            .await
            .unwrap_or_else(|error| panic!("request failed: {error}"));

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 64)
            .await
            .unwrap_or_else(|error| panic!("body read failed: {error}"));
        assert_eq!(body.as_ref(), b"ok");
    }

    #[tokio::test]
    async fn status_endpoint_returns_schema_version() {
        let app = build_router(HttpState::with_stats_provider(
            Instant::now(),
            Arc::new(FixedStatsProvider),
        ));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/status")
                    .body(Body::empty())
                    .unwrap_or_else(|error| panic!("request build failed: {error}")),
            )
            .await
            .unwrap_or_else(|error| panic!("request failed: {error}"));

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024)
            .await
            .unwrap_or_else(|error| panic!("body read failed: {error}"));
        let parsed: serde_json::Value = serde_json::from_slice(&body)
            .unwrap_or_else(|error| panic!("json parse failed: {error}"));
        assert_eq!(parsed["schema_version"], 1);
        assert_eq!(parsed["file_count"], 7);
        assert_eq!(parsed["tag_count"], 3);
    }

    #[tokio::test]
    async fn status_endpoint_reports_metadata_failure() {
        let app = build_router(HttpState::with_stats_provider(
            Instant::now(),
            Arc::new(FailingStatsProvider),
        ));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/status")
                    .body(Body::empty())
                    .unwrap_or_else(|error| panic!("request build failed: {error}")),
            )
            .await
            .unwrap_or_else(|error| panic!("request failed: {error}"));

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = to_bytes(response.into_body(), 1024)
            .await
            .unwrap_or_else(|error| panic!("body read failed: {error}"));
        let parsed: serde_json::Value = serde_json::from_slice(&body)
            .unwrap_or_else(|error| panic!("json parse failed: {error}"));
        assert_eq!(parsed["error"]["code"], "metadata_unavailable");
    }

    #[test]
    fn bind_error_mentions_busy_port() {
        let message = bind_error_message(
            "127.0.0.1:8421"
                .parse()
                .unwrap_or_else(|error| panic!("socket parse failed: {error}")),
            &std::io::Error::from(std::io::ErrorKind::AddrInUse),
        );

        assert!(message.contains("8421"));
        assert!(message.contains("--port"));
    }

    struct FixedStatsProvider;

    impl MetadataStatsProvider for FixedStatsProvider {
        fn stats(&self) -> Result<RepositoryStats, String> {
            Ok(RepositoryStats {
                file_count: 7,
                tag_count: 3,
                pinned_count: 2,
                recent_upload_count: 1,
            })
        }
    }

    struct FailingStatsProvider;

    impl MetadataStatsProvider for FailingStatsProvider {
        fn stats(&self) -> Result<RepositoryStats, String> {
            Err("metadata database is unavailable".to_owned())
        }
    }
}
