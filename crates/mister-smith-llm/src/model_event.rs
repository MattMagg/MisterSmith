use serde::{Deserialize, Serialize};

use mister_smith_core::{SemanticSignal, SemanticSignalKind};

use crate::routing_signal::StepRoutingSignal;
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
    StreamFailed { error: String, recoverable: bool },
    /// Stream was cancelled.
    StreamCancelled { reason: String },
    /// Stream resumed from a checkpoint.
    StreamResumed { from_checkpoint: String },

    // -- Text (3) --
    /// Incremental text content.
    TextDelta { text: String },
    /// Complete assembled text block.
    TextCompleted { full_text: String },
    /// Text annotation metadata.
    TextAnnotation { annotation: serde_json::Value },

    // -- Tool Call (4) --
    /// Model initiated a tool call.
    ToolCallStart { call_id: String, name: String },
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
    UsageUpdate { usage: Usage },
    /// Latency checkpoint marker.
    LatencyMarker { checkpoint: String, elapsed_ms: u64 },
    /// Routing decision for observability.
    RoutingDecision {
        model_id: String,
        tier: String,
        reason: String,
        #[serde(default)]
        step_signal: StepRoutingSignal,
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
    Heartbeat { sequence: u64 },

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

/// Typed execution boundary surfaced from a streaming model event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepBoundary {
    /// A stream has started.
    StreamStarted,
    /// The stream resumed from a known checkpoint.
    StreamResumed { checkpoint: String },
    /// A tool call completed and can define a step boundary.
    ToolCallCompleted { call_id: String, name: String },
    /// A tool result completed and can define a step boundary.
    ToolResult { call_id: String },
    /// A latency marker exposed a named boundary.
    LatencyCheckpoint { checkpoint: String },
    /// A stream has completed.
    StreamCompleted,
}

impl ModelEvent {
    /// Return a typed step boundary when this event marks one.
    pub fn step_boundary(&self) -> Option<StepBoundary> {
        match self {
            Self::StreamStarted { .. } => Some(StepBoundary::StreamStarted),
            Self::StreamResumed { from_checkpoint } => Some(StepBoundary::StreamResumed {
                checkpoint: from_checkpoint.clone(),
            }),
            Self::ToolCallCompleted { call_id, name, .. } => {
                Some(StepBoundary::ToolCallCompleted {
                    call_id: call_id.clone(),
                    name: name.clone(),
                })
            }
            Self::ToolResult { call_id, .. } => Some(StepBoundary::ToolResult {
                call_id: call_id.clone(),
            }),
            Self::LatencyMarker { checkpoint, .. } => Some(StepBoundary::LatencyCheckpoint {
                checkpoint: checkpoint.clone(),
            }),
            Self::StreamCompleted { .. } => Some(StepBoundary::StreamCompleted),
            _ => None,
        }
    }

    /// Convert directly observable event failures into coarse degradation signals.
    pub fn degradation_signal(&self) -> Option<SemanticSignal> {
        match self {
            Self::StreamFailed { error, recoverable } => Some(SemanticSignal {
                signal_kind: SemanticSignalKind::Stalled,
                severity: if *recoverable { 80 } else { 95 },
                detail: error.clone(),
            }),
            Self::StreamCancelled { reason } => Some(SemanticSignal {
                signal_kind: SemanticSignalKind::Stalled,
                severity: 70,
                detail: reason.clone(),
            }),
            Self::Error {
                code,
                message,
                recoverable,
            } => {
                let detail = format!("{code}: {message}");
                let lowercase = detail.to_ascii_lowercase();
                let signal_kind = if lowercase.contains("context") || lowercase.contains("memory") {
                    SemanticSignalKind::MissingContext
                } else if lowercase.contains("policy")
                    || lowercase.contains("auth")
                    || lowercase.contains("permission")
                {
                    SemanticSignalKind::PolicyConflict
                } else if *recoverable {
                    SemanticSignalKind::Stalled
                } else {
                    SemanticSignalKind::LowConfidence
                };

                Some(SemanticSignal {
                    signal_kind,
                    severity: if *recoverable { 72 } else { 82 },
                    detail,
                })
            }
            Self::TextAnnotation { annotation } => annotation
                .get("confidence")
                .and_then(serde_json::Value::as_f64)
                .filter(|confidence| *confidence < 0.5)
                .map(|confidence| SemanticSignal {
                    signal_kind: SemanticSignalKind::LowConfidence,
                    severity: ((1.0 - confidence) * 100.0).round() as u8,
                    detail: format!("stream annotation confidence dropped to {confidence:.2}"),
                }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip_text_delta() {
        let event = ModelEvent::TextDelta {
            text: "hello".into(),
        };
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
            ModelEvent::ToolCallStart {
                call_id: "c".into(),
                name: "t".into()
            }
            .backpressure_policy(),
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
            ModelEvent::ToolCallStart {
                call_id: "c".into(),
                name: "t".into()
            }
            .stream_class(),
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
    fn step_boundary_mapping() {
        assert_eq!(
            ModelEvent::StreamStarted {
                model_id: "gpt-test".into(),
                request_id: "req-1".into(),
            }
            .step_boundary(),
            Some(StepBoundary::StreamStarted)
        );
        assert_eq!(
            ModelEvent::ToolCallCompleted {
                call_id: "call-1".into(),
                name: "search".into(),
                input: serde_json::json!({"q": "guard"}),
            }
            .step_boundary(),
            Some(StepBoundary::ToolCallCompleted {
                call_id: "call-1".into(),
                name: "search".into(),
            })
        );
    }

    #[test]
    fn degradation_signal_mapping() {
        let signal = ModelEvent::StreamFailed {
            error: "stream stalled".into(),
            recoverable: true,
        }
        .degradation_signal()
        .expect("stream failure should produce a degradation signal");

        assert_eq!(signal.signal_kind, SemanticSignalKind::Stalled);
        assert_eq!(signal.severity, 80);
    }

    #[test]
    fn all_variants_serialize() {
        let variants: Vec<ModelEvent> = vec![
            ModelEvent::StreamStarted {
                model_id: "m".into(),
                request_id: "r".into(),
            },
            ModelEvent::StreamCompleted {
                usage: Usage::default(),
                stop_reason: StopReason::Completed,
            },
            ModelEvent::StreamFailed {
                error: "err".into(),
                recoverable: false,
            },
            ModelEvent::StreamCancelled {
                reason: "cancel".into(),
            },
            ModelEvent::StreamResumed {
                from_checkpoint: "cp".into(),
            },
            ModelEvent::TextDelta { text: "t".into() },
            ModelEvent::TextCompleted {
                full_text: "full".into(),
            },
            ModelEvent::TextAnnotation {
                annotation: serde_json::json!({}),
            },
            ModelEvent::ToolCallStart {
                call_id: "c".into(),
                name: "n".into(),
            },
            ModelEvent::ToolCallDelta {
                call_id: "c".into(),
                input_chunk: "i".into(),
            },
            ModelEvent::ToolCallCompleted {
                call_id: "c".into(),
                name: "n".into(),
                input: serde_json::json!({}),
            },
            ModelEvent::ToolResult {
                call_id: "c".into(),
                result: serde_json::json!({}),
                error: None,
            },
            ModelEvent::UsageUpdate {
                usage: Usage::default(),
            },
            ModelEvent::LatencyMarker {
                checkpoint: "cp".into(),
                elapsed_ms: 100,
            },
            ModelEvent::RoutingDecision {
                model_id: "m".into(),
                tier: "t".into(),
                reason: "r".into(),
                step_signal: StepRoutingSignal {
                    metadata: crate::routing_signal::StepRoutingMetadata {
                        step_id: "completion.request".into(),
                        step_index: None,
                        step_kind: Some("completion".into()),
                    },
                    action: crate::routing_signal::StepRoutingAction::Continue,
                    confidence: None,
                    checkpoints: vec![],
                },
            },
            ModelEvent::Error {
                code: "E001".into(),
                message: "msg".into(),
                recoverable: true,
            },
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

    #[test]
    fn routing_decision_deserializes_legacy_payload_without_step_signal() {
        let legacy = serde_json::json!({
            "event_type": "routing_decision",
            "model_id": "legacy-model",
            "tier": "direct",
            "reason": "legacy payload"
        });

        let event: ModelEvent = serde_json::from_value(legacy).unwrap();

        match event {
            ModelEvent::RoutingDecision {
                model_id,
                tier,
                reason,
                step_signal,
            } => {
                assert_eq!(model_id, "legacy-model");
                assert_eq!(tier, "direct");
                assert_eq!(reason, "legacy payload");
                assert_eq!(step_signal.metadata.step_id, "completion.request");
                assert_eq!(
                    step_signal.metadata.step_kind.as_deref(),
                    Some("completion")
                );
                assert_eq!(step_signal.action, crate::routing_signal::StepRoutingAction::Continue);
                assert!(step_signal.confidence.is_none());
                assert!(step_signal.checkpoints.is_empty());
            }
            other => panic!("expected RoutingDecision, got: {other:?}"),
        }
    }
}
