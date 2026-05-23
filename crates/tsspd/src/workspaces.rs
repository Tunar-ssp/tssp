//! Script/workspace storage (no execution).

#![allow(missing_docs)]

use std::path::Path;
use std::sync::Mutex;

use axum::extract::{Path as AxumPath, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::auth::AuthContext;
use crate::{ErrorBody, ErrorResponse, HttpState};

/// SQLite-backed workspace store.
#[derive(Debug)]
pub struct WorkspaceStore {
    connection: Mutex<Connection>,
}

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceRecord {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    pub language: String,
    pub body: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl WorkspaceStore {
    /// Opens the workspace store at the metadata database path.
    ///
    /// # Errors
    ///
    /// Returns [`WorkspaceError`] when the database cannot be opened.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, WorkspaceError> {
        let connection = Connection::open(path)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, WorkspaceError> {
        self.connection
            .lock()
            .map_err(|_| WorkspaceError::Database(rusqlite::Error::InvalidQuery))
    }

    /// Lists workspaces for one owner.
    pub fn list_for_owner(&self, owner_id: &str) -> Result<Vec<WorkspaceRecord>, WorkspaceError> {
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

    /// Returns one workspace when owned by `owner_id` (or any when `owner_id` is None for admin).
    pub fn get(&self, id: &str, owner_id: Option<&str>) -> Result<WorkspaceRecord, WorkspaceError> {
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

    /// Inserts a workspace.
    pub fn insert(&self, record: &WorkspaceRecord) -> Result<(), WorkspaceError> {
        let connection = self.lock()?;
        connection.execute(
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
        )?;
        Ok(())
    }

    /// Updates workspace content.
    pub fn update(&self, record: &WorkspaceRecord) -> Result<(), WorkspaceError> {
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

    /// Deletes a workspace owned by `owner_id`.
    pub fn delete(&self, id: &str, owner_id: &str) -> Result<(), WorkspaceError> {
        let connection = self.lock()?;
        let changed = connection.execute(
            "DELETE FROM workspaces WHERE id = ?1 AND owner_id = ?2",
            params![id, owner_id],
        )?;
        if changed == 0 {
            return Err(WorkspaceError::NotFound);
        }
        Ok(())
    }

    /// Lists all workspaces (admin).
    pub fn list_all(&self) -> Result<Vec<WorkspaceRecord>, WorkspaceError> {
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

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceBody {
    pub name: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub body: String,
}

fn default_language() -> String {
    "text".to_owned()
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspaceBody {
    pub name: String,
    pub language: String,
    pub body: String,
}

fn store(state: &HttpState) -> Option<&WorkspaceStore> {
    state.workspaces.as_deref()
}

/// `GET /api/v1/workspaces`
pub async fn list_workspaces(State(state): State<HttpState>, auth: AuthContext) -> Response {
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
            Json(serde_json::json!({
                "schema_version": 1,
                "workspaces": items,
            })),
        )
            .into_response(),
        Err(error) => internal(error.to_string()),
    }
}

/// `POST /api/v1/workspaces`
pub async fn create_workspace(
    State(state): State<HttpState>,
    auth: AuthContext,
    Json(body): Json<CreateWorkspaceBody>,
) -> Response {
    let Some(store) = store(&state) else {
        return unavailable();
    };
    let now = {
        use tssp_ports::Clock;
        tssp_adapter_system::SystemClock.now().seconds()
    };
    let id = format!("ws-{}", now);
    let record = WorkspaceRecord {
        id,
        owner_id: auth.user_id.as_str().to_owned(),
        name: body.name.trim().to_owned(),
        language: body.language,
        body: body.body,
        created_at: now,
        updated_at: now,
    };
    if record.name.is_empty() {
        return bad_request("name must not be empty");
    }
    match store.insert(&record) {
        Ok(()) => (StatusCode::CREATED, Json(record)).into_response(),
        Err(error) => internal(error.to_string()),
    }
}

/// `GET /api/v1/workspaces/{id}`
pub async fn get_workspace(
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
pub async fn update_workspace(
    State(state): State<HttpState>,
    auth: AuthContext,
    AxumPath(id): AxumPath<String>,
    Json(body): Json<UpdateWorkspaceBody>,
) -> Response {
    let Some(store) = store(&state) else {
        return unavailable();
    };
    let now = {
        use tssp_ports::Clock;
        tssp_adapter_system::SystemClock.now().seconds()
    };
    let record = WorkspaceRecord {
        id,
        owner_id: auth.user_id.as_str().to_owned(),
        name: body.name.trim().to_owned(),
        language: body.language,
        body: body.body,
        created_at: now,
        updated_at: now,
    };
    match store.update(&record) {
        Ok(()) => (StatusCode::OK, Json(record)).into_response(),
        Err(WorkspaceError::NotFound) => not_found(),
        Err(error) => internal(error.to_string()),
    }
}

/// `DELETE /api/v1/workspaces/{id}`
pub async fn delete_workspace(
    State(state): State<HttpState>,
    auth: AuthContext,
    AxumPath(id): AxumPath<String>,
) -> Response {
    let Some(store) = store(&state) else {
        return unavailable();
    };
    match store.delete(&id, auth.user_id.as_str()) {
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
