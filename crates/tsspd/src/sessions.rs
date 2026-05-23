//! Session creation and retrieval endpoints for share links.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use tssp_domain::SessionToken;

use std::sync::atomic::{AtomicU64, Ordering};
use tssp_app::SessionService;
use tssp_ports::{Clock, SessionRepository};

use crate::{ErrorBody, ErrorResponse, HttpState};

#[allow(dead_code)]
static TOKEN_COUNTER: AtomicU64 = AtomicU64::new(0);

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

    /// Associates a received file with a session and marks it used.
    ///
    /// # Errors
    ///
    /// Returns an error when the update cannot be committed.
    fn complete_receive_session(&self, token: &SessionToken, file_id: &str) -> Result<(), String>;
}

/// Static session provider that returns empty/placeholder responses.
#[derive(Debug)]
pub struct StaticSessionProvider;

impl SessionProvider for StaticSessionProvider {
    fn create_send_session(
        &self,
        _file_id: &str,
        _ttl_seconds: u64,
    ) -> Result<SessionResponse, String> {
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

    fn complete_receive_session(
        &self,
        _token: &SessionToken,
        _file_id: &str,
    ) -> Result<(), String> {
        Err("session provider not initialized".to_string())
    }
}

/// Application-level session provider backed by `SessionService`.
#[allow(dead_code)]
pub struct ApplicationSessionProvider<R: SessionRepository, C: Clock> {
    service: SessionService<R>,
    clock: C,
}

impl<R: SessionRepository, C: Clock> ApplicationSessionProvider<R, C> {
    /// Creates a new application session provider.
    #[allow(dead_code)]
    pub fn new(service: SessionService<R>, clock: C) -> Self {
        Self { service, clock }
    }
}

impl<R: SessionRepository + Send + Sync, C: Clock + Send + Sync> SessionProvider
    for ApplicationSessionProvider<R, C>
{
    fn create_send_session(
        &self,
        file_id: &str,
        ttl_seconds: u64,
    ) -> Result<SessionResponse, String> {
        let now = self.clock.now();
        let token = generate_session_token();

        let session = self
            .service
            .create_send_session(token, file_id, ttl_seconds, now)
            .map_err(|e| format!("failed to create send session: {e}"))?;

        Ok(SessionResponse {
            token: session.token.as_str().to_string(),
            kind: match session.kind {
                tssp_domain::SessionKind::Send => "send".to_string(),
                tssp_domain::SessionKind::Receive => "receive".to_string(),
            },
            created_at: session.created_at.seconds(),
            expires_at: session.expires_at.seconds(),
            source_file: session.source_file.as_ref().map(|f| f.as_str().to_string()),
            received_file: session
                .received_file
                .as_ref()
                .map(|f| f.as_str().to_string()),
            expected_name: session
                .expected_name
                .as_ref()
                .map(|n| n.original().to_string()),
            used_at: None,
            download_url: None,
            upload_url: None,
        })
    }

    fn create_receive_session(&self, ttl_seconds: u64) -> Result<SessionResponse, String> {
        let now = self.clock.now();
        let token = generate_session_token();

        let session = self
            .service
            .create_receive_session(token, ttl_seconds, now)
            .map_err(|e| format!("failed to create receive session: {e}"))?;

        Ok(SessionResponse {
            token: session.token.as_str().to_string(),
            kind: match session.kind {
                tssp_domain::SessionKind::Send => "send".to_string(),
                tssp_domain::SessionKind::Receive => "receive".to_string(),
            },
            created_at: session.created_at.seconds(),
            expires_at: session.expires_at.seconds(),
            source_file: session.source_file.as_ref().map(|f| f.as_str().to_string()),
            received_file: session
                .received_file
                .as_ref()
                .map(|f| f.as_str().to_string()),
            expected_name: session
                .expected_name
                .as_ref()
                .map(|n| n.original().to_string()),
            used_at: None,
            download_url: None,
            upload_url: None,
        })
    }

    fn get_session(&self, token: &SessionToken) -> Result<SessionResponse, String> {
        let session = self
            .service
            .get_session(token)
            .map_err(|e| format!("failed to get session: {e}"))?
            .ok_or_else(|| "session not found".to_string())?;

        Ok(SessionResponse {
            token: session.token.as_str().to_string(),
            kind: match session.kind {
                tssp_domain::SessionKind::Send => "send".to_string(),
                tssp_domain::SessionKind::Receive => "receive".to_string(),
            },
            created_at: session.created_at.seconds(),
            expires_at: session.expires_at.seconds(),
            source_file: session.source_file.as_ref().map(|f| f.as_str().to_string()),
            received_file: session
                .received_file
                .as_ref()
                .map(|f| f.as_str().to_string()),
            expected_name: session
                .expected_name
                .as_ref()
                .map(|n| n.original().to_string()),
            used_at: None,
            download_url: None,
            upload_url: None,
        })
    }

    fn use_session(&self, token: &SessionToken) -> Result<(), String> {
        let now = self.clock.now();
        self.service
            .use_session(token, now)
            .map_err(|e| format!("failed to use session: {e}"))
    }

    fn complete_receive_session(&self, token: &SessionToken, file_id: &str) -> Result<(), String> {
        let now = self.clock.now();
        let fid = tssp_domain::FileId::new(file_id).map_err(|e| format!("invalid file id: {e}"))?;
        self.service
            .complete_receive_session(token, &fid, now)
            .map_err(|e| format!("failed to complete receive session: {e}"))
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
    /// Public download URL for send sessions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    /// Public upload URL for receive sessions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload_url: Option<String>,
}

impl SessionResponse {
    fn with_public_urls(mut self, urls: &crate::PublicUrlBuilder) -> Self {
        match self.kind.as_str() {
            "send" => {
                self.download_url = Some(urls.send_download_url(&self.token));
            }
            "receive" => {
                self.upload_url = Some(urls.receive_upload_url(&self.token));
            }
            _ => {}
        }
        self
    }
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

#[allow(dead_code)]
fn generate_session_token() -> SessionToken {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let counter = TOKEN_COUNTER.fetch_add(1, Ordering::SeqCst);

    let alphabet: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut token = String::with_capacity(22);

    let combined = timestamp.wrapping_mul(1_000_000).wrapping_add(counter);
    let mut value = combined;

    for _ in 0..22 {
        let idx = (value % 64) as usize;
        token.push(alphabet[idx] as char);
        value /= 64;
    }

    #[allow(clippy::expect_used)]
    SessionToken::new(&token).expect("alphabet is base64url-safe and length is 22; always valid")
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
            HttpSessionError::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "session_error", msg)
            }
        };

        let response = ErrorResponse {
            error: ErrorBody { code, message },
        };

        (status, Json(response)).into_response()
    }
}

/// Creates a new send session (POST /api/v1/sessions/send).
pub async fn create_send_session(
    State(state): State<HttpState>,
    Json(payload): Json<CreateSendSessionRequest>,
) -> Result<(StatusCode, Json<SessionResponse>), HttpSessionError> {
    let urls = state.public_urls().clone();
    state
        .session_provider
        .create_send_session(&payload.file_id, payload.ttl_seconds)
        .map(|response| {
            (
                StatusCode::CREATED,
                Json(response.with_public_urls(&urls)),
            )
        })
        .map_err(|_| HttpSessionError::InternalError("Failed to create session".to_string()))
}

/// Creates a new receive session (POST /api/v1/sessions/receive).
pub async fn create_receive_session(
    State(state): State<HttpState>,
    Json(payload): Json<CreateReceiveSessionRequest>,
) -> Result<(StatusCode, Json<SessionResponse>), HttpSessionError> {
    let urls = state.public_urls().clone();
    state
        .session_provider
        .create_receive_session(payload.ttl_seconds)
        .map(|response| {
            (
                StatusCode::CREATED,
                Json(response.with_public_urls(&urls)),
            )
        })
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
        .map(|()| StatusCode::NO_CONTENT)
        .map_err(|_| HttpSessionError::NotFound)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
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
        let req: CreateReceiveSessionRequest =
            serde_json::from_str(json).expect("deserialize failed");
        assert_eq!(req.ttl_seconds, 7200);
    }

    #[test]
    fn create_receive_session_request_uses_default_ttl() {
        let json = "{}";
        let req: CreateReceiveSessionRequest =
            serde_json::from_str(json).expect("deserialize failed");
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
            download_url: None,
            upload_url: None,
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
            download_url: None,
            upload_url: None,
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
