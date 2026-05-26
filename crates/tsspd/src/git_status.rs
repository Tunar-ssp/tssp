//! Git repository status detection for workspaces.
//!
//! Detects if workspace root is a git repository and provides
//! branch name, status, and changed files.

use serde::Serialize;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct GitStatus {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub changed_count: u32,
    pub staged_count: u32,
    pub untracked_count: u32,
}

/// Detects git repository and retrieves status.
pub fn detect_git_status(workspace_root: &Path) -> GitStatus {
    // Check if .git directory exists
    let git_dir = workspace_root.join(".git");
    if !git_dir.exists() {
        return GitStatus {
            is_repo: false,
            branch: None,
            changed_count: 0,
            staged_count: 0,
            untracked_count: 0,
        };
    }

    let branch = get_current_branch(workspace_root);
    let (changed, staged, untracked) = get_status_counts(workspace_root);

    GitStatus {
        is_repo: true,
        branch,
        changed_count: changed,
        staged_count: staged,
        untracked_count: untracked,
    }
}

fn get_current_branch(workspace_root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(workspace_root)
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        None
    }
}

fn get_status_counts(workspace_root: &Path) -> (u32, u32, u32) {
    let Ok(output) = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(workspace_root)
        .output()
    else {
        return (0, 0, 0);
    };

    if !output.status.success() {
        return (0, 0, 0);
    }

    let status_text = String::from_utf8_lossy(&output.stdout);
    let mut changed = 0u32;
    let mut staged = 0u32;
    let mut untracked = 0u32;

    for line in status_text.lines() {
        if line.starts_with("??") {
            untracked += 1;
        } else if line.starts_with(['M', 'A', 'D']) {
            staged += 1;
        } else if line.ends_with(['M', 'A', 'D']) {
            changed += 1;
        }
    }

    (changed, staged, untracked)
}

/// Handler for workspace git status with timeout.
pub async fn git_status_handler(workspace_root: &Path) -> Result<GitStatus, String> {
    if !workspace_root.exists() {
        return Err("workspace root not found".to_owned());
    }

    // Run git detection in a blocking task with 5-second timeout
    let workspace_root = workspace_root.to_path_buf();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tokio::task::spawn_blocking(move || detect_git_status(&workspace_root)),
    )
    .await;

    match result {
        Ok(Ok(status)) => Ok(status),
        Ok(Err(_)) => Err("failed to detect git status".to_owned()),
        Err(_) => Err("git status detection timed out".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_non_git_directory() {
        let status = detect_git_status(Path::new("/tmp"));
        assert!(!status.is_repo);
        assert!(status.branch.is_none());
    }

    #[test]
    fn git_status_structure() {
        let status = GitStatus {
            is_repo: false,
            branch: None,
            changed_count: 0,
            staged_count: 0,
            untracked_count: 0,
        };
        assert_eq!(status.changed_count, 0);
    }

    #[tokio::test]
    async fn git_status_handler_rejects_missing_workspace() {
        let result = git_status_handler(Path::new("/nonexistent/path")).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
