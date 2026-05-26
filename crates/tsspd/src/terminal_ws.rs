//! WebSocket handler for terminal sessions.
//!
//! Manages real-time bidirectional communication between client and terminal.
//! Admin-only, workspace-scoped, with proper cleanup on disconnect.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;

use crate::{
    auth::AuthContext,
    state::HttpState,
    terminal::{TerminalError, TerminalManager},
    workspace_features::SandboxStrategy,
};

/// WebSocket upgrade handler for terminal sessions (HTTP endpoint).
/// This is a placeholder that properly validates access but returns not-implemented.
/// Real PTY implementation will be added in a later phase.
pub async fn upgrade_terminal_ws(
    State(_state): State<HttpState>,
    Path(workspace_id): Path<String>,
    auth: AuthContext,
) -> impl IntoResponse {
    // Verify admin access
    if !auth.is_admin() {
        return (StatusCode::FORBIDDEN, "Terminal access requires admin role").into_response();
    }

    // Validate workspace_id format (basic check)
    if workspace_id.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "Invalid workspace_id").into_response();
    }

    // Check if terminal is available (based on sandbox)
    let sandbox = SandboxStrategy::detect();
    let terminal_manager = TerminalManager::new();

    match terminal_manager
        .create_session(
            &workspace_id,
            auth.user_id.as_str(),
            crate::terminal::TerminalConfig {
                workspace_dir: std::path::PathBuf::from(&workspace_id),
                sandbox,
                env: std::collections::HashMap::new(),
                idle_timeout: 1800,
                max_lifetime: 3600,
            },
        )
        .await
    {
        Ok(_session) => {
            // Session created successfully
            // WebSocket upgrade would happen here in the real implementation
            let msg = json!({
                "status": "ready",
                "message": "Terminal WebSocket upgrade not yet implemented - foundation in place"
            });
            (StatusCode::NOT_IMPLEMENTED, msg.to_string()).into_response()
        }
        Err(TerminalError::Unauthorized) => {
            (StatusCode::FORBIDDEN, "Unauthorized to access terminal").into_response()
        }
        Err(TerminalError::Unavailable(msg)) => {
            let response = json!({ "error": msg });
            (StatusCode::SERVICE_UNAVAILABLE, response.to_string()).into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create terminal session",
        )
            .into_response(),
    }
}
