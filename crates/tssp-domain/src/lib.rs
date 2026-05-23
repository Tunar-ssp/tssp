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
mod search_query;
mod session;
mod tag;
mod text;
mod time;
mod user;

pub use error::DomainError;
pub use file::{FileId, FileName, FileRecord, FileSize, MimeType, StorageHandle};
pub use hash::ContentHash;
pub use note::{derive_note_title, NoteBody, NoteId, NoteRecord, NoteTitle, MAX_NOTE_BODY_BYTES};
pub use pagination::{Cursor, PageSize};
pub use search_query::build_fts_query;
pub use session::{SessionKind, SessionToken, TransferSession};
pub use tag::{Tag, TagKey};
pub use time::UnixTimestamp;
pub use user::{UserId, UserName, UserRole, Visibility};
