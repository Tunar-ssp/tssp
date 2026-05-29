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
    add_note_tags, create_note, delete_note, duplicate_note, export_notes, get_note,
    get_note_backlinks, list_notes, move_note, pin_note, remove_note_tag, replace_note_tags,
    set_note_icon, unpin_note, update_note,
};
