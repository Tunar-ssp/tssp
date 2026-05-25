//! Input validation utilities for all endpoints

use crate::error_handler::ApiError;
use axum::http::StatusCode;

/// Validate file name
pub fn validate_filename(name: &str) -> Result<(), ApiError> {
    if name.is_empty() {
        return Err(ApiError::validation("File name cannot be empty"));
    }

    if name.len() > 256 {
        return Err(ApiError::validation("File name too long (max 256 characters)"));
    }

    if name.contains('\0') {
        return Err(ApiError::validation("File name contains invalid characters"));
    }

    if name.starts_with('/') || name.contains("../") {
        return Err(ApiError::validation("File name contains invalid path traversal"));
    }

    Ok(())
}

/// Validate folder path
pub fn validate_folder_path(path: &str) -> Result<(), ApiError> {
    if path.is_empty() {
        return Ok(());
    }

    if path.contains('\0') {
        return Err(ApiError::validation("Path contains invalid characters"));
    }

    if path.contains("../") {
        return Err(ApiError::validation("Path contains invalid path traversal"));
    }

    Ok(())
}

/// Validate email address
pub fn validate_email(email: &str) -> Result<(), ApiError> {
    if email.is_empty() || !email.contains('@') || email.len() > 254 {
        return Err(ApiError::validation("Invalid email address"));
    }
    Ok(())
}

/// Validate note title
pub fn validate_note_title(title: &str) -> Result<(), ApiError> {
    if title.len() > 256 {
        return Err(ApiError::validation("Note title too long (max 256 characters)"));
    }
    Ok(())
}

/// Validate tag
pub fn validate_tag(tag: &str) -> Result<(), ApiError> {
    if tag.is_empty() {
        return Err(ApiError::validation("Tag cannot be empty"));
    }

    if tag.len() > 50 {
        return Err(ApiError::validation("Tag too long (max 50 characters)"));
    }

    if !tag.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(ApiError::validation("Tag contains invalid characters"));
    }

    Ok(())
}

/// Validate pagination offset and limit
pub fn validate_pagination(offset: Option<u32>, limit: Option<u32>) -> Result<(u32, u32), ApiError> {
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(50).min(500); // Max 500 items per page

    if offset > 1_000_000 {
        return Err(ApiError::validation("Offset too large"));
    }

    if limit == 0 {
        return Err(ApiError::validation("Limit must be greater than 0"));
    }

    Ok((offset, limit))
}

/// Validate file content length
pub fn validate_file_size(size: u64, max_bytes: u64) -> Result<(), ApiError> {
    if max_bytes > 0 && size > max_bytes {
        return Err(ApiError::new(
            "FILE_TOO_LARGE",
            format!("File size exceeds limit ({max_bytes} bytes max)"),
            StatusCode::PAYLOAD_TOO_LARGE,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_filename() {
        assert!(validate_filename("test.txt").is_ok());
        assert!(validate_filename("").is_err());
        assert!(validate_filename("../etc/passwd").is_err());
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("invalid").is_err());
    }

    #[test]
    fn test_validate_pagination() {
        let (offset, limit) = validate_pagination(Some(0), Some(50)).unwrap();
        assert_eq!(offset, 0);
        assert_eq!(limit, 50);

        let (_offset, limit) = validate_pagination(None, Some(1000)).unwrap();
        assert_eq!(limit, 500); // Should be capped at 500
    }
}
