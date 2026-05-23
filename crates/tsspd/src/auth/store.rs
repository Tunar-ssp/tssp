//! SQLite persistence for auth configuration and tokens.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection};
use thiserror::Error;

const MIGRATION_VERSION: i64 = 3;

/// Errors from the auth store.
#[derive(Debug, Error)]
pub enum AuthStoreError {
    /// Database failure.
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    /// Password hash is invalid.
    #[error("invalid password hash")]
    InvalidPasswordHash,
}

/// Thread-safe auth metadata store backed by SQLite.
#[derive(Debug)]
pub struct AuthStore {
    connection: Mutex<Connection>,
}

impl AuthStore {
    /// Opens (or creates) auth tables in the metadata database.
    ///
    /// # Errors
    ///
    /// Returns an error when the database cannot be opened or migrated.
    pub fn open(path: &Path) -> Result<Self, AuthStoreError> {
        let connection = Connection::open(path)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "synchronous", "NORMAL")?;
        run_auth_migration(&connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    /// Returns the stored bcrypt password hash, if configured.
    ///
    /// # Errors
    ///
    /// Returns an error when the query fails.
    pub fn password_hash(&self) -> Result<Option<String>, AuthStoreError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| AuthStoreError::Database(rusqlite::Error::ExecuteReturnedResults))?;
        let mut statement = connection.prepare(
            "SELECT value FROM auth_config WHERE key = 'password_hash' LIMIT 1",
        )?;
        let mut rows = statement.query([])?;
        if let Some(row) = rows.next()? {
            return Ok(Some(row.get(0)?));
        }
        Ok(None)
    }

    /// Stores a bcrypt password hash (replacing any previous value).
    ///
    /// # Errors
    ///
    /// Returns an error when the hash is empty or the write fails.
    pub fn set_password_hash(&self, hash: &str) -> Result<(), AuthStoreError> {
        if hash.trim().is_empty() {
            return Err(AuthStoreError::InvalidPasswordHash);
        }
        let connection = self
            .connection
            .lock()
            .map_err(|_| AuthStoreError::Database(rusqlite::Error::ExecuteReturnedResults))?;
        connection.execute(
            "INSERT INTO auth_config (key, value) VALUES ('password_hash', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![hash],
        )?;
        Ok(())
    }

    /// Inserts a bearer/session token with an expiry timestamp (UTC unix seconds).
    ///
    /// # Errors
    ///
    /// Returns an error when the insert fails.
    pub fn insert_token(
        &self,
        token: &str,
        kind: &str,
        created_at: i64,
        expires_at: i64,
    ) -> Result<(), AuthStoreError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| AuthStoreError::Database(rusqlite::Error::ExecuteReturnedResults))?;
        connection.execute(
            "INSERT INTO auth_tokens (token, kind, created_at, expires_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![token, kind, created_at, expires_at],
        )?;
        Ok(())
    }

    /// Returns true when the token exists and has not expired.
    ///
    /// # Errors
    ///
    /// Returns an error when the lookup fails.
    pub fn token_valid(&self, token: &str, now: i64) -> Result<bool, AuthStoreError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| AuthStoreError::Database(rusqlite::Error::ExecuteReturnedResults))?;
        let count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM auth_tokens WHERE token = ?1 AND expires_at > ?2",
            params![token, now],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Deletes a token (logout).
    ///
    /// # Errors
    ///
    /// Returns an error when the delete fails.
    pub fn revoke_token(&self, token: &str) -> Result<(), AuthStoreError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| AuthStoreError::Database(rusqlite::Error::ExecuteReturnedResults))?;
        connection.execute("DELETE FROM auth_tokens WHERE token = ?1", params![token])?;
        Ok(())
    }

    /// Removes expired tokens.
    ///
    /// # Errors
    ///
    /// Returns an error when cleanup fails.
    pub fn cleanup_expired(&self, now: i64) -> Result<u64, AuthStoreError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| AuthStoreError::Database(rusqlite::Error::ExecuteReturnedResults))?;
        let removed = connection.execute(
            "DELETE FROM auth_tokens WHERE expires_at <= ?1",
            params![now],
        )?;
        Ok(u64::try_from(removed).unwrap_or(0))
    }
}

fn run_auth_migration(connection: &Connection) -> Result<(), AuthStoreError> {
    let applied: i64 = connection.query_row(
        "SELECT COUNT(*) FROM schema_migrations WHERE version = ?1",
        params![MIGRATION_VERSION],
        |row| row.get(0),
    )?;
    if applied > 0 {
        return Ok(());
    }

    connection.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS auth_config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS auth_tokens (
            token TEXT PRIMARY KEY,
            kind TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            expires_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_auth_tokens_expires ON auth_tokens(expires_at);
        ",
    )?;
    connection.execute(
        "INSERT OR IGNORE INTO schema_migrations (version) VALUES (?1)",
        params![MIGRATION_VERSION],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::AuthStore;
    use std::path::PathBuf;

    fn temp_db() -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("metadata.sqlite3");
        rusqlite::Connection::open(&path)
            .expect("open")
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY);",
            )
            .expect("schema");
        (temp, path)
    }

    #[test]
    fn password_hash_round_trip() {
        let (_temp, path) = temp_db();
        let store = AuthStore::open(&path).expect("open");
        assert!(store.password_hash().expect("read").is_none());
        store
            .set_password_hash("$2b$12$abcdefghijklmnopqrstuv")
            .expect("write");
        let hash = store.password_hash().expect("read");
        assert_eq!(
            hash.as_deref(),
            Some("$2b$12$abcdefghijklmnopqrstuv")
        );
    }

    #[test]
    fn token_validity_respects_expiry() {
        let (_temp, path) = temp_db();
        let store = AuthStore::open(&path).expect("open");
        store
            .insert_token("tok-a", "api", 100, 200)
            .expect("insert");
        assert!(store.token_valid("tok-a", 150).expect("valid"));
        assert!(!store.token_valid("tok-a", 200).expect("expired"));
        assert!(!store.token_valid("missing", 150).expect("missing"));
    }
}
