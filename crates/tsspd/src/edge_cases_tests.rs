//! Edge case tests for critical functionality.

#[cfg(test)]
mod edge_cases {
    use tssp_app::{normalize_folder_path, validate_folder_path};

    #[test]
    fn folder_path_empty_string() {
        assert!(validate_folder_path("").is_ok());
    }

    #[test]
    fn folder_path_only_slashes() {
        assert_eq!(normalize_folder_path("///"), "");
    }

    #[test]
    fn folder_path_unicode_characters() {
        let path = "photos/2024/ñew_folder";
        assert!(validate_folder_path(path).is_ok());
    }

    #[test]
    fn folder_path_spaces_allowed() {
        let path = "my files/new folder";
        assert!(validate_folder_path(path).is_ok());
    }

    #[test]
    fn folder_path_dots_but_not_traversal() {
        // "..." is not "..", so should be allowed
        assert!(validate_folder_path("files/...").is_ok());
    }

    #[test]
    fn folder_path_max_length() {
        // Exactly at max
        let path = "a".repeat(1024);
        assert!(validate_folder_path(&path).is_ok());

        // Over max
        let path = "a".repeat(1025);
        assert!(validate_folder_path(&path).is_err());
    }

    #[test]
    fn folder_path_contains_null_byte() {
        let path = "folder\0name";
        assert!(validate_folder_path(path).is_err());
    }

    #[test]
    fn folder_normalization_backslashes() {
        assert_eq!(
            normalize_folder_path("photos\\vacation\\2024"),
            "photos/vacation/2024"
        );
    }

    #[test]
    fn folder_normalization_multiple_slashes() {
        assert_eq!(normalize_folder_path("///photos///"), "photos");
    }

    #[test]
    fn folder_traversal_in_middle() {
        assert!(validate_folder_path("photos/../../../etc").is_err());
    }

    #[test]
    fn folder_traversal_at_start() {
        assert!(validate_folder_path("../photos").is_err());
    }

    #[test]
    fn folder_traversal_at_end() {
        assert!(validate_folder_path("photos/..").is_err());
    }

    #[test]
    fn folder_single_dot_allowed() {
        // Single "." is allowed - it just means current directory
        assert!(validate_folder_path("files/.current").is_ok());
    }
}
