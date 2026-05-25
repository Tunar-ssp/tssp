//! Chunked, resumable file uploads with session persistence.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path as StdPath, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::upload::HttpUploadRequest;
use crate::HttpState;

/// 256 KB chunks for Orange Pi efficiency.
const CHUNK_SIZE: u64 = 262_144;

/// Session ID identifying an in-progress upload.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UploadSessionId(String);

impl UploadSessionId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    #[allow(dead_code)]
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
        }
    }

    #[allow(dead_code)]
    pub fn mark_chunk_uploaded(&mut self, chunk_index: usize) {
        if chunk_index < self.uploaded_chunks.len() {
            self.uploaded_chunks[chunk_index] = true;
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

    let session_id = UploadSessionId::new(format!("ses_{}", uuid::Uuid::new_v4()));
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
    let session_id = UploadSessionId::new(session_id_str);

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
    if chunk_index as usize >= session.uploaded_chunks.len() {
        return error_response(
            StatusCode::BAD_REQUEST,
            "invalid_chunk",
            "chunk index out of range",
        );
    }

    let chunk_dir = chunk_directory(&state.upload_temp_dir, &session_id);
    let chunk_path = chunk_file_path(&chunk_dir, chunk_index);

    if let Err(e) = std::fs::create_dir_all(&chunk_dir) {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            &format!("failed to create chunk directory: {e}"),
        );
    }

    if let Err(e) = std::fs::write(&chunk_path, &body) {
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
    let session_id = UploadSessionId::new(session_id_str);

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
    let assembly_path = state.upload_temp_dir.join(format!("{}.assembly", session_id.0));

    if let Err(e) = assemble_chunks(&chunk_dir, &assembly_path, &session) {
        let _ = std::fs::remove_dir_all(&chunk_dir);
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "assembly_error",
            &format!("failed to assemble chunks: {e}"),
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
        source: Box::new(match std::fs::File::open(&assembly_path) {
            Ok(f) => f,
            Err(e) => {
                let _ = std::fs::remove_dir_all(&chunk_dir);
                let _ = std::fs::remove_file(&assembly_path);
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "file_error",
                    &format!("failed to open assembled file: {e}"),
                );
            }
        }),
        staged_path: None,
        content_hash: None,
        size: None,
    };

    let _mutation_guard = state.storage_mutation_lock.lock().await;
    let upload_provider = state.upload_provider.clone();
    let result = tokio::task::spawn_blocking(move || upload_provider.upload(upload_request))
        .await
        .map_err(|e| format!("spawn error: {e}"))
        .and_then(|r| r.map_err(|e| format!("upload error: {:?}", e)));

    let _ = std::fs::remove_dir_all(&chunk_dir);
    let _ = std::fs::remove_file(&assembly_path);
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
    let session_id = UploadSessionId::new(session_id_str);

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

fn assemble_chunks(
    chunk_dir: &StdPath,
    output_path: &StdPath,
    session: &UploadSession,
) -> Result<(), String> {
    let mut output = std::fs::File::create(output_path)
        .map_err(|e| format!("failed to create output file: {e}"))?;

    for chunk_index in 0..session.uploaded_chunks.len() {
        let chunk_path = chunk_file_path(chunk_dir, chunk_index as u32);
        let chunk_data =
            std::fs::read(&chunk_path).map_err(|e| format!("failed to read chunk {}: {e}", chunk_index))?;
        output
            .write_all(&chunk_data)
            .map_err(|e| format!("failed to write chunk {}: {e}", chunk_index))?;
    }

    output
        .sync_all()
        .map_err(|e| format!("failed to sync output file: {e}"))?;
    Ok(())
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
