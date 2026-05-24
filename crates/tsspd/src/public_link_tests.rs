#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use crate::http_tests::common::real_storage_app;
    use axum::body::Body;
    use axum::http::Request;
    use axum::http::StatusCode;
    use tower::Service;

    #[tokio::test]
    async fn test_public_link_security() {
        let (_temp, mut app) = real_storage_app();

        // 1. Unauthenticated access to /p/{token}
        let response = app
            .call(
                Request::builder()
                    .method("GET")
                    .uri("/p/some-invalid-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("request failed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // 2. Path traversal in public token
        let response = app
            .call(
                Request::builder()
                    .method("GET")
                    .uri("/p/..%2f..%2fetc%2fpasswd")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("request failed");

        assert!(
            response.status() == StatusCode::NOT_FOUND
                || response.status() == StatusCode::BAD_REQUEST
        );
    }
}
