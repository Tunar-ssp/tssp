//! Orphaned blob garbage collection on startup.

use std::path::Path;
use tssp_domain::ContentHash;
use tssp_ports::FileRepository;

/// Scans the blob directory and removes unreferenced blobs.
///
/// Returns the number of blobs deleted.
pub fn collect_garbage(
    blob_root: &Path,
    repository: &dyn FileRepository,
) -> Result<u64, String> {
    let blob_dir = blob_root.join("blobs");
    if !blob_dir.exists() {
        return Ok(0);
    }

    let mut deleted_count = 0;

    if let Ok(first_dirs) = std::fs::read_dir(&blob_dir) {
        for first_entry in first_dirs.flatten() {
            if !first_entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                continue;
            }

            if let Ok(second_dirs) = std::fs::read_dir(first_entry.path()) {
                for second_entry in second_dirs.flatten() {
                    if !second_entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                        continue;
                    }

                    if let Ok(blob_files) = std::fs::read_dir(second_entry.path()) {
                        for blob_entry in blob_files.flatten() {
                            if let Some(filename) = blob_entry.file_name().to_str() {
                                if let Err(e) = check_and_delete_orphan(
                                    blob_entry.path(),
                                    filename,
                                    repository,
                                ) {
                                    eprintln!("warning: failed to check blob {}: {e}", filename);
                                } else {
                                    deleted_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(deleted_count)
}

fn check_and_delete_orphan(
    path: std::path::PathBuf,
    filename: &str,
    repository: &dyn FileRepository,
) -> Result<(), String> {
    if filename.len() < 16 {
        return Ok(());
    }

    let content_hash = ContentHash::new(filename)
        .map_err(|e| format!("invalid content hash in filename: {e}"))?;

    match repository.find_file_by_content_hash(&content_hash) {
        Ok(Some(_)) => Ok(()),
        Ok(None) => {
            std::fs::remove_file(&path)
                .map_err(|e| format!("failed to delete orphan blob: {e}"))?;
            Ok(())
        }
        Err(e) => Err(format!("repository error while checking blob: {e}")),
    }
}
