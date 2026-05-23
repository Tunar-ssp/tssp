//! SQLite implementation of SessionRepository.

use rusqlite::{params, Row};
use tssp_domain::{FileId, FileName, SessionKind, SessionToken, TransferSession, UnixTimestamp};
use tssp_ports::{RepositoryError, SessionRepository};

use crate::map_rusqlite_repository_error;

/// Maps a session row to a TransferSession domain object.
#[allow(dead_code)]
fn map_session_row(row: &Row<'_>) -> Result<TransferSession, RepositoryError> {
    let token_str: String = row.get(0).map_err(|e| RepositoryError::OperationFailed {
        message: format!("failed to read session token: {e}"),
    })?;
    let token = SessionToken::new(&token_str).map_err(|e| RepositoryError::OperationFailed {
        message: format!("invalid session token in database: {e}"),
    })?;

    let kind_str: String = row.get(1).map_err(|e| RepositoryError::OperationFailed {
        message: format!("failed to read session kind: {e}"),
    })?;
    let kind = match kind_str.as_str() {
        "send" => SessionKind::Send,
        "receive" => SessionKind::Receive,
        _ => {
            return Err(RepositoryError::OperationFailed {
                message: format!("invalid session kind in database: {kind_str}"),
            })
        }
    };

    let created_at_secs: i64 = row.get(2).map_err(|e| RepositoryError::OperationFailed {
        message: format!("failed to read session created_at: {e}"),
    })?;
    let created_at =
        UnixTimestamp::new(created_at_secs).map_err(|e| RepositoryError::OperationFailed {
            message: format!("invalid session created_at: {e}"),
        })?;

    let expires_at_secs: i64 = row.get(3).map_err(|e| RepositoryError::OperationFailed {
        message: format!("failed to read session expires_at: {e}"),
    })?;
    let expires_at =
        UnixTimestamp::new(expires_at_secs).map_err(|e| RepositoryError::OperationFailed {
            message: format!("invalid session expires_at: {e}"),
        })?;

    let source_file: Option<String> = row.get(4).map_err(|e| RepositoryError::OperationFailed {
        message: format!("failed to read session source_file: {e}"),
    })?;
    let source_file = source_file.as_deref().and_then(|s| FileId::new(s).ok());

    let received_file: Option<String> =
        row.get(5).map_err(|e| RepositoryError::OperationFailed {
            message: format!("failed to read session received_file: {e}"),
        })?;
    let received_file = received_file.as_deref().and_then(|s| FileId::new(s).ok());

    let expected_name: Option<String> =
        row.get(6).map_err(|e| RepositoryError::OperationFailed {
            message: format!("failed to read session expected_name: {e}"),
        })?;
    let expected_name = expected_name.as_deref().and_then(|s| FileName::new(s).ok());

    let mut session = TransferSession::new(token, kind, created_at, expires_at, source_file)
        .map_err(|e| RepositoryError::OperationFailed {
            message: format!("failed to construct session from database: {e}"),
        })?;

    if let Some(rf) = received_file {
        session.received_file = Some(rf);
    }
    if let Some(en) = expected_name {
        session.expected_name = Some(en);
    }

    Ok(session)
}

/// SQLite-backed session repository.
#[allow(dead_code)]
pub struct SqliteSessionRepository {
    connection: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
}

impl SqliteSessionRepository {
    /// Creates a new session repository with a shared database connection.
    pub fn new(connection: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>) -> Self {
        Self { connection }
    }
}

impl SessionRepository for SqliteSessionRepository {
    fn create_session(&self, session: TransferSession) -> Result<(), RepositoryError> {
        let connection = self
            .connection
            .lock()
            .map_err(|e| RepositoryError::OperationFailed {
                message: format!("session connection lock is poisoned: {e}"),
            })?;

        let kind_str = match session.kind {
            SessionKind::Send => "send",
            SessionKind::Receive => "receive",
        };

        connection
            .execute(
                "INSERT INTO sessions (token, kind, created_at, expires_at, source_file, received_file, expected_name, used_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    session.token.as_str(),
                    kind_str,
                    session.created_at.seconds(),
                    session.expires_at.seconds(),
                    session.source_file.as_ref().map(|f| f.as_str()),
                    session.received_file.as_ref().map(|f| f.as_str()),
                    session.expected_name.as_ref().map(|n| n.original()),
                    None::<i64>,
                ],
            )
            .map_err(map_rusqlite_repository_error)?;

        Ok(())
    }

    fn find_session(
        &self,
        token: &SessionToken,
    ) -> Result<Option<TransferSession>, RepositoryError> {
        let connection = self
            .connection
            .lock()
            .map_err(|e| RepositoryError::OperationFailed {
                message: format!("session connection lock is poisoned: {e}"),
            })?;

        let mut statement = connection
            .prepare(
                "SELECT token, kind, created_at, expires_at, source_file, received_file, expected_name
                 FROM sessions
                 WHERE token = ?1",
            )
            .map_err(map_rusqlite_repository_error)?;

        let mut rows = statement
            .query(params![token.as_str()])
            .map_err(map_rusqlite_repository_error)?;

        let Some(row) = rows.next().map_err(map_rusqlite_repository_error)? else {
            return Ok(None);
        };

        let session = map_session_row(row)?;
        Ok(Some(session))
    }

    fn mark_session_used(
        &self,
        token: &SessionToken,
        used_at: UnixTimestamp,
    ) -> Result<(), RepositoryError> {
        let connection = self
            .connection
            .lock()
            .map_err(|e| RepositoryError::OperationFailed {
                message: format!("session connection lock is poisoned: {e}"),
            })?;

        connection
            .execute(
                "UPDATE sessions SET used_at = ?1 WHERE token = ?2",
                params![used_at.seconds(), token.as_str()],
            )
            .map_err(map_rusqlite_repository_error)?;

        Ok(())
    }

    fn cleanup_expired_sessions(&self, before: UnixTimestamp) -> Result<u64, RepositoryError> {
        let connection = self
            .connection
            .lock()
            .map_err(|e| RepositoryError::OperationFailed {
                message: format!("session connection lock is poisoned: {e}"),
            })?;

        connection
            .execute(
                "DELETE FROM sessions WHERE expires_at < ?1",
                params![before.seconds()],
            )
            .map_err(map_rusqlite_repository_error)
            .map(|count| count as u64)
    }

    fn list_sessions_by_kind(
        &self,
        kind: SessionKind,
    ) -> Result<Vec<TransferSession>, RepositoryError> {
        let connection = self
            .connection
            .lock()
            .map_err(|e| RepositoryError::OperationFailed {
                message: format!("session connection lock is poisoned: {e}"),
            })?;

        let kind_str = match kind {
            SessionKind::Send => "send",
            SessionKind::Receive => "receive",
        };

        let mut statement = connection
            .prepare(
                "SELECT token, kind, created_at, expires_at, source_file, received_file, expected_name
                 FROM sessions
                 WHERE kind = ?1",
            )
            .map_err(map_rusqlite_repository_error)?;

        let mut rows = statement
            .query(params![kind_str])
            .map_err(map_rusqlite_repository_error)?;

        let mut sessions = Vec::new();
        while let Some(row) = rows.next().map_err(map_rusqlite_repository_error)? {
            sessions.push(map_session_row(row)?);
        }

        Ok(sessions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tssp_domain::SessionKind;

    fn init_test_db() -> std::sync::Arc<std::sync::Mutex<rusqlite::Connection>> {
        let conn = rusqlite::Connection::open_in_memory().expect("failed to open connection");
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                token TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                source_file TEXT,
                received_file TEXT,
                expected_name TEXT,
                used_at INTEGER
            );
            CREATE INDEX IF NOT EXISTS sessions_expires_at ON sessions(expires_at);
            CREATE INDEX IF NOT EXISTS sessions_kind ON sessions(kind);",
        )
        .expect("failed to initialize test db");
        std::sync::Arc::new(std::sync::Mutex::new(conn))
    }

    #[test]
    fn create_and_find_session() {
        let repo = SqliteSessionRepository::new(init_test_db());

        let token = SessionToken::new("aaaaaaaaaaaaaaaaaaaaaa").expect("invalid token");
        let now = UnixTimestamp::new(1000).expect("invalid timestamp");
        let expires = UnixTimestamp::new(2000).expect("invalid timestamp");
        let file_id = FileId::new("file-001").expect("invalid file id");

        let session = TransferSession::new(
            token.clone(),
            SessionKind::Send,
            now,
            expires,
            Some(file_id),
        )
        .expect("failed to create session");

        repo.create_session(session).expect("create failed");
        let found = repo
            .find_session(&token)
            .expect("find failed")
            .expect("session not found");

        assert_eq!(found.token, token);
        assert_eq!(found.kind, SessionKind::Send);
    }

    #[test]
    fn mark_session_used() {
        let repo = SqliteSessionRepository::new(init_test_db());

        let token = SessionToken::new("bbbbbbbbbbbbbbbbbbbbbb").expect("invalid token");
        let now = UnixTimestamp::new(1000).expect("invalid timestamp");
        let expires = UnixTimestamp::new(2000).expect("invalid timestamp");
        let file_id = FileId::new("file-002").expect("invalid file id");

        let session = TransferSession::new(
            token.clone(),
            SessionKind::Send,
            now,
            expires,
            Some(file_id),
        )
        .expect("failed to create session");

        repo.create_session(session).expect("create failed");

        let used_at = UnixTimestamp::new(1500).expect("invalid timestamp");
        repo.mark_session_used(&token, used_at)
            .expect("mark used failed");

        let found = repo
            .find_session(&token)
            .expect("find failed")
            .expect("session not found");

        assert_eq!(found.token, token);
    }

    #[test]
    fn cleanup_expired_sessions() {
        let repo = SqliteSessionRepository::new(init_test_db());

        let token1 = SessionToken::new("cccccccccccccccccccccc").expect("invalid token");
        let token2 = SessionToken::new("dddddddddddddddddddddd").expect("invalid token");
        let now = UnixTimestamp::new(1000).expect("invalid timestamp");
        let expires_soon = UnixTimestamp::new(1500).expect("invalid timestamp");
        let expires_later = UnixTimestamp::new(3000).expect("invalid timestamp");

        let file_id1 = FileId::new("file-003").expect("invalid file id");
        let file_id2 = FileId::new("file-004").expect("invalid file id");

        let session1 = TransferSession::new(
            token1.clone(),
            SessionKind::Send,
            now,
            expires_soon,
            Some(file_id1),
        )
        .expect("failed to create session");
        let session2 = TransferSession::new(
            token2.clone(),
            SessionKind::Send,
            now,
            expires_later,
            Some(file_id2),
        )
        .expect("failed to create session");

        repo.create_session(session1).expect("create failed");
        repo.create_session(session2).expect("create failed");

        let cutoff = UnixTimestamp::new(2000).expect("invalid timestamp");
        let deleted = repo
            .cleanup_expired_sessions(cutoff)
            .expect("cleanup failed");

        assert_eq!(deleted, 1);

        let found1 = repo.find_session(&token1).expect("find failed");
        let found2 = repo.find_session(&token2).expect("find failed");

        assert!(found1.is_none());
        assert!(found2.is_some());
    }

    #[test]
    fn list_sessions_by_kind() {
        let repo = SqliteSessionRepository::new(init_test_db());

        let token_send = SessionToken::new("eeeeeeeeeeeeeeeeeeeeee").expect("invalid token");
        let token_recv = SessionToken::new("ffffffffffffffffffffff").expect("invalid token");
        let now = UnixTimestamp::new(1000).expect("invalid timestamp");
        let expires = UnixTimestamp::new(2000).expect("invalid timestamp");
        let file_id = FileId::new("file-005").expect("invalid file id");

        let session_send =
            TransferSession::new(token_send, SessionKind::Send, now, expires, Some(file_id))
                .expect("failed to create session");
        let session_recv =
            TransferSession::new(token_recv, SessionKind::Receive, now, expires, None)
                .expect("failed to create session");

        repo.create_session(session_send).expect("create failed");
        repo.create_session(session_recv).expect("create failed");

        let send_sessions = repo
            .list_sessions_by_kind(SessionKind::Send)
            .expect("list failed");
        let recv_sessions = repo
            .list_sessions_by_kind(SessionKind::Receive)
            .expect("list failed");

        assert_eq!(send_sessions.len(), 1);
        assert_eq!(recv_sessions.len(), 1);
        assert_eq!(send_sessions[0].kind, SessionKind::Send);
        assert_eq!(recv_sessions[0].kind, SessionKind::Receive);
    }
}
