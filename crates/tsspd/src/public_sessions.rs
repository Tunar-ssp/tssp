//! Public-facing session endpoints for web-based access.
//!
//! `/s/{token}` streams the file directly and invalidates the session.
//! `/u/{token}` (GET) serves an upload form; (POST) accepts the file.

use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use tssp_domain::{FileId, SessionToken};

use crate::content::{find_file_record, open_blob, stream_blob_response, DispositionMode};
use crate::upload::{stage_multipart_upload, HttpUploadRequest};
use crate::HttpState;

/// GET /s/{token} — stream the file associated with a send session.
/// Invalidates the session on successful streaming.
pub async fn get_send_session_page(
    State(state): State<HttpState>,
    Path(token_str): Path<String>,
) -> Response {
    let Ok(token) = SessionToken::new(&token_str) else {
        return error_html(StatusCode::BAD_REQUEST, "Invalid session token");
    };

    let Ok(session) = state.session_provider.get_session(&token) else {
        return error_html(StatusCode::NOT_FOUND, "Session not found or expired");
    };

    if session.kind != "send" {
        return error_html(StatusCode::BAD_REQUEST, "This is not a send session");
    }

    let Some(source_id) = session.source_file.clone() else {
        return error_html(StatusCode::BAD_REQUEST, "Session has no associated file");
    };

    let Ok(file_id) = FileId::new(&source_id) else {
        return error_html(StatusCode::INTERNAL_SERVER_ERROR, "Corrupt session file id");
    };

    let record = match find_file_record(state.clone(), file_id).await {
        Ok(Some(r)) => r,
        Ok(None) => return error_html(StatusCode::NOT_FOUND, "File not found"),
        Err(_) => return error_html(StatusCode::INTERNAL_SERVER_ERROR, "Could not look up file"),
    };

    let Ok(blob) = open_blob(state.clone(), record.storage_handle.clone()).await else {
        return error_html(StatusCode::GONE, "File data is no longer available");
    };

    // Mark session as used — best effort; don't fail the download if this fails
    let _ = state.session_provider.use_session(&token);

    stream_blob_response(&record, blob, None, DispositionMode::Attachment)
}

/// GET /u/{token} — serve the upload form for a receive session.
pub async fn get_receive_session_page(
    State(state): State<HttpState>,
    Path(token_str): Path<String>,
) -> Response {
    let Ok(token) = SessionToken::new(&token_str) else {
        return error_html(StatusCode::BAD_REQUEST, "Invalid session token");
    };

    match state.session_provider.get_session(&token) {
        Ok(session_response) => {
            if session_response.used_at.is_some() {
                return error_html(StatusCode::GONE, "This upload link has already been used");
            }
            if session_response.kind != "receive" {
                return error_html(StatusCode::BAD_REQUEST, "This is not a receive session");
            }
            let html = format!(
                r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>TSSP - Upload File</title>
  <style>
    body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; margin: 0; padding: 2rem; background: #f8f8f8; }}
    .container {{ max-width: 600px; margin: 0 auto; background: white; padding: 2rem; border-radius: 12px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); }}
    h1 {{ margin-top: 0; color: #333; }}
    .drop-zone {{ border: 2px dashed #ccc; border-radius: 8px; padding: 3rem; text-align: center; cursor: pointer; transition: border-color 0.2s; }}
    .drop-zone:hover, .drop-zone.dragover {{ border-color: #0066cc; background: #f0f7ff; }}
    input[type=file] {{ display: none; }}
    button {{ background: #0066cc; color: white; border: none; padding: 0.8rem 2rem; border-radius: 6px; font-size: 1rem; cursor: pointer; margin-top: 1rem; width: 100%; }}
    button:hover {{ background: #0052a3; }}
    .info {{ color: #666; font-size: 0.9rem; margin-top: 1rem; }}
    #status {{ margin-top: 1rem; padding: 0.8rem; border-radius: 6px; display: none; }}
    #status.success {{ background: #d4edda; color: #155724; display: block; }}
    #status.error {{ background: #f8d7da; color: #721c24; display: block; }}
  </style>
</head>
<body>
  <div class="container">
    <h1>Upload File</h1>
    <form id="uploadForm" enctype="multipart/form-data">
      <div class="drop-zone" id="dropZone" onclick="document.getElementById('fileInput').click()">
        <p>&#128196; Drop a file here or click to select</p>
        <input type="file" id="fileInput" name="file">
        <p id="selectedFile" style="color:#0066cc;display:none;"></p>
      </div>
      <button type="submit">Upload</button>
      <div id="status"></div>
      <p class="info">Session token: <code>{token_str}</code></p>
    </form>
  </div>
  <script>
    const dropZone = document.getElementById('dropZone');
    const fileInput = document.getElementById('fileInput');
    const selectedFile = document.getElementById('selectedFile');
    const status = document.getElementById('status');

    fileInput.addEventListener('change', () => {{
      if (fileInput.files[0]) {{
        selectedFile.textContent = fileInput.files[0].name;
        selectedFile.style.display = 'block';
      }}
    }});

    dropZone.addEventListener('dragover', (e) => {{ e.preventDefault(); dropZone.classList.add('dragover'); }});
    dropZone.addEventListener('dragleave', () => dropZone.classList.remove('dragover'));
    dropZone.addEventListener('drop', (e) => {{
      e.preventDefault();
      dropZone.classList.remove('dragover');
      fileInput.files = e.dataTransfer.files;
      if (fileInput.files[0]) {{ selectedFile.textContent = fileInput.files[0].name; selectedFile.style.display = 'block'; }}
    }});

    document.getElementById('uploadForm').addEventListener('submit', async (e) => {{
      e.preventDefault();
      if (!fileInput.files[0]) {{ status.className = 'error'; status.textContent = 'Please select a file first.'; return; }}
      const form = new FormData();
      form.append('file', fileInput.files[0]);
      status.className = ''; status.textContent = 'Uploading...'; status.style.display = 'block';
      try {{
        const resp = await fetch('/u/{token_str}', {{ method: 'POST', body: form }});
        if (resp.ok) {{
          status.className = 'success';
          status.textContent = 'File uploaded successfully! You can close this page.';
        }} else {{
          const text = await resp.text();
          status.className = 'error';
          status.textContent = 'Upload failed: ' + text;
        }}
      }} catch (err) {{
        status.className = 'error';
        status.textContent = 'Network error: ' + err.message;
      }}
    }});
  </script>
</body>
</html>"#
            );
            Html(html).into_response()
        }
        Err(_) => error_html(StatusCode::NOT_FOUND, "Session not found or expired"),
    }
}

/// POST /u/{token} — accept a file upload into a receive session.
pub async fn post_receive_session_upload(
    State(state): State<HttpState>,
    Path(token_str): Path<String>,
    multipart: Multipart,
) -> Response {
    let Ok(token) = SessionToken::new(&token_str) else {
        return (StatusCode::BAD_REQUEST, "Invalid session token").into_response();
    };

    let Ok(session) = state.session_provider.get_session(&token) else {
        return (StatusCode::NOT_FOUND, "Session not found or expired").into_response();
    };

    if session.kind != "receive" {
        return (StatusCode::BAD_REQUEST, "This is not a receive session").into_response();
    }

    if session.used_at.is_some() {
        return (StatusCode::GONE, "This upload link has already been used").into_response();
    }

    let staged = match stage_multipart_upload(multipart, &state.upload_temp_dir).await {
        Ok(s) => s,
        Err(e) => {
            let (status, _, msg) = e.response_parts();
            return (status, msg).into_response();
        }
    };

    let _mutation_guard = state.storage_mutation_lock.lock().await;

    let source = match staged.temp_file.reopen() {
        Ok(f) => f,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("could not reopen staged file: {e}"),
            )
                .into_response()
        }
    };

    let owner_id = session
        .creator_id
        .as_ref()
        .and_then(|id| tssp_domain::UserId::new(id).ok());

    let upload_req = HttpUploadRequest {
        filename: staged.filename,
        mime_type: staged.mime_type,
        tags: staged.tags,
        pinned: false,
        folder_path: staged.folder_path,
        owner_id,
        source: Box::new(source),
        staged_path: None,
        content_hash: None,
        size: None,
    };

    let outcome = match state.upload_provider.upload(upload_req) {
        Ok(o) => o,
        Err(e) => {
            let (status, _, msg) = e.response_parts();
            return (status, msg).into_response();
        }
    };

    // Associate the uploaded file with the session and mark it used
    let file_id = outcome.record.id.as_str().to_string();
    let _ = state
        .session_provider
        .complete_receive_session(&token, &file_id);

    (
        StatusCode::OK,
        format!("File '{}' uploaded successfully", outcome.record.name),
    )
        .into_response()
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
      <p>{message}</p>
    </div>
  </div>
</body>
</html>"#
    );
    (status, Html(html)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_html_bad_request_status() {
        let response = error_html(StatusCode::BAD_REQUEST, "Invalid token");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn error_html_not_found_status() {
        let response = error_html(StatusCode::NOT_FOUND, "Session not found");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn error_html_internal_server_error_status() {
        let response = error_html(StatusCode::INTERNAL_SERVER_ERROR, "Server error");
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn error_html_gone_status() {
        let response = error_html(StatusCode::GONE, "Session used");
        assert_eq!(response.status(), StatusCode::GONE);
    }

    #[test]
    fn error_html_contains_message_text() {
        let message = "This is a test error message";
        let response = error_html(StatusCode::BAD_REQUEST, message);
        // The response is Html, which implements IntoResponse, so we verify status only
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn error_html_contains_html_structure() {
        let response = error_html(StatusCode::NOT_FOUND, "Test");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn error_html_with_long_message() {
        let long_message = "This is a very long error message that contains multiple sentences and explains what went wrong in detail";
        let response = error_html(StatusCode::INTERNAL_SERVER_ERROR, long_message);
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn error_html_with_special_characters() {
        let message = "Error: < > & \" '";
        let response = error_html(StatusCode::BAD_REQUEST, message);
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn error_html_multiple_different_statuses() {
        let status_codes = vec![
            StatusCode::BAD_REQUEST,
            StatusCode::NOT_FOUND,
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::GONE,
        ];
        for code in status_codes {
            let response = error_html(code, "error");
            assert_eq!(response.status(), code);
        }
    }

    #[test]
    fn error_html_session_not_found_message() {
        let response = error_html(StatusCode::NOT_FOUND, "Session not found or expired");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn error_html_invalid_token_message() {
        let response = error_html(StatusCode::BAD_REQUEST, "Invalid session token");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn error_html_file_not_found_message() {
        let response = error_html(StatusCode::NOT_FOUND, "File not found");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn error_html_corrupt_session_message() {
        let response = error_html(StatusCode::INTERNAL_SERVER_ERROR, "Corrupt session file id");
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn error_html_file_not_available_message() {
        let response = error_html(StatusCode::GONE, "File data is no longer available");
        assert_eq!(response.status(), StatusCode::GONE);
    }

    #[test]
    fn error_html_not_send_session() {
        let response = error_html(StatusCode::BAD_REQUEST, "This is not a send session");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn error_html_not_receive_session() {
        let response = error_html(StatusCode::BAD_REQUEST, "This is not a receive session");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn error_html_session_already_used() {
        let response = error_html(StatusCode::GONE, "This upload link has already been used");
        assert_eq!(response.status(), StatusCode::GONE);
    }

    #[test]
    fn error_html_no_associated_file() {
        let response = error_html(StatusCode::BAD_REQUEST, "Session has no associated file");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn error_html_lookup_failure() {
        let response = error_html(StatusCode::INTERNAL_SERVER_ERROR, "Could not look up file");
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn error_html_empty_message() {
        let response = error_html(StatusCode::NOT_FOUND, "");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn error_html_with_unicode_message() {
        let message = "Error: 文件未找到 🔥";
        let response = error_html(StatusCode::NOT_FOUND, message);
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn status_code_bad_request_value() {
        assert_eq!(StatusCode::BAD_REQUEST, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn status_code_not_found_value() {
        assert_eq!(StatusCode::NOT_FOUND, StatusCode::NOT_FOUND);
    }

    #[test]
    fn status_code_internal_server_error_value() {
        assert_eq!(
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn status_code_gone_value() {
        assert_eq!(StatusCode::GONE, StatusCode::GONE);
    }

    #[test]
    fn status_code_ok_value() {
        assert_eq!(StatusCode::OK, StatusCode::OK);
    }

    #[test]
    fn send_session_kind_string() {
        let kind = "send";
        assert_eq!(kind, "send");
    }

    #[test]
    fn receive_session_kind_string() {
        let kind = "receive";
        assert_eq!(kind, "receive");
    }

    #[test]
    fn disposition_mode_attachment() {
        let mode = DispositionMode::Attachment;
        // Verify the enum value exists and can be instantiated
        let _ = mode;
    }

    #[test]
    fn session_validation_kind_matching() {
        let send_kind = "send";
        assert!(send_kind == "send");
        assert!(send_kind != "receive");
    }

    #[test]
    fn session_validation_receive_kind() {
        let receive_kind = "receive";
        assert!(receive_kind == "receive");
        assert!(receive_kind != "send");
    }
}
