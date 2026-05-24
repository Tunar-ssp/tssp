//! Web dashboard handler tests.

#![allow(clippy::unwrap_used)]

use std::fs;

use axum::body::Body;
use axum::http::header::{
    CACHE_CONTROL, CONTENT_SECURITY_POLICY, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS,
};
use axum::http::{Request, StatusCode};
use tempfile::tempdir;

use super::assets::{INDEX_HTML, SERVICE_WORKER};
use super::{
    serve_asset, serve_web_v2_index_from_dir, serve_web_v2_path_from_dir, web_fallback,
    WEB_V2_MISSING_MESSAGE,
};

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
async fn serve_asset_returns_service_worker_with_no_cache_header() {
    let response = serve_asset(axum::extract::Path("sw.js".to_owned())).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(CACHE_CONTROL)
            .and_then(|v| v.to_str().ok()),
        Some("no-cache")
    );
    assert_eq!(
        response
            .headers()
            .get("service-worker-allowed")
            .and_then(|v| v.to_str().ok()),
        Some("/")
    );
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
    assert_eq!(
        headers.get(CACHE_CONTROL).and_then(|v| v.to_str().ok()),
        Some("no-store, max-age=0")
    );
    let body = axum::body::to_bytes(response.into_body(), 256_000)
        .await
        .unwrap_or_else(|e| panic!("body read: {e}"));
    let text = String::from_utf8_lossy(&body);
    assert!(text.contains("TSSP"));
    assert!(text.contains("/assets/js/app.js"));
}

#[tokio::test]
async fn web_fallback_rejects_api_routes() {
    let state = crate::HttpState::test_http_state(std::env::temp_dir());
    let app = axum::Router::new().fallback(web_fallback).with_state(state);
    let response = tower::util::ServiceExt::oneshot(
        app,
        Request::get("/api/v1/files").body(Body::empty()).unwrap(),
    )
    .await
    .unwrap_or_else(|e| panic!("request failed: {e}"));
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn serve_asset_missing_returns_not_found() {
    let response = serve_asset(axum::extract::Path("missing/asset.js".to_owned())).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn embedded_index_matches_runtime_fallback() {
    assert!(INDEX_HTML.contains("login-screen"));
    assert!(INDEX_HTML.contains("view-admin"));
    assert!(INDEX_HTML.contains("view-note-editor"));
}

#[tokio::test]
async fn serve_asset_returns_new_js_modules() {
    for path in [
        "js/files.js",
        "js/notes.js",
        "js/admin.js",
        "js/features/search.js",
        "js/features/command_palette.js",
        "js/features/overview.js",
    ] {
        let response = serve_asset(axum::extract::Path(path.to_owned())).await;
        assert_eq!(response.status(), StatusCode::OK, "expected 200 for {path}");
        let ct = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            ct.contains("javascript"),
            "expected JS content-type for {path}"
        );
    }
}

#[tokio::test]
async fn index_does_not_load_pro_js() {
    // pro.js was replaced by the four focused modules; it must no longer be referenced
    assert!(
        !INDEX_HTML.contains("pro.js"),
        "index.html must not load the removed pro.js"
    );
    assert!(INDEX_HTML.contains("files.js"));
    assert!(INDEX_HTML.contains("notes.js"));
    assert!(INDEX_HTML.contains("admin.js"));
    assert!(INDEX_HTML.contains("ui/format.js"));
    assert!(INDEX_HTML.contains("features/search.js"));
    assert!(!INDEX_HTML.contains("views.js"));
    assert!(!INDEX_HTML.contains("/assets/app.js"));
}

#[tokio::test]
async fn service_worker_cache_list_matches_current_assets() {
    for path in [
        "/assets/css/tokens.css",
        "/assets/css/base.css",
        "/assets/css/layout.css",
        "/assets/css/components.css",
        "/assets/css/views.css",
        "/assets/css/mobile.css",
        "/assets/css/product.css",
        "/assets/manifest.webmanifest",
        "/assets/js/api.js",
        "/assets/js/ui/format.js",
        "/assets/js/ui/render.js",
        "/assets/js/ui/toast.js",
        "/assets/js/ui/dialogs.js",
        "/assets/js/state.js",
        "/assets/js/upload.js",
        "/assets/js/features/overview.js",
        "/assets/js/features/search.js",
        "/assets/js/features/media.js",
        "/assets/js/features/public.js",
        "/assets/js/features/command_palette.js",
        "/assets/js/features/workspaces.js",
        "/assets/js/files.js",
        "/assets/js/notes.js",
        "/assets/js/admin.js",
        "/assets/js/editor.js",
        "/assets/js/app.js",
        "/assets/app.js",
        "/assets/app.css",
    ] {
        assert!(
            SERVICE_WORKER.contains(path),
            "service worker cache list should include {path}"
        );
    }
    assert!(!SERVICE_WORKER.contains("/assets/js/pro.js"));
    assert!(!SERVICE_WORKER.contains("/assets/js/views.js"));
    assert!(SERVICE_WORKER.contains("/assets/app.js"));
    assert!(SERVICE_WORKER.contains("/assets/app.css"));
}

#[tokio::test]
async fn serve_asset_returns_ui_modules() {
    for path in [
        "js/ui/format.js",
        "js/ui/render.js",
        "js/ui/toast.js",
        "js/ui/dialogs.js",
    ] {
        let response = serve_asset(axum::extract::Path(path.to_owned())).await;
        assert_eq!(response.status(), StatusCode::OK, "missing asset: {path}");
    }
}

#[tokio::test]
async fn serve_asset_returns_console_css_via_views() {
    // Console CSS ships inside views.css — verify the stylesheet is served correctly
    let response = serve_asset(axum::extract::Path("css/views.css".to_owned())).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 256_000)
        .await
        .unwrap_or_else(|e| panic!("body read: {e}"));
    let text = String::from_utf8_lossy(&body);
    assert!(
        text.contains("console-layout"),
        "views.css must include console panel styles"
    );
}

#[tokio::test]
async fn web_v2_preview_serves_built_asset_from_directory() {
    let dir = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
    let assets_dir = dir.path().join("assets");
    fs::create_dir_all(&assets_dir).unwrap_or_else(|error| panic!("mkdir failed: {error}"));
    fs::write(assets_dir.join("index.js"), "console.log('v2');")
        .unwrap_or_else(|error| panic!("write failed: {error}"));

    let response = serve_web_v2_path_from_dir(dir.path(), "assets/index.js");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(CACHE_CONTROL)
            .and_then(|v| v.to_str().ok()),
        Some("public, max-age=86400, immutable")
    );
    assert!(response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .contains("javascript"));
}

#[tokio::test]
async fn web_v2_preview_falls_back_to_index_for_client_route() {
    let dir = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
    fs::write(
        dir.path().join("index.html"),
        "<!doctype html><title>v2</title>",
    )
    .unwrap_or_else(|error| panic!("write failed: {error}"));

    let response = serve_web_v2_path_from_dir(dir.path(), "notes");
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().get(CONTENT_SECURITY_POLICY).is_some());
    let body = axum::body::to_bytes(response.into_body(), 256_000)
        .await
        .unwrap_or_else(|error| panic!("body read: {error}"));
    assert!(String::from_utf8_lossy(&body).contains("v2"));
}

#[tokio::test]
async fn web_v2_preview_reports_missing_bundle_helpfully() {
    let dir = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
    let response = serve_web_v2_index_from_dir(dir.path());
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = axum::body::to_bytes(response.into_body(), 256_000)
        .await
        .unwrap_or_else(|error| panic!("body read: {error}"));
    assert_eq!(String::from_utf8_lossy(&body), WEB_V2_MISSING_MESSAGE);
}
