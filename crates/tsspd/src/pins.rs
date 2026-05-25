//! Pin HTTP delivery.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use tssp_app::{PinError, PinService};
use tssp_domain::FileRecord;
use tssp_ports::{FileRepository, PinOutcome, RepositoryError};

use crate::upload::FileRecordResponse;
use crate::{ErrorBody, ErrorResponse, HttpState};

/// Handles HTTP pin operations through the application layer.
pub trait FilePinProvider: Send + Sync {
    /// Lists pinned files.
    ///
    /// # Errors
    ///
    /// Returns [`HttpPinError`] when metadata is unavailable.
    fn list_pins(&self) -> Result<Vec<FileRecord>, HttpPinError>;

    /// Pins a file.
    ///
    /// # Errors
    ///
    /// Returns [`HttpPinError`] when the request is invalid or metadata fails.
    fn pin(&self, id: String, position: Option<u32>) -> Result<HttpPinMutation, HttpPinError>;

    /// Unpins a file.
    ///
    /// # Errors
    ///
    /// Returns [`HttpPinError`] when the request is invalid or metadata fails.
    fn unpin(&self, id: String) -> Result<HttpPinMutation, HttpPinError>;

    /// Reorders pins.
    ///
    /// # Errors
    ///
    /// Returns [`HttpPinError`] when the request is invalid or metadata fails.
    fn reorder(&self, ids: Vec<String>) -> Result<(), HttpPinError>;
}

#[derive(Debug)]
pub(crate) struct StaticFilePinProvider;

impl FilePinProvider for StaticFilePinProvider {
    fn list_pins(&self) -> Result<Vec<FileRecord>, HttpPinError> {
        Err(HttpPinError::Unavailable {
            message: "pin service is not configured".to_owned(),
        })
    }

    fn pin(&self, _id: String, _position: Option<u32>) -> Result<HttpPinMutation, HttpPinError> {
        Err(HttpPinError::Unavailable {
            message: "pin service is not configured".to_owned(),
        })
    }

    fn unpin(&self, _id: String) -> Result<HttpPinMutation, HttpPinError> {
        Err(HttpPinError::Unavailable {
            message: "pin service is not configured".to_owned(),
        })
    }

    fn reorder(&self, _ids: Vec<String>) -> Result<(), HttpPinError> {
        Err(HttpPinError::Unavailable {
            message: "pin service is not configured".to_owned(),
        })
    }
}

/// Successful pin mutation outcome.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HttpPinMutation {
    /// True if the record existed.
    pub existed: bool,
    /// True if the pin state was changed.
    pub changed: bool,
}

impl From<PinOutcome> for HttpPinMutation {
    fn from(outcome: PinOutcome) -> Self {
        Self {
            existed: outcome.existed,
            changed: outcome.changed,
        }
    }
}

/// Pin failure mapped to HTTP error responses.
#[derive(Debug, Clone)]
pub enum HttpPinError {
    /// Client supplied invalid data.
    InvalidRequest {
        /// Short client-facing message.
        message: String,
    },
    /// File id does not exist.
    NotFound {
        /// Short client-facing message.
        message: String,
    },
    /// User does not have permission.
    Forbidden {
        /// Short client-facing message.
        message: String,
    },
    /// Metadata store is busy.
    Busy {
        /// Short client-facing message.
        message: String,
    },
    /// Pin service is unavailable.
    Unavailable {
        /// Short client-facing message.
        message: String,
    },
    /// Unexpected server-side failure.
    Internal {
        /// Short client-facing message.
        message: String,
    },
}

impl HttpPinError {
    fn response(&self) -> Response {
        let (status, code, message) = match self {
            Self::InvalidRequest { message } => {
                (StatusCode::BAD_REQUEST, "invalid_request", message.clone())
            }
            Self::NotFound { message } => {
                (StatusCode::NOT_FOUND, "file_not_found", message.clone())
            }
            Self::Forbidden { message } => (
                StatusCode::FORBIDDEN,
                "forbidden",
                message.clone(),
            ),
            Self::Busy { message } => (
                StatusCode::SERVICE_UNAVAILABLE,
                "metadata_busy",
                message.clone(),
            ),
            Self::Unavailable { message } => (
                StatusCode::SERVICE_UNAVAILABLE,
                "pin_unavailable",
                message.clone(),
            ),
            Self::Internal { message } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                message.clone(),
            ),
        };
        (
            status,
            Json(ErrorResponse {
                error: ErrorBody { code, message },
            }),
        )
            .into_response()
    }
}

/// Pin provider backed by the core application pin service.
pub struct ApplicationFilePinProvider<R> {
    service: PinService<R>,
}

impl<R> ApplicationFilePinProvider<R> {
    /// Creates a provider from a pin service.
    #[must_use]
    pub const fn new(service: PinService<R>) -> Self {
        Self { service }
    }
}

impl<R> FilePinProvider for ApplicationFilePinProvider<R>
where
    R: FileRepository + Send + Sync,
{
    fn list_pins(&self) -> Result<Vec<FileRecord>, HttpPinError> {
        self.service.list_pins().map_err(map_pin_error)
    }

    fn pin(&self, id: String, position: Option<u32>) -> Result<HttpPinMutation, HttpPinError> {
        self.service
            .pin(&id, position)
            .map(Into::into)
            .map_err(map_pin_error)
    }

    fn unpin(&self, id: String) -> Result<HttpPinMutation, HttpPinError> {
        self.service
            .unpin(&id)
            .map(Into::into)
            .map_err(map_pin_error)
    }

    fn reorder(&self, ids: Vec<String>) -> Result<(), HttpPinError> {
        let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        self.service.reorder(&refs).map_err(map_pin_error)
    }
}

pub(crate) async fn list_pins(
    State(state): State<HttpState>,
    auth: crate::auth::AuthContext,
) -> Response {
    let provider = state.pin_provider.clone();
    let user_id = auth.user_id.clone();
    let is_admin = auth.is_admin();
    match tokio::task::spawn_blocking(move || provider.list_pins()).await {
        Ok(Ok(files)) => {
            // Filter pins by owner: non-admin users only see their own pins
            let filtered = if is_admin {
                files
            } else {
                files
                    .into_iter()
                    .filter(|f| f.owner_id.as_ref() == Some(&user_id))
                    .collect()
            };
            (StatusCode::OK, Json(PinListResponse::from_records(&filtered))).into_response()
        }
        Ok(Err(error)) => error.response(),
        Err(error) => HttpPinError::Internal {
            message: format!("pin worker failed: {error}"),
        }
        .response(),
    }
}

#[derive(Deserialize)]
pub(crate) struct PinRequest {
    position: Option<u32>,
}

pub(crate) async fn pin(
    State(state): State<HttpState>,
    auth: crate::auth::AuthContext,
    Path(id): Path<String>,
    payload: Option<Json<PinRequest>>,
) -> Response {
    let file_id = match tssp_domain::FileId::new(id.clone()) {
        Ok(f) => f,
        Err(e) => {
            return HttpPinError::InvalidRequest {
                message: e.to_string(),
            }
            .response()
        }
    };

    let file = match state.stats_provider.find_file(&file_id) {
        Ok(Some(f)) => f,
        Ok(None) => {
            return HttpPinError::NotFound {
                message: "file not found".to_owned(),
            }
            .response()
        }
        Err(e) => {
            return HttpPinError::Internal {
                message: e,
            }
            .response()
        }
    };

    if !(auth.is_admin() || file.owner_id.as_ref() == Some(&auth.user_id)) {
        return HttpPinError::Forbidden {
            message: "you do not have permission to pin this file".to_owned(),
        }
        .response();
    }

    let provider = state.pin_provider.clone();
    let position = payload.and_then(|Json(p)| p.position);
    match tokio::task::spawn_blocking(move || provider.pin(id, position)).await {
        Ok(Ok(outcome)) => pin_mutation_response(outcome),
        Ok(Err(error)) => error.response(),
        Err(error) => HttpPinError::Internal {
            message: format!("pin worker failed: {error}"),
        }
        .response(),
    }
}

pub(crate) async fn unpin(
    State(state): State<HttpState>,
    auth: crate::auth::AuthContext,
    Path(id): Path<String>,
) -> Response {
    let file_id = match tssp_domain::FileId::new(id.clone()) {
        Ok(f) => f,
        Err(e) => {
            return HttpPinError::InvalidRequest {
                message: e.to_string(),
            }
            .response()
        }
    };

    let file = match state.stats_provider.find_file(&file_id) {
        Ok(Some(f)) => f,
        Ok(None) => {
            return HttpPinError::NotFound {
                message: "file not found".to_owned(),
            }
            .response()
        }
        Err(e) => {
            return HttpPinError::Internal {
                message: e,
            }
            .response()
        }
    };

    if !(auth.is_admin() || file.owner_id.as_ref() == Some(&auth.user_id)) {
        return HttpPinError::Forbidden {
            message: "you do not have permission to unpin this file".to_owned(),
        }
        .response();
    }

    let provider = state.pin_provider.clone();
    match tokio::task::spawn_blocking(move || provider.unpin(id)).await {
        Ok(Ok(outcome)) => pin_mutation_response(outcome),
        Ok(Err(error)) => error.response(),
        Err(error) => HttpPinError::Internal {
            message: format!("pin worker failed: {error}"),
        }
        .response(),
    }
}

#[derive(Deserialize)]
pub(crate) struct ReorderRequest {
    ids: Vec<String>,
}

pub(crate) async fn reorder(
    State(state): State<HttpState>,
    auth: crate::auth::AuthContext,
    Json(payload): Json<ReorderRequest>,
) -> Response {
    if payload.ids.is_empty() {
        return (StatusCode::OK, Json(ReorderResponse { schema_version: 1 })).into_response();
    }

    for id_str in &payload.ids {
        let file_id = match tssp_domain::FileId::new(id_str.clone()) {
            Ok(f) => f,
            Err(e) => {
                return HttpPinError::InvalidRequest {
                    message: e.to_string(),
                }
                .response()
            }
        };

        let file = match state.stats_provider.find_file(&file_id) {
            Ok(Some(f)) => f,
            Ok(None) => {
                return HttpPinError::NotFound {
                    message: "file not found".to_owned(),
                }
                .response()
            }
            Err(e) => {
                return HttpPinError::Internal {
                    message: e,
                }
                .response()
            }
        };

        if !(auth.is_admin() || file.owner_id.as_ref() == Some(&auth.user_id)) {
            return HttpPinError::Forbidden {
                message: "you do not have permission to reorder pins for this file".to_owned(),
            }
            .response();
        }
    }

    let provider = state.pin_provider.clone();
    match tokio::task::spawn_blocking(move || provider.reorder(payload.ids)).await {
        Ok(Ok(())) => (StatusCode::OK, Json(ReorderResponse { schema_version: 1 })).into_response(),
        Ok(Err(error)) => error.response(),
        Err(error) => HttpPinError::Internal {
            message: format!("pin worker failed: {error}"),
        }
        .response(),
    }
}

fn pin_mutation_response(outcome: HttpPinMutation) -> Response {
    (
        StatusCode::OK,
        Json(PinMutationResponse {
            schema_version: 1,
            changed: outcome.changed,
        }),
    )
        .into_response()
}

fn map_pin_error(error: PinError) -> HttpPinError {
    match error {
        PinError::InvalidRequest(error) => HttpPinError::InvalidRequest {
            message: error.to_string(),
        },
        PinError::Repository(RepositoryError::NotFound) => HttpPinError::NotFound {
            message: "file was not found".to_owned(),
        },
        PinError::Repository(RepositoryError::Busy) => HttpPinError::Busy {
            message: "metadata store is busy; retry the pin operation".to_owned(),
        },
        PinError::Repository(error) => HttpPinError::Internal {
            message: error.to_string(),
        },
    }
}

#[derive(Debug, Serialize)]
struct PinListResponse {
    schema_version: u8,
    files: Vec<FileRecordResponse>,
}

impl PinListResponse {
    fn from_records(records: &[FileRecord]) -> Self {
        Self {
            schema_version: 1,
            files: records
                .iter()
                .map(FileRecordResponse::from_record)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
struct PinMutationResponse {
    schema_version: u8,
    changed: bool,
}

#[derive(Debug, Serialize)]
struct ReorderResponse {
    schema_version: u8,
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;
    use axum::http::StatusCode;
    use tssp_app::PinError;
    use tssp_domain::FileRecord;
    use tssp_ports::{PinOutcome, RepositoryError};

    use super::{
        map_pin_error, pin_mutation_response, FilePinProvider, HttpPinError, HttpPinMutation,
        StaticFilePinProvider,
    };

    #[test]
    fn static_pin_provider_reports_unavailable() {
        let provider = StaticFilePinProvider;

        let list = provider.list_pins();
        let pin = provider.pin("file-1".to_owned(), None);
        let unpin = provider.unpin("file-1".to_owned());
        let reorder = provider.reorder(vec!["file-1".to_owned()]);

        assert!(matches!(list, Err(HttpPinError::Unavailable { .. })));
        assert!(matches!(pin, Err(HttpPinError::Unavailable { .. })));
        assert!(matches!(unpin, Err(HttpPinError::Unavailable { .. })));
        assert!(matches!(reorder, Err(HttpPinError::Unavailable { .. })));
    }

    #[tokio::test]
    async fn pin_mutation_response_returns_json_contract() {
        let response = pin_mutation_response(HttpPinMutation {
            existed: true,
            changed: true,
        });

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024)
            .await
            .unwrap_or_else(|error| panic!("body read failed: {error}"));
        let parsed: serde_json::Value = serde_json::from_slice(&body)
            .unwrap_or_else(|error| panic!("json parse failed: {error}"));
        assert_eq!(parsed["schema_version"], 1);
        assert_eq!(parsed["changed"], true);
    }

    #[test]
    fn map_pin_error_translates_repository_failures() {
        assert!(matches!(
            map_pin_error(PinError::Repository(RepositoryError::NotFound)),
            HttpPinError::NotFound { .. }
        ));
        assert!(matches!(
            map_pin_error(PinError::Repository(RepositoryError::Busy)),
            HttpPinError::Busy { .. }
        ));
        assert!(matches!(
            map_pin_error(PinError::Repository(RepositoryError::OperationFailed {
                message: "failed".to_owned()
            })),
            HttpPinError::Internal { .. }
        ));
    }

    #[tokio::test]
    async fn http_pin_error_response_maps_status_codes() {
        use super::HttpPinError;
        use axum::body::to_bytes;

        let cases = vec![
            (
                HttpPinError::InvalidRequest {
                    message: "bad id".to_owned(),
                },
                StatusCode::BAD_REQUEST,
                "invalid_request",
            ),
            (
                HttpPinError::NotFound {
                    message: "missing".to_owned(),
                },
                StatusCode::NOT_FOUND,
                "file_not_found",
            ),
            (
                HttpPinError::Busy {
                    message: "busy".to_owned(),
                },
                StatusCode::SERVICE_UNAVAILABLE,
                "metadata_busy",
            ),
            (
                HttpPinError::Unavailable {
                    message: "off".to_owned(),
                },
                StatusCode::SERVICE_UNAVAILABLE,
                "pin_unavailable",
            ),
            (
                HttpPinError::Internal {
                    message: "crashed".to_owned(),
                },
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
            ),
        ];

        for (error, expected_status, expected_code) in cases {
            let response = error.response();
            assert_eq!(response.status(), expected_status);
            let body = to_bytes(response.into_body(), 1024)
                .await
                .unwrap_or_else(|e| panic!("body read: {e}"));
            let parsed: serde_json::Value =
                serde_json::from_slice(&body).unwrap_or_else(|e| panic!("json parse: {e}"));
            assert_eq!(parsed["error"]["code"], expected_code);
        }
    }

    #[test]
    fn http_pin_mutation_from_pin_outcome() {
        let outcome = PinOutcome {
            existed: true,
            changed: false,
        };
        let mutation: HttpPinMutation = outcome.into();
        assert!(mutation.existed);
        assert!(!mutation.changed);
    }

    #[test]
    fn pin_list_response_builds_from_records() {
        use super::PinListResponse;
        use tssp_domain::{
            ContentHash, FileId, FileName, FileRecord, FileSize, MimeType, StorageHandle,
            UnixTimestamp,
        };

        let record = FileRecord {
            id: FileId::new("pin-1").unwrap_or_else(|e| panic!("{e}")),
            name: FileName::new("file.txt").unwrap_or_else(|e| panic!("{e}")),
            size: FileSize::new(10),
            content_hash: ContentHash::new(
                "abcdefabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123",
            )
            .unwrap_or_else(|e| panic!("{e}")),
            mime_type: MimeType::new("text/plain").unwrap_or_else(|e| panic!("{e}")),
            storage_handle: StorageHandle::new("blobs/ab/cd/test")
                .unwrap_or_else(|e| panic!("{e}")),
            uploaded_at: UnixTimestamp::new(1_700_000_000).unwrap_or_else(|e| panic!("{e}")),
            tags: vec![],
            pinned_at: Some(1),
            folder_path: String::new(),
            owner_id: None,
            visibility: tssp_domain::Visibility::Private,
            public_token: None,
        };

        let response = PinListResponse::from_records(&[record]);
        assert_eq!(response.schema_version, 1);
        assert_eq!(response.files.len(), 1);
    }

    struct ErrorPinProvider;

    impl FilePinProvider for ErrorPinProvider {
        fn list_pins(&self) -> Result<Vec<FileRecord>, HttpPinError> {
            Err(HttpPinError::Internal {
                message: "test error".to_owned(),
            })
        }

        fn pin(
            &self,
            _id: String,
            _position: Option<u32>,
        ) -> Result<HttpPinMutation, HttpPinError> {
            Err(HttpPinError::NotFound {
                message: "not found".to_owned(),
            })
        }

        fn unpin(&self, _id: String) -> Result<HttpPinMutation, HttpPinError> {
            Err(HttpPinError::NotFound {
                message: "not found".to_owned(),
            })
        }

        fn reorder(&self, _ids: Vec<String>) -> Result<(), HttpPinError> {
            Err(HttpPinError::InvalidRequest {
                message: "invalid".to_owned(),
            })
        }
    }

    #[tokio::test]
    async fn list_pins_endpoint_returns_error_from_provider() {
        use crate::HttpState;
        use crate::auth::AuthContext;
        use axum::extract::State;
        use std::sync::Arc;

        let provider = Arc::new(ErrorPinProvider);
        let state = HttpState::test_http_state(std::path::PathBuf::from("/tmp"))
            .with_pin_provider(provider);

        let response = super::list_pins(State(state), AuthContext::open_access()).await;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn pin_endpoint_returns_not_found_error() {
        use crate::HttpState;
        use crate::auth::AuthContext;
        use axum::extract::State;
        use std::sync::Arc;

        let provider = Arc::new(ErrorPinProvider);
        let state = HttpState::test_http_state(std::path::PathBuf::from("/tmp"))
            .with_pin_provider(provider);

        let response = super::pin(
            State(state),
            AuthContext::open_access(),
            axum::extract::Path("file-1".to_string()),
            None,
        )
        .await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn unpin_endpoint_returns_error_from_provider() {
        use crate::HttpState;
        use crate::auth::AuthContext;
        use axum::extract::State;
        use std::sync::Arc;

        let provider = Arc::new(ErrorPinProvider);
        let state = HttpState::test_http_state(std::path::PathBuf::from("/tmp"))
            .with_pin_provider(provider);

        let response = super::unpin(State(state), AuthContext::open_access(), axum::extract::Path("file-1".to_string())).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn reorder_endpoint_handles_empty_ids() {
        use crate::HttpState;
        use crate::auth::AuthContext;
        use axum::extract::State;
        use axum::Json;
        use std::sync::Arc;

        let state = HttpState::test_http_state(std::path::PathBuf::from("/tmp"))
            .with_pin_provider(Arc::new(StaticFilePinProvider));

        let response =
            super::reorder(State(state), AuthContext::open_access(), Json(super::ReorderRequest { ids: vec![] })).await;
        assert_eq!(response.status(), StatusCode::OK);
    }
}
