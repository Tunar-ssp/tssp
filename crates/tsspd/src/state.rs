//! Shared HTTP application state and builder methods.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use tssp_ports::BlobReader;

use crate::auth::AuthService;
use crate::delete::StaticFileDeleteProvider;
use crate::notes::StaticNoteProvider;
use crate::pins::StaticFilePinProvider;
use crate::search::StaticFileSearchProvider;
use crate::sessions::StaticSessionProvider;
use crate::settings::DaemonSettings;
use crate::status::StaticMetadataStatsProvider;
use crate::tags::StaticFileTagProvider;
use crate::upload::StaticFileUploadProvider;
use crate::urls::PublicUrlBuilder;
use crate::{content, delete, notes, pins, search, sessions, status, tags, upload, workspaces};

/// Shared HTTP state.
#[derive(Clone)]
pub struct HttpState {
    pub(crate) started_at: Instant,
    pub(crate) stats_provider: Arc<dyn status::MetadataStatsProvider>,
    pub(crate) upload_provider: Arc<dyn upload::FileUploadProvider>,
    pub(crate) delete_provider: Arc<dyn delete::FileDeleteProvider>,
    pub(crate) tag_provider: Arc<dyn tags::FileTagProvider>,
    pub(crate) pin_provider: Arc<dyn pins::FilePinProvider>,
    pub(crate) search_provider: Arc<dyn search::FileSearchProvider>,
    pub(crate) session_provider: Arc<dyn sessions::SessionProvider>,
    pub(crate) note_provider: Arc<dyn notes::NoteProvider>,
    pub(crate) blob_reader: Arc<dyn BlobReader + Send + Sync>,
    pub(crate) upload_temp_dir: PathBuf,
    pub(crate) storage_mutation_lock: Arc<tokio::sync::Mutex<()>>,
    pub(crate) auth: AuthService,
    pub(crate) workspaces: Option<Arc<workspaces::WorkspaceStore>>,
    settings: Arc<DaemonSettings>,
    public_urls: PublicUrlBuilder,
    pub(crate) corrupt_file_count: u64,
}

impl HttpState {
    /// Creates a base HTTP state with static/placeholder providers.
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
            stats_provider: Arc::new(StaticMetadataStatsProvider),
            upload_provider: Arc::new(StaticFileUploadProvider),
            delete_provider: Arc::new(StaticFileDeleteProvider),
            tag_provider: Arc::new(StaticFileTagProvider),
            pin_provider: Arc::new(StaticFilePinProvider),
            search_provider: Arc::new(StaticFileSearchProvider),
            session_provider: Arc::new(StaticSessionProvider),
            note_provider: Arc::new(StaticNoteProvider),
            blob_reader: Arc::new(content::StaticBlobReader),
            upload_temp_dir,
            storage_mutation_lock: Arc::new(tokio::sync::Mutex::new(())),
            auth: AuthService::disabled(),
            workspaces: None,
            settings,
            public_urls,
            corrupt_file_count,
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

    /// Builds HTTP state for integration tests.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn test_http_state(upload_temp_dir: PathBuf) -> Self {
        let settings = Arc::new(DaemonSettings::default());
        let urls = PublicUrlBuilder::from_settings(&settings);
        Self::new(Instant::now(), upload_temp_dir, settings, urls, 0)
    }

    /// Builds HTTP state for integration tests with custom settings.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn test_http_state_with_settings(
        upload_temp_dir: PathBuf,
        settings: DaemonSettings,
    ) -> Self {
        let settings = Arc::new(settings);
        let urls = PublicUrlBuilder::from_settings(&settings);
        Self::new(Instant::now(), upload_temp_dir, settings, urls, 0)
    }

    /// Sets the metadata stats provider.
    #[must_use]
    pub fn with_stats_provider(mut self, provider: Arc<dyn status::MetadataStatsProvider>) -> Self {
        self.stats_provider = provider;
        self
    }

    /// Sets the file upload provider.
    #[must_use]
    pub fn with_upload_provider(mut self, provider: Arc<dyn upload::FileUploadProvider>) -> Self {
        self.upload_provider = provider;
        self
    }

    /// Sets the file delete provider.
    #[must_use]
    pub fn with_delete_provider(mut self, provider: Arc<dyn delete::FileDeleteProvider>) -> Self {
        self.delete_provider = provider;
        self
    }

    /// Sets the file tag provider.
    #[must_use]
    pub fn with_tag_provider(mut self, provider: Arc<dyn tags::FileTagProvider>) -> Self {
        self.tag_provider = provider;
        self
    }

    /// Sets the file pin provider.
    #[must_use]
    pub fn with_pin_provider(mut self, provider: Arc<dyn pins::FilePinProvider>) -> Self {
        self.pin_provider = provider;
        self
    }

    /// Sets the blob reader provider.
    #[must_use]
    pub fn with_blob_reader(mut self, reader: Arc<dyn BlobReader + Send + Sync>) -> Self {
        self.blob_reader = reader;
        self
    }

    /// Sets the search provider.
    #[must_use]
    pub fn with_search_provider(mut self, provider: Arc<dyn search::FileSearchProvider>) -> Self {
        self.search_provider = provider;
        self
    }

    /// Sets the session provider.
    #[must_use]
    pub fn with_session_provider(mut self, provider: Arc<dyn sessions::SessionProvider>) -> Self {
        self.session_provider = provider;
        self
    }

    /// Sets the note provider.
    #[must_use]
    pub fn with_note_provider(mut self, provider: Arc<dyn notes::NoteProvider>) -> Self {
        self.note_provider = provider;
        self
    }

    /// Sets the authentication service.
    #[must_use]
    pub fn with_auth(mut self, auth: AuthService) -> Self {
        self.auth = auth;
        self
    }
}
