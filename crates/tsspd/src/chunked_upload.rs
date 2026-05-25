//! Chunked, resumable file uploads with session persistence.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    /// Returns an error if the ID doesn't match the expected format (`ses_UUID`) or contains
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
                        return Err(format!("invalid character '{ch}' in session ID"));
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
        let num_chunks = total_size.div_ceil(CHUNK_SIZE);
        Self {
            id,
            filename,
            total_size,
            chunk_size: CHUNK_SIZE,
            uploaded_chunks: vec![false; usize::try_from(num_chunks).unwrap_or(0)],
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
        ((u32::try_from(uploaded).unwrap_or(0) * 100)
            / u32::try_from(self.uploaded_chunks.len()).unwrap_or(0))
        .min(100)
    }
}

/// Session manager for chunked uploads (in-memory only).
///
/// WARNING: Sessions are stored in-memory and are lost on daemon restart.
/// This means clients CANNOT resume uploads after the server restarts.
/// Resumability is only guaranteed within a single daemon lifetime.
/// Consider implementing database persistence for true resumability.
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

/// Response for session state query.
#[derive(Debug, Serialize)]
pub struct SessionStateResponse {
    pub session_id: String,
    pub filename: String,
    pub total_size: u64,
    pub uploaded_chunks: u64,
    pub total_chunks: u64,
    pub progress_percent: u32,
}

/// Query the state of an upload session.
/// Returns 404 if the session doesn't exist (e.g., daemon restarted or session expired).
pub async fn get_upload_session(
    State(state): State<HttpState>,
    auth: crate::auth::OptionalAuthContext,
    Path(session_id_str): Path<String>,
) -> Response {
    let Ok(session_id) = UploadSessionId::new(session_id_str) else {
        return error_response(
            StatusCode::BAD_REQUEST,
            "invalid_session_id",
            "session ID format is invalid",
        );
    };

    let Some(session) = state.upload_session_manager.get_session(&session_id).await else {
        return error_response(
            StatusCode::NOT_FOUND,
            "session_not_found",
            "upload session not found or expired",
        );
    };

    // Verify ownership
    if let Some(session_owner) = &session.owner_id {
        if let Some(auth_ctx) = &auth.0 {
            if auth_ctx.user_id.as_str() != *session_owner && !auth_ctx.is_admin() {
                return error_response(
                    StatusCode::FORBIDDEN,
                    "forbidden",
                    "you do not have permission to query this session",
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

    let uploaded_count = session
        .uploaded_chunks
        .iter()
        .filter(|&&uploaded| uploaded)
        .count();

    (
        StatusCode::OK,
        Json(SessionStateResponse {
            session_id: session.id.0.clone(),
            filename: session.filename.clone(),
            total_size: session.total_size,
            uploaded_chunks: uploaded_count as u64,
            total_chunks: session.uploaded_chunks.len() as u64,
            progress_percent: session.progress_percent(),
        }),
    )
        .into_response()
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

    let total_chunks = req.total_size.div_ceil(CHUNK_SIZE);

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
#[allow(clippy::too_many_lines)]
pub async fn upload_chunk(
    State(state): State<HttpState>,
    auth: crate::auth::OptionalAuthContext,
    Path((session_id_str, chunk_index)): Path<(String, u32)>,
    body: axum::body::Bytes,
) -> Response {
    let Ok(session_id) = UploadSessionId::new(session_id_str) else {
        return error_response(
            StatusCode::BAD_REQUEST,
            "invalid_session_id",
            "session ID format is invalid",
        );
    };

    let Some(session) = state.upload_session_manager.get_session(&session_id).await else {
        return error_response(
            StatusCode::NOT_FOUND,
            "session_not_found",
            "upload session not found",
        );
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

    let Some(updated_session) = state.upload_session_manager.get_session(&session_id).await else {
        return error_response(
            StatusCode::NOT_FOUND,
            "session_not_found",
            "upload session not found",
        );
    };

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

/// Assembles chunks into a single temp file with hash computation.
/// Returns (`temp_file_path`, `content_hash`, `file_size`) to enable `put_staged()` single-write.
fn assemble_chunks_to_temp(
    chunk_dir: &StdPath,
    total_chunks: usize,
    upload_temp_dir: &StdPath,
) -> Result<(PathBuf, tssp_domain::ContentHash, tssp_domain::FileSize), String> {
    use std::fs::File;
    use std::io::{Read, Write};

    let mut temp_file = tempfile::NamedTempFile::new_in(upload_temp_dir)
        .map_err(|e| format!("could not create assembly temp file: {e}"))?;

    let mut hasher = blake3::Hasher::new();
    let mut total_size: u64 = 0;
    let mut buffer = vec![0_u8; 64 * 1024];

    for chunk_idx in 0..total_chunks {
        let chunk_path = chunk_file_path(chunk_dir, u32::try_from(chunk_idx).unwrap_or(0));
        let mut chunk_file = File::open(&chunk_path)
            .map_err(|e| format!("could not read chunk {chunk_idx}: {e}"))?;

        loop {
            let bytes_read = chunk_file
                .read(&mut buffer)
                .map_err(|e| format!("failed to read chunk data: {e}"))?;
            if bytes_read == 0 {
                break;
            }

            let chunk = &buffer[..bytes_read];
            hasher.update(chunk);
            temp_file
                .write_all(chunk)
                .map_err(|e| format!("failed to write to assembly temp: {e}"))?;
            total_size = total_size
                .checked_add(bytes_read as u64)
                .ok_or_else(|| "assembly file size overflow".to_string())?;
        }
    }

    temp_file
        .flush()
        .map_err(|e| format!("failed to flush assembly temp: {e}"))?;
    temp_file
        .as_file()
        .sync_all()
        .map_err(|e| format!("failed to sync assembly temp: {e}"))?;

    let assembled_path = temp_file
        .into_temp_path()
        .keep()
        .map_err(|e| format!("failed to persist assembly temp: {e}"))?
        .clone();
    let hash_hex = hasher.finalize().to_hex();
    let content_hash = tssp_domain::ContentHash::new(hash_hex)
        .map_err(|e| format!("invalid content hash: {e}"))?;
    let file_size = tssp_domain::FileSize::new(total_size);

    Ok((assembled_path, content_hash, file_size))
}

/// Complete an upload by assembling chunks and creating the file.
#[allow(clippy::too_many_lines)]
pub async fn complete_upload(
    State(state): State<HttpState>,
    auth: crate::auth::OptionalAuthContext,
    Path(session_id_str): Path<String>,
) -> Response {
    let Ok(session_id) = UploadSessionId::new(session_id_str) else {
        return error_response(
            StatusCode::BAD_REQUEST,
            "invalid_session_id",
            "session ID format is invalid",
        );
    };

    let Some(session) = state.upload_session_manager.get_session(&session_id).await else {
        return error_response(
            StatusCode::NOT_FOUND,
            "session_not_found",
            "upload session not found",
        );
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
            &format!("{missing} chunks still pending"),
        );
    }

    let chunk_dir = chunk_directory(&state.upload_temp_dir, &session_id);
    let total_chunks = session.uploaded_chunks.len();
    let upload_temp_dir = state.upload_temp_dir.clone();

    // Assemble chunks into a single temp file with hash computation to avoid double-write
    // Wrap in spawn_blocking to avoid blocking the async handler thread
    let chunk_dir_for_assembly = chunk_dir.clone();
    let (assembled_path, content_hash, file_size) = match tokio::task::spawn_blocking(move || {
        assemble_chunks_to_temp(&chunk_dir_for_assembly, total_chunks, &upload_temp_dir)
    })
    .await
    {
        Ok(Ok(result)) => result,
        Ok(Err(e)) => {
            let _ = std::fs::remove_dir_all(&chunk_dir);
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "assembly_error",
                &format!("failed to assemble chunks: {e}"),
            );
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&chunk_dir);
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "spawn_error",
                &format!("failed to spawn assembly task: {e}"),
            );
        }
    };

    // Verify assembled file size matches expected total size
    if file_size.bytes() != session.total_size {
        let _ = std::fs::remove_file(&assembled_path);
        let _ = std::fs::remove_dir_all(&chunk_dir);
        return error_response(
            StatusCode::BAD_REQUEST,
            "corrupt_upload",
            &format!(
                "assembled file size mismatch: expected {} bytes, got {} bytes",
                session.total_size,
                file_size.bytes()
            ),
        );
    }

    let owner_id = auth.0.as_ref().map(|ctx| ctx.user_id.clone());
    let upload_request = HttpUploadRequest {
        filename: session.filename.clone(),
        mime_type: session.mime_type.clone(),
        tags: session.tags.clone(),
        pinned: false,
        folder_path: session.folder_path.clone(),
        owner_id,
        source: Box::new(std::io::empty()),
        staged_path: Some(assembled_path.clone()),
        content_hash: Some(content_hash),
        size: Some(file_size),
    };

    let upload_provider = state.upload_provider.clone();
    let chunk_dir_cleanup = chunk_dir.clone();
    let assembled_path_cleanup = assembled_path.clone();

    let result = tokio::task::spawn_blocking(move || upload_provider.upload(upload_request))
        .await
        .map_err(|e| format!("spawn error: {e}"))
        .and_then(|r| r.map_err(|e| format!("upload error: {e:?}")));

    // Cleanup assembled file if upload failed (in spawn_blocking)
    if result.is_err() {
        let _ = tokio::task::spawn_blocking({
            let path = assembled_path_cleanup.clone();
            move || std::fs::remove_file(path)
        })
        .await;
    }

    // Cleanup chunk directory after upload attempt (success or failure)
    // Do this in spawn_blocking to avoid blocking handler
    tokio::spawn({
        let dir = chunk_dir_cleanup.clone();
        async move {
            let _ = tokio::task::spawn_blocking(move || std::fs::remove_dir_all(dir)).await;
        }
    });

    state
        .upload_session_manager
        .delete_session(&session_id)
        .await;

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
    let Ok(session_id) = UploadSessionId::new(session_id_str) else {
        return error_response(
            StatusCode::BAD_REQUEST,
            "invalid_session_id",
            "session ID format is invalid",
        );
    };

    let Some(session) = state.upload_session_manager.get_session(&session_id).await else {
        return error_response(
            StatusCode::NOT_FOUND,
            "session_not_found",
            "upload session not found",
        );
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
    let chunk_dir_cleanup = chunk_dir.clone();

    // Use tokio::spawn to do cleanup without blocking the handler
    tokio::spawn(async move {
        let _ = tokio::fs::remove_dir_all(&chunk_dir_cleanup).await;
    });

    state
        .upload_session_manager
        .delete_session(&session_id)
        .await;

    StatusCode::NO_CONTENT.into_response()
}

fn chunk_directory(temp_dir: &StdPath, session_id: &UploadSessionId) -> PathBuf {
    temp_dir.join(format!(".{}", session_id.0))
}

fn chunk_file_path(chunk_dir: &StdPath, chunk_index: u32) -> PathBuf {
    chunk_dir.join(format!("chunk_{chunk_index}.part"))
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
