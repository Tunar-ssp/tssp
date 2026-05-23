//! Tag value objects and case-insensitive lookup keys.

use std::fmt;

use crate::{text, DomainError};

const MAX_TAG_CHARS: usize = 64;

/// Normalized tag name attached to a file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tag {
    display: String,
    key: TagKey,
}

impl Tag {
    /// Creates a normalized tag.
    ///
    /// Leading and trailing whitespace is removed, internal whitespace is
    /// collapsed, and Unicode is normalized to NFC. Lookup is case-insensitive
    /// through [`TagKey`].
    ///
    /// # Errors
    ///
    /// Returns [`DomainError`] when the normalized tag is empty, too long, or
    /// contains unsupported control or separator characters.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DomainError> {
        let display = text::trim_and_collapse_whitespace(value.as_ref());
        if display.is_empty() {
            return Err(DomainError::Empty { field: "tag" });
        }

        let character_count = display.chars().count();
        if character_count > MAX_TAG_CHARS {
            return Err(DomainError::TooLong {
                field: "tag",
                max: MAX_TAG_CHARS,
                actual: character_count,
            });
        }

        if let Some(character) = display
            .chars()
            .find(|character| !is_allowed_tag_character(*character))
        {
            return Err(DomainError::InvalidCharacter {
                field: "tag",
                character,
            });
        }

        let key = TagKey(display.to_lowercase());
        Ok(Self { display, key })
    }

    /// Returns the normalized display form.
    #[must_use]
    pub fn display(&self) -> &str {
        &self.display
    }

    /// Returns the case-insensitive lookup key.
    #[must_use]
    pub const fn key(&self) -> &TagKey {
        &self.key
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.display())
    }
}

/// Case-insensitive tag lookup key.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TagKey(String);

impl TagKey {
    /// Creates a lookup key using the same normalization rules as [`Tag`].
    ///
    /// # Errors
    ///
    /// Returns [`DomainError`] when the supplied text is not a valid tag.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DomainError> {
        Tag::new(value).map(|tag| tag.key)
    }

    /// Returns the normalized key.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TagKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

fn is_allowed_tag_character(character: char) -> bool {
    !character.is_control() && !matches!(character, '/' | '\\' | ',' | '\u{7f}')
}

#[cfg(test)]
mod tests {
    use super::{Tag, TagKey};
    use crate::DomainError;

    #[test]
    fn tag_normalizes_whitespace_and_lookup_case() {
        let tag = Tag::new("  Family\t Photos  ");

        assert!(matches!(
            tag,
            Ok(value) if value.display() == "Family Photos" && value.key().as_str() == "family photos"
        ));
    }

    #[test]
    fn tag_display_uses_normalized_text() {
        let tag =
            Tag::new("  Family\t Photos  ").unwrap_or_else(|error| panic!("tag failed: {error}"));

        assert_eq!(tag.to_string(), "Family Photos");
    }

    #[test]
    fn tag_lookup_key_uses_same_rules() {
        let key = TagKey::new("FAMILY PHOTOS");

        assert!(matches!(key, Ok(value) if value.as_str() == "family photos"));
    }

    #[test]
    fn tag_lookup_key_displays_normalized_key() {
        let key =
            TagKey::new("FAMILY PHOTOS").unwrap_or_else(|error| panic!("key failed: {error}"));

        assert_eq!(key.to_string(), "family photos");
    }

    #[test]
    fn empty_tag_is_rejected() {
        assert_eq!(Tag::new(" \n "), Err(DomainError::Empty { field: "tag" }));
    }

    #[test]
    fn control_character_is_rejected() {
        assert_eq!(
            Tag::new("bad\u{0}tag"),
            Err(DomainError::InvalidCharacter {
                field: "tag",
                character: '\u{0}'
            })
        );
    }

    #[test]
    fn separators_are_rejected() {
        assert_eq!(
            Tag::new("bad/tag"),
            Err(DomainError::InvalidCharacter {
                field: "tag",
                character: '/'
            })
        );
        assert_eq!(
            Tag::new("bad\\tag"),
            Err(DomainError::InvalidCharacter {
                field: "tag",
                character: '\\'
            })
        );
        assert_eq!(
            Tag::new("bad,tag"),
            Err(DomainError::InvalidCharacter {
                field: "tag",
                character: ','
            })
        );
    }

    #[test]
    fn tag_has_length_limit() {
        let tag = "a".repeat(65);

        assert_eq!(
            Tag::new(tag),
            Err(DomainError::TooLong {
                field: "tag",
                max: 64,
                actual: 65
            })
        );
    }
}
