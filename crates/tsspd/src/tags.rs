//! Tag HTTP delivery.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use tssp_app::{TagError, TagService};
use tssp_domain::FileId;
use tssp_ports::{FileRepository, RepositoryError, TagSummary};

use crate::{ErrorBody, ErrorResponse, HttpState};

#[derive(Debug, Deserialize)]
pub struct CreateTagBody {
    pub name: String,
}

/// Handles HTTP tag operations through the application layer.
pub trait FileTagProvider: Send + Sync {
    /// Lists tags.
    ///
    /// # Errors
    ///
    /// Returns an error if the repository query fails.
    fn list_tags(&self) -> Result<Vec<TagSummary>, HttpTagError>;

    /// Adds tags to a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the tag operation fails.
    fn add_tags(&self, file_id: FileId, tags: Vec<String>)
        -> Result<HttpTagMutation, HttpTagError>;

    /// Removes a tag from a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the tag operation fails.
    fn remove_tag(&self, file_id: FileId, tag: String) -> Result<HttpTagMutation, HttpTagError>;
}

#[derive(Debug)]
pub(crate) struct StaticFileTagProvider;

impl FileTagProvider for StaticFileTagProvider {
    fn list_tags(&self) -> Result<Vec<TagSummary>, HttpTagError> {
        Err(HttpTagError::Unavailable {
            message: "tag service is not configured".to_owned(),
        })
    }

    fn add_tags(
        &self,
        _file_id: FileId,
        _tags: Vec<String>,
    ) -> Result<HttpTagMutation, HttpTagError> {
        Err(HttpTagError::Unavailable {
            message: "tag service is not configured".to_owned(),
        })
    }

    fn remove_tag(&self, _file_id: FileId, _tag: String) -> Result<HttpTagMutation, HttpTagError> {
        Err(HttpTagError::Unavailable {
            message: "tag service is not configured".to_owned(),
        })
    }
}

/// Successful tag mutation outcome.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HttpTagMutation {
    /// Number of affected rows.
    pub changed_count: u64,
}

/// Tag failure mapped to HTTP error responses.
#[derive(Debug, Clone)]
pub enum HttpTagError {
    /// Client supplied invalid data.
    InvalidRequest {
        /// Short client-facing message.
        message: String,
    },
    /// File id does not exist.
    NotFound {
        /// Short client-facing message.
        message: String,
    },
    /// User does not have permission.
    Forbidden {
        /// Short client-facing message.
        message: String,
    },
    /// Metadata store is busy.
    Busy {
        /// Short client-facing message.
        message: String,
    },
    /// Tag service is unavailable.
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

impl HttpTagError {
    fn response(&self) -> Response {
        let (status, code, message) = match self {
            Self::InvalidRequest { message } => {
                (StatusCode::BAD_REQUEST, "invalid_request", message.clone())
            }
            Self::NotFound { message } => {
                (StatusCode::NOT_FOUND, "file_not_found", message.clone())
            }
            Self::Forbidden { message } => (
                StatusCode::FORBIDDEN,
                "forbidden",
                message.clone(),
            ),
            Self::Busy { message } => (
                StatusCode::SERVICE_UNAVAILABLE,
                "metadata_busy",
                message.clone(),
            ),
            Self::Unavailable { message } => (
                StatusCode::SERVICE_UNAVAILABLE,
                "tag_unavailable",
                message.clone(),
            ),
            Self::Internal { message } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                message.clone(),
            ),
        };
        (
            status,
            Json(ErrorResponse {
                error: ErrorBody { code, message },
            }),
        )
            .into_response()
    }
}

/// Tag provider backed by the core application tag service.
pub struct ApplicationFileTagProvider<R> {
    service: TagService<R>,
}

impl<R> ApplicationFileTagProvider<R> {
    /// Creates a provider from a tag service.
    #[must_use]
    pub const fn new(service: TagService<R>) -> Self {
        Self { service }
    }
}

impl<R> FileTagProvider for ApplicationFileTagProvider<R>
where
    R: FileRepository + Send + Sync,
{
    fn list_tags(&self) -> Result<Vec<TagSummary>, HttpTagError> {
        self.service.list_tags().map_err(map_tag_error)
    }

    fn add_tags(
        &self,
        file_id: FileId,
        tags: Vec<String>,
    ) -> Result<HttpTagMutation, HttpTagError> {
        let refs = tags.iter().map(String::as_str).collect::<Vec<_>>();
        self.service
            .add_tags(&file_id, &refs)
            .map(|outcome| HttpTagMutation {
                changed_count: outcome.changed_count,
            })
            .map_err(map_tag_error)
    }

    fn remove_tag(&self, file_id: FileId, tag: String) -> Result<HttpTagMutation, HttpTagError> {
        self.service
            .remove_tag(&file_id, &tag)
            .map(|outcome| HttpTagMutation {
                changed_count: outcome.changed_count,
            })
            .map_err(map_tag_error)
    }
}

pub(crate) async fn create_tag(
    Json(body): Json<CreateTagBody>,
) -> Response {
    let tag_name = body.name.trim();
    if tag_name.is_empty() {
        return HttpTagError::InvalidRequest {
            message: "tag name cannot be empty".to_owned(),
        }
        .response();
    }

    if tag_name.len() > 50 {
        return HttpTagError::InvalidRequest {
            message: "tag name must be 50 characters or less".to_owned(),
        }
        .response();
    }

    if !tag_name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return HttpTagError::InvalidRequest {
            message: "tag name must contain only alphanumeric characters, hyphens, and underscores"
                .to_owned(),
        }
        .response();
    }

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "schema_version": 1,
            "name": tag_name,
            "file_count": 0,
        })),
    )
        .into_response()
}

pub(crate) async fn list_tags(
    State(state): State<HttpState>,
    auth: crate::auth::AuthContext,
) -> Response {
    // For non-admin users, only show tags from their own files
    // For admin users, show all tags
    if auth.is_admin() {
        let provider = state.tag_provider.clone();
        return match tokio::task::spawn_blocking(move || provider.list_tags()).await {
            Ok(Ok(tags)) => (StatusCode::OK, Json(TagListResponse::from_tags(&tags)))
                .into_response(),
            Ok(Err(error)) => error.response(),
            Err(error) => HttpTagError::Internal {
                message: format!("tag worker failed: {error}"),
            }
            .response(),
        };
    }

    // Non-admin: get user's files and extract unique tags
    let mut query = tssp_ports::ListQuery::default();
    query.owner_id = Some(auth.user_id.clone());
    query.limit = 10_000;

    match tokio::task::spawn_blocking(move || state.stats_provider.list_files(&query)).await {
        Ok(Ok(page)) => {
            let mut tag_counts: std::collections::HashMap<String, u64> =
                std::collections::HashMap::new();

            for file in &page.files {
                for tag in &file.tags {
                    *tag_counts.entry(tag.key().to_string()).or_insert(0) += 1;
                }
            }

            let tags: Vec<tssp_ports::TagSummary> = tag_counts
                .into_iter()
                .filter_map(|(key, count)| {
                    tssp_domain::Tag::new(key).ok().map(|tag| {
                        tssp_ports::TagSummary {
                            tag,
                            file_count: count,
                        }
                    })
                })
                .collect();

            (StatusCode::OK, Json(TagListResponse::from_tags(&tags))).into_response()
        }
        Ok(Err(error)) => HttpTagError::Internal {
            message: error.clone(),
        }
        .response(),
        Err(error) => HttpTagError::Internal {
            message: format!("tag worker failed: {error}"),
        }
        .response(),
    }
}

pub(crate) async fn add_tags(
    State(state): State<HttpState>,
    auth: crate::auth::AuthContext,
    Path(id): Path<String>,
    Json(tags): Json<Vec<String>>,
) -> Response {
    if tags.is_empty() {
        return HttpTagError::InvalidRequest {
            message: "request body must contain at least one tag".to_owned(),
        }
        .response();
    }
    let file_id = match FileId::new(id) {
        Ok(value) => value,
        Err(error) => {
            return HttpTagError::InvalidRequest {
                message: error.to_string(),
            }
            .response()
        }
    };

    let file = match state.stats_provider.find_file(&file_id) {
        Ok(Some(f)) => f,
        Ok(None) => {
            return HttpTagError::NotFound {
                message: "file not found".to_owned(),
            }
            .response()
        }
        Err(e) => {
            return HttpTagError::Internal {
                message: e,
            }
            .response()
        }
    };

    if !(auth.is_admin() || file.owner_id.as_ref() == Some(&auth.user_id)) {
        return HttpTagError::Forbidden {
            message: "you do not have permission to tag this file".to_owned(),
        }
        .response();
    }

    let provider = state.tag_provider.clone();

    match tokio::task::spawn_blocking(move || provider.add_tags(file_id, tags)).await {
        Ok(Ok(outcome)) => tag_mutation_response(outcome.changed_count),
        Ok(Err(error)) => error.response(),
        Err(error) => HttpTagError::Internal {
            message: format!("tag worker failed: {error}"),
        }
        .response(),
    }
}

pub(crate) async fn remove_tag(
    State(state): State<HttpState>,
    auth: crate::auth::AuthContext,
    Path((id, tag)): Path<(String, String)>,
) -> Response {
    let file_id = match FileId::new(id) {
        Ok(value) => value,
        Err(error) => {
            return HttpTagError::InvalidRequest {
                message: error.to_string(),
            }
            .response()
        }
    };

    let file = match state.stats_provider.find_file(&file_id) {
        Ok(Some(f)) => f,
        Ok(None) => {
            return HttpTagError::NotFound {
                message: "file not found".to_owned(),
            }
            .response()
        }
        Err(e) => {
            return HttpTagError::Internal {
                message: e,
            }
            .response()
        }
    };

    if !(auth.is_admin() || file.owner_id.as_ref() == Some(&auth.user_id)) {
        return HttpTagError::Forbidden {
            message: "you do not have permission to tag this file".to_owned(),
        }
        .response();
    }

    let provider = state.tag_provider.clone();

    match tokio::task::spawn_blocking(move || provider.remove_tag(file_id, tag)).await {
        Ok(Ok(outcome)) => tag_mutation_response(outcome.changed_count),
        Ok(Err(error)) => error.response(),
        Err(error) => HttpTagError::Internal {
            message: format!("tag worker failed: {error}"),
        }
        .response(),
    }
}

fn tag_mutation_response(changed_count: u64) -> Response {
    (
        StatusCode::OK,
        Json(TagMutationResponse {
            schema_version: 1,
            changed_count,
        }),
    )
        .into_response()
}

fn map_tag_error(error: TagError) -> HttpTagError {
    match error {
        TagError::InvalidRequest(error) => HttpTagError::InvalidRequest {
            message: error.to_string(),
        },
        TagError::Repository(RepositoryError::NotFound) => HttpTagError::NotFound {
            message: "file was not found".to_owned(),
        },
        TagError::Repository(RepositoryError::Busy) => HttpTagError::Busy {
            message: "metadata store is busy; retry the tag request".to_owned(),
        },
        TagError::Repository(RepositoryError::Conflict { message }) => {
            HttpTagError::InvalidRequest { message }
        }
        TagError::Repository(RepositoryError::OperationFailed { message }) => {
            HttpTagError::Internal { message }
        }
    }
}

#[derive(Debug, Serialize)]
struct TagListResponse {
    schema_version: u8,
    tags: Vec<TagResponse>,
}

impl TagListResponse {
    fn from_tags(tags: &[TagSummary]) -> Self {
        Self {
            schema_version: 1,
            tags: tags.iter().map(TagResponse::from_summary).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
struct TagResponse {
    name: String,
    file_count: u64,
}

impl TagResponse {
    fn from_summary(summary: &TagSummary) -> Self {
        Self {
            name: summary.tag.display().to_owned(),
            file_count: summary.file_count,
        }
    }
}

#[derive(Debug, Serialize)]
struct TagMutationResponse {
    schema_version: u8,
    changed_count: u64,
}
