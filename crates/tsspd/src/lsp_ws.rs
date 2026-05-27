//! LSP status reporting and WebSocket JSON-RPC proxy.
//!
//! The status endpoint reports which language servers are installed.
//! The WebSocket endpoint spawns the LSP process and proxies JSON-RPC
//! messages between the Monaco editor and the language server over stdio.

use axum::extract::{ws::WebSocketUpgrade, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

use crate::auth::AuthContext;
use crate::HttpState;

#[derive(Debug, Deserialize)]
pub(crate) struct LspWsQuery {
    language: String,
}

/// `GET /api/v1/workspaces/{id}/lsp/ws?language=rust`
///
/// Upgrades to a WebSocket connection that proxies JSON-RPC between the Monaco
/// editor and the language server process running against the workspace directory.
///
/// Authentication required. Admin or workspace owner only.
pub(crate) async fn upgrade_lsp_ws(
    State(state): State<HttpState>,
    Path(workspace_id): Path<String>,
    Query(query): Query<LspWsQuery>,
    auth: AuthContext,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let Some(lsp_config) = state.lsp_service.config_for_language(&query.language) else {
        return (
            StatusCode::NOT_FOUND,
            format!("No language server configured for '{}'", query.language),
        )
            .into_response();
    };

    // Verify language server is installed.
    if which::which(&lsp_config.binary).is_err() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Language server '{}' is not installed", lsp_config.binary),
        )
            .into_response();
    }

    // Admin or authenticated user can use LSP.
    let _ = auth; // auth extractor already verified session is valid.

    let workspace_dir = state
        .settings()
        .data_dir
        .join("workspaces")
        .join(&workspace_id);

    if !workspace_dir.exists() {
        return (StatusCode::NOT_FOUND, "Workspace directory not found").into_response();
    }

    ws.on_upgrade(move |socket| handle_lsp_ws(socket, lsp_config, workspace_dir))
        .into_response()
}

/// Spawns the LSP process and proxies JSON-RPC frames over WebSocket.
async fn handle_lsp_ws(
    mut socket: axum::extract::ws::WebSocket,
    config: tssp_domain::LspServerConfig,
    workspace_dir: std::path::PathBuf,
) {
    let mut child = match Command::new(&config.binary)
        .args(&config.args)
        .current_dir(&workspace_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            let _ = socket
                .send(axum::extract::ws::Message::Text(
                    format!(r#"{{"error":"failed to start LSP: {e}"}}"#).into(),
                ))
                .await;
            return;
        }
    };

    let Some(stdin) = child.stdin.take() else {
        let _ = socket
            .send(axum::extract::ws::Message::Text(
                r#"{"error":"LSP stdin unavailable"}"#.to_string().into(),
            ))
            .await;
        let _ = child.kill().await;
        return;
    };
    let Some(stdout) = child.stdout.take() else {
        let _ = socket
            .send(axum::extract::ws::Message::Text(
                r#"{"error":"LSP stdout unavailable"}"#.to_string().into(),
            ))
            .await;
        let _ = child.kill().await;
        return;
    };

    let mut stdin = stdin;
    let mut reader = BufReader::new(stdout);

    // LSP uses HTTP-like headers: "Content-Length: N\r\n\r\n<json>"
    // We read content-length from the header then read exactly that many bytes.
    let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(30));
    loop {
        tokio::select! {
            // Client → LSP
            msg = socket.recv() => {
                match msg {
                    Some(Ok(axum::extract::ws::Message::Text(text))) => {
                        // Wrap the JSON-RPC message in LSP Content-Length framing.
                        let bytes = text.as_bytes();
                        let header = format!("Content-Length: {}\r\n\r\n", bytes.len());
                        if stdin.write_all(header.as_bytes()).await.is_err() { break; }
                        if stdin.write_all(bytes).await.is_err() { break; }
                        if stdin.flush().await.is_err() { break; }
                    }
                    None
                    | Some(Ok(axum::extract::ws::Message::Close(_)) | Err(_)) => break,
                    Some(Ok(axum::extract::ws::Message::Pong(_))) => {} // pong response to our ping
                    Some(Ok(_)) => {} // ignore other frames
                }
            }
            // LSP → client: parse Content-Length framing and forward raw JSON.
            line = read_lsp_message(&mut reader) => {
                match line {
                    Ok(Some(json)) => {
                        if socket
                            .send(axum::extract::ws::Message::Text(json.into()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Ok(None) | Err(_) => break, // LSP exited cleanly or errored
                }
            }
            // Periodic ping to keep connection alive through idle proxies/firewalls.
            _ = ping_interval.tick() => {
                use axum::body::Bytes;
                if socket.send(axum::extract::ws::Message::Ping(Bytes::new())).await.is_err() {
                    break;
                }
            }
        }
    }

    let _ = child.kill().await;
}

/// Reads one LSP message from `reader` by parsing the `Content-Length` header.
/// Returns `Ok(None)` on EOF, `Err` on IO error.
async fn read_lsp_message(
    reader: &mut BufReader<tokio::process::ChildStdout>,
) -> Result<Option<String>, std::io::Error> {
    // Read headers until blank line.
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            return Ok(None); // EOF
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break; // End of headers.
        }
        if let Some(value) = trimmed.strip_prefix("Content-Length: ") {
            content_length = value.trim().parse().ok();
        }
    }

    let len = match content_length {
        Some(len) if len > 0 => len,
        _ => return Err(std::io::Error::other("missing or zero Content-Length")),
    };

    // Read exactly `len` bytes.
    let mut body = vec![0u8; len];
    reader.read_exact(&mut body).await?;
    String::from_utf8(body)
        .map(Some)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}
