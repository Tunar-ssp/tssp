//! Authenticated request context.

use tssp_domain::{UserId, UserRole};

/// Resolved identity for an authenticated HTTP request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthContext {
    /// Authenticated user.
    pub user_id: UserId,
    /// Role for authorization checks.
    pub role: UserRole,
    /// Session bearer token, if any.
    pub session_token: Option<String>,
    /// Trusted device token, if any.
    pub device_token: Option<String>,
}

impl AuthContext {
    /// Returns true when the caller has admin privileges.
    #[must_use]
    pub const fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin)
    }
}
