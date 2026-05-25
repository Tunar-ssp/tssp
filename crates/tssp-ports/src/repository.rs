//! File and note repository port traits.

use std::sync::Arc;

use tssp_domain::{
    ContentHash, FileId, FileName, FileRecord, NoteBody, NoteId, NoteRecord, NoteTitle, Tag,
    TagKey, UnixTimestamp, Visibility,
};

use crate::errors::RepositoryError;
use crate::query::{
    ListQuery, NoteListQuery, PagedFiles, PagedNotes, PinOutcome, RepositoryStats, SearchHit,
    TagMutationOutcome, TagSummary,
};
use crate::record::{DeletedFileRecord, NewFileRecord, NewNoteRecord};

/// Persists and queries file metadata.
pub trait FileRepository {
    /// Inserts a logical file record for an already durable blob.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when metadata cannot be committed.
    fn insert_file(&self, new_file: NewFileRecord) -> Result<FileRecord, RepositoryError>;

    /// Returns one file record by id.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when metadata lookup fails.
    fn find_file(&self, id: &FileId) -> Result<Option<FileRecord>, RepositoryError>;

    /// Returns the oldest file record for a content hash.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when metadata lookup fails.
    fn find_file_by_content_hash(
        &self,
        content_hash: &ContentHash,
    ) -> Result<Option<FileRecord>, RepositoryError>;

    /// Soft-deletes one logical file record and reports remaining blob references.
    ///
    /// Sets deleted_at timestamp without removing the record. The file becomes invisible to normal queries
    /// but can be restored later.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the delete transaction cannot complete.
    fn delete_file(&self, id: &FileId) -> Result<Option<DeletedFileRecord>, RepositoryError>;

    /// Restores a soft-deleted file by clearing its deleted_at timestamp.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the restore cannot be committed, or when the file does not exist.
    fn restore_file(&self, id: &FileId) -> Result<Option<FileRecord>, RepositoryError>;

    /// Returns soft-deleted files older than the given timestamp.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the query fails.
    fn list_deleted_files(&self, older_than: UnixTimestamp) -> Result<Vec<FileRecord>, RepositoryError>;

    /// Permanently deletes a soft-deleted file record.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the delete cannot be committed.
    fn purge_deleted_file(&self, id: &FileId) -> Result<bool, RepositoryError>;

    /// Returns filtered and paginated files according to the supplied query.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the query or cursor cannot be applied.
    fn list_files(&self, query: &ListQuery) -> Result<PagedFiles, RepositoryError>;

    /// Returns recent files in descending order by upload time with an optional limit.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the query fails.
    fn list_files_recent(&self, limit: u64) -> Result<Vec<FileRecord>, RepositoryError>;

    /// Returns recent files containing a specific tag in descending order by upload time with an optional limit.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the query fails.
    fn list_files_by_tag(
        &self,
        tag: &TagKey,
        limit: u64,
    ) -> Result<Vec<FileRecord>, RepositoryError>;

    /// Returns all tags with committed file counts.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the query fails.
    fn list_tags(&self) -> Result<Vec<TagSummary>, RepositoryError>;

    /// Adds tags to an existing file idempotently.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::NotFound`] when the file does not exist, or a
    /// repository failure when the mutation cannot be committed.
    fn add_tags_to_file(
        &self,
        id: &FileId,
        tags: &[Tag],
    ) -> Result<TagMutationOutcome, RepositoryError>;

    /// Removes one tag from an existing file idempotently.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::NotFound`] when the file does not exist, or a
    /// repository failure when the mutation cannot be committed.
    fn remove_tag_from_file(
        &self,
        id: &FileId,
        tag: &TagKey,
    ) -> Result<TagMutationOutcome, RepositoryError>;

    /// Returns aggregate metadata counts for status reporting.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when aggregate queries fail.
    fn stats_since(&self, recent_since: UnixTimestamp) -> Result<RepositoryStats, RepositoryError>;

    /// Pins a file, optionally at a specific position.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::NotFound`] if the file doesn't exist.
    fn pin_file(&self, id: &FileId, position: Option<u32>) -> Result<PinOutcome, RepositoryError>;

    /// Unpins a file.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::NotFound`] if the file doesn't exist.
    fn unpin_file(&self, id: &FileId) -> Result<PinOutcome, RepositoryError>;

    /// Returns pinned files in pin order.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the query fails.
    fn list_pinned_files(&self) -> Result<Vec<FileRecord>, RepositoryError>;

    /// Reorders pins according to the provided list of file IDs.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the operation fails.
    fn reorder_pins(&self, ordered_ids: &[FileId]) -> Result<(), RepositoryError>;

    /// Searches files using full-text search.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the search query fails.
    fn search_files(&self, query: &str) -> Result<Vec<FileRecord>, RepositoryError>;

    /// Renames a file.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::NotFound`] when the file does not exist, or a
    /// repository failure when the mutation cannot be committed.
    fn rename_file(
        &self,
        id: &FileId,
        new_name: &FileName,
    ) -> Result<Option<FileRecord>, RepositoryError>;

    /// Returns file counts grouped by `folder_path`, optionally filtered by owner.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the aggregation query fails.
    fn list_folder_counts(
        &self,
        owner_id: Option<&tssp_domain::UserId>,
    ) -> Result<Vec<(String, u64)>, RepositoryError>;

    /// Updates visibility and optional public link token for one file.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::NotFound`] when the file does not exist.
    fn set_file_visibility(
        &self,
        id: &FileId,
        visibility: Visibility,
        public_token: Option<&str>,
    ) -> Result<Option<FileRecord>, RepositoryError>;

    /// Returns a public file by its link token.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when lookup fails.
    fn find_file_by_public_token(&self, token: &str)
        -> Result<Option<FileRecord>, RepositoryError>;

    /// Renames or moves a logical folder by rewriting `folder_path` prefixes.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the update fails.
    fn update_folder_path_prefix(
        &self,
        from_prefix: &str,
        to_prefix: &str,
    ) -> Result<u64, RepositoryError>;

    /// Sets `folder_path` for one file.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::NotFound`] when the file does not exist.
    fn set_file_folder_path(
        &self,
        id: &FileId,
        folder_path: &str,
    ) -> Result<Option<FileRecord>, RepositoryError>;

    /// Records an audit event for compliance and forensics.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the event cannot be persisted.
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
    ) -> Result<(), RepositoryError>;
}

/// Persists and queries Markdown notes.
pub trait NoteRepository {
    /// Inserts a new note record.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when metadata cannot be committed.
    fn insert_note(&self, new_note: NewNoteRecord) -> Result<NoteRecord, RepositoryError>;

    /// Returns one note by id.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when lookup fails.
    fn find_note(&self, id: &NoteId) -> Result<Option<NoteRecord>, RepositoryError>;

    /// Replaces a note's title and body.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::NotFound`] when the note does not exist.
    fn update_note(
        &self,
        id: &NoteId,
        title: &NoteTitle,
        body: &NoteBody,
        updated_at: UnixTimestamp,
    ) -> Result<NoteRecord, RepositoryError>;

    /// Deletes a note idempotently.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the delete cannot be committed.
    fn delete_note(&self, id: &NoteId) -> Result<bool, RepositoryError>;

    /// Lists notes using the supplied query.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the query fails.
    fn list_notes(&self, query: &NoteListQuery) -> Result<PagedNotes, RepositoryError>;

    /// Adds tags to a note idempotently.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::NotFound`] when the note does not exist.
    fn add_tags_to_note(
        &self,
        id: &NoteId,
        tags: &[Tag],
    ) -> Result<TagMutationOutcome, RepositoryError>;

    /// Removes one tag from a note idempotently.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::NotFound`] when the note does not exist.
    fn remove_tag_from_note(
        &self,
        id: &NoteId,
        tag: &TagKey,
    ) -> Result<TagMutationOutcome, RepositoryError>;

    /// Replaces all tags on a note atomically.
    ///
    /// Deletes existing note tags and inserts the supplied set in one transaction.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::NotFound`] when the note does not exist.
    fn replace_tags_on_note(&self, id: &NoteId, tags: &[Tag]) -> Result<(), RepositoryError>;

    /// Pins a note, optionally at a specific position.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::NotFound`] when the note does not exist.
    fn pin_note(&self, id: &NoteId, position: Option<u32>) -> Result<PinOutcome, RepositoryError>;

    /// Unpins a note.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::NotFound`] when the note does not exist.
    fn unpin_note(&self, id: &NoteId) -> Result<PinOutcome, RepositoryError>;

    /// Full-text search over note titles and bodies.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when the search fails.
    fn search_notes(&self, query: &str) -> Result<Vec<NoteRecord>, RepositoryError>;

    /// Unified ranked search across files and notes.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError`] when either search backend fails.
    fn search_all(&self, query: &str) -> Result<Vec<SearchHit>, RepositoryError>;
}

impl<T> FileRepository for Arc<T>
where
    T: FileRepository,
{
    fn insert_file(&self, new_file: NewFileRecord) -> Result<FileRecord, RepositoryError> {
        self.as_ref().insert_file(new_file)
    }

    fn find_file(&self, id: &FileId) -> Result<Option<FileRecord>, RepositoryError> {
        self.as_ref().find_file(id)
    }

    fn find_file_by_content_hash(
        &self,
        content_hash: &ContentHash,
    ) -> Result<Option<FileRecord>, RepositoryError> {
        self.as_ref().find_file_by_content_hash(content_hash)
    }

    fn delete_file(&self, id: &FileId) -> Result<Option<DeletedFileRecord>, RepositoryError> {
        self.as_ref().delete_file(id)
    }

    fn restore_file(&self, id: &FileId) -> Result<Option<FileRecord>, RepositoryError> {
        self.as_ref().restore_file(id)
    }

    fn list_deleted_files(&self, older_than: UnixTimestamp) -> Result<Vec<FileRecord>, RepositoryError> {
        self.as_ref().list_deleted_files(older_than)
    }

    fn purge_deleted_file(&self, id: &FileId) -> Result<bool, RepositoryError> {
        self.as_ref().purge_deleted_file(id)
    }

    fn list_files(&self, query: &ListQuery) -> Result<PagedFiles, RepositoryError> {
        self.as_ref().list_files(query)
    }

    fn list_files_recent(&self, limit: u64) -> Result<Vec<FileRecord>, RepositoryError> {
        self.as_ref().list_files_recent(limit)
    }

    fn list_files_by_tag(
        &self,
        tag: &TagKey,
        limit: u64,
    ) -> Result<Vec<FileRecord>, RepositoryError> {
        self.as_ref().list_files_by_tag(tag, limit)
    }

    fn list_tags(&self) -> Result<Vec<TagSummary>, RepositoryError> {
        self.as_ref().list_tags()
    }

    fn add_tags_to_file(
        &self,
        id: &FileId,
        tags: &[Tag],
    ) -> Result<TagMutationOutcome, RepositoryError> {
        self.as_ref().add_tags_to_file(id, tags)
    }

    fn remove_tag_from_file(
        &self,
        id: &FileId,
        tag: &TagKey,
    ) -> Result<TagMutationOutcome, RepositoryError> {
        self.as_ref().remove_tag_from_file(id, tag)
    }

    fn stats_since(&self, recent_since: UnixTimestamp) -> Result<RepositoryStats, RepositoryError> {
        self.as_ref().stats_since(recent_since)
    }

    fn pin_file(&self, id: &FileId, position: Option<u32>) -> Result<PinOutcome, RepositoryError> {
        self.as_ref().pin_file(id, position)
    }

    fn unpin_file(&self, id: &FileId) -> Result<PinOutcome, RepositoryError> {
        self.as_ref().unpin_file(id)
    }

    fn list_pinned_files(&self) -> Result<Vec<FileRecord>, RepositoryError> {
        self.as_ref().list_pinned_files()
    }

    fn reorder_pins(&self, ordered_ids: &[FileId]) -> Result<(), RepositoryError> {
        self.as_ref().reorder_pins(ordered_ids)
    }

    fn search_files(&self, query: &str) -> Result<Vec<FileRecord>, RepositoryError> {
        self.as_ref().search_files(query)
    }

    fn rename_file(
        &self,
        id: &FileId,
        new_name: &FileName,
    ) -> Result<Option<FileRecord>, RepositoryError> {
        self.as_ref().rename_file(id, new_name)
    }

    fn list_folder_counts(
        &self,
        owner_id: Option<&tssp_domain::UserId>,
    ) -> Result<Vec<(String, u64)>, RepositoryError> {
        self.as_ref().list_folder_counts(owner_id)
    }

    fn set_file_visibility(
        &self,
        id: &FileId,
        visibility: Visibility,
        public_token: Option<&str>,
    ) -> Result<Option<FileRecord>, RepositoryError> {
        self.as_ref()
            .set_file_visibility(id, visibility, public_token)
    }

    fn find_file_by_public_token(
        &self,
        token: &str,
    ) -> Result<Option<FileRecord>, RepositoryError> {
        self.as_ref().find_file_by_public_token(token)
    }

    fn update_folder_path_prefix(
        &self,
        from_prefix: &str,
        to_prefix: &str,
    ) -> Result<u64, RepositoryError> {
        self.as_ref()
            .update_folder_path_prefix(from_prefix, to_prefix)
    }

    fn set_file_folder_path(
        &self,
        id: &FileId,
        folder_path: &str,
    ) -> Result<Option<FileRecord>, RepositoryError> {
        self.as_ref().set_file_folder_path(id, folder_path)
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
        self.as_ref()
            .insert_audit_event(id, timestamp, user_id, action, resource, resource_id, status, details)
    }
}

impl<T> NoteRepository for Arc<T>
where
    T: NoteRepository,
{
    fn insert_note(&self, new_note: NewNoteRecord) -> Result<NoteRecord, RepositoryError> {
        self.as_ref().insert_note(new_note)
    }

    fn find_note(&self, id: &NoteId) -> Result<Option<NoteRecord>, RepositoryError> {
        self.as_ref().find_note(id)
    }

    fn update_note(
        &self,
        id: &NoteId,
        title: &NoteTitle,
        body: &NoteBody,
        updated_at: UnixTimestamp,
    ) -> Result<NoteRecord, RepositoryError> {
        self.as_ref().update_note(id, title, body, updated_at)
    }

    fn delete_note(&self, id: &NoteId) -> Result<bool, RepositoryError> {
        self.as_ref().delete_note(id)
    }

    fn list_notes(&self, query: &NoteListQuery) -> Result<PagedNotes, RepositoryError> {
        self.as_ref().list_notes(query)
    }

    fn add_tags_to_note(
        &self,
        id: &NoteId,
        tags: &[Tag],
    ) -> Result<TagMutationOutcome, RepositoryError> {
        self.as_ref().add_tags_to_note(id, tags)
    }

    fn remove_tag_from_note(
        &self,
        id: &NoteId,
        tag: &TagKey,
    ) -> Result<TagMutationOutcome, RepositoryError> {
        self.as_ref().remove_tag_from_note(id, tag)
    }

    fn replace_tags_on_note(&self, id: &NoteId, tags: &[Tag]) -> Result<(), RepositoryError> {
        self.as_ref().replace_tags_on_note(id, tags)
    }

    fn pin_note(&self, id: &NoteId, position: Option<u32>) -> Result<PinOutcome, RepositoryError> {
        self.as_ref().pin_note(id, position)
    }

    fn unpin_note(&self, id: &NoteId) -> Result<PinOutcome, RepositoryError> {
        self.as_ref().unpin_note(id)
    }

    fn search_notes(&self, query: &str) -> Result<Vec<NoteRecord>, RepositoryError> {
        self.as_ref().search_notes(query)
    }

    fn search_all(&self, query: &str) -> Result<Vec<SearchHit>, RepositoryError> {
        self.as_ref().search_all(query)
    }
}
