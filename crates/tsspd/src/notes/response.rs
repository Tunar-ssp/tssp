//! Note API response shapes.

use serde::{Deserialize, Serialize};
use tssp_domain::NoteRecord;

/// JSON note record returned by the API.
#[derive(Debug, Serialize, Deserialize)]
pub struct NoteRecordResponse {
    /// Stable schema version.
    pub schema_version: u8,
    /// Note id.
    pub id: String,
    /// Title.
    pub title: String,
    /// Markdown body.
    pub body: String,
    /// Tags.
    pub tags: Vec<String>,
    /// Creation timestamp (UTC seconds).
    pub created_at: i64,
    /// Last update timestamp (UTC seconds).
    pub updated_at: i64,
    /// Pin position when pinned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned_at: Option<u32>,
    /// Parent note id for page nesting (`None` = top level).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// Optional page icon (emoji or short token).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

impl NoteRecordResponse {
    /// Builds an API record from a domain note.
    #[must_use]
    pub(crate) fn from_record(record: &NoteRecord) -> Self {
        Self {
            schema_version: 1,
            id: record.id.as_str().to_owned(),
            title: record.title.as_str().to_owned(),
            body: record.body.as_str().to_owned(),
            tags: record
                .tags
                .iter()
                .map(|tag| tag.display().to_owned())
                .collect(),
            created_at: record.created_at.seconds(),
            updated_at: record.updated_at.seconds(),
            pinned_at: record.pinned_at,
            parent_id: record.parent_id.clone(),
            icon: record.icon.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct NoteListResponse {
    pub(crate) schema_version: u8,
    pub(crate) notes: Vec<NoteRecordResponse>,
    pub(crate) next_cursor: Option<String>,
}
