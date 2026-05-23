//! Application service for session management.

use tssp_domain::{SessionKind, SessionToken, TransferSession, UnixTimestamp};
use tssp_ports::{RepositoryError, SessionRepository};

/// Application service for managing transfer sessions.
pub struct SessionService<R: SessionRepository> {
    repository: R,
}

impl<R: SessionRepository> SessionService<R> {
    /// Creates a new session service.
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Creates a send session for sharing a file.
    ///
    /// # Errors
    ///
    /// Returns an error when the session cannot be created.
    pub fn create_send_session(
        &self,
        token: SessionToken,
        file_id: &str,
        ttl_seconds: u64,
        now: UnixTimestamp,
    ) -> Result<TransferSession, RepositoryError> {
        #[allow(clippy::cast_possible_wrap)]
        let ttl = ttl_seconds as i64;
        let expires_at = UnixTimestamp::new(now.seconds().saturating_add(ttl))
            .map_err(|e| RepositoryError::OperationFailed {
                message: format!("failed to calculate session expiration: {e}"),
            })?;

        let file_id =
            tssp_domain::FileId::new(file_id).map_err(|e| RepositoryError::OperationFailed {
                message: format!("invalid file id: {e}"),
            })?;

        let session =
            TransferSession::new(token, SessionKind::Send, now, expires_at, Some(file_id))
                .map_err(|e| RepositoryError::OperationFailed {
                    message: format!("failed to create send session: {e}"),
                })?;

        self.repository.create_session(session.clone())?;
        Ok(session)
    }

    /// Creates a receive session for accepting uploads.
    ///
    /// # Errors
    ///
    /// Returns an error when the session cannot be created.
    pub fn create_receive_session(
        &self,
        token: SessionToken,
        ttl_seconds: u64,
        now: UnixTimestamp,
    ) -> Result<TransferSession, RepositoryError> {
        #[allow(clippy::cast_possible_wrap)]
        let ttl = ttl_seconds as i64;
        let expires_at = UnixTimestamp::new(now.seconds().saturating_add(ttl))
            .map_err(|e| RepositoryError::OperationFailed {
                message: format!("failed to calculate session expiration: {e}"),
            })?;

        let session = TransferSession::new(token, SessionKind::Receive, now, expires_at, None)
            .map_err(|e| RepositoryError::OperationFailed {
                message: format!("failed to create receive session: {e}"),
            })?;

        self.repository.create_session(session.clone())?;
        Ok(session)
    }

    /// Retrieves a session by token.
    ///
    /// # Errors
    ///
    /// Returns an error when the session cannot be found or retrieved.
    pub fn get_session(
        &self,
        token: &SessionToken,
    ) -> Result<Option<TransferSession>, RepositoryError> {
        self.repository.find_session(token)
    }

    /// Marks a session as used.
    ///
    /// # Errors
    ///
    /// Returns an error when the update cannot be committed.
    pub fn use_session(
        &self,
        token: &SessionToken,
        now: UnixTimestamp,
    ) -> Result<(), RepositoryError> {
        self.repository.mark_session_used(token, now)
    }

    /// Cleans up expired sessions.
    ///
    /// # Errors
    ///
    /// Returns an error when the cleanup cannot be completed.
    pub fn cleanup_expired_sessions(&self, now: UnixTimestamp) -> Result<u64, RepositoryError> {
        self.repository.cleanup_expired_sessions(now)
    }

    /// Lists all sessions of a specific kind.
    ///
    /// # Errors
    ///
    /// Returns an error when the list cannot be retrieved.
    pub fn list_sessions_by_kind(
        &self,
        kind: SessionKind,
    ) -> Result<Vec<TransferSession>, RepositoryError> {
        self.repository.list_sessions_by_kind(kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct MockSessionRepository {
        sessions: Arc<std::sync::Mutex<Vec<TransferSession>>>,
    }

    impl MockSessionRepository {
        fn new() -> Self {
            Self {
                sessions: Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }
    }

    impl SessionRepository for MockSessionRepository {
        fn create_session(&self, session: TransferSession) -> Result<(), RepositoryError> {
            let mut sessions = self.sessions.lock().expect("lock poisoned");
            sessions.push(session);
            Ok(())
        }

        fn find_session(
            &self,
            token: &SessionToken,
        ) -> Result<Option<TransferSession>, RepositoryError> {
            let sessions = self.sessions.lock().expect("lock poisoned");
            Ok(sessions.iter().find(|s| s.token == *token).cloned())
        }

        fn mark_session_used(
            &self,
            _token: &SessionToken,
            _used_at: UnixTimestamp,
        ) -> Result<(), RepositoryError> {
            Ok(())
        }

        fn cleanup_expired_sessions(&self, _before: UnixTimestamp) -> Result<u64, RepositoryError> {
            Ok(0)
        }

        fn list_sessions_by_kind(
            &self,
            _kind: SessionKind,
        ) -> Result<Vec<TransferSession>, RepositoryError> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn create_send_session_generates_valid_session() {
        let repo = MockSessionRepository::new();
        let service = SessionService::new(repo);

        let token = SessionToken::new("aaaaaaaaaaaaaaaaaaaaaa").expect("invalid token");
        let now = UnixTimestamp::new(1000).expect("invalid timestamp");
        let session = service
            .create_send_session(token, "file-001", 3600, now)
            .expect("failed to create session");

        assert_eq!(session.kind, SessionKind::Send);
        assert_eq!(session.created_at, now);
        assert!(session.expires_at > now);
        assert!(session.source_file.is_some());
    }

    #[test]
    fn create_receive_session_generates_valid_session() {
        let repo = MockSessionRepository::new();
        let service = SessionService::new(repo);

        let token = SessionToken::new("bbbbbbbbbbbbbbbbbbbbbb").expect("invalid token");
        let now = UnixTimestamp::new(1000).expect("invalid timestamp");
        let session = service
            .create_receive_session(token, 3600, now)
            .expect("failed to create session");

        assert_eq!(session.kind, SessionKind::Receive);
        assert_eq!(session.created_at, now);
        assert!(session.expires_at > now);
        assert!(session.source_file.is_none());
    }

    #[test]
    fn get_session_returns_created_session() {
        let repo = MockSessionRepository::new();
        let service = SessionService::new(repo);

        let token = SessionToken::new("cccccccccccccccccccccc").expect("invalid token");
        let now = UnixTimestamp::new(1000).expect("invalid timestamp");
        let created = service
            .create_send_session(token.clone(), "file-001", 3600, now)
            .expect("failed to create session");

        let found = service
            .get_session(&created.token)
            .expect("failed to get session")
            .expect("session not found");

        assert_eq!(found.token, created.token);
        assert_eq!(found.kind, created.kind);
    }

    #[test]
    fn use_session_marks_session_as_used() {
        let repo = MockSessionRepository::new();
        let service = SessionService::new(repo);

        let token = SessionToken::new("dddddddddddddddddddddd").expect("invalid token");
        let now = UnixTimestamp::new(1000).expect("invalid timestamp");
        let session = service
            .create_send_session(token, "file-001", 3600, now)
            .expect("failed to create session");

        let used_at = UnixTimestamp::new(1500).expect("invalid timestamp");
        service
            .use_session(&session.token, used_at)
            .expect("failed to use session");
    }

    #[test]
    fn cleanup_expired_sessions_succeeds() {
        let repo = MockSessionRepository::new();
        let service = SessionService::new(repo);

        let now = UnixTimestamp::new(2000).expect("invalid timestamp");
        let _deleted = service
            .cleanup_expired_sessions(now)
            .expect("failed to cleanup");
    }

    #[test]
    fn list_sessions_by_kind_succeeds() {
        let repo = MockSessionRepository::new();
        let service = SessionService::new(repo);

        let sessions = service
            .list_sessions_by_kind(SessionKind::Send)
            .expect("failed to list");
        assert_eq!(sessions.len(), 0);
    }
}
