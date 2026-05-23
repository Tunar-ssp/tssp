//! `tsspd` binary entry point.

use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Instant;

use clap::Parser;
use tokio::net::TcpListener;
use tssp_adapter_fs::FilesystemBlobStore;
use tssp_adapter_sqlite::{SqliteFileRepository, SqliteSessionRepository};
use tssp_adapter_system::SystemClock;
use tssp_adapter_system::UuidV7FileIdGenerator;
use tssp_app::{
    DeleteFileService, NoteService, PinService, SessionService, TagService, UploadService,
};
use tssp_ports::Clock;
use tsspd::workspaces::WorkspaceStore;
use tsspd::{
    auth::{AuthService, AuthStore, DeviceStore, UserStore},
    bind_error_message, build_router, run_startup_integrity_scan, spawn_advertisement,
    ApplicationFileDeleteProvider, ApplicationFilePinProvider, ApplicationFileTagProvider,
    ApplicationFileUploadProvider, ApplicationNoteProvider, ApplicationSessionProvider,
    CliOverrides, DaemonSettings, HttpState, PublicUrlBuilder, RepositoryFileSearchProvider,
    RepositoryMetadataStatsProvider,
};

/// Backend daemon for TSSP.
#[derive(Debug, Parser)]
#[command(name = "tsspd")]
#[command(version, about = "TSSP backend daemon")]
struct Cli {
    /// IP address to bind.
    #[arg(long, env = "TSSPD_BIND")]
    bind: Option<IpAddr>,

    /// TCP port to listen on.
    #[arg(long, env = "TSSPD_PORT")]
    port: Option<u16>,

    /// Directory for metadata and blob storage.
    #[arg(long, value_name = "PATH", env = "TSSPD_DATA_DIR")]
    data_dir: Option<PathBuf>,

    /// Public base URL (e.g. `https://cloud.example.com`).
    #[arg(long, env = "TSSPD_PUBLIC_URL")]
    public_url: Option<String>,

    /// Validate configuration and exit.
    #[arg(long)]
    check_config: bool,

    /// Trust `X-Forwarded-For` for remote/local auth decisions (behind a reverse proxy).
    #[arg(long, env = "TSSPD_TRUST_FORWARDED")]
    trust_forwarded: Option<bool>,

    /// Advertise via mDNS (`_tssp._tcp.local`).
    #[arg(long, env = "TSSPD_MDNS")]
    mdns: Option<bool>,

    /// Enable Prometheus `/metrics` endpoint.
    #[arg(long, env = "TSSPD_METRICS")]
    metrics: Option<bool>,

    /// Serve the embedded web dashboard.
    #[arg(long, env = "TSSPD_WEB")]
    web: Option<bool>,
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    match run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
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

fn run_integrity_check(db_path: &std::path::Path) -> Result<(), String> {
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

fn cleanup_temp_uploads(temp_dir: &std::path::Path) {
    if !temp_dir.exists() {
        return;
    }

    let mut removed = 0;
    if let Ok(entries) = std::fs::read_dir(temp_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    if let Err(e) = std::fs::remove_file(entry.path()) {
                        tracing::warn!(
                            "startup: could not remove temp file {}: {e}",
                            entry.path().display()
                        );
                    } else {
                        removed += 1;
                    }
                }
            }
        }
    }

    if removed > 0 {
        tracing::info!("startup: cleaned up {removed} orphaned temp uploads");
    }
}

struct RuntimePaths {
    upload_temp_dir: PathBuf,
    metadata_path: PathBuf,
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

fn open_repository(metadata_path: &Path) -> Result<Arc<SqliteFileRepository>, String> {
    SqliteFileRepository::open(metadata_path)
        .map(Arc::new)
        .map_err(|error| format!("could not initialize metadata store: {error}"))
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
    metadata_path: &Path,
    upload_temp_dir: &Path,
) -> Result<SessionService<SqliteSessionRepository>, String> {
    let session_connection = rusqlite::Connection::open(metadata_path)
        .map_err(|error| format!("could not open session database connection: {error}"))?;
    let session_repository =
        SqliteSessionRepository::new(Arc::new(std::sync::Mutex::new(session_connection)));
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
    metadata_path: &Path,
    settings: &DaemonSettings,
) -> Result<AuthService, String> {
    let auth_store = Arc::new(
        AuthStore::open(metadata_path)
            .map_err(|error| format!("could not initialize auth store: {error}"))?,
    );
    let user_store = Arc::new(
        UserStore::open(metadata_path)
            .map_err(|error| format!("could not initialize user store: {error}"))?,
    );
    let device_store = Arc::new(
        DeviceStore::open(metadata_path)
            .map_err(|error| format!("could not initialize device store: {error}"))?,
    );
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

fn build_http_state(
    settings: &Arc<DaemonSettings>,
    paths: RuntimePaths,
    repository: Arc<SqliteFileRepository>,
    storage: Arc<FilesystemBlobStore>,
    session_service: SessionService<SqliteSessionRepository>,
    auth_service: AuthService,
    corrupt_file_count: u64,
) -> Result<HttpState, String> {
    let stats_provider = RepositoryMetadataStatsProvider::new(repository.clone(), SystemClock);
    let upload_service = UploadService::new(
        storage.clone(),
        repository.clone(),
        UuidV7FileIdGenerator,
        SystemClock,
    );
    let delete_service = DeleteFileService::new(storage.clone(), repository.clone());
    let tag_service = TagService::new(repository.clone());
    let pin_service = PinService::new(repository.clone());
    let note_service = NoteService::new(repository.clone(), SystemClock, UuidV7FileIdGenerator);
    let workspace_store = Arc::new(
        WorkspaceStore::open(&paths.metadata_path)
            .map_err(|error| format!("could not initialize workspace store: {error}"))?,
    );

    Ok(HttpState::new(
        Instant::now(),
        paths.upload_temp_dir,
        settings.clone(),
        PublicUrlBuilder::from_settings(settings),
        corrupt_file_count,
    )
    .with_workspaces(workspace_store)
    .with_stats_provider(Arc::new(stats_provider))
    .with_upload_provider(Arc::new(ApplicationFileUploadProvider::new(upload_service)))
    .with_delete_provider(Arc::new(ApplicationFileDeleteProvider::new(delete_service)))
    .with_tag_provider(Arc::new(ApplicationFileTagProvider::new(tag_service)))
    .with_pin_provider(Arc::new(ApplicationFilePinProvider::new(pin_service)))
    .with_session_provider(Arc::new(ApplicationSessionProvider::new(
        session_service,
        SystemClock,
    )))
    .with_note_provider(Arc::new(ApplicationNoteProvider::new(note_service)))
    .with_search_provider(Arc::new(RepositoryFileSearchProvider::new(repository)))
    .with_blob_reader(storage)
    .with_auth(auth_service))
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

async fn run(cli: Cli) -> Result<(), String> {
    let settings = load_settings(&cli)?;

    if cli.check_config {
        print_check_config(&settings);
        return Ok(());
    }

    settings.log_effective();
    let paths = prepare_runtime_paths(&settings)?;
    let repository = open_repository(&paths.metadata_path)?;
    run_integrity_check(&paths.metadata_path)
        .map_err(|error| format!("database integrity check failed: {error}"))?;
    let storage = open_storage(&settings)?;
    let corrupt_file_count = run_startup_integrity_scan(&repository, &storage);
    let session_service = start_session_service(&paths.metadata_path, &paths.upload_temp_dir)?;
    let auth_service = start_auth_service(&paths.metadata_path, &settings)?;

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
        paths,
        repository,
        storage,
        session_service,
        auth_service,
        corrupt_file_count,
    )?;

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

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::{run, Cli};

    #[tokio::test]
    async fn run_check_config_exits_before_storage_setup() {
        let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let data_dir = temp.path().join("not-created");
        let cli = cli(data_dir.clone(), true);

        let result = run(cli).await;

        assert_eq!(result, Ok(()));
        assert!(
            !data_dir.join("metadata.sqlite3").exists(),
            "check-config must not initialize storage"
        );
    }

    #[tokio::test]
    async fn run_reports_data_directory_creation_failure() {
        let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let data_dir = temp.path().join("data-file");
        std::fs::write(&data_dir, b"not a directory")
            .unwrap_or_else(|error| panic!("write failed: {error}"));
        let cli = cli(data_dir, false);

        let result = run(cli).await;

        let Err(err) = result else {
            panic!("expected data directory setup to fail");
        };
        assert!(
            err.contains("data directory") || err.contains("create data directory"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn run_fails_on_bad_bind() {
        let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let mut cli_args = cli(temp.path().to_path_buf(), false);
        cli_args.bind = Some(IpAddr::V4(Ipv4Addr::LOCALHOST));
        cli_args.port = Some(80);
        let result = run(cli_args).await;
        assert!(matches!(result, Err(message) if message.contains("could not bind")));
    }

    #[tokio::test]
    async fn run_check_config_does_not_create_data_directory() {
        let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let data_dir = temp.path().join("new-data-dir");
        let cli = cli(data_dir.clone(), true);

        let result = run(cli).await;

        assert_eq!(result, Ok(()));
        assert!(
            !data_dir.join("metadata.sqlite3").exists(),
            "check-config must not initialize storage"
        );
    }

    fn cli(data_dir: std::path::PathBuf, check_config: bool) -> Cli {
        Cli {
            bind: Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
            port: Some(18421),
            data_dir: Some(data_dir),
            public_url: None,
            check_config,
            trust_forwarded: Some(false),
            mdns: Some(false),
            metrics: Some(true),
            web: Some(true),
        }
    }
}
