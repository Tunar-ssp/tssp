//! mDNS advertisement for `_tssp._tcp.local`.

use std::net::IpAddr;
use std::sync::Arc;

use mdns_sd::{ServiceDaemon, ServiceInfo};
use tracing::{info, warn};

const SERVICE_TYPE: &str = "_tssp._tcp.local.";
const INSTANCE: &str = "tsspd";

/// Handle to a background mDNS registration task.
pub struct MdnsAdvertiser {
    _daemon: Arc<ServiceDaemon>,
}

impl MdnsAdvertiser {
    /// Registers this daemon on the local network.
    ///
    /// # Errors
    ///
    /// Returns an error when mDNS registration fails.
    pub fn advertise(port: u16, host_label: &str) -> Result<Self, String> {
        let daemon = ServiceDaemon::new()
            .map_err(|error| format!("could not start mDNS daemon: {error}"))?;
        let host = format!("{host_label}.local.");
        let mut addresses: Vec<IpAddr> = local_ipv4_addresses();
        if addresses.is_empty() {
            addresses.push(IpAddr::from([127, 0, 0, 1]));
        }
        let properties = [("version", env!("CARGO_PKG_VERSION"))];
        let service = ServiceInfo::new(
            SERVICE_TYPE,
            INSTANCE,
            &host,
            &addresses[..],
            port,
            &properties[..],
        )
        .map_err(|error| format!("invalid mDNS service info: {error}"))?;
        daemon
            .register(service)
            .map_err(|error| format!("could not register mDNS service: {error}"))?;
        info!(
            port,
            host = %host,
            "mDNS: advertised {SERVICE_TYPE} as {INSTANCE}"
        );
        Ok(Self {
            _daemon: Arc::new(daemon),
        })
    }
}

/// Spawns mDNS advertisement; logs and continues on failure.
pub fn spawn_advertisement(port: u16) {
    tokio::spawn(async move {
        match MdnsAdvertiser::advertise(port, "tsspd") {
            Ok(_handle) => {
                std::future::pending::<()>().await;
            }
            Err(error) => {
                warn!("mDNS advertisement disabled: {error}");
            }
        }
    });
}

fn local_ipv4_addresses() -> Vec<IpAddr> {
    let mut addrs = Vec::new();
    if let Ok(ip) = local_ip_address::local_ip() {
        addrs.push(ip);
    }
    addrs
}

#[cfg(test)]
mod tests {
    use super::SERVICE_TYPE;

    #[test]
    fn service_type_matches_spec() {
        assert!(SERVICE_TYPE.ends_with(".local."));
    }
}
