use tower::ServiceExt;
use axum::extract::connect_info::ConnectInfo;
use std::net::SocketAddr;

#[tokio::test]
async fn test_rate_limiting_enforced() {
    let (_temp, router) = super::common::real_storage_app();

    let mut last_status = axum::http::StatusCode::OK;
    for _i in 0..5 {
        let mut attempt_request = axum::http::Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("content-type", "application/json")
            .header("x-forwarded-for", "192.0.2.1")
            .body(axum::body::Body::from(
                r#"{"password":"wrong"}"#.to_string(),
            ))
            .unwrap();

        // Add ConnectInfo extension
        let peer_addr: SocketAddr = "127.0.0.1:1234".parse().unwrap();
        attempt_request.extensions_mut().insert(ConnectInfo(peer_addr));

        let router_clone = router.clone();
        let response = router_clone.oneshot(attempt_request).await.unwrap();
        last_status = response.status();
    }

    assert_ne!(last_status, axum::http::StatusCode::TOO_MANY_REQUESTS, "5 attempts shouldn't be limited yet");

    let mut sixth_attempt = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("content-type", "application/json")
        .header("x-forwarded-for", "192.0.2.1")
        .body(axum::body::Body::from(r#"{"password":"wrong"}"#))
        .unwrap();

    // Add ConnectInfo extension
    let peer_addr: SocketAddr = "127.0.0.1:1234".parse().unwrap();
    sixth_attempt.extensions_mut().insert(ConnectInfo(peer_addr));

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

#[tokio::test]
async fn test_rate_limiting_debug_statuses() {
    let (_temp, router) = super::common::real_storage_app();

    for i in 0..2 {
        let mut attempt_request = axum::http::Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("content-type", "application/json")
            .header("x-forwarded-for", "192.0.2.1")
            .body(axum::body::Body::from(
                r#"{"password":"wrong"}"#.to_string(),
            ))
            .unwrap();

        // Add ConnectInfo extension
        let peer_addr: SocketAddr = "127.0.0.1:1234".parse().unwrap();
        attempt_request.extensions_mut().insert(ConnectInfo(peer_addr));

        let router_clone = router.clone();
        let (parts, body) = router_clone.oneshot(attempt_request).await.unwrap().into_parts();
        let body_bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let body_str = String::from_utf8_lossy(&body_bytes);
        println!("Attempt {}: {} - Body: {}", i + 1, parts.status, body_str);
    }
}
