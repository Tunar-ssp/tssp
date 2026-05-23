//! Workspace storage API.
//!
//! Workspaces are saved text/script buffers only. They are intentionally not
//! executable in this milestone, but ownership and validation are enforced now
//! so the future script foundation does not grow out of an unsafe scratchpad.

use std::path::Path;
use std::sync::Mutex;

use axum::extract::{Path as AxumPath, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use rusqlite::{params, Connection, ErrorCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::auth::AuthContext;
use crate::{ErrorBody, ErrorResponse, HttpState};

const MAX_WORKSPACE_NAME_BYTES: usize = 120;
const MAX_WORKSPACE_LANGUAGE_BYTES: usize = 40;
const MAX_WORKSPACE_BODY_BYTES: usize = 1_048_576;

/// SQLite-backed workspace store.
#[derive(Debug)]
pub struct WorkspaceStore {
    connection: Mutex<Connection>,
}

/// Workspace persistence failures.
#[derive(Debug, Error)]
pub enum WorkspaceError {
    /// `SQLite` operation failed.
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    /// The requested workspace does not exist or is not visible to the caller.
    #[error("not found")]
    NotFound,
    /// A generated id collided with an existing workspace.
    #[error("workspace already exists")]
    Conflict,
    /// The connection mutex was poisoned.
    #[error("workspace database lock is poisoned")]
    LockPoisoned,
}

#[derive(Debug, Error, PartialEq, Eq)]
enum WorkspaceValidationError {
    #[error("{field} must not be empty")]
    Empty { field: &'static str },
    #[error("{field} must not exceed {max} bytes")]
    TooLong { field: &'static str, max: usize },
    #[error("{field} contains unsupported characters")]
    InvalidCharacters { field: &'static str },
}

#[derive(Debug, Clone, Serialize)]
struct WorkspaceRecord {
    id: String,
    owner_id: String,
    name: String,
    language: String,
    body: String,
    created_at: i64,
    updated_at: i64,
}

impl WorkspaceStore {
    /// Opens the workspace store at the metadata database path.
    ///
    /// # Errors
    ///
    /// Returns [`WorkspaceError`] when the database cannot be opened.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, WorkspaceError> {
        let connection = Connection::open(path)?;
        connection.busy_timeout(std::time::Duration::from_secs(5))?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, WorkspaceError> {
        self.connection
            .lock()
            .map_err(|_| WorkspaceError::LockPoisoned)
    }

    fn list_for_owner(&self, owner_id: &str) -> Result<Vec<WorkspaceRecord>, WorkspaceError> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(
            "SELECT id, owner_id, name, language, body, created_at, updated_at
             FROM workspaces
             WHERE owner_id = ?1
             ORDER BY updated_at DESC
             LIMIT 200",
        )?;
        let rows = statement.query_map(params![owner_id], map_row)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(WorkspaceError::Database)
    }

    fn get(&self, id: &str, owner_id: Option<&str>) -> Result<WorkspaceRecord, WorkspaceError> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(
            "SELECT id, owner_id, name, language, body, created_at, updated_at
             FROM workspaces
             WHERE id = ?1",
        )?;
        let mut rows = statement.query(params![id])?;
        let Some(row) = rows.next()? else {
            return Err(WorkspaceError::NotFound);
        };
        let record = map_row(row)?;
        if let Some(owner) = owner_id {
            if record.owner_id != owner {
                return Err(WorkspaceError::NotFound);
            }
        }
        Ok(record)
    }

    fn insert(&self, record: &WorkspaceRecord) -> Result<(), WorkspaceError> {
        let connection = self.lock()?;
        connection
            .execute(
            "INSERT INTO workspaces (id, owner_id, name, language, body, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                record.id,
                record.owner_id,
                record.name,
                record.language,
                record.body,
                record.created_at,
                record.updated_at,
            ],
        )
            .map_err(map_write_error)?;
        Ok(())
    }

    fn update(&self, record: &WorkspaceRecord) -> Result<(), WorkspaceError> {
        let connection = self.lock()?;
        let changed = connection.execute(
            "UPDATE workspaces
             SET name = ?1, language = ?2, body = ?3, updated_at = ?4
             WHERE id = ?5 AND owner_id = ?6",
            params![
                record.name,
                record.language,
                record.body,
                record.updated_at,
                record.id,
                record.owner_id,
            ],
        )?;
        if changed == 0 {
            return Err(WorkspaceError::NotFound);
        }
        Ok(())
    }

    fn delete(&self, id: &str, owner_id: Option<&str>) -> Result<(), WorkspaceError> {
        let connection = self.lock()?;
        let changed = if let Some(owner_id) = owner_id {
            connection.execute(
                "DELETE FROM workspaces WHERE id = ?1 AND owner_id = ?2",
                params![id, owner_id],
            )?
        } else {
            connection.execute("DELETE FROM workspaces WHERE id = ?1", params![id])?
        };
        if changed == 0 {
            return Err(WorkspaceError::NotFound);
        }
        Ok(())
    }

    fn list_all(&self) -> Result<Vec<WorkspaceRecord>, WorkspaceError> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(
            "SELECT id, owner_id, name, language, body, created_at, updated_at
             FROM workspaces
             ORDER BY updated_at DESC
             LIMIT 500",
        )?;
        let rows = statement.query_map([], map_row)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(WorkspaceError::Database)
    }
}

fn map_row(row: &rusqlite::Row<'_>) -> Result<WorkspaceRecord, rusqlite::Error> {
    Ok(WorkspaceRecord {
        id: row.get(0)?,
        owner_id: row.get(1)?,
        name: row.get(2)?,
        language: row.get(3)?,
        body: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn map_write_error(error: rusqlite::Error) -> WorkspaceError {
    if let rusqlite::Error::SqliteFailure(sqlite_error, _) = &error {
        if sqlite_error.code == ErrorCode::ConstraintViolation {
            return WorkspaceError::Conflict;
        }
    }
    WorkspaceError::Database(error)
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateWorkspaceBody {
    name: String,
    #[serde(default = "default_language")]
    language: String,
    #[serde(default)]
    body: String,
}

fn default_language() -> String {
    "text".to_owned()
}

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateWorkspaceBody {
    name: String,
    language: String,
    body: String,
}

#[derive(Debug, Serialize)]
struct WorkspaceListResponse {
    schema_version: u8,
    workspaces: Vec<WorkspaceRecord>,
}

fn store(state: &HttpState) -> Option<&WorkspaceStore> {
    state.workspaces.as_deref()
}

fn new_workspace_id() -> String {
    format!("ws-{}", Uuid::now_v7().as_simple())
}

fn validate_name(value: &str) -> Result<String, WorkspaceValidationError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(WorkspaceValidationError::Empty { field: "name" });
    }
    if trimmed.len() > MAX_WORKSPACE_NAME_BYTES {
        return Err(WorkspaceValidationError::TooLong {
            field: "name",
            max: MAX_WORKSPACE_NAME_BYTES,
        });
    }
    if trimmed.chars().any(char::is_control) {
        return Err(WorkspaceValidationError::InvalidCharacters { field: "name" });
    }
    Ok(trimmed.to_owned())
}

fn validate_language(value: &str) -> Result<String, WorkspaceValidationError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(WorkspaceValidationError::Empty { field: "language" });
    }
    if trimmed.len() > MAX_WORKSPACE_LANGUAGE_BYTES {
        return Err(WorkspaceValidationError::TooLong {
            field: "language",
            max: MAX_WORKSPACE_LANGUAGE_BYTES,
        });
    }
    if trimmed.chars().any(|character| {
        !(character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '+' | '#' | '.'))
    }) {
        return Err(WorkspaceValidationError::InvalidCharacters { field: "language" });
    }
    Ok(trimmed.to_ascii_lowercase())
}

fn validate_body(value: String) -> Result<String, WorkspaceValidationError> {
    if value.len() > MAX_WORKSPACE_BODY_BYTES {
        return Err(WorkspaceValidationError::TooLong {
            field: "body",
            max: MAX_WORKSPACE_BODY_BYTES,
        });
    }
    Ok(value)
}

fn validate_workspace(
    name: &str,
    language: &str,
    body: String,
) -> Result<(String, String, String), WorkspaceValidationError> {
    Ok((
        validate_name(name)?,
        validate_language(language)?,
        validate_body(body)?,
    ))
}

/// `GET /api/v1/workspaces`
pub(crate) async fn list_workspaces(State(state): State<HttpState>, auth: AuthContext) -> Response {
    let Some(store) = store(&state) else {
        return unavailable();
    };
    let result = if auth.is_admin() {
        store.list_all()
    } else {
        store.list_for_owner(auth.user_id.as_str())
    };
    match result {
        Ok(items) => (
            StatusCode::OK,
            Json(WorkspaceListResponse {
                schema_version: 1,
                workspaces: items,
            }),
        )
            .into_response(),
        Err(error) => internal(error.to_string()),
    }
}

/// `POST /api/v1/workspaces`
pub(crate) async fn create_workspace(
    State(state): State<HttpState>,
    auth: AuthContext,
    Json(body): Json<CreateWorkspaceBody>,
) -> Response {
    let Some(store) = store(&state) else {
        return unavailable();
    };
    let (name, language, body) = match validate_workspace(&body.name, &body.language, body.body) {
        Ok(value) => value,
        Err(error) => return bad_request(&error.to_string()),
    };
    let now = {
        use tssp_ports::Clock;
        tssp_adapter_system::SystemClock.now().seconds()
    };
    let record = WorkspaceRecord {
        id: new_workspace_id(),
        owner_id: auth.user_id.as_str().to_owned(),
        name,
        language,
        body,
        created_at: now,
        updated_at: now,
    };
    match store.insert(&record) {
        Ok(()) => (StatusCode::CREATED, Json(record)).into_response(),
        Err(WorkspaceError::Conflict) => conflict(),
        Err(error) => internal(error.to_string()),
    }
}

/// `GET /api/v1/workspaces/{id}`
pub(crate) async fn get_workspace(
    State(state): State<HttpState>,
    auth: AuthContext,
    AxumPath(id): AxumPath<String>,
) -> Response {
    let Some(store) = store(&state) else {
        return unavailable();
    };
    let owner = if auth.is_admin() {
        None
    } else {
        Some(auth.user_id.as_str())
    };
    match store.get(&id, owner) {
        Ok(record) => (StatusCode::OK, Json(record)).into_response(),
        Err(WorkspaceError::NotFound) => not_found(),
        Err(error) => internal(error.to_string()),
    }
}

/// `PUT /api/v1/workspaces/{id}`
pub(crate) async fn update_workspace(
    State(state): State<HttpState>,
    auth: AuthContext,
    AxumPath(id): AxumPath<String>,
    Json(body): Json<UpdateWorkspaceBody>,
) -> Response {
    let Some(store) = store(&state) else {
        return unavailable();
    };
    let owner_filter = if auth.is_admin() {
        None
    } else {
        Some(auth.user_id.as_str())
    };
    let existing = match store.get(&id, owner_filter) {
        Ok(record) => record,
        Err(WorkspaceError::NotFound) => return not_found(),
        Err(error) => return internal(error.to_string()),
    };
    let (name, language, body) = match validate_workspace(&body.name, &body.language, body.body) {
        Ok(value) => value,
        Err(error) => return bad_request(&error.to_string()),
    };
    let now = {
        use tssp_ports::Clock;
        tssp_adapter_system::SystemClock.now().seconds()
    };
    let record = WorkspaceRecord {
        id,
        owner_id: existing.owner_id,
        name,
        language,
        body,
        created_at: existing.created_at,
        updated_at: now,
    };
    match store.update(&record) {
        Ok(()) => (StatusCode::OK, Json(record)).into_response(),
        Err(WorkspaceError::NotFound) => not_found(),
        Err(error) => internal(error.to_string()),
    }
}

/// `DELETE /api/v1/workspaces/{id}`
pub(crate) async fn delete_workspace(
    State(state): State<HttpState>,
    auth: AuthContext,
    AxumPath(id): AxumPath<String>,
) -> Response {
    let Some(store) = store(&state) else {
        return unavailable();
    };
    let owner_filter = if auth.is_admin() {
        None
    } else {
        Some(auth.user_id.as_str())
    };
    match store.delete(&id, owner_filter) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(WorkspaceError::NotFound) => not_found(),
        Err(error) => internal(error.to_string()),
    }
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

fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "not_found",
                message: "workspace not found".to_owned(),
            },
        }),
    )
        .into_response()
}

fn conflict() -> Response {
    (
        StatusCode::CONFLICT,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "workspace_conflict",
                message: "workspace id already exists; retry the request".to_owned(),
            },
        }),
    )
        .into_response()
}

fn bad_request(message: &str) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: ErrorBody {
                code: "invalid_request",
                message: message.to_owned(),
            },
        }),
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::sync::Arc;

    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use axum::middleware;
    use axum::routing::get;
    use axum::Router;
    use serde_json::json;
    use tempfile::TempDir;
    use tower::ServiceExt;
    use tssp_adapter_sqlite::SqliteFileRepository;
    use tssp_domain::{UserId, UserRole};

    use super::{
        create_workspace, delete_workspace, get_workspace, list_workspaces, new_workspace_id,
        update_workspace, validate_language, validate_name, WorkspaceRecord, WorkspaceStore,
    };
    use crate::auth::AuthContext;
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
        let state = HttpState::new(
            std::time::Instant::now(),
            std::path::PathBuf::from("/tmp"),
            settings.clone(),
            PublicUrlBuilder::from_settings(&settings),
            0,
        )
        .with_workspaces(Arc::new(store));
        Router::new()
            .route(
                "/api/v1/workspaces",
                get(list_workspaces).post(create_workspace),
            )
            .route(
                "/api/v1/workspaces/{id}",
                get(get_workspace)
                    .put(update_workspace)
                    .delete(delete_workspace),
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

    fn insert_record(store: &WorkspaceStore, id: &str, owner: &str) {
        store
            .insert(&WorkspaceRecord {
                id: id.to_owned(),
                owner_id: owner.to_owned(),
                name: "Draft".to_owned(),
                language: "text".to_owned(),
                body: "body".to_owned(),
                created_at: 1,
                updated_at: 1,
            })
            .expect("insert");
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

    #[test]
    fn generated_workspace_ids_are_prefixed_and_unique() {
        let first = new_workspace_id();
        let second = new_workspace_id();

        assert!(first.starts_with("ws-"));
        assert_ne!(first, second);
    }

    #[test]
    fn validation_rejects_empty_and_invalid_fields() {
        assert!(validate_name("   ").is_err());
        assert!(validate_name("good name").is_ok());
        assert!(validate_language("rust").is_ok());
        assert!(validate_language("../sh").is_err());
    }

    #[tokio::test]
    async fn create_workspace_validates_and_returns_record() {
        let (_temp, store) = open_store();
        let router = app(store, auth("user-tunar", UserRole::User));

        let (status, body) = request_json(
            &router,
            "POST",
            "/api/v1/workspaces",
            json!({
                "name": " Ops note ",
                "language": "Markdown",
                "body": "# Plan"
            }),
        )
        .await;

        assert_eq!(status, StatusCode::CREATED);
        assert!(body["id"].as_str().unwrap().starts_with("ws-"));
        assert_eq!(body["name"], "Ops note");
        assert_eq!(body["language"], "markdown");
    }

    #[tokio::test]
    async fn user_cannot_read_another_users_workspace() {
        let (_temp, store) = open_store();
        insert_record(&store, "ws-private", "user-owner");
        let router = app(store, auth("user-other", UserRole::User));

        let request = Request::builder()
            .uri("/api/v1/workspaces/ws-private")
            .body(Body::empty())
            .expect("request");
        let response = router.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn admin_can_update_and_delete_any_workspace() {
        let (_temp, store) = open_store();
        insert_record(&store, "ws-owned", "user-owner");
        let router = app(store, auth("user-admin", UserRole::Admin));

        let (status, body) = request_json(
            &router,
            "PUT",
            "/api/v1/workspaces/ws-owned",
            json!({
                "name": "Updated",
                "language": "python",
                "body": "print('saved only')"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["owner_id"], "user-owner");
        assert_eq!(body["name"], "Updated");

        let request = Request::builder()
            .method("DELETE")
            .uri("/api/v1/workspaces/ws-owned")
            .body(Body::empty())
            .expect("request");
        let response = router.clone().oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let request = Request::builder()
            .uri("/api/v1/workspaces/ws-owned")
            .body(Body::empty())
            .expect("request");
        let response = router.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
