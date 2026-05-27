//! Daemon process lifecycle and main loop.

use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use tokio::net::TcpListener;
use tssp_app::{
    DeleteFileService, FolderService, GitService, LspService, NoteService, PinService,
    PurgeDeletedFilesService, RestoreFileService, SessionService, TagService, TerminalService,
    UploadService, WorkspaceFileService,
};
use tsspd::auth::initialize_database as initialize_auth_database;
use tsspd::workspaces::WorkspaceStore;
use tsspd::{
    auth::{AuthService, AuthStore, DeviceStore, UserStore},
    bind_error_message, build_router, collect_garbage, spawn_advertisement,
    spawn_startup_integrity_scan,
    trash_cleanup::purge_expired_trash,
    ApplicationFileDeleteProvider, ApplicationFilePinProvider, ApplicationFileRestoreProvider,
    ApplicationFileTagProvider, ApplicationFileUploadProvider, ApplicationFolderProvider,
    ApplicationNoteProvider, ApplicationSessionProvider, CliOverrides, DaemonSettings, HttpState,
    PublicUrlBuilder, RepositoryFileSearchProvider, RepositoryMetadataStatsProvider,
};

use super::Cli;

/// Application runtime path configurations.
#[derive(Debug, Clone)]
struct RuntimePaths {
    data_dir: PathBuf,
    metadata_path: PathBuf,
    _blob_dir: PathBuf,
    upload_temp_dir: PathBuf,
}

/// Runs the `tsspd` daemon process.
///
/// This is the main entry point for the daemon, initializing all services,
/// background tasks, and starting the HTTP server.
pub async fn run(cli: Cli) -> Result<(), String> {
    let settings = load_settings(&cli)?;

    if cli.check_config {
        print_check_config(&settings);
        return Ok(());
    }

    settings.log_effective();
    warn_if_insecure_bind(&settings);
    let paths = prepare_runtime_paths(&settings)?;
    let pool = create_connection_pool(&paths.metadata_path)?;
    {
        let connection = pool
            .get()
            .map_err(|error| format!("could not initialize metadata database: {error}"))?;
        initialize_connection(&connection)
            .map_err(|error| format!("could not initialize metadata database: {error}"))?;
        initialize_auth_database(&connection)
            .map_err(|error| format!("could not initialize auth database: {error}"))?;
    }
    let repository = open_repository(pool.clone());
    run_integrity_check(&paths.metadata_path)
        .map_err(|error| format!("database integrity check failed: {error}"))?;
    let storage = open_storage(&settings)?;

    let session_service = start_session_service(pool.clone(), &paths.upload_temp_dir)?;
    let auth_service = start_auth_service(pool.clone(), &settings)?;

    let settings = Arc::new(settings);
    let address = settings.socket_addr();
    let listener = TcpListener::bind(address)
        .await
        .map_err(|error| bind_error_message(address, &error))?;

    if settings.mdns {
        spawn_advertisement(settings.port);
    }

    let state = build_http_state(
        &settings,
        paths.clone(),
        pool,
        repository.clone(),
        storage.clone(),
        session_service,
        auth_service,
        0, // Background scan will update this
    );

    spawn_startup_integrity_scan(
        repository.clone(),
        Arc::new(storage.clone()),
        state.corrupt_file_count.clone(),
    );

    spawn_background_tasks(
        &state,
        &Arc::new(storage),
        &repository,
        &settings,
        &paths,
    );

    let router = build_router(state);

    tracing::info!("tsspd listening on http://{address}");
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .map_err(|error| format!("server failed: {error}"))?;

    tracing::info!("tsspd stopped cleanly");
    Ok(())
}

fn spawn_background_tasks(
    state: &HttpState,
    storage: &Arc<tssp_adapter_fs::FilesystemBlobStore>,
    repository: &Arc<tssp_adapter_sqlite::SqliteFileRepository>,
    settings: &Arc<DaemonSettings>,
    paths: &RuntimePaths,
) {
    let storage_gc = storage.clone();
    let repository_gc = repository.clone();
    tokio::spawn(async move {
        match collect_garbage(storage_gc.root(), repository_gc.as_ref()) {
            Ok(count) => {
                if count > 0 {
                    tracing::info!("garbage collection: deleted {count} orphaned blobs");
                }
            }
            Err(e) => tracing::warn!("garbage collection failed: {e}"),
        }
    });

    let upload_session_manager = state.upload_session_manager.clone();
    let upload_temp_dir = paths.upload_temp_dir.clone();
    let purge_service = PurgeDeletedFilesService::new(storage.clone(), repository.clone());
    let retention_days = settings.trash_retention_days;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;

            let max_age = std::time::Duration::from_secs(24 * 60 * 60);
            let cleaned = upload_session_manager
                .cleanup_expired_with_disk(max_age, &upload_temp_dir)
                .await;
            if cleaned > 0 {
                tracing::info!("cleanup: removed {cleaned} abandoned upload sessions");
            }

            let purge_report = purge_expired_trash(&purge_service, retention_days);
            if purge_report.files_purged > 0 {
                tracing::info!(
                    "trash: purged {count} files older than {retention_days} days",
                    count = purge_report.files_purged
                );
            }
            if purge_report.error {
                tracing::warn!("trash: purge operation failed");
            }
        }
    });

    let terminal_service = state.terminal_service.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            if let Err(e) = terminal_service.cleanup_expired_sessions().await {
                tracing::warn!("terminal cleanup error: {e}");
            }
        }
    });
}

fn load_settings(cli: &Cli) -> Result<DaemonSettings, String> {
    let data_dir = cli.data_dir.clone().unwrap_or_else(|| PathBuf::from("."));
    let overrides = CliOverrides {
        data_dir: cli.data_dir.clone(),
        bind: cli.bind,
        port: cli.port,
        public_url: Some(cli.public_url.clone()),
        trust_forwarded: cli.trust_forwarded,
        check_config: cli.check_config,
        mdns: cli.mdns,
        metrics: cli.metrics,
        web: cli.web,
    };

    DaemonSettings::load(&data_dir, &overrides)
        .map_err(|error| format!("failed to load settings: {error}"))
}

fn print_check_config(settings: &DaemonSettings) {
    println!("Configuration check successful.");
    println!("- Data directory: {}", settings.data_dir.display());
    println!("- Bind address: {}", settings.bind);
    println!("- Port: {}", settings.port);
    if let Some(ref url) = settings.public_url {
        println!("- Public URL: {url}");
    }
}

fn warn_if_insecure_bind(settings: &DaemonSettings) {
    let unspecified_v4 = IpAddr::from([0, 0, 0, 0]);
    let unspecified_v6 = IpAddr::from([0, 0, 0, 0, 0, 0, 0, 0]);
    if settings.bind == unspecified_v4 || settings.bind == unspecified_v6 {
        tracing::warn!(
            "listening on all interfaces ({}); ensure a firewall is active",
            settings.bind
        );
    }
}

fn prepare_runtime_paths(settings: &DaemonSettings) -> Result<RuntimePaths, String> {
    let data_dir = settings.data_dir.clone();
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)
            .map_err(|error| format!("could not create data directory: {error}"))?;
    }

    let metadata_path = data_dir.join("metadata.db");
    let blob_dir = data_dir.join("blobs");
    let upload_temp_dir = data_dir.join("temp").join("uploads");

    if !blob_dir.exists() {
        std::fs::create_dir_all(&blob_dir)
            .map_err(|error| format!("could not create blob directory: {error}"))?;
    }

    if !upload_temp_dir.exists() {
        std::fs::create_dir_all(&upload_temp_dir)
            .map_err(|error| format!("could not create upload temp directory: {error}"))?;
    }

    Ok(RuntimePaths {
        data_dir,
        metadata_path,
        _blob_dir: blob_dir,
        upload_temp_dir,
    })
}

fn create_connection_pool(
    metadata_path: &Path,
) -> Result<r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>, String> {
    let manager = r2d2_sqlite::SqliteConnectionManager::file(metadata_path);
    r2d2::Pool::new(manager).map_err(|error| format!("could not create connection pool: {error}"))
}

fn initialize_connection(connection: &rusqlite::Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;",
        )
        .map_err(|error| format!("could not initialize connection: {error}"))
}

fn open_repository(
    pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
) -> Arc<tssp_adapter_sqlite::SqliteFileRepository> {
    Arc::new(tssp_adapter_sqlite::SqliteFileRepository::new(pool))
}

fn run_integrity_check(metadata_path: &Path) -> Result<(), String> {
    let connection = rusqlite::Connection::open(metadata_path)
        .map_err(|error| format!("could not open database for integrity check: {error}"))?;
    let mut statement = connection
        .prepare("PRAGMA integrity_check")
        .map_err(|error| format!("could not prepare integrity check: {error}"))?;
    let mut rows = statement
        .query([])
        .map_err(|error| format!("could not run integrity check: {error}"))?;

    if let Some(row) = rows
        .next()
        .map_err(|error| format!("could not read integrity check result: {error}"))?
    {
        let result: String = row
            .get(0)
            .map_err(|error| format!("could not get integrity check result: {error}"))?;
        if result != "ok" {
            return Err(format!("database integrity check failed: {result}"));
        }
    }

    Ok(())
}

fn open_storage(settings: &DaemonSettings) -> Result<tssp_adapter_fs::FilesystemBlobStore, String> {
    let blob_dir = settings.data_dir.join("blobs");
    tssp_adapter_fs::FilesystemBlobStore::new(blob_dir)
        .map_err(|error| format!("could not open blob storage: {error}"))
}

#[allow(clippy::unnecessary_wraps)]
fn start_session_service(
    pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
    _upload_temp_dir: &Path,
) -> Result<SessionService<tssp_adapter_sqlite::SqliteSessionRepository>, String> {
    let repository = tssp_adapter_sqlite::SqliteSessionRepository::new(pool);
    Ok(SessionService::new(repository))
}

#[allow(clippy::unnecessary_wraps)]
fn start_auth_service(
    pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
    settings: &DaemonSettings,
) -> Result<AuthService, String> {
    let user_store = Arc::new(UserStore::new(pool.clone()));
    let auth_store = Arc::new(AuthStore::new(pool.clone()));
    let device_store = Arc::new(DeviceStore::new(pool));

    Ok(AuthService::new(
        auth_store,
        user_store,
        device_store,
        settings.trust_forwarded,
        false, // auth_required flag, set based on settings if needed
    ))
}

#[allow(clippy::too_many_arguments)]
fn build_http_state(
    settings: &Arc<DaemonSettings>,
    paths: RuntimePaths,
    pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
    repository: Arc<tssp_adapter_sqlite::SqliteFileRepository>,
    storage: tssp_adapter_fs::FilesystemBlobStore,
    session_service: SessionService<tssp_adapter_sqlite::SqliteSessionRepository>,
    auth_service: AuthService,
    corrupt_file_count: u64,
) -> HttpState {
    let public_urls = PublicUrlBuilder::from_settings(settings);
    let storage = Arc::new(storage);
    let workspace_store = Arc::new(WorkspaceStore::new(pool));
    let id_generator = tssp_adapter_system::UuidV7FileIdGenerator;
    let clock = tssp_adapter_system::SystemClock;

    let upload_service =
        UploadService::new(storage.clone(), repository.clone(), id_generator, clock);

    let folder_service = FolderService::new(repository.clone());
    let delete_service = DeleteFileService::new(storage.clone(), repository.clone());
    let restore_service = RestoreFileService::new(repository.clone());
    let tag_service = TagService::new(repository.clone());
    let pin_service = PinService::new(repository.clone());
    let note_service = NoteService::new(repository.clone(), clock, id_generator);

    let workspace_file_store = Arc::new(tssp_adapter_fs::FilesystemWorkspaceFileStore::new(
        paths.data_dir.join("workspaces"),
    ));
    let workspace_file_service = Arc::new(WorkspaceFileService::new(workspace_file_store));

    let terminal_provider = Arc::new(tssp_adapter_system::terminal::LinuxTerminalProvider::new());
    let terminal_service = Arc::new(TerminalService::new(terminal_provider));

    let lsp_provider = Arc::new(tssp_adapter_system::lsp::SystemLspProvider::new());
    let lsp_service = Arc::new(LspService::new(lsp_provider));

    let git_provider = Arc::new(tssp_adapter_system::git::SystemGitProvider::new());
    let git_service = Arc::new(GitService::new(git_provider));

    HttpState::new(
        Instant::now(),
        paths.upload_temp_dir,
        settings.clone(),
        public_urls,
        corrupt_file_count,
    )
    .with_auth(auth_service)
    .with_workspaces(workspace_store)
    .with_repository(repository.clone())
    .with_stats_provider(Arc::new(RepositoryMetadataStatsProvider::new(
        repository.clone(),
        clock,
    )))
    .with_upload_provider(Arc::new(ApplicationFileUploadProvider::new(upload_service)))
    .with_folder_provider(Arc::new(ApplicationFolderProvider::new(folder_service)))
    .with_delete_provider(Arc::new(ApplicationFileDeleteProvider::new(delete_service)))
    .with_restore_provider(Arc::new(ApplicationFileRestoreProvider::new(
        restore_service,
    )))
    .with_tag_provider(Arc::new(ApplicationFileTagProvider::new(tag_service)))
    .with_pin_provider(Arc::new(ApplicationFilePinProvider::new(pin_service)))
    .with_session_provider(Arc::new(ApplicationSessionProvider::new(
        session_service,
        clock,
    )))
    .with_note_provider(Arc::new(ApplicationNoteProvider::new(note_service)))
    .with_search_provider(Arc::new(RepositoryFileSearchProvider::new(repository)))
    .with_blob_reader(storage)
    .with_workspace_file_service(workspace_file_service)
    .with_terminal_service(terminal_service)
    .with_lsp_service(lsp_service)
    .with_git_service(git_service)
}
