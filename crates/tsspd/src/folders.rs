//! Virtual folder path normalization for uploads and listing.

/// Normalizes a client-supplied folder path for storage and filtering.
///
/// # Errors
///
/// Returns a validation message when the path is invalid.
pub fn normalize_folder_path(value: &str) -> Result<String, String> {
    let trimmed = value.trim().trim_matches('/');
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    if trimmed.contains("..") {
        return Err("folder path must not contain '..'".to_owned());
    }
    if trimmed
        .chars()
        .any(|ch| !(ch.is_ascii_alphanumeric() || matches!(ch, '/' | '-' | '_' | '.')))
    {
        return Err("folder path contains invalid characters".to_owned());
    }
    Ok(trimmed.to_owned())
}

#[cfg(test)]
mod tests {
    use super::normalize_folder_path;

    #[test]
    fn trims_slashes() {
        assert_eq!(
            normalize_folder_path("/photos/vacation/").unwrap(),
            "photos/vacation"
        );
    }

    #[test]
    fn rejects_parent_segments() {
        assert!(normalize_folder_path("../secret").is_err());
    }
}
