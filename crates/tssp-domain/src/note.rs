//! Markdown note value objects and aggregate records.

use std::fmt;

use crate::{text, DomainError, Tag, UnixTimestamp, UserId};

/// Maximum note body size (1 MiB), per specification.
pub const MAX_NOTE_BODY_BYTES: usize = 1_048_576;

const MAX_ID_LEN: usize = 128;
const MAX_TITLE_BYTES: usize = 512;

/// Opaque note identifier (`UUIDv7` on the wire).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NoteId(String);

impl NoteId {
    /// Creates a note id from a non-empty URL-safe token.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError`] when the id is empty, too long, or invalid.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DomainError> {
        let value = value.as_ref().trim();
        validate_token("note id", value, MAX_ID_LEN)?;
        Ok(Self(value.to_owned()))
    }

    /// Returns the client-visible identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NoteId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Note title shown in lists and search results.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NoteTitle(String);

impl NoteTitle {
    /// Creates a title from user input.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError`] when the title is empty or too long.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DomainError> {
        let value = text::trim_and_collapse_whitespace(value.as_ref());
        if value.is_empty() {
            return Err(DomainError::Empty {
                field: "note title",
            });
        }
        if value.len() > MAX_TITLE_BYTES {
            return Err(DomainError::TooLong {
                field: "note title",
                max: MAX_TITLE_BYTES,
                actual: value.len(),
            });
        }
        Ok(Self(value))
    }

    /// Returns the title text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NoteTitle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Markdown note body stored in the metadata index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteBody(String);

impl NoteBody {
    /// Creates a note body after trimming trailing whitespace on each line edge.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError`] when the body exceeds the size limit.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DomainError> {
        let value = value.as_ref().trim();
        if value.len() > MAX_NOTE_BODY_BYTES {
            return Err(DomainError::TooLong {
                field: "note body",
                max: MAX_NOTE_BODY_BYTES,
                actual: value.len(),
            });
        }
        Ok(Self(value.to_owned()))
    }

    /// Returns the note body (format determined by client).
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Committed note metadata and content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteRecord {
    /// Opaque id.
    pub id: NoteId,
    /// Display title.
    pub title: NoteTitle,
    /// Markdown body.
    pub body: NoteBody,
    /// Creation time (UTC).
    pub created_at: UnixTimestamp,
    /// Last update time (UTC).
    pub updated_at: UnixTimestamp,
    /// Tags attached to the note.
    pub tags: Vec<Tag>,
    /// Pin ordering position when pinned.
    pub pinned_at: Option<u32>,
    /// Virtual folder path for note organization.
    pub folder_path: String,
    /// Owning user id.
    pub owner_id: Option<UserId>,
    /// Parent note id for page nesting (Notion-style tree). `None` = top level.
    pub parent_id: Option<String>,
    /// Optional page icon (emoji or short token) shown in the tree and header.
    pub icon: Option<String>,
    /// Ordering position within the current level of the tree.
    pub sort_order: i64,
}

/// Derives a title from Markdown when the caller did not supply one.
///
/// Uses the first `#` heading, otherwise the first non-empty line (truncated).
#[must_use]
pub fn derive_note_title(body: &str) -> String {
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix('#') {
            let title = heading.trim_start_matches('#').trim();
            if !title.is_empty() {
                return text::truncate_to_bytes(title, MAX_TITLE_BYTES);
            }
        }
    }

    for line in body.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            return text::truncate_to_bytes(trimmed, MAX_TITLE_BYTES);
        }
    }

    "Untitled".to_owned()
}

fn validate_token(field: &'static str, value: &str, max_len: usize) -> Result<(), DomainError> {
    if value.is_empty() {
        return Err(DomainError::Empty { field });
    }
    if value.len() > max_len {
        return Err(DomainError::TooLong {
            field,
            max: max_len,
            actual: value.len(),
        });
    }
    if let Some(character) = value
        .chars()
        .find(|character| !is_token_character(*character))
    {
        return Err(DomainError::InvalidCharacter { field, character });
    }
    Ok(())
}

fn is_token_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '-' || character == '_'
}

#[cfg(test)]
mod tests {
    use super::{derive_note_title, NoteBody, NoteId, NoteTitle};
    use crate::UnixTimestamp;

    #[test]
    fn derive_title_prefers_markdown_heading() {
        let body = "intro\n\n# Project Plan\n\nDetails";
        assert_eq!(derive_note_title(body), "Project Plan");
    }

    #[test]
    fn derive_title_uses_first_line_when_no_heading() {
        assert_eq!(derive_note_title("Shopping list\n- milk"), "Shopping list");
    }

    #[test]
    fn note_body_allows_empty_content() {
        assert!(NoteBody::new("   \n").is_ok());
        assert!(NoteBody::new("").is_ok());
    }

    #[test]
    fn note_record_round_trip_fields() {
        let id = NoteId::new("note-abc").unwrap_or_else(|error| panic!("{error}"));
        let title = NoteTitle::new("Title").unwrap_or_else(|error| panic!("{error}"));
        let body = NoteBody::new("Body").unwrap_or_else(|error| panic!("{error}"));
        let created = UnixTimestamp::new(1_700_000_000).unwrap_or_else(|error| panic!("{error}"));
        let record = super::NoteRecord {
            id,
            title,
            body,
            created_at: created,
            updated_at: created,
            tags: vec![],
            pinned_at: None,
            folder_path: String::new(),
            owner_id: None,
            parent_id: None,
            icon: None,
            sort_order: 0,
        };
        assert_eq!(record.title.as_str(), "Title");
    }
}
