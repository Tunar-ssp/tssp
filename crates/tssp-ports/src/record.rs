//! Input and output record types for repository write operations.

use tssp_domain::{
    ContentHash, FileId, FileName, FileRecord, FileSize, MimeType, NoteBody, NoteId, NoteTitle,
    StorageHandle, Tag, UnixTimestamp, UserId, Visibility,
};

/// Metadata needed to create a logical file record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewFileRecord {
    /// New file id.
    pub id: FileId,
    /// User-facing filename.
    pub name: FileName,
    /// Stored byte count.
    pub size: FileSize,
    /// Content hash of bytes.
    pub content_hash: ContentHash,
    /// MIME type.
    pub mime_type: MimeType,
    /// Opaque blob handle.
    pub storage_handle: StorageHandle,
    /// Upload time.
    pub uploaded_at: UnixTimestamp,
    /// Initial tags.
    pub tags: Vec<Tag>,
    /// Initial pin position.
    pub pinned_at: Option<u32>,
    /// Virtual folder path within the bucket.
    pub folder_path: String,
    /// Owning user id.
    pub owner_id: Option<UserId>,
    /// Visibility at creation time.
    pub visibility: Visibility,
    /// Optional public link token when visibility is public.
    pub public_token: Option<String>,
}

/// Result of a metadata delete transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletedFileRecord {
    /// Record that was removed from the metadata index.
    pub record: FileRecord,
    /// Remaining logical records that still reference the same content hash.
    pub remaining_content_references: u64,
}

/// Metadata needed to create a note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewNoteRecord {
    /// New note id.
    pub id: NoteId,
    /// Display title.
    pub title: NoteTitle,
    /// Markdown body.
    pub body: NoteBody,
    /// Creation timestamp.
    pub created_at: UnixTimestamp,
    /// Last update timestamp.
    pub updated_at: UnixTimestamp,
    /// Initial tags.
    pub tags: Vec<Tag>,
    /// Initial pin position.
    pub pinned_at: Option<u32>,
    /// Virtual folder path.
    pub folder_path: String,
    /// Owning user id.
    pub owner_id: Option<UserId>,
}

/// Durable result of writing a blob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobWriteOutcome {
    /// Content hash of the streamed bytes.
    pub content_hash: ContentHash,
    /// Opaque durable storage handle.
    pub handle: StorageHandle,
    /// Number of bytes written.
    pub size: FileSize,
    /// True when existing bytes were reused.
    pub deduplicated: bool,
}
