use serde::Serialize;
use thiserror::Error;

/// LSP operation errors.
#[derive(Debug, Error)]
pub enum LspError {
    /// Language server not found for the specified language.
    #[error("language server not found for {language}")]
    ServerNotFound {
        /// The programming language.
        language: String,
    },
    /// LSP is unavailable with a reason.
    #[error("lsp unavailable: {0}")]
    Unavailable(String),
    /// User lacks required permissions.
    #[error("unauthorized")]
    Unauthorized,
    /// I/O error during LSP operations.
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

/// Workspace LSP capability status.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum LspCapability {
    /// Language server for this language is available.
    Available {
        /// The language of the server.
        language: String,
    },
    /// LSP is disabled in config.
    Disabled,
    /// Language server not installed or found.
    Unavailable {
        /// Reason for unavailability.
        reason: String,
    },
    /// User lacks required permissions.
    Forbidden,
}
