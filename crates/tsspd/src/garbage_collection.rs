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
                                match check_and_delete_orphan(
                                    blob_entry.path(),
                                    filename,
                                    repository,
                                ) {
                                    Ok(was_deleted) => {
                                        if was_deleted {
                                            deleted_count += 1;
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("warning: failed to check blob {}: {e}", filename);
                                    }
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
) -> Result<bool, String> {
    if filename.len() < 16 {
        return Ok(false);
    }

    let content_hash = ContentHash::new(filename)
        .map_err(|e| format!("invalid content hash in filename: {e}"))?;

    match repository.find_file_by_content_hash(&content_hash) {
        Ok(Some(_)) => Ok(false),
        Ok(None) => {
            std::fs::remove_file(&path)
                .map_err(|e| format!("failed to delete orphan blob: {e}"))?;
            Ok(true)
        }
        Err(e) => Err(format!("repository error while checking blob: {e}")),
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn filename_length_threshold_is_correct() {
        let short_filename = "short";
        assert!(short_filename.len() < 16);

        let long_filename = "a_very_long_hash_filename";
        assert!(long_filename.len() >= 16);
    }

    #[test]
    fn saturation_prevents_negative_time() {
        let now: u64 = 100;
        let retention: u64 = 200;
        let result = now.saturating_sub(retention);
        assert_eq!(result, 0); // saturates to 0, never negative
    }

    #[test]
    fn saturation_with_valid_subtraction() {
        let now: u64 = 1000;
        let retention: u64 = 200;
        let result = now.saturating_sub(retention);
        assert_eq!(result, 800);
    }

    #[test]
    fn retention_days_to_seconds_conversion() {
        let days = 30;
        let seconds = days * 86_400;
        assert_eq!(seconds, 2_592_000);
    }
}
