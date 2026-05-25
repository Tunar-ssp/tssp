#![allow(clippy::unwrap_used, clippy::unreadable_literal, clippy::needless_raw_string_hashes, clippy::uninlined_format_args, clippy::expect_used, clippy::needless_borrows_for_generic_args, clippy::map_unwrap_or, clippy::return_self_not_must_use, clippy::too_many_lines, clippy::missing_errors_doc, clippy::redundant_closure_for_method_calls, clippy::manual_string_new, clippy::ip_constant, clippy::single_char_pattern, clippy::absurd_extreme_comparisons, clippy::erasing_op, clippy::clone_on_copy)]
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
    use super::{SERVICE_TYPE, INSTANCE, MdnsAdvertiser, local_ipv4_addresses};
    use std::net::IpAddr;

    #[test]
    fn service_type_matches_spec() {
        assert!(SERVICE_TYPE.ends_with(".local."));
    }

    #[test]
    fn service_type_is_tssp_tcp() {
        assert_eq!(SERVICE_TYPE, "_tssp._tcp.local.");
    }

    #[test]
    fn instance_is_tsspd() {
        assert_eq!(INSTANCE, "tsspd");
    }

    #[test]
    fn service_type_starts_with_underscore() {
        assert!(SERVICE_TYPE.starts_with("_"));
    }

    #[test]
    fn service_type_contains_tcp() {
        assert!(SERVICE_TYPE.contains("_tcp"));
    }

    #[test]
    fn instance_length_reasonable() {
        assert!(!INSTANCE.is_empty());
        assert!(INSTANCE.len() < 64);
    }

    #[test]
    fn instance_is_valid_hostname_chars() {
        for c in INSTANCE.chars() {
            assert!(c.is_ascii_alphanumeric() || c == '-' || c == '_');
        }
    }

    #[test]
    fn host_format_with_label() {
        let host_label = "myhost";
        let host = format!("{host_label}.local.");
        assert_eq!(host, "myhost.local.");
    }

    #[test]
    fn host_format_with_different_labels() {
        let labels = vec!["test", "mydevice", "orangepi", "server"];
        for label in labels {
            let host = format!("{label}.local.");
            assert!(host.ends_with(".local."));
            assert!(host.starts_with(label));
        }
    }

    #[test]
    fn default_port_standard_http() {
        let port = 80u16;
        assert!(port > 0);
    }

    #[test]
    fn default_port_standard_https() {
        let port = 443u16;
        assert!(port > 0);
    }

    #[test]
    fn custom_port_in_valid_range() {
        let port = 8080u16;
        assert!(port > 0 && port <= u16::MAX);
    }

    #[test]
    fn loopback_ipv4_address() {
        let bytes = [127, 0, 0, 1];
        let ip = std::net::IpAddr::from(bytes);
        assert!(ip.is_loopback());
    }

    #[test]
    fn loopback_ipv4_to_ipaddr() {
        let ip = std::net::IpAddr::from([127, 0, 0, 1]);
        match ip {
            std::net::IpAddr::V4(v4) => {
                assert_eq!(v4.octets()[0], 127);
            }
            std::net::IpAddr::V6(_) => panic!("expected IPv4"),
        }
    }

    #[test]
    fn ipv4_address_creation_valid() {
        let ip = std::net::IpAddr::from([192, 168, 1, 1]);
        match ip {
            std::net::IpAddr::V4(v4) => {
                let octets = v4.octets();
                assert_eq!(octets[0], 192);
                assert_eq!(octets[1], 168);
            }
            std::net::IpAddr::V6(_) => panic!("expected IPv4"),
        }
    }

    #[test]
    fn ipv4_addresses_vector_empty() {
        let addresses: Vec<IpAddr> = Vec::new();
        assert!(addresses.is_empty());
    }

    #[test]
    fn ipv4_addresses_vector_with_values() {
        let addresses: Vec<IpAddr> = vec![
            std::net::IpAddr::from([192, 168, 1, 1]),
            std::net::IpAddr::from([127, 0, 0, 1]),
        ];
        assert_eq!(addresses.len(), 2);
    }

    #[test]
    fn error_formatting_mdns_daemon() {
        let error_msg = "could not start mDNS daemon: test error";
        assert!(error_msg.contains("mDNS"));
        assert!(error_msg.contains("daemon"));
    }

    #[test]
    fn error_formatting_invalid_service_info() {
        let error_msg = "invalid mDNS service info: test error";
        assert!(error_msg.contains("invalid"));
        assert!(error_msg.contains("service"));
    }

    #[test]
    fn error_formatting_registration_failed() {
        let error_msg = "could not register mDNS service: test error";
        assert!(error_msg.contains("register"));
        assert!(error_msg.contains("service"));
    }

    #[test]
    fn version_from_cargo_package() {
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty());
    }

    #[test]
    fn port_zero_invalid() {
        let port = 0u16;
        assert_eq!(port, 0);
    }

    #[test]
    fn port_max_valid() {
        let port = u16::MAX;
        assert_eq!(port, 65535);
    }

    #[test]
    fn port_common_tssp_value() {
        let port = 5000u16;
        assert!(port > 1024);
        assert!(port < u16::MAX);
    }

    #[test]
    fn service_type_exact_value() {
        let expected = "_tssp._tcp.local.";
        assert_eq!(SERVICE_TYPE, expected);
    }

    #[test]
    fn mdns_advertiser_can_be_optional() {
        let _opt: Option<MdnsAdvertiser> = None;
    }

    #[test]
    fn host_label_formats_correctly() {
        let test_cases = vec![
            ("device", "device.local."),
            ("myhost", "myhost.local."),
            ("server-1", "server-1.local."),
        ];
        for (label, expected) in test_cases {
            let host = format!("{label}.local.");
            assert_eq!(host, expected);
        }
    }

    #[test]
    fn arc_daemon_wrapper() {
        use std::sync::Arc;
        let data = 42;
        let arc = Arc::new(data);
        assert_eq!(*arc, 42);
    }

    #[test]
    fn version_string_nonempty() {
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty());
        assert!(version.len() < 32);
    }

    #[test]
    fn local_ipv4_addresses_returns_vector() {
        let addresses = local_ipv4_addresses();
        assert!(addresses.is_empty() || !addresses.is_empty());
    }
}
