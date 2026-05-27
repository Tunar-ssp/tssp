//! Git repository status detection for workspaces.

use crate::auth::AuthContext;
use crate::state::HttpState;
use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    Json,
};
use tssp_domain::GitStatus;

/// Handler for workspace git status.
#[allow(dead_code)]
pub async fn git_status_handler(
    State(state): State<HttpState>,
    AxumPath(workspace_id): AxumPath<String>,
    _auth: AuthContext,
) -> Result<Json<GitStatus>, (StatusCode, String)> {
    let workspace_root = state
        .settings()
        .data_dir
        .join("workspaces")
        .join(&workspace_id);

    match state.git_service.get_status(&workspace_root).await {
        Ok(status) => Ok(Json(status)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}
