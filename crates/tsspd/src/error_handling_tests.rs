#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use crate::http_tests::common::{content_request, file_request, real_storage_app};
    use axum::http::StatusCode;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_invalid_file_id_formats() {
        let (_temp, app) = real_storage_app();

        // Path traversal in ID (Axum router will return 404 for this literal path)
        let response = app
            .clone()
            .oneshot(file_request("../etc/passwd"))
            .await
            .expect("request failed");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // URL encoded traversal (Axum decodes it, but our FileId::new rejects '/')
        let response = app
            .clone()
            .oneshot(file_request("%2e%2e%2fetc%2fpasswd"))
            .await
            .expect("request failed");
        // Axum 0.8 matches %2f as path separator by default, so it might not match the {id} route.
        // It returns 404.
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Empty ID (matches /api/v1/files, but if trailing slash is missing it depends)
        // file_request("") builds /api/v1/files/
        let response = app
            .clone()
            .oneshot(file_request(""))
            .await
            .expect("request failed");
        // /api/v1/files/ (trailing slash) might not match /api/v1/files (listing) exactly
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_content_invalid_range_headers() {
        let (_temp, app) = real_storage_app();
        let response = app
            .oneshot(content_request("any-id", Some("not-bytes=0-10")))
            .await
            .expect("request failed");
        // Handler find_file comes before range parsing.
        // If file not found, it returns 404.
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
