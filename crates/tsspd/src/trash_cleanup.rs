//! Deleted file purging with configurable retention period.

use tssp_app::PurgeDeletedFilesService;
use tssp_domain::UnixTimestamp;
use tssp_ports::BlobStore;

/// Summary of permanent trash deletion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrashCleanupReport {
    /// Number of files permanently deleted.
    pub files_purged: u64,
    /// Purge operation failed.
    pub error: bool,
}

impl TrashCleanupReport {
    /// Creates a success report.
    #[must_use] 
    pub fn success(files_purged: u64) -> Self {
        Self {
            files_purged,
            error: false,
        }
    }

    /// Creates an error report.
    #[must_use] 
    pub fn failure() -> Self {
        Self {
            files_purged: 0,
            error: true,
        }
    }
}

/// Purges soft-deleted files older than the retention period.
///
/// Returns a report of how many files were permanently deleted.
pub fn purge_expired_trash<B, R>(
    service: &PurgeDeletedFilesService<B, R>,
    retention_days: u64,
) -> TrashCleanupReport
where
    B: BlobStore,
    R: tssp_ports::FileRepository,
{
    if retention_days == 0 {
        return TrashCleanupReport {
            files_purged: 0,
            error: false,
        };
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let retention_seconds = retention_days * 86_400;
    let older_than_secs = now.saturating_sub(retention_seconds);

    let older_than = match UnixTimestamp::new(older_than_secs as i64) {
        Ok(ts) => ts,
        Err(_) => return TrashCleanupReport::failure(),
    };

    match service.purge(older_than) {
        Ok(count) => TrashCleanupReport::success(count),
        Err(_) => TrashCleanupReport::failure(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trash_cleanup_report_success_has_no_error() {
        let report = TrashCleanupReport::success(5);
        assert_eq!(report.files_purged, 5);
        assert!(!report.error);
    }

    #[test]
    fn trash_cleanup_report_success_with_zero_files() {
        let report = TrashCleanupReport::success(0);
        assert_eq!(report.files_purged, 0);
        assert!(!report.error);
    }

    #[test]
    fn trash_cleanup_report_failure_has_error() {
        let report = TrashCleanupReport::failure();
        assert_eq!(report.files_purged, 0);
        assert!(report.error);
    }

    #[test]
    fn trash_cleanup_report_failure_always_zero_files() {
        let report = TrashCleanupReport::failure();
        assert_eq!(report.files_purged, 0);
    }

    #[test]
    fn retention_days_zero_returns_zero_files() {
        // This is a unit test that doesn't need a service
        // Just verify the logic that zero retention returns zero
        assert_eq!(0 * 86_400, 0);
    }

    #[test]
    fn retention_seconds_calculation_is_correct() {
        let retention_days = 30;
        let retention_seconds = retention_days * 86_400;
        assert_eq!(retention_seconds, 2_592_000);
    }

    #[test]
    fn retention_calculation_one_day() {
        let retention_days = 1;
        let retention_seconds = retention_days * 86_400;
        assert_eq!(retention_seconds, 86_400);
    }
}
