//! `tsspd` binary entry point.

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Instant;

use clap::Parser;
use tokio::net::TcpListener;
use tssp_adapter_fs::FilesystemBlobStore;
use tssp_adapter_sqlite::{SqliteFileRepository, SqliteSessionRepository};
use tssp_adapter_system::SystemClock;
use tssp_adapter_system::UuidV7FileIdGenerator;
use tssp_app::{DeleteFileService, NoteService, PinService, SessionService, TagService, UploadService};
use tssp_ports::Clock;
use tsspd::{
    auth::{AuthService, AuthStore},
    bind_error_message, build_router, run_startup_integrity_scan, spawn_advertisement,
    ApplicationFileDeleteProvider,
    ApplicationFilePinProvider, ApplicationFileTagProvider, ApplicationFileUploadProvider,
    ApplicationNoteProvider, ApplicationSessionProvider, CliOverrides, DaemonSettings, HttpState,
    PublicUrlBuilder, RepositoryFileSearchProvider, RepositoryMetadataStatsProvider,
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
    let data_dir = cli
        .data_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from("data"));
    let settings = DaemonSettings::load(&data_dir, &cli_overrides(&cli))?;

    if cli.check_config {
        println!(
            "configuration ok: {}, data dir {}",
            settings.socket_addr(),
            settings.data_dir.display()
        );
        println!("config file: {}", settings.config_file_path().display());
        return Ok(());
    }

    settings.log_effective();

    std::fs::create_dir_all(&settings.data_dir)
        .map_err(|error| format!("could not create data directory: {error}"))?;

    if !settings.data_dir.is_dir() {
        return Err(format!(
            "data directory {} is not accessible",
            settings.data_dir.display()
        ));
    }

    let http_upload_temp_dir = settings.data_dir.join("http-upload-tmp");
    std::fs::create_dir_all(&http_upload_temp_dir)
        .map_err(|error| format!("could not create upload temp directory: {error}"))?;

    let metadata_path = settings.data_dir.join("metadata.sqlite3");
    let repository = Arc::new(
        SqliteFileRepository::open(&metadata_path)
            .map_err(|error| format!("could not initialize metadata store: {error}"))?,
    );

    run_integrity_check(&metadata_path)
        .map_err(|error| format!("database integrity check failed: {error}"))?;

    let storage = Arc::new(
        FilesystemBlobStore::new(settings.data_dir.join("storage"))
            .map_err(|error| format!("could not initialize blob storage: {error}"))?,
    );

    let corrupt_file_count =
        run_startup_integrity_scan(repository.clone(), storage.clone());

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
    let note_service = NoteService::new(
        repository.clone(),
        SystemClock,
        UuidV7FileIdGenerator,
    );
    let session_connection = rusqlite::Connection::open(&metadata_path)
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

    cleanup_temp_uploads(&http_upload_temp_dir);

    let auth_store = Arc::new(
        AuthStore::open(&metadata_path)
            .map_err(|error| format!("could not initialize auth store: {error}"))?,
    );
    let auth_service = AuthService::new(auth_store, settings.trust_forwarded);
    configure_auth_password(&auth_service)?;
    let now_secs = SystemClock.now().seconds();
    let removed_tokens = auth_service
        .cleanup_expired(now_secs)
        .map_err(|error| format!("auth token cleanup failed: {error}"))?;
    if removed_tokens > 0 {
        tracing::info!("startup: removed {removed_tokens} expired auth tokens");
    }

    let settings = Arc::new(settings);
    let public_urls = PublicUrlBuilder::from_settings(&settings);
    let address = settings.socket_addr();
    let listener = TcpListener::bind(address)
        .await
        .map_err(|error| bind_error_message(address, &error))?;

    if settings.mdns {
        spawn_advertisement(settings.port);
    }

    let state = HttpState::new(
        Instant::now(),
        http_upload_temp_dir,
        settings.clone(),
        public_urls,
        corrupt_file_count,
    )
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
    .with_search_provider(Arc::new(RepositoryFileSearchProvider::new(
        repository.clone(),
    )))
    .with_blob_reader(storage)
    .with_auth(auth_service);

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
        assert!(!data_dir.exists());
    }

    #[tokio::test]
    async fn run_reports_data_directory_creation_failure() {
        let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let data_dir = temp.path().join("data-file");
        std::fs::write(&data_dir, b"not a directory")
            .unwrap_or_else(|error| panic!("write failed: {error}"));
        let cli = cli(data_dir, false);

        let result = run(cli).await;

        assert!(matches!(result, Err(message) if message.contains("data directory")));
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
        assert!(!data_dir.exists());
    }

    fn cli(data_dir: std::path::PathBuf, check_config: bool) -> Cli {
        Cli {
            bind: Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
            port: Some(0),
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
