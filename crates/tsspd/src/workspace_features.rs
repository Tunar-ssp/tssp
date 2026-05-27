//! Workspace feature detection and capabilities.
//!
//! Terminal and LSP support detection without exposing unsafe implementations.

#![allow(dead_code)]

use serde::Serialize;
#[allow(unused_imports)]
pub use tssp_domain::{LspCapability, SandboxStrategy, TerminalCapability};

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
