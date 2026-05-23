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
    * { box-sizing: border-box; }
    :root {
      color-scheme: light dark;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    }
    body { margin: 0; padding: 1rem; background: #f5f5f5; }
    .container { max-width: 1200px; margin: 0 auto; }
    header { background: white; padding: 1.5rem; border-radius: 8px; margin-bottom: 2rem; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
    h1 { margin: 0; font-size: 2rem; }
    .status { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 1rem; margin-top: 1rem; }
    .stat { background: #f9f9f9; padding: 1rem; border-radius: 4px; }
    .stat-value { font-size: 1.5rem; font-weight: bold; }
    .stat-label { font-size: 0.85rem; color: #666; margin-top: 0.5rem; }
    main { background: white; padding: 2rem; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
    .actions { display: flex; gap: 1rem; margin-bottom: 2rem; flex-wrap: wrap; }
    button { padding: 0.75rem 1.5rem; border: none; border-radius: 4px; cursor: pointer; font-size: 1rem; }
    .btn-primary { background: #0066cc; color: white; }
    .btn-primary:hover { background: #0052a3; }
    .btn-secondary { background: #f0f0f0; color: #333; border: 1px solid #ddd; }
    .btn-secondary:hover { background: #e8e8e8; }
    .file-list { list-style: none; padding: 0; margin: 0; }
    .file-item { padding: 1rem; border-bottom: 1px solid #eee; display: flex; justify-content: space-between; align-items: center; }
    .file-item:last-child { border-bottom: none; }
    .file-name { font-weight: 500; }
    .file-meta { font-size: 0.9rem; color: #666; margin-top: 0.25rem; }
    #loading { text-align: center; color: #666; }
    .error { background: #fee; color: #c00; padding: 1rem; border-radius: 4px; margin-bottom: 1rem; }
  </style>
</head>
<body>
  <div class="container">
    <header>
      <h1>TSSP</h1>
      <p>Self-hosted file transfer for local networks</p>
      <div class="status">
        <div class="stat">
          <div class="stat-value" id="file-count">-</div>
          <div class="stat-label">Files</div>
        </div>
        <div class="stat">
          <div class="stat-value" id="tag-count">-</div>
          <div class="stat-label">Tags</div>
        </div>
        <div class="stat">
          <div class="stat-value" id="storage-used">-</div>
          <div class="stat-label">Storage Used</div>
        </div>
        <div class="stat">
          <div class="stat-value" id="uptime">-</div>
          <div class="stat-label">Uptime</div>
        </div>
      </div>
    </header>
    <main>
      <div class="actions">
        <button class="btn-primary" onclick="document.getElementById('upload-input').click()">Upload File</button>
        <button class="btn-secondary" id="refresh-btn" onclick="loadFiles()">Refresh</button>
      </div>
      <input type="file" id="upload-input" style="display: none;" multiple>
      <div id="error-message"></div>
      <div id="files-container">
        <div id="loading">Loading files...</div>
      </div>
    </main>
  </div>
  <script>
    async function loadStatus() {
      try {
        const res = await fetch('/api/v1/status');
        const data = await res.json();
        document.getElementById('file-count').textContent = data.file_count;
        document.getElementById('tag-count').textContent = data.tag_count;
        document.getElementById('uptime').textContent = Math.floor(data.uptime_seconds / 3600) + 'h';
      } catch (err) {
        console.error('Failed to load status:', err);
      }
    }
    async function loadFiles() {
      try {
        const res = await fetch('/api/v1/files?limit=50');
        if (!res.ok) throw new Error('Failed to load files');
        const data = await res.json();
        const container = document.getElementById('files-container');
        if (data.items && data.items.length > 0) {
          const html = data.items.map(f => `
            <div class="file-item">
              <div>
                <div class="file-name">${escapeHtml(f.name)}</div>
                <div class="file-meta">${(f.size / 1024).toFixed(1)} KB • ${new Date(f.uploaded_at * 1000).toLocaleDateString()}</div>
              </div>
              <a href="/api/v1/files/${f.id}/content?disposition=attachment" style="color: #0066cc; text-decoration: none;">Download</a>
            </div>
          `).join('');
          container.innerHTML = html;
        } else {
          container.innerHTML = '<div style="color: #666; text-align: center; padding: 2rem;">No files yet</div>';
        }
      } catch (err) {
        document.getElementById('error-message').innerHTML = `<div class="error">Error: ${err.message}</div>`;
      }
    }
    function escapeHtml(text) {
      const div = document.createElement('div');
      div.textContent = text;
      return div.innerHTML;
    }
    document.getElementById('upload-input').addEventListener('change', async (e) => {
      const files = Array.from(e.target.files);
      if (files.length === 0) return;
      const formData = new FormData();
      files.forEach(f => formData.append('file', f));
      try {
        const res = await fetch('/api/v1/files', { method: 'POST', body: formData });
        if (!res.ok) throw new Error(`Upload failed: ${res.status}`);
        loadFiles();
        loadStatus();
      } catch (err) {
        document.getElementById('error-message').innerHTML = `<div class="error">Upload failed: ${err.message}</div>`;
      }
      e.target.value = '';
    });
    loadStatus();
    loadFiles();
    setInterval(() => { loadStatus(); loadFiles(); }, 5000);
  </script>
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
        let body = axum::body::to_bytes(response.into_body(), 65536)
            .await
            .unwrap_or_else(|e| panic!("body read: {e}"));
        let text = String::from_utf8_lossy(&body);
        assert!(text.contains("<title>TSSP</title>"));
        assert!(text.contains("/api/v1/status"));
    }
}
