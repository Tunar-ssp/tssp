//! File deletion delivery.

use axum::extract::{Path, State};
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use tssp_app::{
    log_audit_event, AuditAction, DeleteFileError, DeleteFileService, RestoreFileError,
    RestoreFileService,
};
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
    auth: crate::auth::AuthContext,
    Path(id): Path<String>,
) -> Response {
    let file_id = match FileId::new(id) {
        Ok(value) => value,
        Err(error) => return invalid_file_id_response(error.to_string()),
    };

    if let Ok(Some(file)) = state.stats_provider.find_file(&file_id) {
        if !(auth.is_admin() || file.owner_id.as_ref() == Some(&auth.user_id)) {
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::FileDelete,
                Some(&auth.user_id),
                Some("file"),
                Some(file_id.as_str()),
                "denied",
                Some("unauthorized"),
            );
            return forbidden_response();
        }
    }

    let delete_provider = state.delete_provider.clone();
    let file_id_str = file_id.as_str().to_string();
    let audit_user_id = auth.user_id.clone();
    let repository = state.repository.clone();

    match tokio::task::spawn_blocking(move || delete_provider.delete(file_id)).await {
        Ok(Ok(outcome)) => {
            log_audit_event(
                repository.as_ref(),
                AuditAction::FileDelete,
                Some(&audit_user_id),
                Some("file"),
                Some(&file_id_str),
                "success",
                None,
            );
            delete_success_response(outcome)
        }
        Ok(Err(error)) => {
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::FileDelete,
                Some(&auth.user_id),
                Some("file"),
                Some(file_id_str.as_str()),
                "failed",
                Some("delete operation failed"),
            );
            error.response()
        }
        Err(error) => {
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::FileDelete,
                Some(&auth.user_id),
                Some("file"),
                Some(file_id_str.as_str()),
                "failed",
                Some("worker failed"),
            );
            HttpDeleteError::Internal {
                message: format!("delete worker failed: {error}"),
            }
            .response()
        }
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

fn forbidden_response() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "forbidden",
                message: "you do not have permission to delete this file".to_owned(),
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

/// Handles HTTP restore requests through the application layer.
pub trait FileRestoreProvider: Send + Sync {
    /// Restores a soft-deleted file.
    ///
    /// # Errors
    ///
    /// Returns [`HttpRestoreError`] when restoration fails.
    fn restore(&self, id: FileId) -> Result<HttpRestoreOutcome, HttpRestoreError>;
}

#[derive(Debug)]
pub(crate) struct StaticFileRestoreProvider;

impl FileRestoreProvider for StaticFileRestoreProvider {
    fn restore(&self, _id: FileId) -> Result<HttpRestoreOutcome, HttpRestoreError> {
        Err(HttpRestoreError::Unavailable {
            message: "restore service is not configured".to_owned(),
        })
    }
}

/// Successful HTTP restore outcome.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HttpRestoreOutcome {
    /// True when a soft-deleted file was restored.
    pub existed: bool,
}

/// Restore failure mapped to HTTP error responses.
#[derive(Debug)]
pub enum HttpRestoreError {
    /// Metadata store is busy.
    Busy {
        /// Short client-facing message.
        message: String,
    },
    /// Restore service is unavailable.
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

impl HttpRestoreError {
    pub(crate) fn response(&self) -> Response {
        let (status, code, message) = match self {
            Self::Busy { message } => (
                StatusCode::SERVICE_UNAVAILABLE,
                "metadata_busy",
                message.clone(),
            ),
            Self::Unavailable { message } => (
                StatusCode::SERVICE_UNAVAILABLE,
                "restore_unavailable",
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

/// Restore provider backed by the core application restore service.
#[allow(dead_code)]
pub struct ApplicationFileRestoreProvider<R> {
    service: RestoreFileService<R>,
}

impl<R> ApplicationFileRestoreProvider<R> {
    /// Creates a provider from a restore service.
    #[must_use]
    #[allow(dead_code)]
    pub const fn new(service: RestoreFileService<R>) -> Self {
        Self { service }
    }
}

impl<R> FileRestoreProvider for ApplicationFileRestoreProvider<R>
where
    R: FileRepository + Send + Sync,
{
    fn restore(&self, id: FileId) -> Result<HttpRestoreOutcome, HttpRestoreError> {
        self.service
            .restore(&id)
            .map(|outcome| HttpRestoreOutcome {
                existed: outcome.existed,
            })
            .map_err(map_restore_error)
    }
}

pub(crate) async fn restore_file(
    State(state): State<HttpState>,
    auth: crate::auth::AuthContext,
    Path(id): Path<String>,
) -> Response {
    let file_id = match FileId::new(id) {
        Ok(value) => value,
        Err(error) => return invalid_file_id_response(error.to_string()),
    };

    let restore_provider = state.restore_provider.clone();
    let file_id_str = file_id.as_str().to_string();
    let audit_user_id = auth.user_id.clone();
    let repository = state.repository.clone();

    match tokio::task::spawn_blocking(move || restore_provider.restore(file_id)).await {
        Ok(Ok(outcome)) => {
            log_audit_event(
                repository.as_ref(),
                AuditAction::FileRestore,
                Some(&audit_user_id),
                Some("file"),
                Some(&file_id_str),
                "success",
                None,
            );
            restore_success_response(outcome)
        }
        Ok(Err(error)) => {
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::FileRestore,
                Some(&auth.user_id),
                Some("file"),
                Some(file_id_str.as_str()),
                "failed",
                Some("restore operation failed"),
            );
            error.response()
        }
        Err(error) => {
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::FileRestore,
                Some(&auth.user_id),
                Some("file"),
                Some(file_id_str.as_str()),
                "failed",
                Some("worker failed"),
            );
            HttpRestoreError::Internal {
                message: format!("restore worker failed: {error}"),
            }
            .response()
        }
    }
}

#[allow(dead_code)]
fn map_restore_error(error: RestoreFileError) -> HttpRestoreError {
    match error {
        RestoreFileError::Repository(RepositoryError::Busy) => HttpRestoreError::Busy {
            message: "metadata store is busy; retry the restore".to_owned(),
        },
        RestoreFileError::Repository(error) => HttpRestoreError::Internal {
            message: error.to_string(),
        },
    }
}

fn restore_success_response(outcome: HttpRestoreOutcome) -> Response {
    let mut response = StatusCode::OK.into_response();
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/json"),
    );
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "restored": outcome.existed
        })),
    )
        .into_response()
}

/// Permanently deletes a specific file from trash.
pub(crate) async fn permanent_delete(
    State(state): State<HttpState>,
    auth: crate::auth::AuthContext,
    Path(id): Path<String>,
) -> Response {
    let file_id = match FileId::new(id) {
        Ok(value) => value,
        Err(error) => return invalid_file_id_response(error.to_string()),
    };

    let file_id_str = file_id.as_str().to_string();
    let audit_user_id = auth.user_id.clone();
    let repository = state.repository.clone();
    let repository_for_audit = state.repository.clone();

    match tokio::task::spawn_blocking(move || repository.purge_deleted_file(&file_id)).await {
        Ok(Ok(was_deleted)) => {
            if was_deleted {
                log_audit_event(
                    repository_for_audit.as_ref(),
                    AuditAction::FileDelete,
                    Some(&audit_user_id),
                    Some("file"),
                    Some(&file_id_str),
                    "success",
                    Some("permanently deleted from trash"),
                );
                StatusCode::NO_CONTENT.into_response()
            } else {
                log_audit_event(
                    repository_for_audit.as_ref(),
                    AuditAction::FileDelete,
                    Some(&audit_user_id),
                    Some("file"),
                    Some(&file_id_str),
                    "failed",
                    Some("not found in trash"),
                );
                invalid_file_id_response("file not found in trash".to_owned())
            }
        }
        Ok(Err(_)) => {
            log_audit_event(
                repository_for_audit.as_ref(),
                AuditAction::FileDelete,
                Some(&audit_user_id),
                Some("file"),
                Some(file_id_str.as_str()),
                "failed",
                Some("purge operation failed"),
            );
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "purge_failed",
                "failed to permanently delete file",
            )
        }
        Err(_) => {
            log_audit_event(
                repository_for_audit.as_ref(),
                AuditAction::FileDelete,
                Some(&audit_user_id),
                Some("file"),
                Some(file_id_str.as_str()),
                "failed",
                Some("worker failed"),
            );
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "purge worker failed",
            )
        }
    }
}

/// Returns all soft-deleted files (trash).
pub(crate) async fn list_trash(
    State(state): State<HttpState>,
    _auth: crate::auth::AuthContext,
) -> Response {
    match tokio::task::spawn_blocking(move || {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let cutoff = now + 1;

        match tssp_domain::UnixTimestamp::new(i64::try_from(cutoff).unwrap_or(i64::MAX)) {
            Ok(cutoff_ts) => match state.repository.list_deleted_files(cutoff_ts) {
                Ok(files) => Ok(files),
                Err(e) => Err(format!("failed to list deleted files: {e}")),
            },
            Err(e) => Err(format!("invalid timestamp: {e}")),
        }
    })
    .await
    {
        Ok(Ok(files)) => Json(serde_json::json!({
            "files": files
        }))
        .into_response(),
        Ok(Err(msg)) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "query_error",
            msg.as_str(),
        ),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "worker_error",
            "list trash worker failed",
        ),
    }
}

/// Empties trash by permanently deleting all files older than retention period.
pub(crate) async fn empty_trash(
    State(state): State<HttpState>,
    _auth: crate::auth::AuthContext,
) -> Response {
    let settings = state.settings().clone();
    let repo = state.repository.clone();

    let result = tokio::task::spawn_blocking(move || {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let retention_seconds = settings.trash_retention_days * 86_400;
        let older_than_secs = now.saturating_sub(retention_seconds) + 1;

        match tssp_domain::UnixTimestamp::new(i64::try_from(older_than_secs).unwrap_or(i64::MAX)) {
            Ok(cutoff) => match repo.list_deleted_files(cutoff) {
                Ok(deleted_files) => {
                    let mut purged_count = 0;
                    for file in deleted_files {
                        if let Ok(was_deleted) = repo.purge_deleted_file(&file.id) {
                            if was_deleted {
                                purged_count += 1;
                            }
                        }
                    }
                    Ok(purged_count)
                }
                Err(_) => Err("failed to list deleted files"),
            },
            Err(_) => Err("invalid retention period"),
        }
    })
    .await;

    match result {
        Ok(Ok(count)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "purged": count
            })),
        )
            .into_response(),
        Ok(Err(msg)) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "purge_error", msg),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "worker_error",
            "empty trash worker failed",
        ),
    }
}

fn error_response(
    status: StatusCode,
    code: impl Into<String>,
    message: impl Into<String>,
) -> Response {
    (
        status,
        Json(serde_json::json!({
            "error": {
                "code": code.into(),
                "message": message.into(),
            }
        })),
    )
        .into_response()
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
