//! `tsspd` binary entry point.

mod runner;

use std::net::IpAddr;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use runner::run;

/// Backend daemon for TSSP.
#[derive(Debug, Parser)]
#[command(name = "tsspd")]
#[command(version, about = "TSSP backend daemon")]
pub(crate) struct Cli {
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

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::Cli;
    use crate::runner::run;

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
        cli_args.bind = Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)));
        cli_args.port = Some(18421);
        let result = run(cli_args).await;
        assert!(result.is_err(), "expected bind failure, got {result:?}");
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
