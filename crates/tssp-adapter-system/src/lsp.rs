use tssp_domain::{LanguageServerAvailability, LspServerConfig};
use tssp_ports::LspProvider;
use std::collections::HashMap;

/// System implementation of LSP provider using `which` to detect binaries.
pub struct SystemLspProvider {
    static_configs: HashMap<String, LspServerConfig>,
}

impl SystemLspProvider {
    /// Create a new system LSP provider with default detection.
    #[must_use]
    pub fn new() -> Self {
        let mut configs = HashMap::new();

        configs.insert("rust".to_string(), LspServerConfig {
            language: "rust".to_string(),
            binary: "rust-analyzer".to_string(),
            args: vec![],
        });

        configs.insert("typescript".to_string(), LspServerConfig {
            language: "typescript".to_string(),
            binary: "typescript-language-server".to_string(),
            args: vec!["--stdio".to_string()],
        });

        configs.insert("javascript".to_string(), LspServerConfig {
            language: "javascript".to_string(),
            binary: "typescript-language-server".to_string(),
            args: vec!["--stdio".to_string()],
        });

        configs.insert("python".to_string(), LspServerConfig {
            language: "python".to_string(),
            binary: "pylsp".to_string(),
            args: vec![],
        });

        Self { static_configs: configs }
    }
}

impl Default for SystemLspProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LspProvider for SystemLspProvider {
    fn detect_servers(&self) -> Vec<LspServerConfig> {
        self.static_configs.values()
            .filter(|config| which::which(&config.binary).is_ok())
            .cloned()
            .collect()
    }

    fn status_for_language(&self, language: &str) -> LanguageServerAvailability {
        match self.static_configs.get(language) {
            Some(config) => {
                if which::which(&config.binary).is_ok() {
                    LanguageServerAvailability::Available
                } else {
                    LanguageServerAvailability::NotInstalled
                }
            }
            None => LanguageServerAvailability::Unsupported,
        }
    }

    fn get_config(&self, language: &str) -> Option<LspServerConfig> {
        self.static_configs.get(language).cloned()
    }
}
