//! Row mapping, mutation helpers, and query utilities for the file repository.

use rusqlite::{params, Connection, ErrorCode, Row};
use tssp_domain::{
    ContentHash, FileId, FileName, FileRecord, FileSize, MimeType, StorageHandle, Tag,
    UnixTimestamp, UserId, Visibility,
};
use tssp_ports::{NewFileRecord, RepositoryError};

pub(crate) fn validate_list_limit(limit: u64) -> Result<usize, RepositoryError> {
    if limit == 0 {
        return Err(RepositoryError::OperationFailed {
            message: "list limit must be greater than 0".to_owned(),
        });
    }
    if limit > 500 {
        return Err(RepositoryError::OperationFailed {
            message: "list limit must not exceed 500".to_owned(),
        });
    }
    usize::try_from(limit).map_err(|error| RepositoryError::OperationFailed {
        message: format!("list limit does not fit usize: {error}"),
    })
}

pub(crate) fn normalize_folder_prefix(value: &str) -> String {
    value.trim().trim_matches('/').replace('\\', "/")
}

pub(crate) fn insert_file_row(
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
             (id, name, storage_component, size_bytes, content_hash, mime_type, storage_handle, uploaded_at, pinned_at, folder_path, owner_id, visibility, public_token, public_expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
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
                new_file.folder_path,
                new_file
                    .owner_id
                    .as_ref()
                    .map(tssp_domain::UserId::as_str),
                new_file.visibility.as_str(),
                new_file.public_token,
                new_file.public_expires_at,
            ],
        )
        .map(|_rows| ())
        .map_err(map_rusqlite_repository_error)
}

pub(crate) fn ensure_file_exists(
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

pub(crate) fn find_file_in_transaction(
    transaction: &rusqlite::Transaction<'_>,
    id: &FileId,
) -> Result<Option<FileRecord>, RepositoryError> {
    let mut statement = transaction
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
    map_file_row(row).map(Some)
}

pub(crate) fn insert_tags(
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

pub(crate) fn cleanup_orphaned_tags(
    transaction: &rusqlite::Transaction<'_>,
) -> Result<(), RepositoryError> {
    transaction
        .execute(
            "DELETE FROM tags
             WHERE NOT EXISTS (
                 SELECT 1 FROM file_tags
                 JOIN files ON files.id = file_tags.file_id
                 WHERE file_tags.tag_key = tags.key AND files.deleted_at IS NULL
             )
             AND NOT EXISTS (
                 SELECT 1 FROM note_tags
                 JOIN notes ON notes.id = note_tags.note_id
                 WHERE note_tags.tag_key = tags.key AND notes.deleted_at IS NULL
             )",
            [],
        )
        .map(|_rows| ())
        .map_err(map_rusqlite_repository_error)
}

pub(crate) fn load_tags(connection: &Connection, id: &FileId) -> Result<Vec<Tag>, RepositoryError> {
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

pub(crate) fn load_tags_batch(
    connection: &Connection,
    ids: &[FileId],
) -> Result<std::collections::HashMap<FileId, Vec<Tag>>, RepositoryError> {
    if ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let mut map = std::collections::HashMap::with_capacity(ids.len());
    for id in ids {
        map.insert(id.clone(), Vec::new());
    }

    // SQLite has a limit on the number of parameters (usually 999 or 32766).
    // For TSSP, we'll chunk by 100 which is well within limits.
    for chunk in ids.chunks(100) {
        let placeholders = std::iter::repeat_n("?", chunk.len())
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT file_tags.file_id, tags.display
             FROM tags
             JOIN file_tags ON file_tags.tag_key = tags.key
             WHERE file_tags.file_id IN ({placeholders})
             ORDER BY file_tags.file_id, tags.key"
        );

        let mut statement = connection
            .prepare(&sql)
            .map_err(map_rusqlite_repository_error)?;

        let params = rusqlite::params_from_iter(chunk.iter().map(FileId::as_str));
        let mut rows = statement
            .query(params)
            .map_err(map_rusqlite_repository_error)?;

        while let Some(row) = rows.next().map_err(map_rusqlite_repository_error)? {
            let file_id_str: String = row.get(0).map_err(map_rusqlite_repository_error)?;
            let display: String = row.get(1).map_err(map_rusqlite_repository_error)?;

            let file_id =
                FileId::new(file_id_str).map_err(|error| map_domain_repository_error(&error))?;
            let tag = Tag::new(display).map_err(|error| map_domain_repository_error(&error))?;

            if let Some(tags) = map.get_mut(&file_id) {
                tags.push(tag);
            }
        }
    }

    Ok(map)
}

pub(crate) fn count<P>(
    connection: &Connection,
    sql: &str,
    params: P,
) -> Result<u64, RepositoryError>
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

pub(crate) fn map_file_row(row: &Row<'_>) -> Result<FileRecord, RepositoryError> {
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
    let folder_path: String = row.get(8).map_err(map_rusqlite_repository_error)?;
    let owner_id_raw: Option<String> = row.get(9).map_err(map_rusqlite_repository_error)?;
    let visibility_raw: String = row.get(10).map_err(map_rusqlite_repository_error)?;
    let public_token: Option<String> = row.get(11).map_err(map_rusqlite_repository_error)?;
    let public_expires_at: Option<i64> = row.get(12).map_err(map_rusqlite_repository_error)?;
    let owner_id = owner_id_raw
        .map(|value| UserId::new(value).map_err(|error| map_domain_repository_error(&error)))
        .transpose()?;
    let visibility =
        Visibility::parse(&visibility_raw).map_err(|error| map_domain_repository_error(&error))?;

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
        folder_path,
        owner_id,
        visibility,
        public_token,
        public_expires_at,
    })
}

pub(crate) fn map_domain_repository_error(error: &tssp_domain::DomainError) -> RepositoryError {
    RepositoryError::OperationFailed {
        message: error.to_string(),
    }
}

pub(crate) fn map_rusqlite_repository_error(error: rusqlite::Error) -> RepositoryError {
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
