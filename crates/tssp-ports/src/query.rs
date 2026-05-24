//! Query parameter and result types for repository listing operations.

use tssp_domain::{Cursor, FileRecord, NoteRecord, Tag, TagKey, UnixTimestamp, UserId, Visibility};

/// Sort order for file listing queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListSort {
    /// Descending by upload timestamp (newest first). Default.
    #[default]
    UploadedDesc,
    /// Ascending by upload timestamp (oldest first).
    UploadedAsc,
    /// Ascending by filename.
    NameAsc,
    /// Descending by filename.
    NameDesc,
    /// Descending by file size.
    SizeDesc,
    /// Ascending by file size.
    SizeAsc,
}

impl ListSort {
    /// Parses a sort field string as produced by CLI `--sort` flag.
    ///
    /// A leading `-` negates the direction. Unknown fields return `None`.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "uploaded" => Some(Self::UploadedAsc),
            "-uploaded" => Some(Self::UploadedDesc),
            "name" => Some(Self::NameAsc),
            "-name" => Some(Self::NameDesc),
            "size" => Some(Self::SizeAsc),
            "-size" => Some(Self::SizeDesc),
            _ => None,
        }
    }
}

/// Query parameters for filtered and paginated file listing.
#[derive(Debug, Clone)]
pub struct ListQuery {
    /// Maximum files to return. Must be between 1 and 500.
    pub limit: u64,
    /// Required tag keys (AND semantics). Empty means no tag filter.
    pub tags: Vec<TagKey>,
    /// MIME type prefix filter (`"image"` matches `"image/png"`).
    pub mime_prefix: Option<String>,
    /// Optional filename substring filter.
    pub name_substring: Option<String>,
    /// Only return files uploaded at or after this timestamp.
    pub since: Option<UnixTimestamp>,
    /// Only return files uploaded at or before this timestamp.
    pub until: Option<UnixTimestamp>,
    /// When true, only pinned files are returned.
    pub pinned_only: bool,
    /// Sort order for the result.
    pub sort: ListSort,
    /// Opaque cursor from a previous response for the next page.
    pub after_cursor: Option<Cursor>,
    /// When set, only files under this folder prefix (e.g. `photos/`).
    pub folder_prefix: Option<String>,
    /// When set, only files with this visibility.
    pub visibility: Option<Visibility>,
    /// When set, only files owned by this user.
    pub owner_id: Option<UserId>,
}

impl Default for ListQuery {
    fn default() -> Self {
        Self {
            limit: 50,
            tags: Vec::new(),
            mime_prefix: None,
            name_substring: None,
            since: None,
            until: None,
            pinned_only: false,
            sort: ListSort::UploadedDesc,
            after_cursor: None,
            folder_prefix: None,
            visibility: None,
            owner_id: None,
        }
    }
}

/// Paginated file listing result.
#[derive(Debug, Clone)]
pub struct PagedFiles {
    /// File records for the current page.
    pub files: Vec<FileRecord>,
    /// Cursor for the next page, absent when no further results exist.
    pub next_cursor: Option<Cursor>,
}

/// Sort order for note listing queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NoteListSort {
    /// Descending by last update (default).
    #[default]
    UpdatedDesc,
    /// Ascending by last update.
    UpdatedAsc,
    /// Descending by creation time.
    CreatedDesc,
    /// Ascending by creation time.
    CreatedAsc,
    /// Ascending by title.
    TitleAsc,
    /// Descending by title.
    TitleDesc,
}

impl NoteListSort {
    /// Parses a sort field string from CLI `--sort`.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "updated" => Some(Self::UpdatedAsc),
            "-updated" => Some(Self::UpdatedDesc),
            "created" => Some(Self::CreatedAsc),
            "-created" => Some(Self::CreatedDesc),
            "title" => Some(Self::TitleAsc),
            "-title" => Some(Self::TitleDesc),
            _ => None,
        }
    }
}

/// Query parameters for filtered note listing.
#[derive(Debug, Clone)]
pub struct NoteListQuery {
    /// Maximum notes to return (1..=500).
    pub limit: u64,
    /// Required tag keys (AND semantics).
    pub tags: Vec<TagKey>,
    /// Only notes updated at or after this timestamp.
    pub since: Option<UnixTimestamp>,
    /// Only notes updated at or before this timestamp.
    pub until: Option<UnixTimestamp>,
    /// Title substring filter.
    pub title_substring: Option<String>,
    /// When true, only pinned notes are returned.
    pub pinned_only: bool,
    /// Sort order.
    pub sort: NoteListSort,
    /// Cursor from a previous page.
    pub after_cursor: Option<Cursor>,
}

impl Default for NoteListQuery {
    fn default() -> Self {
        Self {
            limit: 50,
            tags: Vec::new(),
            since: None,
            until: None,
            title_substring: None,
            pinned_only: false,
            sort: NoteListSort::UpdatedDesc,
            after_cursor: None,
        }
    }
}

/// Paginated note listing result.
#[derive(Debug, Clone)]
pub struct PagedNotes {
    /// Notes on this page.
    pub notes: Vec<NoteRecord>,
    /// Cursor for the next page.
    pub next_cursor: Option<Cursor>,
}

/// One ranked unified search hit.
#[derive(Debug, Clone)]
pub enum SearchHit {
    /// A matching file.
    File(FileRecord),
    /// A matching note.
    Note(NoteRecord),
}

/// Aggregate metadata counts used by health and status views.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RepositoryStats {
    /// Total logical files.
    pub file_count: u64,
    /// Total notes.
    pub note_count: u64,
    /// Total distinct tags.
    pub tag_count: u64,
    /// Total pinned files.
    pub pinned_count: u64,
    /// Files uploaded at or after the caller-supplied recent cutoff.
    pub recent_upload_count: u64,
    /// Notes created or updated at or after the recent cutoff.
    pub recent_note_count: u64,
}

/// Tag plus the number of logical files that currently use it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagSummary {
    /// Normalized tag display value.
    pub tag: Tag,
    /// Number of committed file records using this tag.
    pub file_count: u64,
}

/// Result of an idempotent tag mutation.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TagMutationOutcome {
    /// Number of tag associations created or removed.
    pub changed_count: u64,
}

/// Result of a pin or unpin operation.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PinOutcome {
    /// True when the file record exists.
    pub existed: bool,
    /// True when the pin state actually changed.
    pub changed: bool,
}
