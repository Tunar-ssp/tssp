//! Embedded dashboard static assets.

pub(crate) const INDEX_HTML: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/web/index.html"));
pub(crate) const SERVICE_WORKER: &str = r"const CACHE_VERSION = 'v2026-05-27-tssp';
const CACHE_ASSETS = [
  '/app',
  '/app/index.html',
  '/app/assets/icons/favicon.ico',
];

self.addEventListener('install', (event) => {
  self.skipWaiting();
  event.waitUntil(
    caches.open(CACHE_VERSION).then((cache) => {
      return cache.addAll(CACHE_ASSETS);
    })
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) => {
      return Promise.all(
        keys.filter((key) => key !== CACHE_VERSION).map((key) => caches.delete(key))
      );
    }).then(() => self.clients.claim())
  );
});

self.addEventListener('fetch', (event) => {
  // Only handle GET requests and exclude API/metrics
  if (event.request.method !== 'GET' || 
      event.request.url.includes('/api/') || 
      event.request.url.includes('/metrics')) {
    return;
  }

  event.respondWith(
    caches.match(event.request).then((response) => {
      if (response) {
        return response;
      }
      return fetch(event.request).catch(() => {
        // Fallback for offline if needed, or just let it fail
        return new Response('Network error occurred', { status: 503 });
      });
    })
  );
});
";

pub(crate) const LEGACY_APP: &str = "console.log('Legacy app loaded');";
pub(crate) const LEGACY_APP_CSS: &str = "body { background: #000; }";

pub(crate) fn asset_for_path(path: &str) -> Option<(&'static str, &'static str)> {
    match path {
        "index.html" | "" => Some((INDEX_HTML, "text/html; charset=utf-8")),
        "sw.js" | "service-worker.js" => {
            Some((SERVICE_WORKER, "application/javascript; charset=utf-8"))
        }
        "app.js" => Some((LEGACY_APP, "application/javascript; charset=utf-8")),
        "app.css" => Some((LEGACY_APP_CSS, "text/css; charset=utf-8")),
        _ => None,
    }
}
