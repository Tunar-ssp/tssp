//! SQLite implementation of SessionRepository.

use tssp_domain::{SessionKind, SessionToken, TransferSession, UnixTimestamp};
use tssp_ports::{RepositoryError, SessionRepository};

/// SQLite session repository (placeholder implementation).
#[allow(dead_code)]
pub struct SqliteSessionRepository;

impl SessionRepository for SqliteSessionRepository {
    fn create_session(&self, _session: TransferSession) -> Result<(), RepositoryError> {
        Err(RepositoryError::OperationFailed {
            message: "session creation not yet implemented".to_owned(),
        })
    }

    fn find_session(&self, _token: &SessionToken) -> Result<Option<TransferSession>, RepositoryError> {
        Err(RepositoryError::OperationFailed {
            message: "session lookup not yet implemented".to_owned(),
        })
    }

    fn mark_session_used(&self, _token: &SessionToken, _used_at: UnixTimestamp) -> Result<(), RepositoryError> {
        Err(RepositoryError::OperationFailed {
            message: "session marking not yet implemented".to_owned(),
        })
    }

    fn cleanup_expired_sessions(&self, _before: UnixTimestamp) -> Result<u64, RepositoryError> {
        Err(RepositoryError::OperationFailed {
            message: "session cleanup not yet implemented".to_owned(),
        })
    }

    fn list_sessions_by_kind(&self, _kind: SessionKind) -> Result<Vec<TransferSession>, RepositoryError> {
        Err(RepositoryError::OperationFailed {
            message: "session listing not yet implemented".to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn sessions_module_compiles() {
        // Placeholder test to ensure module compiles
    }
}
