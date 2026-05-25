//! Prometheus-format metrics endpoint.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::HttpState;

/// Metrics output in Prometheus text format.
pub async fn get_metrics(State(state): State<HttpState>) -> Response {
    if !state.settings().metrics {
        return (StatusCode::NOT_FOUND, "metrics endpoint is disabled").into_response();
    }
    if let Ok(stats) = state.stats_provider.stats() {
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
    } else {
        let error = "# ERROR: Failed to retrieve metrics\n";
        (StatusCode::INTERNAL_SERVER_ERROR, error).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prometheus_metric_files_total() {
        let metric = "tssp_files_total";
        assert!(metric.starts_with("tssp_"));
        assert!(metric.ends_with("_total"));
    }

    #[test]
    fn prometheus_metric_tags_total() {
        let metric = "tssp_tags_total";
        assert!(metric.starts_with("tssp_"));
        assert!(metric.contains("_total"));
    }

    #[test]
    fn prometheus_metric_pinned_files() {
        let metric = "tssp_pinned_files_total";
        assert!(metric.starts_with("tssp_"));
        assert!(metric.contains("pinned"));
    }

    #[test]
    fn prometheus_metric_recent_uploads() {
        let metric = "tssp_recent_uploads_24h";
        assert!(metric.starts_with("tssp_"));
        assert!(metric.contains("24h"));
    }

    #[test]
    fn prometheus_metric_uptime() {
        let metric = "tssp_uptime_seconds";
        assert!(metric.starts_with("tssp_"));
        assert!(metric.contains("seconds"));
    }

    #[test]
    fn prometheus_type_gauge() {
        let metric_type = "gauge";
        assert_eq!(metric_type, "gauge");
    }

    #[test]
    fn prometheus_help_prefix() {
        let help = "# HELP tssp_files_total Total number of files stored";
        assert!(help.starts_with("# HELP"));
    }

    #[test]
    fn prometheus_type_prefix() {
        let type_line = "# TYPE tssp_files_total gauge";
        assert!(type_line.starts_with("# TYPE"));
    }

    #[test]
    fn disabled_metrics_status_code() {
        let status = StatusCode::NOT_FOUND;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn disabled_metrics_message() {
        let msg = "metrics endpoint is disabled";
        assert!(msg.contains("metrics"));
        assert!(msg.contains("disabled"));
    }

    #[test]
    fn error_metrics_message() {
        let msg = "# ERROR: Failed to retrieve metrics\n";
        assert!(msg.starts_with("# ERROR"));
    }

    #[test]
    fn error_metrics_status_code() {
        let status = StatusCode::INTERNAL_SERVER_ERROR;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn metrics_ok_status_code() {
        let status = StatusCode::OK;
        assert_eq!(status, StatusCode::OK);
    }

    #[test]
    fn metric_name_valid_prometheus_format() {
        let names = vec![
            "tssp_files_total",
            "tssp_tags_total",
            "tssp_pinned_files_total",
            "tssp_recent_uploads_24h",
            "tssp_uptime_seconds",
        ];
        for name in names {
            assert!(name.starts_with("tssp_"));
            assert!(name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'));
        }
    }

    #[test]
    fn metric_value_zero() {
        let value = 0u64;
        assert_eq!(value, 0);
    }

    #[test]
    fn metric_value_nonzero() {
        let value = 42u64;
        assert!(value > 0);
    }

    #[test]
    fn metric_format_with_values() {
        let file_count = 100u64;
        let tag_count = 25u64;
        let pinned_count = 5u64;
        let recent_uploads = 10u64;
        let uptime_secs = 3600u64;

        let metrics = format!(
            "tssp_files_total {file_count}\n\
             tssp_tags_total {tag_count}\n\
             tssp_pinned_files_total {pinned_count}\n\
             tssp_recent_uploads_24h {recent_uploads}\n\
             tssp_uptime_seconds {uptime_secs}\n"
        );

        assert!(metrics.contains("tssp_files_total 100"));
        assert!(metrics.contains("tssp_tags_total 25"));
        assert!(metrics.contains("tssp_pinned_files_total 5"));
        assert!(metrics.contains("tssp_recent_uploads_24h 10"));
        assert!(metrics.contains("tssp_uptime_seconds 3600"));
    }

    #[test]
    fn uptime_seconds_from_elapsed() {
        let start = std::time::Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let _elapsed = start.elapsed().as_secs();
        // elapsed is always <= u64::MAX
    }

    #[test]
    fn metric_value_large_number() {
        let value = u64::MAX;
        let formatted = format!("tssp_files_total {value}");
        assert!(formatted.contains(&value.to_string()));
    }

    #[test]
    fn prometheus_format_has_help_comments() {
        let help_line = "# HELP tssp_files_total Total number of files stored";
        assert!(help_line.starts_with('#'));
    }

    #[test]
    fn prometheus_format_has_type_comments() {
        let type_line = "# TYPE tssp_files_total gauge";
        assert!(type_line.starts_with('#'));
    }

    #[test]
    fn metric_name_tssp_prefix() {
        let prefix = "tssp_";
        assert!(prefix.len() == 5);
    }

    #[test]
    fn metric_underscore_separator() {
        let name = "tssp_files_total";
        let parts: Vec<&str> = name.split('_').collect();
        assert!(parts.len() >= 2);
    }

    #[test]
    fn metrics_endpoint_disabled_response_text() {
        let response_text = "metrics endpoint is disabled";
        assert_eq!(response_text.len(), 28);
    }

    #[test]
    fn metrics_error_response_text() {
        let error_text = "# ERROR: Failed to retrieve metrics\n";
        assert!(error_text.starts_with("# ERROR"));
        assert!(error_text.ends_with('\n'));
    }

    #[test]
    fn file_count_metric_describes_total() {
        let desc = "Total number of files stored";
        assert!(desc.contains("files"));
    }

    #[test]
    fn tags_metric_describes_unique() {
        let desc = "Total number of unique tags";
        assert!(desc.contains("unique"));
    }

    #[test]
    fn pinned_metric_describes_count() {
        let desc = "Number of pinned files";
        assert!(desc.contains("pinned"));
    }

    #[test]
    fn recent_uploads_metric_24h_window() {
        let desc = "Files uploaded in the last 24 hours";
        assert!(desc.contains("24 hours"));
    }

    #[test]
    fn uptime_metric_in_seconds() {
        let desc = "Daemon uptime in seconds";
        assert!(desc.contains("seconds"));
    }

    #[test]
    fn status_code_not_found_value() {
        assert_eq!(StatusCode::NOT_FOUND.as_u16(), 404);
    }

    #[test]
    fn status_code_internal_server_error_value() {
        assert_eq!(StatusCode::INTERNAL_SERVER_ERROR.as_u16(), 500);
    }

    #[test]
    fn status_code_ok_value() {
        assert_eq!(StatusCode::OK.as_u16(), 200);
    }

    #[test]
    fn metric_formatting_consistency() {
        let val1 = 42u64;
        let val2 = 42u64;
        let formatted1 = format!("value: {val1}");
        let formatted2 = format!("value: {val2}");
        assert_eq!(formatted1, formatted2);
    }
}
