//! MessageEnvelope — universal message wrapper for all framework communication.

use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::TransportError;
use crate::priority::MessagePriority;
use crate::serialization;

/// Classifies whether a message carries data-plane or control-plane traffic.
///
/// Data-plane messages use NATS request-reply or Core pub/sub (microsecond latency).
/// Control-plane messages use JetStream KV watches and durable consumers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum MessagePlane {
    /// Request-reply, streaming, tool calls (microsecond latency budget).
    #[default]
    Data,
    /// Configuration updates, health telemetry, budget state (JetStream KV watches).
    Control,
}

/// Classifies whether a stream event requires lossless or best-effort delivery.
///
/// Semantic events are delivered losslessly via JetStream (tool calls, lifecycle, errors).
/// UI events are delivered best-effort via NATS Core (text deltas, heartbeats, progress).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum StreamClass {
    /// Lossless delivery via JetStream (tool calls, lifecycle, errors, finalization).
    #[default]
    Semantic,
    /// Best-effort delivery via NATS Core (text deltas, heartbeats, progress indicators).
    Ui,
}

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

    /// Message plane classification (data vs control).
    /// `None` is treated as `Data` for backward compatibility with pre-Phase-9 messages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plane: Option<MessagePlane>,

    /// Stream class classification (semantic vs UI).
    /// `None` is treated as `Semantic` for backward compatibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_class: Option<StreamClass>,
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
            plane: None,
            stream_class: None,
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
    plane: Option<MessagePlane>,
    stream_class: Option<StreamClass>,
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

    /// Set the message plane classification.
    pub fn plane(mut self, plane: MessagePlane) -> Self {
        self.plane = Some(plane);
        self
    }

    /// Set the stream class classification.
    pub fn stream_class(mut self, stream_class: StreamClass) -> Self {
        self.stream_class = Some(stream_class);
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
            plane: self.plane,
            stream_class: self.stream_class,
        })
    }
}

// ---------------------------------------------------------------------------
// W3C Trace Context propagation (Phase 8 — Observability)
// ---------------------------------------------------------------------------

/// W3C Trace Context header name for trace propagation.
pub const TRACEPARENT_HEADER: &str = "traceparent";
/// W3C Trace Context state header for vendor-specific propagation.
pub const TRACESTATE_HEADER: &str = "tracestate";

/// Inject the current tracing span's context into a MessageEnvelope's headers.
///
/// Writes `traceparent` and optionally `tracestate` headers following the
/// W3C Trace Context specification. This enables distributed trace correlation
/// across NATS, HTTP, and gRPC transports.
///
/// If no active span exists, this is a no-op.
pub fn inject_trace_context(envelope: &mut MessageEnvelope) {
    use tracing::Span;

    let span = Span::current();
    if let Some(span_id) = span.id() {
        // Store the span ID as a simplified traceparent for correlation.
        // When the full OTel SDK is wired, this will use the W3C propagator
        // to inject the real traceparent from the OpenTelemetry context.
        let traceparent = format!(
            "00-{:032x}-{:016x}-01",
            span_id.into_u64(),
            span_id.into_u64()
        );
        envelope
            .headers
            .insert(TRACEPARENT_HEADER.to_string(), traceparent);
    }
}

/// Extract trace context from a MessageEnvelope's headers.
///
/// Returns the `traceparent` header value if present, which can be used
/// to create a child span linked to the parent trace.
pub fn extract_trace_context(envelope: &MessageEnvelope) -> Option<&str> {
    envelope.headers.get(TRACEPARENT_HEADER).map(|s| s.as_str())
}

/// Extract the tracestate header from a MessageEnvelope.
pub fn extract_tracestate(envelope: &MessageEnvelope) -> Option<&str> {
    envelope.headers.get(TRACESTATE_HEADER).map(|s| s.as_str())
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
    fn trace_context_injection() {
        let mut envelope = MessageEnvelope::builder("test").build().unwrap();
        assert!(envelope.headers.is_empty());

        // inject_trace_context is a no-op when there's no active span
        inject_trace_context(&mut envelope);
        // No active tracing subscriber, so no span context to inject
        assert!(extract_trace_context(&envelope).is_none());
    }

    #[test]
    fn trace_context_manual_headers() {
        let traceparent = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let mut envelope = MessageEnvelope::builder("test").build().unwrap();
        envelope
            .headers
            .insert(TRACEPARENT_HEADER.to_string(), traceparent.to_string());
        envelope.headers.insert(
            TRACESTATE_HEADER.to_string(),
            "mistersmith=testing".to_string(),
        );

        assert_eq!(extract_trace_context(&envelope), Some(traceparent));
        assert_eq!(extract_tracestate(&envelope), Some("mistersmith=testing"));
    }

    #[test]
    fn pre_phase9_json_without_plane_or_stream_class() {
        // Pre-Phase-9 envelopes have no `plane` or `stream_class` fields.
        // Deserialization must succeed with both defaulting to None.
        let pre_phase9_json = serde_json::json!({
            "message_id": Uuid::new_v4(),
            "message_type": "legacy.message",
            "schema_version": SCHEMA_VERSION,
            "timestamp": chrono::Utc::now(),
            "priority": "Normal",
            "payload": [],
            "headers": {}
        });

        let envelope: MessageEnvelope = serde_json::from_value(pre_phase9_json).unwrap();
        assert!(envelope.plane.is_none());
        assert!(envelope.stream_class.is_none());
    }

    #[test]
    fn plane_defaults_to_data_when_absent() {
        let envelope = MessageEnvelope::builder("test").build().unwrap();
        // None is treated as Data plane
        assert!(envelope.plane.is_none());
        assert_eq!(
            envelope.plane.unwrap_or(MessagePlane::Data),
            MessagePlane::Data
        );
    }

    #[test]
    fn stream_class_defaults_to_semantic_when_absent() {
        let envelope = MessageEnvelope::builder("test").build().unwrap();
        // None is treated as Semantic stream
        assert!(envelope.stream_class.is_none());
        assert_eq!(
            envelope.stream_class.unwrap_or(StreamClass::Semantic),
            StreamClass::Semantic
        );
    }

    #[test]
    fn plane_and_stream_class_round_trip() {
        let envelope = MessageEnvelope::builder("dual.test")
            .plane(MessagePlane::Control)
            .stream_class(StreamClass::Ui)
            .build()
            .unwrap();

        let json = serde_json::to_string(&envelope).unwrap();
        let decoded: MessageEnvelope = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.plane, Some(MessagePlane::Control));
        assert_eq!(decoded.stream_class, Some(StreamClass::Ui));
    }

    #[test]
    fn plane_and_stream_class_msgpack_round_trip() {
        let envelope = MessageEnvelope::builder("msgpack.dual")
            .plane(MessagePlane::Data)
            .stream_class(StreamClass::Semantic)
            .build()
            .unwrap();

        let bytes = envelope.to_bytes().unwrap();
        let decoded = MessageEnvelope::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.plane, Some(MessagePlane::Data));
        assert_eq!(decoded.stream_class, Some(StreamClass::Semantic));
    }

    #[test]
    fn plane_omitted_from_json_when_none() {
        let envelope = MessageEnvelope::builder("test").build().unwrap();
        let json = serde_json::to_string(&envelope).unwrap();
        // plane and stream_class should be omitted, not serialized as null
        assert!(!json.contains("\"plane\""));
        assert!(!json.contains("\"stream_class\""));
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
