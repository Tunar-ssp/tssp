use tssp_domain::{LanguageServerAvailability, LspServerConfig};

/// Port for detecting and managing language servers.
pub trait LspProvider: Send + Sync {
    /// Detects available language servers on the host system.
    fn detect_servers(&self) -> Vec<LspServerConfig>;

    /// Gets the availability status for a specific language.
    fn status_for_language(&self, language: &str) -> LanguageServerAvailability;

    /// Gets the server config for a language.
    fn get_config(&self, language: &str) -> Option<LspServerConfig>;
}
