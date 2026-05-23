//! Note provider trait and adapters.

use tssp_app::{CreateNoteRequest, NoteService, UpdateNoteRequest};
use tssp_domain::{NoteId, NoteRecord};
use tssp_ports::{NoteListQuery, NoteRepository, PagedNotes};

use super::error::{map_note_error, HttpNoteError};

/// HTTP-facing note operations.
pub trait NoteProvider: Send + Sync {
    /// Creates a note.
    fn create_note(&self, request: CreateNoteRequest) -> Result<NoteRecord, HttpNoteError>;

    /// Lists notes.
    fn list_notes(&self, query: NoteListQuery) -> Result<PagedNotes, HttpNoteError>;

    /// Returns one note.
    fn get_note(&self, id: NoteId) -> Result<NoteRecord, HttpNoteError>;

    /// Replaces a note.
    fn update_note(
        &self,
        id: NoteId,
        request: UpdateNoteRequest,
    ) -> Result<NoteRecord, HttpNoteError>;

    /// Deletes a note.
    fn delete_note(&self, id: NoteId) -> Result<(), HttpNoteError>;

    /// Adds tags to a note.
    fn add_tags(&self, id: NoteId, tags: Vec<String>) -> Result<u64, HttpNoteError>;

    /// Removes a tag from a note.
    fn remove_tag(&self, id: NoteId, tag: String) -> Result<u64, HttpNoteError>;

    /// Pins a note.
    fn pin_note(&self, id: NoteId, position: Option<u32>) -> Result<(), HttpNoteError>;

    /// Unpins a note.
    fn unpin_note(&self, id: NoteId) -> Result<(), HttpNoteError>;
}

/// Placeholder provider used before wiring the application service.
#[derive(Debug)]
pub(crate) struct StaticNoteProvider;

impl NoteProvider for StaticNoteProvider {
    fn create_note(&self, _request: CreateNoteRequest) -> Result<NoteRecord, HttpNoteError> {
        Err(unavailable())
    }

    fn list_notes(&self, _query: NoteListQuery) -> Result<PagedNotes, HttpNoteError> {
        Err(unavailable())
    }

    fn get_note(&self, _id: NoteId) -> Result<NoteRecord, HttpNoteError> {
        Err(unavailable())
    }

    fn update_note(
        &self,
        _id: NoteId,
        _request: UpdateNoteRequest,
    ) -> Result<NoteRecord, HttpNoteError> {
        Err(unavailable())
    }

    fn delete_note(&self, _id: NoteId) -> Result<(), HttpNoteError> {
        Err(unavailable())
    }

    fn add_tags(&self, _id: NoteId, _tags: Vec<String>) -> Result<u64, HttpNoteError> {
        Err(unavailable())
    }

    fn remove_tag(&self, _id: NoteId, _tag: String) -> Result<u64, HttpNoteError> {
        Err(unavailable())
    }

    fn pin_note(&self, _id: NoteId, _position: Option<u32>) -> Result<(), HttpNoteError> {
        Err(unavailable())
    }

    fn unpin_note(&self, _id: NoteId) -> Result<(), HttpNoteError> {
        Err(unavailable())
    }
}

fn unavailable() -> HttpNoteError {
    HttpNoteError::Unavailable {
        message: "note service is not configured".to_owned(),
    }
}

/// Application-backed note provider.
pub struct ApplicationNoteProvider<R, C, G> {
    service: NoteService<R, C, G>,
}

impl<R, C, G> ApplicationNoteProvider<R, C, G> {
    /// Creates a provider from a note service.
    #[must_use]
    pub const fn new(service: NoteService<R, C, G>) -> Self {
        Self { service }
    }
}

impl<R, C, G> NoteProvider for ApplicationNoteProvider<R, C, G>
where
    R: NoteRepository + Send + Sync,
    C: tssp_ports::Clock + Send + Sync,
    G: tssp_ports::IdGenerator + Send + Sync,
{
    fn create_note(&self, request: CreateNoteRequest) -> Result<NoteRecord, HttpNoteError> {
        self.service.create_note(request).map_err(map_note_error)
    }

    fn list_notes(&self, query: NoteListQuery) -> Result<PagedNotes, HttpNoteError> {
        self.service.list_notes(&query).map_err(map_note_error)
    }

    fn get_note(&self, id: NoteId) -> Result<NoteRecord, HttpNoteError> {
        self.service
            .get_note(&id)
            .map_err(map_note_error)?
            .ok_or(HttpNoteError::NotFound {
                message: format!("note {} was not found", id.as_str()),
            })
    }

    fn update_note(
        &self,
        id: NoteId,
        request: UpdateNoteRequest,
    ) -> Result<NoteRecord, HttpNoteError> {
        self.service
            .update_note(&id, request)
            .map_err(map_note_error)
    }

    fn delete_note(&self, id: NoteId) -> Result<(), HttpNoteError> {
        self.service
            .delete_note(&id)
            .map_err(map_note_error)
            .map(|_| ())
    }

    fn add_tags(&self, id: NoteId, tags: Vec<String>) -> Result<u64, HttpNoteError> {
        let tag_refs = tags.iter().map(String::as_str).collect::<Vec<_>>();
        self.service
            .add_tags(&id, &tag_refs)
            .map(|outcome| outcome.changed_count)
            .map_err(map_note_error)
    }

    fn remove_tag(&self, id: NoteId, tag: String) -> Result<u64, HttpNoteError> {
        self.service
            .remove_tag(&id, &tag)
            .map(|outcome| outcome.changed_count)
            .map_err(map_note_error)
    }

    fn pin_note(&self, id: NoteId, position: Option<u32>) -> Result<(), HttpNoteError> {
        self.service
            .pin_note(&id, position)
            .map_err(map_note_error)
            .map(|_| ())
    }

    fn unpin_note(&self, id: NoteId) -> Result<(), HttpNoteError> {
        self.service
            .unpin_note(&id)
            .map_err(map_note_error)
            .map(|_| ())
    }
}
