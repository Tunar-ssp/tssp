//! Embedded web shell fallback delivery.

use axum::body::Body;
use axum::http::header::{CONTENT_SECURITY_POLICY, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS};
use axum::http::HeaderValue;
use axum::response::{Html, IntoResponse, Response};

pub(crate) async fn web_fallback() -> Response<Body> {
    let mut response = Html(WEB_PLACEHOLDER).into_response();
    let headers = response.headers_mut();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(
        CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; connect-src 'self'; style-src 'self' 'unsafe-inline'",
        ),
    );
    response
}

const WEB_PLACEHOLDER: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>TSSP</title>
  <style>
    :root { color-scheme: light dark; font-family: system-ui, sans-serif; }
    body { margin: 0; min-height: 100vh; display: grid; place-items: center; }
    main { max-width: 42rem; padding: 2rem; }
    h1 { font-size: clamp(2rem, 8vw, 4rem); margin: 0 0 1rem; }
    p { line-height: 1.6; }
  </style>
</head>
<body>
  <main>
    <h1>TSSP</h1>
    <p>The embedded web shell is available. API connectivity starts at <code>/api/v1/status</code>.</p>
  </main>
</body>
</html>"#;

#[cfg(test)]
mod tests {
    use super::web_fallback;
    use axum::http::header::{CONTENT_SECURITY_POLICY, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS};

    #[tokio::test]
    async fn web_fallback_returns_html_with_security_headers() {
        let response = web_fallback().await;

        assert_eq!(response.status(), axum::http::StatusCode::OK);
        let headers = response.headers();
        let ct = headers
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(ct.contains("text/html"), "content-type should be html");
        assert_eq!(
            headers
                .get(X_CONTENT_TYPE_OPTIONS)
                .and_then(|v| v.to_str().ok()),
            Some("nosniff")
        );
        assert!(
            headers.get(CONTENT_SECURITY_POLICY).is_some(),
            "CSP header should be present"
        );
    }

    #[tokio::test]
    async fn web_fallback_body_contains_tssp_title() {
        let response = web_fallback().await;
        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap_or_else(|e| panic!("body read: {e}"));
        let text = String::from_utf8_lossy(&body);
        assert!(text.contains("<title>TSSP</title>"));
        assert!(text.contains("/api/v1/status"));
    }
}
