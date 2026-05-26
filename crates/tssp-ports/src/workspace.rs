//! Workspace filesystem port.
//!
//! Trait-based interface for workspace file operations.
//! Implementations handle secure path validation and I/O.

use std::io;

/// Workspace filesystem errors.
#[derive(Debug, Clone)]
pub enum WorkspaceFileStoreError {
    /// File or directory not found.
    NotFound,
    /// Path is invalid or contains traversal attempts.
    InvalidPath(String),
    /// Path traversal rejected.
    TraversalAttempt,
    /// I/O error.
    Io(String),
    /// File exceeds size limit.
    FileTooLarge,
    /// Directory exceeds depth limit.
    DirectoryTooDeep,
    /// Directory contains too many files.
    TooManyFiles,
}

impl From<io::Error> for WorkspaceFileStoreError {
    fn from(err: io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl std::fmt::Display for WorkspaceFileStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "not found"),
            Self::InvalidPath(msg) => write!(f, "path invalid: {msg}"),
            Self::TraversalAttempt => write!(f, "path traversal rejected"),
            Self::Io(msg) => write!(f, "io error: {msg}"),
            Self::FileTooLarge => write!(f, "file too large"),
            Self::DirectoryTooDeep => write!(f, "directory too deep"),
            Self::TooManyFiles => write!(f, "too many files"),
        }
    }
}

impl std::error::Error for WorkspaceFileStoreError {}

/// Workspace file tree entry.
#[derive(Debug, Clone)]
pub struct WorkspaceFileEntry {
    /// Relative path from workspace root.
    pub path: String,
    /// True if this is a directory.
    pub is_dir: bool,
    /// Size in bytes (0 for directories).
    pub size_bytes: u64,
    /// Unix timestamp of last modification.
    pub modified_at: u64,
}

/// Workspace filesystem port trait.
///
/// Implementations must enforce path security:
/// - Reject absolute paths
/// - Reject `..` traversal attempts
/// - Canonicalize symlinks to prevent escape
/// - Validate workspace IDs
/// - Enforce size/depth/file count limits
#[async_trait::async_trait]
pub trait WorkspaceFileStore: Send + Sync {
    /// Initializes workspace directory structure.
    async fn init_workspace(&self, workspace_id: &str) -> Result<(), WorkspaceFileStoreError>;

    /// Lists files and directories in a path with bounded depth.
    async fn list_tree(
        &self,
        workspace_id: &str,
        rel_path: &str,
        max_depth: usize,
    ) -> Result<Vec<WorkspaceFileEntry>, WorkspaceFileStoreError>;

    /// Reads a file (bounded to 10MB).
    async fn read_file(
        &self,
        workspace_id: &str,
        rel_path: &str,
    ) -> Result<Vec<u8>, WorkspaceFileStoreError>;

    /// Writes a file (bounded to 10MB).
    async fn write_file(
        &self,
        workspace_id: &str,
        rel_path: &str,
        contents: &[u8],
    ) -> Result<(), WorkspaceFileStoreError>;

    /// Creates a directory.
    async fn create_dir(
        &self,
        workspace_id: &str,
        rel_path: &str,
    ) -> Result<(), WorkspaceFileStoreError>;

    /// Deletes a file or empty directory.
    async fn delete(
        &self,
        workspace_id: &str,
        rel_path: &str,
    ) -> Result<(), WorkspaceFileStoreError>;

    /// Renames or moves a file or directory.
    async fn rename(
        &self,
        workspace_id: &str,
        from: &str,
        to: &str,
    ) -> Result<(), WorkspaceFileStoreError>;
}
