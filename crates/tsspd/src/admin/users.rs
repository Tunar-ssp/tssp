//! Admin user management handlers.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use tssp_domain::{UserId, UserName, UserRole};

use crate::auth::AuthContext;
use crate::{ErrorBody, ErrorResponse, HttpState};

#[derive(Debug, Deserialize)]
pub struct CreateUserBody {
    pub name: String,
    pub code: String,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResetCodeBody {
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct SetRoleBody {
    pub role: String,
}

#[derive(Debug, Serialize)]
struct UserListResponse {
    schema_version: u8,
    users: Vec<UserJson>,
}

#[derive(Debug, Serialize)]
struct UserJson {
    id: String,
    name: String,
    role: String,
    created_at: i64,
    disabled: bool,
}

fn map_user(user: &crate::auth::UserRecord) -> UserJson {
    UserJson {
        id: user.id.as_str().to_owned(),
        name: user.name.as_str().to_owned(),
        role: user.role.as_str().to_owned(),
        created_at: user.created_at,
        disabled: user.disabled_at.is_some(),
    }
}

fn admin_error(message: String) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "admin_user_error",
                message,
            },
        }),
    )
}

/// `GET /api/v1/admin/users`
pub async fn admin_list_users(
    State(state): State<HttpState>,
    _auth: AuthContext,
) -> impl IntoResponse {
    let Some(users) = state.auth.users() else {
        return admin_error("user store unavailable".to_owned()).into_response();
    };
    match users.list_users() {
        Ok(list) => (
            StatusCode::OK,
            Json(UserListResponse {
                schema_version: 1,
                users: list.iter().map(map_user).collect(),
            }),
        )
            .into_response(),
        Err(error) => admin_error(error.to_string()).into_response(),
    }
}

/// `POST /api/v1/admin/users`
pub async fn admin_create_user(
    State(state): State<HttpState>,
    _auth: AuthContext,
    Json(body): Json<CreateUserBody>,
) -> impl IntoResponse {
    let Some(users) = state.auth.users() else {
        return admin_error("user store unavailable".to_owned()).into_response();
    };
    let name = match UserName::new(body.name) {
        Ok(value) => value,
        Err(error) => return admin_error(error.to_string()).into_response(),
    };
    let role = match body.role.as_deref().unwrap_or("user") {
        "admin" => UserRole::Admin,
        "user" => UserRole::User,
        _ => return admin_error("role must be admin or user".to_owned()).into_response(),
    };
    let id = match UserId::new(format!("user-{}", uuid_simple())) {
        Ok(value) => value,
        Err(error) => return admin_error(error.to_string()).into_response(),
    };
    let now = {
        use tssp_ports::Clock;
        tssp_adapter_system::SystemClock.now().seconds()
    };
    match users.create_user(&id, &name, role, &body.code, now) {
        Ok(user) => (StatusCode::CREATED, Json(map_user(&user))).into_response(),
        Err(error) => admin_error(error.to_string()).into_response(),
    }
}

/// `DELETE /api/v1/admin/users/{id}`
pub async fn admin_delete_user(
    State(state): State<HttpState>,
    _auth: AuthContext,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let Some(users) = state.auth.users() else {
        return admin_error("user store unavailable".to_owned()).into_response();
    };
    let user_id = match UserId::new(id) {
        Ok(value) => value,
        Err(error) => return admin_error(error.to_string()).into_response(),
    };
    match users.delete_user(&user_id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => admin_error(error.to_string()).into_response(),
    }
}

/// `POST /api/v1/admin/users/{id}/reset-code`
pub async fn admin_reset_code(
    State(state): State<HttpState>,
    _auth: AuthContext,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(body): Json<ResetCodeBody>,
) -> impl IntoResponse {
    let Some(users) = state.auth.users() else {
        return admin_error("user store unavailable".to_owned()).into_response();
    };
    let user_id = match UserId::new(id) {
        Ok(value) => value,
        Err(error) => return admin_error(error.to_string()).into_response(),
    };
    match users.reset_code(&user_id, &body.code) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => admin_error(error.to_string()).into_response(),
    }
}

/// `PUT /api/v1/admin/users/{id}/role`
pub async fn admin_set_role(
    State(state): State<HttpState>,
    _auth: AuthContext,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(body): Json<SetRoleBody>,
) -> impl IntoResponse {
    let Some(users) = state.auth.users() else {
        return admin_error("user store unavailable".to_owned()).into_response();
    };
    let user_id = match UserId::new(id) {
        Ok(value) => value,
        Err(error) => return admin_error(error.to_string()).into_response(),
    };
    let role = match body.role.as_str() {
        "admin" => UserRole::Admin,
        "user" => UserRole::User,
        _ => return admin_error("role must be admin or user".to_owned()).into_response(),
    };
    match users.set_role(&user_id, role) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => admin_error(error.to_string()).into_response(),
    }
}

fn uuid_simple() -> String {
    use std::fmt::Write as _;

    let mut bytes = [0_u8; 8];
    let _ = getrandom::getrandom(&mut bytes);
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(value, "{byte:02x}");
    }
    value
}
