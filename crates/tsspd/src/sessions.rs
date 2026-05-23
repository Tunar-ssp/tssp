//! Session creation and retrieval endpoints for share links.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use tssp_domain::SessionToken;

use crate::{ErrorBody, ErrorResponse, HttpState};

/// Creates and manages transfer sessions for sharing.
pub trait SessionProvider: Send + Sync {
    /// Creates a new send session for sharing a file.
    ///
    /// # Errors
    ///
    /// Returns an error when the session cannot be created.
    fn create_send_session(
        &self,
        file_id: &str,
        ttl_seconds: u64,
    ) -> Result<SessionResponse, String>;

    /// Creates a new receive session for accepting uploads.
    ///
    /// # Errors
    ///
    /// Returns an error when the session cannot be created.
    fn create_receive_session(&self, ttl_seconds: u64) -> Result<SessionResponse, String>;

    /// Retrieves a session by token.
    ///
    /// # Errors
    ///
    /// Returns an error when the session cannot be found.
    fn get_session(&self, token: &SessionToken) -> Result<SessionResponse, String>;

    /// Marks a session as used.
    ///
    /// # Errors
    ///
    /// Returns an error when the update cannot be committed.
    fn use_session(&self, token: &SessionToken) -> Result<(), String>;
}

/// Static session provider that returns empty/placeholder responses.
#[derive(Debug)]
pub struct StaticSessionProvider;

impl SessionProvider for StaticSessionProvider {
    fn create_send_session(&self, _file_id: &str, _ttl_seconds: u64) -> Result<SessionResponse, String> {
        Err("session provider not initialized".to_string())
    }

    fn create_receive_session(&self, _ttl_seconds: u64) -> Result<SessionResponse, String> {
        Err("session provider not initialized".to_string())
    }

    fn get_session(&self, _token: &SessionToken) -> Result<SessionResponse, String> {
        Err("session provider not initialized".to_string())
    }

    fn use_session(&self, _token: &SessionToken) -> Result<(), String> {
        Err("session provider not initialized".to_string())
    }
}

/// HTTP response for session operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    /// Opaque session token for auth and retrieval.
    pub token: String,
    /// Session kind: "send" or "receive".
    pub kind: String,
    /// Unix timestamp when session was created.
    pub created_at: i64,
    /// Unix timestamp when session expires.
    pub expires_at: i64,
    /// When provided for send sessions, the source file ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
    /// File received in this session, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub received_file: Option<String>,
    /// File expected name for receive sessions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_name: Option<String>,
    /// When this session was last used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_at: Option<i64>,
}

/// Request to create a send session.
#[derive(Debug, Deserialize)]
pub struct CreateSendSessionRequest {
    /// File ID to share.
    pub file_id: String,
    /// TTL in seconds (defaults to 86400 = 24 hours).
    #[serde(default = "default_ttl")]
    pub ttl_seconds: u64,
}

/// Request to create a receive session.
#[derive(Debug, Deserialize)]
pub struct CreateReceiveSessionRequest {
    /// TTL in seconds (defaults to 86400 = 24 hours).
    #[serde(default = "default_ttl")]
    pub ttl_seconds: u64,
}

fn default_ttl() -> u64 {
    86_400
}

/// HTTP error response for session operations.
#[derive(Debug)]
pub enum HttpSessionError {
    /// Session not found.
    NotFound,
    /// Invalid session token.
    InvalidToken,
    /// Session has expired.
    #[allow(dead_code)]
    Expired,
    /// Internal server error.
    InternalError(String),
}

impl IntoResponse for HttpSessionError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            HttpSessionError::NotFound => (
                StatusCode::NOT_FOUND,
                "session_not_found",
                "Session not found".to_string(),
            ),
            HttpSessionError::InvalidToken => (
                StatusCode::BAD_REQUEST,
                "invalid_token",
                "Invalid session token".to_string(),
            ),
            HttpSessionError::Expired => (
                StatusCode::GONE,
                "session_expired",
                "Session has expired".to_string(),
            ),
            HttpSessionError::InternalError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "session_error",
                msg,
            ),
        };

        let response = ErrorResponse {
            error: ErrorBody {
                code,
                message,
            },
        };

        (status, Json(response)).into_response()
    }
}

/// Creates a new send session (POST /api/v1/sessions/send).
pub async fn create_send_session(
    State(state): State<HttpState>,
    Json(payload): Json<CreateSendSessionRequest>,
) -> Result<(StatusCode, Json<SessionResponse>), HttpSessionError> {
    state
        .session_provider
        .create_send_session(&payload.file_id, payload.ttl_seconds)
        .map(|response| (StatusCode::CREATED, Json(response)))
        .map_err(|_| HttpSessionError::InternalError("Failed to create session".to_string()))
}

/// Creates a new receive session (POST /api/v1/sessions/receive).
pub async fn create_receive_session(
    State(state): State<HttpState>,
    Json(payload): Json<CreateReceiveSessionRequest>,
) -> Result<(StatusCode, Json<SessionResponse>), HttpSessionError> {
    state
        .session_provider
        .create_receive_session(payload.ttl_seconds)
        .map(|response| (StatusCode::CREATED, Json(response)))
        .map_err(|_| HttpSessionError::InternalError("Failed to create session".to_string()))
}

/// Retrieves a session by token (GET /api/v1/sessions/{token}).
pub async fn get_session(
    State(state): State<HttpState>,
    Path(token_str): Path<String>,
) -> Result<Json<SessionResponse>, HttpSessionError> {
    let token = SessionToken::new(&token_str).map_err(|_| HttpSessionError::InvalidToken)?;

    state
        .session_provider
        .get_session(&token)
        .map(Json)
        .map_err(|_| HttpSessionError::NotFound)
}

/// Marks a session as used (POST /api/v1/sessions/{token}/use).
pub async fn use_session_endpoint(
    State(state): State<HttpState>,
    Path(token_str): Path<String>,
) -> Result<StatusCode, HttpSessionError> {
    let token = SessionToken::new(&token_str).map_err(|_| HttpSessionError::InvalidToken)?;

    state
        .session_provider
        .use_session(&token)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|_| HttpSessionError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ttl_is_24_hours() {
        assert_eq!(default_ttl(), 86_400);
    }

    #[test]
    fn create_send_session_request_deserializes() {
        let json = r#"{"file_id": "test-file-123", "ttl_seconds": 3600}"#;
        let req: CreateSendSessionRequest = serde_json::from_str(json).expect("deserialize failed");
        assert_eq!(req.file_id, "test-file-123");
        assert_eq!(req.ttl_seconds, 3600);
    }

    #[test]
    fn create_send_session_request_uses_default_ttl() {
        let json = r#"{"file_id": "test-file-456"}"#;
        let req: CreateSendSessionRequest = serde_json::from_str(json).expect("deserialize failed");
        assert_eq!(req.file_id, "test-file-456");
        assert_eq!(req.ttl_seconds, 86_400);
    }

    #[test]
    fn create_receive_session_request_deserializes() {
        let json = r#"{"ttl_seconds": 7200}"#;
        let req: CreateReceiveSessionRequest = serde_json::from_str(json).expect("deserialize failed");
        assert_eq!(req.ttl_seconds, 7200);
    }

    #[test]
    fn create_receive_session_request_uses_default_ttl() {
        let json = r#"{}"#;
        let req: CreateReceiveSessionRequest = serde_json::from_str(json).expect("deserialize failed");
        assert_eq!(req.ttl_seconds, 86_400);
    }

    #[test]
    fn session_response_serializes_with_all_fields() {
        let response = SessionResponse {
            token: "test-token-abc123".to_string(),
            kind: "send".to_string(),
            created_at: 1000,
            expires_at: 2000,
            source_file: Some("file-001".to_string()),
            received_file: None,
            expected_name: None,
            used_at: None,
        };

        let json = serde_json::to_string(&response).expect("serialize failed");
        assert!(json.contains("\"token\":\"test-token-abc123\""));
        assert!(json.contains("\"kind\":\"send\""));
        assert!(json.contains("\"source_file\":\"file-001\""));
        assert!(!json.contains("received_file"));
    }

    #[test]
    fn session_response_skips_none_fields() {
        let response = SessionResponse {
            token: "test-token".to_string(),
            kind: "receive".to_string(),
            created_at: 1000,
            expires_at: 2000,
            source_file: None,
            received_file: None,
            expected_name: None,
            used_at: None,
        };

        let json = serde_json::to_string(&response).expect("serialize failed");
        assert!(!json.contains("source_file"));
        assert!(!json.contains("received_file"));
        assert!(!json.contains("expected_name"));
        assert!(!json.contains("used_at"));
    }

    #[test]
    fn static_session_provider_rejects_operations() {
        let provider = StaticSessionProvider;

        assert!(provider.create_send_session("file-1", 3600).is_err());
        assert!(provider.create_receive_session(3600).is_err());

        let token = SessionToken::new("aaaaaaaaaaaaaaaaaaaaaa").expect("invalid token");
        assert!(provider.get_session(&token).is_err());
        assert!(provider.use_session(&token).is_err());
    }
}
