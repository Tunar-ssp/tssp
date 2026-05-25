//! Legacy assets are deprecated. New Svelte frontend served from assets/web-v2.

pub(crate) const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>TSSP</title>
  <script>window.location.href = '/app-v2';</script>
</head>
<body>
  Redirecting to application...
</body>
</html>"#;

pub(crate) const SERVICE_WORKER: &str = r#"const CACHE_VERSION = 'v2026-05-25-tssp';
const CACHE_ASSETS = [
  '/app-v2',
  '/app-v2/index.html',
  '/api/v1/status',
];
self.addEventListener('install', () => self.skipWaiting());
self.addEventListener('activate', () => self.clients.claim());
"#;

pub(crate) const HTML_CSP: &str =
    "default-src 'self'; connect-src 'self'; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline'; \
     img-src 'self' data: blob:; base-uri 'self'; form-action 'self'";

/// Legacy asset lookup - all assets now served from /app-v2.
pub(crate) fn asset(path: &str) -> Option<(&'static str, &'static str)> {
    match path {
        "index.html" => Some((INDEX_HTML, "text/html; charset=utf-8")),
        "sw.js" => Some((SERVICE_WORKER, "application/javascript; charset=utf-8")),
        _ => None,
    }
}
