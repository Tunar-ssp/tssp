//! Admin session and trusted device handlers.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use tssp_domain::UserId;

use crate::auth::AuthContext;
use crate::{ErrorBody, ErrorResponse, HttpState};

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
    _auth: AuthContext,
    Path(token): Path<String>,
) -> impl IntoResponse {
    let Some(devices) = state.auth.devices() else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    match devices.revoke(&token) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

/// `DELETE /api/v1/admin/users/{id}/devices`
pub async fn admin_revoke_user_devices(
    State(state): State<HttpState>,
    _auth: AuthContext,
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
        Ok(removed) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "schema_version": 1,
                "removed": removed,
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "revoke_devices_failed",
                    message: error.to_string(),
                },
            }),
        )
            .into_response(),
    }
}
