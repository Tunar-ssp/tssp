//! Layered daemon configuration (defaults → file → env → CLI).

use std::net::IpAddr;
use std::path::{Path, PathBuf};

use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

/// Effective daemon settings after merging all configuration layers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
#[serde(default)]
pub struct DaemonSettings {
    /// Bind address.
    pub bind: IpAddr,
    /// TCP port.
    pub port: u16,
    /// Data directory (metadata + blobs).
    pub data_dir: PathBuf,
    /// Public base URL for QR links and session URLs (no trailing slash).
    #[serde(default)]
    pub public_url: Option<String>,
    /// Trust `X-Forwarded-For` from reverse proxies.
    pub trust_forwarded: bool,
    /// Advertise the daemon via mDNS (`_tssp._tcp`).
    pub mdns: bool,
    /// Expose `/metrics` (Prometheus text).
    pub metrics: bool,
    /// Serve the embedded web dashboard.
    pub web: bool,
    /// Default session TTL in seconds.
    pub session_ttl_seconds: u64,
    /// Minimum free bytes before rejecting uploads (absolute floor).
    pub storage_reserve_bytes: u64,
    /// Minimum free percent of total disk before rejecting uploads.
    pub storage_reserve_percent: u64,
    /// Maximum single upload size in bytes (`0` = unlimited).
    pub max_upload_bytes: u64,
    /// Log filter string (passed to `tracing` env layer when set).
    #[serde(default)]
    pub log_level: Option<String>,
}

impl Default for DaemonSettings {
    fn default() -> Self {
        Self {
            bind: IpAddr::from([127, 0, 0, 1]),
            port: 8421,
            data_dir: PathBuf::from("data"),
            public_url: None,
            trust_forwarded: false,
            mdns: true,
            metrics: true,
            web: true,
            session_ttl_seconds: 86_400,
            storage_reserve_bytes: 500 * 1024 * 1024,
            storage_reserve_percent: 1,
            max_upload_bytes: 0,
            log_level: None,
        }
    }
}

/// CLI overrides applied after figment extraction.
#[derive(Debug, Clone, Default)]
pub struct CliOverrides {
    /// Bind address override.
    pub bind: Option<IpAddr>,
    /// Port override.
    pub port: Option<u16>,
    /// Data directory override.
    pub data_dir: Option<PathBuf>,
    /// Public URL override (`None` = do not override).
    pub public_url: Option<Option<String>>,
    /// Trust forwarded headers override.
    pub trust_forwarded: Option<bool>,
    /// mDNS advertisement override.
    pub mdns: Option<bool>,
    /// Metrics endpoint override.
    pub metrics: Option<bool>,
    /// Web dashboard override.
    pub web: Option<bool>,
    /// Exit after printing config.
    pub check_config: bool,
}

impl DaemonSettings {
    /// Loads settings from defaults, `data_dir/tssp.toml`, `TSSPD_*` env, then CLI.
    ///
    /// # Errors
    ///
    /// Returns a message when configuration is invalid or cannot be read.
    pub fn load(data_dir: &Path, cli: &CliOverrides) -> Result<Self, String> {
        let defaults = Self::default();
        let mut figment = Figment::from(Serialized::defaults(defaults));
        let config_path = data_dir.join("tssp.toml");
        if config_path.is_file() {
            figment = figment.merge(Toml::file(&config_path));
        }
        figment = figment.merge(Env::prefixed("TSSPD_").split("_"));
        let mut settings: Self = figment
            .extract()
            .map_err(|error| format!("invalid configuration: {error}"))?;
        settings.apply_cli(cli);
        settings.validate()?;
        Ok(settings)
    }

    fn apply_cli(&mut self, cli: &CliOverrides) {
        if let Some(bind) = cli.bind {
            self.bind = bind;
        }
        if let Some(port) = cli.port {
            self.port = port;
        }
        if let Some(data_dir) = &cli.data_dir {
            self.data_dir.clone_from(data_dir);
        }
        if let Some(public_url) = &cli.public_url {
            self.public_url.clone_from(public_url);
        }
        if let Some(trust_forwarded) = cli.trust_forwarded {
            self.trust_forwarded = trust_forwarded;
        }
        if let Some(mdns) = cli.mdns {
            self.mdns = mdns;
        }
        if let Some(metrics) = cli.metrics {
            self.metrics = metrics;
        }
        if let Some(web) = cli.web {
            self.web = web;
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("port must not be 0".to_owned());
        }
        if let Some(url) = &self.public_url {
            if url.trim().is_empty() {
                return Err("public_url must not be empty when set".to_owned());
            }
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err("public_url must start with http:// or https://".to_owned());
            }
        }
        Ok(())
    }

    /// Path to the layered config file inside the data directory.
    #[must_use]
    pub fn config_file_path(&self) -> PathBuf {
        self.data_dir.join("tssp.toml")
    }

    /// Socket address used for binding the HTTP listener.
    #[must_use]
    pub fn socket_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::new(self.bind, self.port)
    }

    /// Logs the effective configuration at INFO level.
    pub fn log_effective(&self) {
        tracing::info!(
            bind = %self.bind,
            port = self.port,
            data_dir = %self.data_dir.display(),
            public_url = self.public_url.as_deref().unwrap_or("(derived)"),
            trust_forwarded = self.trust_forwarded,
            mdns = self.mdns,
            metrics = self.metrics,
            web = self.web,
            session_ttl_seconds = self.session_ttl_seconds,
            max_upload_bytes = self.max_upload_bytes,
            "effective configuration"
        );
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{CliOverrides, DaemonSettings};
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn toml_file_overrides_defaults() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("tssp.toml");
        std::fs::write(&path, "port = 9001\n").expect("write");
        let settings = DaemonSettings::load(temp.path(), &CliOverrides::default()).expect("load");
        assert_eq!(settings.port, 9001);
    }

    #[test]
    fn cli_overrides_env() {
        let temp = tempfile::tempdir().expect("tempdir");
        let cli = CliOverrides {
            port: Some(7777),
            ..CliOverrides::default()
        };
        let settings = DaemonSettings::load(temp.path(), &cli).expect("load");
        assert_eq!(settings.port, 7777);
    }

    #[test]
    fn rejects_invalid_public_url() {
        let settings = DaemonSettings {
            public_url: Some("ftp://bad".to_owned()),
            ..DaemonSettings::default()
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn default_bind_is_loopback() {
        let settings = DaemonSettings::default();
        assert_eq!(settings.bind, IpAddr::V4(Ipv4Addr::LOCALHOST));
    }
}
