//! Workspace feature detection and capabilities.
//!
//! Terminal and LSP support detection without exposing unsafe implementations.

#![allow(dead_code)]

use serde::Serialize;

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

/// Workspace LSP capability status.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum LspCapability {
    /// Language server for this language is available.
    Available { language: String },
    /// LSP is disabled in config.
    Disabled,
    /// Language server not installed or found.
    Unavailable { reason: String },
    /// User lacks required permissions.
    Forbidden,
}

/// Terminal sandbox implementation strategy.
#[derive(Debug, Clone, Copy)]
pub enum SandboxStrategy {
    /// Use bubblewrap if available.
    Bubblewrap,
    /// Use systemd-nspawn if available.
    Systemd,
    /// No sandbox, terminal unavailable.
    None,
}

impl SandboxStrategy {
    /// Detects available sandbox on the system.
    pub fn detect() -> Self {
        // Check for bubblewrap first (preferred, lighter weight)
        if which::which("bwrap").is_ok() {
            return Self::Bubblewrap;
        }

        // Check for systemd-nspawn (heavier but more capable)
        if which::which("systemd-nspawn").is_ok() {
            return Self::Systemd;
        }

        // No sandbox available
        Self::None
    }

    /// Returns true if sandbox is available.
    pub fn is_available(self) -> bool {
        !matches!(self, Self::None)
    }
}

/// Workspace feature capabilities report.
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceCapabilities {
    pub terminal: TerminalCapability,
    pub lsp_available: Vec<String>, // list of available language servers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_capability_comparison() {
        assert_eq!(TerminalCapability::Available, TerminalCapability::Available);
        assert_ne!(TerminalCapability::Available, TerminalCapability::Disabled);
    }

    #[test]
    fn sandbox_strategy_none_not_available() {
        assert!(!SandboxStrategy::None.is_available());
    }
}
