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
    ContentHash, FileId, FileName, FileRecord, FileSize, MimeType, StorageHandle, Tag,
    UnixTimestamp,
};
use tssp_ports::{FileRepository, NewFileRecord, RepositoryError, RepositoryStats};

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
