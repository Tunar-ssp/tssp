#![allow(clippy::unwrap_used, clippy::unreadable_literal, clippy::needless_raw_string_hashes, clippy::uninlined_format_args, clippy::expect_used, clippy::needless_borrows_for_generic_args, clippy::map_unwrap_or, clippy::return_self_not_must_use, clippy::too_many_lines, clippy::missing_errors_doc, clippy::redundant_closure_for_method_calls, clippy::manual_string_new, clippy::ip_constant, clippy::single_char_pattern, clippy::absurd_extreme_comparisons, clippy::erasing_op, clippy::clone_on_copy)]
//! Upload temporary directory cleanup helpers.

use std::path::Path;
use std::time::{Duration, SystemTime};

/// Summary of removed temporary upload entries.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TempCleanupReport {
    /// Number of files removed.
    pub files_removed: u64,
    /// Number of directories removed.
    pub directories_removed: u64,
    /// Number of removal errors encountered.
    pub errors: u64,
}

impl TempCleanupReport {
    /// Total count of removed files and directories.
    pub fn total_removed(self) -> u64 {
        self.files_removed + self.directories_removed
    }
}

/// Removes children of the upload temp directory.
///
/// `min_age` protects active uploads during manual maintenance. Startup passes
/// `None`, because in-memory chunk sessions cannot survive process restart.
pub fn cleanup_temp_upload_dir(dir: &Path, min_age: Option<Duration>) -> TempCleanupReport {
    let mut report = TempCleanupReport::default();
    if !dir.exists() {
        return report;
    }

    let now = SystemTime::now();
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => {
            report.errors += 1;
            return report;
        }
    };

    for entry in entries {
        let Ok(entry) = entry else {
            report.errors += 1;
            continue;
        };
        let path = entry.path();
        let Ok(metadata) = entry.metadata() else {
            report.errors += 1;
            continue;
        };
        if !is_old_enough(&metadata, now, min_age) {
            continue;
        }

        if metadata.is_dir() {
            if std::fs::remove_dir_all(&path).is_ok() {
                report.directories_removed += 1;
            } else {
                report.errors += 1;
            }
        } else if std::fs::remove_file(&path).is_ok() {
            report.files_removed += 1;
        } else {
            report.errors += 1;
        }
    }

    report
}

fn is_old_enough(metadata: &std::fs::Metadata, now: SystemTime, min_age: Option<Duration>) -> bool {
    let Some(min_age) = min_age else {
        return true;
    };
    let Ok(modified) = metadata.modified() else {
        return false;
    };
    now.duration_since(modified).is_ok_and(|age| age >= min_age)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_case_dir(name: &str) -> std::path::PathBuf {
        let unique = format!(
            "tssp-temp-cleanup-{name}-{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        std::env::temp_dir().join(unique)
    }

    #[test]
    fn removes_files_and_chunk_session_directories() {
        let root = temp_case_dir("recursive");
        let session_dir = root.join(".ses_test");
        std::fs::create_dir_all(&session_dir).unwrap();
        std::fs::write(root.join("upload.part"), b"partial").unwrap();
        std::fs::write(session_dir.join("chunk_0.part"), b"chunk").unwrap();

        let report = cleanup_temp_upload_dir(&root, None);

        assert_eq!(report.files_removed, 1);
        assert_eq!(report.directories_removed, 1);
        assert_eq!(report.errors, 0);
        assert!(!root.join("upload.part").exists());
        assert!(!session_dir.exists());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn keeps_recent_entries_when_min_age_is_set() {
        let root = temp_case_dir("recent");
        let session_dir = root.join(".ses_recent");
        std::fs::create_dir_all(&session_dir).unwrap();
        std::fs::write(root.join("upload.part"), b"partial").unwrap();

        let report = cleanup_temp_upload_dir(&root, Some(Duration::from_secs(24 * 60 * 60)));

        assert_eq!(report.total_removed(), 0);
        assert_eq!(report.errors, 0);
        assert!(root.join("upload.part").exists());
        assert!(session_dir.exists());

        let _ = std::fs::remove_dir_all(&root);
    }
}
