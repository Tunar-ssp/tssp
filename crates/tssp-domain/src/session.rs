//! QR transfer session domain rules.

use std::fmt;

use crate::{DomainError, FileName, UserId};
use crate::{FileId, UnixTimestamp};

const TOKEN_BITS: usize = 128;
const TOKEN_BASE64URL_LEN: usize = 22;

/// Type of one-time transfer session.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SessionKind {
    /// Existing server file exposed for one anonymous download.
    Send,
    /// Anonymous upload placeholder waiting for a phone/browser upload.
    Receive,
}

/// Unguessable URL-safe session token.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionToken(String);

impl SessionToken {
    /// Creates a token from unpadded base64url text carrying 128 bits of entropy.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError`] when the token length or alphabet does not match
    /// the 128-bit unpadded base64url representation.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DomainError> {
        let value = value.as_ref().trim();
        if value.len() != TOKEN_BASE64URL_LEN {
            return Err(DomainError::InvalidFormat {
                field: "session token",
            });
        }

        if let Some(character) = value
            .chars()
            .find(|character| !is_base64url_character(*character))
        {
            return Err(DomainError::InvalidCharacter {
                field: "session token",
                character,
            });
        }

        Ok(Self(value.to_owned()))
    }

    /// Returns the token text. Tokens must be redacted from logs.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the required entropy in bits.
    #[must_use]
    pub const fn entropy_bits() -> usize {
        TOKEN_BITS
    }
}

impl fmt::Display for SessionToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// State for a single-use QR transfer session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferSession {
    /// Unguessable session token.
    pub token: SessionToken,
    /// Session type.
    pub kind: SessionKind,
    /// User who created the session.
    pub creator_id: Option<UserId>,
    /// UTC creation time.
    pub created_at: UnixTimestamp,
    /// UTC expiration time.
    pub expires_at: UnixTimestamp,
    /// Source file for send sessions.
    pub source_file: Option<FileId>,
    /// Completed uploaded file for receive sessions.
    pub received_file: Option<FileId>,
    /// Original expected upload name, when supplied by a receive session.
    pub expected_name: Option<FileName>,
    used_at: Option<UnixTimestamp>,
}

impl TransferSession {
    /// Creates a new unused transfer session.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError`] when expiration is not after creation or a send
    /// session lacks its required source file.
    pub fn new(
        token: SessionToken,
        kind: SessionKind,
        created_at: UnixTimestamp,
        expires_at: UnixTimestamp,
        source_file: Option<FileId>,
    ) -> Result<Self, DomainError> {
        if expires_at <= created_at {
            return Err(DomainError::OutOfRange {
                field: "session expiration",
                min: created_at.seconds_u64().saturating_add(1),
                max: UnixTimestamp::MAX_SECONDS,
                actual: expires_at.seconds_u64(),
            });
        }

        if kind == SessionKind::Send && source_file.is_none() {
            return Err(DomainError::InvalidFormat {
                field: "send session source file",
            });
        }

        Ok(Self {
            token,
            kind,
            creator_id: None,
            created_at,
            expires_at,
            source_file,
            received_file: None,
            expected_name: None,
            used_at: None,
        })
    }

    /// Sets the session creator.
    pub fn with_creator(mut self, creator_id: UserId) -> Self {
        self.creator_id = Some(creator_id);
        self
    }

    /// Returns true when the session is already consumed.
    #[must_use]
    pub const fn is_used(&self) -> bool {
        self.used_at.is_some()
    }

    /// Returns true when the session is expired at `now`.
    #[must_use]
    pub const fn is_expired_at(&self, now: UnixTimestamp) -> bool {
        now.seconds() >= self.expires_at.seconds()
    }

    /// Marks the session as used exactly once.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError`] when the session was already used or the supplied
    /// use time is outside the valid session window.
    pub fn mark_used(&mut self, used_at: UnixTimestamp) -> Result<(), DomainError> {
        if self.used_at.is_some() {
            return Err(DomainError::InvalidFormat {
                field: "session state",
            });
        }
        if self.is_expired_at(used_at) {
            return Err(DomainError::OutOfRange {
                field: "session use time",
                min: self.created_at.seconds_u64(),
                max: self.expires_at.seconds_u64().saturating_sub(1),
                actual: used_at.seconds_u64(),
            });
        }

        self.used_at = Some(used_at);
        Ok(())
    }
}

fn is_base64url_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
}

#[cfg(test)]
mod tests {
    use super::{SessionKind, SessionToken, TransferSession};
    use crate::{DomainError, FileId, UnixTimestamp};

    fn timestamp(seconds: i64) -> UnixTimestamp {
        match UnixTimestamp::new(seconds) {
            Ok(value) => value,
            Err(error) => panic!("invalid test timestamp: {error}"),
        }
    }

    fn token() -> SessionToken {
        match SessionToken::new("abcdefghijklmnopqrstu1") {
            Ok(value) => value,
            Err(error) => panic!("invalid test token: {error}"),
        }
    }

    fn file_id() -> FileId {
        match FileId::new("018f-test") {
            Ok(value) => value,
            Err(error) => panic!("invalid test file id: {error}"),
        }
    }

    #[test]
    fn token_requires_base64url_shape_for_128_bits() {
        assert_eq!(SessionToken::entropy_bits(), 128);
        assert!(SessionToken::new("abcdefghijklmnopqrstu1").is_ok());
        assert_eq!(
            SessionToken::new("short"),
            Err(DomainError::InvalidFormat {
                field: "session token"
            })
        );
    }

    #[test]
    fn token_trims_input_and_displays_value() {
        let token = SessionToken::new(" abcdefghijklmnopqrstu1 ")
            .unwrap_or_else(|error| panic!("token failed: {error}"));

        assert_eq!(token.as_str(), "abcdefghijklmnopqrstu1");
        assert_eq!(token.to_string(), "abcdefghijklmnopqrstu1");
    }

    #[test]
    fn token_rejects_padding() {
        assert_eq!(
            SessionToken::new("abcdefghijklmnopqrstu="),
            Err(DomainError::InvalidCharacter {
                field: "session token",
                character: '='
            })
        );
    }

    #[test]
    fn send_session_requires_source_file() {
        assert_eq!(
            TransferSession::new(
                token(),
                SessionKind::Send,
                timestamp(10),
                timestamp(20),
                None
            ),
            Err(DomainError::InvalidFormat {
                field: "send session source file"
            })
        );
    }

    #[test]
    fn expiration_must_be_after_creation() {
        assert_eq!(
            TransferSession::new(
                token(),
                SessionKind::Receive,
                timestamp(20),
                timestamp(20),
                None
            ),
            Err(DomainError::OutOfRange {
                field: "session expiration",
                min: 21,
                max: UnixTimestamp::MAX_SECONDS,
                actual: 20
            })
        );
    }

    #[test]
    fn receive_session_starts_unused_and_can_expire() {
        let session = TransferSession::new(
            token(),
            SessionKind::Receive,
            timestamp(10),
            timestamp(20),
            None,
        )
        .unwrap_or_else(|error| panic!("session failed: {error}"));

        assert!(!session.is_used());
        assert!(!session.is_expired_at(timestamp(19)));
        assert!(session.is_expired_at(timestamp(20)));
        assert_eq!(session.source_file, None);
        assert_eq!(session.received_file, None);
        assert_eq!(session.expected_name, None);
    }

    #[test]
    fn session_can_be_used_once_before_expiration() {
        let mut session = match TransferSession::new(
            token(),
            SessionKind::Send,
            timestamp(10),
            timestamp(20),
            Some(file_id()),
        ) {
            Ok(value) => value,
            Err(error) => panic!("invalid test session: {error}"),
        };

        assert!(session.mark_used(timestamp(19)).is_ok());
        assert!(session.is_used());
        assert_eq!(
            session.mark_used(timestamp(19)),
            Err(DomainError::InvalidFormat {
                field: "session state"
            })
        );
    }

    #[test]
    fn session_cannot_be_used_at_expiration_boundary() {
        let mut session = match TransferSession::new(
            token(),
            SessionKind::Receive,
            timestamp(10),
            timestamp(20),
            None,
        ) {
            Ok(value) => value,
            Err(error) => panic!("invalid test session: {error}"),
        };

        assert_eq!(
            session.mark_used(timestamp(20)),
            Err(DomainError::OutOfRange {
                field: "session use time",
                min: 10,
                max: 19,
                actual: 20
            })
        );
    }
}
