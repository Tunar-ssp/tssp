//! Folder route integration tests.

use super::common::*;
use super::imports::*;

#[tokio::test]
async fn folders_endpoint_returns_json_not_spa_fallback() {
    let app = build_router(
        HttpState::test_http_state(std::env::temp_dir())
            .with_stats_provider(Arc::new(FixedStatsProvider)),
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/folders")
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request build failed: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("request failed: {error}"));

    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    assert!(
        content_type.starts_with("application/json"),
        "unexpected content-type: {content_type}"
    );

    let body = to_bytes(response.into_body(), 1024)
        .await
        .unwrap_or_else(|error| panic!("body read failed: {error}"));
    let parsed: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|error| panic!("json parse failed: {error}"));
    assert_eq!(parsed["schema_version"], 1);
    assert!(parsed["folders"].is_array());
}
