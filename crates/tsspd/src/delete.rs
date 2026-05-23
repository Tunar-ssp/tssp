//! File deletion delivery.

use axum::extract::{Path, State};
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use tssp_app::{DeleteFileError, DeleteFileService};
use tssp_domain::FileId;
use tssp_ports::{BlobStore, FileRepository, RepositoryError};

use crate::{ErrorBody, ErrorResponse, HttpState};

const ALREADY_GONE_HEADER: HeaderName = HeaderName::from_static("x-tssp-already-gone");
const BLOB_CLEANED_HEADER: HeaderName = HeaderName::from_static("x-tssp-blob-cleaned");

/// Handles completed HTTP delete requests through the application layer.
pub trait FileDeleteProvider: Send + Sync {
    /// Deletes one file id.
    ///
    /// # Errors
    ///
    /// Returns [`HttpDeleteError`] when metadata or storage cleanup fails.
    fn delete(&self, id: FileId) -> Result<HttpDeleteOutcome, HttpDeleteError>;
}

#[derive(Debug)]
pub(crate) struct StaticFileDeleteProvider;

impl FileDeleteProvider for StaticFileDeleteProvider {
    fn delete(&self, _id: FileId) -> Result<HttpDeleteOutcome, HttpDeleteError> {
        Err(HttpDeleteError::Unavailable {
            message: "delete service is not configured".to_owned(),
        })
    }
}

/// Successful HTTP delete outcome.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HttpDeleteOutcome {
    /// True when a metadata record existed and was removed.
    pub existed: bool,
    /// True when the last blob reference was removed from storage.
    pub blob_cleaned: bool,
}

/// Delete failure mapped to HTTP error responses.
#[derive(Debug)]
pub enum HttpDeleteError {
    /// Metadata store is busy.
    Busy {
        /// Short client-facing message.
        message: String,
    },
    /// Delete service is unavailable.
    Unavailable {
        /// Short client-facing message.
        message: String,
    },
    /// Last-reference blob cleanup failed.
    BlobCleanup {
        /// Short client-facing message.
        message: String,
    },
    /// Unexpected server-side failure.
    Internal {
        /// Short client-facing message.
        message: String,
    },
}

impl HttpDeleteError {
    pub(crate) fn response(&self) -> Response {
        let (status, code, message) = match self {
            Self::Busy { message } => (
                StatusCode::SERVICE_UNAVAILABLE,
                "metadata_busy",
                message.clone(),
            ),
            Self::Unavailable { message } => (
                StatusCode::SERVICE_UNAVAILABLE,
                "delete_unavailable",
                message.clone(),
            ),
            Self::BlobCleanup { message } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "blob_cleanup_failed",
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

/// Delete provider backed by the core application delete service.
pub struct ApplicationFileDeleteProvider<B, R> {
    service: DeleteFileService<B, R>,
}

impl<B, R> ApplicationFileDeleteProvider<B, R> {
    /// Creates a provider from a delete service.
    #[must_use]
    pub const fn new(service: DeleteFileService<B, R>) -> Self {
        Self { service }
    }
}

impl<B, R> FileDeleteProvider for ApplicationFileDeleteProvider<B, R>
where
    B: BlobStore + Send + Sync,
    R: FileRepository + Send + Sync,
{
    fn delete(&self, id: FileId) -> Result<HttpDeleteOutcome, HttpDeleteError> {
        self.service
            .delete(&id)
            .map(|outcome| HttpDeleteOutcome {
                existed: outcome.existed,
                blob_cleaned: outcome.blob_cleaned,
            })
            .map_err(map_delete_error)
    }
}

pub(crate) async fn delete_file(
    State(state): State<HttpState>,
    Path(id): Path<String>,
) -> Response {
    let file_id = match FileId::new(id) {
        Ok(value) => value,
        Err(error) => return invalid_file_id_response(error.to_string()),
    };

    let delete_provider = state.delete_provider.clone();
    let _mutation_guard = state.storage_mutation_lock.lock().await;
    match tokio::task::spawn_blocking(move || delete_provider.delete(file_id)).await {
        Ok(Ok(outcome)) => delete_success_response(outcome),
        Ok(Err(error)) => error.response(),
        Err(error) => HttpDeleteError::Internal {
            message: format!("delete worker failed: {error}"),
        }
        .response(),
    }
}

fn invalid_file_id_response(message: String) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "invalid_file_id",
                message,
            },
        }),
    )
        .into_response()
}

fn delete_success_response(outcome: HttpDeleteOutcome) -> Response {
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().insert(
        ALREADY_GONE_HEADER,
        HeaderValue::from_static(if outcome.existed { "false" } else { "true" }),
    );
    response.headers_mut().insert(
        BLOB_CLEANED_HEADER,
        HeaderValue::from_static(if outcome.blob_cleaned {
            "true"
        } else {
            "false"
        }),
    );
    response
}

fn map_delete_error(error: DeleteFileError) -> HttpDeleteError {
    match error {
        DeleteFileError::Repository(RepositoryError::Busy) => HttpDeleteError::Busy {
            message: "metadata store is busy; retry the delete".to_owned(),
        },
        DeleteFileError::Repository(error) => HttpDeleteError::Internal {
            message: error.to_string(),
        },
        DeleteFileError::BlobCleanup(error) => HttpDeleteError::BlobCleanup {
            message: error.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;
    use axum::http::StatusCode;
    use tssp_app::DeleteFileError;
    use tssp_ports::{BlobStoreError, RepositoryError};

    use super::{
        delete_success_response, map_delete_error, FileDeleteProvider, HttpDeleteError,
        HttpDeleteOutcome, StaticFileDeleteProvider, ALREADY_GONE_HEADER, BLOB_CLEANED_HEADER,
    };
    use tssp_domain::{FileId, StorageHandle};

    #[test]
    fn static_delete_provider_reports_unavailable() {
        let provider = StaticFileDeleteProvider;

        let result = provider.delete(file_id("file-1"));

        assert!(
            matches!(result, Err(HttpDeleteError::Unavailable { message }) if message.contains("not configured"))
        );
    }

    #[tokio::test]
    async fn delete_success_response_sets_idempotency_headers() {
        let response = delete_success_response(HttpDeleteOutcome {
            existed: false,
            blob_cleaned: false,
        });

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            response
                .headers()
                .get(ALREADY_GONE_HEADER)
                .and_then(|value| value.to_str().ok()),
            Some("true")
        );
        assert_eq!(
            response
                .headers()
                .get(BLOB_CLEANED_HEADER)
                .and_then(|value| value.to_str().ok()),
            Some("false")
        );
        let body = to_bytes(response.into_body(), 8)
            .await
            .unwrap_or_else(|error| panic!("body read failed: {error}"));
        assert!(body.is_empty());
    }

    #[test]
    fn map_delete_error_translates_failures() {
        assert!(matches!(
            map_delete_error(DeleteFileError::Repository(RepositoryError::Busy)),
            HttpDeleteError::Busy { .. }
        ));
        assert!(matches!(
            map_delete_error(DeleteFileError::Repository(
                RepositoryError::OperationFailed {
                    message: "failed".to_owned()
                }
            )),
            HttpDeleteError::Internal { .. }
        ));
        assert!(matches!(
            map_delete_error(DeleteFileError::BlobCleanup(
                BlobStoreError::CleanupFailed {
                    handle: storage_handle(),
                    message: "failed".to_owned(),
                }
            )),
            HttpDeleteError::BlobCleanup { .. }
        ));
    }

    fn file_id(value: &str) -> FileId {
        FileId::new(value).unwrap_or_else(|error| panic!("invalid file id: {error}"))
    }

    fn storage_handle() -> StorageHandle {
        StorageHandle::new("blobs/ab/cd/abcdef")
            .unwrap_or_else(|error| panic!("invalid storage handle: {error}"))
    }

    #[tokio::test]
    async fn http_delete_error_response_maps_status_codes() {
        use axum::body::to_bytes;

        let cases = vec![
            (
                HttpDeleteError::Busy {
                    message: "busy".to_owned(),
                },
                StatusCode::SERVICE_UNAVAILABLE,
                "metadata_busy",
            ),
            (
                HttpDeleteError::Unavailable {
                    message: "off".to_owned(),
                },
                StatusCode::SERVICE_UNAVAILABLE,
                "delete_unavailable",
            ),
            (
                HttpDeleteError::BlobCleanup {
                    message: "failed".to_owned(),
                },
                StatusCode::INTERNAL_SERVER_ERROR,
                "blob_cleanup_failed",
            ),
            (
                HttpDeleteError::Internal {
                    message: "crash".to_owned(),
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

    #[tokio::test]
    async fn invalid_file_id_response_returns_bad_request() {
        use super::invalid_file_id_response;
        use axum::body::to_bytes;

        let response = invalid_file_id_response("bad-id".to_owned());
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), 1024)
            .await
            .unwrap_or_else(|e| panic!("body read: {e}"));
        let parsed: serde_json::Value =
            serde_json::from_slice(&body).unwrap_or_else(|e| panic!("json parse: {e}"));
        assert_eq!(parsed["error"]["code"], "invalid_file_id");
    }

    #[tokio::test]
    async fn delete_success_response_sets_existing_headers() {
        let response = delete_success_response(HttpDeleteOutcome {
            existed: true,
            blob_cleaned: true,
        });

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            response
                .headers()
                .get(ALREADY_GONE_HEADER)
                .and_then(|v| v.to_str().ok()),
            Some("false")
        );
        assert_eq!(
            response
                .headers()
                .get(BLOB_CLEANED_HEADER)
                .and_then(|v| v.to_str().ok()),
            Some("true")
        );
    }
}
