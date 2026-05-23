//! HTTP daemon foundation for `tsspd`.
//!
//! The current server exposes lifecycle, status, upload, and web shell routes.
//! HTTP handlers stay thin and delegate storage behavior to application
//! services.

mod config;
mod content;
mod delete;
mod file;
mod list;
mod pins;
mod rename;
mod search;
mod sessions;
mod startup;
mod status;
mod tags;
mod upload;
mod web;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::Router;
use serde::Serialize;
use tssp_ports::BlobReader;

pub use config::{bind_error_message, DaemonConfig};
pub use delete::{
    ApplicationFileDeleteProvider, FileDeleteProvider, HttpDeleteError, HttpDeleteOutcome,
};
pub use pins::{ApplicationFilePinProvider, FilePinProvider, HttpPinError, HttpPinMutation};
pub use search::{FileSearchProvider, RepositoryFileSearchProvider};
pub use sessions::{ApplicationSessionProvider, SessionProvider, SessionResponse};
pub use startup::StartupService;
pub use status::{MetadataStatsProvider, RepositoryMetadataStatsProvider, StatusResponse};
pub use tags::{ApplicationFileTagProvider, FileTagProvider, HttpTagError, HttpTagMutation};
pub use upload::{
    ApplicationFileUploadProvider, FileRecordResponse, FileUploadProvider, HttpUploadError,
    HttpUploadOutcome, HttpUploadRequest,
};

use content::StaticBlobReader;
use delete::StaticFileDeleteProvider;
use pins::StaticFilePinProvider;
use search::StaticFileSearchProvider;
use sessions::StaticSessionProvider;
use status::StaticMetadataStatsProvider;
use tags::StaticFileTagProvider;
use upload::StaticFileUploadProvider;

/// Shared HTTP state.
#[derive(Clone)]
pub struct HttpState {
    started_at: Instant,
    stats_provider: Arc<dyn MetadataStatsProvider>,
    upload_provider: Arc<dyn FileUploadProvider>,
    delete_provider: Arc<dyn FileDeleteProvider>,
    tag_provider: Arc<dyn FileTagProvider>,
    pin_provider: Arc<dyn FilePinProvider>,
    search_provider: Arc<dyn search::FileSearchProvider>,
    session_provider: Arc<dyn SessionProvider>,
    blob_reader: Arc<dyn BlobReader + Send + Sync>,
    upload_temp_dir: PathBuf,
    storage_mutation_lock: Arc<tokio::sync::Mutex<()>>,
}

impl HttpState {
    /// Creates a base HTTP state with static/placeholder providers.
    #[must_use]
    pub fn new(started_at: Instant, upload_temp_dir: PathBuf) -> Self {
        Self {
            started_at,
            stats_provider: Arc::new(StaticMetadataStatsProvider),
            upload_provider: Arc::new(StaticFileUploadProvider),
            delete_provider: Arc::new(StaticFileDeleteProvider),
            tag_provider: Arc::new(StaticFileTagProvider),
            pin_provider: Arc::new(StaticFilePinProvider),
            search_provider: Arc::new(StaticFileSearchProvider),
            session_provider: Arc::new(StaticSessionProvider),
            blob_reader: Arc::new(StaticBlobReader),
            upload_temp_dir,
            storage_mutation_lock: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    /// Sets the metadata stats provider.
    #[must_use]
    pub fn with_stats_provider(mut self, provider: Arc<dyn MetadataStatsProvider>) -> Self {
        self.stats_provider = provider;
        self
    }

    /// Sets the file upload provider.
    #[must_use]
    pub fn with_upload_provider(mut self, provider: Arc<dyn FileUploadProvider>) -> Self {
        self.upload_provider = provider;
        self
    }

    /// Sets the file delete provider.
    #[must_use]
    pub fn with_delete_provider(mut self, provider: Arc<dyn FileDeleteProvider>) -> Self {
        self.delete_provider = provider;
        self
    }

    /// Sets the file tag provider.
    #[must_use]
    pub fn with_tag_provider(mut self, provider: Arc<dyn FileTagProvider>) -> Self {
        self.tag_provider = provider;
        self
    }

    /// Sets the file pin provider.
    #[must_use]
    pub fn with_pin_provider(mut self, provider: Arc<dyn FilePinProvider>) -> Self {
        self.pin_provider = provider;
        self
    }

    /// Sets the blob reader provider.
    #[must_use]
    pub fn with_blob_reader(mut self, reader: Arc<dyn BlobReader + Send + Sync>) -> Self {
        self.blob_reader = reader;
        self
    }

    /// Sets the search provider.
    #[must_use]
    pub fn with_search_provider(mut self, provider: Arc<dyn search::FileSearchProvider>) -> Self {
        self.search_provider = provider;
        self
    }

    /// Sets the session provider.
    #[must_use]
    pub fn with_session_provider(mut self, provider: Arc<dyn SessionProvider>) -> Self {
        self.session_provider = provider;
        self
    }
}

async fn options_response() -> (axum::http::StatusCode, axum::http::header::HeaderMap) {
    let mut headers = axum::http::header::HeaderMap::new();
    headers.insert(
        axum::http::header::ALLOW,
        axum::http::HeaderValue::from_static(
            "GET, HEAD, POST, PUT, PATCH, DELETE, OPTIONS",
        ),
    );
    (axum::http::StatusCode::NO_CONTENT, headers)
}

/// Builds the daemon router.
pub fn build_router(state: HttpState) -> Router {
    Router::new()
        .route(
            "/api/v1/files",
            post(upload::upload_file)
                .get(list::list_files)
                .options(options_response)
                .layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/api/v1/files/batch",
            post(upload::upload_files_batch)
                .options(options_response)
                .layer(DefaultBodyLimit::disable()),
        )
        .route("/api/v1/pins", get(pins::list_pins).options(options_response))
        .route(
            "/api/v1/pins/reorder",
            post(pins::reorder).options(options_response),
        )
        .route("/api/v1/tags", get(tags::list_tags).options(options_response))
        .route(
            "/api/v1/files/{id}/tags",
            post(tags::add_tags).options(options_response),
        )
        .route(
            "/api/v1/files/{id}/tags/{tag}",
            axum::routing::delete(tags::remove_tag).options(options_response),
        )
        .route(
            "/api/v1/files/{id}/pin",
            axum::routing::put(pins::pin)
                .delete(pins::unpin)
                .options(options_response),
        )
        .route(
            "/api/v1/files/{id}/content",
            get(content::get_file_content).options(options_response),
        )
        .route(
            "/api/v1/files/{id}",
            get(file::get_file)
                .delete(delete::delete_file)
                .patch(rename::rename_file)
                .options(options_response),
        )
        .route(
            "/api/v1/search",
            get(search::search_files).options(options_response),
        )
        .route(
            "/api/v1/sessions/send",
            post(sessions::create_send_session).options(options_response),
        )
        .route(
            "/api/v1/sessions/receive",
            post(sessions::create_receive_session).options(options_response),
        )
        .route(
            "/api/v1/sessions/{token}",
            get(sessions::get_session).options(options_response),
        )
        .route(
            "/api/v1/sessions/{token}/use",
            post(sessions::use_session_endpoint).options(options_response),
        )
        .route("/healthz", get(status::healthz))
        .route("/readyz", get(status::readyz))
        .route("/api/v1/status", get(status::status).options(options_response))
        .fallback(web::web_fallback)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::cors::CorsLayer::very_permissive())
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            axum::http::header::CONTENT_SECURITY_POLICY,
            axum::http::HeaderValue::from_static("default-src 'self'"),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            axum::http::HeaderValue::from_static("nosniff"),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            axum::http::header::X_FRAME_OPTIONS,
            axum::http::HeaderValue::from_static("DENY"),
        ))
        .with_state(state)
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

#[cfg(test)]
mod tests {
    use std::io::Read;
    use std::net::{IpAddr, Ipv4Addr};
    use std::sync::Arc;
    use std::time::Instant;

    use axum::body::{to_bytes, Body};
    use axum::http::header::{CONTENT_RANGE, CONTENT_TYPE};
    use axum::http::{Request, StatusCode};
    use axum::Router;
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

    use super::{
        bind_error_message, build_router, ApplicationFileDeleteProvider,
        ApplicationFilePinProvider, ApplicationFileTagProvider, ApplicationFileUploadProvider,
        DaemonConfig, FileUploadProvider, HttpState, HttpUploadError, HttpUploadOutcome,
        HttpUploadRequest, MetadataStatsProvider, RepositoryMetadataStatsProvider,
    };
    use tssp_ports::RepositoryStats;

    #[test]
    fn config_builds_socket_address() {
        let config = DaemonConfig {
            bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 8421,
        };

        assert_eq!(config.socket_addr().to_string(), "127.0.0.1:8421");
    }

    #[tokio::test]
    async fn health_endpoint_returns_plain_ok() {
        let app = build_router(HttpState::new(Instant::now(), std::env::temp_dir()));
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
            HttpState::new(Instant::now(), std::env::temp_dir())
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
        let parsed: serde_json::Value = serde_json::from_slice(&body)
            .unwrap_or_else(|error| panic!("json parse failed: {error}"));
        assert_eq!(parsed["schema_version"], 1);
        assert_eq!(parsed["file_count"], 7);
        assert_eq!(parsed["tag_count"], 3);
    }

    #[tokio::test]
    async fn status_endpoint_reports_metadata_failure() {
        let app = build_router(
            HttpState::new(Instant::now(), std::env::temp_dir())
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
        let parsed: serde_json::Value = serde_json::from_slice(&body)
            .unwrap_or_else(|error| panic!("json parse failed: {error}"));
        assert_eq!(parsed["error"]["code"], "metadata_unavailable");
    }

    #[tokio::test]
    async fn upload_endpoint_accepts_single_multipart_file() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let app = build_router(
            HttpState::new(Instant::now(), temp.path().to_path_buf())
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
        let parsed: serde_json::Value = serde_json::from_slice(&body)
            .unwrap_or_else(|error| panic!("json parse failed: {error}"));
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
            HttpState::new(Instant::now(), temp.path().to_path_buf())
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
            HttpState::new(Instant::now(), temp.path().to_path_buf())
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
        let parsed: serde_json::Value = serde_json::from_slice(&body)
            .unwrap_or_else(|error| panic!("json parse failed: {error}"));
        assert_eq!(parsed["error"]["code"], "invalid_request");
    }

    #[tokio::test]
    async fn batch_upload_endpoint_returns_per_file_outcomes() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let app = build_router(
            HttpState::new(Instant::now(), temp.path().to_path_buf())
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
            HttpState::new(Instant::now(), temp.path().to_path_buf())
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
            HttpState::new(Instant::now(), temp.path().join("http-upload-tmp"))
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
            HttpState::new(Instant::now(), temp.path().join("http-upload-tmp"))
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
            HttpState::new(Instant::now(), temp.path().join("http-upload-tmp"))
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
            Some("true")
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
    async fn tag_endpoints_list_add_and_remove_tags() {
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
        let delete_service = DeleteFileService::new(storage.clone(), repository.clone());
        let tag_service = TagService::new(repository);
        let app = build_router(
            HttpState::new(Instant::now(), temp.path().join("http-upload-tmp"))
                .with_stats_provider(Arc::new(stats_provider))
                .with_upload_provider(Arc::new(ApplicationFileUploadProvider::new(upload_service)))
                .with_delete_provider(Arc::new(ApplicationFileDeleteProvider::new(delete_service)))
                .with_blob_reader(storage)
                .with_tag_provider(Arc::new(ApplicationFileTagProvider::new(tag_service))),
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

        let added = app
            .clone()
            .oneshot(add_tags_request(id, r#"["Docs","Family"]"#))
            .await
            .unwrap_or_else(|error| panic!("add tags request failed: {error}"));
        let added_body = response_json(added).await;
        assert_eq!(added_body["changed_count"], 1);

        let listed = app
            .clone()
            .oneshot(tags_request())
            .await
            .unwrap_or_else(|error| panic!("tags request failed: {error}"));
        let listed_body = response_json(listed).await;
        assert_eq!(listed_body["tags"].as_array().map(Vec::len), Some(2));
        assert_eq!(listed_body["tags"][0]["name"], "Docs");
        assert_eq!(listed_body["tags"][1]["name"], "Family");

        let removed = app
            .oneshot(remove_tag_request(id, "Family"))
            .await
            .unwrap_or_else(|error| panic!("remove tag request failed: {error}"));
        let removed_body = response_json(removed).await;
        assert_eq!(removed_body["changed_count"], 1);
    }

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
        let pin_service = PinService::new(repository);
        let app = build_router(
            HttpState::new(Instant::now(), temp.path().join("http-upload-tmp"))
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

        let first_pin = app
            .clone()
            .oneshot(pin_request(&first_id))
            .await
            .unwrap_or_else(|error| panic!("pin request failed: {error}"));
        assert_eq!(first_pin.status(), StatusCode::OK);
        let first_pin_body = response_json(first_pin).await;
        assert_eq!(first_pin_body["changed"], true);

        let second_pin = app
            .clone()
            .oneshot(pin_with_position_request(&second_id, r#"{"position":1}"#))
            .await
            .unwrap_or_else(|error| panic!("pin with position request failed: {error}"));
        assert_eq!(second_pin.status(), StatusCode::OK);
        let second_pin_body = response_json(second_pin).await;
        assert_eq!(second_pin_body["changed"], true);

        let listed = app
            .clone()
            .oneshot(pins_request())
            .await
            .unwrap_or_else(|error| panic!("pins list request failed: {error}"));
        assert_eq!(listed.status(), StatusCode::OK);
        let listed_body = response_json(listed).await;
        assert_eq!(listed_body["files"][0]["id"], second_id);
        assert_eq!(listed_body["files"][1]["id"], first_id);

        let reordered = app
            .clone()
            .oneshot(reorder_pins_request(&format!(
                r#"{{"ids":["{first_id}","{second_id}"]}}"#
            )))
            .await
            .unwrap_or_else(|error| panic!("pins reorder request failed: {error}"));
        assert_eq!(reordered.status(), StatusCode::OK);

        let listed_after_reorder = app
            .clone()
            .oneshot(pins_request())
            .await
            .unwrap_or_else(|error| panic!("pins list after reorder request failed: {error}"));
        let reordered_body = response_json(listed_after_reorder).await;
        assert_eq!(reordered_body["files"][0]["id"], first_id);
        assert_eq!(reordered_body["files"][1]["id"], second_id);

        let unpinned = app
            .oneshot(unpin_request(&first_id))
            .await
            .unwrap_or_else(|error| panic!("unpin request failed: {error}"));
        assert_eq!(unpinned.status(), StatusCode::OK);
        let unpinned_body = response_json(unpinned).await;
        assert_eq!(unpinned_body["changed"], true);
    }

    async fn assert_content_downloads(app: Router, first_id: &str) {
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

    #[tokio::test]
    async fn content_endpoint_reports_missing_blob_as_gone() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let storage = Arc::new(
            FilesystemBlobStore::new(temp.path().join("storage"))
                .unwrap_or_else(|error| panic!("blob store open failed: {error}")),
        );
        let app = build_router(
            HttpState::new(Instant::now(), temp.path().join("http-upload-tmp"))
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
            HttpState::new(Instant::now(), std::env::temp_dir())
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
            HttpState::new(Instant::now(), std::env::temp_dir())
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
            HttpState::new(Instant::now(), std::env::temp_dir())
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
            HttpState::new(Instant::now(), std::env::temp_dir())
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
            HttpState::new(Instant::now(), std::env::temp_dir())
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
            HttpState::new(Instant::now(), std::env::temp_dir())
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

    struct FixedStatsProvider;

    impl MetadataStatsProvider for FixedStatsProvider {
        fn stats(&self) -> Result<RepositoryStats, String> {
            Ok(RepositoryStats {
                file_count: 7,
                tag_count: 3,
                pinned_count: 2,
                recent_upload_count: 1,
            })
        }

        fn list_files(
            &self,
            _query: &tssp_ports::ListQuery,
        ) -> Result<tssp_ports::PagedFiles, String> {
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
    }

    struct FailingStatsProvider;

    impl MetadataStatsProvider for FailingStatsProvider {
        fn stats(&self) -> Result<RepositoryStats, String> {
            Err("metadata database is unavailable".to_owned())
        }

        fn list_files(
            &self,
            _query: &tssp_ports::ListQuery,
        ) -> Result<tssp_ports::PagedFiles, String> {
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
    }

    struct SingleRecordStatsProvider;

    impl MetadataStatsProvider for SingleRecordStatsProvider {
        fn stats(&self) -> Result<RepositoryStats, String> {
            Ok(RepositoryStats {
                file_count: 1,
                tag_count: 0,
                pinned_count: 0,
                recent_upload_count: 0,
            })
        }

        fn list_files(
            &self,
            _query: &tssp_ports::ListQuery,
        ) -> Result<tssp_ports::PagedFiles, String> {
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
    }

    struct EchoUploadProvider {
        deduplicated: bool,
    }

    impl FileUploadProvider for EchoUploadProvider {
        fn upload(
            &self,
            mut request: HttpUploadRequest,
        ) -> Result<HttpUploadOutcome, HttpUploadError> {
            let mut bytes = Vec::new();
            request
                .source
                .read_to_end(&mut bytes)
                .map_err(|error| HttpUploadError::Internal {
                    message: error.to_string(),
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

    struct BatchEchoUploadProvider;

    impl FileUploadProvider for BatchEchoUploadProvider {
        fn upload(
            &self,
            mut request: HttpUploadRequest,
        ) -> Result<HttpUploadOutcome, HttpUploadError> {
            let mut bytes = Vec::new();
            request
                .source
                .read_to_end(&mut bytes)
                .map_err(|error| HttpUploadError::Internal {
                    message: error.to_string(),
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

    fn multipart_request(body: &'static str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/api/v1/files")
            .header(CONTENT_TYPE, "multipart/form-data; boundary=tssp")
            .body(Body::from(body))
            .unwrap_or_else(|error| panic!("request build failed: {error}"))
    }

    fn batch_multipart_request(body: &'static str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/api/v1/files/batch")
            .header(CONTENT_TYPE, "multipart/form-data; boundary=tssp")
            .body(Body::from(body))
            .unwrap_or_else(|error| panic!("request build failed: {error}"))
    }

    fn status_request() -> Request<Body> {
        Request::builder()
            .method("GET")
            .uri("/api/v1/status")
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("request build failed: {error}"))
    }

    fn list_request(query: &str) -> Request<Body> {
        Request::builder()
            .method("GET")
            .uri(format!("/api/v1/files{query}"))
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("request build failed: {error}"))
    }

    fn file_request(id: &str) -> Request<Body> {
        Request::builder()
            .method("GET")
            .uri(format!("/api/v1/files/{id}"))
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("request build failed: {error}"))
    }

    fn content_request(id: &str, range: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder()
            .method("GET")
            .uri(format!("/api/v1/files/{id}/content?disposition=inline"));
        if let Some(range) = range {
            builder = builder.header("range", range);
        }
        builder
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("request build failed: {error}"))
    }

    fn delete_request(id: &str) -> Request<Body> {
        Request::builder()
            .method("DELETE")
            .uri(format!("/api/v1/files/{id}"))
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("request build failed: {error}"))
    }

    fn tags_request() -> Request<Body> {
        Request::builder()
            .method("GET")
            .uri("/api/v1/tags")
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("request build failed: {error}"))
    }

    fn add_tags_request(id: &str, body: &'static str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/files/{id}/tags"))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap_or_else(|error| panic!("request build failed: {error}"))
    }

    fn remove_tag_request(id: &str, tag: &str) -> Request<Body> {
        Request::builder()
            .method("DELETE")
            .uri(format!("/api/v1/files/{id}/tags/{tag}"))
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("request build failed: {error}"))
    }

    fn pin_request(id: &str) -> Request<Body> {
        Request::builder()
            .method("PUT")
            .uri(format!("/api/v1/files/{id}/pin"))
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("request build failed: {error}"))
    }

    fn pin_with_position_request(id: &str, body: &str) -> Request<Body> {
        Request::builder()
            .method("PUT")
            .uri(format!("/api/v1/files/{id}/pin"))
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap_or_else(|error| panic!("request build failed: {error}"))
    }

    fn unpin_request(id: &str) -> Request<Body> {
        Request::builder()
            .method("DELETE")
            .uri(format!("/api/v1/files/{id}/pin"))
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("request build failed: {error}"))
    }

    fn pins_request() -> Request<Body> {
        Request::builder()
            .method("GET")
            .uri("/api/v1/pins")
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("request build failed: {error}"))
    }

    fn reorder_pins_request(body: &str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/api/v1/pins/reorder")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap_or_else(|error| panic!("request build failed: {error}"))
    }

    async fn response_json(response: axum::response::Response) -> serde_json::Value {
        let body = to_bytes(response.into_body(), 4096)
            .await
            .unwrap_or_else(|error| panic!("body read failed: {error}"));
        serde_json::from_slice(&body).unwrap_or_else(|error| panic!("json parse failed: {error}"))
    }

    const REAL_UPLOAD_BODY: &str = "--tssp\r\n\
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

    const DUPLICATE_UPLOAD_BODY: &str = "--tssp\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"other.txt\"\r\n\
        Content-Type: text/plain\r\n\r\n\
        hello upload\r\n\
        --tssp--\r\n";

    const SECOND_UPLOAD_BODY: &str = "--tssp\r\n\
        Content-Disposition: form-data; name=\"file\"; filename=\"later.txt\"\r\n\
        Content-Type: text/plain\r\n\r\n\
        hello second\r\n\
        --tssp--\r\n";

    fn test_record(request: &HttpUploadRequest) -> FileRecord {
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
            tags: request.tags.iter().map(|tag| tag_value(tag)).collect(),
            pinned_at: request.pinned.then_some(1),
        }
    }

    fn single_record() -> FileRecord {
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
        }
    }

    fn file_id(value: &str) -> FileId {
        FileId::new(value).unwrap_or_else(|error| panic!("invalid file id: {error}"))
    }

    fn filename(value: &str) -> FileName {
        FileName::new(value).unwrap_or_else(|error| panic!("invalid filename: {error}"))
    }

    fn content_hash() -> ContentHash {
        ContentHash::new("abcdefabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123")
            .unwrap_or_else(|error| panic!("invalid hash: {error}"))
    }

    fn mime_type(value: &str) -> MimeType {
        MimeType::new(value).unwrap_or_else(|error| panic!("invalid mime type: {error}"))
    }

    fn storage_handle() -> StorageHandle {
        StorageHandle::new("blobs/ab/cd/abcdef")
            .unwrap_or_else(|error| panic!("invalid storage handle: {error}"))
    }

    fn timestamp(seconds: i64) -> UnixTimestamp {
        UnixTimestamp::new(seconds).unwrap_or_else(|error| panic!("invalid timestamp: {error}"))
    }

    fn tag_value(value: &str) -> Tag {
        Tag::new(value).unwrap_or_else(|error| panic!("invalid tag: {error}"))
    }
}
