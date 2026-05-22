//! HTTP daemon foundation for `tsspd`.
//!
//! The current server exposes lifecycle and status endpoints plus an embedded
//! placeholder web shell. Feature routes will be added through application
//! services instead of placing business logic in handlers.

use std::net::{IpAddr, SocketAddr};
use std::time::Instant;

use axum::body::Body;
use axum::http::header::{CONTENT_SECURITY_POLICY, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS};
use axum::http::HeaderValue;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

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
#[derive(Debug, Clone)]
pub struct HttpState {
    started_at: Instant,
}

impl HttpState {
    /// Creates HTTP state using the current process start instant.
    #[must_use]
    pub fn new(started_at: Instant) -> Self {
        Self { started_at }
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

async fn status(
    axum::extract::State(state): axum::extract::State<HttpState>,
) -> Json<StatusResponse> {
    Json(StatusResponse {
        schema_version: 1,
        version: env!("CARGO_PKG_VERSION"),
        status: "ok",
        uptime_seconds: state.started_at.elapsed().as_secs(),
        file_count: 0,
        tag_count: 0,
        pinned_count: 0,
        recent_upload_count_24h: 0,
    })
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
    use std::time::Instant;

    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    use super::{bind_error_message, build_router, DaemonConfig, HttpState};

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
        let app = build_router(HttpState::new(Instant::now()));
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
}
