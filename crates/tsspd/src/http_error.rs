//! Shared HTTP JSON error bodies and unified error handling.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;

/// Top-level error envelope for API responses.
#[derive(Debug, Serialize)]
pub(crate) struct ErrorResponse {
    pub(crate) error: ErrorBody,
}

/// Machine-oriented error details.
#[derive(Debug, Serialize)]
pub(crate) struct ErrorBody {
    pub(crate) code: &'static str,
    pub(crate) message: String,
}

/// Unified API error type that can be converted into an Axum response.
#[derive(Debug, Error)]
pub enum ApiError {
    /// The requested resource was not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// The client request was invalid (e.g. validation failed).
    #[error("bad request: {0}")]
    BadRequest(String),

    /// A conflict occurred (e.g. duplicate key).
    #[error("conflict: {0}")]
    Conflict(String),

    /// The user is not authorized to perform the action.
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// The service is currently unavailable (e.g. database busy).
    #[error("service unavailable: {0}")]
    Unavailable(String),

    /// An internal server error occurred.
    #[error("internal error: {0}")]
    Internal(String),

    /// Storage limit reached.
    #[error("insufficient storage: {0}")]
    InsufficientStorage(String),
}

impl ApiError {
    /// Returns the HTTP status code for this error.
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InsufficientStorage(_) => StatusCode::from_u16(507).unwrap_or(StatusCode::INSUFFICIENT_STORAGE),
        }
    }

    /// Returns the machine-readable error code.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "not_found",
            Self::BadRequest(_) => "invalid_request",
            Self::Conflict(_) => "conflict",
            Self::Forbidden(_) => "forbidden",
            Self::Unavailable(_) => "service_unavailable",
            Self::Internal(_) => "internal_error",
            Self::InsufficientStorage(_) => "insufficient_storage",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.error_code();
        let message = self.to_string();

        (
            status,
            Json(ErrorResponse {
                error: ErrorBody { code, message },
            }),
        )
            .into_response()
    }
}

impl From<tssp_ports::RepositoryError> for ApiError {
    fn from(error: tssp_ports::RepositoryError) -> Self {
        match error {
            tssp_ports::RepositoryError::NotFound => Self::NotFound("resource not found".to_owned()),
            tssp_ports::RepositoryError::Busy => Self::Unavailable("metadata store is busy; please retry".to_owned()),
            tssp_ports::RepositoryError::Conflict { message } => Self::Conflict(message),
            tssp_ports::RepositoryError::OperationFailed { message } => Self::Internal(message),
        }
    }
}

impl From<tssp_domain::DomainError> for ApiError {
    fn from(error: tssp_domain::DomainError) -> Self {
        Self::BadRequest(error.to_string())
    }
}
