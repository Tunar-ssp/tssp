//! Pure domain model for TSSP.
//!
//! This crate contains validation rules and value objects only. It performs no
//! filesystem, network, database, clock, or process I/O so the application and
//! delivery layers can test core behavior without infrastructure.

mod error;
mod file;
mod hash;
mod note;
mod pagination;
mod session;
mod tag;
mod text;
mod time;

pub use error::DomainError;
pub use file::{FileId, FileName, FileRecord, FileSize, MimeType, StorageHandle};
pub use hash::ContentHash;
pub use note::{
    derive_note_title, NoteBody, NoteId, NoteRecord, NoteTitle, MAX_NOTE_BODY_BYTES,
};
pub use pagination::{Cursor, PageSize};
pub use session::{SessionKind, SessionToken, TransferSession};
pub use tag::{Tag, TagKey};
pub use time::UnixTimestamp;
