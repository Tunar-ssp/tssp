//! Axum handlers for note endpoints.

use axum::extract::{Path, Query, State};
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use tssp_app::{CreateNoteRequest, UpdateNoteRequest};

use crate::HttpState;

use super::error::HttpNoteError;
use super::provider::NoteProvider;
use super::query::{build_list_query, ListNotesQuery};
use super::response::{NoteListResponse, NoteRecordResponse};
use super::validate::{parse_note_id, validate_note_body};

#[derive(Debug, Deserialize)]
pub(crate) struct CreateNoteBody {
    pub(crate) title: Option<String>,
    pub(crate) body: String,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    #[serde(default)]
    pub(crate) pin: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateNoteBody {
    pub(crate) title: Option<String>,
    pub(crate) body: String,
}

pub(crate) async fn create_note(
    State(state): State<HttpState>,
    Json(body): Json<CreateNoteBody>,
) -> Response {
    if let Err(error) = validate_note_body(&body.body) {
        return error.response();
    }

    let provider = state.note_provider.clone();
    let request = CreateNoteRequest {
        title: body.title,
        body: body.body,
        tags: body.tags,
        pin: body.pin,
    };

    match run_blocking(provider, move |provider| provider.create_note(request)).await {
        Ok(record) => (
            StatusCode::CREATED,
            Json(NoteRecordResponse::from_record(&record)),
        )
            .into_response(),
        Err(response) => response,
    }
}

pub(crate) async fn list_notes(
    State(state): State<HttpState>,
    Query(params): Query<ListNotesQuery>,
) -> Response {
    let query = match build_list_query(&params) {
        Ok(value) => value,
        Err(message) => {
            return HttpNoteError::InvalidRequest { message }.response();
        }
    };
    let provider = state.note_provider.clone();

    match run_blocking(provider, move |provider| provider.list_notes(query)).await {
        Ok(page) => {
            let response = NoteListResponse {
                schema_version: 1,
                notes: page
                    .notes
                    .iter()
                    .map(NoteRecordResponse::from_record)
                    .collect(),
                next_cursor: page.next_cursor.map(|cursor| cursor.as_str().to_owned()),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(response) => response,
    }
}

pub(crate) async fn get_note(State(state): State<HttpState>, Path(id): Path<String>) -> Response {
    let note_id = match parse_note_id(id) {
        Ok(value) => value,
        Err(error) => return error.response(),
    };
    let provider = state.note_provider.clone();

    match run_blocking(provider, move |provider| provider.get_note(note_id)).await {
        Ok(record) => (
            StatusCode::OK,
            Json(NoteRecordResponse::from_record(&record)),
        )
            .into_response(),
        Err(response) => response,
    }
}

pub(crate) async fn update_note(
    State(state): State<HttpState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateNoteBody>,
) -> Response {
    if let Err(error) = validate_note_body(&body.body) {
        return error.response();
    }

    let note_id = match parse_note_id(id) {
        Ok(value) => value,
        Err(error) => return error.response(),
    };
    let provider = state.note_provider.clone();
    let request = UpdateNoteRequest {
        title: body.title,
        body: body.body,
    };

    match run_blocking(provider, move |provider| {
        provider.update_note(note_id, request)
    })
    .await
    {
        Ok(record) => (
            StatusCode::OK,
            Json(NoteRecordResponse::from_record(&record)),
        )
            .into_response(),
        Err(response) => response,
    }
}

pub(crate) async fn delete_note(
    State(state): State<HttpState>,
    Path(id): Path<String>,
) -> Response {
    let note_id = match parse_note_id(id) {
        Ok(value) => value,
        Err(error) => return error.response(),
    };
    let provider = state.note_provider.clone();

    match run_blocking(provider, move |provider| provider.delete_note(note_id)).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(response) => response,
    }
}

pub(crate) async fn add_note_tags(
    State(state): State<HttpState>,
    Path(id): Path<String>,
    Json(tags): Json<Vec<String>>,
) -> Response {
    if tags.is_empty() {
        return HttpNoteError::InvalidRequest {
            message: "request body must contain at least one tag".to_owned(),
        }
        .response();
    }
    let note_id = match parse_note_id(id) {
        Ok(value) => value,
        Err(error) => return error.response(),
    };
    let provider = state.note_provider.clone();

    match run_blocking(provider, move |provider| provider.add_tags(note_id, tags)).await {
        Ok(_changed) => StatusCode::NO_CONTENT.into_response(),
        Err(response) => response,
    }
}

pub(crate) async fn replace_note_tags(
    State(state): State<HttpState>,
    Path(id): Path<String>,
    Json(tags): Json<Vec<String>>,
) -> Response {
    let note_id = match parse_note_id(id) {
        Ok(value) => value,
        Err(error) => return error.response(),
    };
    let provider = state.note_provider.clone();

    match run_blocking(provider, move |provider| provider.replace_tags(note_id, tags)).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(response) => response,
    }
}

pub(crate) async fn remove_note_tag(
    State(state): State<HttpState>,
    Path((id, tag)): Path<(String, String)>,
) -> Response {
    let note_id = match parse_note_id(id) {
        Ok(value) => value,
        Err(error) => return error.response(),
    };
    let provider = state.note_provider.clone();

    match run_blocking(provider, move |provider| provider.remove_tag(note_id, tag)).await {
        Ok(_changed) => StatusCode::NO_CONTENT.into_response(),
        Err(response) => response,
    }
}

pub(crate) async fn pin_note(State(state): State<HttpState>, Path(id): Path<String>) -> Response {
    pin_with_position(state, id, None).await
}

pub(crate) async fn unpin_note(State(state): State<HttpState>, Path(id): Path<String>) -> Response {
    let note_id = match parse_note_id(id) {
        Ok(value) => value,
        Err(error) => return error.response(),
    };
    let provider = state.note_provider.clone();

    match run_blocking(provider, move |provider| provider.unpin_note(note_id)).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(response) => response,
    }
}

async fn pin_with_position(state: HttpState, id: String, position: Option<u32>) -> Response {
    let note_id = match parse_note_id(id) {
        Ok(value) => value,
        Err(error) => return error.response(),
    };
    let provider = state.note_provider.clone();

    match run_blocking(provider, move |provider| {
        provider.pin_note(note_id, position)
    })
    .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(response) => response,
    }
}

/// `GET /api/v1/notes/export` — download all notes as newline-delimited JSON.
pub(crate) async fn export_notes(State(state): State<HttpState>) -> Response {
    let query = tssp_ports::NoteListQuery {
        limit: 10_000,
        ..tssp_ports::NoteListQuery::default()
    };
    let provider = state.note_provider.clone();
    match run_blocking(provider, move |provider| provider.list_notes(query)).await {
        Ok(page) => {
            let mut body = String::new();
            for note in &page.notes {
                if let Ok(line) = serde_json::to_string(&NoteRecordResponse::from_record(note)) {
                    body.push_str(&line);
                    body.push('\n');
                }
            }
            (
                StatusCode::OK,
                [
                    (CONTENT_TYPE, "application/x-ndjson; charset=utf-8"),
                    (CONTENT_DISPOSITION, "attachment; filename=\"notes-export.ndjson\""),
                ],
                body,
            )
                .into_response()
        }
        Err(response) => response,
    }
}

async fn run_blocking<T, F>(
    provider: std::sync::Arc<dyn NoteProvider>,
    work: F,
) -> Result<T, Response>
where
    F: FnOnce(std::sync::Arc<dyn NoteProvider>) -> Result<T, HttpNoteError> + Send + 'static,
    T: Send + 'static,
{
    match tokio::task::spawn_blocking(move || work(provider)).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => Err(error.response()),
        Err(error) => Err(HttpNoteError::Internal {
            message: format!("note worker failed: {error}"),
        }
        .response()),
    }
}
