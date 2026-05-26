//! Workspace feature detection and capabilities.
//!
//! Terminal and LSP support detection without exposing unsafe implementations.

#![allow(dead_code)]

use serde::Serialize;
pub use tssp_domain::{SandboxStrategy, TerminalCapability};

/// Detects available sandbox on the system.
pub fn detect_sandbox() -> SandboxStrategy {
    // Check for bubblewrap first (preferred, lighter weight)
    if which::which("bwrap").is_ok() {
        return SandboxStrategy::Bubblewrap;
    }

    // Check for systemd-nspawn (heavier but more capable)
    if which::which("systemd-nspawn").is_ok() {
        return SandboxStrategy::Systemd;
    }

    // No sandbox available
    SandboxStrategy::None
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
}
