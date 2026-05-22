//! Cursor-based pagination value objects.

use crate::DomainError;

const DEFAULT_PAGE_SIZE: u16 = 50;
const MAX_PAGE_SIZE: u16 = 500;
const MAX_CURSOR_LEN: usize = 512;

/// Validated page size for list and search requests.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageSize(u16);

impl PageSize {
    /// Creates a page size in the accepted range `1..=500`.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::OutOfRange`] when the page size is zero or larger
    /// than the specification maximum.
    pub fn new(value: u16) -> Result<Self, DomainError> {
        if value == 0 || value > MAX_PAGE_SIZE {
            return Err(DomainError::OutOfRange {
                field: "page size",
                min: 1,
                max: u64::from(MAX_PAGE_SIZE),
                actual: u64::from(value),
            });
        }

        Ok(Self(value))
    }

    /// Returns the specification default page size.
    #[must_use]
    pub const fn default() -> Self {
        Self(DEFAULT_PAGE_SIZE)
    }

    /// Returns the numeric page size.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

/// Opaque pagination cursor.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cursor(String);

impl Cursor {
    /// Creates a cursor from an opaque non-empty URL-safe string.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError`] when the cursor is empty, too long, or contains
    /// characters outside the cursor token set.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DomainError> {
        let value = value.as_ref().trim();
        if value.is_empty() {
            return Err(DomainError::Empty { field: "cursor" });
        }
        if value.len() > MAX_CURSOR_LEN {
            return Err(DomainError::TooLong {
                field: "cursor",
                max: MAX_CURSOR_LEN,
                actual: value.len(),
            });
        }
        if let Some(character) = value
            .chars()
            .find(|character| !is_cursor_character(*character))
        {
            return Err(DomainError::InvalidCharacter {
                field: "cursor",
                character,
            });
        }

        Ok(Self(value.to_owned()))
    }

    /// Returns the cursor string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_cursor_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | '~')
}

#[cfg(test)]
mod tests {
    use super::{Cursor, PageSize};
    use crate::DomainError;

    #[test]
    fn default_page_size_matches_spec() {
        assert_eq!(PageSize::default().get(), 50);
    }

    #[test]
    fn page_size_rejects_zero() {
        assert_eq!(
            PageSize::new(0),
            Err(DomainError::OutOfRange {
                field: "page size",
                min: 1,
                max: 500,
                actual: 0
            })
        );
    }

    #[test]
    fn page_size_rejects_more_than_maximum() {
        assert_eq!(
            PageSize::new(501),
            Err(DomainError::OutOfRange {
                field: "page size",
                min: 1,
                max: 500,
                actual: 501
            })
        );
    }

    #[test]
    fn cursor_rejects_spaces() {
        assert_eq!(
            Cursor::new("bad cursor"),
            Err(DomainError::InvalidCharacter {
                field: "cursor",
                character: ' '
            })
        );
    }
}
