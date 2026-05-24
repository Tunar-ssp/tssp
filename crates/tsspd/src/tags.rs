//! Tag HTTP delivery.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use tssp_app::TagService;
use tssp_domain::FileId;
use tssp_ports::{FileRepository, TagSummary};

use crate::http_error::ApiError;
use crate::HttpState;

pub(crate) async fn list_tags<R>(State(state): State<HttpState<R>>) -> Result<Response, ApiError>
where
    R: FileRepository + Send + Sync + 'static,
{
    let service = state.tag_service.clone();
    let tags = tokio::task::spawn_blocking(move || service.list_tags()).await
        .map_err(|e| ApiError::Internal(format!("tag worker failed: {e}")))?
        .map_err(ApiError::from)?;

    Ok((StatusCode::OK, Json(TagListResponse::from_tags(&tags))).into_response())
}

pub(crate) async fn add_tags<R>(
    State(state): State<HttpState<R>>,
    Path(id): Path<String>,
    Json(tags): Json<Vec<String>>,
) -> Result<Response, ApiError>
where
    R: FileRepository + Send + Sync + 'static,
{
    if tags.is_empty() {
        return Err(ApiError::BadRequest("request body must contain at least one tag".to_owned()));
    }
    let file_id = FileId::new(id).map_err(ApiError::from)?;
    let service = state.tag_service.clone();
    
    let outcome = tokio::task::spawn_blocking(move || {
        let tag_refs = tags.iter().map(String::as_str).collect::<Vec<_>>();
        service.add_tags(&file_id, &tag_refs)
    }).await
    .map_err(|e| ApiError::Internal(format!("tag worker failed: {e}")))?
    .map_err(ApiError::from)?;

    Ok(tag_mutation_response(outcome.changed_count))
}

pub(crate) async fn remove_tag<R>(
    State(state): State<HttpState<R>>,
    Path((id, tag)): Path<(String, String)>,
) -> Result<Response, ApiError>
where
    R: FileRepository + Send + Sync + 'static,
{
    let file_id = FileId::new(id).map_err(ApiError::from)?;
    let service = state.tag_service.clone();
    
    let outcome = tokio::task::spawn_blocking(move || {
        service.remove_tag(&file_id, &tag)
    }).await
    .map_err(|e| ApiError::Internal(format!("tag worker failed: {e}")))?
    .map_err(ApiError::from)?;

    Ok(tag_mutation_response(outcome.changed_count))
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
