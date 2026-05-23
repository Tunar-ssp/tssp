//! PATCH /api/v1/files/{id} handler for renaming files.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tssp_domain::FileName;

use crate::HttpState;

#[derive(Debug, Deserialize)]
pub struct RenameRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct RenameResponse {
    pub schema_version: u8,
    pub id: String,
    pub file: Value,
}

pub async fn rename_file(
    State(state): State<HttpState>,
    Path(id): Path<String>,
    Json(request): Json<RenameRequest>,
) -> Result<(StatusCode, Json<RenameResponse>), (StatusCode, Json<Value>)> {
    let file_id = match tssp_domain::FileId::new(&id) {
        Ok(fid) => fid,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "schema_version": 1,
                    "error": {
                        "code": "invalid_request",
                        "message": "invalid file id"
                    }
                })),
            ))
        }
    };

    let new_name = match FileName::new(&request.name) {
        Ok(fname) => fname,
        Err(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "schema_version": 1,
                    "error": {
                        "code": "invalid_request",
                        "message": "invalid filename"
                    }
                })),
            ))
        }
    };

    match state.stats_provider.rename_file(&file_id, &new_name) {
        Ok(Some(record)) => {
            let file_json = json!({
                "id": record.id.as_str(),
                "name": record.name.original(),
                "size": record.size.bytes(),
                "mime": record.mime_type.as_str(),
                "tags": record.tags.iter().map(|t| t.display()).collect::<Vec<_>>(),
                "uploaded": record.uploaded_at.seconds(),
                "pinned": record.pinned_at.is_some(),
            });

            Ok((
                StatusCode::OK,
                Json(RenameResponse {
                    schema_version: 1,
                    id: id.clone(),
                    file: file_json,
                }),
            ))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "schema_version": 1,
                "error": {
                    "code": "not_found",
                    "message": "file not found"
                }
            })),
        )),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "schema_version": 1,
                "error": {
                    "code": "internal_error",
                    "message": "rename failed"
                }
            })),
        )),
    }
}

