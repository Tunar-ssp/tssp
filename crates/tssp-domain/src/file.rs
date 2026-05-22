//! File metadata value objects and aggregate records.

use std::fmt;

use crate::{text, ContentHash, DomainError, Tag, UnixTimestamp};

const MAX_ID_LEN: usize = 128;
const MAX_FILENAME_BYTES: usize = 4096;
const MAX_STORAGE_COMPONENT_BYTES: usize = 120;
const MAX_MIME_LEN: usize = 127;

/// Opaque file identifier exposed to clients.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileId(String);

impl FileId {
    /// Creates a file id from a non-empty URL-safe token.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError`] when the id is empty, too long, or contains a
    /// character outside the opaque URL-safe token set.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DomainError> {
        let value = value.as_ref().trim();
        validate_token("file id", value, MAX_ID_LEN)?;
        Ok(Self(value.to_owned()))
    }

    /// Returns the client-visible identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FileId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Original filename preserved for display and metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileName {
    original: String,
    storage_component: String,
}

impl FileName {
    /// Creates a filename and derives a safe storage component from it.
    ///
    /// The original name is NFC-normalized and preserved. The storage component
    /// is intentionally lossy and must never be shown as the user-facing name.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Empty`] when the filename is empty, or
    /// [`DomainError::TooLong`] when it exceeds the metadata limit.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DomainError> {
        let original = text::normalize_unicode(value.as_ref());
        if original.is_empty() {
            return Err(DomainError::Empty { field: "filename" });
        }
        if original.len() > MAX_FILENAME_BYTES {
            return Err(DomainError::TooLong {
                field: "filename",
                max: MAX_FILENAME_BYTES,
                actual: original.len(),
            });
        }

        let storage_component = sanitize_storage_component(&original);
        Ok(Self {
            original,
            storage_component,
        })
    }

    /// Returns the user-facing filename.
    #[must_use]
    pub fn original(&self) -> &str {
        &self.original
    }

    /// Returns a filesystem-safe component that contains no user-controlled path separators.
    #[must_use]
    pub fn storage_component(&self) -> &str {
        &self.storage_component
    }
}

impl fmt::Display for FileName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.original())
    }
}

/// Byte length of a stored file.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileSize(u64);

impl FileSize {
    /// Creates a file size. Empty files are valid and represented by zero.
    #[must_use]
    pub const fn new(bytes: u64) -> Self {
        Self(bytes)
    }

    /// Returns the size in bytes.
    #[must_use]
    pub const fn bytes(self) -> u64 {
        self.0
    }
}

/// MIME type recorded for a file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MimeType(String);

impl MimeType {
    /// Creates a MIME type after basic RFC-style shape validation.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError`] when the value is empty, too long, lacks a `/`,
    /// or contains unsupported MIME token characters.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DomainError> {
        let value = value.as_ref().trim().to_ascii_lowercase();
        if value.is_empty() {
            return Err(DomainError::Empty { field: "mime type" });
        }
        if value.len() > MAX_MIME_LEN {
            return Err(DomainError::TooLong {
                field: "mime type",
                max: MAX_MIME_LEN,
                actual: value.len(),
            });
        }
        validate_mime_characters(&value)?;
        Ok(Self(value))
    }

    /// Returns `application/octet-stream`.
    #[must_use]
    pub fn octet_stream() -> Self {
        Self("application/octet-stream".to_owned())
    }

    /// Returns the normalized MIME type.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MimeType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Opaque handle used by storage adapters to find blob bytes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StorageHandle(String);

impl StorageHandle {
    /// Creates a storage handle from an opaque non-empty token.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError`] when the handle is empty, too long, or contains
    /// a character outside the storage-handle token set.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DomainError> {
        let value = value.as_ref().trim();
        validate_token("storage handle", value, 512)?;
        Ok(Self(value.to_owned()))
    }

    /// Returns the opaque handle string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StorageHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Complete metadata record for one logical file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRecord {
    /// Client-visible id.
    pub id: FileId,
    /// Original filename.
    pub name: FileName,
    /// Stored byte count.
    pub size: FileSize,
    /// Content-addressing hash.
    pub content_hash: ContentHash,
    /// MIME type for downloads and previews.
    pub mime_type: MimeType,
    /// Opaque storage location.
    pub storage_handle: StorageHandle,
    /// Upload time in UTC seconds.
    pub uploaded_at: UnixTimestamp,
    /// Tags attached to the file.
    pub tags: Vec<Tag>,
    /// Pin order when the file is pinned.
    pub pinned_at: Option<u32>,
}

impl FileRecord {
    /// Returns true when the file is currently pinned.
    #[must_use]
    pub const fn is_pinned(&self) -> bool {
        self.pinned_at.is_some()
    }
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
    character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | ':' | '/')
}

fn sanitize_storage_component(original: &str) -> String {
    let mut sanitized = String::new();
    let mut last_was_separator = false;

    for character in original.chars() {
        if is_storage_safe_character(character) {
            sanitized.push(character);
            last_was_separator = false;
        } else if !last_was_separator {
            sanitized.push('_');
            last_was_separator = true;
        }
    }

    while sanitized.contains("_.") {
        sanitized = sanitized.replace("_.", ".");
    }

    let trimmed = sanitized.trim_matches(['.', '_', ' ']).to_owned();
    let safe_name = if trimmed.is_empty() || trimmed == "." || trimmed == ".." {
        "file".to_owned()
    } else {
        trimmed
    };

    text::truncate_to_bytes(&safe_name, MAX_STORAGE_COMPONENT_BYTES)
}

fn is_storage_safe_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_')
}

fn validate_mime_characters(value: &str) -> Result<(), DomainError> {
    if !value.contains('/') {
        return Err(DomainError::InvalidFormat { field: "mime type" });
    }

    if let Some(character) = value.chars().find(|character| {
        !(character.is_ascii_alphanumeric() || matches!(character, '/' | '+' | '-' | '.'))
    }) {
        return Err(DomainError::InvalidCharacter {
            field: "mime type",
            character,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{FileId, FileName, FileSize, MimeType, StorageHandle};
    use crate::DomainError;

    #[test]
    fn file_id_accepts_url_safe_tokens() {
        assert!(FileId::new("018f-abc_DEF.1").is_ok());
    }

    #[test]
    fn file_id_rejects_spaces() {
        assert_eq!(
            FileId::new("bad id"),
            Err(DomainError::InvalidCharacter {
                field: "file id",
                character: ' '
            })
        );
    }

    #[test]
    fn filename_preserves_original_and_sanitizes_storage_component() {
        let name = FileName::new("report / Q2\u{0}.pdf");

        assert!(matches!(
            name,
            Ok(value) if value.original() == "report / Q2\u{0}.pdf"
                && value.storage_component() == "report_Q2.pdf"
        ));
    }

    #[test]
    fn filename_rejects_empty_names() {
        assert_eq!(
            FileName::new(""),
            Err(DomainError::Empty { field: "filename" })
        );
    }

    #[test]
    fn filename_sanitizer_never_returns_dot_components() {
        assert!(matches!(FileName::new("../"), Ok(value) if value.storage_component() == "file"));
    }

    #[test]
    fn long_filename_is_rejected_before_storage_sanitization() {
        let filename = "a".repeat(4097);

        assert_eq!(
            FileName::new(filename),
            Err(DomainError::TooLong {
                field: "filename",
                max: 4096,
                actual: 4097
            })
        );
    }

    #[test]
    fn zero_byte_file_size_is_valid() {
        assert_eq!(FileSize::new(0).bytes(), 0);
    }

    #[test]
    fn mime_type_is_lowercase() {
        assert!(matches!(
            MimeType::new("IMAGE/PNG"),
            Ok(value) if value.as_str() == "image/png"
        ));
    }

    #[test]
    fn mime_type_requires_slash() {
        assert_eq!(
            MimeType::new("image"),
            Err(DomainError::InvalidFormat { field: "mime type" })
        );
    }

    #[test]
    fn storage_handle_rejects_empty_values() {
        assert_eq!(
            StorageHandle::new(" "),
            Err(DomainError::Empty {
                field: "storage handle"
            })
        );
    }
}
