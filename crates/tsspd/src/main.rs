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
use tssp_adapter_system::SystemClock;
use tsspd::{
    bind_error_message, build_router, DaemonConfig, HttpState, RepositoryMetadataStatsProvider,
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
    match run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
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
    let metadata_path = cli.data_dir.join("metadata.sqlite3");
    let repository = SqliteFileRepository::open(&metadata_path)
        .map_err(|error| format!("could not initialize metadata store: {error}"))?;
    let _storage = FilesystemBlobStore::new(cli.data_dir.join("storage"))
        .map_err(|error| format!("could not initialize blob storage: {error}"))?;
    let stats_provider = RepositoryMetadataStatsProvider::new(repository, SystemClock);

    let address = config.socket_addr();
    let listener = TcpListener::bind(address)
        .await
        .map_err(|error| bind_error_message(address, &error))?;
    let router = build_router(HttpState::with_stats_provider(
        Instant::now(),
        Arc::new(stats_provider),
    ));

    println!("tsspd listening on http://{address}");
    axum::serve(listener, router)
        .await
        .map_err(|error| format!("server failed: {error}"))
}
