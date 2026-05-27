//! LSP status reporting.
//!
//! Reports language server availability detected at startup.

#![allow(dead_code)]

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

use crate::HttpState;

#[derive(Debug, Serialize)]
pub struct LspStatusResponse {
    pub schema_version: u8,
    pub status: String,
    pub available_languages: Vec<String>,
}

/// `GET /api/v1/workspaces/{id}/lsp/status`
///
/// Reports available language servers.
pub fn lsp_status(
    State(state): State<HttpState>,
    _path: Path<String>,
) -> (StatusCode, Json<LspStatusResponse>) {
    let available = state.lsp_service.available_languages();

    let status = if available.is_empty() {
        "unavailable"
    } else {
        "available"
    };

    (
        StatusCode::OK,
        Json(LspStatusResponse {
            schema_version: 1,
            status: status.to_string(),
            available_languages: available,
        }),
    )
}
