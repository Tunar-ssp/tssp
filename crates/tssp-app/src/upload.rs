//! Upload use case orchestration.

use std::io::Read;

use thiserror::Error;
use tssp_domain::{DomainError, FileName, FileRecord, MimeType, Tag};
use tssp_ports::{
    BlobStore, BlobStoreError, Clock, FileRepository, IdGenerationError, IdGenerator,
    NewFileRecord, RepositoryError,
};

/// Coordinates streaming blob storage and metadata insertion for one upload.
pub struct UploadService<B, R, I, C> {
    blob_store: B,
    repository: R,
    id_generator: I,
    clock: C,
}

impl<B, R, I, C> UploadService<B, R, I, C> {
    /// Creates an upload service from explicit infrastructure ports.
    #[must_use]
    pub const fn new(blob_store: B, repository: R, id_generator: I, clock: C) -> Self {
        Self {
            blob_store,
            repository,
            id_generator,
            clock,
        }
    }
}

impl<B, R, I, C> UploadService<B, R, I, C>
where
    B: BlobStore,
    R: FileRepository,
    I: IdGenerator,
    C: Clock,
{
    /// Streams an upload into storage, creates metadata, and cleans up on failure.
    ///
    /// # Errors
    ///
    /// Returns [`UploadError`] when request metadata is invalid, id generation
    /// fails, blob storage fails, or metadata commit fails.
    pub fn upload(&self, request: &mut UploadRequest<'_>) -> Result<UploadResult, UploadError> {
        let name = FileName::new(request.filename)?;
        let mime_type = request
            .mime_type
            .map(MimeType::new)
            .transpose()?
            .unwrap_or_else(MimeType::octet_stream);
        let tags = normalize_tags(request.tags)?;
        let blob = self.blob_store.put_stream(request.source)?;
        if blob.deduplicated {
            match self
                .repository
                .find_file_by_content_hash(&blob.content_hash)
            {
                Ok(Some(record)) => {
                    return Ok(UploadResult {
                        record,
                        deduplicated: true,
                    });
                }
                Ok(None) => {}
                Err(error) => return Err(UploadError::DedupLookup(error)),
            }
        }

        let new_file = NewFileRecord {
            id: self.id_generator.new_file_id()?,
            name,
            size: blob.size,
            content_hash: blob.content_hash,
            mime_type,
            storage_handle: blob.handle.clone(),
            uploaded_at: self.clock.now(),
            tags,
            pinned_at: request.pinned_at,
            folder_path: request.folder_path.to_owned(),
            owner_id: request.owner_id.clone(),
            visibility: request.visibility,
            public_token: request.public_token.clone(),
        };

        match self.repository.insert_file(new_file) {
            Ok(record) => Ok(UploadResult {
                record,
                deduplicated: blob.deduplicated,
            }),
            Err(error) => {
                let cleanup = if blob.deduplicated {
                    None
                } else {
                    self.blob_store.cleanup_unreferenced(&blob.handle).err()
                };
                Err(UploadError::commit_failed(error, cleanup))
            }
        }
    }
}

/// Input for a single uploaded file.
pub struct UploadRequest<'a> {
    /// User-facing filename supplied by the client.
    pub filename: &'a str,
    /// Optional MIME type supplied or detected by the delivery layer.
    pub mime_type: Option<&'a str>,
    /// Initial tags supplied by the client.
    pub tags: &'a [&'a str],
    /// Optional initial pin order.
    pub pinned_at: Option<u32>,
    /// Virtual folder path within the bucket.
    pub folder_path: &'a str,
    /// Owning user for the new file.
    pub owner_id: Option<tssp_domain::UserId>,
    /// Initial visibility.
    pub visibility: tssp_domain::Visibility,
    /// Public link token when visibility is public.
    pub public_token: Option<String>,
    /// Streaming content source.
    pub source: &'a mut dyn Read,
}

/// Successful upload result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadResult {
    /// Created logical file record.
    pub record: FileRecord,
    /// True when blob bytes were already present.
    pub deduplicated: bool,
}

/// Upload use-case failure.
#[derive(Debug, Error)]
pub enum UploadError {
    /// Invalid request metadata.
    #[error(transparent)]
    InvalidRequest(#[from] DomainError),

    /// File id generation failed.
    #[error(transparent)]
    IdGeneration(#[from] IdGenerationError),

    /// Blob storage failed before metadata commit.
    #[error(transparent)]
    BlobStore(#[from] BlobStoreError),

    /// Metadata lookup for an existing deduplicated blob failed.
    #[error("metadata deduplication lookup failed")]
    DedupLookup(RepositoryError),

    /// Metadata commit failed after blob storage succeeded.
    #[error("metadata commit failed after blob write")]
    CommitFailed {
        /// Repository error that caused the failed commit.
        repository: RepositoryError,
        /// Cleanup error, if cleanup also failed.
        cleanup: Option<BlobStoreError>,
    },
}

impl UploadError {
    fn commit_failed(repository: RepositoryError, cleanup: Option<BlobStoreError>) -> Self {
        Self::CommitFailed {
            repository,
            cleanup,
        }
    }
}

fn normalize_tags(tags: &[&str]) -> Result<Vec<Tag>, DomainError> {
    let mut normalized = Vec::with_capacity(tags.len());
    for tag in tags {
        let parsed = Tag::new(tag)?;
        if !normalized
            .iter()
            .any(|existing: &Tag| existing.key() == parsed.key())
        {
            normalized.push(parsed);
        }
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests;
