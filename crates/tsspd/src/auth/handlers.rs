//! HTTP handlers for user authentication and sessions.

use axum::extract::{ConnectInfo, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use tssp_adapter_system::SystemClock;
use tssp_app::AuditAction;
use tssp_ports::Clock;

use crate::auth::service::{AuthError, AuthService};
use crate::{ErrorBody, ErrorResponse, HttpState};

pub(crate) const SESSION_COOKIE: &str = "tssp_session";
pub(crate) const DEVICE_COOKIE: &str = "tssp_device";

/// Login request body.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// User display name (multi-user mode).
    pub name: Option<String>,
    /// Access code (multi-user mode).
    pub code: Option<String>,
    /// Legacy shared password (single-user bootstrap).
    pub password: Option<String>,
    /// Remember this device on the local network.
    #[serde(default)]
    pub remember_device: bool,
    /// Optional device label.
    #[serde(default)]
    pub device_name: Option<String>,
}

/// Token response for CLI clients.
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    /// Stable schema version.
    pub schema_version: u8,
    /// Bearer token value.
    pub token: String,
    /// Authenticated user name.
    pub name: String,
    /// Role (`admin` or `user`).
    pub role: String,
}

/// Whether authentication is required for the current client.
#[derive(Debug, Serialize)]
pub struct AuthRequiredResponse {
    /// Stable schema version.
    pub schema_version: u8,
    /// True when this client must authenticate.
    pub required: bool,
    /// True when multi-user login is active.
    pub users_enabled: bool,
}

/// Session probe response.
#[derive(Debug, Serialize)]
pub struct AuthMeResponse {
    /// Stable schema version.
    pub schema_version: u8,
    /// Whether the caller is authenticated for remote access.
    pub authenticated: bool,
    /// User name when authenticated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Role when authenticated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

fn now_seconds() -> i64 {
    SystemClock.now().seconds()
}

#[derive(Debug)]
pub enum LoginError {
    RateLimited,
    Auth(AuthError),
}

fn map_login_error(error: LoginError) -> (StatusCode, Json<ErrorResponse>) {
    match error {
        LoginError::RateLimited => (
            StatusCode::TOO_MANY_REQUESTS,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "rate_limited",
                    message: "too many login attempts, try again later".to_owned(),
                },
            }),
        ),
        LoginError::Auth(error) => map_auth_error(error),
    }
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
        AuthError::InvalidCredentials => (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "invalid_credentials",
                    message: "invalid name or code".to_owned(),
                },
            }),
        ),
        AuthError::Store(message) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "auth_store_error",
                    message,
                },
            }),
        ),
    }
}

fn session_cookie_value(name: &str, token: &str, max_age_secs: u64) -> String {
    format!("{name}={token}; HttpOnly; Path=/; Max-Age={max_age_secs}; SameSite=Strict")
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

pub(crate) fn extract_cookie_token(headers: &HeaderMap, cookie_name: &str) -> Option<String> {
    let prefix = format!("{cookie_name}=");
    headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|part| {
                let part = part.trim();
                part.strip_prefix(&prefix)
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
    let users_enabled = state.auth.users_enabled().unwrap_or(false);
    (
        StatusCode::OK,
        Json(AuthRequiredResponse {
            schema_version: 2,
            required,
            users_enabled,
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
                schema_version: 2,
                authenticated: true,
                name: None,
                role: None,
            }),
        )
            .into_response();
    }

    let now = now_seconds();
    let session = extract_bearer(&headers)
        .or_else(|| extract_cookie_token(&headers, SESSION_COOKIE))
        .and_then(|token| state.auth.resolve_token(&token, now).ok().flatten());
    let device = extract_cookie_token(&headers, DEVICE_COOKIE).and_then(|token| {
        state
            .auth
            .resolve_device(
                &token,
                now,
                Some(&client.to_string()),
                headers
                    .get(header::USER_AGENT)
                    .and_then(|v| v.to_str().ok()),
            )
            .ok()
            .flatten()
    });

    if let Some(info) = session.or(device) {
        return (
            StatusCode::OK,
            Json(AuthMeResponse {
                schema_version: 2,
                authenticated: true,
                name: Some(info.name.as_str().to_owned()),
                role: Some(info.role.as_str().to_owned()),
            }),
        )
            .into_response();
    }

    StatusCode::UNAUTHORIZED.into_response()
}

async fn perform_login(
    state: &HttpState,
    body: &LoginRequest,
    kind: &str,
    peer: SocketAddr,
    headers: &HeaderMap,
    set_cookie: bool,
) -> Result<Response, LoginError> {
    let ip = peer.ip();

    if !state.rate_limiter.check_and_record_attempt(ip).await {
        return Err(LoginError::RateLimited);
    }

    let now = now_seconds();
    let client_ip = Some(peer.ip().to_string());
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);

    let users_enabled = state.auth.users_enabled().unwrap_or(false);

    let session = if users_enabled {
        let name = body
            .name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty());
        let code = body
            .code
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty());
        match (name, code) {
            (Some(name), Some(code)) => state.auth.authenticate_user(
                name,
                code,
                kind,
                body.remember_device,
                body.device_name.as_deref().unwrap_or(""),
                now,
                client_ip.as_deref(),
                user_agent.as_deref(),
            ),
            _ => Err(AuthError::InvalidCredentials),
        }
    } else if let Some(password) = body
        .password
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        state
            .auth
            .authenticate_legacy_password(password, kind, now)
            .and_then(legacy_session_info)
    } else {
        Err(AuthError::NotConfigured)
    };

    let session = match session {
        Ok(value) => {
            state.rate_limiter.record_success(ip).await;
            tssp_app::log_audit_event(
                state.repository.as_ref(),
                AuditAction::Login,
                Some(&value.user_id),
                None,
                None,
                "success",
                None,
            );
            value
        }
        Err(error) => {
            state.rate_limiter.record_failure(ip).await;
            return Err(LoginError::Auth(error));
        }
    };

    let mut response = (
        StatusCode::OK,
        Json(TokenResponse {
            schema_version: 2,
            token: session.token.clone(),
            name: session.name.as_str().to_owned(),
            role: session.role.as_str().to_owned(),
        }),
    )
        .into_response();

    if set_cookie {
        if let Ok(value) = header::HeaderValue::from_str(&session_cookie_value(
            SESSION_COOKIE,
            &session.token,
            AuthService::web_cookie_max_age().as_secs(),
        )) {
            response.headers_mut().append(header::SET_COOKIE, value);
        }
        if let Some(device_token) = &session.device_token {
            if let Ok(value) = header::HeaderValue::from_str(&session_cookie_value(
                DEVICE_COOKIE,
                device_token,
                AuthService::trusted_device_max_age().as_secs(),
            )) {
                response.headers_mut().append(header::SET_COOKIE, value);
            }
        }
    }
    Ok(response)
}

fn legacy_session_info(token: String) -> Result<super::service::SessionInfo, AuthError> {
    let user_id = tssp_domain::UserId::new("user-legacy")
        .map_err(|error| AuthError::Store(format!("legacy user id is invalid: {error}")))?;
    let name = tssp_domain::UserName::new("legacy")
        .map_err(|error| AuthError::Store(format!("legacy user name is invalid: {error}")))?;
    Ok(super::service::SessionInfo {
        token,
        user_id,
        name,
        role: tssp_domain::UserRole::Admin,
        device_token: None,
    })
}

/// Web login: sets session cookie and returns token.
pub async fn auth_login(
    State(state): State<HttpState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<LoginRequest>,
) -> Response {
    match perform_login(&state, &body, "web", peer, &headers, true).await {
        Ok(response) => response,
        Err(error) => map_login_error(error).into_response(),
    }
}

/// CLI token exchange (Bearer only, no cookie).
pub async fn auth_token(
    State(state): State<HttpState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<LoginRequest>,
) -> impl IntoResponse {
    match perform_login(&state, &body, "api", peer, &headers, false).await {
        Ok(response) => response,
        Err(error) => map_login_error(error).into_response(),
    }
}

/// Logout: revokes cookie/bearer token when present.
pub async fn auth_logout(State(state): State<HttpState>, headers: HeaderMap) -> impl IntoResponse {
    let now = now_seconds();
    if let Some(token) =
        extract_bearer(&headers).or_else(|| extract_cookie_token(&headers, SESSION_COOKIE))
    {
        if let Ok(Some(session_info)) = state.auth.resolve_token(&token, now) {
            tssp_app::log_audit_event(
                state.repository.as_ref(),
                AuditAction::Logout,
                Some(&session_info.user_id),
                None,
                None,
                "success",
                None,
            );
        }
        let _ = state.auth.revoke_token(&token);
    }
    if let Some(device) = extract_cookie_token(&headers, DEVICE_COOKIE) {
        let _ = state.auth.revoke_device(&device);
    }
    let mut response = StatusCode::NO_CONTENT.into_response();
    for cookie in [SESSION_COOKIE, DEVICE_COOKIE] {
        if let Ok(value) = header::HeaderValue::from_str(&session_cookie_value(cookie, "", 0)) {
            response.headers_mut().append(header::SET_COOKIE, value);
        }
    }
    response
}

/// Device info for listing
#[derive(Debug, serde::Serialize)]
pub struct DeviceInfo {
    pub token: String,
    pub name: String,
    pub created_at: i64,
    pub last_seen_at: i64,
    pub last_ip: Option<String>,
}

/// List user's own trusted devices
pub async fn list_user_devices(
    State(state): State<HttpState>,
    auth: crate::auth::AuthContext,
) -> impl IntoResponse {
    match state.auth.list_user_devices(&auth.user_id) {
        Ok(devices) => {
            let device_infos: Vec<DeviceInfo> = devices
                .into_iter()
                .map(|d| DeviceInfo {
                    token: d.device_token,
                    name: d.device_name,
                    created_at: d.created_at,
                    last_seen_at: d.last_seen_at,
                    last_ip: d.last_ip,
                })
                .collect();
            (StatusCode::OK, Json(device_infos)).into_response()
        }
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "failed_to_list_devices",
                "message": error.to_string()
            })),
        )
            .into_response(),
    }
}

/// Revoke a trusted device (user can only revoke their own)
pub async fn revoke_user_device(
    State(state): State<HttpState>,
    auth: crate::auth::AuthContext,
    axum::extract::Path(device_token): axum::extract::Path<String>,
) -> impl IntoResponse {
    // Verify the device belongs to the current user
    match state.auth.get_device(&device_token) {
        Ok(device) => {
            if device.user_id != auth.user_id {
                return (
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({
                        "error": "forbidden",
                        "message": "You can only revoke your own devices"
                    })),
                )
                    .into_response();
            }

            // Revoke the device
            match state.auth.revoke_device(&device_token) {
                Ok(()) => {
                    tssp_app::log_audit_event(
                        state.repository.as_ref(),
                        AuditAction::DeviceRevoke,
                        Some(&auth.user_id),
                        Some("device"),
                        Some(&device_token),
                        "success",
                        None,
                    );
                    StatusCode::NO_CONTENT.into_response()
                }
                Err(error) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "failed_to_revoke_device",
                        "message": error.to_string()
                    })),
                )
                    .into_response(),
            }
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "device_not_found",
                "message": "Device not found"
            })),
        )
            .into_response(),
    }
}
