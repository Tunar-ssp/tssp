//! Request gate: global/local modes, sessions, trusted devices, admin RBAC.

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::net::SocketAddr;

use tssp_adapter_system::SystemClock;
use tssp_ports::Clock;

use super::context::AuthContext;
use super::handlers::{extract_bearer, extract_cookie_token, DEVICE_COOKIE, SESSION_COOKIE};
use super::local_network::is_local_client;
use crate::{ErrorBody, ErrorResponse, HttpState};

fn now_seconds() -> i64 {
    SystemClock.now().seconds()
}

fn is_public_path(path: &str, method: &axum::http::Method) -> bool {
    if path == "/healthz" || path == "/readyz" || path == "/metrics" {
        return true;
    }
    if path.starts_with("/assets/") {
        return true;
    }
    if path.starts_with("/s/") || path.starts_with("/u/") || path.starts_with("/p/") {
        return true;
    }
    if method == axum::http::Method::OPTIONS {
        return true;
    }
    matches!(
        path,
        "/api/v1/auth/login"
            | "/api/v1/auth/token"
            | "/api/v1/auth/required"
            | "/api/v1/auth/me"
            | "/api/v1/public/files"
    ) || path.starts_with("/api/v1/public/files/")
}

fn requires_admin(path: &str, method: &axum::http::Method) -> bool {
    path.starts_with("/api/v1/admin/") && method != axum::http::Method::OPTIONS
}

/// Enforces authentication and role checks on API routes.
pub async fn auth_middleware(
    State(state): State<HttpState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let path = request.uri().path().to_owned();
    let method = request.method().clone();

    if !path.starts_with("/api/") || is_public_path(&path, &method) {
        return next.run(request).await;
    }

    let peer = request
        .extensions()
        .get::<SocketAddr>()
        .map_or_else(|| std::net::IpAddr::from([127, 0, 0, 1]), SocketAddr::ip);

    let headers = request.headers();
    let forwarded = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok());
    let client = state.auth.resolve_client(peer, forwarded);
    let now = now_seconds();

    let required = match state.auth.remote_auth_required(client) {
        Ok(value) => value,
        Err(error) => {
            return unauthorized_message(error.to_string()).into_response();
        }
    };

    let bearer = extract_bearer(headers);
    let session_token = bearer
        .clone()
        .or_else(|| extract_cookie_token(headers, SESSION_COOKIE));
    let device_token = extract_cookie_token(headers, DEVICE_COOKIE);
    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok());
    let client_ip = Some(client.to_string());

    let auth_ctx = if let Some(token) = session_token.as_deref() {
        state
            .auth
            .resolve_token(token, now)
            .ok()
            .flatten()
            .map(|session| AuthContext {
                user_id: session.user_id,
                role: session.role,
                session_token: Some(session.token),
                device_token: session.device_token,
            })
    } else if !required {
        None
    } else if !state.auth.global_auth_required() && is_local_client(client) {
        if let Some(device) = device_token.as_deref() {
            state
                .auth
                .resolve_device(device, now, client_ip.as_deref(), user_agent)
                .ok()
                .flatten()
                .map(|session| AuthContext {
                    user_id: session.user_id,
                    role: session.role,
                    session_token: None,
                    device_token: session.device_token,
                })
        } else {
            None
        }
    } else {
        None
    };

    if required && auth_ctx.is_none() {
        // Legacy password-only tokens without user join
        if let Some(token) = session_token.as_deref() {
            if state
                .auth
                .store()
                .and_then(|store| store.token_valid(token, now).ok())
                .unwrap_or(false)
            {
                return next.run(request).await;
            }
        }
        return unauthorized_message("authentication required".to_owned()).into_response();
    }

    if let Some(ctx) = auth_ctx {
        if requires_admin(&path, &method) && !ctx.is_admin() {
            return (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "forbidden",
                        message: "admin role required".to_owned(),
                    },
                }),
            )
                .into_response();
        }
        request.extensions_mut().insert(ctx);
    }

    next.run(request).await
}

fn unauthorized_message(message: String) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "unauthorized",
                message,
            },
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::is_public_path;
    use axum::http::Method;

    #[test]
    fn public_paths_skip_middleware() {
        assert!(is_public_path("/healthz", &Method::GET));
        assert!(is_public_path("/api/v1/public/files", &Method::GET));
        assert!(is_public_path("/api/v1/auth/login", &Method::POST));
        assert!(!is_public_path("/api/v1/admin/overview", &Method::GET));
    }
}
