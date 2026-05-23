//! Public-facing session endpoints for web-based access.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use tssp_domain::SessionToken;

use crate::HttpState;

pub async fn get_send_session_page(
    State(state): State<HttpState>,
    Path(token_str): Path<String>,
) -> Response {
    let token = match SessionToken::new(&token_str) {
        Ok(t) => t,
        Err(_) => return error_html(StatusCode::BAD_REQUEST, "Invalid session token"),
    };

    match state.session_provider.get_session(&token) {
        Ok(session_response) => {
            if let Some(source_file) = &session_response.source_file {
                let html = format!(
                    r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>TSSP - Share Session</title>
  <style>
    body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; margin: 0; padding: 2rem; }}
    .container {{ max-width: 600px; margin: 0 auto; }}
    .info {{ background: #f5f5f5; padding: 1rem; border-radius: 8px; }}
  </style>
</head>
<body>
  <div class="container">
    <h1>Download File</h1>
    <div class="info">
      <p>Click below to download the shared file:</p>
      <p><a href="/api/v1/files/{}/content?disposition=attachment">Download</a></p>
    </div>
  </div>
</body>
</html>"#,
                    source_file
                );
                Html(html).into_response()
            } else {
                error_html(
                    StatusCode::BAD_REQUEST,
                    "This send session has no associated file",
                )
            }
        }
        Err(_) => error_html(StatusCode::NOT_FOUND, "Session not found or expired"),
    }
}

pub async fn get_receive_session_page(
    State(state): State<HttpState>,
    Path(token_str): Path<String>,
) -> Response {
    let token = match SessionToken::new(&token_str) {
        Ok(t) => t,
        Err(_) => return error_html(StatusCode::BAD_REQUEST, "Invalid session token"),
    };

    match state.session_provider.get_session(&token) {
        Ok(session_response) => {
            if session_response.kind == "receive" {
                let html = format!(
                    r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>TSSP - Upload Session</title>
  <style>
    body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; margin: 0; padding: 2rem; }}
    .container {{ max-width: 600px; margin: 0 auto; }}
    .info {{ background: #f5f5f5; padding: 1rem; border-radius: 8px; }}
  </style>
</head>
<body>
  <div class="container">
    <h1>Upload File</h1>
    <div class="info">
      <p>Session token: {}</p>
      <p>Upload your file to this session.</p>
      <p><a href="/api/v1/sessions/{}/upload">Upload</a></p>
    </div>
  </div>
</body>
</html>"#,
                    token_str, token_str
                );
                Html(html).into_response()
            } else {
                error_html(StatusCode::BAD_REQUEST, "This is not a receive session")
            }
        }
        Err(_) => error_html(StatusCode::NOT_FOUND, "Session not found or expired"),
    }
}

fn error_html(status: StatusCode, message: &str) -> Response {
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Error</title>
  <style>
    body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; margin: 0; padding: 2rem; }}
    .container {{ max-width: 600px; margin: 0 auto; }}
    .error {{ background: #fee; padding: 1rem; border-radius: 8px; color: #c00; }}
  </style>
</head>
<body>
  <div class="container">
    <div class="error">
      <h1>Error</h1>
      <p>{}</p>
    </div>
  </div>
</body>
</html>"#,
        message
    );
    (status, Html(html)).into_response()
}
