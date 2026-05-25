//! Chunked upload system tests covering security, reliability, and correctness.

#![allow(unused_imports)]

use super::common::*;
use super::imports::*;

#[tokio::test]
async fn start_upload_validates_session_id_format() {
    let app = build_router(HttpState::test_http_state(std::env::temp_dir()));

    let req = serde_json::json!({
        "filename": "test.txt",
        "total_size": 1000000
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/files/upload/start")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    let session_id = body["session_id"].as_str().unwrap();

    // Verify session_id has correct format
    assert!(session_id.starts_with("ses_"));
    assert_eq!(session_id.len(), 40); // ses_ + 36 char UUID
}

#[tokio::test]
async fn upload_chunk_rejects_invalid_session_id_format() {
    let app = build_router(HttpState::test_http_state(std::env::temp_dir()));

    let invalid_ids = vec![
        ("invalid", "invalid"),
        ("ses_", "ses_"),
        ("ses_toolong12341234123412341234123412341", "ses_toolong"),
        ("ses_../etc/passwd", "ses_.."),
    ];

    for (invalid_id, _) in invalid_ids {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/files/upload/{}/chunk/0", invalid_id))
                    .body(Body::from("chunk data"))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Invalid session IDs that don't match the validation will get 400 or 404
        // depending on whether the route matches at all
        assert!(
            response.status() == StatusCode::BAD_REQUEST
                || response.status() == StatusCode::NOT_FOUND
        );
    }
}

#[tokio::test]
async fn upload_chunk_rejects_oversized_chunks() {
    let app = build_router(HttpState::test_http_state(std::env::temp_dir()));

    let req = serde_json::json!({
        "filename": "test.txt",
        "total_size": 500000  // 500KB total
    });

    let start_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/files/upload/start")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response_json(start_response).await;
    let session_id = body["session_id"].as_str().unwrap();

    // Try to upload chunk larger than expected
    let oversized_chunk = vec![0u8; 300_000]; // 300KB, exceeds CHUNK_SIZE for this upload
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/files/upload/{}/chunk/0", session_id))
                .body(Body::from(oversized_chunk))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"].as_str().unwrap(), "invalid_chunk_size");
}

#[tokio::test]
async fn upload_chunk_rejects_out_of_range_index() {
    let app = build_router(HttpState::test_http_state(std::env::temp_dir()));

    let req = serde_json::json!({
        "filename": "test.txt",
        "total_size": 100000  // Only 1 chunk
    });

    let start_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/files/upload/start")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response_json(start_response).await;
    let session_id = body["session_id"].as_str().unwrap();

    // Try to upload chunk 5 when only chunk 0 exists
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/files/upload/{}/chunk/5", session_id))
                .body(Body::from(vec![0u8; 1000]))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"].as_str().unwrap(), "invalid_chunk");
}

#[tokio::test]
async fn cancel_upload_rejects_invalid_session_id() {
    let app = build_router(HttpState::test_http_state(std::env::temp_dir()));

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/files/upload/invalid-session")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Invalid format returns either 400 or 404 depending on route matching
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn complete_upload_rejects_incomplete_chunks() {
    let app = build_router(HttpState::test_http_state(std::env::temp_dir()));

    let req = serde_json::json!({
        "filename": "test.txt",
        "total_size": 500000  // Will need 2 chunks
    });

    let start_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/files/upload/start")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response_json(start_response).await;
    let session_id = body["session_id"].as_str().unwrap();

    // Try to complete without uploading all chunks
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/files/upload/{}/complete", session_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"].as_str().unwrap(), "incomplete_upload");
}

#[tokio::test]
async fn session_not_found_returns_404() {
    let app = build_router(HttpState::test_http_state(std::env::temp_dir()));

    // Try to upload to non-existent session (but with valid format)
    let fake_session = "ses_00000000-0000-0000-0000-000000000000";
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/files/upload/{}/chunk/0", fake_session))
                .body(Body::from(vec![0u8; 100]))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"].as_str().unwrap(), "session_not_found");
}

#[tokio::test]
async fn cancel_upload_cleans_up_resources() {
    let temp = tempdir().unwrap();
    let repository = Arc::new(
        SqliteFileRepository::open(temp.path().join("metadata.sqlite3"))
            .unwrap_or_else(|e| panic!("repository open failed: {e}")),
    );
    let app = build_router(
        HttpState::test_http_state(temp.path().join("http-upload-tmp"))
            .with_repository(repository),
    );

    let req = serde_json::json!({
        "filename": "test.txt",
        "total_size": 500000
    });

    let start_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/files/upload/start")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response_json(start_response).await;
    let session_id = body["session_id"].as_str().unwrap();

    // Upload first chunk
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/files/upload/{}/chunk/0", session_id))
                .body(Body::from(vec![0u8; 100000]))
                .unwrap(),
        )
        .await
        .unwrap();

    // Cancel upload
    let cancel_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/files/upload/{}", session_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(cancel_response.status(), StatusCode::NO_CONTENT);

    // Try to upload to cancelled session - should fail
    let retry_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/files/upload/{}/chunk/1", session_id))
                .body(Body::from(vec![0u8; 100000]))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(retry_response.status(), StatusCode::NOT_FOUND);
}
