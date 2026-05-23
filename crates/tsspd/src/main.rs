//! `tsspd` binary entry point.

use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Instant;

use clap::Parser;
use tokio::net::TcpListener;
use tssp_adapter_fs::FilesystemBlobStore;
use tssp_adapter_sqlite::SqliteFileRepository;
use tssp_adapter_system::{SystemClock, UuidV7FileIdGenerator};
use tssp_app::{DeleteFileService, PinService, TagService, UploadService};
use tsspd::{
    bind_error_message, build_router, ApplicationFileDeleteProvider, ApplicationFilePinProvider,
    ApplicationFileTagProvider, ApplicationFileUploadProvider, DaemonConfig, HttpState,
    RepositoryFileSearchProvider, RepositoryMetadataStatsProvider,
};

/// Backend daemon for TSSP.
#[derive(Debug, Parser)]
#[command(name = "tsspd")]
#[command(version, about = "TSSP backend daemon")]
struct Cli {
    /// IP address to bind.
    #[arg(long, default_value_t = IpAddr::V4(Ipv4Addr::LOCALHOST), env = "TSSPD_BIND")]
    bind: IpAddr,

    /// TCP port to listen on.
    #[arg(long, default_value_t = 8421, env = "TSSPD_PORT")]
    port: u16,

    /// Directory for metadata and blob storage.
    #[arg(
        long,
        value_name = "PATH",
        default_value = "data",
        env = "TSSPD_DATA_DIR"
    )]
    data_dir: PathBuf,

    /// Validate configuration and exit.
    #[arg(long)]
    check_config: bool,
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

async fn shutdown_signal() {
    use tokio::signal;

    #[cfg(unix)]
    {
        use tokio::signal::unix::SignalKind;
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
    let config = DaemonConfig {
        bind: cli.bind,
        port: cli.port,
    };

    if cli.check_config {
        println!(
            "configuration ok: {}, data dir {}",
            config.socket_addr(),
            cli.data_dir.display()
        );
        return Ok(());
    }

    std::fs::create_dir_all(&cli.data_dir)
        .map_err(|error| format!("could not create data directory: {error}"))?;
    let http_upload_temp_dir = cli.data_dir.join("http-upload-tmp");
    std::fs::create_dir_all(&http_upload_temp_dir)
        .map_err(|error| format!("could not create upload temp directory: {error}"))?;
    let metadata_path = cli.data_dir.join("metadata.sqlite3");
    let repository = Arc::new(
        SqliteFileRepository::open(&metadata_path)
            .map_err(|error| format!("could not initialize metadata store: {error}"))?,
    );
    let storage = Arc::new(
        FilesystemBlobStore::new(cli.data_dir.join("storage"))
            .map_err(|error| format!("could not initialize blob storage: {error}"))?,
    );
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
    let upload_provider = ApplicationFileUploadProvider::new(upload_service);
    let delete_provider = ApplicationFileDeleteProvider::new(delete_service);
    let tag_provider = ApplicationFileTagProvider::new(tag_service);
    let pin_provider = ApplicationFilePinProvider::new(pin_service);
    let search_provider = RepositoryFileSearchProvider::new(repository.clone());

    let address = config.socket_addr();
    let listener = TcpListener::bind(address)
        .await
        .map_err(|error| bind_error_message(address, &error))?;
    let state = HttpState::new(Instant::now(), http_upload_temp_dir)
        .with_stats_provider(Arc::new(stats_provider))
        .with_upload_provider(Arc::new(upload_provider))
        .with_delete_provider(Arc::new(delete_provider))
        .with_tag_provider(Arc::new(tag_provider))
        .with_pin_provider(Arc::new(pin_provider))
        .with_search_provider(Arc::new(search_provider))
        .with_blob_reader(storage);
    let router = build_router(state);

    tracing::info!("tsspd listening on http://{address}");
    axum::serve(listener, router)
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
        // Binding to a privileged port will fail immediately
        cli_args.port = 80;
        let result = run(cli_args).await;
        assert!(matches!(result, Err(message) if message.contains("could not bind")));
    }

    #[tokio::test]
    async fn run_check_config_does_not_create_data_directory() {
        let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let data_dir = temp.path().join("new-data-dir");
        assert!(!data_dir.exists());

        let cli_args = cli(data_dir.clone(), true);
        let result = run(cli_args).await;

        assert!(result.is_ok());
        assert!(!data_dir.exists());
    }

    #[tokio::test]
    async fn run_reports_upload_temp_directory_creation_failure() {
        let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir failed: {error}"));
        let data_dir = temp.path().join("data");
        std::fs::create_dir(&data_dir)
            .unwrap_or_else(|error| panic!("mkdir failed: {error}"));

        // Create a file where the upload temp dir would be created
        let upload_temp = data_dir.join("http-upload-tmp");
        std::fs::write(&upload_temp, b"not a directory")
            .unwrap_or_else(|error| panic!("write failed: {error}"));

        let cli_args = cli(data_dir, false);
        let result = run(cli_args).await;

        assert!(matches!(result, Err(message) if message.contains("upload temp directory")));
    }

    fn cli(data_dir: std::path::PathBuf, check_config: bool) -> Cli {
        Cli {
            bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 0,
            data_dir,
            check_config,
        }
    }
}
