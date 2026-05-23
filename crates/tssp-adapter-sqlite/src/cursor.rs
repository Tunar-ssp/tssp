//! Cursor encoding and decoding for paginated file list queries.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use rusqlite::types::Value;
use tssp_domain::{Cursor, FileId, FileRecord};
use tssp_ports::{ListSort, RepositoryError};

pub(crate) struct CursorFilter {
    pub(crate) clause: String,
    pub(crate) parameters: Vec<Value>,
}

pub(crate) fn cursor_filter(
    sort: ListSort,
    cursor: Option<&Cursor>,
) -> Result<Option<CursorFilter>, RepositoryError> {
    let Some(cursor) = cursor else {
        return Ok(None);
    };

    let (prefix, primary, id) = split_cursor(cursor)?;
    if prefix != cursor_prefix(sort) {
        return Err(invalid_cursor(
            "cursor was created for a different sort order",
        ));
    }

    let filter = match sort {
        ListSort::UploadedDesc => CursorFilter {
            clause: "(f.uploaded_at < ? OR (f.uploaded_at = ? AND f.id < ?))".to_owned(),
            parameters: vec![
                Value::from(parse_cursor_i64(primary, "uploaded_at")?),
                Value::from(parse_cursor_i64(primary, "uploaded_at")?),
                Value::from(id),
            ],
        },
        ListSort::UploadedAsc => CursorFilter {
            clause: "(f.uploaded_at > ? OR (f.uploaded_at = ? AND f.id > ?))".to_owned(),
            parameters: vec![
                Value::from(parse_cursor_i64(primary, "uploaded_at")?),
                Value::from(parse_cursor_i64(primary, "uploaded_at")?),
                Value::from(id),
            ],
        },
        ListSort::NameAsc => CursorFilter {
            clause: "(f.name > ? OR (f.name = ? AND f.id > ?))".to_owned(),
            parameters: vec![
                Value::from(decode_cursor_name(primary)?),
                Value::from(decode_cursor_name(primary)?),
                Value::from(id),
            ],
        },
        ListSort::NameDesc => CursorFilter {
            clause: "(f.name < ? OR (f.name = ? AND f.id < ?))".to_owned(),
            parameters: vec![
                Value::from(decode_cursor_name(primary)?),
                Value::from(decode_cursor_name(primary)?),
                Value::from(id),
            ],
        },
        ListSort::SizeDesc => CursorFilter {
            clause: "(f.size_bytes < ? OR (f.size_bytes = ? AND f.id < ?))".to_owned(),
            parameters: vec![
                Value::from(parse_cursor_size(primary)?),
                Value::from(parse_cursor_size(primary)?),
                Value::from(id),
            ],
        },
        ListSort::SizeAsc => CursorFilter {
            clause: "(f.size_bytes > ? OR (f.size_bytes = ? AND f.id > ?))".to_owned(),
            parameters: vec![
                Value::from(parse_cursor_size(primary)?),
                Value::from(parse_cursor_size(primary)?),
                Value::from(id),
            ],
        },
    };

    Ok(Some(filter))
}

pub(crate) fn cursor_prefix(sort: ListSort) -> &'static str {
    match sort {
        ListSort::UploadedDesc => "ud",
        ListSort::UploadedAsc => "ua",
        ListSort::NameAsc => "na",
        ListSort::NameDesc => "nd",
        ListSort::SizeDesc => "sd",
        ListSort::SizeAsc => "sa",
    }
}

pub(crate) fn encode_cursor(
    sort: ListSort,
    record: &FileRecord,
) -> Result<Cursor, RepositoryError> {
    let primary = match sort {
        ListSort::UploadedDesc | ListSort::UploadedAsc => record.uploaded_at.seconds().to_string(),
        ListSort::NameAsc | ListSort::NameDesc => {
            URL_SAFE_NO_PAD.encode(record.name.original().as_bytes())
        }
        ListSort::SizeDesc | ListSort::SizeAsc => record.size.bytes().to_string(),
    };
    Cursor::new(format!(
        "{}.{}.{}",
        cursor_prefix(sort),
        primary,
        record.id.as_str()
    ))
    .map_err(|error| RepositoryError::OperationFailed {
        message: format!("could not encode pagination cursor: {error}"),
    })
}

pub(crate) fn list_limit_plus_one(limit: u64) -> Result<i64, RepositoryError> {
    let fetch_limit = limit
        .checked_add(1)
        .ok_or_else(|| RepositoryError::OperationFailed {
            message: "list limit overflowed".to_owned(),
        })?;
    i64::try_from(fetch_limit).map_err(|error| RepositoryError::OperationFailed {
        message: format!("list limit does not fit sqlite integer: {error}"),
    })
}

pub(crate) fn order_by_clause(sort: ListSort) -> &'static str {
    match sort {
        ListSort::UploadedDesc => "f.uploaded_at DESC, f.id DESC",
        ListSort::UploadedAsc => "f.uploaded_at ASC, f.id ASC",
        ListSort::NameAsc => "f.name ASC, f.id ASC",
        ListSort::NameDesc => "f.name DESC, f.id DESC",
        ListSort::SizeDesc => "f.size_bytes DESC, f.id DESC",
        ListSort::SizeAsc => "f.size_bytes ASC, f.id ASC",
    }
}

fn split_cursor(cursor: &Cursor) -> Result<(&str, &str, String), RepositoryError> {
    let mut parts = cursor.as_str().split('.');
    let prefix = parts
        .next()
        .ok_or_else(|| invalid_cursor("missing cursor prefix"))?;
    let primary = parts
        .next()
        .ok_or_else(|| invalid_cursor("missing cursor value"))?;
    let id = parts
        .next()
        .ok_or_else(|| invalid_cursor("missing cursor id"))?;
    if parts.next().is_some() {
        return Err(invalid_cursor("cursor has too many parts"));
    }

    let parsed_id = FileId::new(id).map_err(|error| invalid_cursor(error.to_string()))?;
    Ok((prefix, primary, parsed_id.as_str().to_owned()))
}

fn parse_cursor_i64(value: &str, field: &str) -> Result<i64, RepositoryError> {
    value
        .parse::<i64>()
        .map_err(|error| invalid_cursor(format!("invalid {field} value: {error}")))
}

fn parse_cursor_size(value: &str) -> Result<i64, RepositoryError> {
    let size = value
        .parse::<u64>()
        .map_err(|error| invalid_cursor(format!("invalid size value: {error}")))?;
    i64::try_from(size)
        .map_err(|error| invalid_cursor(format!("size value does not fit sqlite integer: {error}")))
}

fn decode_cursor_name(value: &str) -> Result<String, RepositoryError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|error| invalid_cursor(format!("invalid name value: {error}")))?;
    String::from_utf8(bytes)
        .map_err(|error| invalid_cursor(format!("name value is not valid utf-8: {error}")))
}

pub(crate) fn invalid_cursor(message: impl Into<String>) -> RepositoryError {
    RepositoryError::OperationFailed {
        message: format!("invalid cursor: {}", message.into()),
    }
}
