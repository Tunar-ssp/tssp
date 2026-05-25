//! `files` route integration tests.

//! HTTP integration tests.


use super::common::*;
use super::imports::*;

#[test]
fn config_builds_socket_address() {
    let settings = DaemonSettings::default();
    assert_eq!(settings.socket_addr().to_string(), "127.0.0.1:8421");
}

#[tokio::test]
async fn health_endpoint_returns_plain_ok() {
    let app = build_router(HttpState::test_http_state(std::env::temp_dir()));
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
    let app = build_router(
        HttpState::test_http_state(std::env::temp_dir())
            .with_stats_provider(Arc::new(FixedStatsProvider)),
    );
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
    let parsed: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|error| panic!("json parse failed: {error}"));
    assert_eq!(parsed["schema_version"], 1);
    assert_eq!(parsed["file_count"], 7);
    assert_eq!(parsed["tag_count"], 3);
}

#[tokio::test]
async fn status_endpoint_reports_metadata_failure() {
    let app = build_router(
        HttpState::test_http_state(std::env::temp_dir())
            .with_stats_provider(Arc::new(FailingStatsProvider)),
    );
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
    let parsed: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|error| panic!("json parse failed: {error}"));
    assert_eq!(parsed["error"]["code"], "metadata_unavailable");
}

#[tokio::test]
async fn delete_endpoint_removes_metadata_and_is_idempotent() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
    let repository = Arc::new(
        SqliteFileRepository::open(temp.path().join("metadata.sqlite3"))
            .unwrap_or_else(|error| panic!("repository open failed: {error}")),
    );
    let storage = Arc::new(
        FilesystemBlobStore::new(temp.path().join("storage"))
            .unwrap_or_else(|error| panic!("blob store open failed: {error}")),
    );
    let stats_provider = RepositoryMetadataStatsProvider::new(repository.clone(), SystemClock);
    let upload_service = UploadService::new(
        storage.clone(),
        repository.clone(),
        UuidV7FileIdGenerator,
        SystemClock,
    );
    let delete_service = DeleteFileService::new(storage.clone(), repository);
    let app = build_router(
        HttpState::test_http_state(temp.path().join("http-upload-tmp"))
            .with_stats_provider(Arc::new(stats_provider))
            .with_upload_provider(Arc::new(ApplicationFileUploadProvider::new(upload_service)))
            .with_delete_provider(Arc::new(ApplicationFileDeleteProvider::new(delete_service)))
            .with_blob_reader(storage),
    );
    let upload = app
        .clone()
        .oneshot(multipart_request(REAL_UPLOAD_BODY))
        .await
        .unwrap_or_else(|error| panic!("upload request failed: {error}"));
    let body = response_json(upload).await;
    let id = body["id"]
        .as_str()
        .unwrap_or_else(|| panic!("uploaded id is missing"));

    let deleted = app
        .clone()
        .oneshot(delete_request(id))
        .await
        .unwrap_or_else(|error| panic!("delete request failed: {error}"));

    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        deleted
            .headers()
            .get("x-tssp-already-gone")
            .and_then(|value| value.to_str().ok()),
        Some("false")
    );
    assert_eq!(
        deleted
            .headers()
            .get("x-tssp-blob-cleaned")
            .and_then(|value| value.to_str().ok()),
        Some("false")
    );

    let found = app
        .clone()
        .oneshot(file_request(id))
        .await
        .unwrap_or_else(|error| panic!("file request failed: {error}"));
    assert_eq!(found.status(), StatusCode::NOT_FOUND);

    let deleted_again = app
        .oneshot(delete_request(id))
        .await
        .unwrap_or_else(|error| panic!("second delete request failed: {error}"));
    assert_eq!(deleted_again.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        deleted_again
            .headers()
            .get("x-tssp-already-gone")
            .and_then(|value| value.to_str().ok()),
        Some("true")
    );
}

#[tokio::test]
async fn content_endpoint_reports_missing_blob_as_gone() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
    let storage = Arc::new(
        FilesystemBlobStore::new(temp.path().join("storage"))
            .unwrap_or_else(|error| panic!("blob store open failed: {error}")),
    );
    let app = build_router(
        HttpState::test_http_state(temp.path().join("http-upload-tmp"))
            .with_stats_provider(Arc::new(SingleRecordStatsProvider))
            .with_upload_provider(Arc::new(EchoUploadProvider {
                deduplicated: false,
            }))
            .with_blob_reader(storage),
    );

    let response = app
        .oneshot(content_request("file-test", None))
        .await
        .unwrap_or_else(|error| panic!("content request failed: {error}"));

    assert_eq!(response.status(), StatusCode::GONE);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "blob_missing");
}

#[tokio::test]
async fn list_endpoint_rejects_zero_limit() {
    let app = build_router(
        HttpState::test_http_state(std::env::temp_dir())
            .with_stats_provider(Arc::new(FixedStatsProvider)),
    );
    let response = app
        .oneshot(list_request("?limit=0"))
        .await
        .unwrap_or_else(|error| panic!("request failed: {error}"));

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "invalid_request");
}

#[tokio::test]
async fn list_endpoint_rejects_limit_above_maximum() {
    let app = build_router(
        HttpState::test_http_state(std::env::temp_dir())
            .with_stats_provider(Arc::new(FixedStatsProvider)),
    );
    let response = app
        .oneshot(list_request("?limit=501"))
        .await
        .unwrap_or_else(|error| panic!("request failed: {error}"));

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "invalid_request");
    assert!(body["error"]["message"]
        .as_str()
        .is_some_and(|message| message.contains("500")));
}

#[tokio::test]
async fn list_endpoint_reports_metadata_failure() {
    let app = build_router(
        HttpState::test_http_state(std::env::temp_dir())
            .with_stats_provider(Arc::new(FailingStatsProvider)),
    );
    let response = app
        .oneshot(list_request("?limit=1"))
        .await
        .unwrap_or_else(|error| panic!("request failed: {error}"));

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "list_failed");
}

#[tokio::test]
async fn file_endpoint_rejects_invalid_id() {
    let app = build_router(
        HttpState::test_http_state(std::env::temp_dir())
            .with_stats_provider(Arc::new(FixedStatsProvider)),
    );
    let response = app
        .oneshot(file_request("bad%20id"))
        .await
        .unwrap_or_else(|error| panic!("request failed: {error}"));

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "invalid_file_id");
}

#[tokio::test]
async fn file_endpoint_returns_not_found() {
    let app = build_router(
        HttpState::test_http_state(std::env::temp_dir())
            .with_stats_provider(Arc::new(FixedStatsProvider)),
    );
    let response = app
        .oneshot(file_request("file-test"))
        .await
        .unwrap_or_else(|error| panic!("request failed: {error}"));

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "file_not_found");
}

#[tokio::test]
async fn file_endpoint_reports_metadata_failure() {
    let app = build_router(
        HttpState::test_http_state(std::env::temp_dir())
            .with_stats_provider(Arc::new(FailingStatsProvider)),
    );
    let response = app
        .oneshot(file_request("file-test"))
        .await
        .unwrap_or_else(|error| panic!("request failed: {error}"));

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "metadata_unavailable");
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
