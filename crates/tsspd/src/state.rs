//! Shared HTTP application state and builder methods.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use tssp_ports::BlobReader;

use crate::auth::AuthService;
use crate::chunked_upload::UploadSessionManager;
use crate::content::StaticBlobReader;
use crate::delete::{FileDeleteProvider, StaticFileDeleteProvider};
use crate::notes::{NoteProvider, StaticNoteProvider};
use crate::pins::{FilePinProvider, StaticFilePinProvider};
use crate::rate_limit::RateLimiter;
use crate::search::{FileSearchProvider, StaticFileSearchProvider};
use crate::sessions::{SessionProvider, StaticSessionProvider};
use crate::settings::DaemonSettings;
use crate::status::{MetadataStatsProvider, StaticMetadataStatsProvider};
use crate::tags::{FileTagProvider, StaticFileTagProvider};
use crate::upload::{FileUploadProvider, StaticFileUploadProvider};
use crate::urls::PublicUrlBuilder;
use crate::workspaces;

/// Shared HTTP state.
#[derive(Clone)]
pub struct HttpState {
    pub(crate) started_at: Instant,
    pub(crate) upload_temp_dir: PathBuf,
    pub(crate) storage_mutation_lock: Arc<tokio::sync::Mutex<()>>,
    pub(crate) auth: AuthService,
    pub(crate) rate_limiter: RateLimiter,
    pub(crate) workspaces: Option<Arc<workspaces::WorkspaceStore>>,
    pub(crate) settings: Arc<DaemonSettings>,
    pub(crate) public_urls: PublicUrlBuilder,
    /// Number of files whose content blob is missing on disk.
    pub corrupt_file_count: Arc<std::sync::atomic::AtomicU64>,
    pub(crate) upload_session_manager: Arc<UploadSessionManager>,
    pub(crate) stats_provider: Arc<dyn MetadataStatsProvider>,
    pub(crate) upload_provider: Arc<dyn FileUploadProvider>,
    pub(crate) delete_provider: Arc<dyn FileDeleteProvider>,
    pub(crate) tag_provider: Arc<dyn FileTagProvider>,
    pub(crate) pin_provider: Arc<dyn FilePinProvider>,
    pub(crate) session_provider: Arc<dyn SessionProvider>,
    pub(crate) note_provider: Arc<dyn NoteProvider>,
    pub(crate) search_provider: Arc<dyn FileSearchProvider>,
    pub(crate) blob_reader: Arc<dyn BlobReader + Send + Sync>,
}

impl HttpState {
    /// Creates a new HTTP state with static fallback providers.
    #[must_use]
    pub fn new(
        started_at: Instant,
        upload_temp_dir: PathBuf,
        settings: Arc<DaemonSettings>,
        public_urls: PublicUrlBuilder,
        corrupt_file_count: u64,
    ) -> Self {
        Self {
            started_at,
            upload_temp_dir,
            storage_mutation_lock: Arc::new(tokio::sync::Mutex::new(())),
            auth: AuthService::disabled(),
            rate_limiter: RateLimiter::new(),
            workspaces: None,
            settings,
            public_urls,
            corrupt_file_count: Arc::new(std::sync::atomic::AtomicU64::new(corrupt_file_count)),
            upload_session_manager: Arc::new(UploadSessionManager::new()),
            stats_provider: Arc::new(StaticMetadataStatsProvider),
            upload_provider: Arc::new(StaticFileUploadProvider),
            delete_provider: Arc::new(StaticFileDeleteProvider),
            tag_provider: Arc::new(StaticFileTagProvider),
            pin_provider: Arc::new(StaticFilePinProvider),
            session_provider: Arc::new(StaticSessionProvider),
            note_provider: Arc::new(StaticNoteProvider),
            search_provider: Arc::new(StaticFileSearchProvider),
            blob_reader: Arc::new(StaticBlobReader),
        }
    }

    /// Attaches the workspace store.
    #[must_use]
    pub fn with_workspaces(mut self, store: Arc<workspaces::WorkspaceStore>) -> Self {
        self.workspaces = Some(store);
        self
    }

    /// Attaches the metadata stats provider.
    #[must_use]
    pub fn with_stats_provider(mut self, provider: Arc<dyn MetadataStatsProvider>) -> Self {
        self.stats_provider = provider;
        self
    }

    /// Attaches the upload provider.
    #[must_use]
    pub fn with_upload_provider(mut self, provider: Arc<dyn FileUploadProvider>) -> Self {
        self.upload_provider = provider;
        self
    }

    /// Attaches the delete provider.
    #[must_use]
    pub fn with_delete_provider(mut self, provider: Arc<dyn FileDeleteProvider>) -> Self {
        self.delete_provider = provider;
        self
    }

    /// Attaches the tag provider.
    #[must_use]
    pub fn with_tag_provider(mut self, provider: Arc<dyn FileTagProvider>) -> Self {
        self.tag_provider = provider;
        self
    }

    /// Attaches the pin provider.
    #[must_use]
    pub fn with_pin_provider(mut self, provider: Arc<dyn FilePinProvider>) -> Self {
        self.pin_provider = provider;
        self
    }

    /// Attaches the session provider.
    #[must_use]
    pub fn with_session_provider(mut self, provider: Arc<dyn SessionProvider>) -> Self {
        self.session_provider = provider;
        self
    }

    /// Attaches the note provider.
    #[must_use]
    pub fn with_note_provider(mut self, provider: Arc<dyn NoteProvider>) -> Self {
        self.note_provider = provider;
        self
    }

    /// Attaches the search provider.
    #[must_use]
    pub fn with_search_provider(mut self, provider: Arc<dyn FileSearchProvider>) -> Self {
        self.search_provider = provider;
        self
    }

    /// Attaches the blob reader.
    #[must_use]
    pub fn with_blob_reader(mut self, reader: Arc<dyn BlobReader + Send + Sync>) -> Self {
        self.blob_reader = reader;
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

    #[cfg(test)]
    /// Builds test HTTP state with default daemon settings.
    #[must_use]
    pub fn test_http_state(upload_temp_dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&upload_temp_dir);
        let settings = Arc::new(DaemonSettings::default());
        Self::new(
            Instant::now(),
            upload_temp_dir,
            settings.clone(),
            PublicUrlBuilder::from_settings(&settings),
            0,
        )
    }

    #[cfg(test)]
    /// Builds test HTTP state with explicit daemon settings.
    #[must_use]
    pub fn test_http_state_with_settings(
        upload_temp_dir: PathBuf,
        settings: DaemonSettings,
    ) -> Self {
        let _ = std::fs::create_dir_all(&upload_temp_dir);
        let settings = Arc::new(settings);
        Self::new(
            Instant::now(),
            upload_temp_dir,
            settings.clone(),
            PublicUrlBuilder::from_settings(&settings),
            0,
        )
    }
}
