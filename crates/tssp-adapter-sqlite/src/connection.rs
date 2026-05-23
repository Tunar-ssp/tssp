//! `SQLite` connection setup and integrity verification.

use rusqlite::Connection;

use crate::SqliteRepositoryError;

pub(crate) fn configure_connection(connection: &Connection) -> Result<(), SqliteRepositoryError> {
    connection
        .pragma_update(None, "journal_mode", "WAL")
        .map_err(SqliteRepositoryError::Configure)?;
    connection
        .pragma_update(None, "synchronous", "NORMAL")
        .map_err(SqliteRepositoryError::Configure)?;
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .map_err(SqliteRepositoryError::Configure)?;
    connection
        .busy_timeout(std::time::Duration::from_secs(5))
        .map_err(SqliteRepositoryError::Configure)
}

pub(crate) fn run_integrity_check(connection: &Connection) -> Result<(), SqliteRepositoryError> {
    let result: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(SqliteRepositoryError::Configure)?;
    if result == "ok" {
        return Ok(());
    }

    Err(SqliteRepositoryError::Integrity { message: result })
}
