//! Embedded web dashboard (GCS-inspired layout, Cursor dark theme).

use axum::body::Body;
use axum::extract::Path;
use axum::http::header::{
    CACHE_CONTROL, CONTENT_SECURITY_POLICY, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS,
};
use axum::http::{HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response};

const INDEX_HTML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/index.html"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/manifest.webmanifest"
));
const SERVICE_WORKER: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/web/sw.js"));

const CSS_TOKENS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/css/tokens.css"
));
const CSS_BASE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/css/base.css"
));
const CSS_LAYOUT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/css/layout.css"
));
const CSS_COMPONENTS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/css/components.css"
));
const CSS_VIEWS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/css/views.css"
));
const CSS_MOBILE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/css/mobile.css"
));

const JS_API: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/web/js/api.js"));
const JS_STATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/js/state.js"
));
const JS_UPLOAD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/js/upload.js"
));
const JS_VIEWS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/js/views.js"
));
const JS_APP: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/web/js/app.js"));

const HTML_CSP: &str =
    "default-src 'self'; connect-src 'self'; style-src 'self'; script-src 'self'; \
     img-src 'self' data: blob:; base-uri 'self'; form-action 'self'";

fn asset(path: &str) -> Option<(&'static str, &'static str)> {
    match path {
        "index.html" => Some((INDEX_HTML, "text/html; charset=utf-8")),
        "manifest.webmanifest" => Some((MANIFEST, "application/manifest+json; charset=utf-8")),
        "sw.js" => Some((SERVICE_WORKER, "application/javascript; charset=utf-8")),
        "css/tokens.css" => Some((CSS_TOKENS, "text/css; charset=utf-8")),
        "css/base.css" => Some((CSS_BASE, "text/css; charset=utf-8")),
        "css/layout.css" => Some((CSS_LAYOUT, "text/css; charset=utf-8")),
        "css/components.css" => Some((CSS_COMPONENTS, "text/css; charset=utf-8")),
        "css/views.css" => Some((CSS_VIEWS, "text/css; charset=utf-8")),
        "css/mobile.css" => Some((CSS_MOBILE, "text/css; charset=utf-8")),
        "js/api.js" => Some((JS_API, "application/javascript; charset=utf-8")),
        "js/state.js" => Some((JS_STATE, "application/javascript; charset=utf-8")),
        "js/upload.js" => Some((JS_UPLOAD, "application/javascript; charset=utf-8")),
        "js/views.js" => Some((JS_VIEWS, "application/javascript; charset=utf-8")),
        "js/app.js" => Some((JS_APP, "application/javascript; charset=utf-8")),
        _ => None,
    }
}

fn response_with_bytes(bytes: &str, mime: &str, cacheable: bool) -> Response<Body> {
    let mut response = Response::new(Body::from(bytes.as_bytes().to_vec()));
    *response.status_mut() = StatusCode::OK;
    let headers = response.headers_mut();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_str(mime).unwrap_or_else(|_| HeaderValue::from_static("text/plain")),
    );
    headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    if cacheable {
        headers.insert(
            CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=86400, immutable"),
        );
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
    response_with_bytes(bytes, mime, true)
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
                return response_with_bytes(bytes, mime, true);
            }
        }
        return StatusCode::NOT_FOUND.into_response();
    }

    serve_index_html().await
}

async fn serve_index_html() -> Response<Body> {
    let mut response = Html(INDEX_HTML.to_owned()).into_response();
    let headers = response.headers_mut();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(CONTENT_SECURITY_POLICY, HeaderValue::from_static(HTML_CSP));
    response
}

#[cfg(test)]
mod tests {
    use super::{serve_asset, web_fallback, INDEX_HTML};
    use axum::body::Body;
    use axum::http::header::{CONTENT_SECURITY_POLICY, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS};
    use axum::http::{Request, StatusCode};

    #[tokio::test]
    async fn serve_asset_returns_modular_css() {
        let response = serve_asset(axum::extract::Path("css/tokens.css".to_owned())).await;
        assert_eq!(response.status(), StatusCode::OK);
        let ct = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(ct.contains("text/css"));
    }

    #[tokio::test]
    async fn serve_asset_rejects_path_traversal() {
        let response = serve_asset(axum::extract::Path("../secret".to_owned())).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn web_fallback_serves_index_with_security_headers() {
        let state = crate::HttpState::test_http_state(std::env::temp_dir());
        let app = axum::Router::new().fallback(web_fallback).with_state(state);
        let response =
            tower::util::ServiceExt::oneshot(app, Request::get("/").body(Body::empty()).unwrap())
                .await
                .unwrap_or_else(|e| panic!("request failed: {e}"));
        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers();
        assert!(headers.get(CONTENT_SECURITY_POLICY).is_some());
        assert_eq!(
            headers
                .get(X_CONTENT_TYPE_OPTIONS)
                .and_then(|v| v.to_str().ok()),
            Some("nosniff")
        );
        let body = axum::body::to_bytes(response.into_body(), 256_000)
            .await
            .unwrap_or_else(|e| panic!("body read: {e}"));
        let text = String::from_utf8_lossy(&body);
        assert!(text.contains("TSSP"));
        assert!(text.contains("/assets/js/app.js"));
    }

    #[tokio::test]
    async fn embedded_index_matches_runtime_fallback() {
        assert!(INDEX_HTML.contains("login-screen"));
        assert!(INDEX_HTML.contains("view-admin"));
    }
}
