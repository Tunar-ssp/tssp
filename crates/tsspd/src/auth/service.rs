//! Authentication policy and token lifecycle.

use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use bcrypt::verify;
use getrandom::getrandom;
use thiserror::Error;

use super::local_network::{client_ip, is_local_client};
use super::store::{AuthStore, AuthStoreError};

const WEB_SESSION_DAYS: i64 = 30;
const API_TOKEN_DAYS: i64 = 365;

/// Authentication errors surfaced to HTTP handlers.
#[derive(Debug, Error)]
pub enum AuthError {
    /// Password is not configured.
    #[error("authentication is not configured")]
    NotConfigured,
    /// Password verification failed.
    #[error("invalid password")]
    InvalidPassword,
    /// Store failure.
    #[error("{0}")]
    Store(#[from] AuthStoreError),
}

/// Shared authentication service.
#[derive(Clone, Debug)]
pub struct AuthService {
    store: Option<Arc<AuthStore>>,
    trust_forwarded: bool,
}

impl AuthService {
    /// Authentication disabled (open access everywhere).
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            store: None,
            trust_forwarded: false,
        }
    }

    /// Creates a service backed by the given store.
    #[must_use]
    pub fn new(store: Arc<AuthStore>, trust_forwarded: bool) -> Self {
        Self {
            store: Some(store),
            trust_forwarded,
        }
    }

    /// Returns true when a password hash is configured.
    ///
    /// # Errors
    ///
    /// Returns an error when the configuration cannot be read.
    pub fn is_enabled(&self) -> Result<bool, AuthStoreError> {
        let Some(store) = &self.store else {
            return Ok(false);
        };
        Ok(store.password_hash()?.is_some())
    }

    /// Whether remote clients must authenticate.
    ///
    /// # Errors
    ///
    /// Returns an error when configuration cannot be read.
    pub fn remote_auth_required(&self, client: IpAddr) -> Result<bool, AuthStoreError> {
        if !self.is_enabled()? {
            return Ok(false);
        }
        Ok(!is_local_client(client))
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

    /// Verifies a plaintext password and issues a new token.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] when authentication is disabled or the password is wrong.
    pub fn authenticate(
        &self,
        password: &str,
        kind: &str,
        now: i64,
    ) -> Result<String, AuthError> {
        let store = self.store.as_deref().ok_or(AuthError::NotConfigured)?;
        let Some(hash) = store.password_hash()? else {
            return Err(AuthError::NotConfigured);
        };
        let valid = verify(password, &hash).map_err(|_| AuthError::InvalidPassword)?;
        if !valid {
            return Err(AuthError::InvalidPassword);
        }
        let ttl_days = if kind == "api" {
            API_TOKEN_DAYS
        } else {
            WEB_SESSION_DAYS
        };
        let token = generate_token()?;
        let expires_at = now + ttl_days * 86_400;
        store.insert_token(&token, kind, now, expires_at)?;
        Ok(token)
    }

    /// Returns whether the bearer/cookie token is valid.
    ///
    /// # Errors
    ///
    /// Returns an error when validation fails.
    pub fn validate_token(&self, token: &str, now: i64) -> Result<bool, AuthStoreError> {
        if token.trim().is_empty() {
            return Ok(false);
        }
        let Some(store) = &self.store else {
            return Ok(false);
        };
        store.token_valid(token, now)
    }

    /// Revokes a token.
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

    /// Stores a bcrypt hash (used at startup from env).
    ///
    /// # Errors
    ///
    /// Returns an error when the hash cannot be stored.
    pub fn set_password_hash(&self, hash: &str) -> Result<(), AuthStoreError> {
        let store = self.store.as_deref().ok_or(AuthStoreError::InvalidPasswordHash)?;
        store.set_password_hash(hash)
    }

    /// Removes expired tokens.
    ///
    /// # Errors
    ///
    /// Returns an error when cleanup fails.
    pub fn cleanup_expired(&self, now: i64) -> Result<u64, AuthStoreError> {
        let Some(store) = &self.store else {
            return Ok(0);
        };
        store.cleanup_expired(now)
    }

    /// Cookie max-age for web sessions.
    #[must_use]
    pub const fn web_cookie_max_age() -> Duration {
        Duration::from_secs((WEB_SESSION_DAYS * 86_400) as u64)
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

    use bcrypt::{hash, DEFAULT_COST};

    use super::AuthService;
    use crate::auth::store::AuthStore;

    fn service_with_password(password: &str) -> (tempfile::TempDir, AuthService) {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("db.sqlite3");
        rusqlite::Connection::open(&path)
            .expect("open")
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY);",
            )
            .expect("schema");
        let store = Arc::new(AuthStore::open(&path).expect("auth store"));
        let hash = hash(password, DEFAULT_COST).expect("hash");
        store.set_password_hash(&hash).expect("set hash");
        (temp, AuthService::new(store, false))
    }

    #[test]
    fn disabled_service_never_requires_auth() {
        let auth = AuthService::disabled();
        let remote = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1));
        assert!(!auth.remote_auth_required(remote).expect("check"));
    }

    #[test]
    fn local_client_skips_remote_auth() {
        let (_temp, auth) = service_with_password("secret");
        let local = IpAddr::V4(Ipv4Addr::new(192, 168, 0, 5));
        assert!(!auth.remote_auth_required(local).expect("check"));
        let remote = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1));
        assert!(auth.remote_auth_required(remote).expect("check"));
    }

    #[test]
    fn authenticate_rejects_wrong_password() {
        let (_temp, auth) = service_with_password("secret");
        let result = auth.authenticate("wrong", "web", 1_000);
        assert!(matches!(result, Err(super::AuthError::InvalidPassword)));
    }

    #[test]
    fn authenticate_issues_valid_token() {
        let (_temp, auth) = service_with_password("secret");
        let token = auth.authenticate("secret", "api", 1_000).expect("token");
        assert!(auth.validate_token(&token, 1_500).expect("valid"));
        assert!(!auth.validate_token(&token, 999_999_999).expect("expired"));
    }
}
