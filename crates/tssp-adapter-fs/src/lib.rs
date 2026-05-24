//! Filesystem implementation of content-addressed blob storage.
//!
//! Bytes are streamed into a temporary file while BLAKE3 is computed. Completed
//! files are moved into a fanout path based on the content hash, so identical
//! content shares one stored blob.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use tssp_domain::{ContentHash, FileSize, StorageHandle};
use tssp_ports::{BlobReadError, BlobReader, BlobStore, BlobStoreError, BlobWriteOutcome};

const BUFFER_SIZE: usize = 64 * 1024;
const TEMP_DIR: &str = "tmp";
const BLOB_DIR: &str = "blobs";

/// Filesystem-backed content-addressed blob store.
#[derive(Debug, Clone)]
pub struct FilesystemBlobStore {
    root: PathBuf,
}

impl FilesystemBlobStore {
    /// Creates a store rooted at `root`.
    ///
    /// # Errors
    ///
    /// Returns [`BlobStoreError`] when the required directory structure cannot
    /// be created.
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, BlobStoreError> {
        let store = Self { root: root.into() };
        store.ensure_layout()?;
        Ok(store)
    }

    /// Returns the configured storage root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn ensure_layout(&self) -> Result<(), BlobStoreError> {
        create_dir_all(self.root.join(TEMP_DIR))?;
        create_dir_all(self.root.join(BLOB_DIR))?;
        Ok(())
    }

    fn create_temp_file(&self) -> Result<(PathBuf, File), BlobStoreError> {
        let temp_dir = self.root.join(TEMP_DIR);
        for attempt in 0..100_u32 {
            let path = temp_dir.join(temp_name(attempt));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => return Ok((path, file)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => {
                    return Err(BlobStoreError::WriteFailed {
                        message: error.to_string(),
                    });
                }
            }
        }

        Err(BlobStoreError::WriteFailed {
            message: "could not allocate a unique temporary blob path".to_owned(),
        })
    }

    fn permanent_path(&self, hash: &ContentHash) -> Result<PathBuf, BlobStoreError> {
        let fanout = fanout(hash)?;
        Ok(self
            .root
            .join(BLOB_DIR)
            .join(fanout.first)
            .join(fanout.second)
            .join(hash.as_str()))
    }

    fn handle_for_hash(hash: &ContentHash) -> Result<StorageHandle, BlobStoreError> {
        let fanout = fanout(hash)?;
        StorageHandle::new(format!(
            "{BLOB_DIR}/{}/{}/{}",
            fanout.first,
            fanout.second,
            hash.as_str()
        ))
        .map_err(|error| BlobStoreError::WriteFailed {
            message: error.to_string(),
        })
    }

    fn path_for_handle(&self, handle: &StorageHandle) -> Result<PathBuf, BlobStoreError> {
        let mut path = self.root.clone();
        for component in handle.as_str().split('/') {
            if component.is_empty() || component == "." || component == ".." {
                return Err(BlobStoreError::WriteFailed {
                    message: "invalid storage handle component".to_owned(),
                });
            }
            path.push(component);
        }
        Ok(path)
    }
}

impl BlobStore for FilesystemBlobStore {
    fn put_stream(&self, source: &mut dyn Read) -> Result<BlobWriteOutcome, BlobStoreError> {
        let (temp_path, mut temp_file) = self.create_temp_file()?;
        let stream_result = stream_to_temp(source, &mut temp_file);
        let (content_hash, size) = match stream_result {
            Ok(value) => value,
            Err(error) => {
                remove_file_best_effort(&temp_path);
                return Err(error);
            }
        };

        if let Err(error) = temp_file.sync_all() {
            remove_file_best_effort(&temp_path);
            return Err(BlobStoreError::WriteFailed {
                message: error.to_string(),
            });
        }
        drop(temp_file);

        let permanent_path = self.permanent_path(&content_hash)?;
        let parent = permanent_path
            .parent()
            .ok_or_else(|| BlobStoreError::WriteFailed {
                message: "blob path has no parent directory".to_owned(),
            })?;
        create_dir_all(parent)?;

        let deduplicated = if permanent_path.exists() {
            remove_file_best_effort(&temp_path);
            true
        } else {
            rename_file(&temp_path, &permanent_path)?;
            sync_directory(parent)?;
            false
        };

        Ok(BlobWriteOutcome {
            content_hash: content_hash.clone(),
            handle: Self::handle_for_hash(&content_hash)?,
            size,
            deduplicated,
        })
    }

    fn put_staged(
        &self,
        temp_path: &Path,
        content_hash: &ContentHash,
        size: FileSize,
    ) -> Result<BlobWriteOutcome, BlobStoreError> {
        let permanent_path = self.permanent_path(content_hash)?;
        let parent = permanent_path
            .parent()
            .ok_or_else(|| BlobStoreError::WriteFailed {
                message: "blob path has no parent directory".to_owned(),
            })?;
        create_dir_all(parent)?;

        let deduplicated = if permanent_path.exists() {
            remove_file_best_effort(temp_path);
            true
        } else {
            rename_file(temp_path, &permanent_path)?;
            sync_directory(parent)?;
            false
        };

        Ok(BlobWriteOutcome {
            content_hash: content_hash.clone(),
            handle: Self::handle_for_hash(content_hash)?,
            size,
            deduplicated,
        })
    }

    fn cleanup_unreferenced(&self, handle: &StorageHandle) -> Result<(), BlobStoreError> {
        let path = self.path_for_handle(handle)?;
        let handle_clone = handle.clone();
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::spawn(async move {
                remove_unreferenced_blob(&path, &handle_clone);
            });
        } else {
            remove_unreferenced_blob(&path, &handle_clone);
        }
        Ok(())
    }
}

impl BlobReader for FilesystemBlobStore {
    fn open_blob(&self, handle: &StorageHandle) -> Result<File, BlobReadError> {
        let path = self
            .path_for_handle(handle)
            .map_err(|error| BlobReadError::ReadFailed {
                message: error.to_string(),
            })?;
        File::open(path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                return BlobReadError::Missing {
                    handle: handle.clone(),
                };
            }
            BlobReadError::ReadFailed {
                message: error.to_string(),
            }
        })
    }
}

fn stream_to_temp(
    source: &mut dyn Read,
    temp_file: &mut File,
) -> Result<(ContentHash, FileSize), BlobStoreError> {
    let mut hasher = blake3::Hasher::new();
    let mut size = 0_u64;
    let mut buffer = vec![0_u8; BUFFER_SIZE];

    loop {
        let bytes_read = source
            .read(&mut buffer)
            .map_err(|error| BlobStoreError::ReadFailed {
                message: error.to_string(),
            })?;
        if bytes_read == 0 {
            break;
        }

        let chunk = &buffer[..bytes_read];
        hasher.update(chunk);
        temp_file
            .write_all(chunk)
            .map_err(|error| BlobStoreError::WriteFailed {
                message: error.to_string(),
            })?;
        size = size
            .checked_add(u64::try_from(bytes_read).unwrap_or(u64::MAX))
            .ok_or_else(|| BlobStoreError::WriteFailed {
                message: "blob size overflow".to_owned(),
            })?;
    }

    let hex = hasher.finalize().to_hex();
    let hash = ContentHash::new(hex.as_str()).map_err(|error| BlobStoreError::WriteFailed {
        message: error.to_string(),
    })?;

    Ok((hash, FileSize::new(size)))
}

fn create_dir_all(path: impl AsRef<Path>) -> Result<(), BlobStoreError> {
    fs::create_dir_all(path).map_err(|error| BlobStoreError::WriteFailed {
        message: error.to_string(),
    })
}

fn rename_file(from: &Path, to: &Path) -> Result<(), BlobStoreError> {
    fs::rename(from, to).map_err(|error| BlobStoreError::WriteFailed {
        message: error.to_string(),
    })
}

fn sync_directory(path: &Path) -> Result<(), BlobStoreError> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| BlobStoreError::WriteFailed {
            message: error.to_string(),
        })
}

fn remove_file_best_effort(path: &Path) {
    let _ignored = fs::remove_file(path);
}

fn remove_unreferenced_blob(path: &Path, handle: &StorageHandle) {
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => tracing::warn!("background cleanup failed for {}: {error}", handle.as_str()),
    }
}

struct Fanout<'a> {
    first: &'a str,
    second: &'a str,
}

fn fanout(hash: &ContentHash) -> Result<Fanout<'_>, BlobStoreError> {
    let value = hash.as_str();
    let first = value.get(0..2).ok_or_else(|| BlobStoreError::WriteFailed {
        message: "content hash missing first fanout component".to_owned(),
    })?;
    let second = value.get(2..4).ok_or_else(|| BlobStoreError::WriteFailed {
        message: "content hash missing second fanout component".to_owned(),
    })?;
    Ok(Fanout { first, second })
}

fn temp_name(attempt: u32) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    format!("upload-{}-{nanos}-{attempt}.tmp", std::process::id())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::{Cursor, Read};

    use tempfile::tempdir;
    use tssp_domain::StorageHandle;
    use tssp_ports::{BlobReadError, BlobReader, BlobStore};

    use super::FilesystemBlobStore;

    #[test]
    fn put_stream_stores_content_by_hash() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let store = FilesystemBlobStore::new(temp.path())
            .unwrap_or_else(|error| panic!("store init failed: {error}"));
        let mut source = Cursor::new(b"hello world".as_slice());

        let outcome = store
            .put_stream(&mut source)
            .unwrap_or_else(|error| panic!("put stream failed: {error}"));

        assert_eq!(outcome.size.bytes(), 11);
        assert!(!outcome.deduplicated);
        assert!(temp.path().join(outcome.handle.as_str()).is_file());
    }

    #[test]
    fn put_stream_deduplicates_identical_content() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let store = FilesystemBlobStore::new(temp.path())
            .unwrap_or_else(|error| panic!("store init failed: {error}"));
        let mut first = Cursor::new(b"same".as_slice());
        let first_outcome = store
            .put_stream(&mut first)
            .unwrap_or_else(|error| panic!("first put failed: {error}"));
        let mut second = Cursor::new(b"same".as_slice());

        let second_outcome = store
            .put_stream(&mut second)
            .unwrap_or_else(|error| panic!("second put failed: {error}"));

        assert_eq!(first_outcome.handle, second_outcome.handle);
        assert!(second_outcome.deduplicated);
    }

    #[test]
    fn put_staged_moves_temp_file_to_permanent_storage() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let store = FilesystemBlobStore::new(temp.path())
            .unwrap_or_else(|error| panic!("store init failed: {error}"));

        let content = b"staged content";
        let hash = blake3::hash(content);
        let content_hash = tssp_domain::ContentHash::new(hash.to_hex().as_str())
            .unwrap_or_else(|error| panic!("invalid content hash: {error}"));
        let size = tssp_domain::FileSize::new(content.len() as u64);

        let temp_file_path = temp.path().join("my-staged-upload.tmp");
        fs::write(&temp_file_path, content).unwrap_or_else(|error| panic!("write failed: {error}"));

        let outcome = store
            .put_staged(&temp_file_path, &content_hash, size)
            .unwrap_or_else(|error| panic!("put staged failed: {error}"));

        assert_eq!(outcome.content_hash, content_hash);
        assert!(!outcome.deduplicated);
        assert!(!temp_file_path.exists());
        assert!(temp.path().join(outcome.handle.as_str()).is_file());
    }

    #[tokio::test]
    async fn cleanup_unreferenced_removes_existing_blob() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let store = FilesystemBlobStore::new(temp.path())
            .unwrap_or_else(|error| panic!("store init failed: {error}"));
        let mut source = Cursor::new(b"delete me".as_slice());
        let outcome = store
            .put_stream(&mut source)
            .unwrap_or_else(|error| panic!("put stream failed: {error}"));
        let path = temp.path().join(outcome.handle.as_str());

        store
            .cleanup_unreferenced(&outcome.handle)
            .unwrap_or_else(|error| panic!("cleanup failed: {error}"));

        tokio::task::yield_now().await;
        assert!(!path.exists());
    }

    #[test]
    fn empty_file_is_stored_normally() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let store = FilesystemBlobStore::new(temp.path())
            .unwrap_or_else(|error| panic!("store init failed: {error}"));
        let mut source = Cursor::new([].as_slice());

        let outcome = store
            .put_stream(&mut source)
            .unwrap_or_else(|error| panic!("put stream failed: {error}"));

        assert_eq!(outcome.size.bytes(), 0);
        assert!(fs::read(temp.path().join(outcome.handle.as_str()))
            .unwrap_or_else(|error| panic!("read failed: {error}"))
            .is_empty());
    }

    #[test]
    fn open_blob_reads_stored_bytes() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let store = FilesystemBlobStore::new(temp.path())
            .unwrap_or_else(|error| panic!("store init failed: {error}"));
        let mut source = Cursor::new(b"hello world".as_slice());
        let outcome = store
            .put_stream(&mut source)
            .unwrap_or_else(|error| panic!("put stream failed: {error}"));

        let mut reader = store
            .open_blob(&outcome.handle)
            .unwrap_or_else(|error| panic!("open blob failed: {error}"));
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .unwrap_or_else(|error| panic!("read failed: {error}"));

        assert_eq!(bytes, b"hello world");
    }

    #[test]
    fn open_blob_reports_missing_blob() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let store = FilesystemBlobStore::new(temp.path())
            .unwrap_or_else(|error| panic!("store init failed: {error}"));
        let handle = StorageHandle::new("blobs/ab/cd/missing")
            .unwrap_or_else(|error| panic!("handle failed: {error}"));

        let result = store.open_blob(&handle);

        assert!(matches!(result, Err(BlobReadError::Missing { .. })));
    }
}
