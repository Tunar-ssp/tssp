//! Blob storage and retrieval port traits.

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use tssp_domain::{ContentHash, FileSize, StorageHandle};

use crate::errors::{BlobReadError, BlobStoreError};
use crate::record::BlobWriteOutcome;

/// Stores and retrieves content-addressed blob bytes.
pub trait BlobStore {
    /// Streams bytes into storage and returns the durable content-addressed blob.
    ///
    /// # Errors
    ///
    /// Returns [`BlobStoreError`] when the source cannot be read or the storage
    /// backend cannot durably write the blob.
    fn put_stream(&self, source: &mut dyn Read) -> Result<BlobWriteOutcome, BlobStoreError>;

    /// Moves a pre-hashed temp file into durable storage.
    ///
    /// # Errors
    ///
    /// Returns [`BlobStoreError`] when the file cannot be moved or verified.
    fn put_staged(
        &self,
        temp_path: &Path,
        content_hash: &ContentHash,
        size: FileSize,
    ) -> Result<BlobWriteOutcome, BlobStoreError>;

    /// Removes a blob that is not referenced by committed metadata.
    ///
    /// # Errors
    ///
    /// Returns [`BlobStoreError`] when cleanup cannot be completed. Callers
    /// should log this because it may require later orphan reclamation.
    fn cleanup_unreferenced(&self, handle: &StorageHandle) -> Result<(), BlobStoreError>;
}

/// Opens durable blob bytes for download.
pub trait BlobReader {
    /// Opens the blob identified by `handle`.
    ///
    /// # Errors
    ///
    /// Returns [`BlobReadError`] when the blob is missing or cannot be opened.
    fn open_blob(&self, handle: &StorageHandle) -> Result<File, BlobReadError>;
}

impl<T> BlobStore for Arc<T>
where
    T: BlobStore,
{
    fn put_stream(&self, source: &mut dyn Read) -> Result<BlobWriteOutcome, BlobStoreError> {
        self.as_ref().put_stream(source)
    }

    fn put_staged(
        &self,
        temp_path: &Path,
        content_hash: &ContentHash,
        size: FileSize,
    ) -> Result<BlobWriteOutcome, BlobStoreError> {
        self.as_ref().put_staged(temp_path, content_hash, size)
    }

    fn cleanup_unreferenced(&self, handle: &StorageHandle) -> Result<(), BlobStoreError> {
        self.as_ref().cleanup_unreferenced(handle)
    }
}

impl<T> BlobReader for Arc<T>
where
    T: BlobReader,
{
    fn open_blob(&self, handle: &StorageHandle) -> Result<File, BlobReadError> {
        self.as_ref().open_blob(handle)
    }
}
