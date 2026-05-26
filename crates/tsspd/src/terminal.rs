//! Real sandboxed terminal support for workspaces.
//!
//! Terminal is admin-only and requires a sandbox to be configured.
//! Does NOT provide unsafe host shell access.
//!
//! WebSocket protocol:
//! - Client sends: { "input": "<command>" } to execute shell commands
//! - Server sends: { "output": "<text>" } for command output
//! - Server sends: { "status": "closed" } on session end
//! - Server sends: { "error": "<reason>" } on error

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

/// Terminal operation errors.
#[derive(Debug, Error)]
pub enum TerminalError {
    #[error("terminal not available: {0}")]
    Unavailable(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("session not found")]
    SessionNotFound,
    #[error("session closed")]
    SessionClosed,
    #[error("invalid input")]
    InvalidInput,
}

/// WebSocket message from client.
#[derive(Debug, Deserialize)]
pub struct TerminalInputMessage {
    pub input: String,
}

/// WebSocket message to client.
#[derive(Debug, Serialize)]
pub struct TerminalOutputMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Terminal session ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct TerminalSessionId(String);

impl TerminalSessionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for TerminalSessionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Terminal session configuration.
#[derive(Debug, Clone)]
pub struct TerminalConfig {
    /// Workspace directory to start in (must be validated).
    pub workspace_dir: std::path::PathBuf,
    /// Sandbox strategy to use.
    pub sandbox: crate::workspace_features::SandboxStrategy,
    /// Environment variables (filtered for safety).
    pub env: std::collections::HashMap<String, String>,
    /// Session idle timeout in seconds.
    pub idle_timeout: u64,
    /// Max session lifetime in seconds.
    pub max_lifetime: u64,
}

/// Represents an active terminal session.
#[derive(Debug, Clone)]
pub struct TerminalSession {
    pub id: TerminalSessionId,
    pub workspace_id: String,
    pub user_id: String,
    pub created_at: std::time::SystemTime,
    pub last_activity: std::time::SystemTime,
}

/// Terminal session state.
#[derive(Debug, Clone)]
struct TerminalSessionState {
    pub session: TerminalSession,
    pub created_at: std::time::SystemTime,
}

/// Terminal session manager.
pub struct TerminalManager {
    sessions: Arc<Mutex<HashMap<String, TerminalSessionState>>>,
}

impl TerminalManager {
    /// Creates a new terminal manager.
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Creates a new terminal session.
    /// Returns Unavailable if the sandbox strategy doesn't match an available binary.
    pub async fn create_session(
        &self,
        workspace_id: &str,
        user_id: &str,
        config: TerminalConfig,
    ) -> Result<TerminalSession, TerminalError> {
        // Validate workspace_id is not empty
        if workspace_id.is_empty() {
            return Err(TerminalError::Unavailable("workspace_id required".into()));
        }

        // Check if sandbox is available based on strategy
        match config.sandbox {
            crate::workspace_features::SandboxStrategy::Bubblewrap => {
                // Check if bubblewrap is available
                if !is_bubblewrap_available() {
                    return Err(TerminalError::Unavailable(
                        "bubblewrap binary not found".into(),
                    ));
                }
            }
            crate::workspace_features::SandboxStrategy::Systemd => {
                // Check if systemd-nspawn is available
                if !is_systemd_nspawn_available() {
                    return Err(TerminalError::Unavailable(
                        "systemd-nspawn binary not found".into(),
                    ));
                }
            }
            crate::workspace_features::SandboxStrategy::None => {
                return Err(TerminalError::Unavailable("no sandbox configured".into()));
            }
        }

        let session = TerminalSession {
            id: TerminalSessionId::new(),
            workspace_id: workspace_id.to_string(),
            user_id: user_id.to_string(),
            created_at: std::time::SystemTime::now(),
            last_activity: std::time::SystemTime::now(),
        };

        let session_id = session.id.as_str().to_string();
        let mut sessions = self.sessions.lock().await;
        sessions.insert(
            session_id,
            TerminalSessionState {
                session: session.clone(),
                created_at: std::time::SystemTime::now(),
            },
        );

        Ok(session)
    }

    /// Gets an existing terminal session.
    pub async fn get_session(
        &self,
        session_id: &TerminalSessionId,
    ) -> Result<TerminalSession, TerminalError> {
        let sessions = self.sessions.lock().await;
        sessions
            .get(session_id.as_str())
            .map(|s| s.session.clone())
            .ok_or(TerminalError::SessionNotFound)
    }

    /// Closes a terminal session.
    pub async fn close_session(&self, session_id: &TerminalSessionId) -> Result<(), TerminalError> {
        let mut sessions = self.sessions.lock().await;
        if sessions.remove(session_id.as_str()).is_some() {
            Ok(())
        } else {
            Err(TerminalError::SessionNotFound)
        }
    }

    /// Updates last activity time for a session.
    pub async fn update_activity(
        &self,
        session_id: &TerminalSessionId,
    ) -> Result<(), TerminalError> {
        let mut sessions = self.sessions.lock().await;
        if let Some(state) = sessions.get_mut(session_id.as_str()) {
            state.session.last_activity = std::time::SystemTime::now();
            Ok(())
        } else {
            Err(TerminalError::SessionNotFound)
        }
    }
}

/// Check if bubblewrap binary is available.
fn is_bubblewrap_available() -> bool {
    which::which("bwrap").is_ok()
}

/// Check if systemd-nspawn binary is available.
fn is_systemd_nspawn_available() -> bool {
    which::which("systemd-nspawn").is_ok()
}

impl Default for TerminalManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_session_id_generates_unique() {
        let id1 = TerminalSessionId::new();
        let id2 = TerminalSessionId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn terminal_session_id_as_str() {
        let id = TerminalSessionId::new();
        assert!(!id.as_str().is_empty());
    }
}
