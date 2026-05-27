//! Integration tests for note HTTP handlers.

use std::sync::Arc;

use axum::body::to_bytes;
use axum::http::{Request, StatusCode};
use axum::Router;
use tower::ServiceExt;
use tssp_adapter_sqlite::SqliteFileRepository;
use tssp_adapter_system::{SystemClock, UuidV7FileIdGenerator};
use tssp_app::NoteService;

use crate::notes::ApplicationNoteProvider;
use crate::{build_router, HttpState};

fn note_router(repository: SqliteFileRepository) -> Router {
    let repository = Arc::new(repository);
    let service = NoteService::new(repository, SystemClock, UuidV7FileIdGenerator);
    let provider = ApplicationNoteProvider::new(service);
    build_router(
        HttpState::test_http_state(std::env::temp_dir()).with_note_provider(Arc::new(provider)),
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
async fn create_note_allows_empty_body() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("open failed: {error}"));
    let app = note_router(repository);

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/notes")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"title":"Untitled","body":""}"#))
        .unwrap_or_else(|error| panic!("request failed: {error}"));

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
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

#[tokio::test]
async fn duplicate_note_creates_a_new_record_with_same_body_and_tags() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("open failed: {error}"));
    let app = note_router(repository);

    let create = Request::builder()
        .method("POST")
        .uri("/api/v1/notes")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            r##"{"title":"Weekly plan","body":"# Weekly\n\nDo the thing","tags":["ideas","ops"],"pin":true}"##,
        ))
        .unwrap_or_else(|error| panic!("request failed: {error}"));
    let created = app.clone().oneshot(create).await.unwrap();
    let created_body = response_json(created).await;
    let id = created_body["id"]
        .as_str()
        .unwrap_or_else(|| panic!("missing id"))
        .to_owned();

    let duplicate = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/notes/{id}/duplicate"))
        .body(axum::body::Body::empty())
        .unwrap_or_else(|error| panic!("request failed: {error}"));
    let duplicated = app.clone().oneshot(duplicate).await.unwrap();
    assert_eq!(duplicated.status(), StatusCode::CREATED);

    let duplicated_body = response_json(duplicated).await;
    assert_ne!(duplicated_body["id"], created_body["id"]);
    assert_eq!(duplicated_body["body"], created_body["body"]);
    assert_eq!(duplicated_body["tags"], created_body["tags"]);
    assert_eq!(duplicated_body["title"], "Weekly plan copy");
    assert!(duplicated_body["pinned_at"].is_null());
}

#[tokio::test]
async fn update_note_with_empty_body_allowed() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("open failed: {error}"));
    let app = note_router(repository);

    let create = Request::builder()
        .method("POST")
        .uri("/api/v1/notes")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"body":"test"}"#))
        .unwrap_or_else(|error| panic!("request failed: {error}"));
    let created = app.clone().oneshot(create).await.unwrap();
    let created_body = response_json(created).await;
    let id = created_body["id"].as_str().unwrap();

    let update = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/notes/{id}"))
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"body":""}"#))
        .unwrap_or_else(|error| panic!("request failed: {error}"));

    let response = app.oneshot(update).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let updated = response_json(response).await;
    assert_eq!(updated["body"], "");
}

#[tokio::test]
async fn create_note_with_many_tags() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("open failed: {error}"));
    let app = note_router(repository);

    let tags: Vec<String> = (0..20).map(|i| format!("tag{i}")).collect();
    let tags_json = serde_json::to_string(&tags).unwrap();
    let body = format!(r#"{{"body":"content","tags":{tags_json}}}"#);

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/notes")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(body))
        .unwrap_or_else(|error| panic!("request failed: {error}"));

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response_json(response).await;
    assert_eq!(body["tags"].as_array().unwrap().len(), 20);
}

#[tokio::test]
async fn delete_note_already_deleted_returns_not_found() {
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
    let id = body["id"].as_str().unwrap().to_owned();

    // Delete once
    let delete1 = Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/notes/{id}"))
        .body(axum::body::Body::empty())
        .unwrap_or_else(|error| panic!("request failed: {error}"));
    assert_eq!(
        app.clone().oneshot(delete1).await.unwrap().status(),
        StatusCode::NO_CONTENT
    );

    // Delete again - should be 404
    let delete2 = Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/notes/{id}"))
        .body(axum::body::Body::empty())
        .unwrap_or_else(|error| panic!("request failed: {error}"));
    assert_eq!(
        app.oneshot(delete2).await.unwrap().status(),
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn update_note_only_title() {
    let repository = SqliteFileRepository::open_in_memory()
        .unwrap_or_else(|error| panic!("open failed: {error}"));
    let app = note_router(repository);

    let create = Request::builder()
        .method("POST")
        .uri("/api/v1/notes")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"body":"original body"}"#))
        .unwrap_or_else(|error| panic!("request failed: {error}"));
    let created = app.clone().oneshot(create).await.unwrap();
    let created_body = response_json(created).await;
    let id = created_body["id"].as_str().unwrap();
    let original_body = created_body["body"].as_str().unwrap();

    let update = Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/notes/{id}"))
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"title":"New Title"}"#))
        .unwrap_or_else(|error| panic!("request failed: {error}"));

    let response = app.oneshot(update).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let updated_body = response_json(response).await;
    assert_eq!(updated_body["title"], "New Title");
    assert_eq!(updated_body["body"], original_body);
}
