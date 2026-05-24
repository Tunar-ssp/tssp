//! Shared HTTP application state and builder methods.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use tssp_app::{
    DeleteFileService, NoteService, PinService, SessionService, TagService, UploadService,
};
use tssp_ports::{BlobReader, BlobStore, Clock, FileRepository, IdGenerator, SessionRepository};

use crate::auth::AuthService;
use crate::settings::DaemonSettings;
use crate::urls::PublicUrlBuilder;
use crate::workspaces;

/// Shared HTTP state.
#[derive(Clone)]
pub struct HttpState<R> {
    pub(crate) started_at: Instant,
    pub(crate) upload_temp_dir: PathBuf,
    pub(crate) storage_mutation_lock: Arc<tokio::sync::Mutex<()>>,
    pub(crate) auth: AuthService,
    pub(crate) workspaces: Option<Arc<workspaces::WorkspaceStore>>,
    pub(crate) settings: Arc<DaemonSettings>,
    pub(crate) public_urls: PublicUrlBuilder,
    pub(crate) corrupt_file_count: u64,

    pub(crate) tag_service: Arc<TagService<R>>,
    pub(crate) note_service: Arc<NoteService<R, tssp_adapter_system::SystemClock, tssp_adapter_system::UuidV7FileIdGenerator>>,
    pub(crate) pin_service: Arc<PinService<R>>,
    pub(crate) delete_service: Arc<DeleteFileService<tssp_adapter_fs::FilesystemBlobStore, R>>,
    pub(crate) upload_service: Arc<UploadService<tssp_adapter_fs::FilesystemBlobStore, R, tssp_adapter_system::UuidV7FileIdGenerator, tssp_adapter_system::SystemClock>>,
    pub(crate) session_service: Arc<SessionService<tssp_adapter_sqlite::SqliteSessionRepository>>,
    pub(crate) blob_reader: Arc<tssp_adapter_fs::FilesystemBlobStore>,
}

impl<R> HttpState<R>
where
    R: FileRepository + Send + Sync + 'static,
{
    /// Creates a new HTTP state.
    #[must_use]
    pub fn new(
        started_at: Instant,
        upload_temp_dir: PathBuf,
        settings: Arc<DaemonSettings>,
        public_urls: PublicUrlBuilder,
        corrupt_file_count: u64,
        tag_service: Arc<TagService<R>>,
        note_service: Arc<NoteService<R, tssp_adapter_system::SystemClock, tssp_adapter_system::UuidV7FileIdGenerator>>,
        pin_service: Arc<PinService<R>>,
        delete_service: Arc<DeleteFileService<tssp_adapter_fs::FilesystemBlobStore, R>>,
        upload_service: Arc<UploadService<tssp_adapter_fs::FilesystemBlobStore, R, tssp_adapter_system::UuidV7FileIdGenerator, tssp_adapter_system::SystemClock>>,
        session_service: Arc<SessionService<tssp_adapter_sqlite::SqliteSessionRepository>>,
        blob_reader: Arc<tssp_adapter_fs::FilesystemBlobStore>,
    ) -> Self {
        Self {
            started_at,
            upload_temp_dir,
            storage_mutation_lock: Arc::new(tokio::sync::Mutex::new(())),
            auth: AuthService::disabled(),
            workspaces: None,
            settings,
            public_urls,
            corrupt_file_count,
            tag_service,
            note_service,
            pin_service,
            delete_service,
            upload_service,
            session_service,
            blob_reader,
        }
    }

    /// Attaches the workspace store.
    #[must_use]
    pub fn with_workspaces(mut self, store: Arc<workspaces::WorkspaceStore>) -> Self {
        self.workspaces = Some(store);
        self
    }

    /// Effective daemon settings.
    #[must_use]
    pub fn settings(&self) -> &DaemonSettings {
        &self.settings
    }

    /// URL builder for public links.
    #[must_use]
    pub fn public_urls(&self) -> &PublicUrlBuilder {
        &self.public_urls
    }

    /// Sets the authentication service.
    #[must_use]
    pub fn with_auth(mut self, auth: AuthService) -> Self {
        self.auth = auth;
        self
    }
}
