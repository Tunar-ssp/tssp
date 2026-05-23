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
