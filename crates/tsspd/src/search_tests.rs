//! Search endpoint integration tests.

use super::*;
use axum::body::to_bytes;
use axum::http::Request;
use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use tower::ServiceExt;
use tssp_ports::FileRepository;

fn build_test_router(provider: Arc<dyn FileSearchProvider>) -> Router {
    let mut state = HttpState::test_http_state(std::path::PathBuf::from("/tmp"));
    state = state.with_search_provider(provider);
    Router::new()
        .route("/search", get(search_files))
        .with_state(state)
}

#[tokio::test]
async fn search_returns_bad_request_on_empty_query() {
    let provider = Arc::new(StaticFileSearchProvider);
    let router = build_test_router(provider);

    let request = Request::builder()
        .uri("/search?q=")
        .body(axum::body::Body::empty())
        .unwrap_or_else(|error| panic!("request build failed: {error}"));

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn search_rejects_invalid_limit() {
    let provider = Arc::new(StaticFileSearchProvider);
    let router = build_test_router(provider);

    let request = Request::builder()
        .uri("/search?q=test&limit=0")
        .body(axum::body::Body::empty())
        .unwrap_or_else(|error| panic!("request build failed: {error}"));

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn search_returns_ok_on_valid_query() {
    let provider = Arc::new(StaticFileSearchProvider);
    let router = build_test_router(provider);

    let request = Request::builder()
        .uri("/search?q=test")
        .body(axum::body::Body::empty())
        .unwrap_or_else(|error| panic!("request build failed: {error}"));

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap_or_else(|error| panic!("body read failed: {error}"));
    let result: SearchResponse =
        serde_json::from_slice(&body).unwrap_or_else(|error| panic!("json parse failed: {error}"));
    assert_eq!(result.schema_version, 1);
    assert_eq!(result.limit, DEFAULT_SEARCH_LIMIT);
    assert!(result.results.is_empty());
}

#[test]
fn search_query_accepts_supported_filters() {
    let query = SearchQuery {
        q: "report".to_owned(),
        limit: Some(10),
        kind: Some("file".to_owned()),
        tag: Some("Finance".to_owned()),
        mime_prefix: Some("application/pdf".to_owned()),
        pinned: true,
        visibility: Some("public".to_owned()),
    };

    let filters = query
        .to_filters()
        .unwrap_or_else(|error| panic!("filters failed: {error}"));

    assert_eq!(filters.limit, 10);
    assert_eq!(filters.kind, SearchKind::File);
    assert!(filters.tag.is_some());
    assert_eq!(filters.mime_prefix.as_deref(), Some("application/pdf"));
    assert_eq!(filters.visibility, Some(Visibility::Public));
}

struct MockRepo;

impl FileRepository for MockRepo {
    fn insert_file(
        &self,
        _new_file: tssp_ports::NewFileRecord,
    ) -> Result<tssp_domain::FileRecord, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn find_file(
        &self,
        _id: &tssp_domain::FileId,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn find_file_by_content_hash(
        &self,
        _content_hash: &tssp_domain::ContentHash,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn delete_file(
        &self,
        _id: &tssp_domain::FileId,
    ) -> Result<Option<tssp_ports::DeletedFileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn restore_file(
        &self,
        _id: &tssp_domain::FileId,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn list_deleted_files(
        &self,
        _older_than: tssp_domain::UnixTimestamp,
    ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn purge_deleted_file(
        &self,
        _id: &tssp_domain::FileId,
    ) -> Result<bool, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn list_files(
        &self,
        _query: &tssp_ports::ListQuery,
    ) -> Result<tssp_ports::PagedFiles, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn list_files_recent(
        &self,
        _limit: u64,
    ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn list_files_by_tag(
        &self,
        _tag: &tssp_domain::TagKey,
        _limit: u64,
    ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn list_tags(&self) -> Result<Vec<tssp_ports::TagSummary>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn add_tags_to_file(
        &self,
        _id: &tssp_domain::FileId,
        _tags: &[tssp_domain::Tag],
    ) -> Result<tssp_ports::TagMutationOutcome, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn remove_tag_from_file(
        &self,
        _id: &tssp_domain::FileId,
        _tag: &tssp_domain::TagKey,
    ) -> Result<tssp_ports::TagMutationOutcome, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn stats_since(
        &self,
        _recent_since: tssp_domain::UnixTimestamp,
    ) -> Result<tssp_ports::RepositoryStats, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn pin_file(
        &self,
        _id: &tssp_domain::FileId,
        _position: Option<u32>,
    ) -> Result<tssp_ports::PinOutcome, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn unpin_file(
        &self,
        _id: &tssp_domain::FileId,
    ) -> Result<tssp_ports::PinOutcome, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn list_pinned_files(
        &self,
    ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn reorder_pins(
        &self,
        _ordered_ids: &[tssp_domain::FileId],
    ) -> Result<(), tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn search_files(
        &self,
        _query: &str,
    ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        Ok(vec![])
    }

    fn rename_file(
        &self,
        _id: &tssp_domain::FileId,
        _new_name: &tssp_domain::FileName,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }

    fn list_folder_counts(
        &self,
        _owner_id: Option<&tssp_domain::UserId>,
    ) -> Result<Vec<(String, u64)>, tssp_ports::RepositoryError> {
        Ok(Vec::new())
    }

    fn set_file_visibility(
        &self,
        _id: &tssp_domain::FileId,
        _visibility: tssp_domain::Visibility,
        _public_token: Option<&str>,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }

    fn find_file_by_public_token(
        &self,
        _token: &str,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }

    fn update_folder_path_prefix(
        &self,
        _from_prefix: &str,
        _to_prefix: &str,
    ) -> Result<u64, tssp_ports::RepositoryError> {
        unimplemented!()
    }

    fn set_file_folder_path(
        &self,
        _id: &tssp_domain::FileId,
        _folder_path: &str,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn insert_audit_event(
        &self,
        _: &str,
        _: i64,
        _: Option<&str>,
        _: &str,
        _: Option<&str>,
        _: Option<&str>,
        _: &str,
        _: Option<&str>,
    ) -> Result<(), tssp_ports::RepositoryError> {
        unimplemented!()
    }
}

impl NoteRepository for MockRepo {
    fn insert_note(
        &self,
        _new_note: tssp_ports::NewNoteRecord,
    ) -> Result<tssp_domain::NoteRecord, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn find_note(
        &self,
        _id: &tssp_domain::NoteId,
    ) -> Result<Option<tssp_domain::NoteRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn update_note(
        &self,
        _id: &tssp_domain::NoteId,
        _title: &tssp_domain::NoteTitle,
        _body: &tssp_domain::NoteBody,
        _updated_at: tssp_domain::UnixTimestamp,
    ) -> Result<tssp_domain::NoteRecord, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn delete_note(&self, _id: &tssp_domain::NoteId) -> Result<bool, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn list_notes(
        &self,
        _query: &tssp_ports::NoteListQuery,
    ) -> Result<tssp_ports::PagedNotes, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn add_tags_to_note(
        &self,
        _id: &tssp_domain::NoteId,
        _tags: &[tssp_domain::Tag],
    ) -> Result<tssp_ports::TagMutationOutcome, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn remove_tag_from_note(
        &self,
        _id: &tssp_domain::NoteId,
        _tag: &tssp_domain::TagKey,
    ) -> Result<tssp_ports::TagMutationOutcome, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn replace_tags_on_note(
        &self,
        _id: &tssp_domain::NoteId,
        _tags: &[tssp_domain::Tag],
    ) -> Result<(), tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn pin_note(
        &self,
        _id: &tssp_domain::NoteId,
        _position: Option<u32>,
    ) -> Result<tssp_ports::PinOutcome, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn unpin_note(
        &self,
        _id: &tssp_domain::NoteId,
    ) -> Result<tssp_ports::PinOutcome, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn search_notes(
        &self,
        _query: &str,
    ) -> Result<Vec<tssp_domain::NoteRecord>, tssp_ports::RepositoryError> {
        Ok(vec![])
    }
    fn search_all(
        &self,
        _query: &str,
    ) -> Result<Vec<tssp_ports::SearchHit>, tssp_ports::RepositoryError> {
        Ok(vec![])
    }
}

#[tokio::test]
async fn repository_search_provider_delegates_to_repo() {
    let provider = RepositoryFileSearchProvider::new(MockRepo);
    let result = provider
        .search("test")
        .unwrap_or_else(|error| panic!("search failed: {error}"));
    assert!(result.is_empty());
}

struct FailingMockRepo;

impl FileRepository for FailingMockRepo {
    fn insert_file(
        &self,
        _new_file: tssp_ports::NewFileRecord,
    ) -> Result<tssp_domain::FileRecord, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn find_file(
        &self,
        _id: &tssp_domain::FileId,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn find_file_by_content_hash(
        &self,
        _content_hash: &tssp_domain::ContentHash,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn delete_file(
        &self,
        _id: &tssp_domain::FileId,
    ) -> Result<Option<tssp_ports::DeletedFileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn restore_file(
        &self,
        _id: &tssp_domain::FileId,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn list_deleted_files(
        &self,
        _older_than: tssp_domain::UnixTimestamp,
    ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn purge_deleted_file(
        &self,
        _id: &tssp_domain::FileId,
    ) -> Result<bool, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn list_files(
        &self,
        _query: &tssp_ports::ListQuery,
    ) -> Result<tssp_ports::PagedFiles, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn list_files_recent(
        &self,
        _limit: u64,
    ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn list_files_by_tag(
        &self,
        _tag: &tssp_domain::TagKey,
        _limit: u64,
    ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn list_tags(&self) -> Result<Vec<tssp_ports::TagSummary>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn add_tags_to_file(
        &self,
        _id: &tssp_domain::FileId,
        _tags: &[tssp_domain::Tag],
    ) -> Result<tssp_ports::TagMutationOutcome, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn remove_tag_from_file(
        &self,
        _id: &tssp_domain::FileId,
        _tag: &tssp_domain::TagKey,
    ) -> Result<tssp_ports::TagMutationOutcome, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn stats_since(
        &self,
        _recent_since: tssp_domain::UnixTimestamp,
    ) -> Result<tssp_ports::RepositoryStats, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn pin_file(
        &self,
        _id: &tssp_domain::FileId,
        _position: Option<u32>,
    ) -> Result<tssp_ports::PinOutcome, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn unpin_file(
        &self,
        _id: &tssp_domain::FileId,
    ) -> Result<tssp_ports::PinOutcome, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn list_pinned_files(
        &self,
    ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn reorder_pins(
        &self,
        _ordered_ids: &[tssp_domain::FileId],
    ) -> Result<(), tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn search_files(
        &self,
        _query: &str,
    ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "db error".into(),
        })
    }

    fn rename_file(
        &self,
        _id: &tssp_domain::FileId,
        _new_name: &tssp_domain::FileName,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }

    fn list_folder_counts(
        &self,
        _owner_id: Option<&tssp_domain::UserId>,
    ) -> Result<Vec<(String, u64)>, tssp_ports::RepositoryError> {
        Ok(Vec::new())
    }

    fn set_file_visibility(
        &self,
        _id: &tssp_domain::FileId,
        _visibility: tssp_domain::Visibility,
        _public_token: Option<&str>,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }

    fn find_file_by_public_token(
        &self,
        _token: &str,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }

    fn update_folder_path_prefix(
        &self,
        _from_prefix: &str,
        _to_prefix: &str,
    ) -> Result<u64, tssp_ports::RepositoryError> {
        unimplemented!()
    }

    fn set_file_folder_path(
        &self,
        _id: &tssp_domain::FileId,
        _folder_path: &str,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn insert_audit_event(
        &self,
        _: &str,
        _: i64,
        _: Option<&str>,
        _: &str,
        _: Option<&str>,
        _: Option<&str>,
        _: &str,
        _: Option<&str>,
    ) -> Result<(), tssp_ports::RepositoryError> {
        unimplemented!()
    }
}

impl NoteRepository for FailingMockRepo {
    fn insert_note(
        &self,
        _new_note: tssp_ports::NewNoteRecord,
    ) -> Result<tssp_domain::NoteRecord, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn find_note(
        &self,
        _id: &tssp_domain::NoteId,
    ) -> Result<Option<tssp_domain::NoteRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn update_note(
        &self,
        _id: &tssp_domain::NoteId,
        _title: &tssp_domain::NoteTitle,
        _body: &tssp_domain::NoteBody,
        _updated_at: tssp_domain::UnixTimestamp,
    ) -> Result<tssp_domain::NoteRecord, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn delete_note(&self, _id: &tssp_domain::NoteId) -> Result<bool, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn list_notes(
        &self,
        _query: &tssp_ports::NoteListQuery,
    ) -> Result<tssp_ports::PagedNotes, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn add_tags_to_note(
        &self,
        _id: &tssp_domain::NoteId,
        _tags: &[tssp_domain::Tag],
    ) -> Result<tssp_ports::TagMutationOutcome, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn remove_tag_from_note(
        &self,
        _id: &tssp_domain::NoteId,
        _tag: &tssp_domain::TagKey,
    ) -> Result<tssp_ports::TagMutationOutcome, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn replace_tags_on_note(
        &self,
        _id: &tssp_domain::NoteId,
        _tags: &[tssp_domain::Tag],
    ) -> Result<(), tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn pin_note(
        &self,
        _id: &tssp_domain::NoteId,
        _position: Option<u32>,
    ) -> Result<tssp_ports::PinOutcome, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn unpin_note(
        &self,
        _id: &tssp_domain::NoteId,
    ) -> Result<tssp_ports::PinOutcome, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn search_notes(
        &self,
        _query: &str,
    ) -> Result<Vec<tssp_domain::NoteRecord>, tssp_ports::RepositoryError> {
        unimplemented!()
    }
    fn search_all(
        &self,
        _query: &str,
    ) -> Result<Vec<tssp_ports::SearchHit>, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "db error".into(),
        })
    }
}

#[tokio::test]
async fn search_returns_internal_error_on_failure() {
    let repo = FailingMockRepo;
    let provider = Arc::new(RepositoryFileSearchProvider::new(repo));
    let router = build_test_router(provider);

    let request = Request::builder()
        .uri("/search?q=test")
        .body(axum::body::Body::empty())
        .unwrap_or_else(|error| panic!("request build failed: {error}"));

    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
