//! Authenticated request context.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::Json;
use tssp_domain::{UserId, UserRole};

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AuthRequiredResponse {
    error: AuthRequiredBody,
}

#[derive(Debug, Serialize)]
struct AuthRequiredBody {
    code: &'static str,
    message: String,
}

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

    /// Synthetic admin identity used when authentication is disabled or not required.
    ///
    /// # Panics
    ///
    /// Panics if `UserId::new` fails on the static literal (should never happen).
    #[must_use]
    #[allow(clippy::expect_used)]
    pub fn open_access() -> Self {
        Self {
            user_id: UserId::new("user-local").expect("static local user id"),
            role: UserRole::Admin,
            session_token: None,
            device_token: None,
        }
    }
}

impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<AuthRequiredResponse>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<Self>().cloned().ok_or((
            StatusCode::UNAUTHORIZED,
            Json(AuthRequiredResponse {
                error: AuthRequiredBody {
                    code: "unauthorized",
                    message: "authentication required".to_owned(),
                },
            }),
        ))
    }
}

/// Optional authenticated context (for uploads on open local networks).
#[derive(Debug, Clone)]
pub struct OptionalAuthContext(pub Option<AuthContext>);

impl<S> FromRequestParts<S> for OptionalAuthContext
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(parts.extensions.get::<AuthContext>().cloned()))
    }
}
