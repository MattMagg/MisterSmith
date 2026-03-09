//! HTTP transport configuration.
//!
//! Defines [`HttpTransportConfig`] with sensible defaults for the HTTP server
//! including bind address, WebSocket settings, and rate limiting.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for the HTTP transport layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpTransportConfig {
    /// Address to bind the HTTP server to.
    pub bind_address: String,
    /// Whether WebSocket endpoints are enabled.
    pub websocket_enabled: bool,
    /// Interval between WebSocket keepalive pings.
    #[serde(with = "duration_secs")]
    pub ws_keepalive_interval: Duration,
    /// Maximum number of concurrent WebSocket connections.
    pub max_ws_connections: usize,
    /// Maximum requests per second per IP for rate limiting.
    pub rate_limit_rps: u32,
    /// List of allowed origins for CORS.
    ///
    /// Set to `["*"]` to allow any origin (permissive).
    /// Default is empty (strict).
    pub allowed_origins: Vec<String>,
}

impl Default for HttpTransportConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:8080".to_string(),
            websocket_enabled: true,
            ws_keepalive_interval: Duration::from_secs(30),
            max_ws_connections: 1000,
            rate_limit_rps: 100,
            allowed_origins: Vec::new(),
        }
    }
}

/// Serde helper for serializing `Duration` as seconds (u64).
mod duration_secs {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error> {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Duration, D::Error> {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = HttpTransportConfig::default();
        assert_eq!(config.bind_address, "0.0.0.0:8080");
        assert!(config.websocket_enabled);
        assert_eq!(config.ws_keepalive_interval, Duration::from_secs(30));
        assert_eq!(config.max_ws_connections, 1000);
        assert_eq!(config.rate_limit_rps, 100);
    }

    #[test]
    fn config_serde_roundtrip() {
        let config = HttpTransportConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: HttpTransportConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.bind_address, config.bind_address);
        assert_eq!(deserialized.websocket_enabled, config.websocket_enabled);
        assert_eq!(
            deserialized.ws_keepalive_interval,
            config.ws_keepalive_interval
        );
        assert_eq!(deserialized.max_ws_connections, config.max_ws_connections);
        assert_eq!(deserialized.rate_limit_rps, config.rate_limit_rps);
    }

    #[test]
    fn config_custom_values() {
        let config = HttpTransportConfig {
            bind_address: "127.0.0.1:9090".to_string(),
            websocket_enabled: false,
            ws_keepalive_interval: Duration::from_secs(60),
            max_ws_connections: 500,
            rate_limit_rps: 50,
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: HttpTransportConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.bind_address, "127.0.0.1:9090");
        assert!(!deserialized.websocket_enabled);
        assert_eq!(deserialized.ws_keepalive_interval, Duration::from_secs(60));
        assert_eq!(deserialized.max_ws_connections, 500);
        assert_eq!(deserialized.rate_limit_rps, 50);
    }
}
