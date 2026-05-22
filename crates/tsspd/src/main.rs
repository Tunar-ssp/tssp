//! `tsspd` binary entry point.

use std::net::{IpAddr, Ipv4Addr};
use std::process::ExitCode;
use std::time::Instant;

use clap::Parser;
use tokio::net::TcpListener;
use tsspd::{bind_error_message, build_router, DaemonConfig, HttpState};

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
        println!("configuration ok: {}", config.socket_addr());
        return Ok(());
    }

    let address = config.socket_addr();
    let listener = TcpListener::bind(address)
        .await
        .map_err(|error| bind_error_message(address, &error))?;
    let router = build_router(HttpState::new(Instant::now()));

    println!("tsspd listening on http://{address}");
    axum::serve(listener, router)
        .await
        .map_err(|error| format!("server failed: {error}"))
}
