//! System adapters for clocks, file identifiers, and session tokens.

/// Git adapter.
pub mod git;
/// LSP adapter.
pub mod lsp;
/// Terminal adapter.
pub mod terminal;
#[cfg(test)]
mod terminal_security_tests;

use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use tssp_domain::{FileId, SessionToken, UnixTimestamp};
use tssp_ports::{Clock, IdGenerationError, IdGenerator, SessionTokenGenerator};
use uuid::Uuid;

/// Clock backed by the host operating system.
#[derive(Debug, Copy, Clone, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> UnixTimestamp {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs());
        let seconds = i64::try_from(seconds).unwrap_or(i64::MAX);
        match UnixTimestamp::new(seconds) {
            Ok(timestamp) => timestamp,
            Err(_error) => UnixTimestamp::max(),
        }
    }
}

/// File id generator using `UUIDv7` for sortable opaque ids.
#[derive(Debug, Copy, Clone, Default)]
pub struct UuidV7FileIdGenerator;

impl IdGenerator for UuidV7FileIdGenerator {
    fn new_file_id(&self) -> Result<FileId, IdGenerationError> {
        FileId::new(Uuid::now_v7().to_string()).map_err(|error| IdGenerationError {
            message: error.to_string(),
        })
    }
}

/// Session token generator using OS randomness and URL-safe base64.
#[derive(Debug, Copy, Clone, Default)]
pub struct RandomSessionTokenGenerator;

impl SessionTokenGenerator for RandomSessionTokenGenerator {
    fn new_session_token(&self) -> Result<SessionToken, IdGenerationError> {
        let mut bytes = [0_u8; 16];
        getrandom::getrandom(&mut bytes).map_err(|error| IdGenerationError {
            message: format!("secure random token generation failed: {error}"),
        })?;
        let token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
        SessionToken::new(token).map_err(|error| IdGenerationError {
            message: error.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use tssp_ports::{Clock, IdGenerator, SessionTokenGenerator};

    use super::{RandomSessionTokenGenerator, SystemClock, UuidV7FileIdGenerator};

    #[test]
    fn system_clock_returns_supported_timestamp() {
        let timestamp = SystemClock.now();

        assert!(timestamp.seconds() > 0);
        assert!(timestamp.seconds_u64() <= tssp_domain::UnixTimestamp::MAX_SECONDS);
    }

    #[test]
    fn uuid_v7_file_ids_are_valid_and_distinct() {
        let generator = UuidV7FileIdGenerator;
        let first = generator
            .new_file_id()
            .unwrap_or_else(|error| panic!("first id failed: {error}"));
        let second = generator
            .new_file_id()
            .unwrap_or_else(|error| panic!("second id failed: {error}"));

        assert_ne!(first, second);
        assert_eq!(first.as_str().len(), 36);
    }

    #[test]
    fn session_tokens_are_valid_and_distinct() {
        let generator = RandomSessionTokenGenerator;
        let mut tokens = BTreeSet::new();

        for _index in 0..8 {
            let token = generator
                .new_session_token()
                .unwrap_or_else(|error| panic!("token failed: {error}"));
            assert_eq!(token.as_str().len(), 22);
            tokens.insert(token.as_str().to_owned());
        }

        assert_eq!(tokens.len(), 8);
    }
}
