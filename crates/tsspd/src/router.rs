//! HTTP route table and middleware stack.

use axum::extract::DefaultBodyLimit;
use axum::middleware;
use axum::routing::{get, post};
use axum::Router;

use crate::state::HttpState;

async fn options_response() -> (axum::http::StatusCode, axum::http::header::HeaderMap) {
    let mut headers = axum::http::header::HeaderMap::new();
    headers.insert(
        axum::http::header::ALLOW,
        axum::http::HeaderValue::from_static("GET, HEAD, POST, PUT, PATCH, DELETE, OPTIONS"),
    );
    (axum::http::StatusCode::NO_CONTENT, headers)
}

/// Returns the body-limit layer for upload routes.
/// `0` means unlimited; any other value caps the request body.
fn upload_body_limit(max_bytes: u64) -> DefaultBodyLimit {
    if max_bytes == 0 {
        DefaultBodyLimit::disable()
    } else {
        let limit = usize::try_from(max_bytes).unwrap_or(usize::MAX);
        DefaultBodyLimit::max(limit)
    }
}

/// Builds the daemon router.
#[allow(clippy::too_many_lines)]
pub fn build_router(state: HttpState) -> Router {
    let max_upload = state.settings().max_upload_bytes;
    let upload_body_limit = upload_body_limit(max_upload);
    Router::new()
        .route(
            "/api/v1/files",
            post(crate::upload::upload_file)
                .get(crate::list::list_files)
                .options(options_response)
                .layer(upload_body_limit),
        )
        .route(
            "/api/v1/files/batch",
            post(crate::upload::upload_files_batch)
                .options(options_response)
                .layer(upload_body_limit),
        )
        .route(
            "/api/v1/pins",
            get(crate::pins::list_pins).options(options_response),
        )
        .route(
            "/api/v1/pins/reorder",
            post(crate::pins::reorder).options(options_response),
        )
        .route(
            "/api/v1/tags",
            get(crate::tags::list_tags).options(options_response),
        )
        .route(
            "/api/v1/files/{id}/tags",
            post(crate::tags::add_tags).options(options_response),
        )
        .route(
            "/api/v1/files/{id}/tags/{tag}",
            axum::routing::delete(crate::tags::remove_tag).options(options_response),
        )
        .route(
            "/api/v1/files/{id}/pin",
            axum::routing::put(crate::pins::pin)
                .delete(crate::pins::unpin)
                .options(options_response),
        )
        .route(
            "/api/v1/files/{id}/content",
            get(crate::content::get_file_content).options(options_response),
        )
        .route(
            "/api/v1/files/{id}",
            get(crate::file::get_file)
                .delete(crate::delete::delete_file)
                .patch(crate::rename::rename_file)
                .options(options_response),
        )
        .route(
            "/api/v1/files/{id}/visibility",
            axum::routing::patch(crate::visibility::patch_file_visibility)
                .options(options_response),
        )
        .route(
            "/api/v1/files/visibility/bulk",
            post(crate::visibility::bulk_file_visibility).options(options_response),
        )
        .route(
            "/api/v1/folders/move",
            post(crate::folders::move_folder).options(options_response),
        )
        .route(
            "/api/v1/search",
            get(crate::search::search_files).options(options_response),
        )
        .route(
            "/api/v1/notes",
            post(crate::notes::create_note)
                .get(crate::notes::list_notes)
                .options(options_response),
        )
        .route(
            "/api/v1/notes/{id}",
            get(crate::notes::get_note)
                .put(crate::notes::update_note)
                .delete(crate::notes::delete_note)
                .options(options_response),
        )
        .route(
            "/api/v1/notes/{id}/tags",
            post(crate::notes::add_note_tags).options(options_response),
        )
        .route(
            "/api/v1/notes/{id}/tags/{tag}",
            axum::routing::delete(crate::notes::remove_note_tag).options(options_response),
        )
        .route(
            "/api/v1/notes/{id}/pin",
            axum::routing::put(crate::notes::pin_note)
                .delete(crate::notes::unpin_note)
                .options(options_response),
        )
        .route(
            "/api/v1/sessions/send",
            post(crate::sessions::create_send_session).options(options_response),
        )
        .route(
            "/api/v1/sessions/receive",
            post(crate::sessions::create_receive_session).options(options_response),
        )
        .route(
            "/api/v1/sessions/{token}",
            get(crate::sessions::get_session).options(options_response),
        )
        .route(
            "/api/v1/sessions/{token}/use",
            post(crate::sessions::use_session_endpoint).options(options_response),
        )
        .route(
            "/s/{token}",
            get(crate::public_sessions::get_send_session_page),
        )
        .route(
            "/u/{token}",
            get(crate::public_sessions::get_receive_session_page)
                .post(crate::public_sessions::post_receive_session_upload)
                .layer(upload_body_limit),
        )
        .route(
            "/api/v1/files/{id}/thumbnail",
            get(crate::file::get_file_thumbnail).options(options_response),
        )
        .route("/healthz", get(crate::status::healthz))
        .route("/readyz", get(crate::status::readyz))
        .route(
            "/api/v1/status",
            get(crate::status::status).options(options_response),
        )
        .route("/metrics", get(crate::metrics::get_metrics))
        .route(
            "/api/v1/auth/required",
            get(crate::auth::auth_required).options(options_response),
        )
        .route(
            "/api/v1/auth/me",
            get(crate::auth::auth_me).options(options_response),
        )
        .route(
            "/api/v1/auth/login",
            post(crate::auth::auth_login).options(options_response),
        )
        .route(
            "/api/v1/auth/token",
            post(crate::auth::auth_token).options(options_response),
        )
        .route(
            "/api/v1/auth/logout",
            post(crate::auth::auth_logout).options(options_response),
        )
        .route(
            "/api/v1/admin/overview",
            get(crate::admin::admin_overview).options(options_response),
        )
        .route(
            "/api/v1/admin/system",
            get(crate::admin::admin_system).options(options_response),
        )
        .route(
            "/api/v1/admin/files",
            get(crate::admin::admin_list_files).options(options_response),
        )
        .route(
            "/api/v1/admin/files/{id}",
            axum::routing::delete(crate::admin::admin_delete_file).options(options_response),
        )
        .route(
            "/api/v1/admin/folders",
            get(crate::admin::admin_folders).options(options_response),
        )
        .route(
            "/api/v1/admin/corrupt",
            get(crate::admin::admin_corrupt_files).options(options_response),
        )
        .route(
            "/api/v1/admin/cleanup/temp",
            post(crate::admin::admin_cleanup_temp).options(options_response),
        )
        .route(
            "/api/v1/admin/cleanup/sessions",
            post(crate::admin::admin_cleanup_sessions).options(options_response),
        )
        .route(
            "/api/v1/admin/users",
            get(crate::admin::admin_list_users)
                .post(crate::admin::admin_create_user)
                .options(options_response),
        )
        .route(
            "/api/v1/admin/users/{id}",
            axum::routing::delete(crate::admin::admin_delete_user).options(options_response),
        )
        .route(
            "/api/v1/admin/users/{id}/reset-code",
            post(crate::admin::admin_reset_code).options(options_response),
        )
        .route(
            "/api/v1/admin/users/{id}/role",
            axum::routing::put(crate::admin::admin_set_role).options(options_response),
        )
        .route(
            "/api/v1/admin/users/{id}/devices",
            axum::routing::delete(crate::admin::admin_revoke_user_devices)
                .options(options_response),
        )
        .route(
            "/api/v1/admin/devices",
            get(crate::admin::admin_list_devices).options(options_response),
        )
        .route(
            "/api/v1/admin/devices/{token}",
            axum::routing::delete(crate::admin::admin_revoke_device).options(options_response),
        )
        .route(
            "/api/v1/public/files",
            get(crate::public_api::list_public_files).options(options_response),
        )
        .route(
            "/api/v1/workspaces",
            get(crate::workspaces::list_workspaces)
                .post(crate::workspaces::create_workspace)
                .options(options_response),
        )
        .route(
            "/api/v1/workspaces/{id}",
            get(crate::workspaces::get_workspace)
                .put(crate::workspaces::update_workspace)
                .delete(crate::workspaces::delete_workspace)
                .options(options_response),
        )
        .route("/p/{token}", get(crate::public_api::public_download))
        .route("/assets/{*path}", get(crate::web::serve_asset))
        .fallback(crate::web::web_fallback)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::auth::auth_middleware,
        ))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::cors::CorsLayer::very_permissive())
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            axum::http::HeaderValue::from_static("nosniff"),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            axum::http::header::X_FRAME_OPTIONS,
            axum::http::HeaderValue::from_static("DENY"),
        ))
        .with_state(state)
}
