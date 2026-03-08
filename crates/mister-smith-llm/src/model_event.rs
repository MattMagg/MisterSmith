use serde::{Deserialize, Serialize};

use crate::types::{StopReason, Usage};

/// Canonical internal event type emitted by stream actors after converting raw
/// `StreamChunk` items from providers. Consumers receive `ModelEvent`, not `StreamChunk`.
///
/// `StreamChunk`/`ChunkDelta` (4 variants) remain the raw provider-to-framework boundary.
/// `ModelEvent` is the canonical internal event type with richer semantics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum ModelEvent {
    // -- Lifecycle (5) --
    /// Stream has started from a specific model.
    StreamStarted {
        model_id: String,
        request_id: String,
    },
    /// Stream completed normally.
    StreamCompleted {
        usage: Usage,
        stop_reason: StopReason,
    },
    /// Stream failed with an error.
    StreamFailed {
        error: String,
        recoverable: bool,
    },
    /// Stream was cancelled.
    StreamCancelled {
        reason: String,
    },
    /// Stream resumed from a checkpoint.
    StreamResumed {
        from_checkpoint: String,
    },

    // -- Text (3) --
    /// Incremental text content.
    TextDelta {
        text: String,
    },
    /// Complete assembled text block.
    TextCompleted {
        full_text: String,
    },
    /// Text annotation metadata.
    TextAnnotation {
        annotation: serde_json::Value,
    },

    // -- Tool Call (4) --
    /// Model initiated a tool call.
    ToolCallStart {
        call_id: String,
        name: String,
    },
    /// Incremental tool call input.
    ToolCallDelta {
        call_id: String,
        input_chunk: String,
    },
    /// Tool call fully assembled.
    ToolCallCompleted {
        call_id: String,
        name: String,
        input: serde_json::Value,
    },
    /// Tool execution result.
    ToolResult {
        call_id: String,
        result: serde_json::Value,
        error: Option<String>,
    },

    // -- Observability (3) --
    /// Token usage update during streaming.
    UsageUpdate {
        usage: Usage,
    },
    /// Latency checkpoint marker.
    LatencyMarker {
        checkpoint: String,
        elapsed_ms: u64,
    },
    /// Routing decision for observability.
    RoutingDecision {
        model_id: String,
        tier: String,
        reason: String,
    },

    // -- Error (1) --
    /// Typed error event.
    Error {
        code: String,
        message: String,
        recoverable: bool,
    },

    // -- Heartbeat (1) --
    /// Periodic heartbeat to indicate stream liveness.
    Heartbeat {
        sequence: u64,
    },

    // -- Forward compatibility (1) --
    /// Unknown event type for forward compatibility.
    #[serde(other)]
    Unknown,
}

impl ModelEvent {
    /// Returns the backpressure policy for this event class.
    pub fn backpressure_policy(&self) -> BackpressurePolicy {
        match self {
            // Lossless: tool calls, lifecycle, errors
            Self::ToolCallStart { .. }
            | Self::ToolCallDelta { .. }
            | Self::ToolCallCompleted { .. }
            | Self::ToolResult { .. }
            | Self::StreamStarted { .. }
            | Self::StreamCompleted { .. }
            | Self::StreamFailed { .. }
            | Self::StreamCancelled { .. }
            | Self::StreamResumed { .. }
            | Self::Error { .. } => BackpressurePolicy::Lossless,

            // Coalescible: text deltas, observability
            Self::TextDelta { .. }
            | Self::TextCompleted { .. }
            | Self::TextAnnotation { .. }
            | Self::UsageUpdate { .. }
            | Self::LatencyMarker { .. }
            | Self::RoutingDecision { .. }
            | Self::Unknown => BackpressurePolicy::Coalescible,

            // Droppable: heartbeats
            Self::Heartbeat { .. } => BackpressurePolicy::Droppable,
        }
    }

    /// Returns the recommended stream class for this event.
    pub fn stream_class(&self) -> StreamClassification {
        match self {
            // Semantic stream: tool calls, lifecycle, errors, observability
            Self::ToolCallStart { .. }
            | Self::ToolCallDelta { .. }
            | Self::ToolCallCompleted { .. }
            | Self::ToolResult { .. }
            | Self::StreamStarted { .. }
            | Self::StreamCompleted { .. }
            | Self::StreamFailed { .. }
            | Self::StreamCancelled { .. }
            | Self::StreamResumed { .. }
            | Self::Error { .. }
            | Self::UsageUpdate { .. }
            | Self::LatencyMarker { .. }
            | Self::RoutingDecision { .. } => StreamClassification::Semantic,

            // UI stream: text content, heartbeats, unknown
            Self::TextDelta { .. }
            | Self::TextCompleted { .. }
            | Self::TextAnnotation { .. }
            | Self::Heartbeat { .. }
            | Self::Unknown => StreamClassification::Ui,
        }
    }
}

/// Per-event-class backpressure behavior for the dual-stream architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum BackpressurePolicy {
    /// Must deliver; apply backpressure to sender (JetStream ack).
    Lossless,
    /// May merge consecutive events of same type under pressure.
    Coalescible,
    /// May drop under extreme pressure (heartbeats, progress indicators).
    Droppable,
}

/// Stream classification for routing events to the correct delivery channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamClassification {
    /// Lossless delivery via JetStream.
    Semantic,
    /// Best-effort delivery via NATS Core.
    Ui,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip_text_delta() {
        let event = ModelEvent::TextDelta { text: "hello".into() };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ModelEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn serde_round_trip_stream_started() {
        let event = ModelEvent::StreamStarted {
            model_id: "gpt-4".into(),
            request_id: "req-123".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ModelEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn serde_round_trip_tool_call_completed() {
        let event = ModelEvent::ToolCallCompleted {
            call_id: "call-1".into(),
            name: "search".into(),
            input: serde_json::json!({"query": "test"}),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ModelEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn serde_round_trip_stream_completed() {
        let event = ModelEvent::StreamCompleted {
            usage: Usage::new(100, 50),
            stop_reason: StopReason::Completed,
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ModelEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    #[test]
    fn unknown_variant_via_serde_other() {
        // Simulate a future event type that this version doesn't know about
        let json = r#"{"event_type":"future_event_v2","data":"something"}"#;
        let event: ModelEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event, ModelEvent::Unknown);
    }

    #[test]
    fn backpressure_policy_mapping() {
        assert_eq!(
            ModelEvent::ToolCallStart { call_id: "c".into(), name: "t".into() }.backpressure_policy(),
            BackpressurePolicy::Lossless
        );
        assert_eq!(
            ModelEvent::TextDelta { text: "hi".into() }.backpressure_policy(),
            BackpressurePolicy::Coalescible
        );
        assert_eq!(
            ModelEvent::Heartbeat { sequence: 1 }.backpressure_policy(),
            BackpressurePolicy::Droppable
        );
    }

    #[test]
    fn stream_class_mapping() {
        assert_eq!(
            ModelEvent::ToolCallStart { call_id: "c".into(), name: "t".into() }.stream_class(),
            StreamClassification::Semantic
        );
        assert_eq!(
            ModelEvent::TextDelta { text: "hi".into() }.stream_class(),
            StreamClassification::Ui
        );
        assert_eq!(
            ModelEvent::Heartbeat { sequence: 1 }.stream_class(),
            StreamClassification::Ui
        );
    }

    #[test]
    fn all_variants_serialize() {
        let variants: Vec<ModelEvent> = vec![
            ModelEvent::StreamStarted { model_id: "m".into(), request_id: "r".into() },
            ModelEvent::StreamCompleted { usage: Usage::default(), stop_reason: StopReason::Completed },
            ModelEvent::StreamFailed { error: "err".into(), recoverable: false },
            ModelEvent::StreamCancelled { reason: "cancel".into() },
            ModelEvent::StreamResumed { from_checkpoint: "cp".into() },
            ModelEvent::TextDelta { text: "t".into() },
            ModelEvent::TextCompleted { full_text: "full".into() },
            ModelEvent::TextAnnotation { annotation: serde_json::json!({}) },
            ModelEvent::ToolCallStart { call_id: "c".into(), name: "n".into() },
            ModelEvent::ToolCallDelta { call_id: "c".into(), input_chunk: "i".into() },
            ModelEvent::ToolCallCompleted { call_id: "c".into(), name: "n".into(), input: serde_json::json!({}) },
            ModelEvent::ToolResult { call_id: "c".into(), result: serde_json::json!({}), error: None },
            ModelEvent::UsageUpdate { usage: Usage::default() },
            ModelEvent::LatencyMarker { checkpoint: "cp".into(), elapsed_ms: 100 },
            ModelEvent::RoutingDecision { model_id: "m".into(), tier: "t".into(), reason: "r".into() },
            ModelEvent::Error { code: "E001".into(), message: "msg".into(), recoverable: true },
            ModelEvent::Heartbeat { sequence: 42 },
            ModelEvent::Unknown,
        ];

        for variant in &variants {
            let json = serde_json::to_string(variant).unwrap();
            assert!(!json.is_empty(), "Failed to serialize: {:?}", variant);
            // Round-trip (skip Unknown as it has special deserialization)
            if !matches!(variant, ModelEvent::Unknown) {
                let round_tripped: ModelEvent = serde_json::from_str(&json).unwrap();
                assert_eq!(variant, &round_tripped, "Round-trip failed for: {json}");
            }
        }
    }
}
