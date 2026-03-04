//! Monitoring types: ComponentId, Status, HealthStatus.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::SystemTime;

/// Unique identifier for a monitored component.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentId(String);

impl ComponentId {
    /// Create a new ComponentId from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ComponentId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ComponentId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// Health status of a monitored component.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    /// Component is fully operational.
    Healthy,
    /// Component is operational but experiencing issues.
    Degraded,
    /// Component is not operational.
    Unhealthy,
    /// Component health is unknown (not yet checked).
    #[default]
    Unknown,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Healthy => write!(f, "Healthy"),
            Status::Degraded => write!(f, "Degraded"),
            Status::Unhealthy => write!(f, "Unhealthy"),
            Status::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Detailed health status of a component, including metadata and timing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// The component this status belongs to.
    pub component_id: ComponentId,
    /// Current health status.
    pub status: Status,
    /// Timestamp of the last health check.
    pub last_check: SystemTime,
    /// Optional human-readable message.
    pub message: Option<String>,
    /// Arbitrary metadata (e.g., latency, version, connection count).
    pub metadata: HashMap<String, serde_json::Value>,
}

impl HealthStatus {
    /// Create a new HealthStatus with the given component ID and status.
    ///
    /// Sets `last_check` to the current time and initialises `message` and
    /// `metadata` to their empty defaults.
    pub fn new(component_id: impl Into<ComponentId>, status: Status) -> Self {
        Self {
            component_id: component_id.into(),
            status,
            last_check: SystemTime::now(),
            message: None,
            metadata: HashMap::new(),
        }
    }

    /// Set an optional message on this health status.
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Insert a metadata key-value pair.
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_id_from_string() {
        let id = ComponentId::from("test-component".to_string());
        assert_eq!(id.as_str(), "test-component");
    }

    #[test]
    fn component_id_from_str() {
        let id = ComponentId::from("test-component");
        assert_eq!(id.as_str(), "test-component");
    }

    #[test]
    fn component_id_display() {
        let id = ComponentId::new("my-comp");
        assert_eq!(id.to_string(), "my-comp");
    }

    #[test]
    fn component_id_equality() {
        let a = ComponentId::new("comp-a");
        let b = ComponentId::new("comp-a");
        let c = ComponentId::new("comp-b");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn status_default_is_unknown() {
        assert_eq!(Status::default(), Status::Unknown);
    }

    #[test]
    fn status_display() {
        assert_eq!(Status::Healthy.to_string(), "Healthy");
        assert_eq!(Status::Degraded.to_string(), "Degraded");
        assert_eq!(Status::Unhealthy.to_string(), "Unhealthy");
        assert_eq!(Status::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn status_serde_roundtrip() {
        let status = Status::Degraded;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: Status = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn health_status_new() {
        let hs = HealthStatus::new("runtime", Status::Healthy);
        assert_eq!(hs.component_id, ComponentId::new("runtime"));
        assert_eq!(hs.status, Status::Healthy);
        assert!(hs.message.is_none());
        assert!(hs.metadata.is_empty());
    }

    #[test]
    fn health_status_with_message() {
        let hs = HealthStatus::new("db", Status::Degraded)
            .with_message("high latency");
        assert_eq!(hs.message.as_deref(), Some("high latency"));
    }

    #[test]
    fn health_status_with_metadata() {
        let hs = HealthStatus::new("db", Status::Healthy)
            .with_metadata("latency_ms", serde_json::json!(42));
        assert_eq!(hs.metadata.get("latency_ms"), Some(&serde_json::json!(42)));
    }

    #[test]
    fn health_status_serde_roundtrip() {
        let hs = HealthStatus::new("nats", Status::Healthy)
            .with_message("connected")
            .with_metadata("connections", serde_json::json!(5));
        let json = serde_json::to_string(&hs).unwrap();
        let deserialized: HealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.component_id, hs.component_id);
        assert_eq!(deserialized.status, hs.status);
        assert_eq!(deserialized.message, hs.message);
        assert_eq!(deserialized.metadata, hs.metadata);
    }
}
