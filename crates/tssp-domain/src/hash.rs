//! Content hash value object.

use crate::DomainError;
use std::fmt;

const BLAKE3_HEX_LEN: usize = 64;

/// Lowercase BLAKE3 content hash encoded as 64 hexadecimal characters.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContentHash(String);

impl ContentHash {
    /// Creates a content hash from a hexadecimal BLAKE3 digest.
    ///
    /// The value is normalized to lowercase after validation. Non-hex input and
    /// values with the wrong length are rejected.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidFormat`] when the digest length is not 64
    /// characters, or [`DomainError::InvalidCharacter`] when non-hex text is
    /// supplied.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DomainError> {
        let value = value.as_ref().trim();
        if value.len() != BLAKE3_HEX_LEN {
            return Err(DomainError::InvalidFormat {
                field: "content hash",
            });
        }

        if let Some(character) = value
            .chars()
            .find(|character| !character.is_ascii_hexdigit())
        {
            return Err(DomainError::InvalidCharacter {
                field: "content hash",
                character,
            });
        }

        Ok(Self(value.to_ascii_lowercase()))
    }

    /// Returns the lowercase hexadecimal representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::ContentHash;
    use crate::DomainError;

    #[test]
    fn valid_hash_is_lowercase() {
        let hash =
            ContentHash::new("ABCDEFabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123");

        assert!(matches!(hash, Ok(value) if value.as_str().starts_with("abcdef")));
    }

    #[test]
    fn wrong_length_is_rejected() {
        let hash = ContentHash::new("abc");

        assert_eq!(
            hash,
            Err(DomainError::InvalidFormat {
                field: "content hash"
            })
        );
    }

    #[test]
    fn non_hex_character_is_rejected() {
        let hash =
            ContentHash::new("zbcdefabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123");

        assert_eq!(
            hash,
            Err(DomainError::InvalidCharacter {
                field: "content hash",
                character: 'z'
            })
        );
    }
}
