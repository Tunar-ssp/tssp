#![allow(clippy::unwrap_used, clippy::unreadable_literal, clippy::needless_raw_string_hashes, clippy::uninlined_format_args, clippy::expect_used, clippy::needless_borrows_for_generic_args, clippy::map_unwrap_or, clippy::return_self_not_must_use, clippy::too_many_lines, clippy::missing_errors_doc, clippy::redundant_closure_for_method_calls, clippy::manual_string_new, clippy::ip_constant, clippy::single_char_pattern, clippy::absurd_extreme_comparisons, clippy::erasing_op, clippy::clone_on_copy)]
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
///
/// SECURITY: Only trusts X-Forwarded-For if the peer IP is in the trusted proxy list.
/// This prevents header spoofing from arbitrary clients.
#[must_use]
pub fn client_ip(
    peer: IpAddr,
    forwarded_for: Option<&str>,
    trust_forwarded: bool,
    trusted_proxies: &[IpAddr],
) -> IpAddr {
    // Only trust X-Forwarded-For if:
    // 1. trust_forwarded is enabled AND
    // 2. trusted_proxies is configured (not empty) AND
    // 3. peer IP is in the trusted proxy list
    if !trust_forwarded {
        return peer;
    }

    // If trusted_proxies is empty, never trust the header (fail-safe)
    if trusted_proxies.is_empty() {
        return peer;
    }

    // If peer is not a trusted proxy, ignore the header completely
    if !trusted_proxies.contains(&peer) {
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
    fn forwarded_for_used_when_peer_is_trusted_proxy() {
        let peer = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let trusted = vec![peer];
        let ip = client_ip(peer, Some("203.0.113.5, 192.168.1.1"), true, &trusted);
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(203, 0, 113, 5)));
    }

    #[test]
    fn forwarded_for_ignored_when_peer_not_trusted_proxy() {
        let peer = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)); // Untrusted IP
        let trusted = vec![IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))]; // Only localhost trusted
        let ip = client_ip(peer, Some("203.0.113.5"), true, &trusted);
        // Should return peer, not the forwarded IP
        assert_eq!(ip, peer);
    }

    #[test]
    fn forwarded_for_ignored_when_flag_disabled() {
        let peer = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let trusted = vec![peer];
        let ip = client_ip(peer, Some("203.0.113.5"), false, &trusted);
        assert_eq!(ip, peer);
    }

    #[test]
    fn empty_trusted_proxy_list_rejects_forwarded_for() {
        let peer = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1));
        let ip = client_ip(peer, Some("203.0.113.5"), true, &[]);
        // Empty trusted list means we never trust X-Forwarded-For (fail-safe)
        assert_eq!(ip, peer);
    }
}
