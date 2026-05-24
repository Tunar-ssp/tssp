//! File content download delivery.

use std::fs::File;
use std::io::{Seek, SeekFrom};

use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::header::{
    ACCEPT_RANGES, CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, ETAG,
    LAST_MODIFIED, RANGE,
};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use tokio::io::AsyncReadExt;
use tokio_util::io::ReaderStream;
use tssp_domain::{FileId, FileRecord, StorageHandle};
use tssp_ports::{BlobReadError, BlobReader};

use crate::{ErrorBody, ErrorResponse, HttpState};

/// Read-side blob provider used by HTTP downloads.
#[derive(Debug)]
pub(crate) struct StaticBlobReader;

impl BlobReader for StaticBlobReader {
    fn open_blob(&self, _handle: &StorageHandle) -> Result<File, BlobReadError> {
        Err(BlobReadError::ReadFailed {
            message: "blob reader is not configured".to_owned(),
        })
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct ContentQuery {
    disposition: Option<String>,
}

pub(crate) async fn get_file_content(
    State(state): State<HttpState>,
    Path(id): Path<String>,
    Query(query): Query<ContentQuery>,
    headers: HeaderMap,
) -> Response {
    let file_id = match FileId::new(id) {
        Ok(value) => value,
        Err(error) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_file_id",
                error.to_string(),
            )
        }
    };
    let disposition = match disposition_mode(query.disposition.as_deref()) {
        Ok(value) => value,
        Err(message) => return error_response(StatusCode::BAD_REQUEST, "invalid_request", message),
    };
    let record = match find_file_record(state.clone(), file_id).await {
        Ok(Some(value)) => value,
        Ok(None) => {
            return error_response(
                StatusCode::NOT_FOUND,
                "file_not_found",
                "file was not found".to_owned(),
            );
        }
        Err(message) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "metadata_unavailable",
                message,
            );
        }
    };
    let Ok(byte_range) = parse_range_header(headers.get(RANGE), record.size.bytes()) else {
        return range_not_satisfiable(record.size.bytes());
    };

    let blob = match open_blob(state, record.storage_handle.clone()).await {
        Ok(value) => value,
        Err(BlobReadError::Missing { .. }) => {
            return error_response(
                StatusCode::GONE,
                "blob_missing",
                "file metadata exists but stored bytes are missing".to_owned(),
            );
        }
        Err(error) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "blob_unavailable",
                error.to_string(),
            )
        }
    };

    stream_blob_response(&record, blob, byte_range, disposition)
}

pub(crate) async fn find_file_record(
    state: HttpState,
    file_id: FileId,
) -> Result<Option<FileRecord>, String> {
    let metadata = state.stats_provider.clone();
    tokio::task::spawn_blocking(move || metadata.find_file(&file_id))
        .await
        .map_err(|error| format!("metadata worker failed: {error}"))?
}

pub(crate) async fn open_blob(
    state: HttpState,
    handle: StorageHandle,
) -> Result<File, BlobReadError> {
    let reader = state.blob_reader.clone();
    tokio::task::spawn_blocking(move || reader.open_blob(&handle))
        .await
        .map_err(|error| BlobReadError::ReadFailed {
            message: format!("blob worker failed: {error}"),
        })?
}

pub(crate) fn stream_blob_response(
    record: &FileRecord,
    mut blob: File,
    range: Option<ByteRange>,
    disposition: DispositionMode,
) -> Response {
    let full_size = record.size.bytes();
    let is_partial = range.is_some();
    let selected = range.unwrap_or(ByteRange {
        start: 0,
        end_inclusive: full_size.saturating_sub(1),
    });
    let selected_len = if full_size == 0 { 0 } else { selected.len() };
    if let Err(error) = blob.seek(SeekFrom::Start(selected.start)) {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "blob_unavailable",
            format!("could not seek blob: {error}"),
        );
    }

    let tokio_file = tokio::fs::File::from_std(blob);
    let body = Body::from_stream(ReaderStream::with_capacity(
        tokio_file.take(selected_len),
        64 * 1024,
    ));

    let mut response = Response::new(body);
    *response.status_mut() = if is_partial {
        StatusCode::PARTIAL_CONTENT
    } else {
        StatusCode::OK
    };
    set_download_headers(
        response.headers_mut(),
        record,
        selected,
        selected_len,
        full_size,
        is_partial,
        disposition,
    );
    response
}

fn set_download_headers(
    headers: &mut HeaderMap,
    record: &FileRecord,
    selected: ByteRange,
    selected_len: u64,
    full_size: u64,
    is_partial: bool,
    disposition: DispositionMode,
) {
    insert_header(headers, CONTENT_TYPE, record.mime_type.as_str());
    insert_header(headers, CONTENT_LENGTH, selected_len.to_string());
    insert_header(headers, ACCEPT_RANGES, "bytes");
    insert_header(
        headers,
        ETAG,
        format!("\"{}\"", record.content_hash.as_str()),
    );
    insert_header(headers, LAST_MODIFIED, last_modified(record));
    insert_header(
        headers,
        CONTENT_DISPOSITION,
        content_disposition(disposition, record.name.original()),
    );
    if is_partial {
        insert_header(
            headers,
            CONTENT_RANGE,
            format!(
                "bytes {}-{}/{}",
                selected.start, selected.end_inclusive, full_size
            ),
        );
    }
}

fn insert_header(
    headers: &mut HeaderMap,
    name: axum::http::header::HeaderName,
    value: impl AsRef<str>,
) {
    if let Ok(value) = HeaderValue::from_str(value.as_ref()) {
        headers.insert(name, value);
    }
}

fn last_modified(record: &FileRecord) -> String {
    http_date(record.uploaded_at.seconds_u64())
}

fn http_date(unix_seconds: u64) -> String {
    const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let days = unix_seconds / 86_400;
    let seconds_of_day = unix_seconds % 86_400;
    let (year, month, day) = civil_from_unix_days(i64::try_from(days).unwrap_or(i64::MAX));
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    let weekday = WEEKDAYS[usize::try_from((days + 4) % 7).unwrap_or(0)];
    let month_name = MONTHS[usize::try_from(month.saturating_sub(1)).unwrap_or(0)];
    format!("{weekday}, {day:02} {month_name} {year:04} {hour:02}:{minute:02}:{second:02} GMT")
}

fn civil_from_unix_days(days: i64) -> (i64, u32, u32) {
    let shifted_days = days + 719_468;
    let era = if shifted_days >= 0 {
        shifted_days
    } else {
        shifted_days - 146_096
    } / 146_097;
    let day_of_era = shifted_days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    (
        year,
        u32::try_from(month).unwrap_or(1),
        u32::try_from(day).unwrap_or(1),
    )
}

fn content_disposition(mode: DispositionMode, filename: &str) -> String {
    format!(
        "{}; filename=\"{}\"",
        mode.as_str(),
        header_quoted(filename)
    )
}

fn header_quoted(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            '"' => "\\\"".to_owned(),
            '\\' => "\\\\".to_owned(),
            '\r' | '\n' => "_".to_owned(),
            control if control.is_control() => "_".to_owned(),
            other => other.to_string(),
        })
        .collect()
}

fn disposition_mode(value: Option<&str>) -> Result<DispositionMode, String> {
    match value.unwrap_or("attachment") {
        "attachment" => Ok(DispositionMode::Attachment),
        "inline" => Ok(DispositionMode::Inline),
        other => Err(format!(
            "disposition must be attachment or inline, got {other}"
        )),
    }
}

fn parse_range_header(
    value: Option<&HeaderValue>,
    full_size: u64,
) -> Result<Option<ByteRange>, ()> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.to_str().map_err(|_error| ())?;
    let Some(range) = value.strip_prefix("bytes=") else {
        return Err(());
    };
    parse_byte_range(range, full_size).map(Some)
}

fn parse_byte_range(value: &str, full_size: u64) -> Result<ByteRange, ()> {
    let (start, end) = value.split_once('-').ok_or(())?;
    if start.is_empty() {
        return parse_suffix_range(end, full_size);
    }
    let start = start.parse::<u64>().map_err(|_error| ())?;
    let end_inclusive = if end.is_empty() {
        full_size.checked_sub(1).ok_or(())?
    } else {
        end.parse::<u64>().map_err(|_error| ())?
    };
    byte_range(start, end_inclusive, full_size)
}

fn parse_suffix_range(value: &str, full_size: u64) -> Result<ByteRange, ()> {
    let suffix_len = value.parse::<u64>().map_err(|_error| ())?;
    if suffix_len == 0 || full_size == 0 {
        return Err(());
    }
    let start = full_size.saturating_sub(suffix_len);
    byte_range(start, full_size - 1, full_size)
}

fn byte_range(start: u64, end_inclusive: u64, full_size: u64) -> Result<ByteRange, ()> {
    if start >= full_size || end_inclusive < start {
        return Err(());
    }
    Ok(ByteRange {
        start,
        end_inclusive: end_inclusive.min(full_size - 1),
    })
}

fn range_not_satisfiable(full_size: u64) -> Response {
    let mut response = error_response(
        StatusCode::RANGE_NOT_SATISFIABLE,
        "invalid_range",
        "range is not satisfiable".to_owned(),
    );
    insert_header(
        response.headers_mut(),
        CONTENT_RANGE,
        format!("bytes */{full_size}"),
    );
    response
}

fn error_response(status: StatusCode, code: &'static str, message: String) -> Response {
    (
        status,
        Json(ErrorResponse {
            error: ErrorBody { code, message },
        }),
    )
        .into_response()
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct ByteRange {
    start: u64,
    end_inclusive: u64,
}

impl ByteRange {
    const fn len(self) -> u64 {
        self.end_inclusive - self.start + 1
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum DispositionMode {
    Attachment,
    Inline,
}

impl DispositionMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Attachment => "attachment",
            Self::Inline => "inline",
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::{
        content_disposition, disposition_mode, parse_range_header, ByteRange, DispositionMode,
    };

    #[test]
    fn parses_standard_byte_ranges() {
        let header = HeaderValue::from_static("bytes=2-5");

        assert_eq!(
            parse_range_header(Some(&header), 10),
            Ok(Some(ByteRange {
                start: 2,
                end_inclusive: 5
            }))
        );
    }

    #[test]
    fn parses_open_ended_and_suffix_ranges() {
        let open = HeaderValue::from_static("bytes=7-");
        let suffix = HeaderValue::from_static("bytes=-4");

        assert_eq!(
            parse_range_header(Some(&open), 10),
            Ok(Some(ByteRange {
                start: 7,
                end_inclusive: 9
            }))
        );
        assert_eq!(
            parse_range_header(Some(&suffix), 10),
            Ok(Some(ByteRange {
                start: 6,
                end_inclusive: 9
            }))
        );
    }

    #[test]
    fn rejects_invalid_ranges() {
        let bad_unit = HeaderValue::from_static("items=1-2");
        let reversed = HeaderValue::from_static("bytes=8-2");
        let outside = HeaderValue::from_static("bytes=10-11");

        assert_eq!(parse_range_header(Some(&bad_unit), 10), Err(()));
        assert_eq!(parse_range_header(Some(&reversed), 10), Err(()));
        assert_eq!(parse_range_header(Some(&outside), 10), Err(()));
    }

    #[test]
    fn validates_disposition_query() {
        assert_eq!(disposition_mode(None), Ok(DispositionMode::Attachment));
        assert_eq!(
            disposition_mode(Some("inline")),
            Ok(DispositionMode::Inline)
        );
        assert!(disposition_mode(Some("preview")).is_err());
    }

    #[test]
    fn content_disposition_escapes_quoted_filename() {
        assert_eq!(
            content_disposition(DispositionMode::Attachment, "a\"b\\c.txt"),
            "attachment; filename=\"a\\\"b\\\\c.txt\""
        );
    }

    #[test]
    fn http_date_formats_epoch_seconds() {
        assert_eq!(super::http_date(0), "Thu, 01 Jan 1970 00:00:00 GMT");
        assert_eq!(
            super::http_date(1_700_000_000),
            "Tue, 14 Nov 2023 22:13:20 GMT"
        );
    }

    #[test]
    fn parse_range_header_returns_none_for_missing_range() {
        assert_eq!(parse_range_header(None, 100), Ok(None));
    }

    #[test]
    fn parse_range_header_handles_empty_suffix_suffix() {
        let header = HeaderValue::from_static("bytes=-0");
        assert_eq!(parse_range_header(Some(&header), 10), Err(()));
    }

    #[test]
    fn byte_range_wraps_at_file_boundary() {
        assert_eq!(
            parse_range_header(Some(&HeaderValue::from_static("bytes=0-999")), 10),
            Ok(Some(ByteRange {
                start: 0,
                end_inclusive: 9
            }))
        );
    }

    #[test]
    fn content_disposition_escapes_control_chars() {
        assert_eq!(
            content_disposition(DispositionMode::Inline, "file\r\n.txt"),
            "inline; filename=\"file__.txt\""
        );
    }

    #[test]
    fn disposition_mode_defaults_to_attachment() {
        assert_eq!(disposition_mode(None), Ok(DispositionMode::Attachment));
    }

    #[test]
    fn disposition_mode_accepts_inline() {
        assert_eq!(
            disposition_mode(Some("inline")),
            Ok(DispositionMode::Inline)
        );
    }

    #[test]
    fn disposition_mode_rejects_invalid() {
        assert!(disposition_mode(Some("preview")).is_err());
    }
}
