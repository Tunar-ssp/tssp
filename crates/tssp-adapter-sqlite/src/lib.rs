//! `SQLite` implementation of the TSSP metadata repository.
//!
//! The adapter owns schema creation, `SQLite` pragmas, and row mapping. All SQL is
//! kept behind the `FileRepository` port so application services stay storage
//! agnostic.

mod connection;
mod cursor;
mod migrations;
mod notes;
mod query;
mod sessions;

pub use sessions::SqliteSessionRepository;

use std::path::Path;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, params_from_iter, types::Value, Connection};
use tssp_domain::{ContentHash, FileId, FileName, FileRecord, Tag, TagKey, UnixTimestamp};
use tssp_ports::{
    DeletedFileRecord, FileRepository, ListQuery, NewFileRecord, PagedFiles, RepositoryError,
    RepositoryStats, TagMutationOutcome, TagSummary,
};

use cursor::{cursor_filter, encode_cursor, list_limit_plus_one, order_by_clause};
pub(crate) use migrations::{migration_applied, record_migration};
pub(crate) use query::map_domain_repository_error;
use query::{
    cleanup_orphaned_tags, count, ensure_file_exists, find_file_in_transaction, insert_file_row,
    insert_tags, load_tags, map_file_row, map_rusqlite_repository_error, normalize_folder_prefix,
    validate_list_limit,
};

/// `SQLite` metadata repository.
#[derive(Debug, Clone)]
pub struct SqliteFileRepository {
    pool: Pool<SqliteConnectionManager>,
}

impl SqliteFileRepository {
    /// Creates a repository from an existing connection pool.
    #[must_use]
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    /// Opens a `SQLite` database file, configures the pool, and runs embedded migrations.
    ///
    /// # Errors
    ///
    /// Returns [`SqliteRepositoryError`] when the database cannot be opened,
    /// configured, checked, or migrated.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, SqliteRepositoryError> {
        let manager = SqliteConnectionManager::file(path.as_ref());
        let pool = Pool::builder()
            .max_size(30)
            .build(manager)
            .map_err(|error| SqliteRepositoryError::Open(error.to_string()))?;

        let connection = pool
            .get()
            .map_err(|error| SqliteRepositoryError::Open(error.to_string()))?;

        initialize_connection(&connection)?;

        Ok(Self { pool })
    }

    /// Opens an in-memory `SQLite` database for tests.
    ///
    /// # Errors
    ///
    /// Returns [`SqliteRepositoryError`] when the database cannot be configured
    /// or migrated.
    pub fn open_in_memory() -> Result<Self, SqliteRepositoryError> {
        let manager = SqliteConnectionManager::memory();
        let pool = Pool::builder()
            .max_size(1) // Single connection for in-memory tests to maintain state
            .build(manager)
            .map_err(|error| SqliteRepositoryError::Open(error.to_string()))?;

        let connection = pool
            .get()
            .map_err(|error| SqliteRepositoryError::Open(error.to_string()))?;

        initialize_connection(&connection)?;

        Ok(Self { pool })
    }

    fn connect(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>, RepositoryError> {
        self.pool
            .get()
            .map_err(|error| RepositoryError::OperationFailed {
                message: format!("metadata connection pool failure: {error}"),
            })
    }
}

/// Configures and migrates an existing `SQLite` connection.
///
/// This is used by startup code that already owns the pool and only needs the schema
/// to be ready before repositories start executing queries.
///
/// # Errors
///
/// Returns [`SqliteRepositoryError`] if configuration, integrity checks, or migrations fail.
pub fn initialize_connection(connection: &Connection) -> Result<(), SqliteRepositoryError> {
    connection::configure_connection(connection)?;
    connection::run_integrity_check(connection)?;
    migrations::run_migrations(connection)?;
    Ok(())
}

impl FileRepository for SqliteFileRepository {
    fn insert_file(&self, new_file: NewFileRecord) -> Result<FileRecord, RepositoryError> {
        let inserted_id = new_file.id.clone();
        let mut connection = self.connect()?;
        let transaction = connection
            .transaction()
            .map_err(map_rusqlite_repository_error)?;
        insert_file_row(&transaction, &new_file)?;
        insert_tags(&transaction, &new_file)?;
        transaction
            .commit()
            .map_err(map_rusqlite_repository_error)?;
        drop(connection);

        self.find_file(&inserted_id)?
            .ok_or_else(|| RepositoryError::OperationFailed {
                message: "inserted file could not be read back".to_owned(),
            })
    }

    fn find_file(&self, id: &FileId) -> Result<Option<FileRecord>, RepositoryError> {
        let connection = self.connect()?;
        let mut statement = connection
            .prepare(
                "SELECT id, name, size_bytes, content_hash, mime_type, storage_handle, uploaded_at, pinned_at, folder_path, owner_id, visibility, public_token, public_expires_at
                 FROM files
                 WHERE id = ?1 AND deleted_at IS NULL",
            )
            .map_err(map_rusqlite_repository_error)?;
        let mut rows = statement
            .query(params![id.as_str()])
            .map_err(map_rusqlite_repository_error)?;
        let Some(row) = rows.next().map_err(map_rusqlite_repository_error)? else {
            return Ok(None);
        };

        let mut record = map_file_row(row)?;
        record.tags = load_tags(&connection, id)?;
        Ok(Some(record))
    }

    fn find_file_by_content_hash(
        &self,
        content_hash: &ContentHash,
    ) -> Result<Option<FileRecord>, RepositoryError> {
        let connection = self.connect()?;
        let mut statement = connection
            .prepare(
                "SELECT id, name, size_bytes, content_hash, mime_type, storage_handle, uploaded_at, pinned_at, folder_path, owner_id, visibility, public_token, public_expires_at
                 FROM files
                 WHERE content_hash = ?1 AND deleted_at IS NULL
                 ORDER BY uploaded_at ASC, id ASC
                 LIMIT 1",
            )
            .map_err(map_rusqlite_repository_error)?;
        let mut rows = statement
            .query(params![content_hash.as_str()])
            .map_err(map_rusqlite_repository_error)?;
        let Some(row) = rows.next().map_err(map_rusqlite_repository_error)? else {
            return Ok(None);
        };

        let mut record = map_file_row(row)?;
        let id = record.id.clone();
        record.tags = load_tags(&connection, &id)?;
        Ok(Some(record))
    }

    fn delete_file(&self, id: &FileId) -> Result<Option<DeletedFileRecord>, RepositoryError> {
        let mut connection = self.connect()?;
        let transaction = connection
            .transaction()
            .map_err(map_rusqlite_repository_error)?;

        let Some(mut record) = find_file_in_transaction(&transaction, id)? else {
            transaction
                .commit()
                .map_err(map_rusqlite_repository_error)?;
            return Ok(None);
        };
        record.tags = load_tags(&transaction, id)?;

        transaction
            .execute(
                "UPDATE files SET deleted_at = CAST(strftime('%s', 'now') AS INTEGER) WHERE id = ?1",
                params![id.as_str()],
            )
            .map_err(map_rusqlite_repository_error)?;
        cleanup_orphaned_tags(&transaction)?;
        let remaining_content_references = count(
            &transaction,
            "SELECT COUNT(*) FROM files WHERE content_hash = ?1 AND deleted_at IS NULL",
            params![record.content_hash.as_str()],
        )?;
        transaction
            .commit()
            .map_err(map_rusqlite_repository_error)?;

        Ok(Some(DeletedFileRecord {
            record,
            remaining_content_references,
        }))
    }

    fn restore_file(&self, id: &FileId) -> Result<Option<FileRecord>, RepositoryError> {
        let mut connection = self.connect()?;
        let transaction = connection
            .transaction()
            .map_err(map_rusqlite_repository_error)?;

        let record = {
            let mut statement = transaction
                .prepare(
                    "SELECT id, name, size_bytes, content_hash, mime_type, storage_handle, uploaded_at, pinned_at, folder_path, owner_id, visibility, public_token, public_expires_at
                     FROM files WHERE id = ?1",
                )
                .map_err(map_rusqlite_repository_error)?;
            let mut rows = statement
                .query([id.as_str()])
                .map_err(map_rusqlite_repository_error)?;
            rows.next()
                .map_err(map_rusqlite_repository_error)?
                .map(|row| map_file_row(row))
                .transpose()?
        };

        match record {
            None => {
                transaction
                    .commit()
                    .map_err(map_rusqlite_repository_error)?;
                Ok(None)
            }
            Some(mut record) => {
                transaction
                    .execute(
                        "UPDATE files SET deleted_at = NULL WHERE id = ?1",
                        params![id.as_str()],
                    )
                    .map_err(map_rusqlite_repository_error)?;

                record.tags = load_tags(&transaction, id)?;
                transaction
                    .commit()
                    .map_err(map_rusqlite_repository_error)?;

                Ok(Some(record))
            }
        }
    }

    fn list_deleted_files(
        &self,
        older_than: UnixTimestamp,
    ) -> Result<Vec<FileRecord>, RepositoryError> {
        let connection = self.connect()?;
        let mut statement = connection
            .prepare(
                "SELECT id, name, size_bytes, content_hash, mime_type, storage_handle, uploaded_at, pinned_at, folder_path, owner_id, visibility, public_token, public_expires_at
                 FROM files WHERE deleted_at IS NOT NULL AND deleted_at < ?1 ORDER BY deleted_at ASC"
            )
            .map_err(map_rusqlite_repository_error)?;

        let mut rows = statement
            .query(params![older_than.seconds()])
            .map_err(map_rusqlite_repository_error)?;

        let mut records = Vec::new();
        while let Some(row) = rows.next().map_err(map_rusqlite_repository_error)? {
            records.push(map_file_row(row)?);
        }

        Ok(records)
    }

    fn purge_deleted_file(&self, id: &FileId) -> Result<bool, RepositoryError> {
        let mut connection = self.connect()?;
        let transaction = connection
            .transaction()
            .map_err(map_rusqlite_repository_error)?;

        let count = transaction
            .execute(
                "DELETE FROM files WHERE id = ?1 AND deleted_at IS NOT NULL",
                params![id.as_str()],
            )
            .map_err(map_rusqlite_repository_error)?;

        cleanup_orphaned_tags(&transaction)?;
        transaction
            .commit()
            .map_err(map_rusqlite_repository_error)?;

        Ok(count > 0)
    }

    #[allow(clippy::too_many_lines)]
    fn list_files(&self, query: &ListQuery) -> Result<PagedFiles, RepositoryError> {
        let page_limit = validate_list_limit(query.limit)?;

        let connection = self.connect()?;
        let mut sql = String::from(
            "SELECT f.id, f.name, f.size_bytes, f.content_hash, f.mime_type, f.storage_handle, f.uploaded_at, f.pinned_at, f.folder_path, f.owner_id, f.visibility, f.public_token, f.public_expires_at
             FROM files f",
        );
        let mut where_clauses = Vec::new();
        let mut parameters = Vec::<Value>::new();

        where_clauses.push("f.deleted_at IS NULL".to_owned());

        for (index, tag) in query.tags.iter().enumerate() {
            where_clauses.push(format!(
                "EXISTS (
                    SELECT 1
                    FROM file_tags ft{index}
                    WHERE ft{index}.file_id = f.id
                      AND ft{index}.tag_key = ?
                )"
            ));
            parameters.push(Value::from(tag.as_str().to_owned()));
        }

        if let Some(mime_prefix) = &query.mime_prefix {
            where_clauses.push("f.mime_type LIKE ?".to_owned());
            parameters.push(Value::from(format!("{mime_prefix}%")));
        }

        if let Some(name_substring) = &query.name_substring {
            where_clauses.push("instr(f.name, ?) > 0".to_owned());
            parameters.push(Value::from(name_substring.clone()));
        }

        if let Some(since) = query.since {
            where_clauses.push("f.uploaded_at >= ?".to_owned());
            parameters.push(Value::from(since.seconds()));
        }

        if let Some(until) = query.until {
            where_clauses.push("f.uploaded_at <= ?".to_owned());
            parameters.push(Value::from(until.seconds()));
        }

        if query.pinned_only {
            where_clauses.push("f.pinned_at IS NOT NULL".to_owned());
        }

        if let Some(folder_prefix) = &query.folder_prefix {
            let prefix = folder_prefix.trim().trim_matches('/');
            if prefix.is_empty() {
                where_clauses.push("f.folder_path = ''".to_owned());
            } else {
                where_clauses
                    .push("(f.folder_path = ? OR f.folder_path LIKE ? ESCAPE '\\')".to_owned());
                parameters.push(Value::from(prefix.to_owned()));
                parameters.push(Value::from(format!("{prefix}/%")));
            }
        }

        if let Some(visibility) = query.visibility {
            where_clauses.push("f.visibility = ?".to_owned());
            parameters.push(Value::from(visibility.as_str().to_owned()));
        }

        if let Some(owner_id) = &query.owner_id {
            where_clauses.push("f.owner_id = ?".to_owned());
            parameters.push(Value::from(owner_id.as_str().to_owned()));
        }

        if let Some(cursor_filter) = cursor_filter(query.sort, query.after_cursor.as_ref())? {
            where_clauses.push(cursor_filter.clause);
            parameters.extend(cursor_filter.parameters);
        }

        if !where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clauses.join(" AND "));
        }

        sql.push_str(" ORDER BY ");
        sql.push_str(order_by_clause(query.sort));
        sql.push_str(" LIMIT ?");
        parameters.push(Value::from(list_limit_plus_one(query.limit)?));

        let mut statement = connection
            .prepare(&sql)
            .map_err(map_rusqlite_repository_error)?;
        let mut rows = statement
            .query(params_from_iter(parameters.iter()))
            .map_err(map_rusqlite_repository_error)?;

        let mut records = Vec::new();
        while let Some(row) = rows.next().map_err(map_rusqlite_repository_error)? {
            let mut record = map_file_row(row)?;
            let id = record.id.clone();
            record.tags = load_tags(&connection, &id)?;
            records.push(record);
        }

        let has_more = records.len() > page_limit;
        if has_more {
            records.truncate(page_limit);
        }
        let next_cursor = if has_more {
            let record = records
                .last()
                .ok_or_else(|| RepositoryError::OperationFailed {
                    message: "list pagination state became empty".to_owned(),
                })?;
            Some(encode_cursor(query.sort, record)?)
        } else {
            None
        };

        Ok(PagedFiles {
            files: records,
            next_cursor,
        })
    }

    fn list_files_recent(&self, limit: u64) -> Result<Vec<FileRecord>, RepositoryError> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        Ok(self
            .list_files(&ListQuery {
                limit,
                ..ListQuery::default()
            })?
            .files)
    }

    fn list_files_by_tag(
        &self,
        tag: &tssp_domain::TagKey,
        limit: u64,
    ) -> Result<Vec<FileRecord>, RepositoryError> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        Ok(self
            .list_files(&ListQuery {
                limit,
                tags: vec![tag.clone()],
                ..ListQuery::default()
            })?
            .files)
    }

    fn list_tags(&self) -> Result<Vec<TagSummary>, RepositoryError> {
        let connection = self.connect()?;
        let mut statement = connection
            .prepare(
                "SELECT tags.display, COUNT(file_tags.file_id) AS file_count
                 FROM tags
                 JOIN file_tags ON file_tags.tag_key = tags.key
                 GROUP BY tags.key, tags.display
                 ORDER BY tags.key ASC",
            )
            .map_err(map_rusqlite_repository_error)?;
        let mut rows = statement.query([]).map_err(map_rusqlite_repository_error)?;

        let mut tags = Vec::new();
        while let Some(row) = rows.next().map_err(map_rusqlite_repository_error)? {
            let display: String = row.get(0).map_err(map_rusqlite_repository_error)?;
            let file_count: i64 = row.get(1).map_err(map_rusqlite_repository_error)?;
            tags.push(TagSummary {
                tag: Tag::new(display).map_err(|error| map_domain_repository_error(&error))?,
                file_count: u64::try_from(file_count).map_err(|error| {
                    RepositoryError::OperationFailed {
                        message: format!("stored tag count is invalid: {error}"),
                    }
                })?,
            });
        }
        Ok(tags)
    }

    fn add_tags_to_file(
        &self,
        id: &FileId,
        tags: &[Tag],
    ) -> Result<TagMutationOutcome, RepositoryError> {
        let mut connection = self.connect()?;
        let transaction = connection
            .transaction()
            .map_err(map_rusqlite_repository_error)?;
        ensure_file_exists(&transaction, id)?;

        let mut changed_count = 0_u64;
        for tag in tags {
            transaction
                .execute(
                    "INSERT OR IGNORE INTO tags (key, display) VALUES (?1, ?2)",
                    params![tag.key().as_str(), tag.display()],
                )
                .map_err(map_rusqlite_repository_error)?;
            let changed = transaction
                .execute(
                    "INSERT OR IGNORE INTO file_tags (file_id, tag_key) VALUES (?1, ?2)",
                    params![id.as_str(), tag.key().as_str()],
                )
                .map_err(map_rusqlite_repository_error)?;
            changed_count = changed_count
                .checked_add(u64::try_from(changed).unwrap_or(u64::MAX))
                .ok_or_else(|| RepositoryError::OperationFailed {
                    message: "tag mutation count overflow".to_owned(),
                })?;
        }
        transaction
            .commit()
            .map_err(map_rusqlite_repository_error)?;

        Ok(TagMutationOutcome { changed_count })
    }

    fn remove_tag_from_file(
        &self,
        id: &FileId,
        tag: &TagKey,
    ) -> Result<TagMutationOutcome, RepositoryError> {
        let mut connection = self.connect()?;
        let transaction = connection
            .transaction()
            .map_err(map_rusqlite_repository_error)?;
        ensure_file_exists(&transaction, id)?;
        let changed = transaction
            .execute(
                "DELETE FROM file_tags WHERE file_id = ?1 AND tag_key = ?2",
                params![id.as_str(), tag.as_str()],
            )
            .map_err(map_rusqlite_repository_error)?;
        cleanup_orphaned_tags(&transaction)?;
        transaction
            .commit()
            .map_err(map_rusqlite_repository_error)?;

        Ok(TagMutationOutcome {
            changed_count: u64::try_from(changed).map_err(|error| {
                RepositoryError::OperationFailed {
                    message: format!("tag mutation count is invalid: {error}"),
                }
            })?,
        })
    }

    fn stats_since(&self, recent_since: UnixTimestamp) -> Result<RepositoryStats, RepositoryError> {
        let connection = self.connect()?;
        let storage_bytes_used: u64 = connection
            .query_row(
                "SELECT COALESCE(SUM(size_bytes), 0) FROM files WHERE deleted_at IS NULL",
                [],
                |row| row.get(0),
            )
            .map_err(map_rusqlite_repository_error)?;
        Ok(RepositoryStats {
            file_count: count(
                &connection,
                "SELECT COUNT(*) FROM files WHERE deleted_at IS NULL",
                [],
            )?,
            note_count: count(
                &connection,
                "SELECT COUNT(*) FROM notes WHERE deleted_at IS NULL",
                [],
            )?,
            tag_count: count(&connection, "SELECT COUNT(*) FROM tags", [])?,
            pinned_count: count(
                &connection,
                "SELECT COUNT(*) FROM files WHERE pinned_at IS NOT NULL AND deleted_at IS NULL",
                [],
            )?,
            recent_upload_count: count(
                &connection,
                "SELECT COUNT(*) FROM files WHERE uploaded_at >= ?1 AND deleted_at IS NULL",
                params![recent_since.seconds()],
            )?,
            recent_note_count: count(
                &connection,
                "SELECT COUNT(*) FROM notes WHERE updated_at >= ?1 AND deleted_at IS NULL",
                params![recent_since.seconds()],
            )?,
            storage_bytes_used,
        })
    }

    fn pin_file(
        &self,
        id: &FileId,
        position: Option<u32>,
    ) -> Result<tssp_ports::PinOutcome, RepositoryError> {
        let mut connection = self.connect()?;
        let transaction = connection
            .transaction()
            .map_err(map_rusqlite_repository_error)?;
        ensure_file_exists(&transaction, id)?;

        let current_position: Option<i64> = transaction
            .query_row(
                "SELECT pinned_at FROM files WHERE id = ?1",
                params![id.as_str()],
                |row| row.get(0),
            )
            .map_err(map_rusqlite_repository_error)?;
        let already_pinned = current_position.is_some();

        let pin_position = position.map_or_else(
            || {
                current_position.unwrap_or_else(|| {
                    let max: Option<i64> = transaction
                        .query_row(
                            "SELECT MAX(pinned_at) FROM files WHERE pinned_at IS NOT NULL",
                            [],
                            |row| row.get(0),
                        )
                        .unwrap_or(None);
                    max.map_or(1_i64, |v| v.saturating_add(1))
                })
            },
            i64::from,
        );

        if let Some(current) = current_position {
            if pin_position < current {
                transaction
                    .execute(
                        "UPDATE files
                         SET pinned_at = pinned_at + 1
                         WHERE pinned_at IS NOT NULL
                           AND pinned_at >= ?1
                           AND pinned_at < ?2
                           AND id <> ?3",
                        params![pin_position, current, id.as_str()],
                    )
                    .map_err(map_rusqlite_repository_error)?;
            } else if pin_position > current {
                transaction
                    .execute(
                        "UPDATE files
                         SET pinned_at = pinned_at - 1
                         WHERE pinned_at IS NOT NULL
                           AND pinned_at > ?1
                           AND pinned_at <= ?2
                           AND id <> ?3",
                        params![current, pin_position, id.as_str()],
                    )
                    .map_err(map_rusqlite_repository_error)?;
            }
        } else if position.is_some() {
            transaction
                .execute(
                    "UPDATE files
                     SET pinned_at = pinned_at + 1
                     WHERE pinned_at IS NOT NULL
                       AND pinned_at >= ?1",
                    params![pin_position],
                )
                .map_err(map_rusqlite_repository_error)?;
        }

        transaction
            .execute(
                "UPDATE files SET pinned_at = ?1 WHERE id = ?2",
                params![pin_position, id.as_str()],
            )
            .map_err(map_rusqlite_repository_error)?;
        transaction
            .commit()
            .map_err(map_rusqlite_repository_error)?;

        Ok(tssp_ports::PinOutcome {
            existed: true,
            changed: !already_pinned,
        })
    }

    fn unpin_file(&self, id: &FileId) -> Result<tssp_ports::PinOutcome, RepositoryError> {
        let mut connection = self.connect()?;
        let transaction = connection
            .transaction()
            .map_err(map_rusqlite_repository_error)?;
        ensure_file_exists(&transaction, id)?;

        let current_position: Option<i64> = transaction
            .query_row(
                "SELECT pinned_at FROM files WHERE id = ?1",
                params![id.as_str()],
                |row| row.get(0),
            )
            .map_err(map_rusqlite_repository_error)?;
        let already_pinned = current_position.is_some();

        if let Some(position) = current_position {
            transaction
                .execute(
                    "UPDATE files
                     SET pinned_at = pinned_at - 1
                     WHERE pinned_at IS NOT NULL
                       AND pinned_at > ?1",
                    params![position],
                )
                .map_err(map_rusqlite_repository_error)?;
        }

        transaction
            .execute(
                "UPDATE files SET pinned_at = NULL WHERE id = ?1",
                params![id.as_str()],
            )
            .map_err(map_rusqlite_repository_error)?;
        transaction
            .commit()
            .map_err(map_rusqlite_repository_error)?;

        Ok(tssp_ports::PinOutcome {
            existed: true,
            changed: already_pinned,
        })
    }

    fn list_pinned_files(&self) -> Result<Vec<FileRecord>, RepositoryError> {
        let connection = self.connect()?;
        let mut statement = connection
            .prepare(
                "SELECT id, name, size_bytes, content_hash, mime_type, storage_handle, uploaded_at, pinned_at, folder_path, owner_id, visibility, public_token, public_expires_at
                 FROM files
                 WHERE pinned_at IS NOT NULL AND deleted_at IS NULL
                 ORDER BY pinned_at ASC, id ASC",
            )
            .map_err(map_rusqlite_repository_error)?;
        let mut rows = statement.query([]).map_err(map_rusqlite_repository_error)?;
        let mut records = Vec::new();
        while let Some(row) = rows.next().map_err(map_rusqlite_repository_error)? {
            let mut record = map_file_row(row)?;
            let id = record.id.clone();
            record.tags = load_tags(&connection, &id)?;
            records.push(record);
        }
        Ok(records)
    }

    fn reorder_pins(&self, ordered_ids: &[FileId]) -> Result<(), RepositoryError> {
        let mut connection = self.connect()?;
        let transaction = connection
            .transaction()
            .map_err(map_rusqlite_repository_error)?;
        for (index, id) in ordered_ids.iter().enumerate() {
            let position = u32::try_from(index.saturating_add(1)).map_err(|error| {
                RepositoryError::OperationFailed {
                    message: format!("pin position overflow: {error}"),
                }
            })?;
            let changed = transaction
                .execute(
                    "UPDATE files SET pinned_at = ?1 WHERE id = ?2 AND pinned_at IS NOT NULL",
                    params![position, id.as_str()],
                )
                .map_err(map_rusqlite_repository_error)?;
            if changed == 0 {
                return Err(RepositoryError::NotFound);
            }
        }
        transaction
            .commit()
            .map_err(map_rusqlite_repository_error)?;
        Ok(())
    }

    fn search_files(&self, query: &str) -> Result<Vec<FileRecord>, RepositoryError> {
        let connection = self.connect()?;
        let mut statement = connection
            .prepare(
                "SELECT f.id, f.name, f.size_bytes, f.content_hash, f.mime_type, f.storage_handle, f.uploaded_at, f.pinned_at, f.folder_path, f.owner_id, f.visibility, f.public_token, f.public_expires_at
                 FROM files f
                 WHERE f.id IN (SELECT file_id FROM file_search WHERE file_search MATCH ?1)
                 AND f.deleted_at IS NULL
                 LIMIT 100",
            )
            .map_err(map_rusqlite_repository_error)?;

        let mut rows = statement
            .query(params![query])
            .map_err(map_rusqlite_repository_error)?;

        let mut records = Vec::new();
        while let Some(row) = rows.next().map_err(map_rusqlite_repository_error)? {
            let mut record = map_file_row(row)?;
            let id = record.id.clone();
            record.tags = load_tags(&connection, &id)?;
            records.push(record);
        }
        Ok(records)
    }

    fn rename_file(
        &self,
        id: &FileId,
        new_name: &FileName,
    ) -> Result<Option<FileRecord>, RepositoryError> {
        let mut connection = self.connect()?;
        let transaction = connection
            .transaction()
            .map_err(map_rusqlite_repository_error)?;
        ensure_file_exists(&transaction, id)?;

        transaction
            .execute(
                "UPDATE files SET name = ?1 WHERE id = ?2",
                params![new_name.original(), id.as_str()],
            )
            .map_err(map_rusqlite_repository_error)?;
        transaction
            .commit()
            .map_err(map_rusqlite_repository_error)?;

        self.find_file(id)?
            .ok_or_else(|| RepositoryError::OperationFailed {
                message: "renamed file could not be read back".to_owned(),
            })
            .map(Some)
    }

    fn list_folder_counts(
        &self,
        owner_id: Option<&tssp_domain::UserId>,
    ) -> Result<Vec<(String, u64)>, RepositoryError> {
        let connection = self.connect()?;
        let mut sql = String::from(
            "SELECT folder_path, COUNT(*)
             FROM files
             WHERE deleted_at IS NULL",
        );
        let mut parameters = Vec::<Value>::new();

        if let Some(owner) = owner_id {
            sql.push_str(" AND owner_id = ?");
            parameters.push(Value::from(owner.as_str().to_owned()));
        }

        sql.push_str(" GROUP BY folder_path ORDER BY folder_path");

        let mut statement = connection
            .prepare(&sql)
            .map_err(map_rusqlite_repository_error)?;
        let mut rows = statement
            .query(params_from_iter(parameters.iter()))
            .map_err(map_rusqlite_repository_error)?;
        let mut counts = Vec::new();
        while let Some(row) = rows.next().map_err(map_rusqlite_repository_error)? {
            let path: String = row.get(0).map_err(map_rusqlite_repository_error)?;
            let count: i64 = row.get(1).map_err(map_rusqlite_repository_error)?;
            let count = u64::try_from(count).map_err(|error| RepositoryError::OperationFailed {
                message: format!("folder count is invalid: {error}"),
            })?;
            counts.push((path, count));
        }
        Ok(counts)
    }

    fn set_file_visibility(
        &self,
        id: &FileId,
        visibility: tssp_domain::Visibility,
        public_token: Option<&str>,
    ) -> Result<Option<FileRecord>, RepositoryError> {
        let connection = self.connect()?;
        let changed = connection
            .execute(
                "UPDATE files SET visibility = ?1, public_token = ?2 WHERE id = ?3",
                params![visibility.as_str(), public_token, id.as_str()],
            )
            .map_err(map_rusqlite_repository_error)?;
        if changed == 0 {
            return Ok(None);
        }
        self.find_file(id)
    }

    fn find_file_by_public_token(
        &self,
        token: &str,
    ) -> Result<Option<FileRecord>, RepositoryError> {
        let connection = self.connect()?;
        let mut statement = connection
            .prepare(
                "SELECT id, name, size_bytes, content_hash, mime_type, storage_handle, uploaded_at, pinned_at, folder_path, owner_id, visibility, public_token, public_expires_at
                 FROM files
                 WHERE public_token = ?1 AND visibility = 'public' AND deleted_at IS NULL
                 LIMIT 1",
            )
            .map_err(map_rusqlite_repository_error)?;
        let mut rows = statement
            .query(params![token])
            .map_err(map_rusqlite_repository_error)?;
        let Some(row) = rows.next().map_err(map_rusqlite_repository_error)? else {
            return Ok(None);
        };
        let mut record = map_file_row(row)?;
        record.tags = load_tags(&connection, &record.id)?;
        Ok(Some(record))
    }

    fn update_folder_path_prefix(
        &self,
        from_prefix: &str,
        to_prefix: &str,
    ) -> Result<u64, RepositoryError> {
        let from = normalize_folder_prefix(from_prefix);
        let to = normalize_folder_prefix(to_prefix);
        let connection = self.connect()?;
        let changed = if from.is_empty() {
            connection
                .execute(
                    "UPDATE files
                     SET folder_path = CASE
                         WHEN folder_path = '' THEN ?1
                         ELSE ?1 || '/' || folder_path
                     END
                     WHERE folder_path = '' OR folder_path LIKE ?2 ESCAPE '\\'",
                    params![to, format!("{from}/%")],
                )
                .map_err(map_rusqlite_repository_error)?
        } else {
            connection
                .execute(
                    "UPDATE files
                     SET folder_path = CASE
                         WHEN folder_path = ?1 THEN ?2
                         WHEN folder_path LIKE ?3 ESCAPE '\\' THEN
                             CASE
                                 WHEN length(folder_path) > length(?1) + 1 AND ?2 != ''
                                 THEN ?2 || '/' || substr(folder_path, length(?1) + 2)
                                 WHEN length(folder_path) > length(?1) + 1 AND ?2 = ''
                                 THEN substr(folder_path, length(?1) + 2)
                                 ELSE ?2
                             END
                         ELSE folder_path
                     END
                     WHERE folder_path = ?1 OR folder_path LIKE ?3 ESCAPE '\\'",
                    params![from, to, format!("{from}/%")],
                )
                .map_err(map_rusqlite_repository_error)?
        };
        Ok(u64::try_from(changed).unwrap_or(0))
    }

    fn set_file_folder_path(
        &self,
        id: &FileId,
        folder_path: &str,
    ) -> Result<Option<FileRecord>, RepositoryError> {
        let normalized = normalize_folder_prefix(folder_path);
        let connection = self.connect()?;
        let changed = connection
            .execute(
                "UPDATE files SET folder_path = ?1 WHERE id = ?2",
                params![normalized, id.as_str()],
            )
            .map_err(map_rusqlite_repository_error)?;
        if changed == 0 {
            return Ok(None);
        }
        self.find_file(id)
    }

    fn insert_audit_event(
        &self,
        id: &str,
        timestamp: i64,
        user_id: Option<&str>,
        action: &str,
        resource: Option<&str>,
        resource_id: Option<&str>,
        status: &str,
        details: Option<&str>,
    ) -> Result<(), RepositoryError> {
        let connection = self.connect()?;
        connection
            .execute(
                "INSERT INTO audit_events (id, timestamp, user_id, action, resource, resource_id, status, details)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![id, timestamp, user_id, action, resource, resource_id, status, details],
            )
            .map_err(map_rusqlite_repository_error)?;
        Ok(())
    }
}

/// Error type returned by low-level `SQLite` operations.
#[derive(Debug, thiserror::Error)]
pub enum SqliteRepositoryError {
    /// `SQLite` open failed.
    #[error("could not open sqlite database: {0}")]
    Open(String),

    /// `SQLite` setup failed.
    #[error("could not configure sqlite database: {0}")]
    Configure(rusqlite::Error),

    /// Database integrity check failed.
    #[error("sqlite integrity check failed: {message}")]
    Integrity {
        /// Integrity failure message.
        message: String,
    },

    /// Migration failed.
    #[error("could not migrate sqlite database: {0}")]
    Migration(rusqlite::Error),
}

#[cfg(test)]
mod folder_tests;
#[cfg(test)]
mod repository_tests;
