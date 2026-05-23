//! Daemon startup configuration helpers.

use std::net::{IpAddr, SocketAddr};

/// Legacy socket config (prefer [`DaemonSettings`](crate::DaemonSettings)).
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct DaemonConfig {
    /// IP address to bind.
    pub bind: IpAddr,
    /// TCP port to listen on.
    pub port: u16,
}

impl DaemonConfig {
    /// Returns the socket address represented by the config.
    #[must_use]
    #[allow(dead_code)]
    pub const fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.bind, self.port)
    }
}

/// Maps startup bind failures to concise user-facing messages.
#[must_use]
pub fn bind_error_message(address: SocketAddr, error: &std::io::Error) -> String {
    if error.kind() == std::io::ErrorKind::AddrInUse {
        return format!(
            "port {} is already in use; choose another port with --port",
            address.port()
        );
    }

    format!("could not bind {address}: {error}")
}
