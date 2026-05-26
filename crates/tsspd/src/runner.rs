//! Startup orchestration for the `tsspd` binary.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tokio::net::TcpListener;
use tssp_adapter_fs::{FilesystemBlobStore, FilesystemWorkspaceFileStore};
use tssp_adapter_sqlite::{initialize_connection, SqliteFileRepository, SqliteSessionRepository};
use tssp_adapter_system::SystemClock;
use tssp_adapter_system::UuidV7FileIdGenerator;
use tssp_app::{
    DeleteFileService, FolderService, NoteService, PinService, PurgeDeletedFilesService,
    RestoreFileService, SessionService, TagService, UploadService, WorkspaceFileService,
};
use tssp_ports::Clock;
use tsspd::workspaces::WorkspaceStore;
use tsspd::{
    auth::{
        initialize_database as initialize_auth_database, AuthService, AuthStore, DeviceStore,
        UserStore,
    },
    bind_error_message, build_router, collect_garbage, spawn_advertisement,
    spawn_startup_integrity_scan, spawn_terminal_cleanup,
    trash_cleanup::purge_expired_trash,
    ApplicationFileDeleteProvider, ApplicationFilePinProvider, ApplicationFileRestoreProvider,
    ApplicationFileTagProvider, ApplicationFileUploadProvider, ApplicationFolderProvider,
    ApplicationNoteProvider, ApplicationSessionProvider, CliOverrides, DaemonSettings, HttpState,
    PublicUrlBuilder, RepositoryFileSearchProvider, RepositoryMetadataStatsProvider,
};

use super::Cli;

#[derive(Clone)]
struct RuntimePaths {
    upload_temp_dir: PathBuf,
    metadata_path: PathBuf,
}

fn cli_overrides(cli: &Cli) -> CliOverrides {
    CliOverrides {
        bind: cli.bind,
        port: cli.port,
        data_dir: cli.data_dir.clone(),
        public_url: if cli.public_url.is_some() {
            Some(cli.public_url.clone())
        } else {
            None
        },
        trust_forwarded: cli.trust_forwarded,
        mdns: cli.mdns,
        metrics: cli.metrics,
        web: cli.web,
        check_config: cli.check_config,
    }
}

fn run_integrity_check(db_path: &Path) -> Result<(), String> {
    if !db_path.exists() {
        return Ok(());
    }
    let conn = rusqlite::Connection::open(db_path)
        .map_err(|e| format!("could not open database for integrity check: {e}"))?;
    let result: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|e| format!("integrity check query failed: {e}"))?;
    if result != "ok" {
        return Err(format!("database integrity check reported: {result}"));
    }
    tracing::info!("startup: database integrity check passed");
    Ok(())
}

fn cleanup_temp_uploads(temp_dir: &Path) {
    let report = tsspd::temp_cleanup::cleanup_temp_upload_dir(temp_dir, None);
    if report.total_removed() > 0 {
        tracing::info!(
            files_removed = report.files_removed,
            directories_removed = report.directories_removed,
            "startup: cleaned up orphaned temp uploads"
        );
    }
    if report.errors > 0 {
        tracing::warn!(
            errors = report.errors,
            "startup: some temp upload entries could not be removed"
        );
    }
}

fn load_settings(cli: &Cli) -> Result<DaemonSettings, String> {
    let data_dir = cli
        .data_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from("data"));
    DaemonSettings::load(&data_dir, &cli_overrides(cli))
}

fn print_check_config(settings: &DaemonSettings) {
    println!(
        "configuration ok: {}, data dir {}",
        settings.socket_addr(),
        settings.data_dir.display()
    );
    println!("config file: {}", settings.config_file_path().display());
}

fn prepare_runtime_paths(settings: &DaemonSettings) -> Result<RuntimePaths, String> {
    std::fs::create_dir_all(&settings.data_dir)
        .map_err(|error| format!("could not create data directory: {error}"))?;

    if !settings.data_dir.is_dir() {
        return Err(format!(
            "data directory {} is not accessible",
            settings.data_dir.display()
        ));
    }

    let upload_temp_dir = settings.data_dir.join("http-upload-tmp");
    std::fs::create_dir_all(&upload_temp_dir)
        .map_err(|error| format!("could not create upload temp directory: {error}"))?;

    Ok(RuntimePaths {
        upload_temp_dir,
        metadata_path: settings.data_dir.join("metadata.sqlite3"),
    })
}

fn create_connection_pool(metadata_path: &Path) -> Result<Pool<SqliteConnectionManager>, String> {
    SqliteFileRepository::create_pool(metadata_path, 30)
        .map_err(|e| format!("could not create metadata connection pool: {e}"))
}

fn open_repository(pool: Pool<SqliteConnectionManager>) -> Arc<SqliteFileRepository> {
    Arc::new(SqliteFileRepository::new(pool))
}

fn open_storage(settings: &DaemonSettings) -> Result<Arc<FilesystemBlobStore>, String> {
    FilesystemBlobStore::new(settings.data_dir.join("storage"))
        .map(Arc::new)
        .map_err(|error| format!("could not initialize blob storage: {error}"))
}

fn bootstrap_admin_user(auth: &AuthService, now: i64) -> Result<(), String> {
    let Some(users) = auth.users() else {
        return Ok(());
    };
    if users.count_users().map_err(|e| e.to_string())? > 0 {
        return Ok(());
    }
    let code =
        std::env::var("TSSPD_BOOTSTRAP_ADMIN_CODE").unwrap_or_else(|_| "changeme".to_owned());
    let code = code.trim();
    if code.len() < 4 {
        return Err("TSSPD_BOOTSTRAP_ADMIN_CODE must be at least 4 characters".to_owned());
    }
    let id = tssp_domain::UserId::new("user-tunar").map_err(|e| e.to_string())?;
    let name = tssp_domain::UserName::new(
        std::env::var("TSSPD_BOOTSTRAP_ADMIN_NAME").unwrap_or_else(|_| "Tunar".to_owned()),
    )
    .map_err(|e| e.to_string())?;
    users
        .create_user(&id, &name, tssp_domain::UserRole::Admin, code, now)
        .map_err(|e| e.to_string())?;
    tracing::info!(
        "startup: created default admin user '{}' (set TSSPD_BOOTSTRAP_ADMIN_CODE to customize)",
        name.as_str()
    );
    Ok(())
}

fn configure_auth_password(auth: &AuthService) -> Result<(), String> {
    if let Ok(hash) = std::env::var("TSSPD_AUTH_PASSWORD_HASH") {
        let hash = hash.trim();
        if !hash.is_empty() {
            auth.set_password_hash(hash)
                .map_err(|error| format!("could not store password hash: {error}"))?;
            tracing::info!("startup: remote authentication enabled (password hash from env)");
            return Ok(());
        }
    }

    if let Ok(password) = std::env::var("TSSPD_AUTH_PASSWORD") {
        let password = password.trim();
        if !password.is_empty() {
            let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
                .map_err(|error| format!("could not hash password: {error}"))?;
            auth.set_password_hash(&hash)
                .map_err(|error| format!("could not store password hash: {error}"))?;
            tracing::info!("startup: remote authentication enabled (password from env)");
        }
    }

    Ok(())
}

fn start_session_service(
    pool: Pool<SqliteConnectionManager>,
    upload_temp_dir: &Path,
) -> Result<SessionService<SqliteSessionRepository>, String> {
    let session_repository = SqliteSessionRepository::new(pool);
    let session_service = SessionService::new(session_repository);

    let now = SystemClock.now();
    let deleted = session_service
        .cleanup_expired_sessions(now)
        .map_err(|error| format!("session cleanup failed: {error}"))?;
    if deleted > 0 {
        tracing::info!("startup: removed {deleted} expired sessions");
    }

    cleanup_temp_uploads(upload_temp_dir);
    Ok(session_service)
}

fn start_auth_service(
    pool: Pool<SqliteConnectionManager>,
    settings: &DaemonSettings,
) -> Result<AuthService, String> {
    let auth_store = Arc::new(AuthStore::new(pool.clone()));
    let user_store = Arc::new(UserStore::new(pool.clone()));
    let device_store = Arc::new(DeviceStore::new(pool));
    let auth_service = AuthService::new(
        auth_store,
        user_store,
        device_store,
        settings.trust_forwarded,
        settings.public_url.is_some(),
    );

    let now_secs = SystemClock.now().seconds();
    bootstrap_admin_user(&auth_service, now_secs)?;
    configure_auth_password(&auth_service)?;
    let (removed_tokens, removed_devices) = auth_service
        .cleanup_expired(now_secs)
        .map_err(|error| format!("auth cleanup failed: {error}"))?;
    if removed_tokens > 0 {
        tracing::info!("startup: removed {removed_tokens} expired auth tokens");
    }
    if removed_devices > 0 {
        tracing::info!("startup: removed {removed_devices} expired trusted devices");
    }

    Ok(auth_service)
}

#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
fn build_http_state(
    settings: &Arc<DaemonSettings>,
    paths: RuntimePaths,
    pool: Pool<SqliteConnectionManager>,
    #[allow(clippy::needless_pass_by_value)] repository: Arc<SqliteFileRepository>,
    storage: Arc<FilesystemBlobStore>,
    session_service: SessionService<SqliteSessionRepository>,
    auth_service: AuthService,
    corrupt_file_count: u64,
) -> HttpState {
    let stats_provider = RepositoryMetadataStatsProvider::new(repository.clone(), SystemClock);
    let upload_service = UploadService::new(
        storage.clone(),
        repository.clone(),
        UuidV7FileIdGenerator,
        SystemClock,
    );
    let delete_service = DeleteFileService::new(storage.clone(), repository.clone());
    let folder_service = FolderService::new(repository.clone());
    let restore_service = RestoreFileService::new(repository.clone());
    let tag_service = TagService::new(repository.clone());
    let pin_service = PinService::new(repository.clone());
    let note_service = NoteService::new(repository.clone(), SystemClock, UuidV7FileIdGenerator);
    let workspace_store = Arc::new(WorkspaceStore::new(pool));
    let workspace_file_store = Arc::new(FilesystemWorkspaceFileStore::new(
        settings.data_dir.join("workspaces"),
    ));
    let workspace_file_service = Arc::new(WorkspaceFileService::new(workspace_file_store));

    HttpState::new(
        Instant::now(),
        paths.upload_temp_dir,
        settings.clone(),
        PublicUrlBuilder::from_settings(settings),
        corrupt_file_count,
    )
    .with_repository(repository.clone())
    .with_workspaces(workspace_store)
    .with_workspace_file_service(workspace_file_service)
    .with_stats_provider(Arc::new(stats_provider))
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
        SystemClock,
    )))
    .with_note_provider(Arc::new(ApplicationNoteProvider::new(note_service)))
    .with_search_provider(Arc::new(RepositoryFileSearchProvider::new(
        repository.clone(),
    )))
    .with_blob_reader(storage)
    .with_auth(auth_service)
}

async fn shutdown_signal() {
    use tokio::signal;

    #[cfg(unix)]
    {
        use tokio::signal::unix::SignalKind;
        #[allow(clippy::expect_used)]
        let mut sigterm = signal::unix::signal(SignalKind::terminate())
            .expect("failed to install SIGTERM handler");

        tokio::select! {
            _ = signal::ctrl_c() => {},
            _ = sigterm.recv() => {},
        }
    }

    #[cfg(not(unix))]
    {
        let _ = signal::ctrl_c().await;
    }
}

fn warn_if_insecure_bind(settings: &DaemonSettings) {
    use std::net::IpAddr;
    let is_unspecified = match settings.bind {
        IpAddr::V4(v4) => v4.is_unspecified(),
        IpAddr::V6(v6) => v6.is_unspecified(),
    };
    if is_unspecified && settings.public_url.is_none() && !settings.trust_forwarded {
        tracing::warn!(
            "server is bound to {} without public_url set — \
             remote clients will bypass authentication (local-mode). \
             Set public_url to enable global authentication mode.",
            settings.bind
        );
    }
}

/// Loads settings, wires services, and runs the HTTP server until shutdown.
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
        storage.clone(),
        state.corrupt_file_count.clone(),
    );

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

            // Clean up abandoned upload sessions
            let max_age = std::time::Duration::from_secs(24 * 60 * 60);
            let cleaned = upload_session_manager
                .cleanup_expired_with_disk(max_age, &upload_temp_dir)
                .await;
            if cleaned > 0 {
                tracing::info!("cleanup: removed {cleaned} abandoned upload sessions");
            }

            // Purge expired trash
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

    spawn_terminal_cleanup(state.terminal_manager.clone());

    let router = build_router(state);

    tracing::info!("tsspd listening on http://{address}");
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .map_err(|error| format!("server failed: {error}"))?;

    tracing::info!("tsspd stopped cleanly");
    Ok(())
}
