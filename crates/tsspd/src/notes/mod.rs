//! Notes HTTP delivery (CRUD, tags, pins).
//!
//! Split into small modules so auth, pagination, and audit hooks can land
//! without touching every handler.

mod error;
mod handlers;
mod provider;
mod query;
mod response;
mod validate;

#[cfg(test)]
mod tests;

pub use provider::{ApplicationNoteProvider, NoteProvider};

pub(crate) use provider::StaticNoteProvider;
pub use response::NoteRecordResponse;

pub(crate) use handlers::{
    add_note_tags, create_note, delete_note, get_note, list_notes, pin_note, remove_note_tag,
    unpin_note, update_note,
};
