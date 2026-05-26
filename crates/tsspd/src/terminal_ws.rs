//! WebSocket handler for real terminal sessions.
//!
//! Manages bidirectional I/O between client and sandboxed shell process.
//! Admin-only, workspace-scoped, with proper PTY lifecycle and cleanup.

use axum::{
    extract::{ws::WebSocketUpgrade, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

use crate::{
    auth::AuthContext, state::HttpState, terminal::TerminalManager, terminal_pty::PtySession,
    workspace_features::SandboxStrategy,
};
use tssp_app::audit::{log_audit_event, AuditAction};

/// WebSocket upgrade handler for terminal sessions.
/// Validates admin access, sandbox availability, creates PTY, streams I/O.
#[allow(clippy::too_many_lines)]
pub async fn upgrade_terminal_ws(
    State(state): State<HttpState>,
    Path(workspace_id): Path<String>,
    auth: AuthContext,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Verify admin access
    if !auth.is_admin() {
        log_audit_event(
            state.repository.as_ref(),
            AuditAction::TerminalStart,
            Some(&auth.user_id),
            Some("workspace"),
            Some(&workspace_id),
            "forbidden",
            Some("Admin role required"),
        );
        return Err((
            StatusCode::FORBIDDEN,
            "Terminal access requires admin role".to_string(),
        ));
    }

    // Validate workspace_id format
    if workspace_id.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Invalid workspace_id".to_string()));
    }

    // Check if terminal is available
    let sandbox = SandboxStrategy::detect();
    let terminal_manager = state.terminal_manager.clone();

    // Validate sandbox is available
    if !sandbox.is_available() {
        log_audit_event(
            state.repository.as_ref(),
            AuditAction::TerminalStart,
            Some(&auth.user_id),
            Some("workspace"),
            Some(&workspace_id),
            "failed",
            Some("Sandbox not available"),
        );
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Sandbox not available (bubblewrap or systemd-nspawn required)".to_string(),
        ));
    }

    // Resolve actual workspace directory from data_dir
    let workspace_dir = state
        .settings()
        .data_dir
        .join("workspaces")
        .join(&workspace_id);

    // Verify workspace directory exists
    if !workspace_dir.exists() {
        log_audit_event(
            state.repository.as_ref(),
            AuditAction::TerminalStart,
            Some(&auth.user_id),
            Some("workspace"),
            Some(&workspace_id),
            "failed",
            Some("workspace directory not found"),
        );
        return Err((
            StatusCode::NOT_FOUND,
            "Workspace directory not found".to_string(),
        ));
    }

    // Create terminal session
    let session = terminal_manager
        .create_session(
            &workspace_id,
            auth.user_id.as_str(),
            crate::terminal::TerminalConfig {
                workspace_dir: workspace_dir.clone(),
                sandbox,
                env: std::collections::HashMap::new(),
                idle_timeout: 1800, // 30 minutes
                max_lifetime: 3600, // 1 hour
            },
        )
        .await
        .map_err(|e| {
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::TerminalStart,
                Some(&auth.user_id),
                Some("workspace"),
                Some(&workspace_id),
                "failed",
                Some(&e.to_string()),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    let session_id = session.id.clone();
    let workspace_id_clone = workspace_id.clone();
    let state_clone = state.clone();
    let workspace_dir_clone = workspace_dir;
    let sandbox_clone = sandbox;

    log_audit_event(
        state.repository.as_ref(),
        AuditAction::TerminalStart,
        Some(&auth.user_id),
        Some("workspace"),
        Some(&workspace_id),
        "started",
        Some(session_id.as_str()),
    );

    // Upgrade WebSocket and handle the connection
    Ok(ws.on_upgrade(move |socket| {
        handle_terminal_ws(
            socket,
            session_id,
            terminal_manager,
            workspace_id,
            workspace_id_clone,
            workspace_dir_clone,
            sandbox_clone,
            state_clone,
        )
    }))
}

/// Handle WebSocket connection for terminal I/O.
#[allow(clippy::too_many_arguments)]
async fn handle_terminal_ws(
    mut socket: axum::extract::ws::WebSocket,
    session_id: crate::terminal::TerminalSessionId,
    terminal_manager: Arc<TerminalManager>,
    _workspace_id: String,
    workspace_id_for_audit: String,
    workspace_dir: PathBuf,
    sandbox: SandboxStrategy,
    state: HttpState,
) {
    // Spawn PTY process in workspace with configured sandbox
    let mut pty_session = match PtySession::spawn_in_workspace(&workspace_dir, sandbox) {
        Ok(pty) => {
            // Mark session as started in manager
            if terminal_manager.mark_started(&session_id).await.is_err() {
                let msg_text = json!({"error": "Failed to mark session started"}).to_string();
                let _ = socket
                    .send(axum::extract::ws::Message::Text(msg_text.into()))
                    .await;
                return;
            }
            pty
        }
        Err(e) => {
            let error_msg = format!("Failed to start terminal: {e}");
            let msg_text = json!({"error": error_msg}).to_string();
            let _ = socket
                .send(axum::extract::ws::Message::Text(msg_text.into()))
                .await;
            let _ = terminal_manager.close_session(&session_id).await;
            return;
        }
    };

    // Send connection established message
    let connect_msg = json!({"type": "connected", "session_id": session_id.as_str()}).to_string();
    if socket
        .send(axum::extract::ws::Message::Text(connect_msg.into()))
        .await
        .is_err()
    {
        let _ = terminal_manager.close_session(&session_id).await;
        return;
    }

    // Main I/O loop
    loop {
        tokio::select! {
            // Handle incoming WebSocket messages (input)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(axum::extract::ws::Message::Text(text))) => {
                        let _ = terminal_manager.update_activity(&session_id).await;

                        if let Ok(input_msg) = serde_json::from_str::<serde_json::Value>(text.as_str()) {
                            if let Some(input_data) = input_msg.get("input").and_then(|v| v.as_str()) {
                                if let Err(e) = pty_session.write_input(input_data.as_bytes()).await {
                                    let err_msg = json!({"error": format!("Write failed: {e}")}).to_string();
                                    let _ = socket.send(axum::extract::ws::Message::Text(err_msg.into())).await;
                                    break;
                                }
                            }
                        }
                    }
                    Some(Ok(axum::extract::ws::Message::Close(_)) | Err(_)) | None => break,
                    _ => {}
                }
            }

            // Poll for output from PTY
            () = tokio::time::sleep(tokio::time::Duration::from_millis(50)) => {
                match pty_session.read_output().await {
                    Ok(data) if data.is_empty() => {
                        // Process exited
                        let _ = socket
                            .send(axum::extract::ws::Message::Text(
                                json!({"type": "exit", "code": 0}).to_string().into()
                            ))
                            .await;
                        break;
                    }
                    Ok(data) => {
                        let _ = terminal_manager.update_activity(&session_id).await;
                        let output_msg = json!({
                            "type": "output",
                            "data": String::from_utf8_lossy(&data)
                        }).to_string();
                        if socket
                            .send(axum::extract::ws::Message::Text(output_msg.into()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    }

    // Cleanup
    let _ = pty_session.kill().await;
    let _ = terminal_manager.close_session(&session_id).await;

    log_audit_event(
        state.repository.as_ref(),
        AuditAction::TerminalStop,
        None,
        Some("workspace"),
        Some(&workspace_id_for_audit),
        "stopped",
        Some(session_id.as_str()),
    );
}
