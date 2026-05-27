use std::path::Path;
use std::process::Command;
use tssp_domain::GitStatus;
use tssp_ports::GitProvider;

/// System implementation of Git provider using the `git` binary.
pub struct SystemGitProvider;

impl SystemGitProvider {
    /// Create a new system git provider.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for SystemGitProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl GitProvider for SystemGitProvider {
    async fn get_status(&self, workspace_root: &Path) -> Result<GitStatus, String> {
        let workspace_root = workspace_root.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let git_dir = workspace_root.join(".git");
            if !git_dir.exists() {
                return Ok(GitStatus {
                    is_repo: false,
                    branch: None,
                    changed_count: 0,
                    staged_count: 0,
                    untracked_count: 0,
                });
            }

            let branch = get_current_branch(&workspace_root);
            let (changed, staged, untracked) = get_status_counts(&workspace_root);

            Ok(GitStatus {
                is_repo: true,
                branch,
                changed_count: changed,
                staged_count: staged,
                untracked_count: untracked,
            })
        })
        .await
        .map_err(|e| e.to_string())?
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
