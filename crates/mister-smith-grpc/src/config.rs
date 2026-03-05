//! gRPC transport configuration.

use serde::{Deserialize, Serialize};

/// Default gRPC bind address.
pub const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0:50051";

/// Default maximum message size in bytes (4 MB).
pub const DEFAULT_MAX_MESSAGE_SIZE: usize = 4_194_304;

/// Configuration for the gRPC transport layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcTransportConfig {
    /// Socket address the gRPC server binds to (e.g., `"0.0.0.0:50051"`).
    pub bind_address: String,
    /// Maximum inbound/outbound message size in bytes.
    pub max_message_size: usize,
}

impl Default for GrpcTransportConfig {
    fn default() -> Self {
        Self {
            bind_address: DEFAULT_BIND_ADDRESS.to_string(),
            max_message_size: DEFAULT_MAX_MESSAGE_SIZE,
        }
    }
}

impl GrpcTransportConfig {
    /// Create a new configuration with the given bind address and default max message size.
    #[must_use]
    pub fn new(bind_address: impl Into<String>) -> Self {
        Self {
            bind_address: bind_address.into(),
            ..Default::default()
        }
    }

    /// Set the maximum message size.
    #[must_use]
    pub fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }

    /// Parse the bind address into a `SocketAddr`.
    ///
    /// # Errors
    ///
    /// Returns an error if the address string cannot be parsed.
    pub fn socket_addr(&self) -> Result<std::net::SocketAddr, std::net::AddrParseError> {
        self.bind_address.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = GrpcTransportConfig::default();
        assert_eq!(config.bind_address, "0.0.0.0:50051");
        assert_eq!(config.max_message_size, 4_194_304);
    }

    #[test]
    fn custom_bind_address() {
        let config = GrpcTransportConfig::new("127.0.0.1:9090");
        assert_eq!(config.bind_address, "127.0.0.1:9090");
        assert_eq!(config.max_message_size, DEFAULT_MAX_MESSAGE_SIZE);
    }

    #[test]
    fn builder_pattern() {
        let config = GrpcTransportConfig::new("[::1]:50052").with_max_message_size(8_388_608);
        assert_eq!(config.bind_address, "[::1]:50052");
        assert_eq!(config.max_message_size, 8_388_608);
    }

    #[test]
    fn socket_addr_valid() {
        let config = GrpcTransportConfig::default();
        let addr = config.socket_addr().unwrap();
        assert_eq!(addr.port(), 50051);
    }

    #[test]
    fn socket_addr_invalid() {
        let config = GrpcTransportConfig::new("not-an-address");
        assert!(config.socket_addr().is_err());
    }

    #[test]
    fn serde_roundtrip() {
        let config = GrpcTransportConfig::new("0.0.0.0:8080").with_max_message_size(1024);
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: GrpcTransportConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.bind_address, "0.0.0.0:8080");
        assert_eq!(deserialized.max_message_size, 1024);
    }
}
