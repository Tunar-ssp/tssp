use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use thiserror::Error;

/// Workspace terminal capability status.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum TerminalCapability {
    /// Terminal is enabled and sandbox is configured.
    Available,
    /// Terminal feature is disabled in config.
    Disabled,
    /// Sandbox binary/setup is missing or unavailable.
    UnavailableSandbox,
    /// User lacks required permissions (admin).
    Forbidden,
}

/// Terminal sandbox implementation strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxStrategy {
    /// Use bubblewrap if available.
    Bubblewrap,
    /// Use systemd-nspawn if available.
    Systemd,
    /// No sandbox, terminal unavailable.
    None,
}

impl SandboxStrategy {
    /// Returns true if sandbox is available.
    #[must_use]
    pub fn is_available(self) -> bool {
        !matches!(self, Self::None)
    }
}

/// Terminal operation errors.
#[derive(Debug, Error)]
pub enum TerminalError {
    /// Terminal is not available with a reason.
    #[error("terminal not available: {0}")]
    Unavailable(String),
    /// User is not authorized to use the terminal.
    #[error("unauthorized")]
    Unauthorized,
    /// I/O error during terminal operations.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// The specified terminal session was not found.
    #[error("session not found")]
    SessionNotFound,
    /// The terminal session is already closed.
    #[error("session closed")]
    SessionClosed,
    /// The terminal input is invalid.
    #[error("invalid input")]
    InvalidInput,
}

/// Terminal session ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TerminalSessionId(String);

impl TerminalSessionId {
    /// Create a new terminal session ID from a string.
    #[must_use]
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Return the string representation of the session ID.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Terminal session configuration.
#[derive(Debug, Clone)]
pub struct TerminalConfig {
    /// Workspace directory to start in (must be validated).
    pub workspace_dir: std::path::PathBuf,
    /// Sandbox strategy to use.
    pub sandbox: SandboxStrategy,
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
    /// Unique session identifier.
    pub id: TerminalSessionId,
    /// Workspace ID the session belongs to.
    pub workspace_id: String,
    /// User ID who owns the session.
    pub user_id: String,
    /// When the session was created.
    pub created_at: SystemTime,
    /// When the last activity occurred in the session.
    pub last_activity: SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_strategy_none_not_available() {
        assert!(!SandboxStrategy::None.is_available());
    }

    #[test]
    fn sandbox_strategy_comparison() {
        assert_eq!(SandboxStrategy::Bubblewrap, SandboxStrategy::Bubblewrap);
        assert_ne!(SandboxStrategy::Bubblewrap, SandboxStrategy::None);
    }
}
