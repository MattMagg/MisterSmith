//! MessageEnvelope — universal message wrapper for all framework communication.

use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::TransportError;
use crate::priority::MessagePriority;
use crate::serialization;

/// Default maximum payload size: 1 MB.
pub const DEFAULT_MAX_PAYLOAD_SIZE: usize = 1_048_576;

/// Default schema version for new envelopes.
pub const SCHEMA_VERSION: &str = "1.0.0";

/// Universal message wrapper for all framework communication.
///
/// All messages flowing through any transport are wrapped in a `MessageEnvelope`.
/// The envelope provides routing metadata, correlation, tracing, and the serialized
/// payload of the inner message type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    /// Unique message identifier (UUID v4).
    pub message_id: Uuid,

    /// Message creation timestamp.
    pub timestamp: DateTime<Utc>,

    /// Envelope schema version (semver).
    pub schema_version: String,

    /// Discriminator for routing and deserialization.
    pub message_type: String,

    /// Links request to response for request-reply patterns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<Uuid>,

    /// Distributed tracing identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<Uuid>,

    /// Sending agent identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_agent_id: Option<Uuid>,

    /// Intended recipient agent identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_agent_id: Option<Uuid>,

    /// Message priority level.
    pub priority: MessagePriority,

    /// Serialized message content (MessagePack or JSON).
    #[serde(with = "serde_bytes")]
    pub payload: Vec<u8>,

    /// Transport-level metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
}

impl MessageEnvelope {
    /// Create a new `MessageEnvelopeBuilder`.
    pub fn builder(message_type: impl Into<String>) -> MessageEnvelopeBuilder {
        MessageEnvelopeBuilder {
            message_type: message_type.into(),
            correlation_id: None,
            trace_id: None,
            source_agent_id: None,
            target_agent_id: None,
            priority: MessagePriority::Normal,
            payload: Vec::new(),
            headers: HashMap::new(),
            max_payload_size: DEFAULT_MAX_PAYLOAD_SIZE,
        }
    }

    /// Deserialize the payload as a MessagePack value.
    pub fn payload_as<T: serde::de::DeserializeOwned>(&self) -> Result<T, TransportError> {
        serialization::from_msgpack(&self.payload)
    }

    /// Deserialize the payload as a JSON value.
    pub fn payload_as_json<T: serde::de::DeserializeOwned>(&self) -> Result<T, TransportError> {
        let json_str = std::str::from_utf8(&self.payload)
            .map_err(|e| TransportError::DeserializationError(e.to_string()))?;
        serialization::from_json(json_str)
    }

    /// Convert the envelope to MessagePack bytes.
    pub fn to_bytes(&self) -> Result<Bytes, TransportError> {
        let bytes = serialization::to_msgpack(self)?;
        Ok(Bytes::from(bytes))
    }

    /// Parse an envelope from MessagePack bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, TransportError> {
        serialization::from_msgpack(bytes)
    }
}

/// Builder for constructing `MessageEnvelope` instances.
pub struct MessageEnvelopeBuilder {
    message_type: String,
    correlation_id: Option<Uuid>,
    trace_id: Option<Uuid>,
    source_agent_id: Option<Uuid>,
    target_agent_id: Option<Uuid>,
    priority: MessagePriority,
    payload: Vec<u8>,
    headers: HashMap<String, String>,
    max_payload_size: usize,
}

impl MessageEnvelopeBuilder {
    /// Set the correlation ID for request-reply linking.
    pub fn correlation_id(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Set the distributed trace ID.
    pub fn trace_id(mut self, id: Uuid) -> Self {
        self.trace_id = Some(id);
        self
    }

    /// Set the source agent ID.
    pub fn source_agent_id(mut self, id: Uuid) -> Self {
        self.source_agent_id = Some(id);
        self
    }

    /// Set the target agent ID.
    pub fn target_agent_id(mut self, id: Uuid) -> Self {
        self.target_agent_id = Some(id);
        self
    }

    /// Set the message priority.
    pub fn priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set the payload from a MessagePack-serializable value.
    pub fn payload_msgpack<T: serde::Serialize>(mut self, val: &T) -> Result<Self, TransportError> {
        self.payload = serialization::to_msgpack(val)?;
        Ok(self)
    }

    /// Set the payload from a JSON-serializable value.
    pub fn payload_json<T: serde::Serialize>(mut self, val: &T) -> Result<Self, TransportError> {
        let json = serialization::to_json(val)?;
        self.payload = json.into_bytes();
        Ok(self)
    }

    /// Set the payload as raw bytes.
    pub fn payload_raw(mut self, bytes: Vec<u8>) -> Self {
        self.payload = bytes;
        self
    }

    /// Add a header key-value pair.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Override the maximum payload size for validation.
    pub fn max_payload_size(mut self, size: usize) -> Self {
        self.max_payload_size = size;
        self
    }

    /// Build the `MessageEnvelope`, validating all fields.
    pub fn build(self) -> Result<MessageEnvelope, TransportError> {
        if self.message_type.is_empty() {
            return Err(TransportError::SubjectInvalid(
                "message_type must not be empty".into(),
            ));
        }

        if self.payload.len() > self.max_payload_size {
            return Err(TransportError::PayloadTooLarge {
                size: self.payload.len(),
                limit: self.max_payload_size,
            });
        }

        Ok(MessageEnvelope {
            message_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            schema_version: SCHEMA_VERSION.to_string(),
            message_type: self.message_type,
            correlation_id: self.correlation_id,
            trace_id: self.trace_id,
            source_agent_id: self.source_agent_id,
            target_agent_id: self.target_agent_id,
            priority: self.priority,
            payload: self.payload,
            headers: self.headers,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_minimal() {
        let envelope = MessageEnvelope::builder("test.message").build().unwrap();
        assert_eq!(envelope.message_type, "test.message");
        assert_eq!(envelope.schema_version, SCHEMA_VERSION);
        assert_eq!(envelope.priority, MessagePriority::Normal);
        assert!(envelope.correlation_id.is_none());
        assert!(envelope.trace_id.is_none());
        assert!(envelope.source_agent_id.is_none());
        assert!(envelope.target_agent_id.is_none());
        assert!(envelope.payload.is_empty());
        assert!(envelope.headers.is_empty());
    }

    #[test]
    fn builder_all_fields() {
        let corr_id = Uuid::new_v4();
        let trace_id = Uuid::new_v4();
        let source = Uuid::new_v4();
        let target = Uuid::new_v4();

        let envelope = MessageEnvelope::builder("task.assignment")
            .correlation_id(corr_id)
            .trace_id(trace_id)
            .source_agent_id(source)
            .target_agent_id(target)
            .priority(MessagePriority::High)
            .header("x-region", "us-east-1")
            .payload_raw(vec![1, 2, 3])
            .build()
            .unwrap();

        assert_eq!(envelope.correlation_id, Some(corr_id));
        assert_eq!(envelope.trace_id, Some(trace_id));
        assert_eq!(envelope.source_agent_id, Some(source));
        assert_eq!(envelope.target_agent_id, Some(target));
        assert_eq!(envelope.priority, MessagePriority::High);
        assert_eq!(envelope.headers.get("x-region").unwrap(), "us-east-1");
        assert_eq!(envelope.payload, vec![1, 2, 3]);
    }

    #[test]
    fn builder_rejects_empty_message_type() {
        let result = MessageEnvelope::builder("").build();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransportError::SubjectInvalid(_)
        ));
    }

    #[test]
    fn builder_rejects_oversized_payload() {
        let result = MessageEnvelope::builder("big")
            .max_payload_size(10)
            .payload_raw(vec![0u8; 100])
            .build();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransportError::PayloadTooLarge { .. }
        ));
    }

    #[test]
    fn builder_payload_msgpack() {
        let data = serde_json::json!({"key": "value", "count": 42});
        let envelope = MessageEnvelope::builder("test")
            .payload_msgpack(&data)
            .unwrap()
            .build()
            .unwrap();
        let decoded: serde_json::Value = envelope.payload_as().unwrap();
        assert_eq!(decoded["key"], "value");
        assert_eq!(decoded["count"], 42);
    }

    #[test]
    fn builder_payload_json() {
        let data = serde_json::json!({"hello": "world"});
        let envelope = MessageEnvelope::builder("test")
            .payload_json(&data)
            .unwrap()
            .build()
            .unwrap();
        let decoded: serde_json::Value = envelope.payload_as_json().unwrap();
        assert_eq!(decoded["hello"], "world");
    }

    #[test]
    fn envelope_msgpack_roundtrip() {
        let envelope = MessageEnvelope::builder("roundtrip.test")
            .correlation_id(Uuid::new_v4())
            .payload_raw(vec![10, 20, 30])
            .build()
            .unwrap();

        let bytes = envelope.to_bytes().unwrap();
        let decoded = MessageEnvelope::from_bytes(&bytes).unwrap();

        assert_eq!(envelope.message_id, decoded.message_id);
        assert_eq!(envelope.message_type, decoded.message_type);
        assert_eq!(envelope.correlation_id, decoded.correlation_id);
        assert_eq!(envelope.payload, decoded.payload);
        assert_eq!(envelope.priority, decoded.priority);
    }

    #[test]
    fn envelope_serde_json_roundtrip() {
        let envelope = MessageEnvelope::builder("json.test")
            .priority(MessagePriority::Critical)
            .header("trace", "abc123")
            .payload_raw(b"hello".to_vec())
            .build()
            .unwrap();

        let json = serde_json::to_string(&envelope).unwrap();
        let decoded: MessageEnvelope = serde_json::from_str(&json).unwrap();

        assert_eq!(envelope.message_id, decoded.message_id);
        assert_eq!(envelope.message_type, decoded.message_type);
        assert_eq!(envelope.priority, decoded.priority);
        assert_eq!(envelope.headers, decoded.headers);
    }

    #[test]
    fn envelope_10000_msgpack_roundtrips() {
        // SC-002: 10,000 MessagePack round-trip cycles with zero data loss.
        let envelope = MessageEnvelope::builder("stress.test")
            .correlation_id(Uuid::new_v4())
            .trace_id(Uuid::new_v4())
            .source_agent_id(Uuid::new_v4())
            .target_agent_id(Uuid::new_v4())
            .priority(MessagePriority::High)
            .header("batch", "true")
            .payload_raw(b"stress test payload data".to_vec())
            .build()
            .unwrap();

        for i in 0..10_000 {
            let bytes = envelope.to_bytes().unwrap_or_else(|e| {
                panic!("Serialization failed at iteration {i}: {e}");
            });
            let decoded = MessageEnvelope::from_bytes(&bytes).unwrap_or_else(|e| {
                panic!("Deserialization failed at iteration {i}: {e}");
            });
            assert_eq!(
                envelope.message_id, decoded.message_id,
                "message_id mismatch at iteration {i}"
            );
            assert_eq!(
                envelope.message_type, decoded.message_type,
                "message_type mismatch at iteration {i}"
            );
            assert_eq!(
                envelope.payload, decoded.payload,
                "payload mismatch at iteration {i}"
            );
        }
    }
}
