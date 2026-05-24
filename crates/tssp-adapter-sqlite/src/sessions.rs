//! `SQLite` implementation of `SessionRepository`.

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Row};
use tssp_domain::{FileId, FileName, SessionKind, SessionToken, TransferSession, UnixTimestamp};
use tssp_ports::{RepositoryError, SessionRepository};

use crate::map_rusqlite_repository_error;

/// Maps a session row to a `TransferSession` domain object.
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
#[derive(Debug, Clone)]
pub struct SqliteSessionRepository {
    pool: Pool<SqliteConnectionManager>,
}

impl SqliteSessionRepository {
    /// Creates a new session repository with a connection pool.
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    fn connect(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>, RepositoryError> {
        self.pool
            .get()
            .map_err(|error| RepositoryError::OperationFailed {
                message: format!("session connection pool failure: {error}"),
            })
    }
}

impl SessionRepository for SqliteSessionRepository {
    fn create_session(&self, session: TransferSession) -> Result<(), RepositoryError> {
        let connection = self.connect()?;
        connection
            .execute(
                "INSERT INTO sessions (token, kind, created_at, expires_at, source_file, received_file, expected_name)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    session.token.as_str(),
                    match session.kind {
                        SessionKind::Send => "send",
                        SessionKind::Receive => "receive",
                    },
                    session.created_at.seconds(),
                    session.expires_at.seconds(),
                    session.source_file.as_ref().map(|id| id.as_str()),
                    session.received_file.as_ref().map(|id| id.as_str()),
                    session.expected_name.as_ref().map(|name| name.original()),
                ],
            )
            .map_err(map_rusqlite_repository_error)?;
        Ok(())
    }

    fn find_session(&self, token: &SessionToken) -> Result<Option<TransferSession>, RepositoryError> {
        let connection = self.connect()?;
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
        map_session_row(row).map(Some)
    }

    fn update_session(&self, session: &TransferSession) -> Result<(), RepositoryError> {
        let connection = self.connect()?;
        let changed = connection
            .execute(
                "UPDATE sessions
                 SET received_file = ?1
                 WHERE token = ?2",
                params![
                    session.received_file.as_ref().map(|id| id.as_str()),
                    session.token.as_str()
                ],
            )
            .map_err(map_rusqlite_repository_error)?;
        if changed == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    fn delete_session(&self, token: &SessionToken) -> Result<(), RepositoryError> {
        let connection = self.connect()?;
        let changed = connection
            .execute("DELETE FROM sessions WHERE token = ?1", params![token.as_str()])
            .map_err(map_rusqlite_repository_error)?;
        if changed == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    fn reap_expired_sessions(&self, now: UnixTimestamp) -> Result<u64, RepositoryError> {
        let connection = self.connect()?;
        let changed = connection
            .execute("DELETE FROM sessions WHERE expires_at < ?1", params![now.seconds()])
            .map_err(map_rusqlite_repository_error)?;
        Ok(u64::try_from(changed).unwrap_or(0))
    }
}
