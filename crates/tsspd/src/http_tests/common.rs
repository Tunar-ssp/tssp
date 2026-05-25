//! Shared test infrastructure for HTTP integration tests.
//!
//! Contains mock providers, request builders, response helpers, and test data
//! builders used across the focused test modules.

use std::io::Read;
use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::header::{CONTENT_RANGE, CONTENT_TYPE};
use axum::http::{Request, StatusCode};
use tempfile::tempdir;
use tower::ServiceExt;
use tssp_adapter_fs::FilesystemBlobStore;
use tssp_adapter_sqlite::SqliteFileRepository;
use tssp_adapter_system::{SystemClock, UuidV7FileIdGenerator};
use tssp_app::{DeleteFileService, PinService, TagService, UploadService};
use tssp_domain::{
    ContentHash, FileId, FileName, FileRecord, FileSize, MimeType, StorageHandle, Tag,
    UnixTimestamp,
};
use tssp_ports::RepositoryStats;

use crate::{
    build_router, ApplicationFileDeleteProvider, ApplicationFilePinProvider,
    ApplicationFileTagProvider, ApplicationFileUploadProvider, FileUploadProvider, HttpState,
    HttpUploadError, HttpUploadOutcome, HttpUploadRequest, MetadataStatsProvider,
    RepositoryMetadataStatsProvider,
};

pub use axum::Router;

// ── Multipart bodies ─────────────────────────────────────────────────────────

pub const REAL_UPLOAD_BODY: &str = "--tssp\r\n\
        Content-Disposition: form-data; name=\"tag\"\r\n\r\n\
        Docs\r\n\
        --tssp\r\n\
        Content-Disposition: form-data; name=\"pin\"\r\n\r\n\
        TRUE\r\n\
        --tssp\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"note.txt\"\r\n\
        Content-Type: text/plain\r\n\r\n\
        hello upload\r\n\
        --tssp--\r\n";

pub const DUPLICATE_UPLOAD_BODY: &str = "--tssp\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"other.txt\"\r\n\
        Content-Type: text/plain\r\n\r\n\
        hello upload\r\n\
        --tssp--\r\n";

pub const SECOND_UPLOAD_BODY: &str = "--tssp\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"later.txt\"\r\n\
        Content-Type: text/plain\r\n\r\n\
        hello second\r\n\
        --tssp--\r\n";

// ── App builders ─────────────────────────────────────────────────────────────

/// Builds a fully-wired test app with real `SQLite` + filesystem storage.
#[allow(dead_code)]
pub fn real_storage_app() -> (tempfile::TempDir, Router) {
    let temp = tempdir().unwrap_or_else(|e| panic!("tempdir failed: {e}"));
    let repository = Arc::new(
        SqliteFileRepository::open(temp.path().join("metadata.sqlite3"))
            .unwrap_or_else(|e| panic!("repository open failed: {e}")),
    );
    let storage = Arc::new(
        FilesystemBlobStore::new(temp.path().join("storage"))
            .unwrap_or_else(|e| panic!("blob store open failed: {e}")),
    );
    let stats_provider = RepositoryMetadataStatsProvider::new(repository.clone(), SystemClock);
    let upload_service = UploadService::new(
        storage.clone(),
        repository.clone(),
        UuidV7FileIdGenerator,
        SystemClock,
    );
    let delete_service = DeleteFileService::new(storage.clone(), repository.clone());
    let pin_service = PinService::new(repository.clone());
    let tag_service = TagService::new(repository.clone());
    let app = build_router(
        HttpState::test_http_state(temp.path().join("http-upload-tmp"))
            .with_repository(repository.clone())
            .with_stats_provider(Arc::new(stats_provider))
            .with_upload_provider(Arc::new(ApplicationFileUploadProvider::new(upload_service)))
            .with_delete_provider(Arc::new(ApplicationFileDeleteProvider::new(delete_service)))
            .with_tag_provider(Arc::new(ApplicationFileTagProvider::new(tag_service)))
            .with_pin_provider(Arc::new(ApplicationFilePinProvider::new(pin_service)))
            .with_blob_reader(storage),
    );
    (temp, app)
}

// ── Request builders ─────────────────────────────────────────────────────────

pub fn multipart_request(body: &'static str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/v1/files")
        .header(CONTENT_TYPE, "multipart/form-data; boundary=tssp")
        .body(Body::from(body))
        .unwrap_or_else(|e| panic!("request build failed: {e}"))
}

pub fn batch_multipart_request(body: &'static str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/v1/files/batch")
        .header(CONTENT_TYPE, "multipart/form-data; boundary=tssp")
        .body(Body::from(body))
        .unwrap_or_else(|e| panic!("request build failed: {e}"))
}

pub fn status_request() -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri("/api/v1/status")
        .body(Body::empty())
        .unwrap_or_else(|e| panic!("request build failed: {e}"))
}

pub fn list_request(query: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(format!("/api/v1/files{query}"))
        .body(Body::empty())
        .unwrap_or_else(|e| panic!("request build failed: {e}"))
}

pub fn file_request(id: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(format!("/api/v1/files/{id}"))
        .body(Body::empty())
        .unwrap_or_else(|e| panic!("request build failed: {e}"))
}

pub fn content_request(id: &str, range: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method("GET")
        .uri(format!("/api/v1/files/{id}/content?disposition=inline"));
    if let Some(range) = range {
        builder = builder.header("range", range);
    }
    builder
        .body(Body::empty())
        .unwrap_or_else(|e| panic!("request build failed: {e}"))
}

pub fn delete_request(id: &str) -> Request<Body> {
    Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/files/{id}"))
        .body(Body::empty())
        .unwrap_or_else(|e| panic!("request build failed: {e}"))
}

pub fn tags_request() -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri("/api/v1/tags")
        .body(Body::empty())
        .unwrap_or_else(|e| panic!("request build failed: {e}"))
}

pub fn add_tags_request(id: &str, body: &'static str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(format!("/api/v1/files/{id}/tags"))
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap_or_else(|e| panic!("request build failed: {e}"))
}

pub fn remove_tag_request(id: &str, tag: &str) -> Request<Body> {
    Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/files/{id}/tags/{tag}"))
        .body(Body::empty())
        .unwrap_or_else(|e| panic!("request build failed: {e}"))
}

#[allow(dead_code)]
pub fn pin_request(id: &str) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/files/{id}/pin"))
        .body(Body::empty())
        .unwrap_or_else(|e| panic!("request build failed: {e}"))
}

pub fn pin_with_position_request(id: &str, body: &str) -> Request<Body> {
    Request::builder()
        .method("PUT")
        .uri(format!("/api/v1/files/{id}/pin"))
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap_or_else(|e| panic!("request build failed: {e}"))
}

pub fn unpin_request(id: &str) -> Request<Body> {
    Request::builder()
        .method("DELETE")
        .uri(format!("/api/v1/files/{id}/pin"))
        .body(Body::empty())
        .unwrap_or_else(|e| panic!("request build failed: {e}"))
}

pub fn pins_request() -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri("/api/v1/pins")
        .body(Body::empty())
        .unwrap_or_else(|e| panic!("request build failed: {e}"))
}

pub fn reorder_pins_request(body: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/v1/pins/reorder")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_owned()))
        .unwrap_or_else(|e| panic!("request build failed: {e}"))
}

// ── Response helpers ─────────────────────────────────────────────────────────

/// Asserts inline content download, byte ranges, and invalid range handling.
pub async fn assert_content_downloads(app: Router, first_id: &str) {
    let content = app
        .clone()
        .oneshot(content_request(first_id, None))
        .await
        .unwrap_or_else(|error| panic!("content request failed: {error}"));
    assert_eq!(content.status(), StatusCode::OK);
    assert_eq!(
        content
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/plain")
    );
    assert_eq!(
        content
            .headers()
            .get("accept-ranges")
            .and_then(|value| value.to_str().ok()),
        Some("bytes")
    );
    let content_body = to_bytes(content.into_body(), 1024)
        .await
        .unwrap_or_else(|error| panic!("body read failed: {error}"));
    assert_eq!(content_body.as_ref(), b"hello upload");

    let range = app
        .clone()
        .oneshot(content_request(first_id, Some("bytes=6-11")))
        .await
        .unwrap_or_else(|error| panic!("range request failed: {error}"));
    assert_eq!(range.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(
        range
            .headers()
            .get(CONTENT_RANGE)
            .and_then(|value| value.to_str().ok()),
        Some("bytes 6-11/12")
    );
    let range_body = to_bytes(range.into_body(), 1024)
        .await
        .unwrap_or_else(|error| panic!("range body read failed: {error}"));
    assert_eq!(range_body.as_ref(), b"upload");

    let invalid_range = app
        .oneshot(content_request(first_id, Some("bytes=50-60")))
        .await
        .unwrap_or_else(|error| panic!("invalid range request failed: {error}"));
    assert_eq!(invalid_range.status(), StatusCode::RANGE_NOT_SATISFIABLE);
    assert_eq!(
        invalid_range
            .headers()
            .get(CONTENT_RANGE)
            .and_then(|value| value.to_str().ok()),
        Some("bytes */12")
    );
}

pub async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), 4096)
        .await
        .unwrap_or_else(|e| panic!("body read failed: {e}"));
    serde_json::from_slice(&body).unwrap_or_else(|e| panic!("json parse failed: {e}"))
}

#[allow(dead_code)]
pub async fn response_status_ok(app: Router, request: Request<Body>) -> StatusCode {
    tower::ServiceExt::oneshot(app, request)
        .await
        .unwrap_or_else(|e| panic!("request failed: {e}"))
        .status()
}

// ── Test data builders ────────────────────────────────────────────────────────

pub fn test_record(request: &HttpUploadRequest) -> FileRecord {
    FileRecord {
        id: file_id("file-test"),
        name: filename(&request.filename),
        size: FileSize::new(12),
        content_hash: content_hash(),
        mime_type: mime_type(
            request
                .mime_type
                .as_deref()
                .unwrap_or("application/octet-stream"),
        ),
        storage_handle: storage_handle(),
        uploaded_at: timestamp(1_700_000_000),
        tags: request.tags.iter().map(|t| tag_value(t)).collect(),
        pinned_at: request.pinned.then_some(1),
        folder_path: String::new(),
        owner_id: None,
        visibility: tssp_domain::Visibility::Private,
        public_token: None,
            public_expires_at: None,
    }
}

pub fn single_record() -> FileRecord {
    FileRecord {
        id: file_id("file-test"),
        name: filename("note.txt"),
        size: FileSize::new(12),
        content_hash: content_hash(),
        mime_type: mime_type("text/plain"),
        storage_handle: storage_handle(),
        uploaded_at: timestamp(1_700_000_000),
        tags: Vec::new(),
        pinned_at: None,
        folder_path: String::new(),
        owner_id: None,
        visibility: tssp_domain::Visibility::Private,
        public_token: None,
            public_expires_at: None,
    }
}

pub fn file_id(value: &str) -> FileId {
    FileId::new(value).unwrap_or_else(|e| panic!("invalid file id: {e}"))
}

pub fn filename(value: &str) -> FileName {
    FileName::new(value).unwrap_or_else(|e| panic!("invalid filename: {e}"))
}

pub fn content_hash() -> ContentHash {
    ContentHash::new("abcdefabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123")
        .unwrap_or_else(|e| panic!("invalid hash: {e}"))
}

pub fn mime_type(value: &str) -> MimeType {
    MimeType::new(value).unwrap_or_else(|e| panic!("invalid mime type: {e}"))
}

pub fn storage_handle() -> StorageHandle {
    StorageHandle::new("blobs/ab/cd/abcdef")
        .unwrap_or_else(|e| panic!("invalid storage handle: {e}"))
}

pub fn timestamp(seconds: i64) -> UnixTimestamp {
    UnixTimestamp::new(seconds).unwrap_or_else(|e| panic!("invalid timestamp: {e}"))
}

pub fn tag_value(value: &str) -> Tag {
    Tag::new(value).unwrap_or_else(|e| panic!("invalid tag: {e}"))
}

// ── Mock providers ────────────────────────────────────────────────────────────

pub struct FixedStatsProvider;

impl MetadataStatsProvider for FixedStatsProvider {
    fn stats(&self) -> Result<RepositoryStats, String> {
        Ok(RepositoryStats {
            file_count: 7,
            note_count: 0,
            tag_count: 3,
            pinned_count: 2,
            recent_upload_count: 1,
            recent_note_count: 0,
            storage_bytes_used: 0,
        })
    }

    fn list_files(&self, _query: &tssp_ports::ListQuery) -> Result<tssp_ports::PagedFiles, String> {
        Ok(tssp_ports::PagedFiles {
            files: Vec::new(),
            next_cursor: None,
        })
    }

    fn list_files_recent(&self, _limit: u64) -> Result<Vec<tssp_domain::FileRecord>, String> {
        Ok(Vec::new())
    }

    fn find_file(
        &self,
        _id: &tssp_domain::FileId,
    ) -> Result<Option<tssp_domain::FileRecord>, String> {
        Ok(None)
    }

    fn list_files_by_tag(
        &self,
        _tag: &tssp_domain::TagKey,
        _limit: u64,
    ) -> Result<Vec<tssp_domain::FileRecord>, String> {
        Ok(Vec::new())
    }

    fn rename_file(
        &self,
        _id: &tssp_domain::FileId,
        _new_name: &tssp_domain::FileName,
    ) -> Result<Option<tssp_domain::FileRecord>, String> {
        Ok(None)
    }

    fn list_folder_counts(
        &self,
        _owner_id: Option<&tssp_domain::UserId>,
    ) -> Result<Vec<(String, u64)>, String> {
        Ok(Vec::new())
    }

    fn set_file_visibility(
        &self,
        _: &tssp_domain::FileId,
        _: tssp_domain::Visibility,
        _: Option<&str>,
    ) -> Result<Option<tssp_domain::FileRecord>, String> {
        Ok(None)
    }

    fn find_file_by_public_token(
        &self,
        _: &str,
    ) -> Result<Option<tssp_domain::FileRecord>, String> {
        Ok(None)
    }

    fn update_folder_path_prefix(&self, _: &str, _: &str) -> Result<u64, String> {
        Ok(0)
    }

    fn set_file_folder_path(
        &self,
        _: &tssp_domain::FileId,
        _: &str,
    ) -> Result<Option<tssp_domain::FileRecord>, String> {
        Ok(None)
    }
}

pub struct FailingStatsProvider;

impl MetadataStatsProvider for FailingStatsProvider {
    fn stats(&self) -> Result<RepositoryStats, String> {
        Err("metadata database is unavailable".to_owned())
    }

    fn list_files(&self, _query: &tssp_ports::ListQuery) -> Result<tssp_ports::PagedFiles, String> {
        Err("metadata database is unavailable".to_owned())
    }

    fn list_files_recent(&self, _limit: u64) -> Result<Vec<tssp_domain::FileRecord>, String> {
        Err("metadata database is unavailable".to_owned())
    }

    fn find_file(
        &self,
        _id: &tssp_domain::FileId,
    ) -> Result<Option<tssp_domain::FileRecord>, String> {
        Err("metadata database is unavailable".to_owned())
    }

    fn list_files_by_tag(
        &self,
        _tag: &tssp_domain::TagKey,
        _limit: u64,
    ) -> Result<Vec<tssp_domain::FileRecord>, String> {
        Err("metadata database is unavailable".to_owned())
    }

    fn rename_file(
        &self,
        _id: &tssp_domain::FileId,
        _new_name: &tssp_domain::FileName,
    ) -> Result<Option<tssp_domain::FileRecord>, String> {
        Err("metadata database is unavailable".to_owned())
    }

    fn list_folder_counts(
        &self,
        _owner_id: Option<&tssp_domain::UserId>,
    ) -> Result<Vec<(String, u64)>, String> {
        Err("metadata database is unavailable".to_owned())
    }

    fn set_file_visibility(
        &self,
        _: &tssp_domain::FileId,
        _: tssp_domain::Visibility,
        _: Option<&str>,
    ) -> Result<Option<tssp_domain::FileRecord>, String> {
        Err("metadata database is unavailable".to_owned())
    }

    fn find_file_by_public_token(
        &self,
        _: &str,
    ) -> Result<Option<tssp_domain::FileRecord>, String> {
        Err("metadata database is unavailable".to_owned())
    }

    fn update_folder_path_prefix(&self, _: &str, _: &str) -> Result<u64, String> {
        Err("metadata database is unavailable".to_owned())
    }

    fn set_file_folder_path(
        &self,
        _: &tssp_domain::FileId,
        _: &str,
    ) -> Result<Option<tssp_domain::FileRecord>, String> {
        Err("metadata database is unavailable".to_owned())
    }
}

pub struct SingleRecordStatsProvider;

impl MetadataStatsProvider for SingleRecordStatsProvider {
    fn stats(&self) -> Result<RepositoryStats, String> {
        Ok(RepositoryStats {
            file_count: 1,
            note_count: 0,
            tag_count: 0,
            pinned_count: 0,
            recent_upload_count: 0,
            recent_note_count: 0,
            storage_bytes_used: 0,
        })
    }

    fn list_files(&self, _query: &tssp_ports::ListQuery) -> Result<tssp_ports::PagedFiles, String> {
        Ok(tssp_ports::PagedFiles {
            files: vec![single_record()],
            next_cursor: None,
        })
    }

    fn list_files_recent(&self, _limit: u64) -> Result<Vec<tssp_domain::FileRecord>, String> {
        Ok(vec![single_record()])
    }

    fn find_file(
        &self,
        _id: &tssp_domain::FileId,
    ) -> Result<Option<tssp_domain::FileRecord>, String> {
        Ok(Some(single_record()))
    }

    fn list_files_by_tag(
        &self,
        _tag: &tssp_domain::TagKey,
        _limit: u64,
    ) -> Result<Vec<tssp_domain::FileRecord>, String> {
        Ok(vec![single_record()])
    }

    fn rename_file(
        &self,
        _id: &tssp_domain::FileId,
        _new_name: &tssp_domain::FileName,
    ) -> Result<Option<tssp_domain::FileRecord>, String> {
        Ok(Some(single_record()))
    }

    fn list_folder_counts(
        &self,
        _owner_id: Option<&tssp_domain::UserId>,
    ) -> Result<Vec<(String, u64)>, String> {
        Ok(vec![(String::new(), 1)])
    }

    fn set_file_visibility(
        &self,
        id: &tssp_domain::FileId,
        _: tssp_domain::Visibility,
        _: Option<&str>,
    ) -> Result<Option<tssp_domain::FileRecord>, String> {
        self.find_file(id)
    }

    fn find_file_by_public_token(
        &self,
        _: &str,
    ) -> Result<Option<tssp_domain::FileRecord>, String> {
        Ok(None)
    }

    fn update_folder_path_prefix(&self, _: &str, _: &str) -> Result<u64, String> {
        Ok(1)
    }

    fn set_file_folder_path(
        &self,
        id: &tssp_domain::FileId,
        _: &str,
    ) -> Result<Option<tssp_domain::FileRecord>, String> {
        self.find_file(id)
    }
}

pub struct EchoUploadProvider {
    pub deduplicated: bool,
}

impl FileUploadProvider for EchoUploadProvider {
    fn upload(&self, mut request: HttpUploadRequest) -> Result<HttpUploadOutcome, HttpUploadError> {
        let mut bytes = Vec::new();
        request
            .source
            .read_to_end(&mut bytes)
            .map_err(|e| HttpUploadError::Internal {
                message: e.to_string(),
            })?;
        if bytes != b"hello upload" {
            return Err(HttpUploadError::InvalidRequest {
                message: "unexpected test upload bytes".to_owned(),
            });
        }
        Ok(HttpUploadOutcome {
            record: test_record(&request),
            deduplicated: self.deduplicated,
        })
    }
}

pub struct BatchEchoUploadProvider;

impl FileUploadProvider for BatchEchoUploadProvider {
    fn upload(&self, mut request: HttpUploadRequest) -> Result<HttpUploadOutcome, HttpUploadError> {
        let mut bytes = Vec::new();
        request
            .source
            .read_to_end(&mut bytes)
            .map_err(|e| HttpUploadError::Internal {
                message: e.to_string(),
            })?;
        if bytes != b"hello upload" {
            return Err(HttpUploadError::InvalidRequest {
                message: "unexpected test upload bytes".to_owned(),
            });
        }
        if request.filename == "broken.txt" {
            return Err(HttpUploadError::InvalidRequest {
                message: "broken test upload".to_owned(),
            });
        }
        Ok(HttpUploadOutcome {
            record: test_record(&request),
            deduplicated: request.filename == "duplicate.txt",
        })
    }
}
