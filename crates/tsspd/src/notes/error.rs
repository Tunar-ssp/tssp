//! HTTP error mapping for note endpoints.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use tssp_app::NoteError;
use tssp_ports::RepositoryError;

use crate::{ErrorBody, ErrorResponse};

/// Note failure mapped to HTTP.
#[derive(Debug)]
pub enum HttpNoteError {
    /// Client supplied invalid input.
    InvalidRequest {
        /// Short client-facing message.
        message: String,
    },
    /// Request body exceeds the configured note size limit.
    PayloadTooLarge {
        /// Short client-facing message.
        message: String,
    },
    /// Note does not exist.
    NotFound {
        /// Short client-facing message.
        message: String,
    },
    /// Note service is not configured.
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

impl HttpNoteError {
    /// Converts this error into an HTTP response.
    #[must_use]
    pub fn response(&self) -> Response {
        let (status, code, message) = match self {
            Self::InvalidRequest { message } => {
                (StatusCode::BAD_REQUEST, "invalid_request", message.clone())
            }
            Self::PayloadTooLarge { message } => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "payload_too_large",
                message.clone(),
            ),
            Self::NotFound { message } => {
                (StatusCode::NOT_FOUND, "note_not_found", message.clone())
            }
            Self::Unavailable { message } => (
                StatusCode::SERVICE_UNAVAILABLE,
                "note_unavailable",
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

/// Maps application note errors to HTTP-facing errors.
pub(crate) fn map_note_error(error: NoteError) -> HttpNoteError {
    match error {
        NoteError::InvalidRequest(domain) => HttpNoteError::InvalidRequest {
            message: domain.to_string(),
        },
        NoteError::NotFound | NoteError::Repository(RepositoryError::NotFound) => {
            HttpNoteError::NotFound {
                message: "note was not found".to_owned(),
            }
        }
        NoteError::IdGeneration(message) => HttpNoteError::Internal { message },
        NoteError::Repository(other) => HttpNoteError::Internal {
            message: other.to_string(),
        },
    }
}
