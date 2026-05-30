//! Virtual folder management orchestration.

use thiserror::Error;
use tssp_domain::UserId;
use tssp_ports::{FileRepository, RepositoryError};

/// Folder use case errors.
#[derive(Debug, Error)]
pub enum FolderError {
    /// Target folder path is invalid.
    #[error("invalid folder path: {0}")]
    InvalidPath(&'static str),
    /// Folder operation failed in the repository.
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),
    /// Permission denied for the operation.
    #[error("permission denied")]
    Forbidden,
}

/// Orchestrates virtual folder operations across the repository.
pub struct FolderService<R> {
    repository: R,
}

impl<R> FolderService<R> {
    /// Creates a folder service from a file repository.
    #[must_use]
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> FolderService<R>
where
    R: FileRepository,
{
    /// Renames or moves a virtual folder by updating the path prefix of all contained files.
    ///
    /// # Errors
    ///
    /// Returns [`FolderError`] when paths are invalid or the repository update fails.
    pub fn move_folder(&self, from: &str, to: &str) -> Result<u64, FolderError> {
        let from = normalize_folder_path(from);
        let to = normalize_folder_path(to);

        if from.is_empty() {
            return Err(FolderError::InvalidPath("cannot move the bucket root"));
        }
        if to == from || to.starts_with(&format!("{from}/")) {
            return Err(FolderError::InvalidPath(
                "cannot move a folder into itself or its own subfolder",
            ));
        }

        validate_folder_path(&from).map_err(FolderError::InvalidPath)?;
        validate_folder_path(&to).map_err(FolderError::InvalidPath)?;

        self.repository
            .update_folder_path_prefix(&from, &to)
            .map_err(FolderError::Repository)
    }

    /// Moves all files out of a virtual folder into the bucket root or a parent path.
    ///
    /// # Errors
    ///
    /// Returns [`FolderError`] when the path is invalid or the repository update fails.
    pub fn delete_folder(&self, path: &str) -> Result<u64, FolderError> {
        let path = normalize_folder_path(path);
        if path.is_empty() {
            return Err(FolderError::InvalidPath("cannot delete the bucket root"));
        }

        validate_folder_path(&path).map_err(FolderError::InvalidPath)?;

        self.repository
            .update_folder_path_prefix(&path, "")
            .map_err(FolderError::Repository)
    }

    /// Renames or moves a virtual folder scoped to a single owner.
    ///
    /// Only files belonging to `owner_id` are affected.
    ///
    /// # Errors
    ///
    /// Returns [`FolderError`] when paths are invalid or the repository update fails.
    pub fn move_folder_for_user(
        &self,
        from: &str,
        to: &str,
        owner_id: &UserId,
    ) -> Result<u64, FolderError> {
        let from = normalize_folder_path(from);
        let to = normalize_folder_path(to);

        if from.is_empty() {
            return Err(FolderError::InvalidPath("cannot move the bucket root"));
        }
        if to == from || to.starts_with(&format!("{from}/")) {
            return Err(FolderError::InvalidPath(
                "cannot move a folder into itself or its own subfolder",
            ));
        }

        validate_folder_path(&from).map_err(FolderError::InvalidPath)?;
        validate_folder_path(&to).map_err(FolderError::InvalidPath)?;

        self.repository
            .update_folder_path_prefix_owned(&from, &to, owner_id)
            .map_err(FolderError::Repository)
    }

    /// Moves all files owned by `owner_id` out of a virtual folder into bucket root.
    ///
    /// # Errors
    ///
    /// Returns [`FolderError`] when the path is invalid or the repository update fails.
    pub fn delete_folder_for_user(
        &self,
        path: &str,
        owner_id: &UserId,
    ) -> Result<u64, FolderError> {
        let path = normalize_folder_path(path);
        if path.is_empty() {
            return Err(FolderError::InvalidPath("cannot delete the bucket root"));
        }

        validate_folder_path(&path).map_err(FolderError::InvalidPath)?;

        self.repository
            .update_folder_path_prefix_owned(&path, "", owner_id)
            .map_err(FolderError::Repository)
    }

    /// Lists folders and their file counts, optionally filtered by owner.
    ///
    /// # Errors
    ///
    /// Returns [`FolderError`] when the repository query fails.
    pub fn list_folders(
        &self,
        owner_id: Option<&tssp_domain::UserId>,
    ) -> Result<Vec<(String, u64)>, FolderError> {
        self.repository
            .list_folder_counts(owner_id)
            .map_err(FolderError::Repository)
    }
}

const MAX_FOLDER_PATH_BYTES: usize = 1024;

/// Normalizes a folder path for storage (no leading/trailing slashes).
#[must_use]
pub fn normalize_folder_path(value: &str) -> String {
    value.trim().trim_matches('/').replace('\\', "/")
}

/// Validates a normalized folder path.
///
/// # Errors
///
/// Returns `Err(&'static str)` when the path fails validation.
pub fn validate_folder_path(path: &str) -> Result<(), &'static str> {
    if path.contains('\0') {
        return Err("folder path must not contain null bytes");
    }
    if path.len() > MAX_FOLDER_PATH_BYTES {
        return Err("folder path is too long (max 1024 bytes)");
    }
    // Reject any component that is exactly ".." after splitting on "/"
    for component in path.split('/') {
        if component == ".." {
            return Err("folder path must not contain '..' components");
        }
    }
    Ok(())
}
