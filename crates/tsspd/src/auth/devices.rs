//! Trusted device sessions for long-lived local login.

use std::path::Path;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection};
use thiserror::Error;
use tssp_domain::{UserId, UserRole};

const MIGRATION_VERSION: i64 = 6;

/// Errors from the trusted device store.
#[derive(Debug, Error)]
pub enum DeviceStoreError {
    /// Database failure.
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    /// Device not found.
    #[error("device not found")]
    NotFound,
}

/// Trusted device row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedDevice {
    /// Opaque device token (cookie value).
    pub device_token: String,
    /// Owning user.
    pub user_id: UserId,
    /// User role at issue time.
    pub role: UserRole,
    /// Friendly label from client.
    pub device_name: String,
    /// Last seen unix seconds.
    pub last_seen_at: i64,
    /// Created unix seconds.
    pub created_at: i64,
    /// Last IP address.
    pub last_ip: Option<String>,
    /// Last user agent.
    pub last_user_agent: Option<String>,
    /// Expiry unix seconds.
    pub expires_at: i64,
}

/// Thread-safe trusted device store.
#[derive(Debug, Clone)]
pub struct DeviceStore {
    pool: Pool<SqliteConnectionManager>,
}

impl DeviceStore {
    /// Creates a device store from an existing pool.
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    /// Opens device tables.
    ///
    /// # Errors
    ///
    /// Returns an error when migration fails.
    pub fn open(path: &Path) -> Result<Self, DeviceStoreError> {
        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::builder().max_size(10).build(manager).map_err(|e| {
            DeviceStoreError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
        })?;

        let connection = pool.get().map_err(|e| {
            DeviceStoreError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
        })?;

        run_devices_migration(&connection)?;
        Ok(Self { pool })
    }

    fn connect(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>, DeviceStoreError> {
        self.pool
            .get()
            .map_err(|_| DeviceStoreError::Database(rusqlite::Error::ExecuteReturnedResults))
    }

    /// Inserts a trusted device record.
    ///
    /// # Errors
    ///
    /// Returns an error when insert fails.
    pub fn insert_device(&self, device: &TrustedDevice) -> Result<(), DeviceStoreError> {
        let connection = self.connect()?;
        connection.execute(
            "INSERT INTO trusted_devices
             (device_token, user_id, role, device_name, last_seen_at, created_at, last_ip, last_user_agent, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                device.device_token,
                device.user_id.as_str(),
                device.role.as_str(),
                device.device_name,
                device.last_seen_at,
                device.created_at,
                device.last_ip,
                device.last_user_agent,
                device.expires_at,
            ],
        )?;
        Ok(())
    }

    /// Returns a valid device by token.
    ///
    /// # Errors
    ///
    /// Returns an error when lookup fails.
    pub fn find_valid(
        &self,
        token: &str,
        now: i64,
    ) -> Result<Option<TrustedDevice>, DeviceStoreError> {
        let connection = self.connect()?;
        let mut statement = connection.prepare(
            "SELECT device_token, user_id, role, device_name, last_seen_at, created_at, last_ip, last_user_agent, expires_at
             FROM trusted_devices WHERE device_token = ?1 AND expires_at > ?2 LIMIT 1",
        )?;
        let mut rows = statement.query(params![token, now])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        Ok(Some(map_device_row(row)?))
    }

    /// Updates last seen metadata.
    ///
    /// # Errors
    ///
    /// Returns an error when update fails.
    pub fn touch(
        &self,
        token: &str,
        now: i64,
        ip: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<(), DeviceStoreError> {
        let connection = self.connect()?;
        let changed = connection.execute(
            "UPDATE trusted_devices
             SET last_seen_at = ?1, last_ip = ?2, last_user_agent = ?3
             WHERE device_token = ?4",
            params![now, ip, user_agent, token],
        )?;
        if changed == 0 {
            return Err(DeviceStoreError::NotFound);
        }
        Ok(())
    }

    /// Lists devices, optionally filtered by user.
    ///
    /// # Errors
    ///
    /// Returns an error when the query fails.
    pub fn list_devices(
        &self,
        user_id: Option<&UserId>,
    ) -> Result<Vec<TrustedDevice>, DeviceStoreError> {
        let connection = self.connect()?;
        let (sql, params_vec): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match user_id {
            Some(id) => (
                "SELECT device_token, user_id, role, device_name, last_seen_at, created_at, last_ip, last_user_agent, expires_at
                 FROM trusted_devices WHERE user_id = ?1 ORDER BY last_seen_at DESC",
                vec![Box::new(id.as_str().to_owned())],
            ),
            None => (
                "SELECT device_token, user_id, role, device_name, last_seen_at, created_at, last_ip, last_user_agent, expires_at
                 FROM trusted_devices ORDER BY last_seen_at DESC",
                vec![],
            ),
        };
        let mut statement = connection.prepare(sql)?;
        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            params_vec.iter().map(std::convert::AsRef::as_ref).collect();
        let mut rows = statement.query(params_ref.as_slice())?;
        let mut devices = Vec::new();
        while let Some(row) = rows.next()? {
            devices.push(map_device_row(row)?);
        }
        Ok(devices)
    }

    /// Revokes one device.
    ///
    /// # Errors
    ///
    /// Returns an error when delete fails.
    pub fn revoke(&self, token: &str) -> Result<(), DeviceStoreError> {
        let connection = self.connect()?;
        let changed = connection.execute(
            "DELETE FROM trusted_devices WHERE device_token = ?1",
            params![token],
        )?;
        if changed == 0 {
            return Err(DeviceStoreError::NotFound);
        }
        Ok(())
    }

    /// Revokes all devices for a user.
    ///
    /// # Errors
    ///
    /// Returns an error when delete fails.
    pub fn revoke_all_for_user(&self, user_id: &UserId) -> Result<u64, DeviceStoreError> {
        let connection = self.connect()?;
        let removed = connection.execute(
            "DELETE FROM trusted_devices WHERE user_id = ?1",
            params![user_id.as_str()],
        )?;
        Ok(u64::try_from(removed).unwrap_or(0))
    }

    /// Removes expired devices.
    ///
    /// # Errors
    ///
    /// Returns an error when cleanup fails.
    pub fn cleanup_expired(&self, now: i64) -> Result<u64, DeviceStoreError> {
        let connection = self.connect()?;
        let removed = connection.execute(
            "DELETE FROM trusted_devices WHERE expires_at <= ?1",
            params![now],
        )?;
        Ok(u64::try_from(removed).unwrap_or(0))
    }
}

fn map_device_row(row: &rusqlite::Row<'_>) -> Result<TrustedDevice, DeviceStoreError> {
    let user_id = UserId::new(row.get::<_, String>(1)?).map_err(|error| {
        DeviceStoreError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(
            std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string()),
        )))
    })?;
    let role = UserRole::parse(&row.get::<_, String>(2)?).map_err(|error| {
        DeviceStoreError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(
            std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string()),
        )))
    })?;
    Ok(TrustedDevice {
        device_token: row.get(0)?,
        user_id,
        role,
        device_name: row.get(3)?,
        last_seen_at: row.get(4)?,
        created_at: row.get(5)?,
        last_ip: row.get(6)?,
        last_user_agent: row.get(7)?,
        expires_at: row.get(8)?,
    })
}

fn run_devices_migration(connection: &Connection) -> Result<(), DeviceStoreError> {
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
        CREATE TABLE IF NOT EXISTS trusted_devices (
            device_token TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            role TEXT NOT NULL,
            device_name TEXT NOT NULL,
            last_seen_at INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            last_ip TEXT,
            last_user_agent TEXT,
            expires_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_trusted_devices_user ON trusted_devices(user_id);
        CREATE INDEX IF NOT EXISTS idx_trusted_devices_expires ON trusted_devices(expires_at);
        ",
    )?;

    connection.execute(
        "INSERT OR IGNORE INTO schema_migrations (version) VALUES (?1)",
        params![MIGRATION_VERSION],
    )?;
    Ok(())
}
