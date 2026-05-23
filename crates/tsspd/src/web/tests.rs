//! Web dashboard handler tests.

#![allow(clippy::unwrap_used)]

use axum::body::Body;
use axum::http::header::{CONTENT_SECURITY_POLICY, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS};
use axum::http::{Request, StatusCode};

use super::assets::INDEX_HTML;
use super::{serve_asset, web_fallback};

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
    assert!(INDEX_HTML.contains("view-note-editor"));
}
