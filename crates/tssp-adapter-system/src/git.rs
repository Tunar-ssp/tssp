//! Git provider backed by the `git2` native crate.
//!
//! Reads `.git` directory metadata without spawning a child process —
//! significantly faster and more reliable than parsing `git` CLI output,
//! especially on Orange Pi where process spawning is expensive.

use std::path::Path;
use tssp_domain::GitStatus;
use tssp_ports::GitProvider;

/// System implementation of Git provider using the `git2` crate.
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

        tokio::task::spawn_blocking(move || git_status_blocking(&workspace_root))
            .await
            .map_err(|e| e.to_string())?
    }
}

fn git_status_blocking(workspace_root: &Path) -> Result<GitStatus, String> {
    // Discover the repository from the workspace root (walks up if needed).
    let repo = match git2::Repository::discover(workspace_root) {
        Ok(repo) => repo,
        Err(e) if e.code() == git2::ErrorCode::NotFound => {
            return Ok(GitStatus {
                is_repo: false,
                branch: None,
                changed_count: 0,
                staged_count: 0,
                untracked_count: 0,
            });
        }
        Err(e) => return Err(format!("git discover failed: {e}")),
    };

    let branch = head_branch_name(&repo);

    let mut status_opts = git2::StatusOptions::new();
    status_opts
        .include_untracked(true)
        .recurse_untracked_dirs(false)
        .include_ignored(false)
        .renames_head_to_index(false)
        .renames_index_to_workdir(false);

    let statuses = repo
        .statuses(Some(&mut status_opts))
        .map_err(|e| format!("git status failed: {e}"))?;

    let mut changed = 0u32;
    let mut staged = 0u32;
    let mut untracked = 0u32;

    for entry in statuses.iter() {
        let flags = entry.status();

        if flags.contains(git2::Status::WT_NEW) {
            untracked += 1;
            continue;
        }

        // Index (staged) changes.
        if flags.intersects(
            git2::Status::INDEX_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_DELETED
                | git2::Status::INDEX_RENAMED
                | git2::Status::INDEX_TYPECHANGE,
        ) {
            staged += 1;
        }

        // Worktree (unstaged) changes.
        if flags.intersects(
            git2::Status::WT_MODIFIED
                | git2::Status::WT_DELETED
                | git2::Status::WT_TYPECHANGE
                | git2::Status::WT_RENAMED,
        ) {
            changed += 1;
        }
    }

    Ok(GitStatus {
        is_repo: true,
        branch,
        changed_count: changed,
        staged_count: staged,
        untracked_count: untracked,
    })
}

/// Returns the short name of the current HEAD branch (e.g. "main"), or `None` for detached HEAD.
fn head_branch_name(repo: &git2::Repository) -> Option<String> {
    let head = repo.head().ok()?;
    if head.is_branch() {
        head.shorthand().map(ToOwned::to_owned)
    } else {
        // Detached HEAD: return the abbreviated commit hash.
        let oid = head.target()?;
        Some(oid.to_string()[..7].to_owned())
    }
}
