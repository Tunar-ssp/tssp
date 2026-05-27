//! Language Server Protocol (LSP) support for workspaces.

use axum::{
    extract::{Path, State},
    Json,
};
use tssp_domain::LspCapability;
use crate::state::HttpState;
use crate::auth::AuthContext;

/// Lists available language servers detected on this system.
#[allow(dead_code)]
pub fn list_available_languages(
    State(state): State<HttpState>,
    _auth: AuthContext,
) -> Json<Vec<String>> {
    Json(state.lsp_service.available_languages())
}

/// Gets availability status for a language in this workspace.
#[allow(dead_code)]
pub fn get_language_status(
    State(state): State<HttpState>,
    Path((_workspace_id, language)): Path<(String, String)>,
    _auth: AuthContext,
) -> Json<LspCapability> {
    Json(state.lsp_service.status_for_language(&language))
}
