//! Embedded web dashboard (GCS-inspired layout, Cursor dark theme).

mod assets;

#[cfg(test)]
mod tests;

use axum::body::Body;
use axum::extract::Path;
use axum::http::header::{
    CACHE_CONTROL, CONTENT_SECURITY_POLICY, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS,
};
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response};

use assets::{asset, HTML_CSP, INDEX_HTML};

fn response_with_bytes(
    bytes: &str,
    mime: &str,
    cache_control: Option<&'static str>,
) -> Response<Body> {
    let mut response = Response::new(Body::from(bytes.as_bytes().to_vec()));
    *response.status_mut() = StatusCode::OK;
    let headers = response.headers_mut();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_str(mime).unwrap_or_else(|_| HeaderValue::from_static("text/plain")),
    );
    headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    if let Some(cache_control) = cache_control {
        headers.insert(CACHE_CONTROL, HeaderValue::from_static(cache_control));
    }
    response
}

/// Serves a file from `assets/web/` at `/assets/{*path}`.
pub(crate) async fn serve_asset(Path(path): Path<String>) -> Response<Body> {
    let normalized = path.trim_start_matches('/');
    if normalized.contains("..") || normalized.is_empty() {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Some((bytes, mime)) = asset(normalized) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let cache_control = if matches!(normalized, "sw.js" | "app.js" | "app.css") {
        Some("no-cache")
    } else {
        Some("public, max-age=86400, immutable")
    };
    let mut response = response_with_bytes(bytes, mime, cache_control);
    if normalized == "sw.js" {
        response.headers_mut().insert(
            HeaderName::from_static("service-worker-allowed"),
            HeaderValue::from_static("/"),
        );
    }
    response
}

/// SPA fallback: static assets, otherwise `index.html`.
pub(crate) async fn web_fallback(
    axum::extract::State(state): axum::extract::State<crate::HttpState>,
    request: axum::extract::Request,
) -> Response<Body> {
    if !state.settings().web {
        return (
            StatusCode::NOT_FOUND,
            [(CONTENT_TYPE, "text/plain; charset=utf-8")],
            "web dashboard is disabled",
        )
            .into_response();
    }
    let path = request.uri().path();
    if path.starts_with("/api/") || path == "/metrics" {
        return StatusCode::NOT_FOUND.into_response();
    }
    if let Some(rest) = path.strip_prefix("/assets/") {
        if !rest.is_empty() && !rest.contains("..") {
            if let Some((bytes, mime)) = asset(rest) {
                let cache_control = if matches!(rest, "sw.js" | "app.js" | "app.css") {
                    Some("no-cache")
                } else {
                    Some("public, max-age=86400, immutable")
                };
                let mut response = response_with_bytes(bytes, mime, cache_control);
                if rest == "sw.js" {
                    response.headers_mut().insert(
                        HeaderName::from_static("service-worker-allowed"),
                        HeaderValue::from_static("/"),
                    );
                }
                return response;
            }
        }
        return StatusCode::NOT_FOUND.into_response();
    }

    serve_index_html()
}

fn serve_index_html() -> Response<Body> {
    let mut response = Html(INDEX_HTML.to_owned()).into_response();
    let headers = response.headers_mut();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(CONTENT_SECURITY_POLICY, HeaderValue::from_static(HTML_CSP));
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_static("no-store, max-age=0"),
    );
    response
}
