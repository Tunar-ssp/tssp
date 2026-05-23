//! SQLite persistence for Markdown notes and unified search.

use rusqlite::{params, types::Value, Connection, Row, Transaction};
use tssp_domain::{NoteBody, NoteId, NoteRecord, NoteTitle, Tag, TagKey, UnixTimestamp};
use tssp_ports::{
    NewNoteRecord, NoteListQuery, NoteListSort, NoteRepository, PagedNotes, PinOutcome,
    RepositoryError, SearchHit, TagMutationOutcome,
};

use crate::{
    cleanup_orphaned_tags, map_domain_repository_error, map_rusqlite_repository_error,
    migration_applied, record_migration, SqliteFileRepository, SqliteRepositoryError,
};

impl NoteRepository for SqliteFileRepository {
    fn insert_note(&self, new_note: NewNoteRecord) -> Result<NoteRecord, RepositoryError> {
        let inserted_id = new_note.id.clone();
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction()
            .map_err(map_rusqlite_repository_error)?;
        insert_note_row(&transaction, &new_note)?;
        insert_note_tags(&transaction, &new_note)?;
        transaction
            .commit()
            .map_err(map_rusqlite_repository_error)?;
        drop(connection);

        self.find_note(&inserted_id)?
            .ok_or_else(|| RepositoryError::OperationFailed {
                message: "inserted note could not be read back".to_owned(),
            })
    }

    fn find_note(&self, id: &NoteId) -> Result<Option<NoteRecord>, RepositoryError> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT id, title, body, created_at, updated_at, pinned_at
                 FROM notes
                 WHERE id = ?1",
            )
            .map_err(map_rusqlite_repository_error)?;
        let mut rows = statement
            .query(params![id.as_str()])
            .map_err(map_rusqlite_repository_error)?;
        let Some(row) = rows.next().map_err(map_rusqlite_repository_error)? else {
            return Ok(None);
        };

        let mut record = map_note_row(row)?;
        record.tags = load_note_tags(&connection, id)?;
        Ok(Some(record))
    }

    fn update_note(
        &self,
        id: &NoteId,
        title: &NoteTitle,
        body: &NoteBody,
        updated_at: UnixTimestamp,
    ) -> Result<NoteRecord, RepositoryError> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction()
            .map_err(map_rusqlite_repository_error)?;
        ensure_note_exists(&transaction, id)?;
        transaction
            .execute(
                "UPDATE notes SET title = ?1, body = ?2, updated_at = ?3 WHERE id = ?4",
                params![
                    title.as_str(),
                    body.as_str(),
                    updated_at.seconds(),
                    id.as_str()
                ],
            )
            .map_err(map_rusqlite_repository_error)?;
        transaction
            .execute(
                "UPDATE note_search SET title = ?1, body = ?2 WHERE note_id = ?3",
                params![title.as_str(), body.as_str(), id.as_str()],
            )
            .map_err(map_rusqlite_repository_error)?;
        transaction
            .commit()
            .map_err(map_rusqlite_repository_error)?;
        drop(connection);

        self.find_note(id)?
            .ok_or_else(|| RepositoryError::OperationFailed {
                message: "updated note could not be read back".to_owned(),
            })
    }

    fn delete_note(&self, id: &NoteId) -> Result<bool, RepositoryError> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction()
            .map_err(map_rusqlite_repository_error)?;
        let changed = transaction
            .execute("DELETE FROM notes WHERE id = ?1", params![id.as_str()])
            .map_err(map_rusqlite_repository_error)?;
        cleanup_orphaned_tags(&transaction)?;
        transaction
            .commit()
            .map_err(map_rusqlite_repository_error)?;
        Ok(changed > 0)
    }

    fn list_notes(&self, query: &NoteListQuery) -> Result<PagedNotes, RepositoryError> {
        if query.limit == 0 || query.limit > 500 {
            return Err(RepositoryError::OperationFailed {
                message: "list limit must be between 1 and 500".to_owned(),
            });
        }

        let connection = self.lock()?;
        let mut sql = String::from(
            "SELECT n.id, n.title, n.body, n.created_at, n.updated_at, n.pinned_at
             FROM notes n",
        );
        let mut where_clauses = Vec::new();
        let mut parameters = Vec::<Value>::new();

        for (index, tag) in query.tags.iter().enumerate() {
            where_clauses.push(format!(
                "EXISTS (
                    SELECT 1 FROM note_tags nt{index}
                    WHERE nt{index}.note_id = n.id AND nt{index}.tag_key = ?{}
                )",
                parameters.len() + 1
            ));
            parameters.push(Value::Text(tag.as_str().to_owned()));
        }

        if let Some(since) = query.since {
            where_clauses.push(format!("n.updated_at >= ?{}", parameters.len() + 1));
            parameters.push(Value::Integer(since.seconds()));
        }
        if let Some(until) = query.until {
            where_clauses.push(format!("n.updated_at <= ?{}", parameters.len() + 1));
            parameters.push(Value::Integer(until.seconds()));
        }
        if let Some(substring) = &query.title_substring {
            where_clauses.push(format!(
                "LOWER(n.title) LIKE ?{}",
                parameters.len() + 1
            ));
            parameters.push(Value::Text(format!("%{}%", substring.to_lowercase())));
        }
        if query.pinned_only {
            where_clauses.push("n.pinned_at IS NOT NULL".to_owned());
        }

        if !where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clauses.join(" AND "));
        }

        sql.push_str(" ORDER BY ");
        sql.push_str(note_order_by_clause(query.sort));
        sql.push_str(&format!(" LIMIT ?{}", parameters.len() + 1));
        parameters.push(Value::Integer(i64::try_from(query.limit).map_err(|error| {
            RepositoryError::OperationFailed {
                message: format!("list limit overflow: {error}"),
            }
        })?));

        let mut statement = connection
            .prepare(&sql)
            .map_err(map_rusqlite_repository_error)?;
        let mut rows = statement
            .query(rusqlite::params_from_iter(parameters.iter()))
            .map_err(map_rusqlite_repository_error)?;

        let mut notes = Vec::new();
        while let Some(row) = rows.next().map_err(map_rusqlite_repository_error)? {
            let mut record = map_note_row(row)?;
            record.tags = load_note_tags(&connection, &record.id)?;
            notes.push(record);
        }

        Ok(PagedNotes {
            notes,
            next_cursor: None,
        })
    }

    fn add_tags_to_note(
        &self,
        id: &NoteId,
        tags: &[Tag],
    ) -> Result<TagMutationOutcome, RepositoryError> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction()
            .map_err(map_rusqlite_repository_error)?;
        ensure_note_exists(&transaction, id)?;
        let mut changed = 0_i64;
        for tag in tags {
            transaction
                .execute(
                    "INSERT OR IGNORE INTO tags (key, display) VALUES (?1, ?2)",
                    params![tag.key().as_str(), tag.display()],
                )
                .map_err(map_rusqlite_repository_error)?;
            changed += transaction
                .execute(
                    "INSERT OR IGNORE INTO note_tags (note_id, tag_key) VALUES (?1, ?2)",
                    params![id.as_str(), tag.key().as_str()],
                )
                .map_err(map_rusqlite_repository_error)? as i64;
        }
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

    fn remove_tag_from_note(
        &self,
        id: &NoteId,
        tag: &TagKey,
    ) -> Result<TagMutationOutcome, RepositoryError> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction()
            .map_err(map_rusqlite_repository_error)?;
        ensure_note_exists(&transaction, id)?;
        let changed = transaction
            .execute(
                "DELETE FROM note_tags WHERE note_id = ?1 AND tag_key = ?2",
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

    fn pin_note(&self, id: &NoteId, position: Option<u32>) -> Result<PinOutcome, RepositoryError> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction()
            .map_err(map_rusqlite_repository_error)?;
        ensure_note_exists(&transaction, id)?;

        let pin_position = position.unwrap_or_else(|| next_note_pin_position(&transaction));
        let changed = transaction
            .execute(
                "UPDATE notes SET pinned_at = ?1 WHERE id = ?2",
                params![pin_position, id.as_str()],
            )
            .map_err(map_rusqlite_repository_error)?;
        transaction
            .commit()
            .map_err(map_rusqlite_repository_error)?;

        Ok(PinOutcome {
            existed: true,
            changed: changed > 0,
        })
    }

    fn unpin_note(&self, id: &NoteId) -> Result<PinOutcome, RepositoryError> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction()
            .map_err(map_rusqlite_repository_error)?;
        ensure_note_exists(&transaction, id)?;
        let changed = transaction
            .execute(
                "UPDATE notes SET pinned_at = NULL WHERE id = ?1 AND pinned_at IS NOT NULL",
                params![id.as_str()],
            )
            .map_err(map_rusqlite_repository_error)?;
        transaction
            .commit()
            .map_err(map_rusqlite_repository_error)?;

        Ok(PinOutcome {
            existed: true,
            changed: changed > 0,
        })
    }

    fn search_notes(&self, query: &str) -> Result<Vec<NoteRecord>, RepositoryError> {
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT n.id, n.title, n.body, n.created_at, n.updated_at, n.pinned_at
                 FROM note_search s
                 JOIN notes n ON n.id = s.note_id
                 WHERE note_search MATCH ?1
                 ORDER BY rank
                 LIMIT 100",
            )
            .map_err(map_rusqlite_repository_error)?;
        let mut rows = statement
            .query(params![query])
            .map_err(map_rusqlite_repository_error)?;

        let mut records = Vec::new();
        while let Some(row) = rows.next().map_err(map_rusqlite_repository_error)? {
            let mut record = map_note_row(row)?;
            record.tags = load_note_tags(&connection, &record.id)?;
            records.push(record);
        }
        Ok(records)
    }

    fn search_all(&self, query: &str) -> Result<Vec<SearchHit>, RepositoryError> {
        let files = FileRepository::search_files(self, query)?;
        let notes = self.search_notes(query)?;
        let mut hits = Vec::with_capacity(files.len() + notes.len());
        for file in files {
            hits.push(SearchHit::File(file));
        }
        for note in notes {
            hits.push(SearchHit::Note(note));
        }
        Ok(hits)
    }
}

use tssp_ports::FileRepository;

/// Applies schema version 2 (notes tables and FTS).
///
/// # Errors
///
/// Returns [`SqliteRepositoryError`] when migration SQL fails.
pub(crate) fn migrate_notes_schema(connection: &Connection) -> Result<(), SqliteRepositoryError> {
    if migration_applied(connection, 2)? {
        return Ok(());
    }

    connection
        .execute_batch(
            "
            CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                body TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                pinned_at INTEGER
            );

            CREATE TABLE IF NOT EXISTS note_tags (
                note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
                tag_key TEXT NOT NULL REFERENCES tags(key) ON DELETE CASCADE,
                PRIMARY KEY (note_id, tag_key)
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS note_search
                USING fts5(note_id UNINDEXED, title, body, tags);

            CREATE TRIGGER IF NOT EXISTS notes_ai AFTER INSERT ON notes BEGIN
                INSERT INTO note_search (rowid, note_id, title, body, tags)
                VALUES (new.rowid, new.id, new.title, new.body, '');
            END;

            CREATE TRIGGER IF NOT EXISTS notes_ad AFTER DELETE ON notes BEGIN
                DELETE FROM note_search WHERE note_id = old.id;
            END;

            CREATE TRIGGER IF NOT EXISTS notes_au AFTER UPDATE OF title, body ON notes BEGIN
                UPDATE note_search SET title = new.title, body = new.body WHERE note_id = new.id;
            END;

            CREATE TRIGGER IF NOT EXISTS note_tags_ai AFTER INSERT ON note_tags BEGIN
                UPDATE note_search
                SET tags = tags || ' ' || new.tag_key
                WHERE note_id = new.note_id;
            END;

            CREATE TRIGGER IF NOT EXISTS note_tags_ad AFTER DELETE ON note_tags BEGIN
                UPDATE note_search
                SET tags = (SELECT group_concat(tag_key, ' ') FROM note_tags WHERE note_id = old.note_id)
                WHERE note_id = old.note_id;
            END;
            ",
        )
        .map_err(SqliteRepositoryError::Migration)?;

    record_migration(connection, 2)?;
    Ok(())
}

pub(crate) fn ensure_note_exists(
    transaction: &Transaction<'_>,
    id: &NoteId,
) -> Result<(), RepositoryError> {
    let exists: bool = transaction
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM notes WHERE id = ?1)",
            params![id.as_str()],
            |row| row.get(0),
        )
        .map_err(map_rusqlite_repository_error)?;
    if exists {
        return Ok(());
    }
    Err(RepositoryError::NotFound)
}

pub(crate) fn map_note_row(row: &Row<'_>) -> Result<NoteRecord, RepositoryError> {
    let id = NoteId::new(
        row.get::<_, String>(0)
            .map_err(map_rusqlite_repository_error)?,
    )
    .map_err(|error| map_domain_repository_error(&error))?;
    let title = NoteTitle::new(
        row.get::<_, String>(1)
            .map_err(map_rusqlite_repository_error)?,
    )
    .map_err(|error| map_domain_repository_error(&error))?;
    let body = NoteBody::new(
        row.get::<_, String>(2)
            .map_err(map_rusqlite_repository_error)?,
    )
    .map_err(|error| map_domain_repository_error(&error))?;
    let created_at = UnixTimestamp::new(
        row.get::<_, i64>(3)
            .map_err(map_rusqlite_repository_error)?,
    )
    .map_err(|error| map_domain_repository_error(&error))?;
    let updated_at = UnixTimestamp::new(
        row.get::<_, i64>(4)
            .map_err(map_rusqlite_repository_error)?,
    )
    .map_err(|error| map_domain_repository_error(&error))?;
    let pinned_at = row
        .get::<_, Option<i64>>(5)
        .map_err(map_rusqlite_repository_error)?
        .map(|value| {
            u32::try_from(value).map_err(|error| RepositoryError::OperationFailed {
                message: format!("invalid note pin position: {error}"),
            })
        })
        .transpose()?;

    Ok(NoteRecord {
        id,
        title,
        body,
        created_at,
        updated_at,
        tags: Vec::new(),
        pinned_at,
    })
}

pub(crate) fn load_note_tags(connection: &Connection, id: &NoteId) -> Result<Vec<Tag>, RepositoryError> {
    let mut statement = connection
        .prepare(
            "SELECT tags.display
             FROM tags
             JOIN note_tags ON note_tags.tag_key = tags.key
             WHERE note_tags.note_id = ?1
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

fn insert_note_row(
    transaction: &Transaction<'_>,
    new_note: &NewNoteRecord,
) -> Result<(), RepositoryError> {
    transaction
        .execute(
            "INSERT INTO notes (id, title, body, created_at, updated_at, pinned_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                new_note.id.as_str(),
                new_note.title.as_str(),
                new_note.body.as_str(),
                new_note.created_at.seconds(),
                new_note.updated_at.seconds(),
                new_note.pinned_at,
            ],
        )
        .map(|_rows| ())
        .map_err(map_rusqlite_repository_error)
}

pub(crate) fn insert_note_tags(
    transaction: &Transaction<'_>,
    new_note: &NewNoteRecord,
) -> Result<(), RepositoryError> {
    for tag in &new_note.tags {
        transaction
            .execute(
                "INSERT OR IGNORE INTO tags (key, display) VALUES (?1, ?2)",
                params![tag.key().as_str(), tag.display()],
            )
            .map_err(map_rusqlite_repository_error)?;
        transaction
            .execute(
                "INSERT OR IGNORE INTO note_tags (note_id, tag_key) VALUES (?1, ?2)",
                params![new_note.id.as_str(), tag.key().as_str()],
            )
            .map_err(map_rusqlite_repository_error)?;
    }
    Ok(())
}

fn note_order_by_clause(sort: NoteListSort) -> &'static str {
    match sort {
        NoteListSort::UpdatedDesc => "n.updated_at DESC, n.id DESC",
        NoteListSort::UpdatedAsc => "n.updated_at ASC, n.id ASC",
        NoteListSort::CreatedDesc => "n.created_at DESC, n.id DESC",
        NoteListSort::CreatedAsc => "n.created_at ASC, n.id ASC",
        NoteListSort::TitleAsc => "n.title ASC, n.id ASC",
        NoteListSort::TitleDesc => "n.title DESC, n.id DESC",
    }
}

fn next_note_pin_position(transaction: &Transaction<'_>) -> u32 {
    let max_position: Option<i64> = transaction
        .query_row(
            "SELECT MAX(pinned_at) FROM notes WHERE pinned_at IS NOT NULL",
            [],
            |row| row.get(0),
        )
        .unwrap_or(None);
    u32::try_from(max_position.unwrap_or(0).saturating_add(1)).unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use crate::SqliteFileRepository;
    use tssp_domain::{derive_note_title, NoteBody, NoteId, NoteTitle, Tag, UnixTimestamp};
    use tssp_ports::{NewNoteRecord, NoteListQuery, NoteRepository};

    fn timestamp(seconds: i64) -> UnixTimestamp {
        UnixTimestamp::new(seconds).unwrap_or_else(|error| panic!("timestamp failed: {error}"))
    }

    #[test]
    fn derive_title_matches_domain_helper() {
        assert_eq!(derive_note_title("# Hello\n\nworld"), "Hello");
    }

    #[test]
    fn note_tags_are_idempotent() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("open failed: {error}"));
        let now = timestamp(1_700_000_000);
        let id = NoteId::new("note-tags-1").unwrap_or_else(|error| panic!("{error}"));
        repository
            .insert_note(NewNoteRecord {
                id: id.clone(),
                title: NoteTitle::new("Tagged").unwrap_or_else(|error| panic!("{error}")),
                body: NoteBody::new("Body").unwrap_or_else(|error| panic!("{error}")),
                created_at: now,
                updated_at: now,
                tags: vec![Tag::new("ideas").unwrap_or_else(|error| panic!("{error}"))],
                pinned_at: None,
            })
            .unwrap_or_else(|error| panic!("insert failed: {error}"));

        let tag = Tag::new("ideas").unwrap_or_else(|error| panic!("{error}"));
        let first = repository
            .add_tags_to_note(&id, &[tag.clone()])
            .unwrap_or_else(|error| panic!("add failed: {error}"));
        let second = repository
            .add_tags_to_note(&id, &[tag])
            .unwrap_or_else(|error| panic!("add failed: {error}"));
        assert_eq!(first.changed_count, 0);
        assert_eq!(second.changed_count, 0);
    }

    #[test]
    fn note_crud_round_trip() {
        let repository = SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("open failed: {error}"));
        let now = timestamp(1_700_000_000);
        let body = NoteBody::new("Body text").unwrap_or_else(|error| panic!("{error}"));
        let title =
            NoteTitle::new(derive_note_title(body.as_str())).unwrap_or_else(|error| panic!("{error}"));
        let id = NoteId::new("note-test-1").unwrap_or_else(|error| panic!("{error}"));

        let created = repository
            .insert_note(NewNoteRecord {
                id: id.clone(),
                title,
                body: body.clone(),
                created_at: now,
                updated_at: now,
                tags: vec![],
                pinned_at: None,
            })
            .unwrap_or_else(|error| panic!("insert failed: {error}"));
        assert_eq!(created.body.as_str(), "Body text");

        let listed = repository
            .list_notes(&NoteListQuery::default())
            .unwrap_or_else(|error| panic!("list failed: {error}"));
        assert_eq!(listed.notes.len(), 1);

        assert!(repository
            .delete_note(&id)
            .unwrap_or_else(|error| panic!("delete failed: {error}")));
    }
}
