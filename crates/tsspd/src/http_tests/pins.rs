#![allow(clippy::unwrap_used, clippy::unreadable_literal, clippy::needless_raw_string_hashes)]
//! `pins` route integration tests.

//! HTTP integration tests.

#![allow(unused_imports)]

use super::common::*;
use super::imports::*;

#[tokio::test]
async fn pin_endpoints_accept_bodyless_pin_and_support_reorder() {
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
    let pin_service = PinService::new(repository.clone());
    let app = build_router(
        HttpState::test_http_state(temp.path().join("http-upload-tmp"))
            .with_repository(repository)
            .with_stats_provider(Arc::new(stats_provider))
            .with_upload_provider(Arc::new(ApplicationFileUploadProvider::new(upload_service)))
            .with_pin_provider(Arc::new(ApplicationFilePinProvider::new(pin_service)))
            .with_blob_reader(storage),
    );

    let first_upload = app
        .clone()
        .oneshot(multipart_request(DUPLICATE_UPLOAD_BODY))
        .await
        .unwrap_or_else(|error| panic!("first upload request failed: {error}"));
    let first_id = response_json(first_upload).await["id"]
        .as_str()
        .unwrap_or_else(|| panic!("first uploaded id is missing"))
        .to_owned();

    let second_upload = app
        .clone()
        .oneshot(multipart_request(SECOND_UPLOAD_BODY))
        .await
        .unwrap_or_else(|error| panic!("second upload request failed: {error}"));
    let second_id = response_json(second_upload).await["id"]
        .as_str()
        .unwrap_or_else(|| panic!("second uploaded id is missing"))
        .to_owned();

    let url = format!("/api/v1/files/{}/pin", first_id);
    let mut req = Request::builder()
        .method("PUT")
        .uri(url)
        .body(Body::empty())
        .unwrap();
    let peer_addr: std::net::SocketAddr = "127.0.0.1:1234".parse().unwrap();
    req.extensions_mut()
        .insert(axum::extract::connect_info::ConnectInfo(peer_addr));
    let first_pin = app
        .clone()
        .oneshot(req)
        .await
        .unwrap_or_else(|error| panic!("pin request failed: {error}"));
    if first_pin.status() != StatusCode::OK {
        let status = first_pin.status();
        let body_bytes = axum::body::to_bytes(first_pin.into_body(), 10000).await.unwrap();
        let body = String::from_utf8_lossy(&body_bytes);
        panic!("pin request for id {first_id} (PUT /api/v1/files/{first_id}/pin) failed with status {status} and body: {body}. Ensure the router is configured correctly.");
    }

    let mut second_pin_req = pin_with_position_request(&second_id, r#"{"position":1}"#);
    let peer_addr: std::net::SocketAddr = "127.0.0.1:1234".parse().unwrap();
    second_pin_req
        .extensions_mut()
        .insert(axum::extract::connect_info::ConnectInfo(peer_addr));
    let second_pin = app
        .clone()
        .oneshot(second_pin_req)
        .await
        .unwrap_or_else(|error| panic!("pin with position request failed: {error}"));
    assert_eq!(second_pin.status(), StatusCode::OK);
    let second_pin_body = response_json(second_pin).await;
    assert_eq!(second_pin_body["changed"], true);

    let mut list_req = pins_request();
    let peer_addr: std::net::SocketAddr = "127.0.0.1:1234".parse().unwrap();
    list_req
        .extensions_mut()
        .insert(axum::extract::connect_info::ConnectInfo(peer_addr));
    let listed = app
        .clone()
        .oneshot(list_req)
        .await
        .unwrap_or_else(|error| panic!("pins list request failed: {error}"));
    assert_eq!(listed.status(), StatusCode::OK);
    let listed_body = response_json(listed).await;
    assert_eq!(listed_body["files"][0]["id"], second_id);
    assert_eq!(listed_body["files"][1]["id"], first_id);

    let mut reorder_req = reorder_pins_request(&format!(
        r#"{{"ids":["{first_id}","{second_id}"]}}"#
    ));
    let peer_addr: std::net::SocketAddr = "127.0.0.1:1234".parse().unwrap();
    reorder_req
        .extensions_mut()
        .insert(axum::extract::connect_info::ConnectInfo(peer_addr));
    let reordered = app
        .clone()
        .oneshot(reorder_req)
        .await
        .unwrap_or_else(|error| panic!("pins reorder request failed: {error}"));
    assert_eq!(reordered.status(), StatusCode::OK);

    let mut list_after_reorder_req = pins_request();
    let peer_addr: std::net::SocketAddr = "127.0.0.1:1234".parse().unwrap();
    list_after_reorder_req
        .extensions_mut()
        .insert(axum::extract::connect_info::ConnectInfo(peer_addr));
    let listed_after_reorder = app
        .clone()
        .oneshot(list_after_reorder_req)
        .await
        .unwrap_or_else(|error| panic!("pins list after reorder request failed: {error}"));
    let reordered_body = response_json(listed_after_reorder).await;
    assert_eq!(reordered_body["files"][0]["id"], first_id);
    assert_eq!(reordered_body["files"][1]["id"], second_id);

    let mut unpin_req = unpin_request(&first_id);
    let peer_addr: std::net::SocketAddr = "127.0.0.1:1234".parse().unwrap();
    unpin_req
        .extensions_mut()
        .insert(axum::extract::connect_info::ConnectInfo(peer_addr));
    let unpinned = app
        .oneshot(unpin_req)
        .await
        .unwrap_or_else(|error| panic!("unpin request failed: {error}"));
    assert_eq!(unpinned.status(), StatusCode::OK);
    let unpinned_body = response_json(unpinned).await;
    assert_eq!(unpinned_body["changed"], true);
}
