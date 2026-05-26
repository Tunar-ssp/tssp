//! Filesystem-backed workspace storage with safe path handling.
//!
//! Each workspace gets its own directory: data/workspaces/`<workspace_id>`/
//! Files are stored directly on disk, not in `SQLite`.
//!
//! # Security
//!
//! Path security is enforced at every operation using lexical path normalization:
//! - Absolute paths are rejected
//! - Paths with `..` segments are validated to never escape the workspace root
//! - Null bytes are rejected
//! - Symlink escapes are prevented via canonicalization of existing paths
//! - For non-existent paths, the nearest existing parent is canonicalized to verify safety

use std::path::{Path, PathBuf};

use tssp_ports::{WorkspaceFileEntry, WorkspaceFileStore, WorkspaceFileStoreError};

/// Filesystem-backed workspace file store.
#[derive(Debug, Clone)]
pub struct FilesystemWorkspaceFileStore {
    root: PathBuf,
}

impl FilesystemWorkspaceFileStore {
    /// Creates a new workspace filesystem pointing to the workspace data root.
    /// Typically: `data/workspaces/`
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    /// Gets the path to a workspace directory.
    fn workspace_dir(&self, workspace_id: &str) -> Result<PathBuf, WorkspaceFileStoreError> {
        validate_workspace_id(workspace_id)?;
        Ok(self.root.join(workspace_id))
    }

    /// Gets the full filesystem path for a workspace file, with security checks.
    ///
    /// Implements safe lexical path normalization to prevent directory traversal attacks.
    /// Empty paths and `.` resolve to the workspace root.
    /// The function processes paths in stages:
    ///
    /// 1. Validation: Rejects null bytes, absolute paths
    /// 2. Lexical normalization: Resolves `.` and `..` without filesystem access
    ///    - Rejects if `..` would escape the workspace root
    /// 3. Filesystem verification:
    ///    - For existing paths: Canonicalizes and verifies they stay within workspace
    ///    - For non-existent paths: Canonicalizes nearest existing parent and verifies safety
    ///
    /// This multi-layered approach prevents both common traversal attacks (`../evil`)
    /// and edge cases like symlink escapes.
    fn resolve_path(
        &self,
        workspace_id: &str,
        rel_path: &str,
    ) -> Result<PathBuf, WorkspaceFileStoreError> {
        let workspace_dir = self.workspace_dir(workspace_id)?;

        if rel_path.contains('\0') {
            return Err(WorkspaceFileStoreError::InvalidPath(
                "null byte in path".to_string(),
            ));
        }

        if rel_path.starts_with('/') {
            return Err(WorkspaceFileStoreError::InvalidPath(
                "absolute paths rejected".to_string(),
            ));
        }

        let rel_path = rel_path.trim();
        if rel_path.is_empty() || rel_path == "." {
            return Ok(workspace_dir);
        }

        let mut normalized = Vec::new();
        for component in rel_path.split('/') {
            match component {
                "" | "." => {}
                ".." => {
                    if normalized.is_empty() {
                        return Err(WorkspaceFileStoreError::TraversalAttempt);
                    }
                    normalized.pop();
                }
                name => {
                    if name.contains('/') || name.contains('\0') {
                        return Err(WorkspaceFileStoreError::InvalidPath(
                            "invalid path component".to_string(),
                        ));
                    }
                    normalized.push(name);
                }
            }
        }

        let mut requested = workspace_dir.clone();
        for component in normalized {
            requested.push(component);
        }

        if requested.exists() {
            let canonical = requested.canonicalize()?;
            let canonical_workspace = workspace_dir.canonicalize()?;
            if !canonical.starts_with(&canonical_workspace) {
                return Err(WorkspaceFileStoreError::TraversalAttempt);
            }
            Ok(canonical)
        } else {
            // For non-existent paths, find the nearest existing parent and verify it's within workspace
            let mut check_path = requested.clone();
            loop {
                if check_path == workspace_dir {
                    // Parent chain leads back to workspace root - safe
                    return Ok(requested);
                }
                if check_path.pop() {
                    if check_path.exists() {
                        // Found an existing parent - verify it's within workspace
                        let canonical_parent = check_path.canonicalize()?;
                        let canonical_workspace = workspace_dir.canonicalize()?;
                        if !canonical_parent.starts_with(&canonical_workspace) {
                            return Err(WorkspaceFileStoreError::TraversalAttempt);
                        }
                        return Ok(requested);
                    }
                } else {
                    return Err(WorkspaceFileStoreError::TraversalAttempt);
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl WorkspaceFileStore for FilesystemWorkspaceFileStore {
    async fn init_workspace(&self, workspace_id: &str) -> Result<(), WorkspaceFileStoreError> {
        let dir = self.workspace_dir(workspace_id)?;
        tokio::fs::create_dir_all(&dir).await?;
        Ok(())
    }

    async fn list_tree(
        &self,
        workspace_id: &str,
        rel_path: &str,
        max_depth: usize,
    ) -> Result<Vec<WorkspaceFileEntry>, WorkspaceFileStoreError> {
        if max_depth == 0 {
            return Err(WorkspaceFileStoreError::DirectoryTooDeep);
        }

        let path = self.resolve_path(workspace_id, rel_path)?;
        let workspace_dir = self.workspace_dir(workspace_id)?;

        // Create workspace directory if it doesn't exist (empty workspace)
        if rel_path.trim().is_empty() || rel_path == "." {
            tokio::fs::create_dir_all(&workspace_dir).await?;
        }

        let mut entries = Vec::new();

        if !path.exists() {
            return Err(WorkspaceFileStoreError::NotFound);
        }

        if path.is_file() {
            let metadata = tokio::fs::metadata(&path).await?;
            entries.push(WorkspaceFileEntry {
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

        let mut read_dir = tokio::fs::read_dir(&path).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            if entries.len() > 10_000 {
                return Err(WorkspaceFileStoreError::TooManyFiles);
            }

            let metadata = entry.metadata().await?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            entries.push(WorkspaceFileEntry {
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

    async fn read_file(
        &self,
        workspace_id: &str,
        rel_path: &str,
    ) -> Result<Vec<u8>, WorkspaceFileStoreError> {
        const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

        let path = self.resolve_path(workspace_id, rel_path)?;

        if !path.exists() {
            return Err(WorkspaceFileStoreError::NotFound);
        }

        if path.is_dir() {
            return Err(WorkspaceFileStoreError::InvalidPath(
                "cannot read directory as file".to_string(),
            ));
        }

        let metadata = tokio::fs::metadata(&path).await?;
        if metadata.len() > MAX_FILE_SIZE {
            return Err(WorkspaceFileStoreError::FileTooLarge);
        }

        tokio::fs::read(&path).await.map_err(Into::into)
    }

    async fn write_file(
        &self,
        workspace_id: &str,
        rel_path: &str,
        contents: &[u8],
    ) -> Result<(), WorkspaceFileStoreError> {
        const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

        if contents.len() as u64 > MAX_FILE_SIZE {
            return Err(WorkspaceFileStoreError::FileTooLarge);
        }

        let path = self.resolve_path(workspace_id, rel_path)?;

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&path, contents).await?;
        Ok(())
    }

    async fn create_dir(
        &self,
        workspace_id: &str,
        rel_path: &str,
    ) -> Result<(), WorkspaceFileStoreError> {
        let path = self.resolve_path(workspace_id, rel_path)?;
        tokio::fs::create_dir_all(&path).await?;
        Ok(())
    }

    async fn delete(
        &self,
        workspace_id: &str,
        rel_path: &str,
    ) -> Result<(), WorkspaceFileStoreError> {
        let path = self.resolve_path(workspace_id, rel_path)?;

        if !path.exists() {
            return Err(WorkspaceFileStoreError::NotFound);
        }

        if path.is_dir() {
            tokio::fs::remove_dir(&path).await?;
        } else {
            tokio::fs::remove_file(&path).await?;
        }

        Ok(())
    }

    async fn rename(
        &self,
        workspace_id: &str,
        from: &str,
        to: &str,
    ) -> Result<(), WorkspaceFileStoreError> {
        let from_path = self.resolve_path(workspace_id, from)?;
        let to_path = self.resolve_path(workspace_id, to)?;

        if !from_path.exists() {
            return Err(WorkspaceFileStoreError::NotFound);
        }

        if let Some(parent) = to_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::rename(&from_path, &to_path).await?;
        Ok(())
    }
}

/// Validates workspace ID format (must be valid UUID or safe string).
fn validate_workspace_id(id: &str) -> Result<(), WorkspaceFileStoreError> {
    if id.is_empty() {
        return Err(WorkspaceFileStoreError::InvalidPath(
            "id cannot be empty".to_string(),
        ));
    }

    if !id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(WorkspaceFileStoreError::InvalidPath(
            "id contains invalid characters".to_string(),
        ));
    }

    if id.len() > 128 {
        return Err(WorkspaceFileStoreError::InvalidPath(
            "id too long".to_string(),
        ));
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

    #[test]
    fn resolve_path_rejects_traversal_with_dotdot() {
        let fs = FilesystemWorkspaceFileStore::new("/tmp/test");
        assert!(fs
            .resolve_path("ws1", "../evil")
            .is_err_and(|e| matches!(e, WorkspaceFileStoreError::TraversalAttempt)));
    }

    #[test]
    fn resolve_path_rejects_nested_traversal() {
        let fs = FilesystemWorkspaceFileStore::new("/tmp/test");
        assert!(fs
            .resolve_path("ws1", "a/../../etc/passwd")
            .is_err_and(|e| matches!(e, WorkspaceFileStoreError::TraversalAttempt)));
    }

    #[test]
    fn resolve_path_rejects_absolute_path() {
        let fs = FilesystemWorkspaceFileStore::new("/tmp/test");
        assert!(fs
            .resolve_path("ws1", "/etc/passwd")
            .is_err_and(|e| matches!(e, WorkspaceFileStoreError::InvalidPath(_))));
    }

    #[test]
    fn resolve_path_rejects_null_byte() {
        let fs = FilesystemWorkspaceFileStore::new("/tmp/test");
        assert!(fs
            .resolve_path("ws1", "file\0name")
            .is_err_and(|e| matches!(e, WorkspaceFileStoreError::InvalidPath(_))));
    }

    #[test]
    fn resolve_path_accepts_safe_relative_path() {
        let fs = FilesystemWorkspaceFileStore::new("/tmp/test");
        let result = fs.resolve_path("ws1", "subdir/file.txt");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("ws1"));
        assert!(path.to_string_lossy().contains("subdir/file.txt"));
    }

    #[test]
    fn resolve_path_normalizes_dots() {
        let fs = FilesystemWorkspaceFileStore::new("/tmp/test");
        let result = fs.resolve_path("ws1", "a/./b/../c");
        assert!(result.is_ok());
        let path = result.unwrap();
        let s = path.to_string_lossy();
        assert!(s.contains("ws1"));
        assert!(s.contains("a/c"));
    }
}
