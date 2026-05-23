//! Request gate for remote clients when authentication is enabled.

use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::net::SocketAddr;

use tssp_adapter_system::SystemClock;
use tssp_ports::Clock;

use crate::auth::handlers::{extract_bearer, extract_cookie_token};
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
    if path.starts_with("/s/") || path.starts_with("/u/") {
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
    )
}

/// Enforces dual-mode authentication on API routes.
pub async fn auth_middleware(
    State(state): State<HttpState>,
    request: Request<Body>,
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
        .map(SocketAddr::ip)
        .unwrap_or_else(|| std::net::IpAddr::from([127, 0, 0, 1]));

    let forwarded = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok());
    let client = state.auth.resolve_client(peer, forwarded);

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
        return next.run(request).await;
    }

    let headers = request.headers();
    let token = extract_bearer(headers).or_else(|| extract_cookie_token(headers));
    let authorized = match token.as_deref() {
        Some(token) => state
            .auth
            .validate_token(token, now_seconds())
            .unwrap_or(false),
        None => false,
    };

    if authorized {
        return next.run(request).await;
    }

    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "unauthorized",
                message: "authentication required".to_owned(),
            },
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::is_public_path;
    use axum::http::Method;

    #[test]
    fn public_paths_skip_middleware() {
        assert!(is_public_path("/healthz", &Method::GET));
        assert!(is_public_path("/assets/app.js", &Method::GET));
        assert!(is_public_path("/api/v1/auth/login", &Method::POST));
        assert!(!is_public_path("/api/v1/files", &Method::GET));
    }
}
