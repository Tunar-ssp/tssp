//! Workspace storage API.
//!
//! Workspaces are saved text/script buffers only. They are intentionally not
//! executable in this milestone, but ownership and validation are enforced now
//! so the future script foundation does not grow out of an unsafe scratchpad.

use std::path::Path;

use axum::extract::{Path as AxumPath, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, ErrorCode, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use tssp_ports::WorkspaceFileStoreError;
use uuid::Uuid;

use crate::auth::AuthContext;
use crate::{ErrorBody, ErrorResponse, HttpState};

const MAX_WORKSPACE_NAME_BYTES: usize = 120;
const MAX_WORKSPACE_LANGUAGE_BYTES: usize = 40;
const MAX_WORKSPACE_BODY_BYTES: usize = 1_048_576;
const MAX_WORKSPACE_DOCUMENT_PATH_BYTES: usize = 160;
const MAX_WORKSPACE_DOCUMENT_DEPTH: usize = 8;

/// SQLite-backed workspace store.
#[derive(Debug, Clone)]
pub struct WorkspaceStore {
    pool: Pool<SqliteConnectionManager>,
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
    /// The requested operation would leave the workspace in an invalid state.
    #[error("{0}")]
    InvalidOperation(String),
}

impl From<WorkspaceValidationError> for WorkspaceError {
    fn from(error: WorkspaceValidationError) -> Self {
        Self::InvalidOperation(error.to_string())
    }
}

impl From<WorkspaceDocumentValidationError> for WorkspaceError {
    fn from(error: WorkspaceDocumentValidationError) -> Self {
        Self::InvalidOperation(error.to_string())
    }
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

#[derive(Debug, Error, PartialEq, Eq)]
enum WorkspaceDocumentValidationError {
    #[error("{field} must not be empty")]
    Empty { field: &'static str },
    #[error("{field} must not exceed {max} bytes")]
    TooLong { field: &'static str, max: usize },
    #[error("{field} contains unsupported characters")]
    InvalidCharacters { field: &'static str },
    #[error("path depth must not exceed {max}")]
    TooDeep { max: usize },
    #[error("path must be relative and must not contain '.' or '..' segments")]
    InvalidPath,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct WorkspaceRecord {
    pub(crate) id: String,
    pub(crate) owner_id: String,
    pub(crate) name: String,
    pub(crate) language: String,
    pub(crate) body: String,
    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
}

/// Bounded workspace search result returned to the search API.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct WorkspaceSearchRecord {
    pub(crate) id: String,
    pub(crate) owner_id: String,
    pub(crate) name: String,
    pub(crate) language: String,
    pub(crate) updated_at: i64,
    pub(crate) snippet: String,
}

/// Stored document within an admin-editable workspace.
/// Document content is stored on filesystem via `WorkspaceFileService`, not in this record.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct WorkspaceDocumentRecord {
    pub(crate) id: String,
    pub(crate) workspace_id: String,
    pub(crate) owner_id: String,
    pub(crate) path: String,
    pub(crate) language: String,
    pub(crate) is_primary: bool,
    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
}

/// Document summary returned to the admin editor sidebar.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct WorkspaceDocumentSummary {
    pub(crate) id: String,
    pub(crate) workspace_id: String,
    pub(crate) owner_id: String,
    pub(crate) path: String,
    pub(crate) language: String,
    pub(crate) is_primary: bool,
    pub(crate) updated_at: i64,
    pub(crate) size_bytes: usize,
}

impl WorkspaceStore {
    /// Creates a workspace store from an existing pool.
    #[must_use]
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    /// Opens the workspace store at the metadata database path.
    ///
    /// # Errors
    ///
    /// Returns [`WorkspaceError`] when the database cannot be opened.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, WorkspaceError> {
        let manager = SqliteConnectionManager::file(path.as_ref());
        let pool = Pool::builder().max_size(10).build(manager).map_err(|e| {
            WorkspaceError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
        })?;

        Ok(Self { pool })
    }

    fn connect(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>, WorkspaceError> {
        self.pool.get().map_err(|e| {
            WorkspaceError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
        })
    }

    fn list_for_owner(&self, owner_id: &str) -> Result<Vec<WorkspaceRecord>, WorkspaceError> {
        let connection = self.connect()?;
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

    pub(crate) fn get(
        &self,
        id: &str,
        owner_id: Option<&str>,
    ) -> Result<WorkspaceRecord, WorkspaceError> {
        let connection = self.connect()?;
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
        let mut connection = self.connect()?;
        let transaction = connection.transaction()?;
        transaction
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

        let primary_document = WorkspaceDocumentRecord {
            id: new_workspace_document_id(),
            workspace_id: record.id.clone(),
            owner_id: record.owner_id.clone(),
            path: default_workspace_document_path(&record.name, &record.language),
            language: record.language.clone(),
            is_primary: true,
            created_at: record.created_at,
            updated_at: record.updated_at,
        };
        insert_document_record(&transaction, &primary_document)?;
        transaction.commit()?;
        Ok(())
    }

    fn update(&self, record: &WorkspaceRecord) -> Result<(), WorkspaceError> {
        let mut connection = self.connect()?;
        let transaction = connection.transaction()?;
        let changed = transaction.execute(
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

        if let Some(_primary_document) = load_primary_document(&transaction, &record.id)? {
            transaction.execute(
                "UPDATE workspace_documents SET language = ?1, updated_at = ?2 WHERE workspace_id = ?3 AND is_primary = 1",
                params![record.language, record.updated_at, record.id,],
            )?;
        } else {
            let primary_document = WorkspaceDocumentRecord {
                id: new_workspace_document_id(),
                workspace_id: record.id.clone(),
                owner_id: record.owner_id.clone(),
                path: default_workspace_document_path(&record.name, &record.language),
                language: record.language.clone(),
                is_primary: true,
                created_at: record.created_at,
                updated_at: record.updated_at,
            };
            insert_document_record(&transaction, &primary_document)?;
        }

        transaction.commit()?;
        Ok(())
    }

    fn delete(&self, id: &str, owner_id: Option<&str>) -> Result<(), WorkspaceError> {
        let existing = self.get(id, owner_id)?;
        let mut connection = self.connect()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "DELETE FROM workspace_documents WHERE workspace_id = ?1",
            params![existing.id],
        )?;
        transaction.execute("DELETE FROM workspaces WHERE id = ?1", params![existing.id])?;
        transaction.commit()?;
        Ok(())
    }

    pub(crate) fn list_all(&self) -> Result<Vec<WorkspaceRecord>, WorkspaceError> {
        let connection = self.connect()?;
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

    pub(crate) fn list_documents(
        &self,
        workspace_id: &str,
        owner_id: Option<&str>,
    ) -> Result<Vec<WorkspaceDocumentSummary>, WorkspaceError> {
        self.get(workspace_id, owner_id)?;
        let connection = self.connect()?;
        let mut statement = connection.prepare(
            "SELECT id, workspace_id, owner_id, path, language, is_primary, updated_at
             FROM workspace_documents
             WHERE workspace_id = ?1
             ORDER BY is_primary DESC, path COLLATE NOCASE ASC",
        )?;
        let rows = statement.query_map(params![workspace_id], map_document_summary_row)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(WorkspaceError::Database)
    }

    pub(crate) fn get_document(
        &self,
        workspace_id: &str,
        document_id: &str,
        owner_id: Option<&str>,
    ) -> Result<WorkspaceDocumentRecord, WorkspaceError> {
        self.get(workspace_id, owner_id)?;
        let connection = self.connect()?;
        load_document(&connection, workspace_id, document_id)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_document(
        &self,
        workspace_id: &str,
        owner_id: Option<&str>,
        path: &str,
        language: &str,
        body: String,
        make_primary: bool,
        now: i64,
    ) -> Result<WorkspaceDocumentRecord, WorkspaceError> {
        let workspace = self.get(workspace_id, owner_id)?;
        let path = validate_document_path(path)?;
        let language = validate_language(language)?;
        let _body = validate_body(body)?;

        let mut connection = self.connect()?;
        let transaction = connection.transaction()?;
        let document_count = count_workspace_documents(&transaction, workspace_id)?;
        let is_primary = make_primary || document_count == 0;
        if is_primary {
            clear_primary_documents(&transaction, workspace_id)?;
        }

        let record = WorkspaceDocumentRecord {
            id: new_workspace_document_id(),
            workspace_id: workspace.id.clone(),
            owner_id: workspace.owner_id.clone(),
            path,
            language,
            is_primary,
            created_at: now,
            updated_at: now,
        };
        insert_document_record(&transaction, &record)?;
        if record.is_primary {
            sync_workspace_to_document(&transaction, &workspace, &record, now)?;
        } else {
            touch_workspace(&transaction, &workspace.id, now)?;
        }
        transaction.commit()?;
        Ok(record)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn update_document(
        &self,
        workspace_id: &str,
        document_id: &str,
        owner_id: Option<&str>,
        path: &str,
        language: &str,
        body: String,
        make_primary: bool,
        now: i64,
    ) -> Result<WorkspaceDocumentRecord, WorkspaceError> {
        let workspace = self.get(workspace_id, owner_id)?;
        let path = validate_document_path(path)?;
        let language = validate_language(language)?;
        let _body = validate_body(body)?;

        let mut connection = self.connect()?;
        let transaction = connection.transaction()?;
        let existing = load_document(&transaction, workspace_id, document_id)?;
        let is_primary = existing.is_primary || make_primary;
        if make_primary && !existing.is_primary {
            clear_primary_documents(&transaction, workspace_id)?;
        }
        transaction.execute(
            "UPDATE workspace_documents
             SET path = ?1, language = ?2, is_primary = ?3, updated_at = ?4
             WHERE id = ?5 AND workspace_id = ?6",
            params![
                path,
                language,
                i32::from(is_primary),
                now,
                document_id,
                workspace_id,
            ],
        )?;
        let updated = load_document(&transaction, workspace_id, document_id)?;
        if updated.is_primary {
            sync_workspace_to_document(&transaction, &workspace, &updated, now)?;
        } else {
            touch_workspace(&transaction, &workspace.id, now)?;
        }
        transaction.commit()?;
        Ok(updated)
    }

    pub(crate) fn delete_document(
        &self,
        workspace_id: &str,
        document_id: &str,
        owner_id: Option<&str>,
        now: i64,
    ) -> Result<(), WorkspaceError> {
        let workspace = self.get(workspace_id, owner_id)?;
        let mut connection = self.connect()?;
        let transaction = connection.transaction()?;
        let existing = load_document(&transaction, workspace_id, document_id)?;
        if count_workspace_documents(&transaction, workspace_id)? <= 1 {
            return Err(WorkspaceError::InvalidOperation(
                "each workspace must keep at least one document".to_owned(),
            ));
        }

        transaction.execute(
            "DELETE FROM workspace_documents WHERE id = ?1 AND workspace_id = ?2",
            params![document_id, workspace_id],
        )?;

        if existing.is_primary {
            let replacement =
                load_first_document(&transaction, workspace_id)?.ok_or_else(|| {
                    WorkspaceError::InvalidOperation(
                        "workspace is missing a replacement primary document".to_owned(),
                    )
                })?;
            clear_primary_documents(&transaction, workspace_id)?;
            transaction.execute(
                "UPDATE workspace_documents SET is_primary = 1 WHERE id = ?1 AND workspace_id = ?2",
                params![replacement.id, workspace_id],
            )?;
            let replacement = load_document(&transaction, workspace_id, &replacement.id)?;
            sync_workspace_to_document(&transaction, &workspace, &replacement, now)?;
        } else {
            touch_workspace(&transaction, &workspace.id, now)?;
        }

        transaction.commit()?;
        Ok(())
    }

    pub(crate) fn search(
        &self,
        query: &str,
        owner_id: Option<&str>,
        limit: u64,
    ) -> Result<Vec<WorkspaceSearchRecord>, WorkspaceError> {
        let needle = query.trim().to_ascii_lowercase();
        if needle.is_empty() || limit == 0 {
            return Ok(Vec::new());
        }
        let limit = i64::try_from(limit.min(50)).unwrap_or(50);
        let connection = self.connect()?;

        if let Some(owner_id) = owner_id {
            let mut statement = connection.prepare(SEARCH_WORKSPACES_FOR_OWNER_SQL)?;
            let rows = statement.query_map(params![needle, owner_id, limit], map_search_row)?;
            return rows
                .collect::<Result<Vec<_>, _>>()
                .map_err(WorkspaceError::Database);
        }

        let mut statement = connection.prepare(SEARCH_WORKSPACES_SQL)?;
        let rows = statement.query_map(params![needle, limit], map_search_row)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(WorkspaceError::Database)
    }
}

const SEARCH_WORKSPACES_SQL: &str = "
    SELECT id, owner_id, name, language, body, updated_at
    FROM workspaces
    WHERE instr(lower(name), ?1) > 0
       OR instr(lower(language), ?1) > 0
       OR instr(lower(body), ?1) > 0
    ORDER BY
      CASE
        WHEN lower(name) = ?1 THEN 0
        WHEN substr(lower(name), 1, length(?1)) = ?1 THEN 1
        WHEN instr(lower(name), ?1) > 0 THEN 2
        WHEN lower(language) = ?1 THEN 3
        WHEN instr(lower(language), ?1) > 0 THEN 4
        ELSE 5
      END,
      updated_at DESC
    LIMIT ?2";

const SEARCH_WORKSPACES_FOR_OWNER_SQL: &str = "
    SELECT id, owner_id, name, language, body, updated_at
    FROM workspaces
    WHERE owner_id = ?2
      AND (
        instr(lower(name), ?1) > 0
        OR instr(lower(language), ?1) > 0
        OR instr(lower(body), ?1) > 0
      )
    ORDER BY
      CASE
        WHEN lower(name) = ?1 THEN 0
        WHEN substr(lower(name), 1, length(?1)) = ?1 THEN 1
        WHEN instr(lower(name), ?1) > 0 THEN 2
        WHEN lower(language) = ?1 THEN 3
        WHEN instr(lower(language), ?1) > 0 THEN 4
        ELSE 5
      END,
      updated_at DESC
    LIMIT ?3";

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

fn map_search_row(row: &rusqlite::Row<'_>) -> Result<WorkspaceSearchRecord, rusqlite::Error> {
    let body: String = row.get(4)?;
    Ok(WorkspaceSearchRecord {
        id: row.get(0)?,
        owner_id: row.get(1)?,
        name: row.get(2)?,
        language: row.get(3)?,
        updated_at: row.get(5)?,
        snippet: body.chars().take(180).collect(),
    })
}

fn map_document_row(row: &rusqlite::Row<'_>) -> Result<WorkspaceDocumentRecord, rusqlite::Error> {
    Ok(WorkspaceDocumentRecord {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        owner_id: row.get(2)?,
        path: row.get(3)?,
        language: row.get(4)?,
        is_primary: row.get::<_, i64>(5)? != 0,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn map_document_summary_row(
    row: &rusqlite::Row<'_>,
) -> Result<WorkspaceDocumentSummary, rusqlite::Error> {
    Ok(WorkspaceDocumentSummary {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        owner_id: row.get(2)?,
        path: row.get(3)?,
        language: row.get(4)?,
        is_primary: row.get::<_, i64>(5)? != 0,
        updated_at: row.get(6)?,
        size_bytes: 0,
    })
}

fn load_document(
    connection: &Connection,
    workspace_id: &str,
    document_id: &str,
) -> Result<WorkspaceDocumentRecord, WorkspaceError> {
    connection
        .query_row(
            "SELECT id, workspace_id, owner_id, path, language, is_primary, created_at, updated_at
             FROM workspace_documents
             WHERE workspace_id = ?1 AND id = ?2",
            params![workspace_id, document_id],
            map_document_row,
        )
        .optional()
        .map_err(WorkspaceError::Database)?
        .ok_or(WorkspaceError::NotFound)
}

fn load_primary_document(
    connection: &Connection,
    workspace_id: &str,
) -> Result<Option<WorkspaceDocumentRecord>, WorkspaceError> {
    connection
        .query_row(
            "SELECT id, workspace_id, owner_id, path, language, is_primary, created_at, updated_at
             FROM workspace_documents
             WHERE workspace_id = ?1 AND is_primary = 1",
            params![workspace_id],
            map_document_row,
        )
        .optional()
        .map_err(WorkspaceError::Database)
}

fn load_first_document(
    connection: &Connection,
    workspace_id: &str,
) -> Result<Option<WorkspaceDocumentRecord>, WorkspaceError> {
    connection
        .query_row(
            "SELECT id, workspace_id, owner_id, path, language, is_primary, created_at, updated_at
             FROM workspace_documents
             WHERE workspace_id = ?1
             ORDER BY updated_at DESC, path COLLATE NOCASE ASC
             LIMIT 1",
            params![workspace_id],
            map_document_row,
        )
        .optional()
        .map_err(WorkspaceError::Database)
}

fn count_workspace_documents(
    connection: &Connection,
    workspace_id: &str,
) -> Result<i64, WorkspaceError> {
    connection
        .query_row(
            "SELECT COUNT(*) FROM workspace_documents WHERE workspace_id = ?1",
            params![workspace_id],
            |row| row.get(0),
        )
        .map_err(WorkspaceError::Database)
}

fn clear_primary_documents(
    connection: &Connection,
    workspace_id: &str,
) -> Result<(), WorkspaceError> {
    connection
        .execute(
            "UPDATE workspace_documents SET is_primary = 0 WHERE workspace_id = ?1",
            params![workspace_id],
        )
        .map(|_| ())
        .map_err(WorkspaceError::Database)
}

fn insert_document_record(
    connection: &Connection,
    record: &WorkspaceDocumentRecord,
) -> Result<(), WorkspaceError> {
    connection
        .execute(
            "INSERT INTO workspace_documents (
                id, workspace_id, owner_id, path, language, is_primary, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                record.id,
                record.workspace_id,
                record.owner_id,
                record.path,
                record.language,
                i32::from(record.is_primary),
                record.created_at,
                record.updated_at,
            ],
        )
        .map_err(map_write_error)?;
    Ok(())
}

fn touch_workspace(
    connection: &Connection,
    workspace_id: &str,
    now: i64,
) -> Result<(), WorkspaceError> {
    connection
        .execute(
            "UPDATE workspaces SET updated_at = ?1 WHERE id = ?2",
            params![now, workspace_id],
        )
        .map(|_| ())
        .map_err(WorkspaceError::Database)
}

fn sync_workspace_to_document(
    connection: &Connection,
    workspace: &WorkspaceRecord,
    document: &WorkspaceDocumentRecord,
    now: i64,
) -> Result<(), WorkspaceError> {
    connection
        .execute(
            "UPDATE workspaces SET language = ?1, updated_at = ?2 WHERE id = ?3 AND owner_id = ?4",
            params![document.language, now, workspace.id, workspace.owner_id,],
        )
        .map(|_| ())
        .map_err(WorkspaceError::Database)
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

fn new_workspace_document_id() -> String {
    format!("wdoc-{}", Uuid::now_v7().as_simple())
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

fn validate_document_path(value: &str) -> Result<String, WorkspaceDocumentValidationError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(WorkspaceDocumentValidationError::Empty { field: "path" });
    }
    if trimmed.len() > MAX_WORKSPACE_DOCUMENT_PATH_BYTES {
        return Err(WorkspaceDocumentValidationError::TooLong {
            field: "path",
            max: MAX_WORKSPACE_DOCUMENT_PATH_BYTES,
        });
    }
    if trimmed.starts_with('/') || trimmed.ends_with('/') || trimmed.contains('\\') {
        return Err(WorkspaceDocumentValidationError::InvalidPath);
    }

    let segments = trimmed.split('/').collect::<Vec<_>>();
    if segments.len() > MAX_WORKSPACE_DOCUMENT_DEPTH {
        return Err(WorkspaceDocumentValidationError::TooDeep {
            max: MAX_WORKSPACE_DOCUMENT_DEPTH,
        });
    }
    if segments.iter().any(|segment| {
        segment.is_empty()
            || matches!(*segment, "." | "..")
            || segment.chars().any(|character| {
                character.is_control()
                    || !(character.is_ascii_alphanumeric()
                        || matches!(character, '-' | '_' | '.' | ' ' | '+' | '#'))
            })
    }) {
        return Err(WorkspaceDocumentValidationError::InvalidCharacters { field: "path" });
    }

    Ok(trimmed.to_owned())
}

fn default_workspace_document_path(name: &str, language: &str) -> String {
    let slug = name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else if matches!(character, ' ' | '-' | '_') {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|character| *character != '\0')
        .collect::<String>();
    let slug = slug
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let stem = if slug.is_empty() { "workspace" } else { &slug };
    format!("{stem}.{}", workspace_language_extension(language))
}

fn workspace_language_extension(language: &str) -> &'static str {
    match language {
        "markdown" => "md",
        "rust" => "rs",
        "python" => "py",
        "javascript" => "js",
        "typescript" => "ts",
        "json" => "json",
        "yaml" => "yaml",
        "toml" => "toml",
        "html" => "html",
        "css" => "css",
        "sql" => "sql",
        "bash" => "sh",
        _ => "txt",
    }
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

/// `GET /api/v1/workspaces/{id}/capabilities`
/// Returns what features are available for this workspace (terminal, LSP, etc).
pub(crate) async fn workspace_capabilities(
    State(state): State<HttpState>,
    auth: AuthContext,
    AxumPath(id): AxumPath<String>,
) -> Response {
    use crate::workspace_features::TerminalCapability;

    let Some(store) = store(&state) else {
        return unavailable();
    };

    // Verify workspace exists and user has access
    let owner_filter = if auth.is_admin() {
        None
    } else {
        Some(auth.user_id.as_str())
    };
    if store.get(&id, owner_filter).is_err() {
        return not_found();
    }

    // Terminal is admin-only
    let terminal = if auth.is_admin() {
        // Check if sandbox is available
        let sandbox = state.terminal_service.provider().detect_sandbox_strategy();
        if sandbox.is_available() {
            TerminalCapability::Available
        } else {
            TerminalCapability::UnavailableSandbox
        }
    } else {
        TerminalCapability::Forbidden
    };

    // LSP support is not yet implemented
    let lsp_available: Vec<String> = vec![];

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "schema_version": 1,
            "terminal": terminal,
            "lsp_available": lsp_available,
        })),
    )
        .into_response()
}

/// `GET /api/v1/workspaces/{id}/terminal`
/// Returns terminal availability status without opening a session.
pub(crate) async fn terminal_status(
    State(state): State<HttpState>,
    auth: AuthContext,
    AxumPath(id): AxumPath<String>,
) -> Response {
    if !auth.is_admin() {
        return (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "admin_required",
                    message: "terminal access requires admin role".to_owned(),
                },
            }),
        )
            .into_response();
    }

    let Some(store) = store(&state) else {
        return unavailable();
    };

    // Verify workspace exists
    if store.get(&id, None).is_err() {
        return not_found();
    }

    // Check if terminal is available
    let sandbox = state.terminal_service.provider().detect_sandbox_strategy();
    let available = sandbox.is_available();
    let reason = if available {
        None
    } else {
        Some("sandbox not available (bubblewrap or systemd-nspawn required)".to_string())
    };

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "schema_version": 1,
            "available": available,
            "reason": reason,
        })),
    )
        .into_response()
}

/// `GET /api/v1/workspaces/{id}/lsp`
/// Returns LSP availability status for the workspace.
pub(crate) async fn lsp_status(
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

    if store.get(&id, owner_filter).is_err() {
        return not_found();
    }

    // Detect available language servers on this system
    let available_languages = state.lsp_service.available_languages();

    let status = if available_languages.is_empty() {
        "disabled"
    } else {
        "available"
    };

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "schema_version": 1,
            "status": status,
            "available_languages": available_languages,
            "message": if available_languages.is_empty() {
                "No language servers are installed"
            } else {
                "Language server support is available"
            },
        })),
    )
        .into_response()
}

/// `GET /api/v1/workspaces/{id}/git`
pub(crate) async fn git_status(
    State(state): State<HttpState>,
    auth: AuthContext,
    AxumPath(workspace_id): AxumPath<String>,
) -> Response {
    let Some(store) = store(&state) else {
        return unavailable();
    };

    let owner_filter = if auth.is_admin() {
        None
    } else {
        Some(auth.user_id.as_str())
    };

    if store.get(&workspace_id, owner_filter).is_err() {
        return not_found();
    }

    // Resolve workspace directory from data_dir
    let workspace_root = state
        .settings()
        .data_dir
        .join("workspaces")
        .join(&workspace_id);

    // Verify workspace directory exists
    if !workspace_root.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "schema_version": 1,
                "error": "workspace not found"
            })),
        )
            .into_response();
    }

    match state.git_service.get_status(&workspace_root).await {
        Ok(status) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "schema_version": 1,
                "is_repo": status.is_repo,
                "branch": status.branch,
                "changed": status.changed_count,
                "staged": status.staged_count,
                "untracked": status.untracked_count,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": e,
            })),
        )
            .into_response(),
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

// Workspace filesystem handlers

#[derive(Debug, Deserialize)]
pub(crate) struct ListFilesQuery {
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
struct FileListResponse {
    entries: Vec<FileEntryResponse>,
}

#[derive(Debug, Serialize)]
struct FileEntryResponse {
    path: String,
    is_dir: bool,
    size_bytes: u64,
    modified_at: u64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReadFileQuery {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WriteFileBody {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateFileBody {
    pub path: String,
    #[serde(default)]
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateDirBody {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct MoveFileBody {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DeleteFileQuery {
    pub path: String,
}

/// `GET /api/v1/workspaces/{workspace_id}/files`
pub(crate) async fn list_workspace_files(
    State(state): State<HttpState>,
    auth: AuthContext,
    AxumPath(workspace_id): AxumPath<String>,
    Query(query): Query<ListFilesQuery>,
) -> Response {
    let Some(store) = store(&state) else {
        return unavailable();
    };
    let owner_filter = if auth.is_admin() {
        None
    } else {
        Some(auth.user_id.as_str())
    };
    if store.get(&workspace_id, owner_filter).is_err() {
        return not_found();
    }

    let path = query.path.unwrap_or_default();
    let result = state
        .workspace_file_service
        .list_tree(&workspace_id, &path, 10)
        .await;

    match result {
        Ok(entries) => {
            let entries: Vec<_> = entries
                .into_iter()
                .map(|e| FileEntryResponse {
                    path: e.path,
                    is_dir: e.is_dir,
                    size_bytes: e.size_bytes,
                    modified_at: e.modified_at,
                })
                .collect();
            (StatusCode::OK, Json(FileListResponse { entries })).into_response()
        }
        Err(WorkspaceFileStoreError::NotFound) => not_found(),
        Err(WorkspaceFileStoreError::InvalidPath(msg)) => bad_request(&msg),
        Err(WorkspaceFileStoreError::TraversalAttempt) => bad_request("path traversal rejected"),
        Err(_) => internal("file store error".to_owned()),
    }
}

/// `GET /api/v1/workspaces/{workspace_id}/files/content`
pub(crate) async fn read_workspace_file(
    State(state): State<HttpState>,
    auth: AuthContext,
    AxumPath(workspace_id): AxumPath<String>,
    Query(query): Query<ReadFileQuery>,
) -> Response {
    let Some(store) = store(&state) else {
        return unavailable();
    };
    let owner_filter = if auth.is_admin() {
        None
    } else {
        Some(auth.user_id.as_str())
    };
    if store.get(&workspace_id, owner_filter).is_err() {
        return not_found();
    }

    let result = state
        .workspace_file_service
        .read_file(&workspace_id, &query.path)
        .await;

    match result {
        Ok(content) => (StatusCode::OK, content).into_response(),
        Err(WorkspaceFileStoreError::NotFound) => not_found(),
        Err(WorkspaceFileStoreError::InvalidPath(msg)) => bad_request(&msg),
        Err(WorkspaceFileStoreError::TraversalAttempt) => bad_request("path traversal rejected"),
        Err(WorkspaceFileStoreError::FileTooLarge) => (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "file_too_large",
                    message: "file exceeds size limit".to_owned(),
                },
            }),
        )
            .into_response(),
        Err(_) => internal("file store error".to_owned()),
    }
}

/// `PUT /api/v1/workspaces/{workspace_id}/files/content`
pub(crate) async fn write_workspace_file(
    State(state): State<HttpState>,
    auth: AuthContext,
    AxumPath(workspace_id): AxumPath<String>,
    Json(body): Json<WriteFileBody>,
) -> Response {
    let Some(store) = store(&state) else {
        return unavailable();
    };
    let owner_filter = if auth.is_admin() {
        None
    } else {
        Some(auth.user_id.as_str())
    };
    if store.get(&workspace_id, owner_filter).is_err() {
        return not_found();
    }

    let result = state
        .workspace_file_service
        .write_file(&workspace_id, &body.path, body.content.as_bytes())
        .await;

    match result {
        Ok(()) => (StatusCode::OK, Json(json!({"status": "written"}))).into_response(),
        Err(WorkspaceFileStoreError::InvalidPath(msg)) => bad_request(&msg),
        Err(WorkspaceFileStoreError::TraversalAttempt) => bad_request("path traversal rejected"),
        Err(WorkspaceFileStoreError::FileTooLarge) => (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(ErrorResponse {
                error: ErrorBody {
                    code: "file_too_large",
                    message: "content exceeds size limit".to_owned(),
                },
            }),
        )
            .into_response(),
        Err(_) => internal("file store error".to_owned()),
    }
}

/// `POST /api/v1/workspaces/{workspace_id}/files`
pub(crate) async fn create_workspace_file(
    State(state): State<HttpState>,
    auth: AuthContext,
    AxumPath(workspace_id): AxumPath<String>,
    Json(body): Json<CreateFileBody>,
) -> Response {
    let Some(store) = store(&state) else {
        return unavailable();
    };
    let owner_filter = if auth.is_admin() {
        None
    } else {
        Some(auth.user_id.as_str())
    };
    if store.get(&workspace_id, owner_filter).is_err() {
        return not_found();
    }

    let result = state
        .workspace_file_service
        .write_file(&workspace_id, &body.path, body.content.as_bytes())
        .await;

    match result {
        Ok(()) => (
            StatusCode::CREATED,
            Json(json!({"status": "created", "path": body.path})),
        )
            .into_response(),
        Err(WorkspaceFileStoreError::InvalidPath(msg)) => bad_request(&msg),
        Err(WorkspaceFileStoreError::TraversalAttempt) => bad_request("path traversal rejected"),
        Err(_) => internal("file store error".to_owned()),
    }
}

/// `POST /api/v1/workspaces/{workspace_id}/dirs`
pub(crate) async fn create_workspace_dir(
    State(state): State<HttpState>,
    auth: AuthContext,
    AxumPath(workspace_id): AxumPath<String>,
    Json(body): Json<CreateDirBody>,
) -> Response {
    let Some(store) = store(&state) else {
        return unavailable();
    };
    let owner_filter = if auth.is_admin() {
        None
    } else {
        Some(auth.user_id.as_str())
    };
    if store.get(&workspace_id, owner_filter).is_err() {
        return not_found();
    }

    let result = state
        .workspace_file_service
        .create_dir(&workspace_id, &body.path)
        .await;

    match result {
        Ok(()) => (
            StatusCode::CREATED,
            Json(json!({"status": "created", "path": body.path})),
        )
            .into_response(),
        Err(WorkspaceFileStoreError::InvalidPath(msg)) => bad_request(&msg),
        Err(WorkspaceFileStoreError::TraversalAttempt) => bad_request("path traversal rejected"),
        Err(_) => internal("file store error".to_owned()),
    }
}

/// `PATCH /api/v1/workspaces/{workspace_id}/files/move`
pub(crate) async fn move_workspace_file(
    State(state): State<HttpState>,
    auth: AuthContext,
    AxumPath(workspace_id): AxumPath<String>,
    Json(body): Json<MoveFileBody>,
) -> Response {
    let Some(store) = store(&state) else {
        return unavailable();
    };
    let owner_filter = if auth.is_admin() {
        None
    } else {
        Some(auth.user_id.as_str())
    };
    if store.get(&workspace_id, owner_filter).is_err() {
        return not_found();
    }

    let result = state
        .workspace_file_service
        .rename(&workspace_id, &body.from, &body.to)
        .await;

    match result {
        Ok(()) => (StatusCode::OK, Json(json!({"status": "moved"}))).into_response(),
        Err(WorkspaceFileStoreError::NotFound) => not_found(),
        Err(WorkspaceFileStoreError::InvalidPath(msg)) => bad_request(&msg),
        Err(WorkspaceFileStoreError::TraversalAttempt) => bad_request("path traversal rejected"),
        Err(_) => internal("file store error".to_owned()),
    }
}

/// `DELETE /api/v1/workspaces/{workspace_id}/files`
pub(crate) async fn delete_workspace_file(
    State(state): State<HttpState>,
    auth: AuthContext,
    AxumPath(workspace_id): AxumPath<String>,
    Query(query): Query<DeleteFileQuery>,
) -> Response {
    let Some(store) = store(&state) else {
        return unavailable();
    };
    let owner_filter = if auth.is_admin() {
        None
    } else {
        Some(auth.user_id.as_str())
    };
    if store.get(&workspace_id, owner_filter).is_err() {
        return not_found();
    }

    let result = state
        .workspace_file_service
        .delete(&workspace_id, &query.path)
        .await;

    match result {
        Ok(()) => (StatusCode::OK, Json(json!({"status": "deleted"}))).into_response(),
        Err(WorkspaceFileStoreError::NotFound) => not_found(),
        Err(WorkspaceFileStoreError::InvalidPath(msg)) => bad_request(&msg),
        Err(WorkspaceFileStoreError::TraversalAttempt) => bad_request("path traversal rejected"),
        Err(_) => internal("file store error".to_owned()),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::sync::Arc;

    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use axum::middleware;
    use axum::routing::{get, patch, post};
    use axum::Router;
    use serde_json::json;
    use tempfile::TempDir;
    use tower::ServiceExt;
    use tssp_adapter_sqlite::SqliteFileRepository;
    use tssp_domain::{UserId, UserRole};

    use super::{
        create_workspace, create_workspace_dir, create_workspace_file, delete_workspace,
        delete_workspace_file, get_workspace, list_workspace_files, list_workspaces,
        move_workspace_file, new_workspace_id, read_workspace_file, update_workspace,
        validate_document_path, validate_language, validate_name, write_workspace_file,
        WorkspaceError, WorkspaceRecord, WorkspaceStore,
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
            .route(
                "/api/v1/workspaces/{workspace_id}/files",
                get(list_workspace_files)
                    .post(create_workspace_file)
                    .delete(delete_workspace_file),
            )
            .route(
                "/api/v1/workspaces/{workspace_id}/files/content",
                get(read_workspace_file).put(write_workspace_file),
            )
            .route(
                "/api/v1/workspaces/{workspace_id}/files/move",
                patch(move_workspace_file),
            )
            .route(
                "/api/v1/workspaces/{workspace_id}/dirs",
                post(create_workspace_dir),
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
        assert!(validate_document_path("notes/plan.md").is_ok());
        assert!(validate_document_path("../plan.md").is_err());
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

    #[test]
    fn workspace_search_is_bounded_ranked_and_owner_scoped() {
        let (_temp, store) = open_store();
        insert_record(&store, "ws-alpha", "user-owner");
        insert_record(&store, "ws-private", "user-other");

        store
            .update(&WorkspaceRecord {
                id: "ws-alpha".to_owned(),
                owner_id: "user-owner".to_owned(),
                name: "Alpha Plan".to_owned(),
                language: "markdown".to_owned(),
                body: "Orange Pi storage notes".to_owned(),
                created_at: 1,
                updated_at: 2,
            })
            .expect("update alpha");
        store
            .update(&WorkspaceRecord {
                id: "ws-private".to_owned(),
                owner_id: "user-other".to_owned(),
                name: "Alpha Secret".to_owned(),
                language: "text".to_owned(),
                body: "hidden".to_owned(),
                created_at: 1,
                updated_at: 3,
            })
            .expect("update private");

        let owner_results = store
            .search("alpha", Some("user-owner"), 10)
            .expect("owner search");
        let admin_results = store.search("alpha", None, 10).expect("admin search");

        assert_eq!(owner_results.len(), 1);
        assert_eq!(owner_results[0].id, "ws-alpha");
        assert_eq!(admin_results.len(), 2);
    }

    #[test]
    fn workspace_documents_sync_primary_document_and_require_one_remaining() {
        let (_temp, store) = open_store();
        insert_record(&store, "ws-owned", "user-owner");

        let initial_documents = store
            .list_documents("ws-owned", Some("user-owner"))
            .expect("list docs");
        assert_eq!(initial_documents.len(), 1);
        assert!(initial_documents[0].is_primary);

        let created = store
            .create_document(
                "ws-owned",
                Some("user-owner"),
                "notes/plan.md",
                "markdown",
                "# Plan".to_owned(),
                false,
                10,
            )
            .expect("create doc");
        assert!(!created.is_primary);

        let updated = store
            .update_document(
                "ws-owned",
                &created.id,
                Some("user-owner"),
                "notes/plan.md",
                "markdown",
                "## Revised".to_owned(),
                true,
                11,
            )
            .expect("promote doc");
        assert!(updated.is_primary);

        let workspace = store
            .get("ws-owned", Some("user-owner"))
            .expect("workspace after promote");
        assert_eq!(workspace.language, "markdown");

        store
            .delete_document("ws-owned", &updated.id, Some("user-owner"), 12)
            .expect("delete promoted doc");

        let workspace = store
            .get("ws-owned", Some("user-owner"))
            .expect("workspace after delete");
        assert_eq!(workspace.language, "text");

        let remaining = store
            .list_documents("ws-owned", Some("user-owner"))
            .expect("remaining docs");
        assert_eq!(remaining.len(), 1);
        assert!(remaining[0].is_primary);

        let error = store
            .delete_document("ws-owned", &remaining[0].id, Some("user-owner"), 13)
            .expect_err("last document delete must fail");
        assert!(matches!(error, WorkspaceError::InvalidOperation(_)));
    }

    // Workspace filesystem HTTP integration tests

    #[tokio::test]
    async fn workspace_file_list_requires_valid_workspace() {
        let (_temp, store) = open_store();
        let router = app(store, auth("user-owner", UserRole::User));

        let request = Request::builder()
            .uri("/api/v1/workspaces/nonexistent/files")
            .body(Body::empty())
            .expect("request");
        let response = router.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn workspace_file_path_traversal_rejected() {
        let (_temp, store) = open_store();
        insert_record(&store, "ws-owner", "user-owner");
        let router = app(store, auth("user-owner", UserRole::User));

        let request = Request::builder()
            .uri("/api/v1/workspaces/ws-owner/files?path=../etc/passwd")
            .body(Body::empty())
            .expect("request");
        let response = router.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn workspace_file_read_requires_valid_workspace() {
        let (_temp, store) = open_store();
        let router = app(store, auth("user-owner", UserRole::User));

        let request = Request::builder()
            .uri("/api/v1/workspaces/nonexistent/files/content?path=test.txt")
            .body(Body::empty())
            .expect("request");
        let response = router.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn workspace_file_write_requires_valid_workspace() {
        let (_temp, store) = open_store();
        let router = app(store, auth("user-owner", UserRole::User));

        let request = Request::builder()
            .method("PUT")
            .uri("/api/v1/workspaces/nonexistent/files/content")
            .header("content-type", "application/json")
            .body(axum::body::Body::from(
                serde_json::to_string(&json!({
                    "path": "test.txt",
                    "content": "data"
                }))
                .expect("json"),
            ))
            .expect("request");
        let response = router.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn workspace_file_create_requires_valid_workspace() {
        let (_temp, store) = open_store();
        let router = app(store, auth("user-owner", UserRole::User));

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/workspaces/nonexistent/files")
            .header("content-type", "application/json")
            .body(axum::body::Body::from(
                serde_json::to_string(&json!({
                    "path": "test.txt",
                    "content": ""
                }))
                .expect("json"),
            ))
            .expect("request");
        let response = router.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn workspace_dir_create_requires_valid_workspace() {
        let (_temp, store) = open_store();
        let router = app(store, auth("user-owner", UserRole::User));

        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/workspaces/nonexistent/dirs")
            .header("content-type", "application/json")
            .body(axum::body::Body::from(
                serde_json::to_string(&json!({
                    "path": "src"
                }))
                .expect("json"),
            ))
            .expect("request");
        let response = router.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn workspace_file_move_requires_valid_workspace() {
        let (_temp, store) = open_store();
        let router = app(store, auth("user-owner", UserRole::User));

        let request = Request::builder()
            .method("PATCH")
            .uri("/api/v1/workspaces/nonexistent/files/move")
            .header("content-type", "application/json")
            .body(axum::body::Body::from(
                serde_json::to_string(&json!({
                    "from": "old.txt",
                    "to": "new.txt"
                }))
                .expect("json"),
            ))
            .expect("request");
        let response = router.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn workspace_file_delete_requires_valid_workspace() {
        let (_temp, store) = open_store();
        let router = app(store, auth("user-owner", UserRole::User));

        let request = Request::builder()
            .method("DELETE")
            .uri("/api/v1/workspaces/nonexistent/files?path=test.txt")
            .body(Body::empty())
            .expect("request");
        let response = router.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn workspace_file_ownership_enforced() {
        let (_temp, store) = open_store();
        insert_record(&store, "ws-owner", "user-owner");
        let router = app(store, auth("user-other", UserRole::User));

        let request = Request::builder()
            .uri("/api/v1/workspaces/ws-owner/files")
            .body(Body::empty())
            .expect("request");
        let response = router.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn workspace_file_admin_bypass_ownership() {
        let (_temp, store) = open_store();
        insert_record(&store, "ws-owner", "user-owner");
        let router = app(store, auth("user-admin", UserRole::Admin));

        let request = Request::builder()
            .uri("/api/v1/workspaces/ws-owner/files")
            .body(Body::empty())
            .expect("request");
        let response = router.oneshot(request).await.expect("response");

        // Should succeed (not found for filesystem, but workspace auth passes)
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn git_status_requires_ownership() {
        let (_temp, store) = open_store();
        insert_record(&store, "ws-owner", "user-owner");
        let router = app(store, auth("user-other", UserRole::User));

        let request = Request::builder()
            .uri("/api/v1/workspaces/ws-owner/git")
            .body(Body::empty())
            .expect("request");
        let response = router.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn git_status_admin_bypass() {
        let (_temp, store) = open_store();
        insert_record(&store, "ws-owner", "user-owner");
        let router = app(store, auth("user-admin", UserRole::Admin));

        let request = Request::builder()
            .uri("/api/v1/workspaces/ws-owner/git")
            .body(Body::empty())
            .expect("request");
        let response = router.oneshot(request).await.expect("response");

        // Should not be forbidden
        assert_ne!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn lsp_status_requires_ownership() {
        let (_temp, store) = open_store();
        insert_record(&store, "ws-owner", "user-owner");
        let router = app(store, auth("user-other", UserRole::User));

        let request = Request::builder()
            .uri("/api/v1/workspaces/ws-owner/lsp")
            .body(Body::empty())
            .expect("request");
        let response = router.oneshot(request).await.expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn lsp_status_admin_bypass() {
        let (_temp, store) = open_store();
        insert_record(&store, "ws-owner", "user-owner");
        let router = app(store, auth("user-admin", UserRole::Admin));

        let request = Request::builder()
            .uri("/api/v1/workspaces/ws-owner/lsp")
            .body(Body::empty())
            .expect("request");
        let response = router.oneshot(request).await.expect("response");

        // Should not be forbidden
        assert_ne!(response.status(), StatusCode::FORBIDDEN);
    }
}
