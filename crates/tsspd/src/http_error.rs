//! Shared HTTP JSON error bodies and unified error handling.

use serde::Serialize;

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
