use tower::ServiceExt;

#[tokio::test]
async fn test_rate_limiting_enforced() {
    let (_temp, router) = super::common::real_storage_app();

    let mut last_status = axum::http::StatusCode::OK;
    for _i in 0..5 {
        let attempt_request = axum::http::Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("content-type", "application/json")
            .header("x-forwarded-for", "192.0.2.1")
            .body(axum::body::Body::from(
                r#"{"password":"wrong"}"#.to_string(),
            ))
            .unwrap();

        let router_clone = router.clone();
        let response = router_clone.oneshot(attempt_request).await.unwrap();
        last_status = response.status();
    }

    assert_ne!(last_status, axum::http::StatusCode::TOO_MANY_REQUESTS, "5 attempts shouldn't be limited yet");

    let sixth_attempt = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("content-type", "application/json")
        .header("x-forwarded-for", "192.0.2.1")
        .body(axum::body::Body::from(r#"{"password":"wrong"}"#))
        .unwrap();

    let response = router.oneshot(sixth_attempt).await.unwrap();
    assert_eq!(
        response.status(),
        axum::http::StatusCode::TOO_MANY_REQUESTS,
        "6th attempt should be rate limited"
    );
}

#[tokio::test]
async fn test_rate_limiter_unit() {
    let limiter = crate::rate_limit::RateLimiter::new();
    let ip: std::net::IpAddr = "192.0.2.1".parse().unwrap();

    for _ in 0..5 {
        assert!(limiter.check_and_record_attempt(ip).await);
        limiter.record_failure(ip).await;
    }

    assert!(!limiter.check_and_record_attempt(ip).await);
}

#[tokio::test]
async fn test_rate_limiter_resets_on_success() {
    let limiter = crate::rate_limit::RateLimiter::new();
    let ip: std::net::IpAddr = "192.0.2.1".parse().unwrap();

    for _ in 0..3 {
        limiter.record_failure(ip).await;
    }

    limiter.record_success(ip).await;

    for _ in 0..5 {
        assert!(limiter.check_and_record_attempt(ip).await);
        limiter.record_failure(ip).await;
    }

    assert!(!limiter.check_and_record_attempt(ip).await);
}
