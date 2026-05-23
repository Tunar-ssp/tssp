//! `SQLite` implementation of the TSSP metadata repository.
//!
//! The adapter owns schema creation, `SQLite` pragmas, and row mapping. All SQL is
//! kept behind the `FileRepository` port so application services stay storage
//! agnostic.

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use rusqlite::{params, Connection, ErrorCode, Row};
use thiserror::Error;
use tssp_domain::{
    ContentHash, FileId, FileName, FileRecord, FileSize, MimeType, StorageHandle, Tag, TagKey,
    UnixTimestamp,
};
use tssp_ports::{
    DeletedFileRecord, FileRepository, NewFileRecord, RepositoryError, RepositoryStats,
    TagMutationOutcome, TagSummary,
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
        configure_connection(&connection)?;
        run_integrity_check(&connection)?;
        run_migrations(&connection)?;
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
        configure_connection(&connection)?;
        run_integrity_check(&connection)?;
        run_migrations(&connection)?;
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
                "SELECT id, name, size_bytes, content_hash, mime_type, storage_handle, uploaded_at, pinned_at
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
                "SELECT id, name, size_bytes, content_hash, mime_type, storage_handle, uploaded_at, pinned_at
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

    fn list_files_recent(&self, limit: u64) -> Result<Vec<FileRecord>, RepositoryError> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT id, name, size_bytes, content_hash, mime_type, storage_handle, uploaded_at, pinned_at
                 FROM files
                 ORDER BY uploaded_at DESC, id DESC
                 LIMIT ?1",
            )
            .map_err(map_rusqlite_repository_error)?;

        let limit_i64 = i64::try_from(limit).map_err(|error| RepositoryError::OperationFailed {
            message: format!("list limit does not fit sqlite integer: {error}"),
        })?;

        let mut rows = statement
            .query(params![limit_i64])
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

    fn list_files_by_tag(
        &self,
        tag: &tssp_domain::TagKey,
        limit: u64,
    ) -> Result<Vec<FileRecord>, RepositoryError> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT f.id, f.name, f.size_bytes, f.content_hash, f.mime_type, f.storage_handle, f.uploaded_at, f.pinned_at
                 FROM files f
                 JOIN file_tags ft ON ft.file_id = f.id
                 WHERE ft.tag_key = ?1
                 ORDER BY f.uploaded_at DESC, f.id DESC
                 LIMIT ?2",
            )
            .map_err(map_rusqlite_repository_error)?;

        let limit_i64 = i64::try_from(limit).map_err(|error| RepositoryError::OperationFailed {
            message: format!("list limit does not fit sqlite integer: {error}"),
        })?;

        let mut rows = statement
            .query(params![tag.as_str(), limit_i64])
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
                "SELECT id, name, size_bytes, content_hash, mime_type, storage_handle, uploaded_at, pinned_at
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
                "SELECT f.id, f.name, f.size_bytes, f.content_hash, f.mime_type, f.storage_handle, f.uploaded_at, f.pinned_at
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
}

/// Errors raised while opening or migrating the `SQLite` adapter.
#[derive(Debug, Error)]
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

fn configure_connection(connection: &Connection) -> Result<(), SqliteRepositoryError> {
    connection
        .pragma_update(None, "journal_mode", "WAL")
        .map_err(SqliteRepositoryError::Configure)?;
    connection
        .pragma_update(None, "synchronous", "NORMAL")
        .map_err(SqliteRepositoryError::Configure)?;
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .map_err(SqliteRepositoryError::Configure)?;
    connection
        .busy_timeout(std::time::Duration::from_secs(5))
        .map_err(SqliteRepositoryError::Configure)
}

fn run_integrity_check(connection: &Connection) -> Result<(), SqliteRepositoryError> {
    let result: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(SqliteRepositoryError::Configure)?;
    if result == "ok" {
        return Ok(());
    }

    Err(SqliteRepositoryError::Integrity { message: result })
}

fn run_migrations(connection: &Connection) -> Result<(), SqliteRepositoryError> {
    connection
        .execute_batch(
            "
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS files (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                storage_component TEXT NOT NULL,
                size_bytes INTEGER NOT NULL CHECK (size_bytes >= 0),
                content_hash TEXT NOT NULL,
                mime_type TEXT NOT NULL,
                storage_handle TEXT NOT NULL,
                uploaded_at INTEGER NOT NULL,
                pinned_at INTEGER
            );

            CREATE TABLE IF NOT EXISTS tags (
                key TEXT PRIMARY KEY,
                display TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS file_tags (
                file_id TEXT NOT NULL REFERENCES files(id) ON DELETE CASCADE,
                tag_key TEXT NOT NULL REFERENCES tags(key) ON DELETE CASCADE,
                PRIMARY KEY (file_id, tag_key)
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS file_search
                USING fts5(file_id UNINDEXED, name, tags);

            CREATE TRIGGER IF NOT EXISTS files_ai AFTER INSERT ON files BEGIN
                INSERT INTO file_search (rowid, file_id, name, tags) 
                VALUES (new.rowid, new.id, new.name, '');
            END;

            CREATE TRIGGER IF NOT EXISTS files_ad AFTER DELETE ON files BEGIN
                DELETE FROM file_search WHERE file_id = old.id;
            END;

            CREATE TRIGGER IF NOT EXISTS files_au AFTER UPDATE OF name ON files BEGIN
                UPDATE file_search SET name = new.name WHERE file_id = new.id;
            END;

            CREATE TRIGGER IF NOT EXISTS file_tags_ai AFTER INSERT ON file_tags BEGIN
                UPDATE file_search 
                SET tags = tags || ' ' || new.tag_key 
                WHERE file_id = new.file_id;
            END;

            CREATE TRIGGER IF NOT EXISTS file_tags_ad AFTER DELETE ON file_tags BEGIN
                UPDATE file_search 
                SET tags = (SELECT group_concat(tag_key, ' ') FROM file_tags WHERE file_id = old.file_id)
                WHERE file_id = old.file_id;
            END;

            INSERT OR IGNORE INTO schema_migrations (version) VALUES (1);
            ",
        )
        .map_err(SqliteRepositoryError::Migration)
}

fn insert_file_row(
    transaction: &rusqlite::Transaction<'_>,
    new_file: &NewFileRecord,
) -> Result<(), RepositoryError> {
    let size =
        i64::try_from(new_file.size.bytes()).map_err(|error| RepositoryError::OperationFailed {
            message: format!("file size does not fit sqlite integer: {error}"),
        })?;
    transaction
        .execute(
            "INSERT INTO files
             (id, name, storage_component, size_bytes, content_hash, mime_type, storage_handle, uploaded_at, pinned_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                new_file.id.as_str(),
                new_file.name.original(),
                new_file.name.storage_component(),
                size,
                new_file.content_hash.as_str(),
                new_file.mime_type.as_str(),
                new_file.storage_handle.as_str(),
                new_file.uploaded_at.seconds(),
                new_file.pinned_at,
            ],
        )
        .map(|_rows| ())
        .map_err(map_rusqlite_repository_error)
}

fn ensure_file_exists(
    transaction: &rusqlite::Transaction<'_>,
    id: &FileId,
) -> Result<(), RepositoryError> {
    let exists: bool = transaction
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM files WHERE id = ?1)",
            params![id.as_str()],
            |row| row.get(0),
        )
        .map_err(map_rusqlite_repository_error)?;
    if exists {
        return Ok(());
    }
    Err(RepositoryError::NotFound)
}

fn find_file_in_transaction(
    transaction: &rusqlite::Transaction<'_>,
    id: &FileId,
) -> Result<Option<FileRecord>, RepositoryError> {
    let mut statement = transaction
        .prepare(
            "SELECT id, name, size_bytes, content_hash, mime_type, storage_handle, uploaded_at, pinned_at
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
    map_file_row(row).map(Some)
}

fn insert_tags(
    transaction: &rusqlite::Transaction<'_>,
    new_file: &NewFileRecord,
) -> Result<(), RepositoryError> {
    for tag in &new_file.tags {
        transaction
            .execute(
                "INSERT OR IGNORE INTO tags (key, display) VALUES (?1, ?2)",
                params![tag.key().as_str(), tag.display()],
            )
            .map_err(map_rusqlite_repository_error)?;
        transaction
            .execute(
                "INSERT OR IGNORE INTO file_tags (file_id, tag_key) VALUES (?1, ?2)",
                params![new_file.id.as_str(), tag.key().as_str()],
            )
            .map_err(map_rusqlite_repository_error)?;
    }
    Ok(())
}

fn cleanup_orphaned_tags(transaction: &rusqlite::Transaction<'_>) -> Result<(), RepositoryError> {
    transaction
        .execute(
            "DELETE FROM tags
             WHERE NOT EXISTS (
                 SELECT 1 FROM file_tags WHERE file_tags.tag_key = tags.key
             )",
            [],
        )
        .map(|_rows| ())
        .map_err(map_rusqlite_repository_error)
}

fn load_tags(connection: &Connection, id: &FileId) -> Result<Vec<Tag>, RepositoryError> {
    let mut statement = connection
        .prepare(
            "SELECT tags.display
             FROM tags
             JOIN file_tags ON file_tags.tag_key = tags.key
             WHERE file_tags.file_id = ?1
             ORDER BY tags.key",
        )
        .map_err(map_rusqlite_repository_error)?;
    let mapped = statement
        .query_map(params![id.as_str()], |row| row.get::<_, String>(0))
        .map_err(map_rusqlite_repository_error)?;

    let mut tags = Vec::new();
    for tag in mapped {
        let display = tag.map_err(map_rusqlite_repository_error)?;
        tags.push(Tag::new(display).map_err(|error| map_domain_repository_error(&error))?);
    }
    Ok(tags)
}

fn count<P>(connection: &Connection, sql: &str, params: P) -> Result<u64, RepositoryError>
where
    P: rusqlite::Params,
{
    let count: i64 = connection
        .query_row(sql, params, |row| row.get(0))
        .map_err(map_rusqlite_repository_error)?;
    u64::try_from(count).map_err(|error| RepositoryError::OperationFailed {
        message: format!("metadata count is invalid: {error}"),
    })
}

fn map_file_row(row: &Row<'_>) -> Result<FileRecord, RepositoryError> {
    let id: String = row.get(0).map_err(map_rusqlite_repository_error)?;
    let name: String = row.get(1).map_err(map_rusqlite_repository_error)?;
    let size: i64 = row.get(2).map_err(map_rusqlite_repository_error)?;
    let content_hash: String = row.get(3).map_err(map_rusqlite_repository_error)?;
    let mime_type: String = row.get(4).map_err(map_rusqlite_repository_error)?;
    let storage_handle: String = row.get(5).map_err(map_rusqlite_repository_error)?;
    let uploaded_at: i64 = row.get(6).map_err(map_rusqlite_repository_error)?;
    let pinned_at_raw: Option<i64> = row.get(7).map_err(map_rusqlite_repository_error)?;
    let size = u64::try_from(size).map_err(|error| RepositoryError::OperationFailed {
        message: format!("stored file size is invalid: {error}"),
    })?;
    let pinned_at = pinned_at_raw
        .map(u32::try_from)
        .transpose()
        .map_err(|error| RepositoryError::OperationFailed {
            message: format!("stored pin position is invalid: {error}"),
        })?;

    Ok(FileRecord {
        id: FileId::new(id).map_err(|error| map_domain_repository_error(&error))?,
        name: FileName::new(name).map_err(|error| map_domain_repository_error(&error))?,
        size: FileSize::new(size),
        content_hash: ContentHash::new(content_hash)
            .map_err(|error| map_domain_repository_error(&error))?,
        mime_type: MimeType::new(mime_type).map_err(|error| map_domain_repository_error(&error))?,
        storage_handle: StorageHandle::new(storage_handle)
            .map_err(|error| map_domain_repository_error(&error))?,
        uploaded_at: UnixTimestamp::new(uploaded_at)
            .map_err(|error| map_domain_repository_error(&error))?,
        tags: Vec::new(),
        pinned_at,
    })
}

fn map_domain_repository_error(error: &tssp_domain::DomainError) -> RepositoryError {
    RepositoryError::OperationFailed {
        message: error.to_string(),
    }
}

fn map_rusqlite_repository_error(error: rusqlite::Error) -> RepositoryError {
    match error {
        rusqlite::Error::SqliteFailure(failure, _message)
            if failure.code == ErrorCode::DatabaseBusy
                || failure.code == ErrorCode::DatabaseLocked =>
        {
            RepositoryError::Busy
        }
        rusqlite::Error::SqliteFailure(failure, message)
            if failure.code == ErrorCode::ConstraintViolation =>
        {
            RepositoryError::Conflict {
                message: message.unwrap_or_else(|| "constraint violation".to_owned()),
            }
        }
        other => RepositoryError::OperationFailed {
            message: other.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use tssp_domain::{
        ContentHash, FileId, FileName, FileSize, MimeType, StorageHandle, Tag, UnixTimestamp,
    };
    use tssp_ports::{FileRepository, NewFileRecord, RepositoryError};

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
        assert_eq!(stats.tag_count, 2);
        assert_eq!(stats.pinned_count, 2);
        assert_eq!(stats.recent_upload_count, 1);
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
}
