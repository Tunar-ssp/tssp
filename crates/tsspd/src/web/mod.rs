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
use std::path::{Path as FsPath, PathBuf};

use assets::{asset, HTML_CSP, INDEX_HTML};

fn response_with_bytes(
    bytes: &str,
    mime: &str,
    cache_control: Option<&'static str>,
) -> Response<Body> {
    response_with_body(Body::from(bytes.as_bytes().to_vec()), mime, cache_control)
}

fn response_with_body(
    body: Body,
    mime: &str,
    cache_control: Option<&'static str>,
) -> Response<Body> {
    let mut response = Response::new(body);
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

fn response_with_file_bytes(
    bytes: Vec<u8>,
    mime: &str,
    cache_control: Option<&'static str>,
) -> Response<Body> {
    response_with_body(Body::from(bytes), mime, cache_control)
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

const WEB_V2_MISSING_MESSAGE: &str =
    "web-v2 bundle not built; run `cd frontend && npm run build` first";

fn web_v2_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/web-v2")
}

fn web_v2_mime(path: &FsPath) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
    {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" | "map" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn serve_web_v2_index_from_dir(base: &FsPath) -> Response<Body> {
    let index = base.join("index.html");
    let Ok(bytes) = std::fs::read(&index) else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            [(CONTENT_TYPE, "text/plain; charset=utf-8")],
            WEB_V2_MISSING_MESSAGE,
        )
            .into_response();
    };
    let mut response = response_with_file_bytes(
        bytes,
        "text/html; charset=utf-8",
        Some("no-store, max-age=0"),
    );
    let headers = response.headers_mut();
    headers.insert(CONTENT_SECURITY_POLICY, HeaderValue::from_static(HTML_CSP));
    response
}

fn serve_web_v2_path_from_dir(base: &FsPath, requested: &str) -> Response<Body> {
    let normalized = requested.trim_start_matches('/');
    if normalized.contains("..") {
        return StatusCode::NOT_FOUND.into_response();
    }
    if normalized.is_empty() {
        return serve_web_v2_index_from_dir(base);
    }

    let candidate = base.join(normalized);
    if candidate.is_file() {
        let Ok(bytes) = std::fs::read(&candidate) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };
        let cache = if normalized == "index.html" {
            Some("no-store, max-age=0")
        } else {
            Some("public, max-age=86400, immutable")
        };
        let mut response = response_with_file_bytes(bytes, web_v2_mime(&candidate), cache);
        if normalized == "index.html" {
            response
                .headers_mut()
                .insert(CONTENT_SECURITY_POLICY, HeaderValue::from_static(HTML_CSP));
        }
        return response;
    }

    if normalized.starts_with("assets/") || normalized.contains('.') {
        return StatusCode::NOT_FOUND.into_response();
    }

    serve_web_v2_index_from_dir(base)
}

/// Serves the built Svelte/Vite preview bundle at `/app-v2`.
pub(crate) async fn serve_web_v2_index() -> Response<Body> {
    serve_web_v2_index_from_dir(&web_v2_dir())
}

/// Serves the built Svelte/Vite preview bundle paths at `/app-v2/{*path}`.
pub(crate) async fn serve_web_v2_path(Path(path): Path<String>) -> Response<Body> {
    serve_web_v2_path_from_dir(&web_v2_dir(), &path)
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
