//! Unified error handling and response formatting for all APIs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;

/// Unified API error response format
#[derive(Debug, Serialize)]
pub struct ApiError {
    /// Error code for categorization
    pub code: String,
    /// User-friendly error message
    pub message: String,
    /// Optional detailed information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// HTTP status code
    #[serde(skip)]
    pub status: StatusCode,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl ApiError {
    /// Create a new API error
    pub fn new(code: impl Into<String>, message: impl Into<String>, status: StatusCode) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            status,
        }
    }

    /// Add details to the error
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Validation error (400 Bad Request)
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new("VALIDATION_ERROR", message, StatusCode::BAD_REQUEST)
    }

    /// Authentication error (401 Unauthorized)
    pub fn auth(message: impl Into<String>) -> Self {
        Self::new("AUTH_ERROR", message, StatusCode::UNAUTHORIZED)
    }

    /// Permission error (403 Forbidden)
    pub fn permission(message: impl Into<String>) -> Self {
        Self::new("PERMISSION_ERROR", message, StatusCode::FORBIDDEN)
    }

    /// Not found error (404)
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("NOT_FOUND", message, StatusCode::NOT_FOUND)
    }

    /// Conflict error (409)
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new("CONFLICT", message, StatusCode::CONFLICT)
    }

    /// Server error (500)
    pub fn server_error(message: impl Into<String>) -> Self {
        Self::new("SERVER_ERROR", message, StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Unavailable error (503)
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new("SERVICE_UNAVAILABLE", message, StatusCode::SERVICE_UNAVAILABLE)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status;
        let body = Json(self);
        (status, body).into_response()
    }
}

/// Result type for API handlers
pub type ApiResult<T> = Result<T, ApiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error() {
        let err = ApiError::validation("Invalid input");
        assert_eq!(err.code, "VALIDATION_ERROR");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_with_details() {
        let err = ApiError::not_found("User not found")
            .with_details("User ID: 123");
        assert_eq!(err.details, Some("User ID: 123".to_string()));
    }
}
