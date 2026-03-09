use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;

use crate::model_event::{BackpressurePolicy, ModelEvent, StreamClassification};
use crate::streaming::{ChunkDelta, StreamChunk};
use crate::types::Usage;

/// Configuration for the dual-stream actor.
#[derive(Debug, Clone)]
pub struct DualStreamConfig {
    /// Semantic stream channel capacity (lossless — will apply backpressure).
    pub semantic_capacity: usize,
    /// UI stream channel capacity (best-effort — may drop under pressure).
    pub ui_capacity: usize,
    /// Maximum number of consecutive text deltas to coalesce under backpressure.
    pub max_coalesce_count: usize,
}

impl Default for DualStreamConfig {
    fn default() -> Self {
        Self {
            semantic_capacity: 256,
            ui_capacity: 64,
            max_coalesce_count: 10,
        }
    }
}

/// Dual-stream handle providing access to both semantic and UI event receivers.
pub struct DualStreamHandle {
    pub semantic_rx: mpsc::Receiver<ModelEvent>,
    pub ui_rx: mpsc::Receiver<ModelEvent>,
}

/// Dual-stream actor that converts `StreamChunk` items to `ModelEvent` items
/// and routes them to the appropriate delivery channel.
pub struct DualStreamActor {
    config: DualStreamConfig,
    semantic_tx: mpsc::Sender<ModelEvent>,
    ui_tx: mpsc::Sender<ModelEvent>,
    // Text coalescing state for UI stream under backpressure
    pending_text: Option<String>,
    coalesce_count: usize,
    // Tool call assembly state
    active_tool_calls: std::collections::HashMap<String, ActiveToolCall>,
}

#[derive(Debug, Clone)]
struct ActiveToolCall {
    name: String,
    accumulated_input: String,
}

impl DualStreamActor {
    /// Create a new dual-stream actor with channels.
    pub fn new(config: DualStreamConfig) -> (Self, DualStreamHandle) {
        let (semantic_tx, semantic_rx) = mpsc::channel(config.semantic_capacity);
        let (ui_tx, ui_rx) = mpsc::channel(config.ui_capacity);

        let actor = Self {
            config,
            semantic_tx,
            ui_tx,
            pending_text: None,
            coalesce_count: 0,
            active_tool_calls: std::collections::HashMap::new(),
        };

        let handle = DualStreamHandle { semantic_rx, ui_rx };

        (actor, handle)
    }

    /// Convert a StreamChunk to one or more ModelEvents and route them.
    pub async fn process_chunk(&mut self, chunk: StreamChunk, model_id: &str, request_id: &str) {
        let events = self.convert_chunk(chunk, model_id, request_id);
        for event in events {
            self.route_event(event).await;
        }
    }

    /// Convert a StreamChunk into ModelEvent(s).
    fn convert_chunk(
        &mut self,
        chunk: StreamChunk,
        _model_id: &str,
        _request_id: &str,
    ) -> Vec<ModelEvent> {
        match chunk.delta {
            ChunkDelta::Text { text } => {
                vec![ModelEvent::TextDelta { text }]
            }
            ChunkDelta::ToolUseStart { call_id, name } => {
                self.active_tool_calls.insert(
                    call_id.clone(),
                    ActiveToolCall {
                        name: name.clone(),
                        accumulated_input: String::new(),
                    },
                );
                vec![ModelEvent::ToolCallStart { call_id, name }]
            }
            ChunkDelta::ToolUseInput { call_id, input } => {
                // Accumulate input for the tool call
                if let Some(accumulated) = self.active_tool_calls.get_mut(&call_id) {
                    accumulated.accumulated_input.push_str(&input.to_string());
                }
                vec![ModelEvent::ToolCallDelta {
                    call_id,
                    input_chunk: input.to_string(),
                }]
            }
            ChunkDelta::Stop { reason } => {
                // Emit completed events for any active tool calls
                let mut events: Vec<ModelEvent> = self
                    .active_tool_calls
                    .drain()
                    .map(|(call_id, active_tool_call)| {
                        let input = serde_json::from_str(&active_tool_call.accumulated_input)
                            .unwrap_or_else(|_| {
                                serde_json::json!(active_tool_call.accumulated_input)
                            });
                        ModelEvent::ToolCallCompleted {
                            call_id,
                            name: active_tool_call.name,
                            input,
                        }
                    })
                    .collect();

                events.push(ModelEvent::StreamCompleted {
                    usage: Usage::default(),
                    stop_reason: reason,
                });

                events
            }
        }
    }

    /// Route an event to the appropriate stream based on its classification.
    async fn route_event(&mut self, event: ModelEvent) {
        let policy = event.backpressure_policy();
        let classification = event.stream_class();

        match classification {
            StreamClassification::Semantic => {
                // Lossless delivery — must not drop
                match policy {
                    BackpressurePolicy::Lossless => {
                        // Use blocking send for guaranteed delivery
                        let _ = self.semantic_tx.send(event).await;
                    }
                    BackpressurePolicy::Coalescible => {
                        // Still deliver to semantic, but can coalesce if needed
                        let _ = self.semantic_tx.send(event).await;
                    }
                    BackpressurePolicy::Droppable => {
                        // Shouldn't happen for semantic events, but handle gracefully
                        let _ = self.semantic_tx.try_send(event);
                    }
                }
            }
            StreamClassification::Ui => {
                match policy {
                    BackpressurePolicy::Coalescible => {
                        // Text deltas can be coalesced under backpressure
                        if let ModelEvent::TextDelta { ref text } = event {
                            if self.pending_text.is_none() {
                                match self.ui_tx.try_send(event.clone()) {
                                    Ok(()) => return,
                                    Err(TrySendError::Full(_)) => {
                                        // Start coalescing only after backpressure is observed.
                                    }
                                    Err(TrySendError::Closed(_)) => return,
                                }
                            }

                            self.coalesce_count += 1;
                            if let Some(pending) = &mut self.pending_text {
                                pending.push_str(text);
                            } else {
                                self.pending_text = Some(text.clone());
                            }

                            // Flush if we've coalesced enough
                            if self.coalesce_count >= self.config.max_coalesce_count {
                                self.flush_pending_text().await;
                            }
                        } else {
                            // Flush any pending text first, then send this event
                            self.flush_pending_text().await;
                            let _ = self.ui_tx.try_send(event);
                        }
                    }
                    BackpressurePolicy::Droppable => {
                        // Best-effort: try_send, drop if full
                        let _ = self.ui_tx.try_send(event);
                    }
                    BackpressurePolicy::Lossless => {
                        // Shouldn't happen for UI events, but deliver best-effort
                        let _ = self.ui_tx.try_send(event);
                    }
                }
            }
        }
    }

    /// Flush accumulated text delta to the UI stream.
    async fn flush_pending_text(&mut self) {
        if let Some(text) = self.pending_text.as_ref() {
            if self
                .ui_tx
                .try_send(ModelEvent::TextDelta { text: text.clone() })
                .is_ok()
            {
                self.pending_text = None;
                self.coalesce_count = 0;
            }
        }
    }

    /// Signal stream completion and flush any remaining state.
    pub async fn finish(&mut self) {
        self.flush_pending_text().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StopReason;

    #[tokio::test]
    async fn text_delta_routes_to_ui_stream() {
        let (mut actor, mut handle) = DualStreamActor::new(DualStreamConfig::default());

        actor
            .process_chunk(
                StreamChunk {
                    index: 0,
                    delta: ChunkDelta::Text {
                        text: "hello".into(),
                    },
                },
                "test-model",
                "req-1",
            )
            .await;

        // Flush to ensure delivery
        actor.finish().await;

        // Text should appear on UI stream
        let event = handle.ui_rx.try_recv();
        assert!(event.is_ok());
        assert!(matches!(event.unwrap(), ModelEvent::TextDelta { text } if text == "hello"));
    }

    #[tokio::test]
    async fn tool_call_routes_to_semantic_stream() {
        let (mut actor, mut handle) = DualStreamActor::new(DualStreamConfig::default());

        actor
            .process_chunk(
                StreamChunk {
                    index: 0,
                    delta: ChunkDelta::ToolUseStart {
                        call_id: "call-1".into(),
                        name: "search".into(),
                    },
                },
                "test-model",
                "req-1",
            )
            .await;

        let event = handle.semantic_rx.try_recv();
        assert!(event.is_ok());
        assert!(matches!(
            event.unwrap(),
            ModelEvent::ToolCallStart { call_id, name } if call_id == "call-1" && name == "search"
        ));
    }

    #[tokio::test]
    async fn stop_chunk_produces_stream_completed() {
        let (mut actor, mut handle) = DualStreamActor::new(DualStreamConfig::default());

        actor
            .process_chunk(
                StreamChunk::stop(0, StopReason::Completed),
                "test-model",
                "req-1",
            )
            .await;

        let event = handle.semantic_rx.try_recv();
        assert!(event.is_ok());
        assert!(matches!(
            event.unwrap(),
            ModelEvent::StreamCompleted { stop_reason, .. } if stop_reason == StopReason::Completed
        ));
    }

    #[tokio::test]
    async fn tool_call_completed_preserves_name_and_input() {
        let (mut actor, mut handle) = DualStreamActor::new(DualStreamConfig::default());

        actor
            .process_chunk(
                StreamChunk {
                    index: 0,
                    delta: ChunkDelta::ToolUseStart {
                        call_id: "call-1".into(),
                        name: "search".into(),
                    },
                },
                "test-model",
                "req-1",
            )
            .await;

        actor
            .process_chunk(
                StreamChunk {
                    index: 1,
                    delta: ChunkDelta::ToolUseInput {
                        call_id: "call-1".into(),
                        input: serde_json::json!({"query": "rust"}),
                    },
                },
                "test-model",
                "req-1",
            )
            .await;

        actor
            .process_chunk(
                StreamChunk::stop(2, StopReason::Completed),
                "test-model",
                "req-1",
            )
            .await;

        let start_event = handle.semantic_rx.try_recv().unwrap();
        assert!(matches!(
            start_event,
            ModelEvent::ToolCallStart { call_id, name } if call_id == "call-1" && name == "search"
        ));

        let delta_event = handle.semantic_rx.try_recv().unwrap();
        assert!(matches!(
            delta_event,
            ModelEvent::ToolCallDelta { call_id, input_chunk } if call_id == "call-1" && input_chunk == r#"{"query":"rust"}"#
        ));

        let completed_event = handle.semantic_rx.try_recv().unwrap();
        assert!(matches!(
            completed_event,
            ModelEvent::ToolCallCompleted { call_id, name, input }
                if call_id == "call-1" && name == "search" && input == serde_json::json!({"query": "rust"})
        ));

        let stream_event = handle.semantic_rx.try_recv().unwrap();
        assert!(matches!(
            stream_event,
            ModelEvent::StreamCompleted { stop_reason, .. } if stop_reason == StopReason::Completed
        ));
    }

    #[tokio::test]
    async fn heartbeats_dropped_when_channel_full() {
        let config = DualStreamConfig {
            ui_capacity: 1,
            ..Default::default()
        };
        let (mut actor, _handle) = DualStreamActor::new(config);

        // Fill the UI channel
        actor
            .route_event(ModelEvent::Heartbeat { sequence: 1 })
            .await;
        // This should be silently dropped (Droppable policy + full channel)
        actor
            .route_event(ModelEvent::Heartbeat { sequence: 2 })
            .await;
        // No panic = success
    }

    #[tokio::test]
    async fn text_coalescing_under_pressure() {
        let config = DualStreamConfig {
            ui_capacity: 1,
            max_coalesce_count: 3,
            ..Default::default()
        };
        let (mut actor, mut handle) = DualStreamActor::new(config);

        // First text is immediately passed through while there is capacity.
        actor
            .route_event(ModelEvent::TextDelta {
                text: "part0".into(),
            })
            .await;

        let first = handle.ui_rx.try_recv();
        assert!(matches!(first, Ok(ModelEvent::TextDelta { text }) if text == "part0"));

        // Saturate the UI channel so coalescing is required.
        actor
            .route_event(ModelEvent::Heartbeat { sequence: 1 })
            .await;

        // Send multiple text deltas — these should now coalesce.
        for i in 0..3 {
            actor
                .route_event(ModelEvent::TextDelta {
                    text: format!("part{}", i + 1),
                })
                .await;
        }

        // Channel remains full with the heartbeat; coalesced payload is buffered.
        assert!(matches!(
            handle.ui_rx.try_recv(),
            Ok(ModelEvent::Heartbeat { sequence: 1 })
        ));

        // Freeing capacity allows the pending coalesced payload to flush.
        actor.finish().await;

        let event = handle.ui_rx.try_recv();
        assert!(matches!(event, Ok(ModelEvent::TextDelta { text }) if text == "part1part2part3"));
    }

    #[tokio::test]
    async fn coalesced_text_is_not_lost_when_flush_is_temporarily_full() {
        let config = DualStreamConfig {
            ui_capacity: 1,
            max_coalesce_count: 1,
            ..Default::default()
        };
        let (mut actor, mut handle) = DualStreamActor::new(config);

        // Fill the UI channel and force the first text delta into pending coalesced state.
        actor
            .route_event(ModelEvent::Heartbeat { sequence: 1 })
            .await;
        actor
            .route_event(ModelEvent::TextDelta {
                text: "hello".into(),
            })
            .await;

        // Flush attempt while full should keep pending text buffered.
        actor.finish().await;
        assert!(matches!(
            handle.ui_rx.try_recv(),
            Ok(ModelEvent::Heartbeat { sequence: 1 })
        ));

        // A second flush now succeeds and delivers the original buffered text.
        actor.finish().await;
        assert!(
            matches!(handle.ui_rx.try_recv(), Ok(ModelEvent::TextDelta { text }) if text == "hello")
        );
    }
}
