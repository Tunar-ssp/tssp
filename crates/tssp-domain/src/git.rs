use serde::Serialize;

/// Git repository status.
#[derive(Debug, Clone, Serialize)]
pub struct GitStatus {
    /// Whether the workspace root is a git repository.
    pub is_repo: bool,
    /// Current branch name if in a repo.
    pub branch: Option<String>,
    /// Number of modified files.
    pub changed_count: u32,
    /// Number of staged files.
    pub staged_count: u32,
    /// Number of untracked files.
    pub untracked_count: u32,
}
