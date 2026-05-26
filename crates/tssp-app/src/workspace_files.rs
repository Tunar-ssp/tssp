//! Workspace file operations service.
//!
//! Orchestrates filesystem operations through the `WorkspaceFileStore` port,
//! providing a clean application-level interface.

use std::sync::Arc;

use tssp_ports::{WorkspaceFileEntry, WorkspaceFileStore, WorkspaceFileStoreError};

/// Application-level workspace file service.
pub struct WorkspaceFileService {
    store: Arc<dyn WorkspaceFileStore>,
}

impl WorkspaceFileService {
    /// Creates a new workspace file service.
    pub fn new(store: Arc<dyn WorkspaceFileStore>) -> Self {
        Self { store }
    }

    /// Initializes workspace directory structure.
    ///
    /// # Errors
    ///
    /// Returns `WorkspaceFileStoreError` on I/O or validation failures.
    pub async fn init_workspace(&self, workspace_id: &str) -> Result<(), WorkspaceFileStoreError> {
        self.store.init_workspace(workspace_id).await
    }

    /// Lists files and directories in a path.
    ///
    /// # Errors
    ///
    /// Returns `WorkspaceFileStoreError` on path validation, I/O, or limit violations.
    pub async fn list_tree(
        &self,
        workspace_id: &str,
        rel_path: &str,
        max_depth: usize,
    ) -> Result<Vec<WorkspaceFileEntry>, WorkspaceFileStoreError> {
        self.store
            .list_tree(workspace_id, rel_path, max_depth)
            .await
    }

    /// Reads a file.
    ///
    /// # Errors
    ///
    /// Returns `WorkspaceFileStoreError` on path validation, I/O, or size limit violations.
    pub async fn read_file(
        &self,
        workspace_id: &str,
        rel_path: &str,
    ) -> Result<Vec<u8>, WorkspaceFileStoreError> {
        self.store.read_file(workspace_id, rel_path).await
    }

    /// Writes a file.
    ///
    /// # Errors
    ///
    /// Returns `WorkspaceFileStoreError` on path validation, I/O, or size limit violations.
    pub async fn write_file(
        &self,
        workspace_id: &str,
        rel_path: &str,
        contents: &[u8],
    ) -> Result<(), WorkspaceFileStoreError> {
        self.store
            .write_file(workspace_id, rel_path, contents)
            .await
    }

    /// Creates a directory.
    ///
    /// # Errors
    ///
    /// Returns `WorkspaceFileStoreError` on path validation or I/O failures.
    pub async fn create_dir(
        &self,
        workspace_id: &str,
        rel_path: &str,
    ) -> Result<(), WorkspaceFileStoreError> {
        self.store.create_dir(workspace_id, rel_path).await
    }

    /// Deletes a file or empty directory.
    ///
    /// # Errors
    ///
    /// Returns `WorkspaceFileStoreError` on path validation, I/O, or missing file.
    pub async fn delete(
        &self,
        workspace_id: &str,
        rel_path: &str,
    ) -> Result<(), WorkspaceFileStoreError> {
        self.store.delete(workspace_id, rel_path).await
    }

    /// Renames or moves a file or directory.
    ///
    /// # Errors
    ///
    /// Returns `WorkspaceFileStoreError` on path validation, I/O, or missing source.
    pub async fn rename(
        &self,
        workspace_id: &str,
        from: &str,
        to: &str,
    ) -> Result<(), WorkspaceFileStoreError> {
        self.store.rename(workspace_id, from, to).await
    }
}
