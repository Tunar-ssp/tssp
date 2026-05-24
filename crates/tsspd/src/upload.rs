//! Multipart file upload delivery.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use axum::extract::{Multipart, State};
use axum::http::header::LOCATION;
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use tokio::task;
use tssp_app::{UploadError, UploadRequest, UploadService};
use tssp_domain::FileRecord;
use tssp_ports::{BlobStore, Clock, FileRepository, IdGenerator, RepositoryError};

use crate::{ErrorBody, ErrorResponse, HttpState};

/// Handles completed HTTP upload streams through the application layer.
pub trait FileUploadProvider: Send + Sync {
    /// Stores one uploaded file.
    ///
    /// # Errors
    ///
    /// Returns [`HttpUploadError`] when metadata is invalid, storage fails, or
    /// the metadata commit cannot complete.
    fn upload(&self, request: HttpUploadRequest) -> Result<HttpUploadOutcome, HttpUploadError>;
}

#[derive(Debug)]
pub(crate) struct StaticFileUploadProvider;

impl FileUploadProvider for StaticFileUploadProvider {
    fn upload(&self, _request: HttpUploadRequest) -> Result<HttpUploadOutcome, HttpUploadError> {
        Err(HttpUploadError::Unavailable {
            message: "upload service is not configured".to_owned(),
        })
    }
}

/// Request passed from HTTP delivery to the upload provider.
pub struct HttpUploadRequest {
    /// Original filename from multipart metadata.
    pub filename: String,
    /// Optional MIME type from multipart metadata.
    pub mime_type: Option<String>,
    /// Repeated tag fields.
    pub tags: Vec<String>,
    /// Whether the file should be pinned at upload time.
    pub pinned: bool,
    /// Virtual folder path within the bucket.
    pub folder_path: String,
    /// Owning user when authenticated at upload time.
    pub owner_id: Option<tssp_domain::UserId>,
    /// Streaming file content.
    pub source: Box<dyn Read + Send>,
    /// Pre-hashed path for optimized I/O.
    pub staged_path: Option<PathBuf>,
    /// Pre-computed content hash.
    pub content_hash: Option<tssp_domain::ContentHash>,
    /// Pre-computed file size.
    pub size: Option<tssp_domain::FileSize>,
}

/// Result returned by the upload provider.
pub struct HttpUploadOutcome {
    /// Created or deduplicated file record.
    pub record: FileRecord,
    /// True when stored bytes already existed.
    pub deduplicated: bool,
}

/// Upload failure mapped to HTTP error responses.
#[derive(Debug)]
pub enum HttpUploadError {
    /// Client supplied invalid metadata.
    InvalidRequest {
        /// Short client-facing message.
        message: String,
    },
    /// Storage does not have enough room.
    InsufficientStorage {
        /// Short client-facing message.
        message: String,
    },
    /// Metadata store is busy.
    Busy {
        /// Short client-facing message.
        message: String,
    },
    /// Upload could not commit because of a conflict.
    Conflict {
        /// Short client-facing message.
        message: String,
    },
    /// Upload service is unavailable.
    Unavailable {
        /// Short client-facing message.
        message: String,
    },
    /// Unexpected server-side failure.
    Internal {
        /// Short client-facing message.
        message: String,
    },
}

impl HttpUploadError {
    pub(crate) fn response_parts(&self) -> (StatusCode, &'static str, String) {
        match self {
            Self::InvalidRequest { message } => {
                (StatusCode::BAD_REQUEST, "invalid_request", message.clone())
            }
            Self::InsufficientStorage { message } => {
                (status_code(507), "insufficient_storage", message.clone())
            }
            Self::Busy { message } => (
                StatusCode::SERVICE_UNAVAILABLE,
                "metadata_busy",
                message.clone(),
            ),
            Self::Conflict { message } => (StatusCode::CONFLICT, "conflict", message.clone()),
            Self::Unavailable { message } => (
                StatusCode::SERVICE_UNAVAILABLE,
                "upload_unavailable",
                message.clone(),
            ),
            Self::Internal { message } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                message.clone(),
            ),
        }
    }

    fn response(&self) -> Response {
        let (status, code, message) = self.response_parts();

        (
            status,
            Json(ErrorResponse {
                error: ErrorBody { code, message },
            }),
        )
            .into_response()
    }
}

/// Upload provider backed by the core application upload service.
pub struct ApplicationFileUploadProvider<B, R, I, C> {
    service: UploadService<B, R, I, C>,
}

impl<B, R, I, C> ApplicationFileUploadProvider<B, R, I, C> {
    /// Creates a provider from an upload service.
    #[must_use]
    pub const fn new(service: UploadService<B, R, I, C>) -> Self {
        Self { service }
    }
}

impl<B, R, I, C> FileUploadProvider for ApplicationFileUploadProvider<B, R, I, C>
where
    B: BlobStore + Send + Sync,
    R: FileRepository + Send + Sync,
    I: IdGenerator + Send + Sync,
    C: Clock + Send + Sync,
{
    fn upload(&self, mut request: HttpUploadRequest) -> Result<HttpUploadOutcome, HttpUploadError> {
        let tag_refs = request.tags.iter().map(String::as_str).collect::<Vec<_>>();
        let mut upload_request = UploadRequest {
            filename: &request.filename,
            mime_type: request.mime_type.as_deref(),
            tags: &tag_refs,
            pinned_at: request.pinned.then_some(1),
            folder_path: &request.folder_path,
            owner_id: request.owner_id.clone(),
            visibility: tssp_domain::Visibility::Private,
            public_token: None,
            source: request.source.as_mut(),
            staged_path: request.staged_path,
            content_hash: request.content_hash,
            size: request.size,
        };
        self.service
            .upload(&mut upload_request)
            .map(|outcome| HttpUploadOutcome {
                record: outcome.record,
                deduplicated: outcome.deduplicated,
            })
            .map_err(map_upload_error)
    }
}

pub(crate) async fn upload_file(
    State(state): State<HttpState>,
    auth: crate::auth::OptionalAuthContext,
    multipart: Multipart,
) -> Response {
    if let Err(error) = check_free_space(&state, &state.upload_temp_dir).await {
        return error.response();
    }

    let staged = match stage_multipart_upload(multipart, &state.upload_temp_dir).await {
        Ok(value) => value,
        Err(error) => return error.response(),
    };

    let _mutation_guard = state.storage_mutation_lock.lock().await;
    let owner_id = auth.0.as_ref().map(|ctx| ctx.user_id.clone());
    match upload_staged_file(&state, staged, owner_id).await {
        Ok(outcome) => upload_success_response(&outcome),
        Err(error) => error.response(),
    }
}

pub(crate) async fn upload_files_batch(
    State(state): State<HttpState>,
    auth: crate::auth::OptionalAuthContext,
    multipart: Multipart,
) -> Response {
    if let Err(error) = check_free_space(&state, &state.upload_temp_dir).await {
        return error.response();
    }

    let staged_files = match stage_batch_multipart_upload(multipart, &state.upload_temp_dir).await {
        Ok(value) => value,
        Err(error) => return error.response(),
    };

    let _mutation_guard = state.storage_mutation_lock.lock().await;
    let mut results = Vec::with_capacity(staged_files.len());
    for staged in staged_files {
        let filename = staged.filename.clone();
        let owner_id = auth.0.as_ref().map(|ctx| ctx.user_id.clone());
        let outcome = upload_staged_file(&state, staged, owner_id).await;
        results.push(BatchUploadItemResponse::from_upload_result(
            filename, outcome,
        ));
    }

    Json(BatchUploadResponse::from_results(results)).into_response()
}

async fn check_free_space(state: &HttpState, path: &Path) -> Result<(), HttpUploadError> {
    let settings = state.settings().clone();
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let stat = nix_statvfs(&path)?;
        let free_bytes = stat.free_bytes;
        let total_bytes = stat.total_bytes;
        let threshold = std::cmp::max(
            settings.storage_reserve_bytes,
            total_bytes * settings.storage_reserve_percent / 100,
        );
        if free_bytes < threshold {
            Err(HttpUploadError::InsufficientStorage {
                message: format!(
                    "insufficient storage: {free_bytes} bytes free, {threshold} bytes required"
                ),
            })
        } else {
            Ok(())
        }
    })
    .await
    .map_err(|e| HttpUploadError::Internal {
        message: format!("could not check disk space: {e}"),
    })?
}

struct DiskStat {
    free_bytes: u64,
    total_bytes: u64,
}

fn nix_statvfs(path: &Path) -> Result<DiskStat, HttpUploadError> {
    // Fall back to the parent directory if the path doesn't exist yet
    let check_path = if path.exists() {
        path.to_path_buf()
    } else {
        path.parent()
            .filter(|p| p.exists())
            .unwrap_or(std::path::Path::new("/tmp"))
            .to_path_buf()
    };

    let stat = rustix::fs::statvfs(&check_path).map_err(|e| HttpUploadError::Internal {
        message: format!("statvfs on {} failed: {e}", check_path.display()),
    })?;

    let block = stat.f_frsize;
    Ok(DiskStat {
        free_bytes: stat.f_bavail * block,
        total_bytes: stat.f_blocks * block,
    })
}

async fn upload_staged_file(
    state: &HttpState,
    staged: StagedMultipartUpload,
    owner_id: Option<tssp_domain::UserId>,
) -> Result<HttpUploadOutcome, HttpUploadError> {
    let staged_path = staged.temp_file.path().to_path_buf();
    let source = match staged.temp_file.reopen() {
        Ok(file) => file,
        Err(error) => {
            return Err(HttpUploadError::Internal {
                message: format!("could not reopen staged upload: {error}"),
            });
        }
    };

    let upload_provider = state.upload_provider.clone();
    let upload_request = HttpUploadRequest {
        filename: staged.filename,
        mime_type: staged.mime_type,
        tags: staged.tags,
        pinned: staged.pinned,
        folder_path: staged.folder_path,
        owner_id,
        source: Box::new(source),
        staged_path: Some(staged_path),
        content_hash: Some(staged.content_hash),
        size: Some(staged.size),
    };

    match task::spawn_blocking(move || upload_provider.upload(upload_request)).await {
        Ok(result) => result,
        Err(error) => Err(HttpUploadError::Internal {
            message: format!("upload worker failed: {error}"),
        }),
    }
}

pub(crate) struct StagedMultipartUpload {
    pub(crate) filename: String,
    pub(crate) mime_type: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) pinned: bool,
    pub(crate) folder_path: String,
    pub(crate) temp_file: NamedTempFile,
    pub(crate) content_hash: tssp_domain::ContentHash,
    pub(crate) size: tssp_domain::FileSize,
}

struct StagedBatchFile {
    filename: String,
    mime_type: Option<String>,
    temp_file: NamedTempFile,
    content_hash: tssp_domain::ContentHash,
    size: tssp_domain::FileSize,
}

pub(crate) async fn stage_multipart_upload(
    mut multipart: Multipart,
    upload_temp_dir: &Path,
) -> Result<StagedMultipartUpload, HttpUploadError> {
    std::fs::create_dir_all(upload_temp_dir).map_err(|error| HttpUploadError::Internal {
        message: format!("could not create upload temp directory: {error}"),
    })?;
    let mut temp_file =
        NamedTempFile::new_in(upload_temp_dir).map_err(|error| HttpUploadError::Internal {
            message: format!("could not create staged upload file: {error}"),
        })?;
    let mut filename = None;
    let mut mime_type = None;
    let mut tags = Vec::new();
    let mut pinned = false;
    let mut folder_path = String::new();
    let mut hash_and_size = None;

    while let Some(field) = next_field(&mut multipart).await? {
        let Some(name) = field.name().map(str::to_owned) else {
            return Err(HttpUploadError::InvalidRequest {
                message: "multipart field is missing a name".to_owned(),
            });
        };

        match name.as_str() {
            "file" => {
                ensure_single_file(filename.as_ref())?;
                let next_filename = field
                    .file_name()
                    .map_or_else(|| "upload.bin".to_owned(), str::to_owned);
                mime_type = field.content_type().map(str::to_owned);
                hash_and_size = Some(write_field_to_temp(field, &mut temp_file).await?);
                filename = Some(next_filename);
            }
            "tag" | "tags" => tags.push(field_text(field).await?),
            "pin" => pinned = parse_pin_field(&field_text(field).await?)?,
            "folder" | "folder_path" => {
                let text = field_text(field).await?;
                folder_path = crate::folders::normalize_folder_path(&text);
                crate::folders::validate_folder_path(&folder_path).map_err(|message| {
                    HttpUploadError::InvalidRequest {
                        message: format!("invalid folder path: {message}"),
                    }
                })?;
            }
            "destination" | "destination_hint" => {
                let _ignored = field_text(field).await?;
            }
            _unknown => return Err(unknown_field(&name)),
        }
    }

    let (content_hash, size) = hash_and_size.ok_or_else(|| HttpUploadError::InvalidRequest {
        message: "multipart upload must include a file field".to_owned(),
    })?;

    finish_staged_upload(
        filename,
        mime_type,
        tags,
        pinned,
        folder_path,
        temp_file,
        content_hash,
        size,
    )
}

async fn stage_batch_multipart_upload(
    mut multipart: Multipart,
    upload_temp_dir: &Path,
) -> Result<Vec<StagedMultipartUpload>, HttpUploadError> {
    std::fs::create_dir_all(upload_temp_dir).map_err(|error| HttpUploadError::Internal {
        message: format!("could not create upload temp directory: {error}"),
    })?;

    let mut files = Vec::new();
    let mut tags = Vec::new();
    let mut pinned = false;
    let mut folder_path = String::new();

    while let Some(field) = next_field(&mut multipart).await? {
        let Some(name) = field.name().map(str::to_owned) else {
            return Err(HttpUploadError::InvalidRequest {
                message: "multipart field is missing a name".to_owned(),
            });
        };

        match name.as_str() {
            "file" => files.push(stage_batch_file(field, upload_temp_dir).await?),
            "tag" | "tags" => tags.push(field_text(field).await?),
            "pin" => pinned = parse_pin_field(&field_text(field).await?)?,
            "folder" | "folder_path" => {
                let text = field_text(field).await?;
                folder_path = crate::folders::normalize_folder_path(&text);
                crate::folders::validate_folder_path(&folder_path).map_err(|message| {
                    HttpUploadError::InvalidRequest {
                        message: format!("invalid folder path: {message}"),
                    }
                })?;
            }
            "destination" | "destination_hint" => {
                let _ignored = field_text(field).await?;
            }
            _unknown => return Err(unknown_field(&name)),
        }
    }

    if files.is_empty() {
        return Err(HttpUploadError::InvalidRequest {
            message: "batch upload must include at least one file field".to_owned(),
        });
    }

    Ok(files
        .into_iter()
        .map(|file| StagedMultipartUpload {
            filename: file.filename,
            mime_type: file.mime_type,
            tags: tags.clone(),
            pinned,
            folder_path: folder_path.clone(),
            temp_file: file.temp_file,
            content_hash: file.content_hash,
            size: file.size,
        })
        .collect())
}

async fn stage_batch_file(
    field: axum::extract::multipart::Field<'_>,
    upload_temp_dir: &Path,
) -> Result<StagedBatchFile, HttpUploadError> {
    let filename = field
        .file_name()
        .map_or_else(|| "upload.bin".to_owned(), str::to_owned);
    let mime_type = field.content_type().map(str::to_owned);
    let mut temp_file =
        NamedTempFile::new_in(upload_temp_dir).map_err(|error| HttpUploadError::Internal {
            message: format!("could not create staged upload file: {error}"),
        })?;
    let (content_hash, size) = write_field_to_temp(field, &mut temp_file).await?;
    sync_staged_upload(&mut temp_file)?;

    Ok(StagedBatchFile {
        filename,
        mime_type,
        temp_file,
        content_hash,
        size,
    })
}

async fn next_field(
    multipart: &mut Multipart,
) -> Result<Option<axum::extract::multipart::Field<'_>>, HttpUploadError> {
    multipart
        .next_field()
        .await
        .map_err(|error| HttpUploadError::InvalidRequest {
            message: format!("invalid multipart upload: {error}"),
        })
}

fn ensure_single_file(filename: Option<&String>) -> Result<(), HttpUploadError> {
    if filename.is_some() {
        return Err(HttpUploadError::InvalidRequest {
            message: "single-file upload accepts exactly one file field".to_owned(),
        });
    }
    Ok(())
}

fn unknown_field(name: &str) -> HttpUploadError {
    HttpUploadError::InvalidRequest {
        message: format!("unknown multipart field: {name}"),
    }
}

fn finish_staged_upload(
    filename: Option<String>,
    mime_type: Option<String>,
    tags: Vec<String>,
    pinned: bool,
    folder_path: String,
    mut temp_file: NamedTempFile,
    content_hash: tssp_domain::ContentHash,
    size: tssp_domain::FileSize,
) -> Result<StagedMultipartUpload, HttpUploadError> {
    let Some(filename) = filename else {
        return Err(HttpUploadError::InvalidRequest {
            message: "multipart upload must include a file field".to_owned(),
        });
    };

    sync_staged_upload(&mut temp_file)?;

    Ok(StagedMultipartUpload {
        filename,
        mime_type,
        tags,
        pinned,
        folder_path,
        temp_file,
        content_hash,
        size,
    })
}

fn sync_staged_upload(temp_file: &mut NamedTempFile) -> Result<(), HttpUploadError> {
    temp_file
        .flush()
        .map_err(|error| HttpUploadError::Internal {
            message: format!("could not flush staged upload: {error}"),
        })?;
    temp_file
        .as_file()
        .sync_all()
        .map_err(|error| HttpUploadError::Internal {
            message: format!("could not sync staged upload: {error}"),
        })?;
    Ok(())
}

async fn write_field_to_temp(
    mut field: axum::extract::multipart::Field<'_>,
    temp_file: &mut NamedTempFile,
) -> Result<(tssp_domain::ContentHash, tssp_domain::FileSize), HttpUploadError> {
    let mut hasher = blake3::Hasher::new();
    let mut size = 0_u64;

    while let Some(chunk) =
        field
            .chunk()
            .await
            .map_err(|error| HttpUploadError::InvalidRequest {
                message: format!("invalid file field: {error}"),
            })?
    {
        hasher.update(&chunk);
        size = size
            .checked_add(chunk.len() as u64)
            .ok_or_else(|| HttpUploadError::Internal {
                message: "upload size overflow".to_owned(),
            })?;

        temp_file
            .write_all(&chunk)
            .map_err(|error| HttpUploadError::Internal {
                message: format!("could not write staged upload: {error}"),
            })?;
    }

    let hash = tssp_domain::ContentHash::new(hasher.finalize().to_hex().as_str()).map_err(
        |error| HttpUploadError::Internal {
            message: format!("could not compute hash: {error}"),
        },
    )?;

    Ok((hash, tssp_domain::FileSize::new(size)))
}

async fn field_text(field: axum::extract::multipart::Field<'_>) -> Result<String, HttpUploadError> {
    field
        .text()
        .await
        .map_err(|error| HttpUploadError::InvalidRequest {
            message: format!("invalid text field: {error}"),
        })
}

fn parse_pin_field(value: &str) -> Result<bool, HttpUploadError> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Ok(true);
    }

    if matches!(normalized.as_str(), "1" | "true" | "yes" | "on") {
        return Ok(true);
    }
    if matches!(normalized.as_str(), "0" | "false" | "no" | "off") {
        return Ok(false);
    }

    Err(HttpUploadError::InvalidRequest {
        message: format!("invalid pin value: {value}"),
    })
}

fn upload_success_response(outcome: &HttpUploadOutcome) -> Response {
    let status = if outcome.deduplicated {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };
    let record = FileRecordResponse::from_record(&outcome.record);
    let mut response = (status, Json(record)).into_response();
    response.headers_mut().insert(
        HeaderName::from_static("x-tssp-deduplicated"),
        HeaderValue::from_static(if outcome.deduplicated {
            "true"
        } else {
            "false"
        }),
    );
    if let Ok(location) = HeaderValue::from_str(&format!("/api/v1/files/{}", outcome.record.id)) {
        response.headers_mut().insert(LOCATION, location);
    }
    response
}

#[derive(Debug, Serialize)]
struct BatchUploadResponse {
    schema_version: u8,
    results: Vec<BatchUploadItemResponse>,
    created_count: usize,
    deduplicated_count: usize,
    failed_count: usize,
}

impl BatchUploadResponse {
    fn from_results(results: Vec<BatchUploadItemResponse>) -> Self {
        let created_count = results
            .iter()
            .filter(|result| result.outcome == "created")
            .count();
        let deduplicated_count = results
            .iter()
            .filter(|result| result.outcome == "deduplicated")
            .count();
        let failed_count = results
            .iter()
            .filter(|result| result.outcome == "failed")
            .count();

        Self {
            schema_version: 1,
            results,
            created_count,
            deduplicated_count,
            failed_count,
        }
    }
}

#[derive(Debug, Serialize)]
struct BatchUploadItemResponse {
    name: String,
    outcome: &'static str,
    http_status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<FileRecordResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<BatchUploadErrorResponse>,
}

impl BatchUploadItemResponse {
    fn from_upload_result(
        name: String,
        result: Result<HttpUploadOutcome, HttpUploadError>,
    ) -> Self {
        match result {
            Ok(outcome) => {
                let (outcome_name, http_status) = if outcome.deduplicated {
                    ("deduplicated", StatusCode::OK)
                } else {
                    ("created", StatusCode::CREATED)
                };
                Self {
                    name,
                    outcome: outcome_name,
                    http_status: http_status.as_u16(),
                    file: Some(FileRecordResponse::from_record(&outcome.record)),
                    error: None,
                }
            }
            Err(error) => {
                let (status, code, message) = error.response_parts();
                Self {
                    name,
                    outcome: "failed",
                    http_status: status.as_u16(),
                    file: None,
                    error: Some(BatchUploadErrorResponse {
                        code: code.to_owned(),
                        message,
                    }),
                }
            }
        }
    }
}

#[derive(Debug, Serialize)]
struct BatchUploadErrorResponse {
    code: String,
    message: String,
}

/// JSON representation of one file record.
#[derive(Debug, Serialize, Deserialize)]
pub struct FileRecordResponse {
    /// Stable response schema version.
    pub schema_version: u8,
    /// Client-visible file id.
    pub id: String,
    /// Original filename.
    pub name: String,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Lowercase BLAKE3 content hash.
    pub content_hash: String,
    /// MIME type used for serving the file.
    pub mime_type: String,
    /// Server-side UTC upload timestamp in Unix seconds.
    pub uploaded_at: i64,
    /// Normalized tag display names.
    pub tags: Vec<String>,
    /// Whether the file is pinned.
    pub pinned: bool,
    /// Virtual folder path, empty at bucket root.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub folder_path: String,
    /// Public visibility.
    pub visibility: String,
    /// Public download token when visible.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_token: Option<String>,
}

impl FileRecordResponse {
    pub(crate) fn from_record(record: &FileRecord) -> Self {
        Self {
            schema_version: 1,
            id: record.id.as_str().to_owned(),
            name: record.name.original().to_owned(),
            size_bytes: record.size.bytes(),
            content_hash: record.content_hash.as_str().to_owned(),
            mime_type: record.mime_type.as_str().to_owned(),
            uploaded_at: record.uploaded_at.seconds(),
            tags: record
                .tags
                .iter()
                .map(|tag| tag.display().to_owned())
                .collect(),
            pinned: record.is_pinned(),
            folder_path: record.folder_path.clone(),
            visibility: record.visibility.as_str().to_owned(),
            public_token: record.public_token.clone(),
        }
    }
}

fn map_upload_error(error: UploadError) -> HttpUploadError {
    match error {
        UploadError::InvalidRequest(error) => HttpUploadError::InvalidRequest {
            message: error.to_string(),
        },
        UploadError::IdGeneration(error) => HttpUploadError::Internal {
            message: error.to_string(),
        },
        UploadError::BlobStore(tssp_ports::BlobStoreError::InsufficientStorage {
            required_bytes,
            available_bytes,
        }) => HttpUploadError::InsufficientStorage {
            message: format!(
                "not enough storage for upload: required {required_bytes} bytes, available {available_bytes} bytes"
            ),
        },
        UploadError::BlobStore(error) => HttpUploadError::Internal {
            message: error.to_string(),
        },
        UploadError::DedupLookup(error) => map_commit_error(&error, None),
        UploadError::CommitFailed {
            repository,
            cleanup,
        } => map_commit_error(&repository, cleanup.as_ref()),
    }
}

fn map_commit_error(
    repository: &RepositoryError,
    cleanup: Option<&tssp_ports::BlobStoreError>,
) -> HttpUploadError {
    if let Some(cleanup_error) = cleanup {
        return HttpUploadError::Internal {
            message: format!(
                "metadata commit failed and cleanup also failed: {repository}; {cleanup_error}"
            ),
        };
    }

    match repository {
        RepositoryError::Busy => HttpUploadError::Busy {
            message: "metadata store is busy; retry the upload".to_owned(),
        },
        RepositoryError::Conflict { message } => HttpUploadError::Conflict {
            message: message.clone(),
        },
        RepositoryError::NotFound | RepositoryError::OperationFailed { .. } => {
            HttpUploadError::Internal {
                message: repository.to_string(),
            }
        }
    }
}

fn status_code(code: u16) -> StatusCode {
    StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};

    use axum::body::to_bytes;
    use axum::http::StatusCode;
    use tssp_app::UploadError;
    use tssp_domain::{
        ContentHash, FileId, FileName, FileRecord, FileSize, MimeType, StorageHandle, Tag,
        UnixTimestamp,
    };
    use tssp_ports::{BlobStoreError, IdGenerationError, RepositoryError};

    use super::{
        ensure_single_file, finish_staged_upload, map_commit_error, map_upload_error,
        parse_pin_field, status_code, upload_success_response, FileUploadProvider, HttpUploadError,
        HttpUploadOutcome, HttpUploadRequest, StaticFileUploadProvider,
    };

    #[test]
    fn parse_pin_field_accepts_common_boolean_values() {
        assert!(parse_pin_field("").unwrap_or_else(|error| panic!("{error:?}")));
        assert!(parse_pin_field(" YES ").unwrap_or_else(|error| panic!("{error:?}")));
        assert!(parse_pin_field("on").unwrap_or_else(|error| panic!("{error:?}")));
        assert!(!parse_pin_field("0").unwrap_or_else(|error| panic!("{error:?}")));
        assert!(!parse_pin_field("false").unwrap_or_else(|error| panic!("{error:?}")));
    }

    #[test]
    fn parse_pin_field_rejects_unknown_value() {
        let result = parse_pin_field("maybe");

        assert!(
            matches!(result, Err(HttpUploadError::InvalidRequest { message }) if message.contains("maybe"))
        );
    }

    #[test]
    fn ensure_single_file_rejects_duplicate_file_fields() {
        let filename = "one.txt".to_owned();

        let result = ensure_single_file(Some(&filename));

        assert!(
            matches!(result, Err(HttpUploadError::InvalidRequest { message }) if message.contains("exactly one"))
        );
    }

    #[test]
    fn finish_staged_upload_requires_file_field() {
        let temp = tempfile::NamedTempFile::new()
            .unwrap_or_else(|error| panic!("temp file failed: {error}"));

        let result = finish_staged_upload(
            None,
            None,
            Vec::new(),
            false,
            String::new(),
            temp,
            content_hash(),
            FileSize::new(0),
        );

        assert!(
            matches!(result, Err(HttpUploadError::InvalidRequest { message }) if message.contains("file field"))
        );
    }

    #[test]
    fn finish_staged_upload_flushes_and_returns_metadata() {
        let mut temp = tempfile::NamedTempFile::new()
            .unwrap_or_else(|error| panic!("temp file failed: {error}"));
        temp.write_all(b"hello")
            .unwrap_or_else(|error| panic!("write failed: {error}"));

        let staged = finish_staged_upload(
            Some("note.txt".to_owned()),
            Some("text/plain".to_owned()),
            vec!["Docs".to_owned()],
            true,
            "photos".to_owned(),
            temp,
            content_hash(),
            FileSize::new(5),
        )
        .unwrap_or_else(|error| panic!("finish failed: {error:?}"));

        assert_eq!(staged.filename, "note.txt");
        assert_eq!(staged.mime_type.as_deref(), Some("text/plain"));
        assert_eq!(staged.tags, vec!["Docs"]);
        assert!(staged.pinned);
        assert_eq!(staged.folder_path, "photos");
        assert_eq!(staged.size.bytes(), 5);
    }

    #[test]
    fn static_upload_provider_reports_unavailable() {
        let provider = StaticFileUploadProvider;
        let request = HttpUploadRequest {
            filename: "note.txt".to_owned(),
            mime_type: None,
            tags: Vec::new(),
            pinned: false,
            folder_path: String::new(),
            owner_id: None,
            source: Box::new(Cursor::new(Vec::<u8>::new())),
            staged_path: None,
            content_hash: None,
            size: None,
        };

        let result = provider.upload(request);

        assert!(
            matches!(result, Err(HttpUploadError::Unavailable { message }) if message.contains("not configured"))
        );
    }

    #[tokio::test]
    async fn http_upload_error_response_uses_stable_status_and_error_codes() {
        let cases = vec![
            (
                HttpUploadError::InvalidRequest {
                    message: "bad".to_owned(),
                },
                StatusCode::BAD_REQUEST,
                "invalid_request",
            ),
            (
                HttpUploadError::InsufficientStorage {
                    message: "full".to_owned(),
                },
                status_code(507),
                "insufficient_storage",
            ),
            (
                HttpUploadError::Busy {
                    message: "busy".to_owned(),
                },
                StatusCode::SERVICE_UNAVAILABLE,
                "metadata_busy",
            ),
            (
                HttpUploadError::Conflict {
                    message: "conflict".to_owned(),
                },
                StatusCode::CONFLICT,
                "conflict",
            ),
            (
                HttpUploadError::Unavailable {
                    message: "missing".to_owned(),
                },
                StatusCode::SERVICE_UNAVAILABLE,
                "upload_unavailable",
            ),
            (
                HttpUploadError::Internal {
                    message: "failed".to_owned(),
                },
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
            ),
        ];

        for (error, expected_status, expected_code) in cases {
            let response = error.response();

            assert_eq!(response.status(), expected_status);
            let body = to_bytes(response.into_body(), 1024)
                .await
                .unwrap_or_else(|error| panic!("body read failed: {error}"));
            let parsed: serde_json::Value = serde_json::from_slice(&body)
                .unwrap_or_else(|error| panic!("json parse failed: {error}"));
            assert_eq!(parsed["error"]["code"], expected_code);
        }
    }

    #[test]
    fn map_upload_error_translates_application_errors() {
        let Err(invalid) = FileName::new("") else {
            panic!("empty filename unexpectedly parsed");
        };
        assert!(matches!(
            map_upload_error(UploadError::InvalidRequest(invalid)),
            HttpUploadError::InvalidRequest { .. }
        ));

        assert!(matches!(
            map_upload_error(UploadError::IdGeneration(IdGenerationError {
                message: "id failed".to_owned(),
            })),
            HttpUploadError::Internal { .. }
        ));

        assert!(matches!(
            map_upload_error(UploadError::BlobStore(
                BlobStoreError::InsufficientStorage {
                    required_bytes: 10,
                    available_bytes: 1,
                },
            )),
            HttpUploadError::InsufficientStorage { .. }
        ));

        assert!(matches!(
            map_upload_error(UploadError::BlobStore(BlobStoreError::ReadFailed {
                message: "read failed".to_owned(),
            })),
            HttpUploadError::Internal { .. }
        ));

        assert!(matches!(
            map_upload_error(UploadError::DedupLookup(RepositoryError::Busy)),
            HttpUploadError::Busy { .. }
        ));
    }

    #[test]
    fn map_commit_error_preserves_retriable_and_conflict_failures() {
        assert!(matches!(
            map_commit_error(&RepositoryError::Busy, None),
            HttpUploadError::Busy { .. }
        ));
        assert!(matches!(
            map_commit_error(
                &RepositoryError::Conflict {
                    message: "duplicate id".to_owned(),
                },
                None,
            ),
            HttpUploadError::Conflict { .. }
        ));
        assert!(matches!(
            map_commit_error(&RepositoryError::NotFound, None),
            HttpUploadError::Internal { .. }
        ));
        assert!(matches!(
            map_commit_error(
                &RepositoryError::OperationFailed {
                    message: "query failed".to_owned(),
                },
                Some(&BlobStoreError::CleanupFailed {
                    handle: storage_handle(),
                    message: "cleanup failed".to_owned(),
                }),
            ),
            HttpUploadError::Internal { message } if message.contains("cleanup also failed")
        ));
    }

    #[test]
    fn status_code_falls_back_to_internal_error_for_invalid_values() {
        assert_eq!(status_code(99), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn upload_success_response_sets_status_headers_and_body() {
        let outcome = HttpUploadOutcome {
            record: file_record(),
            deduplicated: true,
        };

        let response = upload_success_response(&outcome);

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("x-tssp-deduplicated")
                .and_then(|value| value.to_str().ok()),
            Some("true")
        );
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::LOCATION)
                .and_then(|value| value.to_str().ok()),
            Some("/api/v1/files/file-test")
        );
        let body = to_bytes(response.into_body(), 2048)
            .await
            .unwrap_or_else(|error| panic!("body read failed: {error}"));
        let parsed: serde_json::Value = serde_json::from_slice(&body)
            .unwrap_or_else(|error| panic!("json parse failed: {error}"));
        assert_eq!(parsed["name"], "note.txt");
        assert_eq!(parsed["pinned"], true);
    }

    fn file_record() -> FileRecord {
        FileRecord {
            id: file_id("file-test"),
            name: filename("note.txt"),
            size: FileSize::new(12),
            content_hash: content_hash(),
            mime_type: mime_type("text/plain"),
            storage_handle: storage_handle(),
            uploaded_at: timestamp(1_700_000_000),
            tags: vec![tag_value("Docs")],
            pinned_at: Some(1),
            folder_path: String::new(),
            owner_id: None,
            visibility: tssp_domain::Visibility::Private,
            public_token: None,
        }
    }

    fn file_id(value: &str) -> FileId {
        FileId::new(value).unwrap_or_else(|error| panic!("invalid file id: {error}"))
    }

    fn filename(value: &str) -> FileName {
        FileName::new(value).unwrap_or_else(|error| panic!("invalid filename: {error}"))
    }

    fn content_hash() -> ContentHash {
        ContentHash::new("abcdefabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123")
            .unwrap_or_else(|error| panic!("invalid hash: {error}"))
    }

    fn mime_type(value: &str) -> MimeType {
        MimeType::new(value).unwrap_or_else(|error| panic!("invalid mime type: {error}"))
    }

    fn storage_handle() -> StorageHandle {
        StorageHandle::new("blobs/ab/cd/abcdef")
            .unwrap_or_else(|error| panic!("invalid storage handle: {error}"))
    }

    fn timestamp(seconds: i64) -> UnixTimestamp {
        UnixTimestamp::new(seconds).unwrap_or_else(|error| panic!("invalid timestamp: {error}"))
    }

    fn tag_value(value: &str) -> Tag {
        Tag::new(value).unwrap_or_else(|error| panic!("invalid tag: {error}"))
    }
}
