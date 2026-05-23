//! Authentication policy, user login, and trusted devices.

use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use bcrypt::verify;
use getrandom::getrandom;
use thiserror::Error;
use tssp_domain::{UserId, UserName, UserRole};

use super::devices::{DeviceStore, DeviceStoreError, TrustedDevice};
use super::local_network::{client_ip, is_local_client};
use super::store::{AuthStore, AuthStoreError};
use super::users::{UserRecord, UserStore, UserStoreError};

const WEB_SESSION_DAYS: i64 = 30;
const API_TOKEN_DAYS: i64 = 365;
const TRUSTED_DEVICE_DAYS: i64 = 180;

/// Session metadata returned after successful authentication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInfo {
    /// Bearer/session token.
    pub token: String,
    /// Authenticated user id.
    pub user_id: UserId,
    /// Display name.
    pub name: UserName,
    /// Role.
    pub role: UserRole,
    /// Trusted device token when "remember device" was requested.
    pub device_token: Option<String>,
}

/// Authentication errors surfaced to HTTP handlers.
#[derive(Debug, Error)]
pub enum AuthError {
    /// No users or legacy password configured.
    #[error("authentication is not configured")]
    NotConfigured,
    /// Credentials rejected.
    #[error("invalid credentials")]
    InvalidCredentials,
    /// Store failure.
    #[error("{0}")]
    Store(String),
}

impl From<AuthStoreError> for AuthError {
    fn from(error: AuthStoreError) -> Self {
        Self::Store(error.to_string())
    }
}

impl From<UserStoreError> for AuthError {
    fn from(error: UserStoreError) -> Self {
        match error {
            UserStoreError::NotFound => Self::InvalidCredentials,
            other => Self::Store(other.to_string()),
        }
    }
}

impl From<DeviceStoreError> for AuthError {
    fn from(error: DeviceStoreError) -> Self {
        Self::Store(error.to_string())
    }
}

/// Shared authentication service.
#[derive(Clone, Debug)]
pub struct AuthService {
    store: Option<Arc<AuthStore>>,
    users: Option<Arc<UserStore>>,
    devices: Option<Arc<DeviceStore>>,
    trust_forwarded: bool,
    /// When true, every client must authenticate (public/global domain mode).
    global_auth_required: bool,
}

impl AuthService {
    /// Authentication disabled (open access).
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            store: None,
            users: None,
            devices: None,
            trust_forwarded: false,
            global_auth_required: false,
        }
    }

    /// Creates a fully configured service.
    #[must_use]
    pub fn new(
        store: Arc<AuthStore>,
        users: Arc<UserStore>,
        devices: Arc<DeviceStore>,
        trust_forwarded: bool,
        global_auth_required: bool,
    ) -> Self {
        Self {
            store: Some(store),
            users: Some(users),
            devices: Some(devices),
            trust_forwarded,
            global_auth_required,
        }
    }

    /// Whether multi-user auth is active.
    ///
    /// # Errors
    ///
    /// Returns an error when user count cannot be read.
    pub fn users_enabled(&self) -> Result<bool, UserStoreError> {
        let Some(users) = &self.users else {
            return Ok(false);
        };
        Ok(users.count_users()? > 0)
    }

    /// Whether this client must authenticate for API access.
    ///
    /// # Errors
    ///
    /// Returns an error when configuration cannot be read.
    pub fn remote_auth_required(&self, client: IpAddr) -> Result<bool, AuthStoreError> {
        if self.global_auth_required {
            return Ok(true);
        }
        if !self.legacy_password_enabled()? && !self.users_enabled().map_err(|e| {
            AuthStoreError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(
                std::io::Error::other(e.to_string()),
            )))
        })? {
            return Ok(false);
        }
        Ok(!is_local_client(client))
    }

    fn legacy_password_enabled(&self) -> Result<bool, AuthStoreError> {
        let Some(store) = &self.store else {
            return Ok(false);
        };
        Ok(store.password_hash()?.is_some())
    }

    /// Resolves the effective client IP.
    #[must_use]
    pub fn resolve_client(
        &self,
        peer: IpAddr,
        forwarded_for: Option<&str>,
    ) -> IpAddr {
        client_ip(peer, forwarded_for, self.trust_forwarded)
    }

    /// Legacy single-password login (used when no users exist).
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] when authentication fails.
    pub fn authenticate_legacy_password(
        &self,
        password: &str,
        kind: &str,
        now: i64,
    ) -> Result<String, AuthError> {
        let store = self.store.as_deref().ok_or(AuthError::NotConfigured)?;
        let Some(hash) = store.password_hash()? else {
            return Err(AuthError::NotConfigured);
        };
        let valid = verify(password, &hash).map_err(|_| AuthError::InvalidCredentials)?;
        if !valid {
            return Err(AuthError::InvalidCredentials);
        }
        let token = generate_token()?;
        let expires_at = now + token_ttl_days(kind) * 86_400;
        store.insert_token(&token, kind, None, None, now, expires_at)?;
        Ok(token)
    }

    /// User login with name + access code.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] when credentials are invalid.
    #[allow(clippy::too_many_arguments)]
    pub fn authenticate_user(
        &self,
        name: &str,
        code: &str,
        kind: &str,
        remember_device: bool,
        device_name: &str,
        now: i64,
        client_ip: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<SessionInfo, AuthError> {
        let users = self.users.as_deref().ok_or(AuthError::NotConfigured)?;
        let store = self.store.as_deref().ok_or(AuthError::NotConfigured)?;
        let devices = self.devices.as_deref().ok_or(AuthError::NotConfigured)?;
        let user = users.verify_credentials(name, code)?;
        let token = generate_token()?;
        let expires_at = now + token_ttl_days(kind) * 86_400;
        store.insert_token(
            &token,
            kind,
            Some(user.id.as_str()),
            None,
            now,
            expires_at,
        )?;

        let device_token = if remember_device {
            let device_token = generate_token()?;
            let device = TrustedDevice {
                device_token: device_token.clone(),
                user_id: user.id.clone(),
                role: user.role,
                device_name: if device_name.trim().is_empty() {
                    "Trusted device".to_owned()
                } else {
                    device_name.trim().to_owned()
                },
                last_seen_at: now,
                created_at: now,
                last_ip: client_ip.map(str::to_owned),
                last_user_agent: user_agent.map(str::to_owned),
                expires_at: now + TRUSTED_DEVICE_DAYS * 86_400,
            };
            devices.insert_device(&device)?;
            Some(device_token)
        } else {
            None
        };

        Ok(SessionInfo {
            token,
            user_id: user.id,
            name: user.name,
            role: user.role,
            device_token,
        })
    }

    /// Validates bearer token and returns session info.
    ///
    /// # Errors
    ///
    /// Returns an error when validation fails.
    pub fn resolve_token(&self, token: &str, now: i64) -> Result<Option<SessionInfo>, AuthError> {
        if token.trim().is_empty() {
            return Ok(None);
        }
        let store = self.store.as_deref().ok_or(AuthError::NotConfigured)?;
        let Some((user_id, role, name)) = store.token_session(token, now)? else {
            return Ok(None);
        };
        let user_id = UserId::new(user_id).map_err(|e| AuthError::Store(e.to_string()))?;
        let role = UserRole::parse(&role).map_err(|e| AuthError::Store(e.to_string()))?;
        let name = UserName::new(name).map_err(|e| AuthError::Store(e.to_string()))?;
        Ok(Some(SessionInfo {
            token: token.to_owned(),
            user_id,
            name,
            role,
            device_token: None,
        }))
    }

    /// Validates trusted device cookie on local network.
    ///
    /// # Errors
    ///
    /// Returns an error when lookup fails.
    pub fn resolve_device(
        &self,
        device_token: &str,
        now: i64,
        client_ip: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<Option<SessionInfo>, AuthError> {
        let devices = self.devices.as_deref().ok_or(AuthError::NotConfigured)?;
        let users = self.users.as_deref().ok_or(AuthError::NotConfigured)?;
        let Some(device) = devices.find_valid(device_token, now)? else {
            return Ok(None);
        };
        let _ = devices.touch(device_token, now, client_ip, user_agent);
        let user = users
            .find_user(&device.user_id)?
            .ok_or(AuthError::InvalidCredentials)?;
        Ok(Some(SessionInfo {
            token: String::new(),
            user_id: user.id,
            name: user.name,
            role: user.role,
            device_token: Some(device_token.to_owned()),
        }))
    }

    /// Revokes a session token.
    ///
    /// # Errors
    ///
    /// Returns an error when revocation fails.
    pub fn revoke_token(&self, token: &str) -> Result<(), AuthStoreError> {
        let Some(store) = &self.store else {
            return Ok(());
        };
        store.revoke_token(token)
    }

    /// Revokes a trusted device.
    ///
    /// # Errors
    ///
    /// Returns an error when revocation fails.
    pub fn revoke_device(&self, token: &str) -> Result<(), DeviceStoreError> {
        let Some(devices) = &self.devices else {
            return Ok(());
        };
        devices.revoke(token)
    }

    /// Stores a legacy bcrypt hash (bootstrap / env).
    ///
    /// # Errors
    ///
    /// Returns an error when the hash cannot be stored.
    pub fn set_password_hash(&self, hash: &str) -> Result<(), AuthStoreError> {
        let store = self.store.as_deref().ok_or(AuthStoreError::InvalidPasswordHash)?;
        store.set_password_hash(hash)
    }

    /// Removes expired tokens and devices.
    ///
    /// # Errors
    ///
    /// Returns an error when cleanup fails.
    pub fn cleanup_expired(&self, now: i64) -> Result<(u64, u64), AuthError> {
        let tokens = self
            .store
            .as_deref()
            .map(|store| store.cleanup_expired(now))
            .transpose()?
            .unwrap_or(0);
        let devices = self
            .devices
            .as_deref()
            .map(|devices| devices.cleanup_expired(now))
            .transpose()
            .map_err(AuthError::from)?
            .unwrap_or(0);
        Ok((tokens, devices))
    }

    /// User store for admin handlers.
    #[must_use]
    pub fn users(&self) -> Option<&UserStore> {
        self.users.as_deref()
    }

    /// Device store for admin handlers.
    #[must_use]
    pub fn devices(&self) -> Option<&DeviceStore> {
        self.devices.as_deref()
    }

    /// Auth token store.
    #[must_use]
    pub fn store(&self) -> Option<&AuthStore> {
        self.store.as_deref()
    }

    /// Cookie max-age for web sessions.
    #[must_use]
    pub const fn web_cookie_max_age() -> Duration {
        Duration::from_secs((WEB_SESSION_DAYS * 86_400) as u64)
    }

    pub(crate) const fn trusted_device_max_age() -> Duration {
        Duration::from_secs((TRUSTED_DEVICE_DAYS * 86_400) as u64)
    }
}

fn token_ttl_days(kind: &str) -> i64 {
    if kind == "api" {
        API_TOKEN_DAYS
    } else {
        WEB_SESSION_DAYS
    }
}

fn generate_token() -> Result<String, AuthStoreError> {
    let mut bytes = [0_u8; 32];
    getrandom(&mut bytes).map_err(|error| {
        AuthStoreError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
    })?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use std::sync::Arc;

    use super::AuthService;
    use crate::auth::devices::DeviceStore;
    use crate::auth::store::AuthStore;
    use crate::auth::users::UserStore;
    use tssp_domain::{UserId, UserName, UserRole};

    fn temp_service() -> (tempfile::TempDir, AuthService) {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("db.sqlite3");
        rusqlite::Connection::open(&path)
            .expect("open")
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY);",
            )
            .expect("schema");
        let store = Arc::new(AuthStore::open(&path).expect("auth"));
        let users = Arc::new(UserStore::open(&path).expect("users"));
        let devices = Arc::new(DeviceStore::open(&path).expect("devices"));
        let id = UserId::new("user-tunar").expect("id");
        let name = UserName::new("Tunar").expect("name");
        users
            .create_user(&id, &name, UserRole::Admin, "secret-code", 1_000)
            .expect("user");
        (
            temp,
            AuthService::new(store, users, devices, false, false),
        )
    }

    #[test]
    fn user_login_issues_session() {
        let (_temp, auth) = temp_service();
        let session = auth
            .authenticate_user("Tunar", "secret-code", "web", false, "", 1_000, None, None)
            .expect("login");
        assert_eq!(session.name.as_str(), "Tunar");
        assert!(auth
            .resolve_token(&session.token, 1_500)
            .expect("resolve")
            .is_some());
    }

    #[test]
    fn global_mode_requires_auth_even_on_lan() {
        let (_temp, auth) = temp_service();
        let global = AuthService::new(
            auth.store.clone(),
            Arc::new(UserStore::open(_temp.path().join("db.sqlite3")).expect("users")),
            Arc::new(DeviceStore::open(_temp.path().join("db.sqlite3")).expect("devices")),
            false,
            true,
        );
        let local = IpAddr::V4(Ipv4Addr::new(192, 168, 0, 2));
        assert!(global.remote_auth_required(local).expect("check"));
    }
}
