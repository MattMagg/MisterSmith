//! NATS transport configuration.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Configuration for the NATS transport connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsTransportConfig {
    /// NATS server URLs.
    #[serde(default = "default_server_urls")]
    pub server_urls: Vec<String>,

    /// Connection name for identification.
    #[serde(default = "default_name")]
    pub name: String,

    /// Maximum reconnection attempts. `None` for unlimited.
    #[serde(default)]
    pub max_reconnects: Option<usize>,

    /// Initial connection timeout.
    #[serde(
        default = "default_connection_timeout",
        with = "humantime_serde_duration"
    )]
    pub connection_timeout: Duration,

    /// Default request-reply timeout.
    #[serde(default = "default_request_timeout", with = "humantime_serde_duration")]
    pub request_timeout: Duration,

    /// Internal send buffer size.
    #[serde(default = "default_client_capacity")]
    pub client_capacity: usize,

    /// Per-subscriber buffer size.
    #[serde(default = "default_subscription_capacity")]
    pub subscription_capacity: usize,

    /// Require TLS for the connection.
    #[serde(default)]
    pub tls_required: bool,

    /// Client certificate path.
    #[serde(default)]
    pub tls_cert: Option<PathBuf>,

    /// Client key path.
    #[serde(default)]
    pub tls_key: Option<PathBuf>,

    /// CA certificate path.
    #[serde(default)]
    pub tls_ca: Option<PathBuf>,

    /// JetStream configuration.
    #[serde(default)]
    pub jetstream: JetStreamConfig,
}

/// JetStream subsystem configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JetStreamConfig {
    /// Enable JetStream support.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// JetStream domain.
    #[serde(default)]
    pub domain: Option<String>,

    /// Maximum pending publish acknowledgments.
    #[serde(default = "default_max_ack_inflight")]
    pub max_ack_inflight: usize,

    /// Publish acknowledgment timeout.
    #[serde(default = "default_ack_timeout", with = "humantime_serde_duration")]
    pub ack_timeout: Duration,
}

impl Default for NatsTransportConfig {
    fn default() -> Self {
        Self {
            server_urls: default_server_urls(),
            name: default_name(),
            max_reconnects: None,
            connection_timeout: default_connection_timeout(),
            request_timeout: default_request_timeout(),
            client_capacity: default_client_capacity(),
            subscription_capacity: default_subscription_capacity(),
            tls_required: false,
            tls_cert: None,
            tls_key: None,
            tls_ca: None,
            jetstream: JetStreamConfig::default(),
        }
    }
}

impl Default for JetStreamConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            domain: None,
            max_ack_inflight: default_max_ack_inflight(),
            ack_timeout: default_ack_timeout(),
        }
    }
}

fn default_server_urls() -> Vec<String> {
    vec!["nats://localhost:4222".to_string()]
}

fn default_name() -> String {
    "mister-smith".to_string()
}

fn default_connection_timeout() -> Duration {
    Duration::from_secs(5)
}

fn default_request_timeout() -> Duration {
    Duration::from_secs(10)
}

fn default_client_capacity() -> usize {
    2048
}

fn default_subscription_capacity() -> usize {
    65536
}

fn default_max_ack_inflight() -> usize {
    5000
}

fn default_ack_timeout() -> Duration {
    Duration::from_secs(30)
}

fn default_true() -> bool {
    true
}

/// Serde helper for Duration as seconds (u64).
mod humantime_serde_duration {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(duration: &Duration, s: S) -> Result<S::Ok, S::Error> {
        duration.as_secs().serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let secs = u64::deserialize(d)?;
        Ok(Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = NatsTransportConfig::default();
        assert_eq!(config.server_urls, vec!["nats://localhost:4222"]);
        assert_eq!(config.name, "mister-smith");
        assert_eq!(config.connection_timeout, Duration::from_secs(5));
        assert_eq!(config.request_timeout, Duration::from_secs(10));
        assert_eq!(config.client_capacity, 2048);
        assert!(config.jetstream.enabled);
    }

    #[test]
    fn config_serde_roundtrip() {
        let config = NatsTransportConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let decoded: NatsTransportConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.server_urls, decoded.server_urls);
        assert_eq!(config.name, decoded.name);
    }
}
