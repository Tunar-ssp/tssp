//! Embedded dashboard static assets.

pub(crate) const INDEX_HTML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/index.html"
));
pub(crate) const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/manifest.webmanifest"
));
pub(crate) const SERVICE_WORKER: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/web/sw.js"));

pub(crate) const CSS_TOKENS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/css/tokens.css"
));
pub(crate) const CSS_BASE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/css/base.css"
));
pub(crate) const CSS_LAYOUT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/css/layout.css"
));
pub(crate) const CSS_COMPONENTS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/css/components.css"
));
pub(crate) const CSS_VIEWS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/css/views.css"
));
pub(crate) const CSS_MOBILE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/css/mobile.css"
));

pub(crate) const JS_API: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/web/js/api.js"));
pub(crate) const JS_STATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/js/state.js"
));
pub(crate) const JS_UPLOAD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/js/upload.js"
));
pub(crate) const JS_VIEWS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/js/views.js"
));
pub(crate) const JS_FILES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/js/files.js"
));
pub(crate) const JS_NOTES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/js/notes.js"
));
pub(crate) const JS_ADMIN: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/js/admin.js"
));
pub(crate) const JS_EDITOR: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/web/js/editor.js"
));
pub(crate) const JS_APP: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/web/js/app.js"));

pub(crate) const HTML_CSP: &str =
    "default-src 'self'; connect-src 'self'; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline'; \
     img-src 'self' data: blob:; base-uri 'self'; form-action 'self'";

/// Lookup embedded asset bytes and MIME type by path under `assets/web/`.
pub(crate) fn asset(path: &str) -> Option<(&'static str, &'static str)> {
    match path {
        "index.html" => Some((INDEX_HTML, "text/html; charset=utf-8")),
        "manifest.webmanifest" => Some((MANIFEST, "application/manifest+json; charset=utf-8")),
        "sw.js" => Some((SERVICE_WORKER, "application/javascript; charset=utf-8")),
        "css/tokens.css" => Some((CSS_TOKENS, "text/css; charset=utf-8")),
        "css/base.css" => Some((CSS_BASE, "text/css; charset=utf-8")),
        "css/layout.css" => Some((CSS_LAYOUT, "text/css; charset=utf-8")),
        "css/components.css" => Some((CSS_COMPONENTS, "text/css; charset=utf-8")),
        "css/views.css" => Some((CSS_VIEWS, "text/css; charset=utf-8")),
        "css/mobile.css" => Some((CSS_MOBILE, "text/css; charset=utf-8")),
        "js/api.js" => Some((JS_API, "application/javascript; charset=utf-8")),
        "js/state.js" => Some((JS_STATE, "application/javascript; charset=utf-8")),
        "js/upload.js" => Some((JS_UPLOAD, "application/javascript; charset=utf-8")),
        "js/views.js" => Some((JS_VIEWS, "application/javascript; charset=utf-8")),
        "js/files.js" => Some((JS_FILES, "application/javascript; charset=utf-8")),
        "js/notes.js" => Some((JS_NOTES, "application/javascript; charset=utf-8")),
        "js/admin.js" => Some((JS_ADMIN, "application/javascript; charset=utf-8")),
        "js/editor.js" => Some((JS_EDITOR, "application/javascript; charset=utf-8")),
        "js/app.js" => Some((JS_APP, "application/javascript; charset=utf-8")),
        _ => None,
    }
}
