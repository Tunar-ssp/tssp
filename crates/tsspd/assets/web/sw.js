/* Minimal offline shell cache for TSSP dashboard */
const CACHE = "tssp-v7";
const VERSION = "2026-05-24-4";
const withVersion = (path) => `${path}?v=${VERSION}`;

const ASSETS = [
  withVersion("/assets/css/tokens.css"),
  withVersion("/assets/css/base.css"),
  withVersion("/assets/css/layout.css"),
  withVersion("/assets/css/components.css"),
  withVersion("/assets/css/views.css"),
  withVersion("/assets/css/mobile.css"),
  withVersion("/assets/css/product.css"),
  withVersion("/assets/manifest.webmanifest"),
  withVersion("/assets/js/api.js"),
  withVersion("/assets/js/ui/format.js"),
  withVersion("/assets/js/ui/render.js"),
  withVersion("/assets/js/ui/toast.js"),
  withVersion("/assets/js/ui/dialogs.js"),
  withVersion("/assets/js/state.js"),
  withVersion("/assets/js/upload.js"),
  withVersion("/assets/js/features/overview.js"),
  withVersion("/assets/js/features/search.js"),
  withVersion("/assets/js/features/media.js"),
  withVersion("/assets/js/features/public.js"),
  withVersion("/assets/js/features/workspaces.js"),
  withVersion("/assets/js/files.js"),
  withVersion("/assets/js/notes.js"),
  withVersion("/assets/js/admin.js"),
  withVersion("/assets/js/editor.js"),
  withVersion("/assets/js/app.js"),
  "/assets/app.js",
  "/assets/app.css",
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
  if (url.origin !== self.location.origin) return;
  if (url.pathname.startsWith("/api/")) return;
  if (event.request.mode === "navigate" || event.request.destination === "document") {
    event.respondWith(
      (async () => {
        try {
          const response = await fetch(event.request);
          if (response.ok) {
            const cache = await caches.open(CACHE);
            cache.put("/", response.clone());
          }
          return response;
        } catch {
          const cached = (await caches.match(event.request)) || (await caches.match("/"));
          if (cached) {
            return cached;
          }
          return Response.error();
        }
      })(),
    );
    return;
  }

  event.respondWith(
    (async () => {
      try {
        const response = await fetch(event.request);
        if (response.ok) {
          const cache = await caches.open(CACHE);
          cache.put(event.request, response.clone());
        }
        return response;
      } catch {
        const cached = await caches.match(event.request);
        return cached || Response.error();
      }
    })(),
  );
});
