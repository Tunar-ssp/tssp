//! Integration tests for note HTTP handlers.

use std::sync::Arc;
use std::time::Instant;

use axum::body::to_bytes;
use axum::http::{Request, StatusCode};
use axum::Router;
use tssp_adapter_sqlite::SqliteFileRepository;
use tssp_adapter_system::{SystemClock, UuidV7FileIdGenerator};
use tssp_app::NoteService;
use tower::ServiceExt;

use crate::notes::ApplicationNoteProvider;
use crate::{build_router, HttpState};

fn note_router(repository: SqliteFileRepository) -> Router {
    let repository = Arc::new(repository);
    let service = NoteService::new(repository, SystemClock, UuidV7FileIdGenerator);
    let provider = ApplicationNoteProvider::new(service);
    build_router(
        HttpState::new(Instant::now(), std::env::temp_dir())
            .with_note_provider(Arc::new(provider)),
    )
}

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap_or_else(|error| panic!("body read failed: {error}"));
    serde_json::from_slice(&body).unwrap_or_else(|error| panic!("json parse failed: {error}"))
}

#[tokio::test]
async fn create_note_returns_201_and_derives_title() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("open failed: {error}"));
    let app = note_router(repository);

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/notes")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            r##"{"body":"# Plan\n\nDo the thing","tags":["ideas"]}"##,
        ))
        .unwrap_or_else(|error| panic!("request failed: {error}"));

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response_json(response).await;
    assert_eq!(body["title"], "Plan");
    assert_eq!(body["body"], "# Plan\n\nDo the thing");
    assert!(body["tags"].as_array().is_some_and(|tags| !tags.is_empty()));
}

#[tokio::test]
async fn create_note_rejects_empty_body() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("open failed: {error}"));
    let app = note_router(repository);

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/notes")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"body":"   "}"#))
        .unwrap_or_else(|error| panic!("request failed: {error}"));

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_note_returns_not_found_for_missing_id() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("open failed: {error}"));
    let app = note_router(repository);

    let request = Request::builder()
        .uri("/api/v1/notes/does-not-exist")
        .body(axum::body::Body::empty())
        .unwrap_or_else(|error| panic!("request failed: {error}"));

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_note_is_idempotent_from_client_view() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("open failed: {error}"));
    let app = note_router(repository);

    let create = Request::builder()
        .method("POST")
        .uri("/api/v1/notes")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"body":"temp"}"#))
        .unwrap_or_else(|error| panic!("request failed: {error}"));
    let created = app.clone().oneshot(create).await.unwrap();
    let body = response_json(created).await;
    let id = body["id"].as_str().unwrap_or_else(|| panic!("missing id"));

    let delete = Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/notes/{id}"))
        .body(axum::body::Body::empty())
        .unwrap_or_else(|error| panic!("request failed: {error}"));
    assert_eq!(
        app.clone().oneshot(delete).await.unwrap().status(),
        StatusCode::NO_CONTENT
    );

    let get = Request::builder()
        .uri(format!("/api/v1/notes/{id}"))
        .body(axum::body::Body::empty())
        .unwrap_or_else(|error| panic!("request failed: {error}"));
    assert_eq!(
        app.oneshot(get).await.unwrap().status(),
        StatusCode::NOT_FOUND
    );
}
