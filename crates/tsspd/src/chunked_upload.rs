//! Chunked, resumable file uploads with session persistence.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::BufReader;
use std::path::{Path as StdPath, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::upload::HttpUploadRequest;
use crate::HttpState;

/// 256 KB chunks for Orange Pi efficiency.
const CHUNK_SIZE: u64 = 262_144;

/// Session ID identifying an in-progress upload.
///
/// Format: "ses_" followed by a UUID v4. Strictly validated to prevent path traversal.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UploadSessionId(String);

impl UploadSessionId {
    /// Creates a new session ID with strict validation.
    ///
    /// # Errors
    ///
    /// Returns an error if the ID doesn't match the expected format (ses_UUID) or contains
    /// path traversal characters (/, \, ..).
    pub fn new(id: String) -> Result<Self, String> {
        // Strict validation: must be "ses_" followed by UUID characters only
        // UUID v4 format: 8-4-4-4-12 hex digits with hyphens
        if !id.starts_with("ses_") {
            return Err("session ID must start with 'ses_'".to_string());
        }

        let uuid_part = &id[4..];

        // Check length: UUID is 36 chars (8-4-4-4-12 + 4 hyphens)
        if uuid_part.len() != 36 {
            return Err("session ID must be 40 characters (ses_ + UUID)".to_string());
        }

        // Strict character validation: only hex digits and hyphens allowed
        for (i, ch) in uuid_part.chars().enumerate() {
            match i {
                8 | 13 | 18 | 23 => {
                    if ch != '-' {
                        return Err(format!("invalid UUID format at position {}", i + 4));
                    }
                }
                _ => {
                    if !ch.is_ascii_hexdigit() {
                        return Err(format!("invalid character '{}' in session ID", ch));
                    }
                }
            }
        }

        // Explicitly reject path traversal patterns
        if id.contains('/') || id.contains('\\') || id.contains("..") {
            return Err("session ID contains invalid path characters".to_string());
        }

        Ok(Self(id))
    }

    /// Returns the session ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// In-progress chunked upload session.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UploadSession {
    pub id: UploadSessionId,
    pub filename: String,
    pub total_size: u64,
    pub chunk_size: u64,
    pub uploaded_chunks: Vec<bool>,
    pub folder_path: String,
    pub owner_id: Option<String>,
    pub tags: Vec<String>,
    pub mime_type: Option<String>,
    pub updated_at: std::time::Instant,
}

impl UploadSession {
    #[allow(dead_code)]
    pub fn new(
        id: UploadSessionId,
        filename: String,
        total_size: u64,
        folder_path: String,
        owner_id: Option<String>,
        tags: Vec<String>,
        mime_type: Option<String>,
    ) -> Self {
        let num_chunks = (total_size + CHUNK_SIZE - 1) / CHUNK_SIZE;
        Self {
            id,
            filename,
            total_size,
            chunk_size: CHUNK_SIZE,
            uploaded_chunks: vec![false; num_chunks as usize],
            folder_path,
            owner_id,
            tags,
            mime_type,
            updated_at: std::time::Instant::now(),
        }
    }

    #[allow(dead_code)]
    pub fn mark_chunk_uploaded(&mut self, chunk_index: usize) {
        if chunk_index < self.uploaded_chunks.len() {
            self.uploaded_chunks[chunk_index] = true;
            self.updated_at = std::time::Instant::now();
        }
    }

    #[allow(dead_code)]
    pub fn is_complete(&self) -> bool {
        self.uploaded_chunks.iter().all(|&uploaded| uploaded)
    }

    #[allow(dead_code)]
    pub fn progress_percent(&self) -> u32 {
        if self.uploaded_chunks.is_empty() {
            return 0;
        }
        let uploaded = self.uploaded_chunks.iter().filter(|&&u| u).count();
        ((uploaded as u32 * 100) / self.uploaded_chunks.len() as u32).min(100)
    }
}

/// Session manager for chunked uploads (in-memory for now).
#[derive(Clone)]
pub struct UploadSessionManager {
    sessions: Arc<RwLock<HashMap<String, UploadSession>>>,
}

impl UploadSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_session(&self, session: UploadSession) -> UploadSessionId {
        let id = session.id.clone();
        self.sessions.write().await.insert(id.0.clone(), session);
        id
    }

    pub async fn get_session(&self, id: &UploadSessionId) -> Option<UploadSession> {
        self.sessions.read().await.get(&id.0).cloned()
    }

    pub async fn mark_chunk_uploaded(&self, id: &UploadSessionId, chunk_index: usize) {
        if let Some(session) = self.sessions.write().await.get_mut(&id.0) {
            session.mark_chunk_uploaded(chunk_index);
        }
    }

    pub async fn delete_session(&self, id: &UploadSessionId) {
        self.sessions.write().await.remove(&id.0);
    }

    /// Removes sessions that haven't been updated for a while.
    pub async fn cleanup_expired(&self, max_age: std::time::Duration) -> Vec<UploadSessionId> {
        let mut sessions = self.sessions.write().await;
        let now = std::time::Instant::now();
        let mut expired = Vec::new();

        sessions.retain(|id, session| {
            if now.duration_since(session.updated_at) > max_age {
                expired.push(UploadSessionId(id.clone()));
                false
            } else {
                true
            }
        });

        expired
    }

    /// Cleans up expired sessions and their disk files.
    #[allow(dead_code)]
    pub async fn cleanup_expired_with_disk(
        &self,
        max_age: std::time::Duration,
        temp_dir: &StdPath,
    ) -> usize {
        let expired = self.cleanup_expired(max_age).await;
        for session_id in &expired {
            let chunk_dir = chunk_directory(temp_dir, session_id);
            let _ = tokio::fs::remove_dir_all(&chunk_dir).await;
        }
        expired.len()
    }
}

impl Default for UploadSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Request to start a new chunked upload.
#[derive(Debug, Deserialize)]
pub struct StartUploadRequest {
    pub filename: String,
    pub total_size: u64,
    pub folder_path: Option<String>,
    pub tags: Option<Vec<String>>,
    pub mime_type: Option<String>,
}

/// Response with upload session details.
#[derive(Debug, Serialize)]
pub struct StartUploadResponse {
    pub session_id: String,
    pub chunk_size: u64,
    pub total_chunks: u64,
}

/// Response for chunk upload confirmation.
#[derive(Debug, Serialize)]
pub struct ChunkUploadResponse {
    pub chunk_index: u32,
    pub progress_percent: u32,
    pub total_chunks: u64,
}

/// Response for upload completion.
#[derive(Debug, Serialize)]
pub struct CompleteUploadResponse {
    pub session_id: String,
    pub status: String,
}

/// Start a new chunked upload session.
pub async fn start_upload(
    State(state): State<HttpState>,
    auth: crate::auth::OptionalAuthContext,
    Json(req): Json<StartUploadRequest>,
) -> Response {
    if req.total_size == 0 {
        return error_response(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "file size must be greater than 0",
        );
    }

    let session_id = match UploadSessionId::new(format!("ses_{}", uuid::Uuid::new_v4())) {
        Ok(id) => id,
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "session_error",
                &format!("could not generate session ID: {e}"),
            )
        }
    };
    let session = UploadSession::new(
        session_id.clone(),
        req.filename,
        req.total_size,
        req.folder_path.unwrap_or_default(),
        auth.0.as_ref().map(|ctx| ctx.user_id.as_str().to_string()),
        req.tags.unwrap_or_default(),
        req.mime_type,
    );

    let total_chunks = (req.total_size + CHUNK_SIZE - 1) / CHUNK_SIZE;

    let _id = state.upload_session_manager.create_session(session).await;

    (
        StatusCode::OK,
        Json(StartUploadResponse {
            session_id: session_id.0,
            chunk_size: CHUNK_SIZE,
            total_chunks,
        }),
    )
        .into_response()
}

/// Upload a single chunk within a session.
pub async fn upload_chunk(
    State(state): State<HttpState>,
    auth: crate::auth::OptionalAuthContext,
    Path((session_id_str, chunk_index)): Path<(String, u32)>,
    body: axum::body::Bytes,
) -> Response {
    let session_id = match UploadSessionId::new(session_id_str) {
        Ok(id) => id,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_session_id",
                "session ID format is invalid",
            )
        }
    };

    let session = match state.upload_session_manager.get_session(&session_id).await {
        Some(s) => s,
        None => {
            return error_response(
                StatusCode::NOT_FOUND,
                "session_not_found",
                "upload session not found",
            )
        }
    };

    // Verify ownership
    if let Some(session_owner) = &session.owner_id {
        if let Some(auth_ctx) = &auth.0 {
            if auth_ctx.user_id.as_str() != *session_owner && !auth_ctx.is_admin() {
                return error_response(
                    StatusCode::FORBIDDEN,
                    "forbidden",
                    "you do not have permission to upload to this session",
                );
            }
        } else {
            return error_response(
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "authentication required",
            );
        }
    }

    // Validate chunk index
    let chunk_index_usize = chunk_index as usize;
    if chunk_index_usize >= session.uploaded_chunks.len() {
        return error_response(
            StatusCode::BAD_REQUEST,
            "invalid_chunk",
            "chunk index out of range",
        );
    }

    // Validate chunk size (allow up to CHUNK_SIZE, last chunk may be smaller)
    let is_last_chunk = chunk_index_usize == session.uploaded_chunks.len() - 1;
    let expected_max_size = if is_last_chunk {
        let remaining = session.total_size % CHUNK_SIZE;
        if remaining > 0 {
            remaining
        } else {
            CHUNK_SIZE
        }
    } else {
        CHUNK_SIZE
    };

    if body.len() as u64 > expected_max_size {
        return error_response(
            StatusCode::BAD_REQUEST,
            "invalid_chunk_size",
            &format!(
                "chunk size {} exceeds maximum {} for this chunk",
                body.len(),
                expected_max_size
            ),
        );
    }

    let chunk_dir = chunk_directory(&state.upload_temp_dir, &session_id);
    let chunk_path = chunk_file_path(&chunk_dir, chunk_index);

    if let Err(e) = tokio::fs::create_dir_all(&chunk_dir).await {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            &format!("failed to create chunk directory: {e}"),
        );
    }

    if let Err(e) = tokio::fs::write(&chunk_path, &body).await {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            &format!("failed to write chunk: {e}"),
        );
    }

    state
        .upload_session_manager
        .mark_chunk_uploaded(&session_id, chunk_index as usize)
        .await;

    let updated_session = state
        .upload_session_manager
        .get_session(&session_id)
        .await
        .unwrap();

    (
        StatusCode::OK,
        Json(ChunkUploadResponse {
            chunk_index,
            progress_percent: updated_session.progress_percent(),
            total_chunks: updated_session.uploaded_chunks.len() as u64,
        }),
    )
        .into_response()
}

/// Complete an upload by assembling chunks and creating the file.
pub async fn complete_upload(
    State(state): State<HttpState>,
    auth: crate::auth::OptionalAuthContext,
    Path(session_id_str): Path<String>,
) -> Response {
    let session_id = match UploadSessionId::new(session_id_str) {
        Ok(id) => id,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_session_id",
                "session ID format is invalid",
            )
        }
    };

    let session = match state.upload_session_manager.get_session(&session_id).await {
        Some(s) => s,
        None => {
            return error_response(
                StatusCode::NOT_FOUND,
                "session_not_found",
                "upload session not found",
            )
        }
    };

    // Verify ownership
    if let Some(session_owner) = &session.owner_id {
        if let Some(auth_ctx) = &auth.0 {
            if auth_ctx.user_id.as_str() != *session_owner && !auth_ctx.is_admin() {
                return error_response(
                    StatusCode::FORBIDDEN,
                    "forbidden",
                    "you do not have permission to complete this upload",
                );
            }
        } else {
            return error_response(
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "authentication required",
            );
        }
    }

    // Verify all chunks are uploaded
    if !session.is_complete() {
        let missing = session
            .uploaded_chunks
            .iter()
            .enumerate()
            .filter(|(_, &uploaded)| !uploaded)
            .count();
        return error_response(
            StatusCode::BAD_REQUEST,
            "incomplete_upload",
            &format!("{} chunks still pending", missing),
        );
    }

    let chunk_dir = chunk_directory(&state.upload_temp_dir, &session_id);
    let total_chunks = session.uploaded_chunks.len();

    // Create ChunkReader that streams chunks directly without intermediate file
    let chunk_reader = match ChunkReader::new(chunk_dir.clone(), total_chunks) {
        Ok(reader) => reader,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&chunk_dir);
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "reader_error",
                &format!("failed to create chunk reader: {e}"),
            );
        }
    };

    let owner_id = auth.0.as_ref().map(|ctx| ctx.user_id.clone());
    let upload_request = HttpUploadRequest {
        filename: session.filename.clone(),
        mime_type: session.mime_type.clone(),
        tags: session.tags.clone(),
        pinned: false,
        folder_path: session.folder_path.clone(),
        owner_id,
        source: Box::new(chunk_reader),
        staged_path: None,
        content_hash: None,
        size: None,
    };

    let _mutation_guard = state.storage_mutation_lock.lock().await;
    let upload_provider = state.upload_provider.clone();
    let chunk_dir_cleanup = chunk_dir.clone();
    let result = tokio::task::spawn_blocking(move || upload_provider.upload(upload_request))
        .await
        .map_err(|e| format!("spawn error: {e}"))
        .and_then(|r| r.map_err(|e| format!("upload error: {:?}", e)));

    // Cleanup chunk directory after upload attempt (success or failure)
    let _ = std::fs::remove_dir_all(&chunk_dir_cleanup);
    state.upload_session_manager.delete_session(&session_id).await;

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(CompleteUploadResponse {
                session_id: session_id.0,
                status: "completed".to_string(),
            }),
        )
            .into_response(),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "upload_error",
            &format!("failed to upload assembled file: {e}"),
        ),
    }
}

/// Cancel an upload and cleanup resources.
pub async fn cancel_upload(
    State(state): State<HttpState>,
    auth: crate::auth::OptionalAuthContext,
    Path(session_id_str): Path<String>,
) -> Response {
    let session_id = match UploadSessionId::new(session_id_str) {
        Ok(id) => id,
        Err(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_session_id",
                "session ID format is invalid",
            )
        }
    };

    let session = match state.upload_session_manager.get_session(&session_id).await {
        Some(s) => s,
        None => {
            return error_response(
                StatusCode::NOT_FOUND,
                "session_not_found",
                "upload session not found",
            )
        }
    };

    // Verify ownership
    if let Some(session_owner) = &session.owner_id {
        if let Some(auth_ctx) = &auth.0 {
            if auth_ctx.user_id.as_str() != *session_owner && !auth_ctx.is_admin() {
                return error_response(
                    StatusCode::FORBIDDEN,
                    "forbidden",
                    "you do not have permission to cancel this upload",
                );
            }
        } else {
            return error_response(
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "authentication required",
            );
        }
    }

    let chunk_dir = chunk_directory(&state.upload_temp_dir, &session_id);
    let _ = std::fs::remove_dir_all(&chunk_dir);
    state.upload_session_manager.delete_session(&session_id).await;

    StatusCode::NO_CONTENT.into_response()
}

fn chunk_directory(temp_dir: &StdPath, session_id: &UploadSessionId) -> PathBuf {
    temp_dir.join(format!(".{}", session_id.0))
}

fn chunk_file_path(chunk_dir: &StdPath, chunk_index: u32) -> PathBuf {
    chunk_dir.join(format!("chunk_{}.part", chunk_index))
}

/// Streams chunks directly without writing an intermediate .assembly file.
/// This eliminates write amplification on Orange Pi's SD card.
struct ChunkReader {
    chunk_dir: PathBuf,
    current_chunk_index: usize,
    current_file: Option<BufReader<std::fs::File>>,
    total_chunks: usize,
    exhausted: bool,
}

impl ChunkReader {
    fn new(chunk_dir: PathBuf, total_chunks: usize) -> Result<Self, String> {
        Ok(Self {
            chunk_dir,
            current_chunk_index: 0,
            current_file: None,
            total_chunks,
            exhausted: false,
        })
    }

    fn open_next_chunk(&mut self) -> Result<(), String> {
        if self.current_chunk_index >= self.total_chunks {
            self.exhausted = true;
            return Ok(());
        }

        let chunk_path = chunk_file_path(&self.chunk_dir, self.current_chunk_index as u32);
        let file = std::fs::File::open(&chunk_path)
            .map_err(|e| format!("failed to read chunk {}: {e}", self.current_chunk_index))?;
        self.current_file = Some(BufReader::new(file));
        self.current_chunk_index += 1;
        Ok(())
    }
}

impl std::io::Read for ChunkReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            // If we have an open file, try to read from it
            if let Some(ref mut file) = self.current_file {
                match file.read(buf) {
                    Ok(0) => {
                        // End of current chunk, move to next
                        self.current_file = None;
                        if self.current_chunk_index >= self.total_chunks {
                            self.exhausted = true;
                            return Ok(0);
                        }
                        // Open next chunk and loop to read from it
                        self.open_next_chunk()
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                    }
                    other => return other,
                }
            } else {
                // No open file, try to open the next chunk
                if self.exhausted || self.current_chunk_index >= self.total_chunks {
                    return Ok(0);
                }
                self.open_next_chunk()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            }
        }
    }
}

fn error_response(status: StatusCode, code: &str, message: &str) -> Response {
    (
        status,
        Json(serde_json::json!({
            "error": {
                "code": code,
                "message": message
            }
        })),
    )
        .into_response()
}
