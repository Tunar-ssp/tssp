//! Shared HTTP application state and builder methods.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use tssp_ports::{BlobReader, FileRepository};

use crate::auth::AuthService;
use crate::chunked_upload::UploadSessionManager;
use crate::content::StaticBlobReader;
use crate::delete::{
    FileDeleteProvider, FileRestoreProvider, StaticFileDeleteProvider, StaticFileRestoreProvider,
};
use crate::folders::FolderProvider;
use crate::notes::{NoteProvider, StaticNoteProvider};
use crate::pins::{FilePinProvider, StaticFilePinProvider};
use crate::rate_limit::RateLimiter;
use crate::search::{FileSearchProvider, StaticFileSearchProvider};
use crate::sessions::{SessionProvider, StaticSessionProvider};
use crate::settings::DaemonSettings;
use crate::stats_cache::StatsCache;
use crate::status::{MetadataStatsProvider, StaticMetadataStatsProvider};
use crate::tags::{FileTagProvider, StaticFileTagProvider};
use crate::upload::{FileUploadProvider, StaticFileUploadProvider};
use crate::urls::PublicUrlBuilder;
use crate::workspaces;

/// Shared HTTP state.
pub struct HttpState {
    pub(crate) started_at: Instant,
    pub(crate) upload_temp_dir: PathBuf,
    pub(crate) storage_mutation_lock: Arc<tokio::sync::Mutex<()>>,
    pub(crate) auth: AuthService,
    pub(crate) rate_limiter: RateLimiter,
    pub(crate) stats_cache: StatsCache,
    pub(crate) workspaces: Option<Arc<workspaces::WorkspaceStore>>,
    pub(crate) settings: Arc<DaemonSettings>,
    pub(crate) public_urls: PublicUrlBuilder,
    /// Number of files whose content blob is missing on disk.
    pub corrupt_file_count: Arc<std::sync::atomic::AtomicU64>,
    /// Session manager for chunked uploads.
    pub upload_session_manager: Arc<UploadSessionManager>,
    pub(crate) repository: Arc<dyn FileRepository + Send + Sync>,
    pub(crate) stats_provider: Arc<dyn MetadataStatsProvider>,
    pub(crate) upload_provider: Arc<dyn FileUploadProvider>,
    pub(crate) folder_provider: Arc<dyn FolderProvider>,
    pub(crate) delete_provider: Arc<dyn FileDeleteProvider>,
    pub(crate) restore_provider: Arc<dyn FileRestoreProvider>,
    pub(crate) tag_provider: Arc<dyn FileTagProvider>,
    pub(crate) pin_provider: Arc<dyn FilePinProvider>,
    pub(crate) session_provider: Arc<dyn SessionProvider>,
    pub(crate) note_provider: Arc<dyn NoteProvider>,
    pub(crate) search_provider: Arc<dyn FileSearchProvider>,
    pub(crate) blob_reader: Arc<dyn BlobReader + Send + Sync>,
    /// Terminal session manager for WebSocket connections.
    pub terminal_manager: Arc<crate::terminal::TerminalManager>,
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
            stats_cache: StatsCache::new(),
            workspaces: None,
            settings,
            public_urls,
            corrupt_file_count: Arc::new(std::sync::atomic::AtomicU64::new(corrupt_file_count)),
            upload_session_manager: Arc::new(UploadSessionManager::new()),
            repository: Arc::new(StaticFileRepository),
            stats_provider: Arc::new(StaticMetadataStatsProvider),
            upload_provider: Arc::new(StaticFileUploadProvider),
            folder_provider: Arc::new(StaticFolderProvider),
            delete_provider: Arc::new(StaticFileDeleteProvider),
            restore_provider: Arc::new(StaticFileRestoreProvider),
            tag_provider: Arc::new(StaticFileTagProvider),
            pin_provider: Arc::new(StaticFilePinProvider),
            session_provider: Arc::new(StaticSessionProvider),
            note_provider: Arc::new(StaticNoteProvider),
            search_provider: Arc::new(StaticFileSearchProvider),
            blob_reader: Arc::new(StaticBlobReader),
            terminal_manager: Arc::new(crate::terminal::TerminalManager::new()),
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

    /// Attaches the folder provider.
    #[must_use]
    pub fn with_folder_provider(mut self, provider: Arc<dyn FolderProvider>) -> Self {
        self.folder_provider = provider;
        self
    }

    /// Attaches the delete provider.
    #[must_use]
    pub fn with_delete_provider(mut self, provider: Arc<dyn FileDeleteProvider>) -> Self {
        self.delete_provider = provider;
        self
    }

    /// Attaches the restore provider.
    #[must_use]
    pub fn with_restore_provider(mut self, provider: Arc<dyn FileRestoreProvider>) -> Self {
        self.restore_provider = provider;
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

    /// Attaches the file repository.
    #[must_use]
    pub fn with_repository<R: FileRepository + Send + Sync + 'static>(
        mut self,
        repository: Arc<R>,
    ) -> Self {
        self.repository = repository as Arc<dyn FileRepository + Send + Sync>;
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

impl Clone for HttpState {
    fn clone(&self) -> Self {
        Self {
            started_at: self.started_at,
            upload_temp_dir: self.upload_temp_dir.clone(),
            storage_mutation_lock: self.storage_mutation_lock.clone(),
            auth: self.auth.clone(),
            rate_limiter: self.rate_limiter.clone(),
            stats_cache: self.stats_cache.clone(),
            workspaces: self.workspaces.clone(),
            settings: self.settings.clone(),
            public_urls: self.public_urls.clone(),
            corrupt_file_count: self.corrupt_file_count.clone(),
            upload_session_manager: self.upload_session_manager.clone(),
            repository: self.repository.clone(),
            stats_provider: self.stats_provider.clone(),
            upload_provider: self.upload_provider.clone(),
            folder_provider: self.folder_provider.clone(),
            delete_provider: self.delete_provider.clone(),
            restore_provider: self.restore_provider.clone(),
            tag_provider: self.tag_provider.clone(),
            pin_provider: self.pin_provider.clone(),
            session_provider: self.session_provider.clone(),
            note_provider: self.note_provider.clone(),
            search_provider: self.search_provider.clone(),
            blob_reader: self.blob_reader.clone(),
            terminal_manager: self.terminal_manager.clone(),
        }
    }
}

/// No-op file repository for testing.
#[derive(Clone, Copy)]
struct StaticFileRepository;

impl FileRepository for StaticFileRepository {
    fn insert_file(
        &self,
        _new_file: tssp_ports::NewFileRecord,
    ) -> Result<tssp_domain::FileRecord, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn find_file(
        &self,
        _id: &tssp_domain::FileId,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn find_file_by_content_hash(
        &self,
        _content_hash: &tssp_domain::ContentHash,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn delete_file(
        &self,
        _id: &tssp_domain::FileId,
    ) -> Result<Option<tssp_ports::DeletedFileRecord>, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn restore_file(
        &self,
        _id: &tssp_domain::FileId,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn list_deleted_files(
        &self,
        _older_than: tssp_domain::UnixTimestamp,
    ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn purge_deleted_file(
        &self,
        _id: &tssp_domain::FileId,
    ) -> Result<bool, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn list_files(
        &self,
        _query: &tssp_ports::ListQuery,
    ) -> Result<tssp_ports::PagedFiles, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn list_files_recent(
        &self,
        _limit: u64,
    ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn list_files_by_tag(
        &self,
        _tag: &tssp_domain::TagKey,
        _limit: u64,
    ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn list_tags(&self) -> Result<Vec<tssp_ports::TagSummary>, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn add_tags_to_file(
        &self,
        _id: &tssp_domain::FileId,
        _tags: &[tssp_domain::Tag],
    ) -> Result<tssp_ports::TagMutationOutcome, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn remove_tag_from_file(
        &self,
        _id: &tssp_domain::FileId,
        _tag: &tssp_domain::TagKey,
    ) -> Result<tssp_ports::TagMutationOutcome, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn stats_since(
        &self,
        _recent_since: tssp_domain::UnixTimestamp,
    ) -> Result<tssp_ports::RepositoryStats, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn pin_file(
        &self,
        _id: &tssp_domain::FileId,
        _position: Option<u32>,
    ) -> Result<tssp_ports::PinOutcome, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn unpin_file(
        &self,
        _id: &tssp_domain::FileId,
    ) -> Result<tssp_ports::PinOutcome, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn list_pinned_files(
        &self,
    ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn reorder_pins(
        &self,
        _ordered_ids: &[tssp_domain::FileId],
    ) -> Result<(), tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn search_files(
        &self,
        _query: &str,
    ) -> Result<Vec<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn rename_file(
        &self,
        _id: &tssp_domain::FileId,
        _new_name: &tssp_domain::FileName,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn list_folder_counts(
        &self,
        _owner_id: Option<&tssp_domain::UserId>,
    ) -> Result<Vec<(String, u64)>, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn set_file_visibility(
        &self,
        _id: &tssp_domain::FileId,
        _visibility: tssp_domain::Visibility,
        _public_token: Option<&str>,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn find_file_by_public_token(
        &self,
        _token: &str,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn update_folder_path_prefix(
        &self,
        _from_prefix: &str,
        _to_prefix: &str,
    ) -> Result<u64, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn set_file_folder_path(
        &self,
        _id: &tssp_domain::FileId,
        _folder_path: &str,
    ) -> Result<Option<tssp_domain::FileRecord>, tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }

    fn insert_audit_event(
        &self,
        _id: &str,
        _timestamp: i64,
        _user_id: Option<&str>,
        _action: &str,
        _resource: Option<&str>,
        _resource_id: Option<&str>,
        _status: &str,
        _details: Option<&str>,
    ) -> Result<(), tssp_ports::RepositoryError> {
        Err(tssp_ports::RepositoryError::OperationFailed {
            message: "static repository is not configured".to_owned(),
        })
    }
}

/// No-op folder provider for testing.
pub struct StaticFolderProvider;

impl crate::folders::FolderProvider for StaticFolderProvider {
    fn move_folder(&self, _from: &str, _to: &str) -> Result<u64, crate::folders::HttpFolderError> {
        Err(crate::folders::HttpFolderError::Internal(
            "folder service is not configured".to_owned(),
        ))
    }

    fn delete_folder(&self, _path: &str) -> Result<u64, crate::folders::HttpFolderError> {
        Err(crate::folders::HttpFolderError::Internal(
            "folder service is not configured".to_owned(),
        ))
    }

    fn list_folders(
        &self,
        _owner_id: Option<&tssp_domain::UserId>,
    ) -> Result<Vec<(String, u64)>, crate::folders::HttpFolderError> {
        Err(crate::folders::HttpFolderError::Internal(
            "folder service is not configured".to_owned(),
        ))
    }
}
