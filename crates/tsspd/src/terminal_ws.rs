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
use std::path::{Path as StdPath, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::auth::AuthContext;
use crate::state::HttpState;
use tssp_app::audit::{log_audit_event, AuditAction};
use tssp_app::terminal::TerminalService;
use tssp_domain::{SandboxStrategy, TerminalConfig, TerminalSessionId};

/// Max size for a single WebSocket input message (64KB).
const MAX_WS_INPUT_SIZE: usize = 65_536;

/// WebSocket upgrade handler for terminal sessions.
/// Validates admin access, sandbox availability, creates PTY, streams I/O.
pub async fn upgrade_terminal_ws(
    State(state): State<HttpState>,
    Path(workspace_id): Path<String>,
    auth: AuthContext,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    validate_admin(&state, &auth, &workspace_id)?;

    let terminal_service = state.terminal_service.clone();
    let sandbox = terminal_service.provider().detect_sandbox_strategy();
    validate_sandbox(&state, &auth, &workspace_id, sandbox)?;

    let workspace_dir = state
        .settings()
        .data_dir
        .join("workspaces")
        .join(&workspace_id);
    validate_workspace_dir(&state, &auth, &workspace_id, &workspace_dir)?;

    let session = create_session(&state, &auth, &workspace_id, &workspace_dir, sandbox).await?;

    log_audit_event(
        state.repository.as_ref(),
        AuditAction::TerminalStart,
        Some(&auth.user_id),
        Some("workspace"),
        Some(&workspace_id),
        "started",
        Some(session.id.as_str()),
    );

    Ok(ws.on_upgrade(move |socket| {
        handle_terminal_ws(
            socket,
            session.id,
            terminal_service,
            workspace_id,
            workspace_dir,
            sandbox,
            state,
        )
    }))
}

fn validate_admin(
    state: &HttpState,
    auth: &AuthContext,
    workspace_id: &str,
) -> Result<(), (StatusCode, String)> {
    if !auth.is_admin() {
        log_audit_event(
            state.repository.as_ref(),
            AuditAction::TerminalStart,
            Some(&auth.user_id),
            Some("workspace"),
            Some(workspace_id),
            "forbidden",
            Some("Admin role required"),
        );
        return Err((
            StatusCode::FORBIDDEN,
            "Terminal access requires admin role".to_string(),
        ));
    }
    if workspace_id.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Invalid workspace_id".to_string()));
    }
    Ok(())
}

fn validate_sandbox(
    state: &HttpState,
    auth: &AuthContext,
    workspace_id: &str,
    sandbox: SandboxStrategy,
) -> Result<(), (StatusCode, String)> {
    if !sandbox.is_available() {
        log_audit_event(
            state.repository.as_ref(),
            AuditAction::TerminalStart,
            Some(&auth.user_id),
            Some("workspace"),
            Some(workspace_id),
            "failed",
            Some("Sandbox not available"),
        );
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Sandbox not available (bubblewrap or systemd-nspawn required)".to_string(),
        ));
    }
    Ok(())
}

fn validate_workspace_dir(
    state: &HttpState,
    auth: &AuthContext,
    workspace_id: &str,
    workspace_dir: &StdPath,
) -> Result<(), (StatusCode, String)> {
    if !workspace_dir.exists() {
        log_audit_event(
            state.repository.as_ref(),
            AuditAction::TerminalStart,
            Some(&auth.user_id),
            Some("workspace"),
            Some(workspace_id),
            "failed",
            Some("workspace directory not found"),
        );
        return Err((
            StatusCode::NOT_FOUND,
            "Workspace directory not found".to_string(),
        ));
    }
    Ok(())
}

async fn create_session(
    state: &HttpState,
    auth: &AuthContext,
    workspace_id: &str,
    workspace_dir: &StdPath,
    sandbox: SandboxStrategy,
) -> Result<tssp_domain::TerminalSession, (StatusCode, String)> {
    state
        .terminal_service
        .create_session(
            workspace_id,
            auth.user_id.as_str(),
            TerminalConfig {
                workspace_dir: workspace_dir.to_path_buf(),
                sandbox,
                env: std::collections::HashMap::new(),
                idle_timeout: 1800,
                max_lifetime: 3600,
            },
        )
        .await
        .map_err(|e| {
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::TerminalStart,
                Some(&auth.user_id),
                Some("workspace"),
                Some(workspace_id),
                "failed",
                Some(&e.to_string()),
            );
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })
}

/// Handle WebSocket connection for terminal I/O.
async fn handle_terminal_ws(
    mut socket: axum::extract::ws::WebSocket,
    session_id: TerminalSessionId,
    terminal_service: Arc<TerminalService>,
    workspace_id_for_audit: String,
    workspace_dir: PathBuf,
    sandbox: SandboxStrategy,
    state: HttpState,
) {
    let config = TerminalConfig {
        workspace_dir: workspace_dir.clone(),
        sandbox,
        env: std::collections::HashMap::new(),
        idle_timeout: 1800,
        max_lifetime: 3600,
    };

    let mut pty_process = match terminal_service
        .provider()
        .spawn_pty(&workspace_dir, &config)
        .await
    {
        Ok(pty) => {
            if terminal_service.mark_started(&session_id).await.is_err() {
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
            let _ = terminal_service.close_session(&session_id).await;
            return;
        }
    };

    if let Some(child) = pty_process.child.downcast_mut::<tokio::process::Child>() {
        if let (Some(mut stdin), Some(mut stdout)) = (child.stdin.take(), child.stdout.take()) {
            let connect_msg =
                json!({"type": "connected", "session_id": session_id.as_str()}).to_string();
            if socket
                .send(axum::extract::ws::Message::Text(connect_msg.into()))
                .await
                .is_err()
            {
                let _ = terminal_service.close_session(&session_id).await;
                let _ = child.kill().await;
                return;
            }

            let mut buf = [0u8; 4096];
            loop {
                tokio::select! {
                    msg = socket.recv() => {
                        match msg {
                            Some(Ok(axum::extract::ws::Message::Text(text))) => {
                                if text.len() <= MAX_WS_INPUT_SIZE {
                                    let _ = terminal_service.update_activity(&session_id).await;
                                    if let Ok(input_msg) = serde_json::from_str::<serde_json::Value>(text.as_str()) {
                                        if let Some(input_data) = input_msg.get("input").and_then(|v| v.as_str()) {
                                            if stdin.write_all(input_data.as_bytes()).await.is_err() { break; }
                                            let _ = stdin.flush().await;
                                        }
                                    }
                                } else { break; }
                            }
                            _ => break,
                        }
                    }
                    n = stdout.read(&mut buf) => {
                        match n {
                            Ok(0) | Err(_) => break,
                            Ok(n) => {
                                let _ = terminal_service.update_activity(&session_id).await;
                                let output_msg = json!({"type": "output", "data": String::from_utf8_lossy(&buf[..n])}).to_string();
                                if socket.send(axum::extract::ws::Message::Text(output_msg.into())).await.is_err() { break; }
                            }
                        }
                    }
                }
            }
            let _ = child.kill().await;
        }
    }

    let _ = terminal_service.close_session(&session_id).await;
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
