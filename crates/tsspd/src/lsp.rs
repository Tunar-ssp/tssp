//! Language Server Protocol (LSP) support for workspaces.
//!
//! `LSP` provides real autocomplete, diagnostics, go-to-definition, etc.
//! This module detects installed language servers and manages their lifecycle.

#![allow(dead_code)]

use std::collections::HashMap;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageServerAvailability {
    /// Language server binary is installed and configured.
    Available,
    /// Language server binary is not installed on the system.
    NotInstalled,
    /// Language server is not configured (no config entry exists).
    NotConfigured,
    /// Language is not supported by any configured server.
    Unsupported,
}

impl LanguageServerAvailability {
    /// Checks if a language server for the given language is available.
    pub fn for_language(language: &str) -> Self {
        let registry = LspRegistry::default();
        registry.availability_for(language)
    }
}

/// Language server binary and configuration.
#[derive(Debug, Clone)]
pub struct LspServerConfig {
    /// Programming language (e.g., "rust", "typescript", "python").
    pub language: String,
    /// Binary name or path (e.g., "rust-analyzer", "pyright").
    pub binary: String,
    /// Command-line arguments to start the server.
    pub args: Vec<String>,
}

/// Registry of available language servers.
pub struct LspRegistry {
    servers: HashMap<String, LspServerConfig>,
}

impl LspRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
        }
    }

    /// Creates a registry with detection of installed servers.
    pub fn detect() -> Self {
        let mut registry = Self::new();

        // Try to detect commonly installed language servers
        // These checks are non-blocking; missing servers don't halt the server

        // Rust analyzer
        if which::which("rust-analyzer").is_ok() {
            registry.servers.insert(
                "rust".to_string(),
                LspServerConfig {
                    language: "rust".to_string(),
                    binary: "rust-analyzer".to_string(),
                    args: vec![],
                },
            );
        }

        // TypeScript / JavaScript
        if which::which("typescript-language-server").is_ok() {
            registry.servers.insert(
                "typescript".to_string(),
                LspServerConfig {
                    language: "typescript".to_string(),
                    binary: "typescript-language-server".to_string(),
                    args: vec!["--stdio".to_string()],
                },
            );
            registry.servers.insert(
                "javascript".to_string(),
                LspServerConfig {
                    language: "javascript".to_string(),
                    binary: "typescript-language-server".to_string(),
                    args: vec!["--stdio".to_string()],
                },
            );
        }

        // Python
        if which::which("pylsp").is_ok() {
            registry.servers.insert(
                "python".to_string(),
                LspServerConfig {
                    language: "python".to_string(),
                    binary: "pylsp".to_string(),
                    args: vec![],
                },
            );
        }

        registry
    }

    /// Gets availability status for a language.
    pub fn availability_for(&self, language: &str) -> LanguageServerAvailability {
        match self.servers.get(language) {
            Some(config) => {
                // Check if the binary actually exists (double-check in case PATH changed)
                if which::which(&config.binary).is_ok() {
                    LanguageServerAvailability::Available
                } else {
                    LanguageServerAvailability::NotInstalled
                }
            }
            None => {
                // Language not in registry means it's not configured or supported
                LanguageServerAvailability::Unsupported
            }
        }
    }

    /// Gets the server config for a language.
    pub fn get(&self, language: &str) -> Option<&LspServerConfig> {
        self.servers.get(language)
    }

    /// Lists available language servers.
    pub fn available_languages(&self) -> Vec<String> {
        self.servers
            .iter()
            .filter(|(_lang, config)| which::which(&config.binary).is_ok())
            .map(|(lang, _)| lang.clone())
            .collect()
    }
}

impl Default for LspRegistry {
    fn default() -> Self {
        Self::detect()
    }
}

/// LSP session (foundation only, not implemented).
#[derive(Debug, Clone)]
pub struct LspSession {
    /// Workspace identifier.
    pub workspace_id: String,
    /// Programming language.
    pub language: String,
}

/// LSP manager for workspace language servers.
pub struct LspManager {
    /// Registry of available language servers.
    registry: LspRegistry,
}

impl LspManager {
    /// Creates a new LSP manager with server detection.
    pub fn new() -> Self {
        Self {
            registry: LspRegistry::detect(),
        }
    }

    /// Gets availability status for a language in this workspace.
    pub fn status_for_language(&self, language: &str) -> LanguageServerAvailability {
        self.registry.availability_for(language)
    }

    /// Lists available language servers detected on this system.
    pub fn available_languages(&self) -> Vec<String> {
        self.registry.available_languages()
    }

    /// Creates an LSP session (currently deferred, but availability is real).
    pub fn create_session(
        &self,
        _workspace_id: &str,
        language: &str,
    ) -> Result<LspSession, LspError> {
        // Check if server is available
        match self.status_for_language(language) {
            LanguageServerAvailability::Available => {
                // Availability is detected; actual session proxy not yet implemented
                Err(LspError::Unavailable(
                    "LSP session proxy not yet implemented; language server available but connection deferred"
                        .to_string(),
                ))
            }
            LanguageServerAvailability::NotInstalled => Err(LspError::Unavailable(format!(
                "language server for {language} is configured but not installed"
            ))),
            LanguageServerAvailability::NotConfigured => Err(LspError::Unavailable(format!(
                "language server for {language} is not configured"
            ))),
            LanguageServerAvailability::Unsupported => Err(LspError::Unavailable(format!(
                "language {language} is not supported"
            ))),
        }
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
    fn lsp_manager_detects_availability() {
        let manager = LspManager::new();
        // The actual availability depends on what's installed
        // but we can verify the detection works without panicking
        let _langs = manager.available_languages();
    }

    #[test]
    fn lsp_registry_unknown_language_unsupported() {
        let registry = LspRegistry::new();
        let avail = registry.availability_for("unknown_language_xyz_123");
        assert_eq!(avail, LanguageServerAvailability::Unsupported);
    }

    #[test]
    fn lsp_registry_empty_available_languages() {
        let registry = LspRegistry::new();
        let langs = registry.available_languages();
        // Empty registry should have no available languages
        assert_eq!(langs.len(), 0);
    }

    #[test]
    fn lsp_manager_create_session_reports_unavailable() {
        let manager = LspManager::new();
        let result = manager.create_session("ws-1", "unknown_language");
        assert!(result.is_err());
    }

    #[test]
    fn lsp_server_config_stores_language_binary() {
        let config = LspServerConfig {
            language: "rust".to_string(),
            binary: "rust-analyzer".to_string(),
            args: vec![],
        };
        assert_eq!(config.language, "rust");
        assert_eq!(config.binary, "rust-analyzer");
    }
}
