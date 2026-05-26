//! Admin session and trusted device handlers.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use tssp_domain::UserId;
use tssp_ports::Clock;

use tssp_app::{log_audit_event, AuditAction};

use crate::auth::AuthContext;
use crate::{ErrorBody, ErrorResponse, HttpState};
use tssp_adapter_system::SystemClock;

#[derive(Debug, Serialize)]
struct DeviceJson {
    device_token: String,
    user_id: String,
    role: String,
    device_name: String,
    last_seen_at: i64,
    created_at: i64,
    last_ip: Option<String>,
    last_user_agent: Option<String>,
    expires_at: i64,
}

#[derive(Debug, Serialize)]
struct DeviceListResponse {
    schema_version: u8,
    devices: Vec<DeviceJson>,
}

#[derive(Debug, Deserialize)]
pub struct AdminSessionsQuery {
    #[serde(default = "default_session_limit")]
    pub limit: u64,
    #[serde(default)]
    pub user_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct AdminSessionJson {
    token: String,
    token_preview: String,
    kind: String,
    user_id: Option<String>,
    user_name: Option<String>,
    role: Option<String>,
    created_at: i64,
    expires_at: i64,
    current: bool,
}

#[derive(Debug, Serialize)]
struct AdminSessionListResponse {
    schema_version: u8,
    sessions: Vec<AdminSessionJson>,
}

fn default_session_limit() -> u64 {
    100
}

fn session_token_preview(token: &str) -> String {
    let prefix: String = token.chars().take(12).collect();
    if token.chars().count() > 12 {
        format!("{prefix}...")
    } else {
        prefix
    }
}

fn map_device(device: &crate::auth::TrustedDevice) -> DeviceJson {
    DeviceJson {
        device_token: device.device_token.clone(),
        user_id: device.user_id.as_str().to_owned(),
        role: device.role.as_str().to_owned(),
        device_name: device.device_name.clone(),
        last_seen_at: device.last_seen_at,
        created_at: device.created_at,
        last_ip: device.last_ip.clone(),
        last_user_agent: device.last_user_agent.clone(),
        expires_at: device.expires_at,
    }
}

/// `GET /api/v1/admin/sessions`
pub async fn admin_list_sessions(
    State(state): State<HttpState>,
    auth: AuthContext,
    Query(params): Query<AdminSessionsQuery>,
) -> impl IntoResponse {
    let now = SystemClock.now().seconds();
    let user_id = match params.user_id {
        Some(id) => match UserId::new(id) {
            Ok(value) => Some(value),
            Err(error) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: ErrorBody {
                            code: "invalid_user_id",
                            message: error.to_string(),
                        },
                    }),
                )
                    .into_response();
            }
        },
        None => None,
    };

    match state
        .auth
        .list_active_sessions(now, user_id.as_ref(), params.limit)
    {
        Ok(sessions) => (
            StatusCode::OK,
            Json(AdminSessionListResponse {
                schema_version: 1,
                sessions: sessions
                    .into_iter()
                    .map(|session| AdminSessionJson {
                        token_preview: session_token_preview(&session.token),
                        current: auth.session_token.as_deref() == Some(session.token.as_str()),
                        token: session.token,
                        kind: session.kind,
                        user_id: session.user_id,
                        user_name: session.user_name,
                        role: session.role,
                        created_at: session.created_at,
                        expires_at: session.expires_at,
                    })
                    .collect(),
            }),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "sessions_list_failed",
                    message: error.to_string(),
                },
            }),
        )
            .into_response(),
    }
}

/// `DELETE /api/v1/admin/sessions/{token}`
pub async fn admin_revoke_session(
    State(state): State<HttpState>,
    auth: AuthContext,
    Path(token): Path<String>,
) -> impl IntoResponse {
    match state.auth.revoke_token_existing(&token) {
        Ok(true) => {
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::SessionRevoke,
                Some(&auth.user_id),
                Some("session"),
                Some(&session_token_preview(&token)),
                "success",
                None,
            );
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::SessionRevoke,
                Some(&auth.user_id),
                Some("session"),
                Some(&session_token_preview(&token)),
                "failure",
                Some(&error.to_string()),
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "revoke_session_failed",
                        message: error.to_string(),
                    },
                }),
            )
                .into_response()
        }
    }
}

/// `DELETE /api/v1/admin/users/{id}/sessions`
pub async fn admin_revoke_user_sessions(
    State(state): State<HttpState>,
    auth: AuthContext,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let user_id = match UserId::new(id) {
        Ok(value) => value,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "invalid_user_id",
                        message: error.to_string(),
                    },
                }),
            )
                .into_response();
        }
    };
    match state.auth.revoke_all_sessions_for_user(&user_id) {
        Ok(removed) => {
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::SessionRevoke,
                Some(&auth.user_id),
                Some("user_sessions"),
                Some(user_id.as_str()),
                "success",
                Some(&format!("revoked {removed} sessions")),
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "schema_version": 1,
                    "removed": removed,
                })),
            )
                .into_response()
        }
        Err(error) => {
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::SessionRevoke,
                Some(&auth.user_id),
                Some("user_sessions"),
                Some(user_id.as_str()),
                "failure",
                Some(&error.to_string()),
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "revoke_user_sessions_failed",
                        message: error.to_string(),
                    },
                }),
            )
                .into_response()
        }
    }
}

/// `GET /api/v1/admin/devices`
pub async fn admin_list_devices(
    State(state): State<HttpState>,
    _auth: AuthContext,
) -> impl IntoResponse {
    let Some(devices) = state.auth.devices() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "devices_unavailable",
                    message: "device store unavailable".to_owned(),
                },
            }),
        )
            .into_response();
    };
    match devices.list_devices(None) {
        Ok(list) => (
            StatusCode::OK,
            Json(DeviceListResponse {
                schema_version: 1,
                devices: list.iter().map(map_device).collect(),
            }),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "devices_list_failed",
                    message: error.to_string(),
                },
            }),
        )
            .into_response(),
    }
}

/// `DELETE /api/v1/admin/devices/{token}`
pub async fn admin_revoke_device(
    State(state): State<HttpState>,
    auth: AuthContext,
    Path(token): Path<String>,
) -> impl IntoResponse {
    let Some(devices) = state.auth.devices() else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    match devices.revoke(&token) {
        Ok(()) => {
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::DeviceRevoke,
                Some(&auth.user_id),
                Some("device"),
                Some(&session_token_preview(&token)),
                "success",
                None,
            );
            StatusCode::NO_CONTENT.into_response()
        }
        Err(error) => {
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::DeviceRevoke,
                Some(&auth.user_id),
                Some("device"),
                Some(&session_token_preview(&token)),
                "failure",
                Some(&error.to_string()),
            );
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

/// `DELETE /api/v1/admin/users/{id}/devices`
pub async fn admin_revoke_user_devices(
    State(state): State<HttpState>,
    auth: AuthContext,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let Some(devices) = state.auth.devices() else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let user_id = match UserId::new(id) {
        Ok(value) => value,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "invalid_user_id",
                        message: error.to_string(),
                    },
                }),
            )
                .into_response();
        }
    };
    match devices.revoke_all_for_user(&user_id) {
        Ok(removed) => {
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::DeviceRevoke,
                Some(&auth.user_id),
                Some("user_devices"),
                Some(user_id.as_str()),
                "success",
                Some(&format!("revoked {removed} devices")),
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "schema_version": 1,
                    "removed": removed,
                })),
            )
                .into_response()
        }
        Err(error) => {
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::DeviceRevoke,
                Some(&auth.user_id),
                Some("user_devices"),
                Some(user_id.as_str()),
                "failure",
                Some(&error.to_string()),
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "revoke_devices_failed",
                        message: error.to_string(),
                    },
                }),
            )
                .into_response()
        }
    }
}
