use std::path::Path;
use tssp_domain::GitStatus;

/// Port for retrieving git repository status.
#[async_trait::async_trait]
pub trait GitProvider: Send + Sync {
    /// Gets the git status for a workspace root.
    async fn get_status(&self, workspace_root: &Path) -> Result<GitStatus, String>;
}
