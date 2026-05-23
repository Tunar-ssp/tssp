//! HTTP handlers for dual-mode authentication.

use axum::extract::{ConnectInfo, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use tssp_adapter_system::SystemClock;
use tssp_ports::Clock;

use crate::auth::service::{AuthError, AuthService};
use crate::{ErrorBody, ErrorResponse, HttpState};

pub(crate) const SESSION_COOKIE: &str = "tssp_session";

/// Login request body.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// Shared access password.
    pub password: String,
}

/// Token response for CLI clients.
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    /// Stable schema version.
    pub schema_version: u8,
    /// Bearer token value.
    pub token: String,
}

/// Whether authentication is required for the current client.
#[derive(Debug, Serialize)]
pub struct AuthRequiredResponse {
    /// Stable schema version.
    pub schema_version: u8,
    /// True when this client must authenticate.
    pub required: bool,
}

/// Session probe response.
#[derive(Debug, Serialize)]
pub struct AuthMeResponse {
    /// Stable schema version.
    pub schema_version: u8,
    /// Whether the caller is authenticated for remote access.
    pub authenticated: bool,
}

fn now_seconds() -> i64 {
    SystemClock.now().seconds()
}

fn map_auth_error(error: AuthError) -> (StatusCode, Json<ErrorResponse>) {
    match error {
        AuthError::NotConfigured => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "auth_not_configured",
                    message: "authentication is not configured on this server".to_owned(),
                },
            }),
        ),
        AuthError::InvalidPassword => (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "invalid_credentials",
                    message: "invalid password".to_owned(),
                },
            }),
        ),
        AuthError::Store(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "auth_store_error",
                    message: error.to_string(),
                },
            }),
        ),
    }
}

fn session_cookie_value(token: &str) -> String {
    let max_age = AuthService::web_cookie_max_age().as_secs();
    format!(
        "{SESSION_COOKIE}={token}; HttpOnly; Path=/; Max-Age={max_age}; SameSite=Strict"
    )
}

pub(crate) fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_owned)
}

pub(crate) fn extract_cookie_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|part| {
                let part = part.trim();
                part.strip_prefix(&format!("{SESSION_COOKIE}="))
                    .map(str::trim)
                    .filter(|token| !token.is_empty())
                    .map(str::to_owned)
            })
        })
}

/// Returns whether the caller must authenticate.
pub async fn auth_required(
    State(state): State<HttpState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let client = state.auth.resolve_client(
        peer.ip(),
        headers
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok()),
    );
    let required = match state.auth.remote_auth_required(client) {
        Ok(value) => value,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "auth_check_failed",
                        message: error.to_string(),
                    },
                }),
            )
                .into_response();
        }
    };
    (
        StatusCode::OK,
        Json(AuthRequiredResponse {
            schema_version: 1,
            required,
        }),
    )
        .into_response()
}

/// Reports whether the current session is valid.
pub async fn auth_me(
    State(state): State<HttpState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let client = state.auth.resolve_client(
        peer.ip(),
        headers
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok()),
    );
    let required = match state.auth.remote_auth_required(client) {
        Ok(value) => value,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "auth_check_failed",
                        message: error.to_string(),
                    },
                }),
            )
                .into_response();
        }
    };
    if !required {
        return (
            StatusCode::OK,
            Json(AuthMeResponse {
                schema_version: 1,
                authenticated: true,
            }),
        )
            .into_response();
    }

    let token = extract_bearer(&headers).or_else(|| extract_cookie_token(&headers));
    let authenticated = match token {
        Some(token) => state
            .auth
            .validate_token(&token, now_seconds())
            .unwrap_or(false),
        None => false,
    };
    if !authenticated {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    (
        StatusCode::OK,
        Json(AuthMeResponse {
            schema_version: 1,
            authenticated: true,
        }),
    )
        .into_response()
}

/// Web login: sets session cookie and returns token for API clients.
pub async fn auth_login(
    State(state): State<HttpState>,
    Json(body): Json<LoginRequest>,
) -> Response {
    let now = now_seconds();
    let token = match state.auth.authenticate(body.password.trim(), "web", now) {
        Ok(token) => token,
        Err(error) => return map_auth_error(error).into_response(),
    };
    let mut response = (
        StatusCode::OK,
        Json(TokenResponse {
            schema_version: 1,
            token: token.clone(),
        }),
    )
        .into_response();
    if let Ok(value) = header::HeaderValue::from_str(&session_cookie_value(&token)) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
    response
}

/// CLI token exchange (Bearer only, no cookie).
pub async fn auth_token(
    State(state): State<HttpState>,
    Json(body): Json<LoginRequest>,
) -> impl IntoResponse {
    let now = now_seconds();
    match state.auth.authenticate(body.password.trim(), "api", now) {
        Ok(token) => (
            StatusCode::OK,
            Json(TokenResponse {
                schema_version: 1,
                token,
            }),
        )
            .into_response(),
        Err(error) => map_auth_error(error).into_response(),
    }
}

/// Logout: revokes cookie/bearer token when present.
pub async fn auth_logout(State(state): State<HttpState>, headers: HeaderMap) -> impl IntoResponse {
    if let Some(token) = extract_bearer(&headers).or_else(|| extract_cookie_token(&headers)) {
        let _ = state.auth.revoke_token(&token);
    }
    let mut response = StatusCode::NO_CONTENT.into_response();
    if let Ok(value) = header::HeaderValue::from_str(&format!(
        "{SESSION_COOKIE}=; HttpOnly; Path=/; Max-Age=0; SameSite=Strict"
    )) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
    response
}
