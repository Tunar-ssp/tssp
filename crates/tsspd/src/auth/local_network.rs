//! Client IP classification for dual-mode access.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Returns true when the client is on a trusted local network.
#[must_use]
pub fn is_local_client(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_private_ipv4(v4) || v4.is_loopback() || v4.is_link_local(),
        IpAddr::V6(v6) => v6.is_loopback() || is_unique_local_ipv6(v6) || is_link_local_ipv6(v6),
    }
}

fn is_private_ipv4(addr: Ipv4Addr) -> bool {
    addr.is_private() || addr.is_link_local()
}

fn is_unique_local_ipv6(addr: Ipv6Addr) -> bool {
    (addr.segments()[0] & 0xfe00) == 0xfc00
}

fn is_link_local_ipv6(addr: Ipv6Addr) -> bool {
    (addr.segments()[0] & 0xffc0) == 0xfe80
}

/// Resolves the client IP from the TCP peer and optional `X-Forwarded-For`.
#[must_use]
pub fn client_ip(peer: IpAddr, forwarded_for: Option<&str>, trust_forwarded: bool) -> IpAddr {
    if !trust_forwarded {
        return peer;
    }
    let Some(header) = forwarded_for else {
        return peer;
    };
    parse_forwarded_for(header).unwrap_or(peer)
}

fn parse_forwarded_for(header: &str) -> Option<IpAddr> {
    header.split(',').next().map(str::trim).and_then(|value| {
        if let Some(stripped) = value.strip_prefix('[').and_then(|v| v.strip_suffix(']')) {
            stripped.parse().ok()
        } else {
            value.parse().ok()
        }
    })
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    use super::{client_ip, is_local_client};

    #[test]
    fn private_ipv4_is_local() {
        assert!(is_local_client(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10))));
        assert!(is_local_client(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert!(is_local_client(IpAddr::V4(Ipv4Addr::LOCALHOST)));
    }

    #[test]
    fn public_ipv4_is_not_local() {
        assert!(!is_local_client(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
    }

    #[test]
    fn loopback_ipv6_is_local() {
        assert!(is_local_client(IpAddr::V6(Ipv6Addr::LOCALHOST)));
    }

    #[test]
    fn forwarded_for_used_when_trusted() {
        let peer = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let ip = client_ip(peer, Some("203.0.113.5, 192.168.1.1"), true);
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(203, 0, 113, 5)));
    }

    #[test]
    fn forwarded_for_ignored_when_untrusted() {
        let peer = IpAddr::V4(Ipv4Addr::new(192, 168, 0, 2));
        let ip = client_ip(peer, Some("203.0.113.5"), false);
        assert_eq!(ip, peer);
    }
}
