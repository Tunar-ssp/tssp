//! Admin-only code editor endpoints for workspace management.
//!
//! Admins can view and open any user's workspace. These endpoints are distinct
//! from the regular `/api/v1/workspaces` routes which scope to the caller's own
//! workspaces (unless the caller is already an admin).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

use crate::auth::AuthContext;
use crate::{ErrorBody, ErrorResponse, HttpState};

#[derive(Debug, Serialize)]
struct EditorWorkspaceList {
    schema_version: u8,
    workspaces: Vec<EditorWorkspaceSummary>,
}

#[derive(Debug, Serialize)]
struct EditorWorkspaceSummary {
    id: String,
    owner_id: String,
    name: String,
    language: String,
    updated_at: i64,
}

#[derive(Debug, Serialize)]
struct ExecutionCheckResponse {
    execution_disabled: bool,
    message: &'static str,
}

fn forbidden() -> axum::response::Response {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "forbidden",
                message: "admin role required".to_owned(),
            },
        }),
    )
        .into_response()
}

fn internal_error(message: String) -> axum::response::Response {
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

/// `GET /api/v1/admin/editor/workspaces`
///
/// Lists all workspaces across all users. Admin-only.
pub(crate) async fn admin_editor_list_workspaces(
    State(state): State<HttpState>,
    auth: AuthContext,
) -> impl IntoResponse {
    if !auth.is_admin() {
        return forbidden();
    }
    let Some(store) = state.workspaces.as_deref() else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    match store.list_all() {
        Ok(items) => {
            let summaries = items
                .into_iter()
                .map(|w| EditorWorkspaceSummary {
                    id: w.id,
                    owner_id: w.owner_id,
                    name: w.name,
                    language: w.language,
                    updated_at: w.updated_at,
                })
                .collect();
            (
                StatusCode::OK,
                Json(EditorWorkspaceList {
                    schema_version: 1,
                    workspaces: summaries,
                }),
            )
                .into_response()
        }
        Err(error) => internal_error(error.to_string()),
    }
}

/// `GET /api/v1/admin/editor/workspaces/{id}`
///
/// Returns any workspace by id. Admin-only.
pub(crate) async fn admin_editor_get_workspace(
    State(state): State<HttpState>,
    auth: AuthContext,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if !auth.is_admin() {
        return forbidden();
    }
    let Some(store) = state.workspaces.as_deref() else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    match store.get(&id, None) {
        Ok(record) => (StatusCode::OK, Json(record)).into_response(),
        Err(crate::workspaces::WorkspaceError::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "not_found",
                    message: "workspace not found".to_owned(),
                },
            }),
        )
            .into_response(),
        Err(error) => internal_error(error.to_string()),
    }
}

/// `POST /api/v1/admin/editor/check`
///
/// Safe stub: returns a fixed message indicating execution is disabled.
/// This endpoint exists so the frontend can surface a clear "not available"
/// message rather than guessing.
pub(crate) async fn admin_editor_check(auth: AuthContext) -> impl IntoResponse {
    if !auth.is_admin() {
        return forbidden();
    }
    (
        StatusCode::OK,
        Json(ExecutionCheckResponse {
            execution_disabled: true,
            message: "Script execution requires a sandboxed runtime not yet available",
        }),
    )
        .into_response()
}
