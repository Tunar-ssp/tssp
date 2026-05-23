//! Absolute URL generation for sessions and API responses.

use std::net::IpAddr;

use crate::settings::DaemonSettings;

/// Builds externally reachable URLs for QR codes and session links.
#[derive(Debug, Clone)]
pub struct PublicUrlBuilder {
    base: String,
}

impl PublicUrlBuilder {
    /// Creates a builder from effective daemon settings.
    #[must_use]
    pub fn from_settings(settings: &DaemonSettings) -> Self {
        let base = settings
            .public_url
            .as_deref()
            .map(trim_trailing_slash)
            .unwrap_or_else(|| default_base_url(settings.bind, settings.port));
        Self { base }
    }

    /// Base URL without trailing slash.
    #[must_use]
    pub fn base(&self) -> &str {
        &self.base
    }

    /// Public download URL for a send session token.
    #[must_use]
    pub fn send_download_url(&self, token: &str) -> String {
        format!("{}/s/{}", self.base, token)
    }

    /// Public upload URL for a receive session token.
    #[must_use]
    pub fn receive_upload_url(&self, token: &str) -> String {
        format!("{}/u/{}", self.base, token)
    }

    /// API status URL.
    #[must_use]
    pub fn status_url(&self) -> String {
        format!("{}/api/v1/status", self.base)
    }

    /// Public file download URL for a public link token.
    #[must_use]
    pub fn public_file_url(&self, token: &str) -> String {
        format!("{}/p/{}", self.base, token)
    }
}

fn trim_trailing_slash(url: &str) -> String {
    url.trim_end_matches('/').to_owned()
}

fn default_base_url(bind: IpAddr, port: u16) -> String {
    let host = match bind {
        IpAddr::V4(v4) if v4.is_unspecified() => "127.0.0.1".to_owned(),
        IpAddr::V6(v6) if v6.is_unspecified() => "::1".to_owned(),
        other => other.to_string(),
    };
    format!("http://{host}:{port}")
}

#[cfg(test)]
mod tests {
    use super::{default_base_url, PublicUrlBuilder};
    use crate::settings::DaemonSettings;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn uses_public_url_when_set() {
        let mut settings = DaemonSettings::default();
        settings.public_url = Some("https://cloud.example.com/".to_owned());
        let urls = PublicUrlBuilder::from_settings(&settings);
        assert_eq!(urls.base(), "https://cloud.example.com");
        assert_eq!(
            urls.send_download_url("abc"),
            "https://cloud.example.com/s/abc"
        );
    }

    #[test]
    fn unspecified_bind_uses_loopback() {
        let base = default_base_url(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8421);
        assert_eq!(base, "http://127.0.0.1:8421");
    }
}
