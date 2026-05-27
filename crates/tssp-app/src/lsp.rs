use std::sync::Arc;
use tssp_domain::{LanguageServerAvailability, LspCapability, LspError, LspServerConfig};
use tssp_ports::LspProvider;

/// Service for managing workspace LSP capabilities.
pub struct LspService {
    provider: Arc<dyn LspProvider>,
}

impl LspService {
    /// Create a new LSP service.
    pub fn new(provider: Arc<dyn LspProvider>) -> Self {
        Self { provider }
    }

    /// Get the status of a language server.
    pub fn status_for_language(&self, language: &str) -> LspCapability {
        match self.provider.status_for_language(language) {
            LanguageServerAvailability::Available => LspCapability::Available {
                language: language.to_string(),
            },
            LanguageServerAvailability::NotInstalled => LspCapability::Unavailable {
                reason: format!("language server for {language} is not installed"),
            },
            LanguageServerAvailability::NotConfigured => LspCapability::Unavailable {
                reason: format!("language server for {language} is not configured"),
            },
            LanguageServerAvailability::Unsupported => LspCapability::Unavailable {
                reason: format!("language {language} is not supported"),
            },
        }
    }

    /// List all available language servers.
    pub fn available_languages(&self) -> Vec<String> {
        self.provider
            .detect_servers()
            .into_iter()
            .map(|s| s.language)
            .collect()
    }

    /// Get server config for a language.
    pub fn get_config(&self, language: &str) -> Result<LspServerConfig, LspError> {
        self.provider
            .get_config(language)
            .ok_or_else(|| LspError::ServerNotFound {
                language: language.to_string(),
            })
    }
}
