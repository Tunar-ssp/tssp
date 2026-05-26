//! Real sandboxed terminal support for workspaces.
//!
//! Terminal is admin-only and requires a sandbox to be configured.
//! Does NOT provide unsafe host shell access.

#![allow(dead_code)]

use serde::Serialize;
use thiserror::Error;

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

/// Terminal session manager (foundation only, not implemented yet).
pub struct TerminalManager {
    // Sessions would be stored here
}

impl TerminalManager {
    /// Creates a new terminal manager.
    pub fn new() -> Self {
        Self {}
    }

    /// Creates a new terminal session (currently returns unavailable).
    pub async fn create_session(
        &self,
        _workspace_id: &str,
        _user_id: &str,
        _config: TerminalConfig,
    ) -> Result<TerminalSession, TerminalError> {
        // Real implementation deferred
        Err(TerminalError::Unavailable(
            "terminal sessions not yet implemented; foundation in place".to_string(),
        ))
    }

    /// Gets an existing terminal session.
    pub async fn get_session(
        &self,
        _session_id: &TerminalSessionId,
    ) -> Result<TerminalSession, TerminalError> {
        Err(TerminalError::SessionNotFound)
    }

    /// Closes a terminal session.
    pub async fn close_session(&self, _session_id: &TerminalSessionId) -> Result<(), TerminalError> {
        Err(TerminalError::SessionNotFound)
    }
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
