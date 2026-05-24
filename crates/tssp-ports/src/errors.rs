//! Error types returned by port traits.

use thiserror::Error;
use tssp_domain::StorageHandle;

/// Identifier generation failure.
#[derive(Debug, Error)]
#[error("could not generate id: {message}")]
pub struct IdGenerationError {
    /// Stable diagnostic message.
    pub message: String,
}

/// Blob storage failure.
#[derive(Debug, Error)]
pub enum BlobStoreError {
    /// The storage device has insufficient free space.
    #[error(
        "insufficient storage: required {required_bytes} bytes, available {available_bytes} bytes"
    )]
    InsufficientStorage {
        /// Bytes needed to complete the write.
        required_bytes: u64,
        /// Bytes available at failure time.
        available_bytes: u64,
    },

    /// The storage backend could not read the supplied stream.
    #[error("could not read upload stream: {message}")]
    ReadFailed {
        /// Stable diagnostic message.
        message: String,
    },

    /// The storage backend could not write durable bytes.
    #[error("could not write blob: {message}")]
    WriteFailed {
        /// Stable diagnostic message.
        message: String,
    },

    /// Cleanup failed after a later application step failed.
    #[error("could not clean up unreferenced blob {handle}: {message}")]
    CleanupFailed {
        /// Opaque storage handle.
        handle: StorageHandle,
        /// Stable diagnostic message.
        message: String,
    },
}

/// Blob read failure.
#[derive(Debug, Error)]
pub enum BlobReadError {
    /// The metadata record points at a blob that no longer exists.
    #[error("blob {handle} is missing")]
    Missing {
        /// Opaque storage handle.
        handle: StorageHandle,
    },

    /// The storage backend could not open or read durable bytes.
    #[error("could not read blob: {message}")]
    ReadFailed {
        /// Stable diagnostic message.
        message: String,
    },
}

/// Metadata persistence failure.
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// The repository could not acquire a write lock in time.
    #[error("metadata store is busy")]
    Busy,

    /// The repository rejected a conflicting write.
    #[error("metadata conflict: {message}")]
    Conflict {
        /// Stable diagnostic message.
        message: String,
    },

    /// The requested record does not exist.
    #[error("metadata record was not found")]
    NotFound,

    /// The repository operation failed.
    #[error("metadata operation failed: {message}")]
    OperationFailed {
        /// Stable diagnostic message.
        message: String,
    },
}
