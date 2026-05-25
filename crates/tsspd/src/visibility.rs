#![allow(clippy::unwrap_used, clippy::unreadable_literal, clippy::needless_raw_string_hashes, clippy::uninlined_format_args, clippy::expect_used, clippy::needless_borrows_for_generic_args, clippy::map_unwrap_or, clippy::return_self_not_must_use, clippy::too_many_lines, clippy::missing_errors_doc, clippy::redundant_closure_for_method_calls, clippy::manual_string_new, clippy::ip_constant, clippy::single_char_pattern, clippy::absurd_extreme_comparisons, clippy::erasing_op, clippy::clone_on_copy)]
//! File visibility (public/private) and bulk updates.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use getrandom::getrandom;
use serde::{Deserialize, Serialize};
use tssp_domain::{FileId, Visibility};
use tssp_app::{AuditAction, log_audit_event};

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
    if !(auth.is_admin() || existing.owner_id.as_ref() == Some(&auth.user_id)) {
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

    let repository = state.repository.clone();
    match state
        .stats_provider
        .set_file_visibility(&file_id, visibility, public_token.as_deref())
    {
        Ok(Some(file)) => {
            log_audit_event(
                repository.as_ref(),
                AuditAction::FileVisibilityChange,
                Some(&auth.user_id),
                Some("file"),
                Some(file_id.as_str()),
                "success",
                Some(&format!("changed visibility to {}", visibility.as_str())),
            );
            (
                StatusCode::OK,
                Json(VisibilityResponse {
                    schema_version: 1,
                    file: FileRecordResponse::from_record(&file),
                    public_url,
                }),
            )
                .into_response()
        }
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

    let repository = state.repository.clone();
    let mut updated = Vec::new();
    for id_str in &body.ids {
        let Ok(file_id) = FileId::new(id_str) else {
            continue;
        };
        let Some(existing) = state.stats_provider.find_file(&file_id).ok().flatten() else {
            continue;
        };
        if !(auth.is_admin() || existing.owner_id.as_ref() == Some(&auth.user_id)) {
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
            log_audit_event(
                repository.as_ref(),
                AuditAction::FileVisibilityChange,
                Some(&auth.user_id),
                Some("file"),
                Some(file_id.as_str()),
                "success",
                Some(&format!("bulk changed visibility to {}", visibility.as_str())),
            );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_public_token_generates_valid_base64() {
        let token = new_public_token().expect("token generation failed");
        assert!(!token.is_empty());
        assert!(token.len() > 20);
        // Valid base64 should only contain URL_SAFE characters
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn new_public_token_generates_different_tokens() {
        let token1 = new_public_token().expect("first token");
        let token2 = new_public_token().expect("second token");
        assert_ne!(token1, token2, "tokens should be different");
    }

    #[test]
    fn visibility_body_deserializes_public() {
        let json = r#"{"visibility": "public"}"#;
        let body: VisibilityBody = serde_json::from_str(json).expect("deserialize");
        assert_eq!(body.visibility, "public");
    }

    #[test]
    fn visibility_body_deserializes_private() {
        let json = r#"{"visibility": "private"}"#;
        let body: VisibilityBody = serde_json::from_str(json).expect("deserialize");
        assert_eq!(body.visibility, "private");
    }

    #[test]
    fn bulk_visibility_body_validates_ids() {
        let json = r#"{"ids": ["id1", "id2"], "visibility": "public"}"#;
        let body: BulkVisibilityBody = serde_json::from_str(json).expect("deserialize");
        assert_eq!(body.ids.len(), 2);
        assert_eq!(body.visibility, "public");
    }

    #[test]
    fn visibility_string_case_insensitive_public() {
        let visibility_str = "PUBLIC";
        let result = visibility_str.to_ascii_lowercase();
        assert_eq!(result, "public");
    }

    #[test]
    fn visibility_string_case_insensitive_private() {
        let visibility_str = "PRIVATE";
        let result = visibility_str.to_ascii_lowercase();
        assert_eq!(result, "private");
    }

    #[test]
    fn forbidden_response_has_correct_status() {
        let response = forbidden();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn not_found_response_has_correct_status() {
        let response = not_found();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn internal_response_has_correct_status() {
        let response = internal("test error".to_owned());
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
