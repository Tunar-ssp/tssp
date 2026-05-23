//! Prometheus-format metrics endpoint.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::HttpState;

/// Metrics output in Prometheus text format.
pub async fn get_metrics(State(state): State<HttpState>) -> Response {
    match state.stats_provider.stats() {
        Ok(stats) => {
            let uptime_secs = state.started_at.elapsed().as_secs();

            let metrics = format!(
                "# HELP tssp_files_total Total number of files stored\n\
                 # TYPE tssp_files_total gauge\n\
                 tssp_files_total {}\n\
                 \n\
                 # HELP tssp_tags_total Total number of unique tags\n\
                 # TYPE tssp_tags_total gauge\n\
                 tssp_tags_total {}\n\
                 \n\
                 # HELP tssp_pinned_files_total Number of pinned files\n\
                 # TYPE tssp_pinned_files_total gauge\n\
                 tssp_pinned_files_total {}\n\
                 \n\
                 # HELP tssp_recent_uploads_24h Files uploaded in the last 24 hours\n\
                 # TYPE tssp_recent_uploads_24h gauge\n\
                 tssp_recent_uploads_24h {}\n\
                 \n\
                 # HELP tssp_uptime_seconds Daemon uptime in seconds\n\
                 # TYPE tssp_uptime_seconds gauge\n\
                 tssp_uptime_seconds {}\n",
                stats.file_count,
                stats.tag_count,
                stats.pinned_count,
                stats.recent_upload_count,
                uptime_secs,
            );

            (StatusCode::OK, metrics).into_response()
        }
        Err(_) => {
            let error = "# ERROR: Failed to retrieve metrics\n";
            (StatusCode::INTERNAL_SERVER_ERROR, error).into_response()
        }
    }
}
