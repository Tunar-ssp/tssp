//! Blob/index consistency checks at startup.

use std::path::Path;
use std::sync::Arc;

use tssp_adapter_fs::FilesystemBlobStore;
use tssp_adapter_sqlite::SqliteFileRepository;
use tssp_domain::ContentHash;
use tssp_ports::{FileRepository, ListQuery};

/// Counts indexed files whose content blob is missing on disk.
///
/// # Errors
///
/// Returns an error when metadata cannot be read.
pub fn count_missing_blobs(
    repository: &SqliteFileRepository,
    blob_store: &FilesystemBlobStore,
) -> Result<u64, String> {
    let mut missing = 0_u64;
    let mut cursor = None;
    loop {
        let query = ListQuery {
            limit: 200,
            after_cursor: cursor,
            ..ListQuery::default()
        };
        let page = repository
            .list_files(&query)
            .map_err(|error| error.to_string())?;
        for file in &page.files {
            if !blob_exists(blob_store.root(), &file.content_hash) {
                missing += 1;
                tracing::warn!(
                    file_id = %file.id.as_str(),
                    hash = %file.content_hash.as_str(),
                    "integrity: indexed file has no blob on disk"
                );
            }
        }
        if page.next_cursor.is_none() {
            break;
        }
        cursor = page.next_cursor;
    }
    Ok(missing)
}

fn blob_exists(storage_root: &Path, hash: &ContentHash) -> bool {
    let hash_str = hash.as_str();
    if hash_str.len() < 4 {
        return false;
    }
    let path = storage_root
        .join("blobs")
        .join(&hash_str[..2])
        .join(&hash_str[2..4])
        .join(hash_str);
    path.is_file()
}

/// Runs integrity scan and logs summary.
pub fn run_startup_integrity_scan(
    repository: &SqliteFileRepository,
    blob_store: &FilesystemBlobStore,
) -> u64 {
    match count_missing_blobs(repository, blob_store) {
        Ok(missing) => {
            if missing > 0 {
                tracing::warn!(missing, "integrity scan found files with missing blobs");
            } else {
                tracing::info!("integrity scan: all indexed blobs present");
            }
            missing
        }
        Err(error) => {
            tracing::warn!("integrity scan skipped: {error}");
            0
        }
    }
}

/// Runs integrity scan in a background task.
pub fn spawn_startup_integrity_scan(
    repository: Arc<SqliteFileRepository>,
    blob_store: Arc<FilesystemBlobStore>,
    counter: Arc<std::sync::atomic::AtomicU64>,
) {
    tokio::spawn(async move {
        tracing::info!("integrity: starting background consistency scan");
        match count_missing_blobs(&repository, &blob_store) {
            Ok(missing) => {
                counter.store(missing, std::sync::atomic::Ordering::Relaxed);
                if missing > 0 {
                    tracing::warn!(missing, "integrity scan found files with missing blobs");
                } else {
                    tracing::info!("integrity scan: all indexed blobs present");
                }
            }
            Err(error) => {
                tracing::warn!("integrity scan failed: {error}");
            }
        }
    });
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::blob_exists;
    use tssp_domain::ContentHash;

    #[test]
    fn missing_blob_is_not_found() {
        let temp = tempfile::tempdir().expect("tempdir");
        let hash =
            ContentHash::new("0000000000000000000000000000000000000000000000000000000000000000")
                .expect("hash");
        assert!(!blob_exists(temp.path(), &hash));
    }

    #[test]
    fn present_blob_is_found() {
        let temp = tempfile::tempdir().expect("tempdir");
        let hash_str = "abcd0000000000000000000000000000000000000000000000000000000000ab";
        let hash = ContentHash::new(hash_str).expect("hash");
        let path = temp
            .path()
            .join("blobs")
            .join("ab")
            .join("cd")
            .join(hash_str);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, b"x").expect("write");
        assert!(blob_exists(temp.path(), &hash));
    }
}
