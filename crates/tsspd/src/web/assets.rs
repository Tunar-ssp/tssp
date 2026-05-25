//! Embedded dashboard static assets.

pub(crate) const INDEX_HTML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web-v2/index.html"
));
pub(crate) const SERVICE_WORKER: &str = r"const CACHE_VERSION = 'v2026-05-25-tssp';
const CACHE_ASSETS = [
  '/app-v2',
  '/app-v2/index.html',
  '/app-v2/app.js',
  '/app-v2/app.css',
  '/app-v2/assets/icons/favicon.ico',
];

self.addEventListener('install', (event) => {
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
    })
  );
});

self.addEventListener('fetch', (event) => {
  event.respondWith(
    caches.match(event.request).then((response) => {
      return response || fetch(event.request);
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
