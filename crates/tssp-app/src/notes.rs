//! Note lifecycle use cases.

use thiserror::Error;
use tssp_domain::{derive_note_title, DomainError, NoteBody, NoteId, NoteRecord, NoteTitle, Tag};
use tssp_ports::{
    Clock, IdGenerator, NewNoteRecord, NoteListQuery, NoteRepository, PagedNotes, PinOutcome,
    RepositoryError, TagMutationOutcome,
};

/// Coordinates note CRUD and tag mutations.
pub struct NoteService<R, C, G> {
    repository: R,
    clock: C,
    id_generator: G,
}

impl<R, C, G> NoteService<R, C, G> {
    /// Creates a note service from explicit ports.
    #[must_use]
    pub const fn new(repository: R, clock: C, id_generator: G) -> Self {
        Self {
            repository,
            clock,
            id_generator,
        }
    }
}

/// Input for creating a note.
#[derive(Debug, Clone)]
pub struct CreateNoteRequest {
    /// Optional explicit title.
    pub title: Option<String>,
    /// Markdown body.
    pub body: String,
    /// Initial tags.
    pub tags: Vec<String>,
    /// Pin immediately when `Some(position)` or at end when `Some` with default.
    pub pin: bool,
}

/// Input for replacing a note.
#[derive(Debug, Clone)]
pub struct UpdateNoteRequest {
    /// Optional new title.
    pub title: Option<String>,
    /// Replacement Markdown body.
    pub body: String,
}

impl<R, C, G> NoteService<R, C, G>
where
    R: NoteRepository,
    C: Clock,
    G: IdGenerator,
{
    /// Creates a note with optional title derivation.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] when validation or persistence fails.
    pub fn create_note(&self, request: CreateNoteRequest) -> Result<NoteRecord, NoteError> {
        let body = NoteBody::new(request.body)?;
        let title_text = request
            .title
            .unwrap_or_else(|| derive_note_title(body.as_str()));
        let title = NoteTitle::new(title_text)?;
        let tags = normalize_tags(&request.tags)?;
        let now = self.clock.now();
        let id = NoteId::new(
            self.id_generator
                .new_file_id()
                .map_err(|error| NoteError::IdGeneration(error.message))?
                .as_str(),
        )?;
        let pinned_at = request.pin.then_some(1_u32);

        self.repository
            .insert_note(NewNoteRecord {
                id,
                title,
                body,
                created_at: now,
                updated_at: now,
                tags,
                pinned_at,
                folder_path: String::new(),
            })
            .map_err(NoteError::Repository)
    }

    /// Returns one note by id.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] when lookup fails.
    pub fn get_note(&self, id: &NoteId) -> Result<Option<NoteRecord>, NoteError> {
        self.repository.find_note(id).map_err(NoteError::Repository)
    }

    /// Replaces a note body and optional title.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] when the note is missing or validation fails.
    pub fn update_note(
        &self,
        id: &NoteId,
        request: UpdateNoteRequest,
    ) -> Result<NoteRecord, NoteError> {
        let existing = self
            .repository
            .find_note(id)
            .map_err(NoteError::Repository)?
            .ok_or(NoteError::NotFound)?;

        let body = NoteBody::new(request.body)?;
        let title = match request.title {
            Some(value) => NoteTitle::new(value)?,
            None => existing.title,
        };
        let updated_at = self.clock.now();

        self.repository
            .update_note(id, &title, &body, updated_at)
            .map_err(NoteError::Repository)
    }

    /// Deletes a note.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] when deletion fails.
    pub fn delete_note(&self, id: &NoteId) -> Result<bool, NoteError> {
        self.repository
            .delete_note(id)
            .map_err(NoteError::Repository)
    }

    /// Lists notes using the supplied query.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] when listing fails.
    pub fn list_notes(&self, query: &NoteListQuery) -> Result<PagedNotes, NoteError> {
        self.repository
            .list_notes(query)
            .map_err(NoteError::Repository)
    }

    /// Adds tags to a note idempotently.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] when validation or mutation fails.
    pub fn add_tags(&self, id: &NoteId, tags: &[&str]) -> Result<TagMutationOutcome, NoteError> {
        let tags = normalize_tag_refs(tags)?;
        self.repository
            .add_tags_to_note(id, &tags)
            .map_err(NoteError::Repository)
    }

    /// Removes one tag from a note.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] when validation or mutation fails.
    pub fn remove_tag(&self, id: &NoteId, tag: &str) -> Result<TagMutationOutcome, NoteError> {
        let key = tssp_domain::TagKey::new(tag)?;
        self.repository
            .remove_tag_from_note(id, &key)
            .map_err(NoteError::Repository)
    }

    /// Replaces all tags on a note atomically.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] when validation or mutation fails.
    pub fn replace_tags(&self, id: &NoteId, tags: &[&str]) -> Result<(), NoteError> {
        let tags = normalize_tag_refs(tags)?;
        self.repository
            .replace_tags_on_note(id, &tags)
            .map_err(NoteError::Repository)
    }

    /// Pins a note.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] when the note is missing.
    pub fn pin_note(&self, id: &NoteId, position: Option<u32>) -> Result<PinOutcome, NoteError> {
        self.repository
            .pin_note(id, position)
            .map_err(NoteError::Repository)
    }

    /// Unpins a note.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] when the note is missing.
    pub fn unpin_note(&self, id: &NoteId) -> Result<PinOutcome, NoteError> {
        self.repository
            .unpin_note(id)
            .map_err(NoteError::Repository)
    }
}

/// Note use-case failure.
#[derive(Debug, Error)]
pub enum NoteError {
    /// Invalid request payload.
    #[error(transparent)]
    InvalidRequest(#[from] DomainError),

    /// Note was not found.
    #[error("note was not found")]
    NotFound,

    /// Identifier generation failed.
    #[error("could not generate note id: {0}")]
    IdGeneration(String),

    /// Metadata repository failure.
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

fn normalize_tags(tags: &[String]) -> Result<Vec<Tag>, DomainError> {
    let mut normalized = Vec::with_capacity(tags.len());
    for tag in tags {
        let parsed = Tag::new(tag.as_str())?;
        if !normalized
            .iter()
            .any(|existing: &Tag| existing.key() == parsed.key())
        {
            normalized.push(parsed);
        }
    }
    Ok(normalized)
}

fn normalize_tag_refs(tags: &[&str]) -> Result<Vec<Tag>, DomainError> {
    normalize_tags(&tags.iter().map(|tag| (*tag).to_owned()).collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use tssp_adapter_sqlite::SqliteFileRepository;
    use tssp_adapter_system::{SystemClock, UuidV7FileIdGenerator};
    use tssp_domain::{NoteBody, NoteId, NoteTitle, UnixTimestamp};
    use tssp_ports::{NewNoteRecord, NoteListQuery, NoteRepository};

    use super::{CreateNoteRequest, NoteService, UpdateNoteRequest};

    fn open_repo() -> SqliteFileRepository {
        SqliteFileRepository::open_in_memory()
            .unwrap_or_else(|error| panic!("repository open failed: {error}"))
    }

    #[test]
    fn create_note_derives_title_from_markdown_heading() {
        let service = NoteService::new(open_repo(), SystemClock, UuidV7FileIdGenerator);
        let record = service
            .create_note(CreateNoteRequest {
                title: None,
                body: "# Weekly\n\nTasks".to_owned(),
                tags: vec!["journal".to_owned()],
                pin: false,
            })
            .unwrap_or_else(|error| panic!("create failed: {error}"));

        assert_eq!(record.title.as_str(), "Weekly");
        assert_eq!(record.tags.len(), 1);
    }

    #[test]
    fn update_note_replaces_body_and_preserves_title_when_omitted() {
        let repository = open_repo();
        let now = UnixTimestamp::new(1_700_000_000).unwrap_or_else(|error| panic!("{error}"));
        let id = NoteId::new("note-update-1").unwrap_or_else(|error| panic!("{error}"));
        let title = NoteTitle::new("Original").unwrap_or_else(|error| panic!("{error}"));
        let body = NoteBody::new("first").unwrap_or_else(|error| panic!("{error}"));
        repository
            .insert_note(NewNoteRecord {
                id: id.clone(),
                title,
                body,
                created_at: now,
                updated_at: now,
                tags: vec![],
                pinned_at: None,
            })
            .unwrap_or_else(|error| panic!("insert failed: {error}"));

        let service = NoteService::new(repository, SystemClock, UuidV7FileIdGenerator);
        let updated = service
            .update_note(
                &id,
                UpdateNoteRequest {
                    title: None,
                    body: "second".to_owned(),
                },
            )
            .unwrap_or_else(|error| panic!("update failed: {error}"));

        assert_eq!(updated.title.as_str(), "Original");
        assert_eq!(updated.body.as_str(), "second");
    }

    #[test]
    fn search_all_returns_files_and_notes() {
        let repository = open_repo();
        let now = UnixTimestamp::new(1_700_000_000).unwrap_or_else(|error| panic!("{error}"));
        let id = NoteId::new("note-search-1").unwrap_or_else(|error| panic!("{error}"));
        repository
            .insert_note(NewNoteRecord {
                id,
                title: NoteTitle::new("Rust notes").unwrap_or_else(|error| panic!("{error}")),
                body: NoteBody::new("ownership and borrowing")
                    .unwrap_or_else(|error| panic!("{error}")),
                created_at: now,
                updated_at: now,
                tags: vec![],
                pinned_at: None,
            })
            .unwrap_or_else(|error| panic!("insert failed: {error}"));

        let hits = repository
            .search_all("ownership")
            .unwrap_or_else(|error| panic!("search failed: {error}"));
        assert!(hits
            .iter()
            .any(|hit| matches!(hit, tssp_ports::SearchHit::Note(_))));
    }

    #[test]
    fn list_notes_respects_limit() {
        let repository = open_repo();
        let now = UnixTimestamp::new(1_700_000_000).unwrap_or_else(|error| panic!("{error}"));
        for index in 0..3 {
            let id =
                NoteId::new(format!("note-list-{index}")).unwrap_or_else(|error| panic!("{error}"));
            repository
                .insert_note(NewNoteRecord {
                    id,
                    title: NoteTitle::new(format!("Title {index}"))
                        .unwrap_or_else(|error| panic!("{error}")),
                    body: NoteBody::new(format!("Body {index}"))
                        .unwrap_or_else(|error| panic!("{error}")),
                    created_at: now,
                    updated_at: now,
                    tags: vec![],
                    pinned_at: None,
                })
                .unwrap_or_else(|error| panic!("insert failed: {error}"));
        }

        let page = repository
            .list_notes(&NoteListQuery {
                limit: 2,
                ..NoteListQuery::default()
            })
            .unwrap_or_else(|error| panic!("list failed: {error}"));
        assert_eq!(page.notes.len(), 2);
    }
}
