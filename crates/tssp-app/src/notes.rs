//! Note lifecycle use cases.

use thiserror::Error;
use tssp_domain::{
    derive_note_title, DomainError, NoteBody, NoteId, NoteRecord, NoteTitle, Tag, UserId,
};
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
    /// Note body (format determined by client: HTML, Markdown, etc).
    pub body: String,
    /// Initial tags.
    pub tags: Vec<String>,
    /// Pin immediately when `Some(position)` or at end when `Some` with default.
    pub pin: bool,
    /// Owning user id.
    pub owner_id: Option<UserId>,
    /// Optional parent note id for page nesting.
    pub parent_id: Option<String>,
    /// Optional page icon.
    pub icon: Option<String>,
    /// Optional explicit sort order.
    pub sort_order: Option<i64>,
}

/// Input for replacing a note.
#[derive(Debug, Clone)]
pub struct UpdateNoteRequest {
    /// Optional new title.
    pub title: Option<String>,
    /// Replacement note body (format determined by client: HTML, Markdown, etc).
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
        let parent_id = request.parent_id.filter(|value| !value.trim().is_empty());
        let icon = request.icon.filter(|value| !value.trim().is_empty());
        let sort_order = request.sort_order.unwrap_or(0);

        let record = self
            .repository
            .insert_note(NewNoteRecord {
                id: id.clone(),
                title,
                body: body.clone(),
                created_at: now,
                updated_at: now,
                tags,
                pinned_at,
                folder_path: String::new(),
                owner_id: request.owner_id,
                parent_id,
                icon,
                sort_order,
            })
            .map_err(NoteError::Repository)?;

        // Best-effort: populate note_links from [[Title]] references.
        let linked = self.resolve_wiki_links(body.as_str());
        if !linked.is_empty() {
            let _ = self.repository.update_note_links(&id, &linked);
        }

        Ok(record)
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

        let record = self
            .repository
            .update_note(id, &title, &body, updated_at)
            .map_err(NoteError::Repository)?;

        // Best-effort: refresh note_links from [[Title]] references.
        let linked = self.resolve_wiki_links(body.as_str());
        let _ = self.repository.update_note_links(id, &linked);

        Ok(record)
    }

    /// Moves a note under a new parent (`None` = top level), guarding against
    /// cycles (a note cannot become its own ancestor).
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] when the note is missing or the move is invalid.
    pub fn move_note(&self, id: &NoteId, parent_id: Option<&str>) -> Result<NoteRecord, NoteError> {
        let parent_id = parent_id.map(str::trim).filter(|value| !value.is_empty());
        if let Some(parent) = parent_id {
            if parent == id.as_str() {
                return Err(NoteError::InvalidMove);
            }
            // Walk the prospective parent's ancestry; reject if `id` appears.
            let mut cursor = Some(parent.to_owned());
            while let Some(current) = cursor {
                if current == id.as_str() {
                    return Err(NoteError::InvalidMove);
                }
                let ancestor = NoteId::new(&current).ok();
                cursor = match ancestor {
                    Some(ancestor_id) => self
                        .repository
                        .find_note(&ancestor_id)
                        .map_err(NoteError::Repository)?
                        .and_then(|note| note.parent_id),
                    None => None,
                };
            }
        }
        let updated_at = self.clock.now();
        self.repository
            .set_note_parent(id, parent_id, updated_at)
            .map_err(NoteError::Repository)
    }

    /// Sets a note's icon (`None` clears it).
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] when the note is missing or persistence fails.
    pub fn set_note_icon(&self, id: &NoteId, icon: Option<&str>) -> Result<NoteRecord, NoteError> {
        let icon = icon.map(str::trim).filter(|value| !value.is_empty());
        let updated_at = self.clock.now();
        self.repository
            .set_note_icon(id, icon, updated_at)
            .map_err(NoteError::Repository)
    }

    /// Deletes a note and cleans up its link graph entries.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] when deletion fails.
    pub fn delete_note(&self, id: &NoteId) -> Result<bool, NoteError> {
        let deleted = self
            .repository
            .delete_note(id)
            .map_err(NoteError::Repository)?;
        if deleted {
            // Best-effort: remove outgoing links; ignore errors.
            let _ = self.repository.update_note_links(id, &[]);
        }
        Ok(deleted)
    }

    /// Resolves `[[Title]]` links in `body` to `NoteId`s via exact title match.
    fn resolve_wiki_links(&self, body: &str) -> Vec<NoteId> {
        let titles = extract_wiki_links(body);
        let mut ids = Vec::new();
        for title in titles {
            let query = NoteListQuery {
                limit: 10,
                title_substring: Some(title.clone()),
                ..NoteListQuery::default()
            };
            if let Ok(page) = self.repository.list_notes(&query) {
                for note in page.notes {
                    if note.title.as_str().eq_ignore_ascii_case(&title) {
                        ids.push(note.id);
                        break;
                    }
                }
            }
        }
        ids
    }

    /// Returns all notes that contain a `[[link]]` pointing to `target_id`.
    ///
    /// # Errors
    ///
    /// Returns [`NoteError`] when the lookup fails.
    pub fn get_backlinks(&self, target_id: &NoteId) -> Result<Vec<NoteId>, NoteError> {
        self.repository
            .get_note_backlinks(target_id)
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

    /// Attempted to move a note under itself or one of its descendants.
    #[error("invalid note move: would create a cycle")]
    InvalidMove,

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

/// Extracts all `[[Title]]` wiki-link targets from a note body.
/// Returns raw title strings; callers must resolve them to `NoteId`s.
fn extract_wiki_links(body: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut search = body;
    while let Some(start) = search.find("[[") {
        let after_open = &search[start + 2..];
        if let Some(end) = after_open.find("]]") {
            let title = after_open[..end].trim().to_owned();
            if !title.is_empty() {
                links.push(title);
            }
            search = &after_open[end + 2..];
        } else {
            break;
        }
    }
    links
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
                owner_id: None,
                parent_id: None,
                icon: None,
                sort_order: None,
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
                folder_path: String::new(),
                owner_id: None,
                parent_id: None,
                icon: None,
                sort_order: 0,
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
                folder_path: String::new(),
                owner_id: None,
                parent_id: None,
                icon: None,
                sort_order: 0,
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
                    folder_path: String::new(),
                    owner_id: None,
                    parent_id: None,
                    icon: None,
                    sort_order: 0,
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
