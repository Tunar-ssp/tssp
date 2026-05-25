//! `SQLite` persistence for Markdown notes and unified search.

use std::collections::HashSet;
use std::fmt::Write as _;

use rusqlite::{params, types::Value, Connection, Row, Transaction};
use tssp_domain::{
    search_terms, FileRecord, NoteBody, NoteId, NoteRecord, NoteTitle, Tag, TagKey, UnixTimestamp,
    UserId,
};
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
        let mut connection = self.connect()?;
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
        let connection = self.connect()?;
        let mut statement = connection
            .prepare(
                "SELECT id, title, body, created_at, updated_at, pinned_at, folder_path, owner_id
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
        let mut connection = self.connect()?;
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
        let mut connection = self.connect()?;
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

        let connection = self.connect()?;
        let mut sql = String::from(
            "SELECT n.id, n.title, substr(n.body, 1, 200) as body, n.created_at, n.updated_at, n.pinned_at, n.folder_path, n.owner_id
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
            where_clauses.push(format!("LOWER(n.title) LIKE ?{}", parameters.len() + 1));
            parameters.push(Value::Text(format!("%{}%", substring.to_lowercase())));
        }
        if query.pinned_only {
            where_clauses.push("n.pinned_at IS NOT NULL".to_owned());
        }
        if let Some(owner_id) = &query.owner_id {
            where_clauses.push(format!("n.owner_id = ?{}", parameters.len() + 1));
            parameters.push(Value::Text(owner_id.as_str().to_owned()));
        }

        if !where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clauses.join(" AND "));
        }

        sql.push_str(" ORDER BY ");
        sql.push_str(note_order_by_clause(query.sort));
        write!(sql, " LIMIT ?{}", parameters.len() + 1).map_err(|error| {
            RepositoryError::OperationFailed {
                message: format!("could not build notes list query: {error}"),
            }
        })?;
        parameters.push(Value::Integer(i64::try_from(query.limit).map_err(
            |error| RepositoryError::OperationFailed {
                message: format!("list limit overflow: {error}"),
            },
        )?));

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
        let mut connection = self.connect()?;
        let transaction = connection
            .transaction()
            .map_err(map_rusqlite_repository_error)?;
        ensure_note_exists(&transaction, id)?;
        let mut changed = 0_u64;
        for tag in tags {
            transaction
                .execute(
                    "INSERT OR IGNORE INTO tags (key, display) VALUES (?1, ?2)",
                    params![tag.key().as_str(), tag.display()],
                )
                .map_err(map_rusqlite_repository_error)?;
            let rows = transaction
                .execute(
                    "INSERT OR IGNORE INTO note_tags (note_id, tag_key) VALUES (?1, ?2)",
                    params![id.as_str(), tag.key().as_str()],
                )
                .map_err(map_rusqlite_repository_error)?;
            changed = changed
                .checked_add(u64::try_from(rows).map_err(|error| {
                    RepositoryError::OperationFailed {
                        message: format!("tag mutation count is invalid: {error}"),
                    }
                })?)
                .ok_or_else(|| RepositoryError::OperationFailed {
                    message: "tag mutation count overflow".to_owned(),
                })?;
        }
        transaction
            .commit()
            .map_err(map_rusqlite_repository_error)?;
        Ok(TagMutationOutcome {
            changed_count: changed,
        })
    }

    fn remove_tag_from_note(
        &self,
        id: &NoteId,
        tag: &TagKey,
    ) -> Result<TagMutationOutcome, RepositoryError> {
        let mut connection = self.connect()?;
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

    fn replace_tags_on_note(&self, id: &NoteId, tags: &[Tag]) -> Result<(), RepositoryError> {
        let mut connection = self.connect()?;
        let transaction = connection
            .transaction()
            .map_err(map_rusqlite_repository_error)?;
        ensure_note_exists(&transaction, id)?;
        transaction
            .execute(
                "DELETE FROM note_tags WHERE note_id = ?1",
                params![id.as_str()],
            )
            .map_err(map_rusqlite_repository_error)?;
        for tag in tags {
            transaction
                .execute(
                    "INSERT OR IGNORE INTO tags (key, display) VALUES (?1, ?2)",
                    params![tag.key().as_str(), tag.display()],
                )
                .map_err(map_rusqlite_repository_error)?;
            transaction
                .execute(
                    "INSERT OR IGNORE INTO note_tags (note_id, tag_key) VALUES (?1, ?2)",
                    params![id.as_str(), tag.key().as_str()],
                )
                .map_err(map_rusqlite_repository_error)?;
        }
        cleanup_orphaned_tags(&transaction)?;
        transaction
            .commit()
            .map_err(map_rusqlite_repository_error)?;
        Ok(())
    }

    fn pin_note(&self, id: &NoteId, position: Option<u32>) -> Result<PinOutcome, RepositoryError> {
        let mut connection = self.connect()?;
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
        let mut connection = self.connect()?;
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
        let connection = self.connect()?;
        let mut statement = connection
            .prepare(
                "SELECT n.id, n.title, n.body, n.created_at, n.updated_at, n.pinned_at, n.folder_path, n.owner_id
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
        let terms = search_terms(query);
        if terms.is_empty() {
            return Ok(Vec::new());
        }

        let fts_query = tssp_domain::build_fts_query(query);
        if fts_query.is_empty() {
            return Ok(Vec::new());
        }
        let files = FileRepository::search_files(self, &fts_query)?;
        let notes = self.search_notes(&fts_query)?;
        let mut seen = HashSet::new();
        let mut hits = Vec::with_capacity(files.len() + notes.len());
        for file in files {
            let key = format!("file:{}", file.id.as_str());
            if seen.insert(key) {
                hits.push(SearchHit::File(file));
            }
        }
        for note in notes {
            let key = format!("note:{}", note.id.as_str());
            if seen.insert(key) {
                hits.push(SearchHit::Note(note));
            }
        }

        let connection = self.connect()?;
        for file in fuzzy_file_candidates(&connection, &terms)? {
            let key = format!("file:{}", file.id.as_str());
            if seen.insert(key) {
                hits.push(SearchHit::File(file));
            }
        }
        for note in fuzzy_note_candidates(&connection, &terms)? {
            let key = format!("note:{}", note.id.as_str());
            if seen.insert(key) {
                hits.push(SearchHit::Note(note));
            }
        }
        hits.sort_by(|left, right| {
            search_hit_score(right, &terms).cmp(&search_hit_score(left, &terms))
        });
        hits.truncate(100);
        Ok(hits)
    }
}

use tssp_ports::FileRepository;

const FUZZY_CANDIDATE_LIMIT: i64 = 200;

fn fuzzy_file_candidates(
    connection: &Connection,
    terms: &[String],
) -> Result<Vec<FileRecord>, RepositoryError> {
    let Some(prefix) = fuzzy_prefix(terms) else {
        return Ok(Vec::new());
    };
    let like_prefix = format!("{prefix}%");
    let mut statement = connection
        .prepare(
            "SELECT f.id, f.name, f.size_bytes, f.content_hash, f.mime_type, f.storage_handle, f.uploaded_at, f.pinned_at, f.folder_path, f.owner_id, f.visibility, f.public_token, f.public_expires_at
             FROM files f
             WHERE f.name LIKE ?1 COLLATE NOCASE
                OR f.mime_type LIKE ?1 COLLATE NOCASE
                OR EXISTS (
                    SELECT 1 FROM file_tags ft
                    WHERE ft.file_id = f.id AND ft.tag_key LIKE ?1 COLLATE NOCASE
                )
             ORDER BY f.uploaded_at DESC, f.id DESC
             LIMIT ?2",
        )
        .map_err(map_rusqlite_repository_error)?;
    let mut rows = statement
        .query(params![like_prefix, FUZZY_CANDIDATE_LIMIT])
        .map_err(map_rusqlite_repository_error)?;
    let mut records = Vec::new();
    while let Some(row) = rows.next().map_err(map_rusqlite_repository_error)? {
        let mut record = crate::map_file_row(row)?;
        let id = record.id.clone();
        record.tags = crate::load_tags(connection, &id)?;
        if file_matches_fuzzy(&record, terms) {
            records.push(record);
        }
    }
    Ok(records)
}

fn fuzzy_note_candidates(
    connection: &Connection,
    terms: &[String],
) -> Result<Vec<NoteRecord>, RepositoryError> {
    let Some(prefix) = fuzzy_prefix(terms) else {
        return Ok(Vec::new());
    };
    let like_prefix = format!("{prefix}%");
    let mut statement = connection
        .prepare(
            "SELECT n.id, n.title, n.body, n.created_at, n.updated_at, n.pinned_at, n.folder_path, n.owner_id
             FROM notes n
             WHERE n.title LIKE ?1 COLLATE NOCASE
                OR EXISTS (
                    SELECT 1 FROM note_tags nt
                    WHERE nt.note_id = n.id AND nt.tag_key LIKE ?1 COLLATE NOCASE
                )
             ORDER BY n.updated_at DESC, n.id DESC
             LIMIT ?2",
        )
        .map_err(map_rusqlite_repository_error)?;
    let mut rows = statement
        .query(params![like_prefix, FUZZY_CANDIDATE_LIMIT])
        .map_err(map_rusqlite_repository_error)?;
    let mut records = Vec::new();
    while let Some(row) = rows.next().map_err(map_rusqlite_repository_error)? {
        let mut record = map_note_row(row)?;
        record.tags = load_note_tags(connection, &record.id)?;
        if note_matches_fuzzy(&record, terms) {
            records.push(record);
        }
    }
    Ok(records)
}

fn fuzzy_prefix(terms: &[String]) -> Option<String> {
    terms.iter().find_map(|term| {
        let prefix: String = term.chars().take(2).collect();
        (prefix.len() >= 2).then_some(prefix)
    })
}

fn file_matches_fuzzy(record: &FileRecord, terms: &[String]) -> bool {
    terms.iter().all(|term| {
        text_matches_term(record.name.original(), term)
            || text_matches_term(record.mime_type.as_str(), term)
            || record
                .tags
                .iter()
                .any(|tag| text_matches_term(tag.display(), term))
    })
}

fn note_matches_fuzzy(record: &NoteRecord, terms: &[String]) -> bool {
    terms.iter().all(|term| {
        text_matches_term(record.title.as_str(), term)
            || record
                .tags
                .iter()
                .any(|tag| text_matches_term(tag.display(), term))
    })
}

fn search_hit_score(hit: &SearchHit, terms: &[String]) -> u64 {
    match hit {
        SearchHit::File(file) => {
            let tag_score = file
                .tags
                .iter()
                .map(|tag| text_score(tag.display(), terms))
                .max()
                .unwrap_or(0);
            text_score(file.name.original(), terms)
                .saturating_mul(4)
                .saturating_add(text_score(file.mime_type.as_str(), terms))
                .saturating_add(tag_score.saturating_mul(2))
                .saturating_add(u64::from(file.is_pinned()) * 25)
                .saturating_add(u64::from(file.visibility == tssp_domain::Visibility::Public) * 5)
        }
        SearchHit::Note(note) => {
            let tag_score = note
                .tags
                .iter()
                .map(|tag| text_score(tag.display(), terms))
                .max()
                .unwrap_or(0);
            text_score(note.title.as_str(), terms)
                .saturating_mul(4)
                .saturating_add(text_score(note.body.as_str(), terms) / 2)
                .saturating_add(tag_score.saturating_mul(2))
                .saturating_add(u64::from(note.pinned_at.is_some()) * 25)
        }
    }
}

fn text_score(text: &str, terms: &[String]) -> u64 {
    terms
        .iter()
        .map(|term| {
            let normalized = text.to_lowercase();
            if normalized == *term {
                return 1_000;
            }
            let mut best = if normalized.starts_with(term) {
                700
            } else if normalized.contains(term) {
                350
            } else {
                0
            };
            for word in normalized.split(|ch: char| !ch.is_alphanumeric()) {
                if word == term {
                    best = best.max(900);
                } else if word.starts_with(term) {
                    best = best.max(650);
                } else if term.len() >= 4 && edit_distance_at_most_two(word, term) {
                    best = best.max(300);
                }
            }
            best
        })
        .sum()
}

fn text_matches_term(text: &str, term: &str) -> bool {
    text_score(text, &[term.to_owned()]) > 0
}

fn edit_distance_at_most_two(left: &str, right: &str) -> bool {
    let left_chars = left.chars().collect::<Vec<_>>();
    let right_chars = right.chars().collect::<Vec<_>>();
    if left_chars.len().abs_diff(right_chars.len()) > 2 {
        return false;
    }
    bounded_levenshtein(&left_chars, &right_chars, 2) <= 2
}

fn bounded_levenshtein(left: &[char], right: &[char], max_distance: usize) -> usize {
    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    let mut current = vec![0; right.len() + 1];

    for (left_index, left_char) in left.iter().enumerate() {
        current[0] = left_index + 1;
        let mut row_min = current[0];
        for (right_index, right_char) in right.iter().enumerate() {
            let substitution = usize::from(left_char != right_char);
            current[right_index + 1] = (previous[right_index + 1] + 1)
                .min(current[right_index] + 1)
                .min(previous[right_index] + substitution);
            row_min = row_min.min(current[right_index + 1]);
        }
        if row_min > max_distance {
            return max_distance + 1;
        }
        std::mem::swap(&mut previous, &mut current);
    }

    previous[right.len()]
}

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
            ) STRICT;

            CREATE TABLE IF NOT EXISTS note_tags (
                note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
                tag_key TEXT NOT NULL REFERENCES tags(key) ON DELETE CASCADE,
                PRIMARY KEY (note_id, tag_key)
            ) STRICT;

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

/// Adds `folder_path` to notes (schema v12).
pub(crate) fn migrate_notes_folders(connection: &Connection) -> Result<(), SqliteRepositoryError> {
    if migration_applied(connection, 12)? {
        return Ok(());
    }

    connection
        .execute_batch(
            "
            ALTER TABLE notes ADD COLUMN folder_path TEXT NOT NULL DEFAULT '';
            CREATE INDEX IF NOT EXISTS idx_notes_folder_path ON notes(folder_path);
            ",
        )
        .map_err(SqliteRepositoryError::Migration)?;

    record_migration(connection, 12)?;
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
    let owner_id_raw: Option<String> = row
        .get(7)
        .map_err(map_rusqlite_repository_error)?;
    let owner_id = owner_id_raw
        .map(|value| UserId::new(value).map_err(|error| map_domain_repository_error(&error)))
        .transpose()?;

    Ok(NoteRecord {
        id,
        title,
        body,
        created_at,
        updated_at,
        tags: Vec::new(),
        pinned_at,
        folder_path: row.get(6).map_err(map_rusqlite_repository_error)?,
        owner_id,
    })
}

pub(crate) fn load_note_tags(
    connection: &Connection,
    id: &NoteId,
) -> Result<Vec<Tag>, RepositoryError> {
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
            "INSERT INTO notes (id, title, body, created_at, updated_at, pinned_at, folder_path, owner_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                new_note.id.as_str(),
                new_note.title.as_str(),
                new_note.body.as_str(),
                new_note.created_at.seconds(),
                new_note.updated_at.seconds(),
                new_note.pinned_at,
                new_note.folder_path,
                new_note.owner_id.as_ref().map(tssp_domain::UserId::as_str),
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
                folder_path: String::new(),
                owner_id: None,
            })
            .unwrap_or_else(|error| panic!("insert failed: {error}"));

        let tag = Tag::new("ideas").unwrap_or_else(|error| panic!("{error}"));
        let first = repository
            .add_tags_to_note(&id, std::slice::from_ref(&tag))
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
        let title = NoteTitle::new(derive_note_title(body.as_str()))
            .unwrap_or_else(|error| panic!("{error}"));
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
                folder_path: String::new(),
                owner_id: None,
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
