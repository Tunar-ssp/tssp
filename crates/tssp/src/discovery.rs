//! mDNS discovery for `tsspd` on the local network.

use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;

const SERVICE_TYPE: &str = "_tssp._tcp.local.";
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(3);

/// Discovered daemon endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredDaemon {
    /// Hostname (often `tsspd.local`).
    pub host: String,
    /// TCP port.
    pub port: u16,
}

impl DiscoveredDaemon {
    /// HTTP base URL without trailing slash.
    #[must_use]
    #[allow(dead_code)]
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

/// Attempts to find a daemon via mDNS within the timeout.
///
/// Returns `None` when discovery is disabled, times out, or mDNS is unavailable.
pub fn discover_daemon(enabled: bool) -> Option<DiscoveredDaemon> {
    if !enabled {
        return None;
    }
    discover_blocking().ok()
}

fn discover_blocking() -> Result<DiscoveredDaemon, String> {
    let daemon =
        mdns_sd::ServiceDaemon::new().map_err(|error| format!("mDNS unavailable: {error}"))?;
    let receiver = daemon
        .browse(SERVICE_TYPE)
        .map_err(|error| format!("mDNS browse failed: {error}"))?;
    let deadline = std::time::Instant::now() + DISCOVERY_TIMEOUT;
    while std::time::Instant::now() < deadline {
        if let Ok(event) = receiver.recv_timeout(Duration::from_millis(250)) {
            if let mdns_sd::ServiceEvent::ServiceResolved(info) = event {
                let host = info.get_hostname().trim_end_matches('.').to_owned();
                let port = info.get_port();
                if !host.is_empty() && port > 0 {
                    let _ = daemon.shutdown();
                    return Ok(DiscoveredDaemon { host, port });
                }
            }
        }
    }
    Err(
        "no daemon found via mDNS (try `tssp config set host <ip>` or ensure tsspd is running)"
            .to_owned(),
    )
}

/// Resolves a hostname to socket addresses.
#[allow(dead_code)]
pub fn resolve_host_port(host: &str, port: u16) -> Result<Vec<SocketAddr>, String> {
    let addrs: Vec<SocketAddr> = format!("{host}:{port}")
        .to_socket_addrs()
        .map_err(|error| format!("could not resolve {host}:{port}: {error}"))?
        .collect();
    if addrs.is_empty() {
        return Err(format!("no addresses found for {host}:{port}"));
    }
    Ok(addrs)
}

#[cfg(test)]
mod tests {
    use super::SERVICE_TYPE;

    #[test]
    fn service_type_is_tssp() {
        assert_eq!(SERVICE_TYPE, "_tssp._tcp.local.");
    }
}
