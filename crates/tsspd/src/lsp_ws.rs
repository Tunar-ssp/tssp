//! LSP status reporting.
//!
//! Reports language server availability detected at startup.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

use crate::HttpState;
use crate::lsp::LspManager;

#[derive(Debug, Serialize)]
pub struct LspStatusResponse {
    pub schema_version: u8,
    pub status: String,
    pub available_languages: Vec<String>,
}

/// `GET /api/v1/workspaces/{id}/lsp/status`
///
/// Reports available language servers.
pub async fn lsp_status(
    _state: State<HttpState>,
    _path: Path<String>,
) -> (StatusCode, Json<LspStatusResponse>) {
    let manager = LspManager::default();
    let available = manager.available_languages();

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
