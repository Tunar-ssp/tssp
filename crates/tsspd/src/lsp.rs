//! Language Server Protocol (LSP) support for workspaces.
//!
//! LSP provides real autocomplete, diagnostics, go-to-definition, etc.
//! This module provides the foundation; real language server integration is deferred.

#![allow(dead_code)]

use thiserror::Error;

/// LSP operation errors.
#[derive(Debug, Error)]
pub enum LspError {
    #[error("language server not found for {language}")]
    ServerNotFound { language: String },
    #[error("lsp unavailable: {0}")]
    Unavailable(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Language server binary availability check.
#[derive(Debug, Clone, Copy)]
pub enum LanguageServerAvailability {
    Available,
    NotInstalled,
    NotConfigured,
}

impl LanguageServerAvailability {
    /// Checks if a language server for the given language is available.
    pub fn for_language(_language: &str) -> Self {
        // Real implementation would check if the language server binary exists
        // For now, return NotConfigured (foundation only)
        Self::NotConfigured
    }
}

/// Language server configuration.
#[derive(Debug, Clone)]
pub struct LspServerConfig {
    pub language: String,
    pub binary: String,
    pub args: Vec<String>,
}

/// LSP session (foundation only, not implemented).
#[derive(Debug, Clone)]
pub struct LspSession {
    pub workspace_id: String,
    pub language: String,
}

/// LSP manager (foundation only).
pub struct LspManager {
    // Configuration would be stored here
}

impl LspManager {
    /// Creates a new LSP manager.
    pub fn new() -> Self {
        Self {}
    }

    /// Gets availability status for a language.
    pub fn status_for_language(&self, _language: &str) -> LanguageServerAvailability {
        // Real implementation would check config and installed servers
        LanguageServerAvailability::NotConfigured
    }

    /// Lists available language servers.
    pub fn available_languages(&self) -> Vec<String> {
        // Real implementation would read from config
        vec![]
    }

    /// Creates an LSP session (currently deferred).
    pub async fn create_session(
        &self,
        _workspace_id: &str,
        _language: &str,
    ) -> Result<LspSession, LspError> {
        Err(LspError::Unavailable(
            "LSP sessions not yet implemented; foundation in place".to_string(),
        ))
    }
}

impl Default for LspManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_server_availability_not_configured() {
        let avail = LanguageServerAvailability::for_language("rust");
        assert!(matches!(avail, LanguageServerAvailability::NotConfigured));
    }

    #[test]
    fn lsp_manager_default() {
        let _manager = LspManager::default();
    }
}
