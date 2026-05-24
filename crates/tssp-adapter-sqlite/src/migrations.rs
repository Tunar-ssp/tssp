//! Schema migrations for the `SQLite` metadata store.
//!
//! Each migration is idempotent — guarded by `migration_applied`. The baseline
//! schema is v1; incremental migrations add columns, indexes, or tables.

use rusqlite::{params, Connection};

use crate::notes;
use crate::SqliteRepositoryError;

pub(crate) fn run_migrations(connection: &Connection) -> Result<(), SqliteRepositoryError> {
    connection
        .execute_batch(
            "
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            ) STRICT;

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
            ) STRICT;

            CREATE TABLE IF NOT EXISTS tags (
                key TEXT PRIMARY KEY,
                display TEXT NOT NULL
            ) STRICT;

            CREATE TABLE IF NOT EXISTS file_tags (
                file_id TEXT NOT NULL REFERENCES files(id) ON DELETE CASCADE,
                tag_key TEXT NOT NULL REFERENCES tags(key) ON DELETE CASCADE,
                PRIMARY KEY (file_id, tag_key)
            ) STRICT;

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

            CREATE TABLE IF NOT EXISTS sessions (
                token TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                source_file TEXT REFERENCES files(id) ON DELETE SET NULL,
                received_file TEXT REFERENCES files(id) ON DELETE SET NULL,
                expected_name TEXT,
                used_at INTEGER
            ) STRICT;

            CREATE INDEX IF NOT EXISTS sessions_expires_at ON sessions(expires_at);
            CREATE INDEX IF NOT EXISTS sessions_kind ON sessions(kind);

            INSERT OR IGNORE INTO schema_migrations (version) VALUES (1);
            ",
        )
        .map_err(SqliteRepositoryError::Migration)?;
    notes::migrate_notes_schema(connection)?;
    migrate_folders_schema(connection)?;
    migrate_cloud_schema(connection)?;
    migrate_search_indexes(connection)?;
    migrate_content_hash_index(connection)?;
    migrate_workspace_documents_schema(connection)
}

/// Adds ownership, visibility, and public link columns (schema v7/v8).
pub(crate) fn migrate_cloud_schema(connection: &Connection) -> Result<(), SqliteRepositoryError> {
    if !migration_applied(connection, 7)? {
        connection
            .execute_batch(
                "
                ALTER TABLE files ADD COLUMN owner_id TEXT;
                ALTER TABLE files ADD COLUMN visibility TEXT NOT NULL DEFAULT 'private';
                ALTER TABLE files ADD COLUMN public_token TEXT;
                CREATE UNIQUE INDEX IF NOT EXISTS idx_files_public_token ON files(public_token)
                    WHERE public_token IS NOT NULL;
                CREATE INDEX IF NOT EXISTS idx_files_owner ON files(owner_id);
                CREATE INDEX IF NOT EXISTS idx_files_visibility ON files(visibility);
                ",
            )
            .map_err(SqliteRepositoryError::Migration)?;

        record_migration(connection, 7)?;
    }

    if !migration_applied(connection, 8)? {
        connection
            .execute_batch(
                "
                ALTER TABLE notes ADD COLUMN owner_id TEXT;
                ALTER TABLE notes ADD COLUMN visibility TEXT NOT NULL DEFAULT 'private';
                CREATE INDEX IF NOT EXISTS idx_notes_owner ON notes(owner_id);

                CREATE TABLE IF NOT EXISTS workspaces (
                    id TEXT PRIMARY KEY,
                    owner_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    language TEXT NOT NULL DEFAULT 'text',
                    body TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                ) STRICT;
                CREATE INDEX IF NOT EXISTS idx_workspaces_owner ON workspaces(owner_id);
                ",
            )
            .map_err(SqliteRepositoryError::Migration)?;

        record_migration(connection, 8)?;
    }
    Ok(())
}

/// Adds indexes used by bounded fuzzy search candidate queries (schema v9).
pub(crate) fn migrate_search_indexes(connection: &Connection) -> Result<(), SqliteRepositoryError> {
    if migration_applied(connection, 9)? {
        return Ok(());
    }

    connection
        .execute_batch(
            "
            CREATE INDEX IF NOT EXISTS idx_files_name_nocase ON files(name COLLATE NOCASE);
            CREATE INDEX IF NOT EXISTS idx_files_mime_nocase ON files(mime_type COLLATE NOCASE);
            CREATE INDEX IF NOT EXISTS idx_notes_title_nocase ON notes(title COLLATE NOCASE);
            CREATE INDEX IF NOT EXISTS idx_file_tags_tag_key ON file_tags(tag_key);
            CREATE INDEX IF NOT EXISTS idx_note_tags_tag_key ON note_tags(tag_key);
            ",
        )
        .map_err(SqliteRepositoryError::Migration)?;

    record_migration(connection, 9)?;
    Ok(())
}

/// Adds `folder_path` to files (schema v4).
pub(crate) fn migrate_folders_schema(connection: &Connection) -> Result<(), SqliteRepositoryError> {
    if migration_applied(connection, 4)? {
        return Ok(());
    }

    connection
        .execute_batch(
            "
            ALTER TABLE files ADD COLUMN folder_path TEXT NOT NULL DEFAULT '';
            CREATE INDEX IF NOT EXISTS idx_files_folder_path ON files(folder_path);
            ",
        )
        .map_err(SqliteRepositoryError::Migration)?;

    record_migration(connection, 4)?;
    Ok(())
}

/// Adds an index on `content_hash` for fast deduplication lookups (schema v10).
pub(crate) fn migrate_content_hash_index(
    connection: &Connection,
) -> Result<(), SqliteRepositoryError> {
    if migration_applied(connection, 10)? {
        return Ok(());
    }
    connection
        .execute_batch("CREATE INDEX IF NOT EXISTS idx_files_content_hash ON files(content_hash);")
        .map_err(SqliteRepositoryError::Migration)?;
    record_migration(connection, 10)?;
    Ok(())
}

/// Adds normalized workspace document storage for the admin editor (schema v11).
pub(crate) fn migrate_workspace_documents_schema(
    connection: &Connection,
) -> Result<(), SqliteRepositoryError> {
    if migration_applied(connection, 11)? {
        return Ok(());
    }

    connection
        .execute_batch(
            "
            CREATE TABLE IF NOT EXISTS workspace_documents (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
                owner_id TEXT NOT NULL,
                path TEXT NOT NULL,
                language TEXT NOT NULL DEFAULT 'text',
                body TEXT NOT NULL,
                is_primary INTEGER NOT NULL DEFAULT 0 CHECK (is_primary IN (0, 1)),
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                UNIQUE(workspace_id, path)
            ) STRICT;

            CREATE INDEX IF NOT EXISTS idx_workspace_documents_workspace
                ON workspace_documents(workspace_id, updated_at DESC);
            CREATE INDEX IF NOT EXISTS idx_workspace_documents_owner
                ON workspace_documents(owner_id, updated_at DESC);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_workspace_documents_primary
                ON workspace_documents(workspace_id)
                WHERE is_primary = 1;

            INSERT INTO workspace_documents (
                id,
                workspace_id,
                owner_id,
                path,
                language,
                body,
                is_primary,
                created_at,
                updated_at
            )
            SELECT
                'wdoc-' || lower(hex(randomblob(16))),
                workspaces.id,
                workspaces.owner_id,
                CASE
                    WHEN workspaces.language = 'markdown' THEN 'main.md'
                    WHEN workspaces.language = 'rust' THEN 'main.rs'
                    WHEN workspaces.language = 'python' THEN 'main.py'
                    WHEN workspaces.language = 'javascript' THEN 'main.js'
                    WHEN workspaces.language = 'typescript' THEN 'main.ts'
                    WHEN workspaces.language = 'json' THEN 'main.json'
                    WHEN workspaces.language = 'yaml' THEN 'main.yaml'
                    WHEN workspaces.language = 'toml' THEN 'main.toml'
                    WHEN workspaces.language = 'html' THEN 'main.html'
                    WHEN workspaces.language = 'css' THEN 'main.css'
                    WHEN workspaces.language = 'sql' THEN 'main.sql'
                    WHEN workspaces.language = 'bash' THEN 'main.sh'
                    ELSE 'main.txt'
                END,
                workspaces.language,
                workspaces.body,
                1,
                workspaces.created_at,
                workspaces.updated_at
            FROM workspaces
            WHERE NOT EXISTS (
                SELECT 1
                FROM workspace_documents
                WHERE workspace_documents.workspace_id = workspaces.id
            );
            ",
        )
        .map_err(SqliteRepositoryError::Migration)?;

    record_migration(connection, 11)?;
    Ok(())
}

pub(crate) fn migration_applied(
    connection: &Connection,
    version: i64,
) -> Result<bool, SqliteRepositoryError> {
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM schema_migrations WHERE version = ?1",
            params![version],
            |row| row.get(0),
        )
        .map_err(SqliteRepositoryError::Migration)?;
    Ok(count > 0)
}

pub(crate) fn record_migration(
    connection: &Connection,
    version: i64,
) -> Result<(), SqliteRepositoryError> {
    connection
        .execute(
            "INSERT OR IGNORE INTO schema_migrations (version) VALUES (?1)",
            params![version],
        )
        .map(|_rows| ())
        .map_err(SqliteRepositoryError::Migration)
}
