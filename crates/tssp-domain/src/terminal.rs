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

/// Terminal sandbox implementation strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
