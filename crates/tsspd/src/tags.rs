//! Tag HTTP delivery.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use tssp_app::{TagError, TagService};
use tssp_domain::FileId;
use tssp_ports::{FileRepository, RepositoryError, TagSummary};

use crate::{ErrorBody, ErrorResponse, HttpState};

/// Handles HTTP tag listing and mutation through the application layer.
pub trait FileTagProvider: Send + Sync {
    /// Lists all tags with file counts.
    ///
    /// # Errors
    ///
    /// Returns [`HttpTagError`] when metadata is unavailable.
    fn list_tags(&self) -> Result<Vec<TagSummary>, HttpTagError>;

    /// Adds tags to a file idempotently.
    ///
    /// # Errors
    ///
    /// Returns [`HttpTagError`] when the request is invalid or metadata fails.
    fn add_tags(&self, id: FileId, tags: Vec<String>) -> Result<HttpTagMutation, HttpTagError>;

    /// Removes one tag from a file idempotently.
    ///
    /// # Errors
    ///
    /// Returns [`HttpTagError`] when the request is invalid or metadata fails.
    fn remove_tag(&self, id: FileId, tag: String) -> Result<HttpTagMutation, HttpTagError>;
}

#[derive(Debug)]
pub(crate) struct StaticFileTagProvider;

impl FileTagProvider for StaticFileTagProvider {
    fn list_tags(&self) -> Result<Vec<TagSummary>, HttpTagError> {
        Err(HttpTagError::Unavailable {
            message: "tag service is not configured".to_owned(),
        })
    }

    fn add_tags(&self, _id: FileId, _tags: Vec<String>) -> Result<HttpTagMutation, HttpTagError> {
        Err(HttpTagError::Unavailable {
            message: "tag service is not configured".to_owned(),
        })
    }

    fn remove_tag(&self, _id: FileId, _tag: String) -> Result<HttpTagMutation, HttpTagError> {
        Err(HttpTagError::Unavailable {
            message: "tag service is not configured".to_owned(),
        })
    }
}

/// Successful tag mutation outcome.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HttpTagMutation {
    /// Number of tag associations created or removed.
    pub changed_count: u64,
}

/// Tag failure mapped to HTTP error responses.
#[derive(Debug)]
pub enum HttpTagError {
    /// Client supplied invalid tag text.
    InvalidRequest {
        /// Short client-facing message.
        message: String,
    },
    /// File id does not exist.
    NotFound {
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

    fn add_tags(&self, id: FileId, tags: Vec<String>) -> Result<HttpTagMutation, HttpTagError> {
        let tag_refs = tags.iter().map(String::as_str).collect::<Vec<_>>();
        self.service
            .add_tags(&id, &tag_refs)
            .map(|outcome| HttpTagMutation {
                changed_count: outcome.changed_count,
            })
            .map_err(map_tag_error)
    }

    fn remove_tag(&self, id: FileId, tag: String) -> Result<HttpTagMutation, HttpTagError> {
        self.service
            .remove_tag(&id, &tag)
            .map(|outcome| HttpTagMutation {
                changed_count: outcome.changed_count,
            })
            .map_err(map_tag_error)
    }
}

pub(crate) async fn list_tags(State(state): State<HttpState>) -> Response {
    let provider = state.tag_provider.clone();
    match tokio::task::spawn_blocking(move || provider.list_tags()).await {
        Ok(Ok(tags)) => (StatusCode::OK, Json(TagListResponse::from_tags(&tags))).into_response(),
        Ok(Err(error)) => error.response(),
        Err(error) => HttpTagError::Internal {
            message: format!("tag worker failed: {error}"),
        }
        .response(),
    }
}

pub(crate) async fn add_tags(
    State(state): State<HttpState>,
    Path(id): Path<String>,
    Json(tags): Json<Vec<String>>,
) -> Response {
    if tags.is_empty() {
        return invalid_request_response("request body must contain at least one tag".to_owned());
    }
    let file_id = match FileId::new(id) {
        Ok(value) => value,
        Err(error) => return invalid_file_id_response(error.to_string()),
    };
    let provider = state.tag_provider.clone();
    match tokio::task::spawn_blocking(move || provider.add_tags(file_id, tags)).await {
        Ok(Ok(outcome)) => tag_mutation_response(outcome),
        Ok(Err(error)) => error.response(),
        Err(error) => HttpTagError::Internal {
            message: format!("tag worker failed: {error}"),
        }
        .response(),
    }
}

pub(crate) async fn remove_tag(
    State(state): State<HttpState>,
    Path((id, tag)): Path<(String, String)>,
) -> Response {
    let file_id = match FileId::new(id) {
        Ok(value) => value,
        Err(error) => return invalid_file_id_response(error.to_string()),
    };
    let provider = state.tag_provider.clone();
    match tokio::task::spawn_blocking(move || provider.remove_tag(file_id, tag)).await {
        Ok(Ok(outcome)) => tag_mutation_response(outcome),
        Ok(Err(error)) => error.response(),
        Err(error) => HttpTagError::Internal {
            message: format!("tag worker failed: {error}"),
        }
        .response(),
    }
}

fn invalid_file_id_response(message: String) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "invalid_file_id",
                message,
            },
        }),
    )
        .into_response()
}

fn invalid_request_response(message: String) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "invalid_request",
                message,
            },
        }),
    )
        .into_response()
}

fn tag_mutation_response(outcome: HttpTagMutation) -> Response {
    (
        StatusCode::OK,
        Json(TagMutationResponse {
            schema_version: 1,
            changed_count: outcome.changed_count,
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
            message: "metadata store is busy; retry the tag operation".to_owned(),
        },
        TagError::Repository(error) => HttpTagError::Internal {
            message: error.to_string(),
        },
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

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;
    use axum::http::StatusCode;
    use tssp_app::TagError;
    use tssp_domain::{FileId, Tag};
    use tssp_ports::{RepositoryError, TagSummary};

    use super::{
        map_tag_error, tag_mutation_response, FileTagProvider, HttpTagError, HttpTagMutation,
        StaticFileTagProvider, TagListResponse,
    };

    #[test]
    fn static_tag_provider_reports_unavailable() {
        let provider = StaticFileTagProvider;

        let list = provider.list_tags();
        let add = provider.add_tags(file_id("file-1"), vec!["Docs".to_owned()]);
        let remove = provider.remove_tag(file_id("file-1"), "Docs".to_owned());

        assert!(matches!(list, Err(HttpTagError::Unavailable { .. })));
        assert!(matches!(add, Err(HttpTagError::Unavailable { .. })));
        assert!(matches!(remove, Err(HttpTagError::Unavailable { .. })));
    }

    #[tokio::test]
    async fn tag_mutation_response_returns_json_contract() {
        let response = tag_mutation_response(HttpTagMutation { changed_count: 2 });

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024)
            .await
            .unwrap_or_else(|error| panic!("body read failed: {error}"));
        let parsed: serde_json::Value = serde_json::from_slice(&body)
            .unwrap_or_else(|error| panic!("json parse failed: {error}"));
        assert_eq!(parsed["schema_version"], 1);
        assert_eq!(parsed["changed_count"], 2);
    }

    #[test]
    fn tag_list_response_maps_summaries() {
        let response = TagListResponse::from_tags(&[TagSummary {
            tag: tag_value("Docs"),
            file_count: 3,
        }]);

        assert_eq!(response.tags.len(), 1);
        assert_eq!(response.tags[0].name, "Docs");
        assert_eq!(response.tags[0].file_count, 3);
    }

    #[test]
    fn map_tag_error_translates_repository_failures() {
        assert!(matches!(
            map_tag_error(TagError::Repository(RepositoryError::NotFound)),
            HttpTagError::NotFound { .. }
        ));
        assert!(matches!(
            map_tag_error(TagError::Repository(RepositoryError::Busy)),
            HttpTagError::Busy { .. }
        ));
        assert!(matches!(
            map_tag_error(TagError::Repository(RepositoryError::OperationFailed {
                message: "failed".to_owned()
            })),
            HttpTagError::Internal { .. }
        ));
    }

    fn file_id(value: &str) -> FileId {
        FileId::new(value).unwrap_or_else(|error| panic!("invalid file id: {error}"))
    }

    fn tag_value(value: &str) -> Tag {
        Tag::new(value).unwrap_or_else(|error| panic!("invalid tag: {error}"))
    }

    #[tokio::test]
    async fn http_tag_error_response_maps_status_codes() {
        use axum::body::to_bytes;

        let cases = vec![
            (
                HttpTagError::InvalidRequest {
                    message: "bad tag".to_owned(),
                },
                StatusCode::BAD_REQUEST,
                "invalid_request",
            ),
            (
                HttpTagError::NotFound {
                    message: "gone".to_owned(),
                },
                StatusCode::NOT_FOUND,
                "file_not_found",
            ),
            (
                HttpTagError::Busy {
                    message: "busy".to_owned(),
                },
                StatusCode::SERVICE_UNAVAILABLE,
                "metadata_busy",
            ),
            (
                HttpTagError::Unavailable {
                    message: "off".to_owned(),
                },
                StatusCode::SERVICE_UNAVAILABLE,
                "tag_unavailable",
            ),
            (
                HttpTagError::Internal {
                    message: "crash".to_owned(),
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
                .unwrap_or_else(|e| panic!("body read: {e}"));
            let parsed: serde_json::Value =
                serde_json::from_slice(&body).unwrap_or_else(|e| panic!("json parse: {e}"));
            assert_eq!(parsed["error"]["code"], expected_code);
        }
    }

    #[tokio::test]
    async fn invalid_file_id_response_returns_bad_request() {
        use super::invalid_file_id_response;
        use axum::body::to_bytes;

        let response = invalid_file_id_response("bad id".to_owned());
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), 1024)
            .await
            .unwrap_or_else(|e| panic!("body read: {e}"));
        let parsed: serde_json::Value =
            serde_json::from_slice(&body).unwrap_or_else(|e| panic!("json parse: {e}"));
        assert_eq!(parsed["error"]["code"], "invalid_file_id");
    }

    #[tokio::test]
    async fn invalid_request_response_returns_bad_request() {
        use super::invalid_request_response;
        use axum::body::to_bytes;

        let response = invalid_request_response("empty tags".to_owned());
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), 1024)
            .await
            .unwrap_or_else(|e| panic!("body read: {e}"));
        let parsed: serde_json::Value =
            serde_json::from_slice(&body).unwrap_or_else(|e| panic!("json parse: {e}"));
        assert_eq!(parsed["error"]["code"], "invalid_request");
    }

    #[test]
    fn map_tag_error_translates_invalid_request() {
        let err = match tssp_domain::TagKey::new("") {
            Ok(_) => panic!("empty tag key unexpectedly parsed"),
            Err(error) => TagError::InvalidRequest(error),
        };
        assert!(matches!(
            map_tag_error(err),
            HttpTagError::InvalidRequest { .. }
        ));
    }
}
