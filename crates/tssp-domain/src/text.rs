//! Shared string normalization helpers for domain values.

use unicode_normalization::UnicodeNormalization;

/// Returns an NFC-normalized copy of `value`.
pub(crate) fn normalize_unicode(value: &str) -> String {
    value.nfc().collect()
}

/// Trims and collapses all Unicode whitespace to a single ASCII space.
pub(crate) fn trim_and_collapse_whitespace(value: &str) -> String {
    let mut output = String::new();
    let mut pending_space = false;

    for character in normalize_unicode(value).trim().chars() {
        if character.is_whitespace() {
            pending_space = !output.is_empty();
            continue;
        }

        if pending_space {
            output.push(' ');
            pending_space = false;
        }
        output.push(character);
    }

    output
}

/// Truncates a string at a UTF-8 character boundary.
pub(crate) fn truncate_to_bytes(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_owned();
    }

    let mut output = String::new();
    for character in value.chars() {
        let next_len = output.len() + character.len_utf8();
        if next_len > max_bytes {
            break;
        }
        output.push(character);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{normalize_unicode, trim_and_collapse_whitespace, truncate_to_bytes};

    #[test]
    fn collapse_whitespace_trims_edges_and_preserves_words() {
        assert_eq!(
            trim_and_collapse_whitespace(" \t alpha\n\nbeta  "),
            "alpha beta"
        );
    }

    #[test]
    fn normalization_uses_composed_unicode() {
        assert_eq!(normalize_unicode("e\u{301}"), "é");
    }

    #[test]
    fn truncation_keeps_valid_utf8() {
        assert_eq!(truncate_to_bytes("éclair", 2), "é");
    }
}
