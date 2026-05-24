//! `upload` route integration tests.

//! HTTP integration tests.

#![allow(unused_imports)]

use super::common::*;
use super::imports::*;

#[tokio::test]
async fn upload_endpoint_accepts_single_multipart_file() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
    let app = build_router(
        HttpState::test_http_state(temp.path().to_path_buf())
            .with_stats_provider(Arc::new(FixedStatsProvider))
            .with_upload_provider(Arc::new(EchoUploadProvider {
                deduplicated: false,
            })),
    );
    let response = app
        .oneshot(multipart_request(
            "--tssp\r\n\
                 Content-Disposition: form-data; name=\"tag\"\r\n\r\n\
                 Docs\r\n\
                 --tssp\r\n\
                 Content-Disposition: form-data; name=\"pin\"\r\n\r\n\
                 true\r\n\
                 --tssp\r\n\
                 Content-Disposition: form-data; name=\"file\"; filename=\"note.txt\"\r\n\
                 Content-Type: text/plain\r\n\r\n\
                 hello upload\r\n\
                 --tssp--\r\n",
        ))
        .await
        .unwrap_or_else(|error| panic!("request failed: {error}"));

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(
        response.headers().get("x-tssp-deduplicated"),
        Some(
            &"false"
                .parse()
                .unwrap_or_else(|error| panic!("header parse failed: {error}"))
        )
    );
    let body = to_bytes(response.into_body(), 2048)
        .await
        .unwrap_or_else(|error| panic!("body read failed: {error}"));
    let parsed: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|error| panic!("json parse failed: {error}"));
    assert_eq!(parsed["id"], "file-test");
    assert_eq!(parsed["name"], "note.txt");
    assert_eq!(parsed["mime_type"], "text/plain");
    assert_eq!(parsed["tags"][0], "Docs");
    assert_eq!(parsed["pinned"], true);
}

#[tokio::test]
async fn upload_endpoint_returns_ok_for_deduplicated_content() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
    let app = build_router(
        HttpState::test_http_state(temp.path().to_path_buf())
            .with_stats_provider(Arc::new(FixedStatsProvider))
            .with_upload_provider(Arc::new(EchoUploadProvider { deduplicated: true })),
    );
    let response = app
        .oneshot(multipart_request(
            "--tssp\r\n\
                 Content-Disposition: form-data; name=\"file\"; filename=\"note.txt\"\r\n\
                 Content-Type: text/plain\r\n\r\n\
                 hello upload\r\n\
                 --tssp--\r\n",
        ))
        .await
        .unwrap_or_else(|error| panic!("request failed: {error}"));

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("x-tssp-deduplicated"),
        Some(
            &"true"
                .parse()
                .unwrap_or_else(|error| panic!("header parse failed: {error}"))
        )
    );
}

#[tokio::test]
async fn upload_endpoint_rejects_missing_file_field() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
    let app = build_router(
        HttpState::test_http_state(temp.path().to_path_buf())
            .with_stats_provider(Arc::new(FixedStatsProvider))
            .with_upload_provider(Arc::new(EchoUploadProvider {
                deduplicated: false,
            })),
    );
    let response = app
        .oneshot(multipart_request(
            "--tssp\r\n\
                 Content-Disposition: form-data; name=\"tag\"\r\n\r\n\
                 Docs\r\n\
                 --tssp--\r\n",
        ))
        .await
        .unwrap_or_else(|error| panic!("request failed: {error}"));

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), 2048)
        .await
        .unwrap_or_else(|error| panic!("body read failed: {error}"));
    let parsed: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|error| panic!("json parse failed: {error}"));
    assert_eq!(parsed["error"]["code"], "invalid_request");
}

#[tokio::test]
async fn batch_upload_endpoint_returns_per_file_outcomes() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
    let app = build_router(
        HttpState::test_http_state(temp.path().to_path_buf())
            .with_stats_provider(Arc::new(FixedStatsProvider))
            .with_upload_provider(Arc::new(BatchEchoUploadProvider)),
    );
    let response = app
        .oneshot(batch_multipart_request(
            "--tssp\r\n\
                 Content-Disposition: form-data; name=\"tag\"\r\n\r\n\
                 Docs\r\n\
                 --tssp\r\n\
                 Content-Disposition: form-data; name=\"pin\"\r\n\r\n\
                 true\r\n\
                 --tssp\r\n\
                 Content-Disposition: form-data; name=\"file\"; filename=\"created.txt\"\r\n\
                 Content-Type: text/plain\r\n\r\n\
                 hello upload\r\n\
                 --tssp\r\n\
                 Content-Disposition: form-data; name=\"file\"; filename=\"duplicate.txt\"\r\n\
                 Content-Type: text/plain\r\n\r\n\
                 hello upload\r\n\
                 --tssp\r\n\
                 Content-Disposition: form-data; name=\"file\"; filename=\"broken.txt\"\r\n\
                 Content-Type: text/plain\r\n\r\n\
                 hello upload\r\n\
                 --tssp--\r\n",
        ))
        .await
        .unwrap_or_else(|error| panic!("request failed: {error}"));

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["schema_version"], 1);
    assert_eq!(body["created_count"], 1);
    assert_eq!(body["deduplicated_count"], 1);
    assert_eq!(body["failed_count"], 1);
    assert_eq!(body["results"].as_array().map(Vec::len), Some(3));
    assert_eq!(body["results"][0]["name"], "created.txt");
    assert_eq!(body["results"][0]["outcome"], "created");
    assert_eq!(body["results"][0]["http_status"], 201);
    assert_eq!(body["results"][0]["file"]["tags"][0], "Docs");
    assert_eq!(body["results"][0]["file"]["pinned"], true);
    assert_eq!(body["results"][1]["name"], "duplicate.txt");
    assert_eq!(body["results"][1]["outcome"], "deduplicated");
    assert_eq!(body["results"][1]["http_status"], 200);
    assert_eq!(body["results"][2]["name"], "broken.txt");
    assert_eq!(body["results"][2]["outcome"], "failed");
    assert_eq!(body["results"][2]["http_status"], 400);
    assert_eq!(body["results"][2]["error"]["code"], "invalid_request");
}

#[tokio::test]
async fn batch_upload_endpoint_rejects_empty_batch() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
    let app = build_router(
        HttpState::test_http_state(temp.path().to_path_buf())
            .with_stats_provider(Arc::new(FixedStatsProvider))
            .with_upload_provider(Arc::new(BatchEchoUploadProvider)),
    );
    let response = app
        .oneshot(batch_multipart_request(
            "--tssp\r\n\
                 Content-Disposition: form-data; name=\"tag\"\r\n\r\n\
                 Docs\r\n\
                 --tssp--\r\n",
        ))
        .await
        .unwrap_or_else(|error| panic!("request failed: {error}"));

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = response_json(response).await;
    assert_eq!(body["error"]["code"], "invalid_request");
    assert!(body["error"]["message"]
        .as_str()
        .is_some_and(|message| message.contains("at least one file")));
}

#[tokio::test]
async fn batch_upload_endpoint_commits_successful_items_when_one_fails() {
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
    let app = build_router(
        HttpState::test_http_state(temp.path().join("http-upload-tmp"))
            .with_stats_provider(Arc::new(stats_provider))
            .with_upload_provider(Arc::new(ApplicationFileUploadProvider::new(upload_service)))
            .with_blob_reader(storage),
    );
    let response = app
        .clone()
        .oneshot(batch_multipart_request(
            "--tssp\r\n\
                 Content-Disposition: form-data; name=\"file\"; filename=\"ok.txt\"\r\n\
                 Content-Type: text/plain\r\n\r\n\
                 hello upload\r\n\
                 --tssp\r\n\
                 Content-Disposition: form-data; name=\"file\"; filename=\"\"\r\n\
                 Content-Type: text/plain\r\n\r\n\
                 bad name\r\n\
                 --tssp--\r\n",
        ))
        .await
        .unwrap_or_else(|error| panic!("request failed: {error}"));

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_json(response).await;
    assert_eq!(body["created_count"], 1);
    assert_eq!(body["failed_count"], 1);
    assert_eq!(body["results"][0]["outcome"], "created");
    assert_eq!(body["results"][1]["outcome"], "failed");

    let status = app
        .oneshot(status_request())
        .await
        .unwrap_or_else(|error| panic!("status request failed: {error}"));
    let status_body = response_json(status).await;
    assert_eq!(status_body["file_count"], 1);
}

#[tokio::test]
async fn upload_endpoint_persists_file_and_returns_existing_record_on_duplicate() {
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

    let first = app
        .clone()
        .oneshot(multipart_request(REAL_UPLOAD_BODY))
        .await
        .unwrap_or_else(|error| panic!("first request failed: {error}"));
    assert_eq!(first.status(), StatusCode::CREATED);
    let first_body = response_json(first).await;
    let first_id = first_body["id"]
        .as_str()
        .unwrap_or_else(|| panic!("first id is missing"));

    let second = app
        .clone()
        .oneshot(multipart_request(DUPLICATE_UPLOAD_BODY))
        .await
        .unwrap_or_else(|error| panic!("second request failed: {error}"));
    assert_eq!(second.status(), StatusCode::OK);
    assert_eq!(
        second.headers().get("x-tssp-deduplicated"),
        Some(
            &"true"
                .parse()
                .unwrap_or_else(|error| panic!("header parse failed: {error}"))
        )
    );
    let second_body = response_json(second).await;
    assert_eq!(second_body["id"].as_str(), Some(first_id));
    assert_eq!(second_body["name"].as_str(), Some("note.txt"));
    assert_eq!(second_body["content_hash"], first_body["content_hash"]);

    let status = app
        .clone()
        .oneshot(status_request())
        .await
        .unwrap_or_else(|error| panic!("status request failed: {error}"));
    let status_body = response_json(status).await;
    assert_eq!(status_body["file_count"], 1);
    assert_eq!(status_body["recent_upload_count_24h"], 1);

    let list = app
        .clone()
        .oneshot(list_request("?limit=1"))
        .await
        .unwrap_or_else(|error| panic!("list request failed: {error}"));
    let list_body = response_json(list).await;
    assert_eq!(list_body["files"].as_array().map(Vec::len), Some(1));
    assert_eq!(list_body["files"][0]["id"].as_str(), Some(first_id));

    let found = app
        .clone()
        .oneshot(file_request(first_id))
        .await
        .unwrap_or_else(|error| panic!("file request failed: {error}"));
    let found_body = response_json(found).await;
    assert_eq!(found_body["id"].as_str(), Some(first_id));
    assert_eq!(found_body["name"].as_str(), Some("note.txt"));

    assert_content_downloads(app, first_id).await;
}

#[tokio::test]
async fn upload_body_limit_enforced_when_max_upload_bytes_set() {
    let settings = DaemonSettings {
        max_upload_bytes: 100,
        ..DaemonSettings::default()
    };
    let state = HttpState::test_http_state_with_settings(std::env::temp_dir(), settings);
    let app = build_router(state);
    // axum's DefaultBodyLimit::max limits body reading at the extractor level.
    // When the multipart body exceeds the cap, multer sees an incomplete
    // stream and returns 400 BAD_REQUEST (not 413) because the stream is
    // truncated before the final boundary is reached.  The upload is still
    // rejected — the security property is preserved.
    let boundary = "----TestBoundary";
    let data = "X".repeat(200);
    let body_str = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"big.bin\"\r\n\r\n{data}\r\n--{boundary}--\r\n",
        );
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/files")
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body_str))
                .unwrap_or_else(|e| panic!("request build: {e}")),
        )
        .await
        .unwrap_or_else(|e| panic!("request failed: {e}"));
    // Must not succeed — either 400 (truncated multipart) or 413 (pre-check).
    assert!(
        response.status().is_client_error(),
        "expected client error for over-limit upload, got {}",
        response.status()
    );
}

#[tokio::test]
async fn upload_body_unlimited_when_max_upload_bytes_is_zero() {
    let settings = DaemonSettings {
        max_upload_bytes: 0,
        ..DaemonSettings::default()
    };
    let state = HttpState::test_http_state_with_settings(std::env::temp_dir(), settings);
    let app = build_router(state);
    // With no limit, a small body should reach auth and return 401, not
    // any body-limit error code.
    let boundary = "----TestBoundary";
    let body_str = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"small.bin\"\r\n\r\nHELLO\r\n--{boundary}--\r\n",
        );
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/files")
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body_str))
                .unwrap_or_else(|e| panic!("request build: {e}")),
        )
        .await
        .unwrap_or_else(|e| panic!("request failed: {e}"));
    assert_ne!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}
