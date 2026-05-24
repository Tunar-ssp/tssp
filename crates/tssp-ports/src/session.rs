//! Session repository port trait.

use std::sync::Arc;

use tssp_domain::{SessionKind, SessionToken, TransferSession, UnixTimestamp};

use tssp_domain::FileId;

use crate::errors::RepositoryError;

/// Persists and queries transfer sessions.
pub trait SessionRepository: Send + Sync {
    /// Creates a new transfer session.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the session cannot be persisted.
    fn create_session(&self, session: TransferSession) -> Result<(), RepositoryError>;

    /// Retrieves a session by token.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the query fails.
    fn find_session(
        &self,
        token: &SessionToken,
    ) -> Result<Option<TransferSession>, RepositoryError>;

    /// Marks a session as used.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the update cannot be committed.
    fn mark_session_used(
        &self,
        token: &SessionToken,
        used_at: UnixTimestamp,
    ) -> Result<(), RepositoryError>;

    /// Associates a file with a receive session (records what was uploaded).
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the update cannot be committed.
    fn set_received_file(
        &self,
        token: &SessionToken,
        file_id: &FileId,
    ) -> Result<(), RepositoryError>;

    /// Deletes expired sessions.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the cleanup fails.
    fn cleanup_expired_sessions(&self, before: UnixTimestamp) -> Result<u64, RepositoryError>;

    /// Lists sessions of a specific kind.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the query fails.
    fn list_sessions_by_kind(
        &self,
        kind: SessionKind,
    ) -> Result<Vec<TransferSession>, RepositoryError>;
}

impl<T> SessionRepository for Arc<T>
where
    T: SessionRepository,
{
    fn create_session(&self, session: TransferSession) -> Result<(), RepositoryError> {
        self.as_ref().create_session(session)
    }

    fn find_session(
        &self,
        token: &SessionToken,
    ) -> Result<Option<TransferSession>, RepositoryError> {
        self.as_ref().find_session(token)
    }

    fn mark_session_used(
        &self,
        token: &SessionToken,
        used_at: UnixTimestamp,
    ) -> Result<(), RepositoryError> {
        self.as_ref().mark_session_used(token, used_at)
    }

    fn set_received_file(
        &self,
        token: &SessionToken,
        file_id: &FileId,
    ) -> Result<(), RepositoryError> {
        self.as_ref().set_received_file(token, file_id)
    }

    fn cleanup_expired_sessions(&self, before: UnixTimestamp) -> Result<u64, RepositoryError> {
        self.as_ref().cleanup_expired_sessions(before)
    }

    fn list_sessions_by_kind(
        &self,
        kind: SessionKind,
    ) -> Result<Vec<TransferSession>, RepositoryError> {
        self.as_ref().list_sessions_by_kind(kind)
    }
}
