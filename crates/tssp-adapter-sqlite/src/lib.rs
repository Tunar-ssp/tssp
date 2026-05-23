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
use std::sync::{Mutex, MutexGuard};

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
#[derive(Debug)]
pub struct SqliteFileRepository {
    connection: Mutex<Connection>,
}

impl SqliteFileRepository {
    /// Opens a `SQLite` database file and runs embedded migrations.
    ///
    /// # Errors
    ///
    /// Returns [`SqliteRepositoryError`] when the database cannot be opened,
    /// configured, checked, or migrated.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, SqliteRepositoryError> {
        let connection = Connection::open(path).map_err(SqliteRepositoryError::Open)?;
        connection::configure_connection(&connection)?;
        connection::run_integrity_check(&connection)?;
        migrations::run_migrations(&connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    /// Opens an in-memory `SQLite` database for tests.
    ///
    /// # Errors
    ///
    /// Returns [`SqliteRepositoryError`] when the database cannot be configured
    /// or migrated.
    pub fn open_in_memory() -> Result<Self, SqliteRepositoryError> {
        let connection = Connection::open_in_memory().map_err(SqliteRepositoryError::Open)?;
        connection::configure_connection(&connection)?;
        connection::run_integrity_check(&connection)?;
        migrations::run_migrations(&connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, RepositoryError> {
        self.connection
            .lock()
            .map_err(|error| RepositoryError::OperationFailed {
                message: format!("metadata connection lock is poisoned: {error}"),
            })
    }
}

impl FileRepository for SqliteFileRepository {
    fn insert_file(&self, new_file: NewFileRecord) -> Result<FileRecord, RepositoryError> {
        let inserted_id = new_file.id.clone();
        let mut connection = self.lock()?;
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
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT id, name, size_bytes, content_hash, mime_type, storage_handle, uploaded_at, pinned_at, folder_path, owner_id, visibility, public_token
                 FROM files
                 WHERE id = ?1",
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
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT id, name, size_bytes, content_hash, mime_type, storage_handle, uploaded_at, pinned_at, folder_path, owner_id, visibility, public_token
                 FROM files
                 WHERE content_hash = ?1
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
        let mut connection = self.lock()?;
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
            .execute("DELETE FROM files WHERE id = ?1", params![id.as_str()])
            .map_err(map_rusqlite_repository_error)?;
        cleanup_orphaned_tags(&transaction)?;
        let remaining_content_references = count(
            &transaction,
            "SELECT COUNT(*) FROM files WHERE content_hash = ?1",
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

    fn list_files(&self, query: &ListQuery) -> Result<PagedFiles, RepositoryError> {
        let page_limit = validate_list_limit(query.limit)?;

        let connection = self.lock()?;
        let mut sql = String::from(
            "SELECT f.id, f.name, f.size_bytes, f.content_hash, f.mime_type, f.storage_handle, f.uploaded_at, f.pinned_at, f.folder_path, f.owner_id, f.visibility, f.public_token
             FROM files f",
        );
        let mut where_clauses = Vec::new();
        let mut parameters = Vec::<Value>::new();

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
        let connection = self.lock()?;
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
        let mut connection = self.lock()?;
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
        let mut connection = self.lock()?;
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
        let connection = self.lock()?;
        Ok(RepositoryStats {
            file_count: count(&connection, "SELECT COUNT(*) FROM files", [])?,
            note_count: count(&connection, "SELECT COUNT(*) FROM notes", [])?,
            tag_count: count(&connection, "SELECT COUNT(*) FROM tags", [])?,
            pinned_count: count(
                &connection,
                "SELECT COUNT(*) FROM files WHERE pinned_at IS NOT NULL",
                [],
            )?,
            recent_upload_count: count(
                &connection,
                "SELECT COUNT(*) FROM files WHERE uploaded_at >= ?1",
                params![recent_since.seconds()],
            )?,
            recent_note_count: count(
                &connection,
                "SELECT COUNT(*) FROM notes WHERE updated_at >= ?1",
                params![recent_since.seconds()],
            )?,
        })
    }

    fn pin_file(
        &self,
        id: &FileId,
        position: Option<u32>,
    ) -> Result<tssp_ports::PinOutcome, RepositoryError> {
        let mut connection = self.lock()?;
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
        let mut connection = self.lock()?;
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
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT id, name, size_bytes, content_hash, mime_type, storage_handle, uploaded_at, pinned_at, folder_path, owner_id, visibility, public_token
                 FROM files
                 WHERE pinned_at IS NOT NULL
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
        let mut connection = self.lock()?;
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
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT f.id, f.name, f.size_bytes, f.content_hash, f.mime_type, f.storage_handle, f.uploaded_at, f.pinned_at, f.folder_path, f.owner_id, f.visibility, f.public_token
                 FROM file_search s
                 JOIN files f ON f.id = s.file_id
                 WHERE file_search MATCH ?1
                 ORDER BY rank
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
        let mut connection = self.lock()?;
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

    fn list_folder_counts(&self) -> Result<Vec<(String, u64)>, RepositoryError> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT folder_path, COUNT(*)
                 FROM files
                 GROUP BY folder_path
                 ORDER BY folder_path",
            )
            .map_err(map_rusqlite_repository_error)?;
        let mut rows = statement.query([]).map_err(map_rusqlite_repository_error)?;
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
        let connection = self.lock()?;
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
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT id, name, size_bytes, content_hash, mime_type, storage_handle, uploaded_at, pinned_at, folder_path, owner_id, visibility, public_token
                 FROM files
                 WHERE public_token = ?1 AND visibility = 'public'
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
        let connection = self.lock()?;
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
                         WHEN folder_path LIKE ?3 ESCAPE '\\' THEN ?2 || substr(folder_path, length(?1) + 2)
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
        let connection = self.lock()?;
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
}

/// Error type returned by low-level `SQLite` operations.
#[derive(Debug, thiserror::Error)]
pub enum SqliteRepositoryError {
    /// `SQLite` open failed.
    #[error("could not open sqlite database: {0}")]
    Open(rusqlite::Error),

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
mod tests {
    use tempfile::tempdir;
    use tssp_domain::{
        ContentHash, Cursor, FileId, FileName, FileSize, MimeType, StorageHandle, Tag, TagKey,
        UnixTimestamp,
    };
    use tssp_ports::{
        FileRepository, ListQuery, ListSort, NewFileRecord, NoteRepository, RepositoryError,
        SearchHit,
    };

    use super::SqliteFileRepository;

    #[test]
    fn open_file_database_runs_migrations() {
        let temp = tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let path = temp.path().join("metadata.sqlite3");

        let repository = SqliteFileRepository::open(path);

        assert!(repository.is_ok());
    }

    #[test]
    fn insert_and_find_file_roundtrips_metadata_and_tags() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));
        let file = new_file("file-1", &["Docs", "family photos"], 1_700_000_000);

        let inserted = repository
            .insert_file(file)
            .unwrap_or_else(|error| panic!("insert failed: {error}"));
        let found = repository
            .find_file(&inserted.id)
            .unwrap_or_else(|error| panic!("find failed: {error}"));

        assert!(matches!(
            found,
            Some(record) if record.id.as_str() == "file-1"
                && record.name.original() == "report.pdf"
                && record.tags.len() == 2
                && record.pinned_at == Some(2)
        ));
    }

    #[test]
    fn duplicate_file_id_returns_conflict() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));
        repository
            .insert_file(new_file("file-1", &[], 1_700_000_000))
            .unwrap_or_else(|error| panic!("insert failed: {error}"));

        let duplicate = repository.insert_file(new_file("file-1", &[], 1_700_000_000));

        assert!(matches!(duplicate, Err(RepositoryError::Conflict { .. })));
    }

    #[test]
    fn missing_file_returns_none() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));
        let missing = repository
            .find_file(&file_id("missing"))
            .unwrap_or_else(|error| panic!("find failed: {error}"));

        assert!(missing.is_none());
    }

    #[test]
    fn find_file_by_content_hash_returns_oldest_matching_record() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));
        repository
            .insert_file(new_file("file-2", &["new"], 2_000))
            .unwrap_or_else(|error| panic!("new insert failed: {error}"));
        repository
            .insert_file(new_file("file-1", &["old"], 1_000))
            .unwrap_or_else(|error| panic!("old insert failed: {error}"));

        let found = repository
            .find_file_by_content_hash(&hash())
            .unwrap_or_else(|error| panic!("hash lookup failed: {error}"));

        assert!(matches!(
            found,
            Some(record) if record.id.as_str() == "file-1"
                && record.tags == vec![tag_value("old")]
        ));
    }

    #[test]
    fn stats_since_counts_files_tags_pins_and_recent_uploads() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));
        repository
            .insert_file(new_file("old", &["archive"], 1_000))
            .unwrap_or_else(|error| panic!("old insert failed: {error}"));
        repository
            .insert_file(new_file("new", &["archive", "fresh"], 2_000))
            .unwrap_or_else(|error| panic!("new insert failed: {error}"));

        let stats = repository
            .stats_since(timestamp(1_500))
            .unwrap_or_else(|error| panic!("stats failed: {error}"));

        assert_eq!(stats.file_count, 2);
        assert_eq!(stats.note_count, 0);
        assert_eq!(stats.tag_count, 2);
        assert_eq!(stats.pinned_count, 2);
        assert_eq!(stats.recent_upload_count, 1);
        assert_eq!(stats.recent_note_count, 0);
    }

    #[test]
    fn list_files_recent_returns_newest_first() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));
        repository
            .insert_file(new_file("old", &[], 1_000))
            .unwrap_or_else(|error| panic!("old insert failed: {error}"));
        repository
            .insert_file(new_file("middle", &[], 2_000))
            .unwrap_or_else(|error| panic!("middle insert failed: {error}"));
        repository
            .insert_file(new_file("new", &[], 3_000))
            .unwrap_or_else(|error| panic!("new insert failed: {error}"));

        let list = repository
            .list_files_recent(10)
            .unwrap_or_else(|error| panic!("list failed: {error}"));

        assert_eq!(list.len(), 3);
        assert_eq!(list[0].id.as_str(), "new");
        assert_eq!(list[1].id.as_str(), "middle");
        assert_eq!(list[2].id.as_str(), "old");
    }

    #[test]
    fn list_files_recent_respects_limit() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));
        repository
            .insert_file(new_file("1", &[], 1_000))
            .unwrap_or_else(|error| panic!("insert failed: {error}"));
        repository
            .insert_file(new_file("2", &[], 2_000))
            .unwrap_or_else(|error| panic!("insert failed: {error}"));
        repository
            .insert_file(new_file("3", &[], 3_000))
            .unwrap_or_else(|error| panic!("insert failed: {error}"));

        let list = repository
            .list_files_recent(2)
            .unwrap_or_else(|error| panic!("list failed: {error}"));

        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id.as_str(), "3");
        assert_eq!(list[1].id.as_str(), "2");
    }

    #[test]
    fn list_files_applies_filters_and_cursor_pagination() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));

        let mut earliest = new_file("file-1", &["Docs", "Family"], 1_000);
        earliest.name = filename("report-alpha.png");
        earliest.mime_type = mime_type("image/png");
        earliest.pinned_at = Some(1);
        repository
            .insert_file(earliest)
            .unwrap_or_else(|error| panic!("first insert failed: {error}"));

        let mut second = new_file("file-2", &["Docs", "Family"], 2_000);
        second.name = filename("report-beta.png");
        second.mime_type = mime_type("image/png");
        second.pinned_at = Some(2);
        repository
            .insert_file(second)
            .unwrap_or_else(|error| panic!("second insert failed: {error}"));

        let mut wrong_tags = new_file("file-3", &["Docs"], 1_500);
        wrong_tags.name = filename("report-missing-tag.png");
        wrong_tags.mime_type = mime_type("image/png");
        wrong_tags.pinned_at = Some(3);
        repository
            .insert_file(wrong_tags)
            .unwrap_or_else(|error| panic!("third insert failed: {error}"));

        let mut wrong_mime = new_file("file-4", &["Docs", "Family"], 1_600);
        wrong_mime.name = filename("report-text.txt");
        wrong_mime.mime_type = mime_type("text/plain");
        wrong_mime.pinned_at = Some(4);
        repository
            .insert_file(wrong_mime)
            .unwrap_or_else(|error| panic!("fourth insert failed: {error}"));

        let mut unpinned = new_file("file-5", &["Docs", "Family"], 1_700);
        unpinned.name = filename("report-unpinned.png");
        unpinned.mime_type = mime_type("image/png");
        unpinned.pinned_at = None;
        repository
            .insert_file(unpinned)
            .unwrap_or_else(|error| panic!("fifth insert failed: {error}"));

        let query = ListQuery {
            limit: 1,
            tags: vec![tag_key("Docs"), tag_key("Family")],
            mime_prefix: Some("image".to_owned()),
            name_substring: Some("report".to_owned()),
            since: Some(timestamp(900)),
            until: Some(timestamp(2_100)),
            pinned_only: true,
            sort: ListSort::UploadedAsc,
            ..ListQuery::default()
        };

        let first_page = repository
            .list_files(&query)
            .unwrap_or_else(|error| panic!("first list failed: {error}"));
        assert_eq!(first_page.files.len(), 1);
        assert_eq!(first_page.files[0].id.as_str(), "file-1");
        assert!(first_page.next_cursor.is_some());

        let second_page = repository
            .list_files(&ListQuery {
                after_cursor: first_page.next_cursor,
                ..query
            })
            .unwrap_or_else(|error| panic!("second list failed: {error}"));
        assert_eq!(second_page.files.len(), 1);
        assert_eq!(second_page.files[0].id.as_str(), "file-2");
        assert!(second_page.next_cursor.is_none());
    }

    #[test]
    fn list_files_supports_name_and_size_sorts() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));

        let mut alpha = new_file("alpha", &[], 1_000);
        alpha.name = filename("alpha.txt");
        alpha.size = FileSize::new(20);
        alpha.pinned_at = None;
        repository
            .insert_file(alpha)
            .unwrap_or_else(|error| panic!("alpha insert failed: {error}"));

        let mut gamma = new_file("gamma", &[], 1_100);
        gamma.name = filename("gamma.txt");
        gamma.size = FileSize::new(30);
        gamma.pinned_at = None;
        repository
            .insert_file(gamma)
            .unwrap_or_else(|error| panic!("gamma insert failed: {error}"));

        let mut beta = new_file("beta", &[], 1_200);
        beta.name = filename("beta.txt");
        beta.size = FileSize::new(10);
        beta.pinned_at = None;
        repository
            .insert_file(beta)
            .unwrap_or_else(|error| panic!("beta insert failed: {error}"));

        let by_name = repository
            .list_files(&ListQuery {
                limit: 10,
                sort: ListSort::NameAsc,
                ..ListQuery::default()
            })
            .unwrap_or_else(|error| panic!("name list failed: {error}"));
        assert_eq!(by_name.files.len(), 3);
        assert_eq!(by_name.files[0].name.original(), "alpha.txt");
        assert_eq!(by_name.files[1].name.original(), "beta.txt");
        assert_eq!(by_name.files[2].name.original(), "gamma.txt");

        let by_size = repository
            .list_files(&ListQuery {
                limit: 10,
                sort: ListSort::SizeDesc,
                ..ListQuery::default()
            })
            .unwrap_or_else(|error| panic!("size list failed: {error}"));
        assert_eq!(by_size.files.len(), 3);
        assert_eq!(by_size.files[0].size.bytes(), 30);
        assert_eq!(by_size.files[1].size.bytes(), 20);
        assert_eq!(by_size.files[2].size.bytes(), 10);
    }

    #[test]
    fn list_files_rejects_invalid_cursor() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));
        repository
            .insert_file(new_file("file-1", &[], 1_000))
            .unwrap_or_else(|error| panic!("insert failed: {error}"));

        let result = repository.list_files(&ListQuery {
            limit: 10,
            sort: ListSort::UploadedAsc,
            after_cursor: Some(
                Cursor::new("ua.bad-value.file-1")
                    .unwrap_or_else(|cursor_error| panic!("cursor parse failed: {cursor_error}")),
            ),
            ..ListQuery::default()
        });
        let error = match result {
            Ok(page) => panic!(
                "expected invalid cursor error, got {} files",
                page.files.len()
            ),
            Err(error) => error,
        };

        assert!(matches!(
            error,
            RepositoryError::OperationFailed { message } if message.starts_with("invalid cursor:")
        ));
    }

    #[test]
    fn delete_file_removes_metadata_tags_and_reports_last_reference() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));
        repository
            .insert_file(new_file("file-1", &["docs"], 1_000))
            .unwrap_or_else(|error| panic!("insert failed: {error}"));

        let deleted = repository
            .delete_file(&file_id("file-1"))
            .unwrap_or_else(|error| panic!("delete failed: {error}"));
        let stats = repository
            .stats_since(timestamp(0))
            .unwrap_or_else(|error| panic!("stats failed: {error}"));

        assert!(matches!(
            deleted,
            Some(record) if record.record.id.as_str() == "file-1"
                && record.record.tags == vec![tag_value("docs")]
                && record.remaining_content_references == 0
        ));
        assert_eq!(stats.file_count, 0);
        assert_eq!(stats.tag_count, 0);
        assert!(repository
            .find_file(&file_id("file-1"))
            .unwrap_or_else(|error| panic!("find failed: {error}"))
            .is_none());
    }

    #[test]
    fn delete_file_keeps_shared_tags_and_reports_remaining_references() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));
        repository
            .insert_file(new_file("file-1", &["shared"], 1_000))
            .unwrap_or_else(|error| panic!("first insert failed: {error}"));
        repository
            .insert_file(new_file("file-2", &["shared"], 2_000))
            .unwrap_or_else(|error| panic!("second insert failed: {error}"));

        let deleted = repository
            .delete_file(&file_id("file-1"))
            .unwrap_or_else(|error| panic!("delete failed: {error}"));
        let stats = repository
            .stats_since(timestamp(0))
            .unwrap_or_else(|error| panic!("stats failed: {error}"));

        assert!(matches!(
            deleted,
            Some(record) if record.remaining_content_references == 1
        ));
        assert_eq!(stats.file_count, 1);
        assert_eq!(stats.tag_count, 1);
    }

    #[test]
    fn delete_missing_file_is_idempotent() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));

        let deleted = repository
            .delete_file(&file_id("missing"))
            .unwrap_or_else(|error| panic!("delete failed: {error}"));

        assert!(deleted.is_none());
    }

    #[test]
    fn list_tags_returns_counts_in_key_order() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));
        repository
            .insert_file(new_file("file-1", &["Beta", "alpha"], 1_000))
            .unwrap_or_else(|error| panic!("first insert failed: {error}"));
        repository
            .insert_file(new_file("file-2", &["beta"], 2_000))
            .unwrap_or_else(|error| panic!("second insert failed: {error}"));

        let tags = repository
            .list_tags()
            .unwrap_or_else(|error| panic!("list tags failed: {error}"));

        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].tag.display(), "alpha");
        assert_eq!(tags[0].file_count, 1);
        assert_eq!(tags[1].tag.display(), "Beta");
        assert_eq!(tags[1].file_count, 2);
    }

    #[test]
    fn add_tags_to_file_is_idempotent_and_normalizes_duplicates() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));
        repository
            .insert_file(new_file("file-1", &["Docs"], 1_000))
            .unwrap_or_else(|error| panic!("insert failed: {error}"));
        let tags = vec![tag_value("docs"), tag_value("Family")];

        let outcome = repository
            .add_tags_to_file(&file_id("file-1"), &tags)
            .unwrap_or_else(|error| panic!("add tags failed: {error}"));
        let found = repository
            .find_file(&file_id("file-1"))
            .unwrap_or_else(|error| panic!("find failed: {error}"));

        assert_eq!(outcome.changed_count, 1);
        assert!(matches!(
            found,
            Some(record) if record.tags == vec![tag_value("Docs"), tag_value("Family")]
        ));
    }

    #[test]
    fn tag_mutations_report_missing_file() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));
        let tags = vec![tag_value("Docs")];

        let add = repository.add_tags_to_file(&file_id("missing"), &tags);
        let remove = repository.remove_tag_from_file(&file_id("missing"), tag_value("Docs").key());

        assert!(matches!(add, Err(RepositoryError::NotFound)));
        assert!(matches!(remove, Err(RepositoryError::NotFound)));
    }

    #[test]
    fn remove_tag_from_file_is_idempotent_and_cleans_orphaned_tag() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));
        repository
            .insert_file(new_file("file-1", &["Docs"], 1_000))
            .unwrap_or_else(|error| panic!("insert failed: {error}"));

        let first = repository
            .remove_tag_from_file(&file_id("file-1"), tag_value("Docs").key())
            .unwrap_or_else(|error| panic!("remove failed: {error}"));
        let second = repository
            .remove_tag_from_file(&file_id("file-1"), tag_value("Docs").key())
            .unwrap_or_else(|error| panic!("second remove failed: {error}"));
        let tags = repository
            .list_tags()
            .unwrap_or_else(|error| panic!("list tags failed: {error}"));

        assert_eq!(first.changed_count, 1);
        assert_eq!(second.changed_count, 0);
        assert!(tags.is_empty());
    }

    #[test]
    fn pin_file_sets_position_and_returns_changed() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));
        repository
            .insert_file(NewFileRecord {
                id: file_id("file-1"),
                name: filename("report.pdf"),
                size: FileSize::new(42),
                content_hash: hash(),
                mime_type: mime_type("application/pdf"),
                storage_handle: storage_handle(),
                uploaded_at: timestamp(1_000),
                tags: vec![],
                pinned_at: None,
                folder_path: String::new(),
                owner_id: None,
                visibility: tssp_domain::Visibility::Private,
                public_token: None,
            })
            .unwrap_or_else(|error| panic!("insert failed: {error}"));

        let first = repository
            .pin_file(&file_id("file-1"), Some(5))
            .unwrap_or_else(|error| panic!("pin failed: {error}"));
        let second = repository
            .pin_file(&file_id("file-1"), Some(5))
            .unwrap_or_else(|error| panic!("second pin failed: {error}"));

        let list = repository
            .list_pinned_files()
            .unwrap_or_else(|error| panic!("list failed: {error}"));

        assert!(first.existed);
        assert!(first.changed);
        assert!(second.existed);
        assert!(!second.changed);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id.as_str(), "file-1");
        assert_eq!(list[0].pinned_at, Some(5));
    }

    #[test]
    fn pin_file_inserts_before_existing_positions() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));
        let mut first = new_file("file-1", &[], 1_000);
        first.pinned_at = None;
        repository
            .insert_file(first)
            .unwrap_or_else(|error| panic!("first insert failed: {error}"));
        let mut second = new_file("file-2", &[], 2_000);
        second.pinned_at = None;
        repository
            .insert_file(second)
            .unwrap_or_else(|error| panic!("second insert failed: {error}"));

        repository
            .pin_file(&file_id("file-1"), None)
            .unwrap_or_else(|error| panic!("first pin failed: {error}"));
        repository
            .pin_file(&file_id("file-2"), Some(1))
            .unwrap_or_else(|error| panic!("second pin failed: {error}"));

        let list = repository
            .list_pinned_files()
            .unwrap_or_else(|error| panic!("list failed: {error}"));

        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id.as_str(), "file-2");
        assert_eq!(list[0].pinned_at, Some(1));
        assert_eq!(list[1].id.as_str(), "file-1");
        assert_eq!(list[1].pinned_at, Some(2));
    }

    #[test]
    fn unpin_file_clears_position_and_returns_changed() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));
        repository
            .insert_file(new_file("file-1", &[], 1_000)) // new_file pins by default
            .unwrap_or_else(|error| panic!("insert failed: {error}"));

        let first = repository
            .unpin_file(&file_id("file-1"))
            .unwrap_or_else(|error| panic!("unpin failed: {error}"));
        let second = repository
            .unpin_file(&file_id("file-1"))
            .unwrap_or_else(|error| panic!("second unpin failed: {error}"));

        let list = repository
            .list_pinned_files()
            .unwrap_or_else(|error| panic!("list failed: {error}"));

        assert!(first.existed);
        assert!(first.changed);
        assert!(second.existed);
        assert!(!second.changed);
        assert!(list.is_empty());
    }

    #[test]
    fn unpin_file_compacts_remaining_positions() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));
        let mut first = new_file("file-1", &[], 1_000);
        first.pinned_at = None;
        repository
            .insert_file(first)
            .unwrap_or_else(|error| panic!("first insert failed: {error}"));
        let mut second = new_file("file-2", &[], 2_000);
        second.pinned_at = None;
        repository
            .insert_file(second)
            .unwrap_or_else(|error| panic!("second insert failed: {error}"));

        repository
            .pin_file(&file_id("file-1"), None)
            .unwrap_or_else(|error| panic!("first pin failed: {error}"));
        repository
            .pin_file(&file_id("file-2"), None)
            .unwrap_or_else(|error| panic!("second pin failed: {error}"));

        repository
            .unpin_file(&file_id("file-1"))
            .unwrap_or_else(|error| panic!("unpin failed: {error}"));

        let list = repository
            .list_pinned_files()
            .unwrap_or_else(|error| panic!("list failed: {error}"));

        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id.as_str(), "file-2");
        assert_eq!(list[0].pinned_at, Some(1));
    }

    #[test]
    fn reorder_pins_updates_positions() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));
        repository
            .insert_file(new_file("file-1", &[], 1_000))
            .unwrap_or_else(|error| panic!("insert failed: {error}"));
        repository
            .insert_file(new_file("file-2", &[], 1_000))
            .unwrap_or_else(|error| panic!("insert failed: {error}"));

        repository
            .reorder_pins(&[file_id("file-2"), file_id("file-1")])
            .unwrap_or_else(|error| panic!("reorder failed: {error}"));

        let list = repository
            .list_pinned_files()
            .unwrap_or_else(|error| panic!("list failed: {error}"));

        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id.as_str(), "file-2");
        assert_eq!(list[0].pinned_at, Some(1));
        assert_eq!(list[1].id.as_str(), "file-1");
        assert_eq!(list[1].pinned_at, Some(2));
    }

    #[test]
    fn search_files_returns_matching_records() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));

        let mut file1 = new_file("file-1", &["Docs", "Work"], 1_000);
        file1.name = filename("annual_report_2023.pdf");
        repository
            .insert_file(file1)
            .unwrap_or_else(|error| panic!("insert failed: {error}"));

        let mut file2 = new_file("file-2", &["Images"], 1_000);
        file2.name = filename("vacation_photo.jpg");
        repository
            .insert_file(file2)
            .unwrap_or_else(|error| panic!("insert failed: {error}"));

        let mut file3 = new_file("file-3", &["Docs", "Personal"], 1_000);
        file3.name = filename("personal_budget_2023.xlsx");
        repository
            .insert_file(file3)
            .unwrap_or_else(|error| panic!("insert failed: {error}"));

        // Search by name
        let results = repository
            .search_files("report")
            .unwrap_or_else(|error| panic!("search failed: {error}"));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id.as_str(), "file-1");

        // Search by tag
        let results = repository
            .search_files("Docs")
            .unwrap_or_else(|error| panic!("search failed: {error}"));
        assert_eq!(results.len(), 2);

        // Search matching across different files
        let results = repository
            .search_files("2023")
            .unwrap_or_else(|error| panic!("search failed: {error}"));
        assert_eq!(results.len(), 2);

        // Search with no matches
        let results = repository
            .search_files("nonexistent")
            .unwrap_or_else(|error| panic!("search failed: {error}"));
        assert!(results.is_empty());
    }

    #[test]
    fn unified_search_ranks_exact_prefix_and_fuzzy_file_hits() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"));

        let mut exact = new_file("file-exact", &["Finance"], 3_000);
        exact.name = filename("annual_report.pdf");
        repository
            .insert_file(exact)
            .unwrap_or_else(|error| panic!("insert failed: {error}"));

        let mut prefix = new_file("file-prefix", &["Finance"], 2_000);
        prefix.name = filename("reporting-notes.txt");
        repository
            .insert_file(prefix)
            .unwrap_or_else(|error| panic!("insert failed: {error}"));

        let mut fuzzy = new_file("file-fuzzy", &["Finance"], 1_000);
        fuzzy.name = filename("reprot-draft.txt");
        repository
            .insert_file(fuzzy)
            .unwrap_or_else(|error| panic!("insert failed: {error}"));

        let hits = repository
            .search_all("report")
            .unwrap_or_else(|error| panic!("search failed: {error}"));
        let ids = hits
            .iter()
            .filter_map(|hit| match hit {
                SearchHit::File(file) => Some(file.id.as_str().to_owned()),
                SearchHit::Note(_) => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(ids.first().map(String::as_str), Some("file-exact"));
        assert!(ids.iter().any(|id| id == "file-fuzzy"));
    }

    fn new_file(id: &str, tags: &[&str], uploaded_at: i64) -> NewFileRecord {
        NewFileRecord {
            id: file_id(id),
            name: filename("report.pdf"),
            size: FileSize::new(42),
            content_hash: hash(),
            mime_type: mime_type("application/pdf"),
            storage_handle: storage_handle(),
            uploaded_at: timestamp(uploaded_at),
            tags: tags.iter().map(|tag| tag_value(tag)).collect(),
            pinned_at: Some(2),
            folder_path: String::new(),
            owner_id: None,
            visibility: tssp_domain::Visibility::Private,
            public_token: None,
        }
    }

    fn file_id(value: &str) -> FileId {
        FileId::new(value).unwrap_or_else(|error| panic!("invalid file id: {error}"))
    }

    fn filename(value: &str) -> FileName {
        FileName::new(value).unwrap_or_else(|error| panic!("invalid filename: {error}"))
    }

    fn hash() -> ContentHash {
        ContentHash::new("abcdefabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123")
            .unwrap_or_else(|error| panic!("invalid content hash: {error}"))
    }

    fn mime_type(value: &str) -> MimeType {
        MimeType::new(value).unwrap_or_else(|error| panic!("invalid mime type: {error}"))
    }

    fn storage_handle() -> StorageHandle {
        StorageHandle::new("blobs/ab/cd/abcdef")
            .unwrap_or_else(|error| panic!("invalid storage handle: {error}"))
    }

    fn timestamp(seconds: i64) -> UnixTimestamp {
        UnixTimestamp::new(seconds).unwrap_or_else(|error| panic!("invalid timestamp: {error}"))
    }

    fn tag_value(value: &str) -> Tag {
        Tag::new(value).unwrap_or_else(|error| panic!("invalid tag: {error}"))
    }

    fn tag_key(value: &str) -> TagKey {
        TagKey::new(value).unwrap_or_else(|error| panic!("invalid tag key: {error}"))
    }
}
