//! File visibility (public/private) and bulk updates.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use getrandom::getrandom;
use serde::{Deserialize, Serialize};
use tssp_domain::{FileId, Visibility};

use crate::auth::AuthContext;
use crate::upload::FileRecordResponse;
use crate::{ErrorBody, ErrorResponse, HttpState};

fn new_public_token() -> Result<String, String> {
    let mut bytes = [0_u8; 24];
    getrandom(&mut bytes).map_err(|error| format!("failed to generate public token: {error}"))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn forbidden() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "forbidden",
                message: "you do not have permission to change this file".to_owned(),
            },
        }),
    )
        .into_response()
}

fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "not_found",
                message: "file not found".to_owned(),
            },
        }),
    )
        .into_response()
}

/// Body for single-file visibility change.
#[derive(Debug, Deserialize)]
pub struct VisibilityBody {
    /// `public` or `private`.
    pub visibility: String,
}

/// Body for bulk visibility change.
#[derive(Debug, Deserialize)]
pub struct BulkVisibilityBody {
    /// File ids to update.
    pub ids: Vec<String>,
    /// `public` or `private`.
    pub visibility: String,
}

/// Response for visibility mutations.
#[derive(Debug, Serialize)]
pub struct VisibilityResponse {
    pub schema_version: u8,
    pub file: FileRecordResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_url: Option<String>,
}

/// `PATCH /api/v1/files/{id}/visibility`
pub async fn patch_file_visibility(
    State(state): State<HttpState>,
    auth: AuthContext,
    Path(id): Path<String>,
    Json(body): Json<VisibilityBody>,
) -> Response {
    let file_id = match FileId::new(id) {
        Ok(value) => value,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "invalid_file_id",
                        message: error.to_string(),
                    },
                }),
            )
                .into_response();
        }
    };
    let visibility = match body.visibility.to_ascii_lowercase().as_str() {
        "public" => Visibility::Public,
        "private" => Visibility::Private,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "invalid_visibility",
                        message: "visibility must be public or private".to_owned(),
                    },
                }),
            )
                .into_response();
        }
    };

    let existing = match state.stats_provider.find_file(&file_id) {
        Ok(Some(file)) => file,
        Ok(None) => return not_found(),
        Err(message) => return internal(message),
    };
    if !auth.can_manage_file(&existing) {
        return forbidden();
    }

    let (public_token, public_url) = match visibility {
        Visibility::Public => {
            let token = match existing.public_token.clone() {
                Some(token) => token,
                None => match new_public_token() {
                    Ok(token) => token,
                    Err(message) => return internal(message),
                },
            };
            let url = Some(state.public_urls().public_file_url(&token));
            (Some(token), url)
        }
        Visibility::Private => (None, None),
    };

    match state
        .stats_provider
        .set_file_visibility(&file_id, visibility, public_token.as_deref())
    {
        Ok(Some(file)) => (
            StatusCode::OK,
            Json(VisibilityResponse {
                schema_version: 1,
                file: FileRecordResponse::from_record(&file),
                public_url,
            }),
        )
            .into_response(),
        Ok(None) => not_found(),
        Err(message) => internal(message),
    }
}

/// `POST /api/v1/files/visibility/bulk`
pub async fn bulk_file_visibility(
    State(state): State<HttpState>,
    auth: AuthContext,
    Json(body): Json<BulkVisibilityBody>,
) -> Response {
    if body.ids.is_empty() || body.ids.len() > 200 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "invalid_bulk",
                    message: "ids must contain between 1 and 200 file ids".to_owned(),
                },
            }),
        )
            .into_response();
    }
    let visibility = match body.visibility.to_ascii_lowercase().as_str() {
        "public" => Visibility::Public,
        "private" => Visibility::Private,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: ErrorBody {
                        code: "invalid_visibility",
                        message: "visibility must be public or private".to_owned(),
                    },
                }),
            )
                .into_response();
        }
    };

    let mut updated = Vec::new();
    for id_str in &body.ids {
        let Ok(file_id) = FileId::new(id_str) else {
            continue;
        };
        let Some(existing) = state.stats_provider.find_file(&file_id).ok().flatten() else {
            continue;
        };
        if !auth.can_manage_file(&existing) {
            continue;
        }
        let token = match visibility {
            Visibility::Public => match existing.public_token.clone() {
                Some(token) => Some(token),
                None => match new_public_token() {
                    Ok(token) => Some(token),
                    Err(message) => return internal(message),
                },
            },
            Visibility::Private => None,
        };
        if let Ok(Some(file)) =
            state
                .stats_provider
                .set_file_visibility(&file_id, visibility, token.as_deref())
        {
            updated.push(FileRecordResponse::from_record(&file));
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "schema_version": 1,
            "updated": updated,
            "count": updated.len(),
        })),
    )
        .into_response()
}

fn internal(message: String) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "internal_error",
                message,
            },
        }),
    )
        .into_response()
}
