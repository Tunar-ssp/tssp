/* Minimal offline shell cache for TSSP dashboard */
const CACHE = "tssp-v4";
const ASSETS = [
  "/",
  "/assets/css/tokens.css",
  "/assets/css/base.css",
  "/assets/css/layout.css",
  "/assets/css/components.css",
  "/assets/css/views.css",
  "/assets/css/mobile.css",
  "/assets/js/api.js",
  "/assets/js/state.js",
  "/assets/js/upload.js",
  "/assets/js/views.js",
  "/assets/js/files.js",
  "/assets/js/notes.js",
  "/assets/js/admin.js",
  "/assets/js/editor.js",
  "/assets/js/app.js",
];

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE).then((cache) => cache.addAll(ASSETS)).then(() => self.skipWaiting())
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) => Promise.all(keys.filter((key) => key !== CACHE).map((key) => caches.delete(key))))
      .then(() => self.clients.claim())
  );
});

self.addEventListener("fetch", (event) => {
  if (event.request.method !== "GET") return;
  const url = new URL(event.request.url);
  if (url.pathname.startsWith("/api/")) return;
  event.respondWith(
    caches.match(event.request).then((cached) => cached || fetch(event.request))
  );
});
