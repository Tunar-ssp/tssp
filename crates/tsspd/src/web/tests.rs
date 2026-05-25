//! Web dashboard handler tests.


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
    // Legacy CSS assets are no longer embedded; new Svelte app handles styling
    let response = serve_asset(axum::extract::Path("css/tokens.css".to_owned())).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
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
    // Index now redirects to /app-v2
    assert!(text.contains("TSSP"));
    assert!(text.contains("/app-v2"));
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
async fn embedded_index_redirects_to_app_v2() {
    // Legacy index.html now redirects to the new Svelte app at /app-v2
    assert!(INDEX_HTML.contains("/app-v2"));
}

#[tokio::test]
async fn serve_asset_returns_new_js_modules() {
    // Legacy JS modules are no longer embedded; new Svelte app at /app-v2 handles everything
    for path in [
        "js/files.js",
        "js/notes.js",
        "js/admin.js",
        "js/features/search.js",
        "js/features/command_palette.js",
        "js/features/overview.js",
    ] {
        let response = serve_asset(axum::extract::Path(path.to_owned())).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "legacy asset {path} should not exist");
    }
}

#[tokio::test]
async fn index_redirects_to_new_app() {
    // Legacy index.html has been replaced with redirect to new Svelte app
    assert!(!INDEX_HTML.contains("pro.js"));
    assert!(!INDEX_HTML.contains("files.js"));
    assert!(!INDEX_HTML.contains("notes.js"));
    assert!(INDEX_HTML.contains("/app-v2"));
}

#[tokio::test]
async fn service_worker_caches_new_app() {
    // Service worker now caches the new app at /app-v2
    assert!(SERVICE_WORKER.contains("/app-v2"));
    assert!(SERVICE_WORKER.contains("CACHE_VERSION"));
}

#[tokio::test]
async fn serve_asset_returns_ui_modules() {
    // Legacy UI modules are no longer embedded; new Svelte app handles this
    for path in [
        "js/ui/format.js",
        "js/ui/render.js",
        "js/ui/toast.js",
        "js/ui/dialogs.js",
    ] {
        let response = serve_asset(axum::extract::Path(path.to_owned())).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "legacy asset {path} should not exist");
    }
}

#[tokio::test]
async fn serve_asset_returns_console_css_via_views() {
    // Legacy CSS files are no longer embedded; new Svelte app handles styling
    let response = serve_asset(axum::extract::Path("css/views.css".to_owned())).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn web_v2_preview_serves_built_asset_from_directory() {
    let dir = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
    let assets_dir = dir.path().join("assets");
    fs::create_dir_all(&assets_dir).unwrap_or_else(|error| panic!("mkdir failed: {error}"));
    fs::write(assets_dir.join("index.js"), "console.log('v2');")
        .unwrap_or_else(|error| panic!("write failed: {error}"));

    let response = serve_web_v2_path_from_dir(dir.path(), "assets/index.js").await;
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

    let response = serve_web_v2_path_from_dir(dir.path(), "notes").await;
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
    let response = serve_web_v2_index_from_dir(dir.path()).await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = axum::body::to_bytes(response.into_body(), 256_000)
        .await
        .unwrap_or_else(|error| panic!("body read: {error}"));
    assert_eq!(String::from_utf8_lossy(&body), WEB_V2_MISSING_MESSAGE);
}
