//! Admin-only code editor endpoints for workspace management.
//!
//! Admins can inspect and edit any user's workspace and its stored documents.
//! The editor is intentionally limited to structured document operations: no
//! shell access, no filesystem paths, and no code execution.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use tssp_ports::Clock;

use tssp_app::{log_audit_event, AuditAction};

use crate::auth::AuthContext;
use crate::workspaces::{WorkspaceDocumentSummary, WorkspaceError};
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
struct EditorWorkspaceDetail {
    schema_version: u8,
    workspace: crate::workspaces::WorkspaceRecord,
    documents: Vec<WorkspaceDocumentSummary>,
}

#[derive(Debug, Serialize)]
struct EditorDocumentList {
    schema_version: u8,
    documents: Vec<WorkspaceDocumentSummary>,
}

#[derive(Debug, Serialize)]
struct ExecutionCheckResponse {
    execution_disabled: bool,
    message: &'static str,
}

#[derive(Debug, Serialize)]
struct EditorDocument {
    id: String,
    workspace_id: String,
    owner_id: String,
    path: String,
    language: String,
    body: String,
    is_primary: bool,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct EditorDocumentBody {
    path: String,
    #[serde(default = "default_language")]
    language: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    make_primary: bool,
}

fn default_language() -> String {
    "text".to_owned()
}

fn forbidden() -> Response {
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

fn unavailable() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "workspaces_unavailable",
                message: "workspace store is not configured".to_owned(),
            },
        }),
    )
        .into_response()
}

fn error_response(code: StatusCode, error_code: &'static str, message: String) -> Response {
    (
        code,
        Json(ErrorResponse {
            error: ErrorBody {
                code: error_code,
                message,
            },
        }),
    )
        .into_response()
}

fn editor_error(error: WorkspaceError, conflict_message: &'static str) -> Response {
    match error {
        WorkspaceError::NotFound => error_response(
            StatusCode::NOT_FOUND,
            "not_found",
            "workspace or document not found".to_owned(),
        ),
        WorkspaceError::Conflict => error_response(
            StatusCode::CONFLICT,
            "workspace_conflict",
            conflict_message.to_owned(),
        ),
        WorkspaceError::InvalidOperation(message) => {
            error_response(StatusCode::BAD_REQUEST, "invalid_request", message)
        }
        WorkspaceError::Database(error) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            error.to_string(),
        ),
    }
}

fn now_seconds() -> i64 {
    tssp_adapter_system::SystemClock.now().seconds()
}

/// `GET /api/v1/admin/editor/workspaces`
pub(crate) async fn admin_editor_list_workspaces(
    State(state): State<HttpState>,
    auth: AuthContext,
) -> impl IntoResponse {
    if !auth.is_admin() {
        return forbidden();
    }
    let Some(store) = state.workspaces.as_deref() else {
        return unavailable();
    };
    match store.list_all() {
        Ok(items) => {
            let summaries = items
                .into_iter()
                .map(|workspace| EditorWorkspaceSummary {
                    id: workspace.id,
                    owner_id: workspace.owner_id,
                    name: workspace.name,
                    language: workspace.language,
                    updated_at: workspace.updated_at,
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
        Err(error) => editor_error(error, "workspace list could not be loaded"),
    }
}

/// `GET /api/v1/admin/editor/workspaces/{id}`
pub(crate) async fn admin_editor_get_workspace(
    State(state): State<HttpState>,
    auth: AuthContext,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if !auth.is_admin() {
        return forbidden();
    }
    let Some(store) = state.workspaces.as_deref() else {
        return unavailable();
    };
    match (store.get(&id, None), store.list_documents(&id, None)) {
        (Ok(workspace), Ok(documents)) => (
            StatusCode::OK,
            Json(EditorWorkspaceDetail {
                schema_version: 1,
                workspace,
                documents,
            }),
        )
            .into_response(),
        (Err(error), _) | (_, Err(error)) => {
            editor_error(error, "workspace detail could not be loaded")
        }
    }
}

/// `GET /api/v1/admin/editor/workspaces/{id}/documents`
pub(crate) async fn admin_editor_list_documents(
    State(state): State<HttpState>,
    auth: AuthContext,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if !auth.is_admin() {
        return forbidden();
    }
    let Some(store) = state.workspaces.as_deref() else {
        return unavailable();
    };
    match store.list_documents(&id, None) {
        Ok(documents) => (
            StatusCode::OK,
            Json(EditorDocumentList {
                schema_version: 1,
                documents,
            }),
        )
            .into_response(),
        Err(error) => editor_error(error, "workspace documents could not be loaded"),
    }
}

/// `POST /api/v1/admin/editor/workspaces/{id}/documents`
#[allow(clippy::too_many_lines)]
pub(crate) async fn admin_editor_create_document(
    State(state): State<HttpState>,
    auth: AuthContext,
    Path(workspace_id): Path<String>,
    Json(body): Json<EditorDocumentBody>,
) -> impl IntoResponse {
    if !auth.is_admin() {
        return forbidden();
    }
    let Some(store) = state.workspaces.as_deref() else {
        return unavailable();
    };
    if let Err(e) = state
        .workspace_file_service
        .init_workspace(&workspace_id)
        .await
    {
        log_audit_event(
            state.repository.as_ref(),
            AuditAction::NoteUpdate,
            Some(&auth.user_id),
            Some("workspace_document"),
            None,
            "failure",
            Some(&format!(
                "admin failed to initialize workspace {workspace_id}: {e:?}"
            )),
        );
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "workspace_init_failed",
            format!("failed to initialize workspace: {e:?}"),
        );
    }

    match store.create_document(
        &workspace_id,
        None,
        &body.path,
        &body.language,
        body.body.clone(),
        body.make_primary,
        now_seconds(),
    ) {
        Ok(document) => {
            if let Err(e) = state
                .workspace_file_service
                .write_file(&workspace_id, &document.path, body.body.as_bytes())
                .await
            {
                log_audit_event(
                    state.repository.as_ref(),
                    AuditAction::NoteUpdate,
                    Some(&auth.user_id),
                    Some("workspace_document"),
                    Some(&document.id),
                    "failure",
                    Some(&format!(
                        "admin created document metadata but failed to write body: {e:?}"
                    )),
                );
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "file_write_failed",
                    format!("failed to write document body: {e:?}"),
                );
            }
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::NoteUpdate, // Workspace document create
                Some(&auth.user_id),
                Some("workspace_document"),
                Some(&document.id),
                "success",
                Some(&format!(
                    "admin created document {path} in workspace {workspace_id}",
                    path = body.path
                )),
            );
            let response = EditorDocument {
                id: document.id,
                workspace_id: document.workspace_id,
                owner_id: document.owner_id,
                path: document.path,
                language: document.language,
                body: body.body,
                is_primary: document.is_primary,
                created_at: document.created_at,
                updated_at: document.updated_at,
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(error) => {
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::NoteUpdate,
                Some(&auth.user_id),
                Some("workspace_document"),
                None,
                "failure",
                Some(&format!(
                    "admin failed to create document {path} in workspace {workspace_id}: {error:?}",
                    path = body.path
                )),
            );
            editor_error(
                error,
                "a document with this path already exists in the workspace",
            )
        }
    }
}

/// `GET /api/v1/admin/editor/workspaces/{id}/documents/{document_id}`
pub(crate) async fn admin_editor_get_document(
    State(state): State<HttpState>,
    auth: AuthContext,
    Path((workspace_id, document_id)): Path<(String, String)>,
) -> impl IntoResponse {
    if !auth.is_admin() {
        return forbidden();
    }
    let Some(store) = state.workspaces.as_deref() else {
        return unavailable();
    };
    match store.get_document(&workspace_id, &document_id, None) {
        Ok(document) => {
            match state
                .workspace_file_service
                .read_file(&workspace_id, &document.path)
                .await
            {
                Ok(bytes) => {
                    let Ok(body) = String::from_utf8(bytes) else {
                        return error_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "invalid_utf8",
                            "document body is not valid UTF-8".to_owned(),
                        );
                    };
                    let response = EditorDocument {
                        id: document.id,
                        workspace_id: document.workspace_id,
                        owner_id: document.owner_id,
                        path: document.path,
                        language: document.language,
                        body,
                        is_primary: document.is_primary,
                        created_at: document.created_at,
                        updated_at: document.updated_at,
                    };
                    (StatusCode::OK, Json(response)).into_response()
                }
                Err(_) => error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "file_not_found",
                    format!("could not read document file at path: {}", document.path),
                ),
            }
        }
        Err(error) => editor_error(error, "workspace document could not be loaded"),
    }
}

/// `PUT /api/v1/admin/editor/workspaces/{id}/documents/{document_id}`
#[allow(clippy::too_many_lines)]
pub(crate) async fn admin_editor_update_document(
    State(state): State<HttpState>,
    auth: AuthContext,
    Path((workspace_id, document_id)): Path<(String, String)>,
    Json(body): Json<EditorDocumentBody>,
) -> impl IntoResponse {
    if !auth.is_admin() {
        return forbidden();
    }
    let Some(store) = state.workspaces.as_deref() else {
        return unavailable();
    };

    if let Err(e) = state
        .workspace_file_service
        .init_workspace(&workspace_id)
        .await
    {
        log_audit_event(
            state.repository.as_ref(),
            AuditAction::NoteUpdate,
            Some(&auth.user_id),
            Some("workspace_document"),
            None,
            "failure",
            Some(&format!(
                "admin failed to initialize workspace {workspace_id}: {e:?}"
            )),
        );
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "workspace_init_failed",
            format!("failed to initialize workspace: {e:?}"),
        );
    }

    match store.update_document(
        &workspace_id,
        &document_id,
        None,
        &body.path,
        &body.language,
        body.body.clone(),
        body.make_primary,
        now_seconds(),
    ) {
        Ok(document) => {
            if let Err(e) = state
                .workspace_file_service
                .write_file(&workspace_id, &document.path, body.body.as_bytes())
                .await
            {
                log_audit_event(
                    state.repository.as_ref(),
                    AuditAction::NoteUpdate,
                    Some(&auth.user_id),
                    Some("workspace_document"),
                    Some(&document.id),
                    "failure",
                    Some(&format!(
                        "admin updated document metadata but failed to write body: {e:?}"
                    )),
                );
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "file_write_failed",
                    format!("failed to write document body: {e:?}"),
                );
            }
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::NoteUpdate,
                Some(&auth.user_id),
                Some("workspace_document"),
                Some(&document.id),
                "success",
                Some(&format!(
                    "admin updated document {path} in workspace {workspace_id}",
                    path = body.path
                )),
            );
            let response = EditorDocument {
                id: document.id,
                workspace_id: document.workspace_id,
                owner_id: document.owner_id,
                path: document.path,
                language: document.language,
                body: body.body,
                is_primary: document.is_primary,
                created_at: document.created_at,
                updated_at: document.updated_at,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(error) => {
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::NoteUpdate,
                Some(&auth.user_id),
                Some("workspace_document"),
                Some(&document_id),
                "failure",
                Some(&format!(
                    "admin failed to update document {path} in workspace {workspace_id}: {error:?}",
                    path = body.path
                )),
            );
            editor_error(
                error,
                "a document with this path already exists in the workspace",
            )
        }
    }
}

/// `DELETE /api/v1/admin/editor/workspaces/{id}/documents/{document_id}`
pub(crate) async fn admin_editor_delete_document(
    State(state): State<HttpState>,
    auth: AuthContext,
    Path((workspace_id, document_id)): Path<(String, String)>,
) -> impl IntoResponse {
    if !auth.is_admin() {
        return forbidden();
    }
    let Some(store) = state.workspaces.as_deref() else {
        return unavailable();
    };
    match store.delete_document(&workspace_id, &document_id, None, now_seconds()) {
        Ok(()) => {
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::NoteDelete,
                Some(&auth.user_id),
                Some("workspace_document"),
                Some(&document_id),
                "success",
                Some(&format!(
                    "admin deleted document {document_id} from workspace {workspace_id}"
                )),
            );
            StatusCode::NO_CONTENT.into_response()
        }
        Err(error) => {
            log_audit_event(
                state.repository.as_ref(),
                AuditAction::NoteDelete,
                Some(&auth.user_id),
                Some("workspace_document"),
                Some(&document_id),
                "failure",
                Some(&format!(
                    "admin failed to delete document {document_id} from workspace {workspace_id}: {error:?}"
                )),
            );
            editor_error(error, "workspace document could not be deleted")
        }
    }
}

/// `POST /api/v1/admin/editor/check`
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::sync::Arc;

    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use axum::middleware;
    use axum::routing::{get, post};
    use axum::Router;
    use serde_json::json;
    use tempfile::TempDir;
    use tower::ServiceExt;
    use tssp_adapter_fs::FilesystemWorkspaceFileStore;
    use tssp_adapter_sqlite::SqliteFileRepository;
    use tssp_app::WorkspaceFileService;
    use tssp_domain::{UserId, UserRole};

    use super::{
        admin_editor_create_document, admin_editor_get_workspace, admin_editor_list_workspaces,
    };
    use crate::auth::AuthContext;
    use crate::workspaces::{
        create_workspace, delete_workspace, get_workspace, update_workspace, WorkspaceStore,
    };
    use crate::{HttpState, PublicUrlBuilder};

    fn auth(user: &str, role: UserRole) -> AuthContext {
        AuthContext {
            user_id: UserId::new(user).expect("valid user"),
            role,
            session_token: Some("session".to_owned()),
            device_token: None,
        }
    }

    fn open_store() -> (TempDir, WorkspaceStore) {
        let temp = tempfile::tempdir().expect("tempdir");
        let db = temp.path().join("metadata.sqlite3");
        let _repository = SqliteFileRepository::open(&db).expect("repository");
        let store = WorkspaceStore::open(&db).expect("workspace store");
        (temp, store)
    }

    fn app(store: WorkspaceStore, auth: AuthContext) -> Router {
        let settings = Arc::new(crate::DaemonSettings::default());
        let temp_dir = std::path::PathBuf::from("/tmp");
        let workspace_root = temp_dir.join("workspaces");
        let _ = std::fs::create_dir_all(&workspace_root);
        let fs_store = Arc::new(FilesystemWorkspaceFileStore::new(workspace_root));
        let workspace_file_service = Arc::new(WorkspaceFileService::new(fs_store));

        let state = HttpState::new(
            std::time::Instant::now(),
            temp_dir,
            settings.clone(),
            PublicUrlBuilder::from_settings(&settings),
            0,
        )
        .with_workspaces(Arc::new(store))
        .with_workspace_file_service(workspace_file_service);
        Router::new()
            .route("/api/v1/workspaces", post(create_workspace))
            .route(
                "/api/v1/workspaces/{id}",
                get(get_workspace)
                    .put(update_workspace)
                    .delete(delete_workspace),
            )
            .route(
                "/api/v1/admin/editor/workspaces",
                get(admin_editor_list_workspaces),
            )
            .route(
                "/api/v1/admin/editor/workspaces/{id}",
                get(admin_editor_get_workspace),
            )
            .route(
                "/api/v1/admin/editor/workspaces/{id}/documents",
                post(admin_editor_create_document),
            )
            .layer(middleware::from_fn(
                move |mut request: axum::extract::Request, next: axum::middleware::Next| {
                    let auth = auth.clone();
                    async move {
                        request.extensions_mut().insert(auth);
                        next.run(request).await
                    }
                },
            ))
            .with_state(state)
    }

    async fn request_json(
        router: &Router,
        method: &str,
        uri: &str,
        body: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .expect("request");
        let response = router.clone().oneshot(request).await.expect("response");
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let value = if bytes.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_slice(&bytes).expect("json")
        };
        (status, value)
    }

    #[tokio::test]
    async fn non_admin_cannot_use_editor_routes() {
        let (_temp, store) = open_store();
        let router = app(store, auth("user-tunar", UserRole::User));

        let request = Request::builder()
            .uri("/api/v1/admin/editor/workspaces")
            .body(Body::empty())
            .expect("request");
        let response = router.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn admin_editor_exposes_workspace_documents() {
        let (_temp, store) = open_store();
        let router = app(store, auth("user-admin", UserRole::Admin));

        let (status, created) = request_json(
            &router,
            "POST",
            "/api/v1/workspaces",
            json!({
                "name": "Automation",
                "language": "markdown",
                "body": "# Launch"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        let workspace_id = created["id"].as_str().expect("workspace id");

        let request = Request::builder()
            .uri(format!("/api/v1/admin/editor/workspaces/{workspace_id}"))
            .body(Body::empty())
            .expect("request");
        let response = router.clone().oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let detail: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(detail["documents"].as_array().map(Vec::len), Some(1));

        let (status, document) = request_json(
            &router,
            "POST",
            &format!("/api/v1/admin/editor/workspaces/{workspace_id}/documents"),
            json!({
                "path": "scripts/deploy.sh",
                "language": "bash",
                "body": "echo ready",
                "make_primary": false
            }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(document["path"], "scripts/deploy.sh");
    }
}
