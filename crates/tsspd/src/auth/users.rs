//! Multi-user accounts (name + code hash, roles).

use std::path::Path;
use std::sync::Mutex;

use bcrypt::{hash, verify, DEFAULT_COST};
use rusqlite::{params, Connection};
use thiserror::Error;
use tssp_domain::{UserId, UserName, UserRole};

const MIGRATION_VERSION: i64 = 5;

/// Errors from the user store.
#[derive(Debug, Error)]
pub enum UserStoreError {
    /// Database failure.
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    /// User not found.
    #[error("user not found")]
    NotFound,
    /// Name already taken.
    #[error("user name already exists")]
    NameTaken,
    /// Invalid input.
    #[error("{0}")]
    Invalid(String),
}

/// Stored user row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserRecord {
    /// Stable id.
    pub id: UserId,
    /// Login display name.
    pub name: UserName,
    /// Role.
    pub role: UserRole,
    /// Created at (unix seconds).
    pub created_at: i64,
    /// Disabled at, if any.
    pub disabled_at: Option<i64>,
}

/// Thread-safe user store.
#[derive(Debug)]
pub struct UserStore {
    connection: Mutex<Connection>,
}

impl UserStore {
    /// Opens user tables in the metadata database.
    ///
    /// # Errors
    ///
    /// Returns an error when migration fails.
    pub fn open(path: &Path) -> Result<Self, UserStoreError> {
        let connection = Connection::open(path)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        run_users_migration(&connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    /// Returns the number of users.
    ///
    /// # Errors
    ///
    /// Returns an error when the query fails.
    pub fn count_users(&self) -> Result<u64, UserStoreError> {
        let connection = self.lock()?;
        let count: i64 =
            connection.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
        Ok(u64::try_from(count).unwrap_or(0))
    }

    /// Creates a user with a plaintext code (stored as bcrypt hash only).
    ///
    /// # Errors
    ///
    /// Returns an error when the name exists or insert fails.
    pub fn create_user(
        &self,
        id: &UserId,
        name: &UserName,
        role: UserRole,
        code: &str,
        created_at: i64,
    ) -> Result<UserRecord, UserStoreError> {
        if code.trim().len() < 4 {
            return Err(UserStoreError::Invalid(
                "access code must be at least 4 characters".to_owned(),
            ));
        }
        let code_hash = hash(code, DEFAULT_COST)
            .map_err(|error| UserStoreError::Invalid(format!("could not hash code: {error}")))?;
        let connection = self.lock()?;
        let result = connection.execute(
            "INSERT INTO users (id, name, name_lower, role, code_hash, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                id.as_str(),
                name.as_str(),
                name.as_str().to_ascii_lowercase(),
                role.as_str(),
                code_hash,
                created_at,
            ],
        );
        if let Err(rusqlite::Error::SqliteFailure(err, _)) = result {
            if err.code == rusqlite::ErrorCode::ConstraintViolation {
                return Err(UserStoreError::NameTaken);
            }
            return Err(UserStoreError::Database(rusqlite::Error::SqliteFailure(
                err, None,
            )));
        }
        result?;
        Ok(UserRecord {
            id: id.clone(),
            name: name.clone(),
            role,
            created_at,
            disabled_at: None,
        })
    }

    /// Verifies name + code and returns the user.
    ///
    /// # Errors
    ///
    /// Returns [`UserStoreError::NotFound`] when credentials are wrong.
    pub fn verify_credentials(&self, name: &str, code: &str) -> Result<UserRecord, UserStoreError> {
        let connection = self.lock()?;
        let name_lower = name.trim().to_ascii_lowercase();
        let mut statement = connection.prepare(
            "SELECT id, name, role, code_hash, created_at, disabled_at
             FROM users WHERE name_lower = ?1 LIMIT 1",
        )?;
        let row = statement
            .query_row(params![name_lower], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                ))
            })
            .map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => UserStoreError::NotFound,
                other => UserStoreError::Database(other),
            })?;
        if row.5.is_some() {
            return Err(UserStoreError::NotFound);
        }
        let valid = verify(code, &row.3).unwrap_or(false);
        if !valid {
            return Err(UserStoreError::NotFound);
        }
        let id = UserId::new(row.0).map_err(|e| UserStoreError::Invalid(e.to_string()))?;
        let name = UserName::new(row.1).map_err(|e| UserStoreError::Invalid(e.to_string()))?;
        let role = UserRole::parse(&row.2).map_err(|e| UserStoreError::Invalid(e.to_string()))?;
        Ok(UserRecord {
            id,
            name,
            role,
            created_at: row.4,
            disabled_at: row.5,
        })
    }

    /// Lists all users.
    ///
    /// # Errors
    ///
    /// Returns an error when the query fails.
    pub fn list_users(&self) -> Result<Vec<UserRecord>, UserStoreError> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(
            "SELECT id, name, role, created_at, disabled_at FROM users ORDER BY name_lower",
        )?;
        let mut rows = statement.query([])?;
        let mut users = Vec::new();
        while let Some(row) = rows.next()? {
            let id = UserId::new(row.get::<_, String>(0)?)
                .map_err(|e| UserStoreError::Invalid(e.to_string()))?;
            let name = UserName::new(row.get::<_, String>(1)?)
                .map_err(|e| UserStoreError::Invalid(e.to_string()))?;
            let role = UserRole::parse(&row.get::<_, String>(2)?)
                .map_err(|e| UserStoreError::Invalid(e.to_string()))?;
            users.push(UserRecord {
                id,
                name,
                role,
                created_at: row.get(3)?,
                disabled_at: row.get(4)?,
            });
        }
        Ok(users)
    }

    /// Finds a user by id.
    ///
    /// # Errors
    ///
    /// Returns an error when lookup fails.
    pub fn find_user(&self, id: &UserId) -> Result<Option<UserRecord>, UserStoreError> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare("SELECT id, name, role, created_at, disabled_at FROM users WHERE id = ?1")?;
        let mut rows = statement.query(params![id.as_str()])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        let id = UserId::new(row.get::<_, String>(0)?)
            .map_err(|e| UserStoreError::Invalid(e.to_string()))?;
        let name = UserName::new(row.get::<_, String>(1)?)
            .map_err(|e| UserStoreError::Invalid(e.to_string()))?;
        let role = UserRole::parse(&row.get::<_, String>(2)?)
            .map_err(|e| UserStoreError::Invalid(e.to_string()))?;
        Ok(Some(UserRecord {
            id,
            name,
            role,
            created_at: row.get(3)?,
            disabled_at: row.get(4)?,
        }))
    }

    /// Updates a user's role.
    ///
    /// # Errors
    ///
    /// Returns an error when the user does not exist or when the change would
    /// leave the system without an enabled admin.
    pub fn set_role(&self, id: &UserId, role: UserRole) -> Result<(), UserStoreError> {
        let connection = self.lock()?;
        if matches!(role, UserRole::User) {
            let admin_count: i64 = connection.query_row(
                "SELECT COUNT(*) FROM users WHERE role = 'admin' AND disabled_at IS NULL",
                [],
                |row| row.get(0),
            )?;
            let target_role: String = connection
                .query_row(
                    "SELECT role FROM users WHERE id = ?1",
                    params![id.as_str()],
                    |row| row.get(0),
                )
                .map_err(|_| UserStoreError::NotFound)?;
            if target_role == "admin" && admin_count <= 1 {
                return Err(UserStoreError::Invalid(
                    "cannot demote the last admin user".to_owned(),
                ));
            }
        }
        let changed = connection.execute(
            "UPDATE users SET role = ?1 WHERE id = ?2",
            params![role.as_str(), id.as_str()],
        )?;
        if changed == 0 {
            return Err(UserStoreError::NotFound);
        }
        Ok(())
    }

    /// Replaces a user's access code.
    ///
    /// # Errors
    ///
    /// Returns an error when the user does not exist.
    pub fn reset_code(&self, id: &UserId, code: &str) -> Result<(), UserStoreError> {
        if code.trim().len() < 4 {
            return Err(UserStoreError::Invalid(
                "access code must be at least 4 characters".to_owned(),
            ));
        }
        let code_hash = hash(code, DEFAULT_COST)
            .map_err(|error| UserStoreError::Invalid(format!("could not hash code: {error}")))?;
        let connection = self.lock()?;
        let changed = connection.execute(
            "UPDATE users SET code_hash = ?1 WHERE id = ?2",
            params![code_hash, id.as_str()],
        )?;
        if changed == 0 {
            return Err(UserStoreError::NotFound);
        }
        Ok(())
    }

    /// Deletes a user (not the last admin).
    ///
    /// # Errors
    ///
    /// Returns an error when delete is not allowed.
    pub fn delete_user(&self, id: &UserId) -> Result<(), UserStoreError> {
        let connection = self.lock()?;
        let admin_count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM users WHERE role = 'admin' AND disabled_at IS NULL",
            [],
            |row| row.get(0),
        )?;
        let target_role: String = connection
            .query_row(
                "SELECT role FROM users WHERE id = ?1",
                params![id.as_str()],
                |row| row.get(0),
            )
            .map_err(|_| UserStoreError::NotFound)?;
        if target_role == "admin" && admin_count <= 1 {
            return Err(UserStoreError::Invalid(
                "cannot delete the last admin user".to_owned(),
            ));
        }
        let changed =
            connection.execute("DELETE FROM users WHERE id = ?1", params![id.as_str()])?;
        if changed == 0 {
            return Err(UserStoreError::NotFound);
        }
        Ok(())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, UserStoreError> {
        self.connection
            .lock()
            .map_err(|_| UserStoreError::Database(rusqlite::Error::ExecuteReturnedResults))
    }
}

fn run_users_migration(connection: &Connection) -> Result<(), UserStoreError> {
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
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            name_lower TEXT NOT NULL UNIQUE,
            role TEXT NOT NULL CHECK (role IN ('admin', 'user')),
            code_hash TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            disabled_at INTEGER
        );
        CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
        ",
    )?;

    connection.execute(
        "INSERT OR IGNORE INTO schema_migrations (version) VALUES (?1)",
        params![MIGRATION_VERSION],
    )?;
    Ok(())
}
