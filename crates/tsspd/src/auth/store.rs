//! `SQLite` persistence for auth configuration and tokens.

use std::path::Path;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
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

/// Thread-safe auth metadata store backed by `SQLite`.
#[derive(Debug, Clone)]
pub struct AuthStore {
    pool: Pool<SqliteConnectionManager>,
}

impl AuthStore {
    /// Creates an auth store from an existing pool.
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    /// Opens (or creates) auth tables in the metadata database.
    ///
    /// # Errors
    ///
    /// Returns an error when the database cannot be opened or migrated.
    pub fn open(path: &Path) -> Result<Self, AuthStoreError> {
        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::builder()
            .max_size(10)
            .build(manager)
            .map_err(|e| AuthStoreError::Database(rusqlite::Error::from_error(Box::new(e))))?;

        let connection = pool
            .get()
            .map_err(|e| AuthStoreError::Database(rusqlite::Error::from_error(Box::new(e))))?;

        run_auth_migration(&connection)?;
        Ok(Self { pool })
    }

    fn connect(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>, AuthStoreError> {
        self.pool
            .get()
            .map_err(|e| AuthStoreError::Database(rusqlite::Error::from_error(Box::new(e))))
    }

    /// Returns the stored bcrypt password hash, if configured.
    ///
    /// # Errors
    ///
    /// Returns an error when the query fails.
    pub fn password_hash(&self) -> Result<Option<String>, AuthStoreError> {
        let connection = self.connect()?;
        let mut statement = connection
            .prepare("SELECT value FROM auth_config WHERE key = 'password_hash' LIMIT 1")?;
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
        let connection = self.connect()?;
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
        user_id: Option<&str>,
        device_id: Option<&str>,
        created_at: i64,
        expires_at: i64,
    ) -> Result<(), AuthStoreError> {
        let connection = self.connect()?;
        connection.execute(
            "INSERT INTO auth_tokens (token, kind, user_id, device_id, created_at, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![token, kind, user_id, device_id, created_at, expires_at],
        )?;
        Ok(())
    }

    /// Returns session identity for a valid token.
    ///
    /// # Errors
    ///
    /// Returns an error when the lookup fails.
    pub fn token_session(
        &self,
        token: &str,
        now: i64,
    ) -> Result<Option<(String, String, String)>, AuthStoreError> {
        let connection = self.connect()?;
        let mut statement = connection.prepare(
            "SELECT u.id, u.role, u.name
             FROM auth_tokens t
             JOIN users u ON u.id = t.user_id
             WHERE t.token = ?1 AND t.expires_at > ?2 AND u.disabled_at IS NULL
             LIMIT 1",
        )?;
        let mut rows = statement.query(params![token, now])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?)))
    }

    /// Returns true when the token exists and has not expired (legacy).
    ///
    /// # Errors
    ///
    /// Returns an error when the lookup fails.
    pub fn token_valid(&self, token: &str, now: i64) -> Result<bool, AuthStoreError> {
        if token.trim().is_empty() {
            return Ok(false);
        }
        let connection = self.connect()?;
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
        let connection = self.connect()?;
        connection.execute("DELETE FROM auth_tokens WHERE token = ?1", params![token])?;
        Ok(())
    }

    /// Removes expired tokens.
    ///
    /// # Errors
    ///
    /// Returns an error when cleanup fails.
    pub fn cleanup_expired(&self, now: i64) -> Result<u64, AuthStoreError> {
        let connection = self.connect()?;
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
            user_id TEXT,
            device_id TEXT,
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
