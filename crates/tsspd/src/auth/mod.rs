//! Authentication: users, roles, trusted devices, and dual-mode access.

mod context;
mod devices;
mod handlers;
#[cfg(test)]
mod integration_tests;
mod local_network;
mod middleware;
mod service;
mod store;
mod users;
#[cfg(test)]
mod users_test;

use rusqlite::Connection;
use thiserror::Error;

pub use context::{AuthContext, OptionalAuthContext};
pub use devices::{DeviceStore, DeviceStoreError, TrustedDevice};
pub use handlers::{
    auth_login, auth_logout, auth_me, auth_required, auth_token, LoginRequest, TokenResponse,
};
pub use middleware::auth_middleware;
pub use service::{AuthError, AuthService, SessionInfo};
pub use store::{AuthStore, AuthStoreError};
pub use users::UserRecord;
pub use users::{UserStore, UserStoreError};

/// Auth database initialization failures.
#[derive(Debug, Error)]
pub enum AuthDatabaseError {
    /// Auth store migration failed.
    #[error("auth store migration failed: {0}")]
    Store(#[from] store::AuthStoreError),

    /// User store migration failed.
    #[error("user store migration failed: {0}")]
    Users(#[from] users::UserStoreError),

    /// Trusted device migration failed.
    #[error("trusted device migration failed: {0}")]
    Devices(#[from] devices::DeviceStoreError),
}

/// Initializes the auth tables on an existing `SQLite` connection.
///
/// This must run after the shared metadata schema has created `schema_migrations`.
///
/// # Errors
///
/// Returns an error if any auth table migration fails.
pub fn initialize_database(connection: &Connection) -> Result<(), AuthDatabaseError> {
    store::initialize_store_schema(connection)?;
    users::initialize_user_schema(connection)?;
    devices::initialize_device_schema(connection)?;
    Ok(())
}
