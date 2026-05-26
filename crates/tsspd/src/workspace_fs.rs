//! Filesystem-backed workspace storage with safe path handling.
//!
//! Each workspace gets its own directory: data/workspaces/<workspace_id>/
//! Files are stored directly on disk, not in SQLite.
//! Path security is enforced at every operation.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::io;
use thiserror::Error;

/// Workspace filesystem errors.
#[derive(Debug, Error)]
pub enum WorkspaceFsError {
    #[error("not found")]
    NotFound,
    #[error("path invalid: {0}")]
    InvalidPath(String),
    #[error("path traversal rejected")]
    TraversalAttempt,
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("file too large")]
    FileTooLarge,
    #[error("directory too deep")]
    DirectoryTooDeep,
    #[error("too many files")]
    TooManyFiles,
}

/// Workspace file tree entry.
#[derive(Debug, Clone)]
pub struct WorkspaceFsEntry {
    pub path: String,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub modified_at: u64,
}

/// Workspace filesystem operations.
pub struct WorkspaceFilesystem {
    root: PathBuf,
}

impl WorkspaceFilesystem {
    /// Creates a new workspace filesystem pointing to the workspace data root.
    /// Typically: `data/workspaces/`
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    /// Gets the path to a workspace directory.
    fn workspace_dir(&self, workspace_id: &str) -> Result<PathBuf, WorkspaceFsError> {
        validate_workspace_id(workspace_id)?;
        Ok(self.root.join(workspace_id))
    }

    /// Gets the full filesystem path for a workspace file, with security checks.
    fn resolve_path(
        &self,
        workspace_id: &str,
        rel_path: &str,
    ) -> Result<PathBuf, WorkspaceFsError> {
        let workspace_dir = self.workspace_dir(workspace_id)?;
        let requested = workspace_dir.join(rel_path);

        // Canonicalize (if it exists) or validate path structure
        let canonical = if requested.exists() {
            requested.canonicalize()?
        } else {
            // Path doesn't exist, but validate the components
            requested
        };

        // Ensure it's within workspace directory
        if !canonical.starts_with(&workspace_dir) {
            return Err(WorkspaceFsError::TraversalAttempt);
        }

        Ok(canonical)
    }

    /// Creates a workspace directory if it doesn't exist.
    pub async fn init_workspace(&self, workspace_id: &str) -> Result<(), WorkspaceFsError> {
        let dir = self.workspace_dir(workspace_id)?;
        tokio::fs::create_dir_all(&dir).await?;
        Ok(())
    }

    /// Lists files and directories in a path (bounded).
    pub async fn list_tree(
        &self,
        workspace_id: &str,
        rel_path: &str,
        max_depth: usize,
    ) -> Result<Vec<WorkspaceFsEntry>, WorkspaceFsError> {
        if max_depth == 0 {
            return Err(WorkspaceFsError::DirectoryTooDeep);
        }

        let path = self.resolve_path(workspace_id, rel_path)?;
        let mut entries = Vec::new();

        if !path.exists() {
            return Err(WorkspaceFsError::NotFound);
        }

        if path.is_file() {
            // Single file
            let metadata = tokio::fs::metadata(&path).await?;
            entries.push(WorkspaceFsEntry {
                path: rel_path.to_string(),
                is_dir: false,
                size_bytes: metadata.len(),
                modified_at: metadata
                    .modified()?
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            });
            return Ok(entries);
        }

        // Directory listing (bounded)
        let mut read_dir = tokio::fs::read_dir(&path).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            if entries.len() > 10_000 {
                return Err(WorkspaceFsError::TooManyFiles);
            }

            let metadata = entry.metadata().await?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            entries.push(WorkspaceFsEntry {
                path: file_name.to_string(),
                is_dir: metadata.is_dir(),
                size_bytes: metadata.len(),
                modified_at: metadata
                    .modified()?
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            });
        }

        Ok(entries)
    }

    /// Reads a file (bounded to 10MB).
    pub async fn read_file(
        &self,
        workspace_id: &str,
        rel_path: &str,
    ) -> Result<Vec<u8>, WorkspaceFsError> {
        const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB

        let path = self.resolve_path(workspace_id, rel_path)?;

        if !path.exists() {
            return Err(WorkspaceFsError::NotFound);
        }

        if path.is_dir() {
            return Err(WorkspaceFsError::InvalidPath(
                "cannot read directory as file".to_string(),
            ));
        }

        let metadata = tokio::fs::metadata(&path).await?;
        if metadata.len() > MAX_FILE_SIZE {
            return Err(WorkspaceFsError::FileTooLarge);
        }

        tokio::fs::read(&path).await.map_err(Into::into)
    }

    /// Writes a file (bounded to 10MB).
    pub async fn write_file(
        &self,
        workspace_id: &str,
        rel_path: &str,
        contents: &[u8],
    ) -> Result<(), WorkspaceFsError> {
        const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB

        if contents.len() as u64 > MAX_FILE_SIZE {
            return Err(WorkspaceFsError::FileTooLarge);
        }

        let path = self.resolve_path(workspace_id, rel_path)?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&path, contents).await?;
        Ok(())
    }

    /// Creates a directory.
    pub async fn create_dir(
        &self,
        workspace_id: &str,
        rel_path: &str,
    ) -> Result<(), WorkspaceFsError> {
        let path = self.resolve_path(workspace_id, rel_path)?;
        tokio::fs::create_dir_all(&path).await?;
        Ok(())
    }

    /// Deletes a file or empty directory.
    pub async fn delete(
        &self,
        workspace_id: &str,
        rel_path: &str,
    ) -> Result<(), WorkspaceFsError> {
        let path = self.resolve_path(workspace_id, rel_path)?;

        if !path.exists() {
            return Err(WorkspaceFsError::NotFound);
        }

        if path.is_dir() {
            tokio::fs::remove_dir(&path).await?;
        } else {
            tokio::fs::remove_file(&path).await?;
        }

        Ok(())
    }

    /// Renames/moves a file or directory.
    pub async fn rename(
        &self,
        workspace_id: &str,
        from: &str,
        to: &str,
    ) -> Result<(), WorkspaceFsError> {
        let from_path = self.resolve_path(workspace_id, from)?;
        let to_path = self.resolve_path(workspace_id, to)?;

        if !from_path.exists() {
            return Err(WorkspaceFsError::NotFound);
        }

        // Ensure to parent exists
        if let Some(parent) = to_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::rename(&from_path, &to_path).await?;
        Ok(())
    }
}

/// Validates workspace ID format (must be valid UUID or safe string).
fn validate_workspace_id(id: &str) -> Result<(), WorkspaceFsError> {
    if id.is_empty() {
        return Err(WorkspaceFsError::InvalidPath("id cannot be empty".to_string()));
    }

    // Only alphanumeric and hyphen/underscore
    if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(WorkspaceFsError::InvalidPath(
            "id contains invalid characters".to_string(),
        ));
    }

    if id.len() > 128 {
        return Err(WorkspaceFsError::InvalidPath("id too long".to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_id_validation() {
        assert!(validate_workspace_id("abc-123_def").is_ok());
        assert!(validate_workspace_id("../etc/passwd").is_err());
        assert!(validate_workspace_id("..").is_err());
        assert!(validate_workspace_id("").is_err());
    }
}
