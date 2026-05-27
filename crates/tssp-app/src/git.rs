use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tssp_domain::GitStatus;
use tssp_ports::GitProvider;

/// Service for retrieving git status for workspaces.
pub struct GitService {
    provider: Arc<dyn GitProvider>,
}

impl GitService {
    /// Create a new git service.
    pub fn new(provider: Arc<dyn GitProvider>) -> Self {
        Self { provider }
    }

    /// Get the git status with a timeout.
    ///
    /// # Errors
    ///
    /// Returns an error if the workspace root does not exist,
    /// if the git detection times out, or if the provider fails.
    pub async fn get_status(&self, workspace_root: &Path) -> Result<GitStatus, String> {
        if !workspace_root.exists() {
            return Err("workspace root not found".to_owned());
        }

        let result = tokio::time::timeout(
            Duration::from_secs(5),
            self.provider.get_status(workspace_root),
        )
        .await;

        match result {
            Ok(status) => status,
            Err(_) => Err("git status detection timed out".to_owned()),
        }
    }
}
