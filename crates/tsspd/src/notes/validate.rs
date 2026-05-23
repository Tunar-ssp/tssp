//! Request validation for note HTTP handlers.

use tssp_domain::{DomainError, NoteId, MAX_NOTE_BODY_BYTES};

use super::error::HttpNoteError;

/// Parses and validates a note id from a URL segment.
pub(crate) fn parse_note_id(raw: String) -> Result<NoteId, HttpNoteError> {
    NoteId::new(raw).map_err(|error| HttpNoteError::InvalidRequest {
        message: error.to_string(),
    })
}

/// Rejects note bodies that exceed the domain size limit before persistence.
pub(crate) fn validate_note_body(body: &str) -> Result<(), HttpNoteError> {
    if body.trim().is_empty() {
        return Err(HttpNoteError::InvalidRequest {
            message: "note body must not be empty".to_owned(),
        });
    }
    if body.len() > MAX_NOTE_BODY_BYTES {
        return Err(HttpNoteError::PayloadTooLarge {
            message: format!("note body exceeds maximum size of {MAX_NOTE_BODY_BYTES} bytes"),
        });
    }
    // Exercise domain rules early so clients get consistent errors.
    if let Err(error) = tssp_domain::NoteBody::new(body) {
        return Err(map_domain_validation(error));
    }
    Ok(())
}

fn map_domain_validation(error: DomainError) -> HttpNoteError {
    match error {
        DomainError::TooLong { field, max, .. } => HttpNoteError::PayloadTooLarge {
            message: format!("{field} exceeds maximum size of {max} bytes"),
        },
        other => HttpNoteError::InvalidRequest {
            message: other.to_string(),
        },
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{parse_note_id, validate_note_body};
    use tssp_domain::MAX_NOTE_BODY_BYTES;

    #[test]
    fn parse_note_id_rejects_empty_value() {
        assert!(parse_note_id(String::new()).is_err());
    }

    #[test]
    fn validate_note_body_rejects_empty_trimmed_content() {
        assert!(validate_note_body("  \n").is_err());
    }

    #[test]
    fn validate_note_body_rejects_oversized_payload() {
        let body = "x".repeat(MAX_NOTE_BODY_BYTES + 1);
        let error = validate_note_body(&body).unwrap_err();
        assert!(matches!(
            error,
            crate::notes::error::HttpNoteError::PayloadTooLarge { .. }
        ));
    }
}
