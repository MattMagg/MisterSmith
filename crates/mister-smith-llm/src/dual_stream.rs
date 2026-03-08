use tokio::sync::mpsc;

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
    active_tool_calls: std::collections::HashMap<String, String>,
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

        let handle = DualStreamHandle {
            semantic_rx,
            ui_rx,
        };

        (actor, handle)
    }

    /// Convert a StreamChunk to one or more ModelEvents and route them.
    pub async fn process_chunk(
        &mut self,
        chunk: StreamChunk,
        model_id: &str,
        request_id: &str,
    ) {
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
                self.active_tool_calls.insert(call_id.clone(), String::new());
                vec![ModelEvent::ToolCallStart { call_id, name }]
            }
            ChunkDelta::ToolUseInput { call_id, input } => {
                // Accumulate input for the tool call
                if let Some(accumulated) = self.active_tool_calls.get_mut(&call_id) {
                    accumulated.push_str(&input.to_string());
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
                    .map(|(call_id, accumulated_input)| {
                        let input = serde_json::from_str(&accumulated_input)
                            .unwrap_or_else(|_| serde_json::json!(accumulated_input));
                        ModelEvent::ToolCallCompleted {
                            call_id,
                            name: String::new(), // Name was in the Start event
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
        if let Some(text) = self.pending_text.take() {
            let _ = self.ui_tx.try_send(ModelEvent::TextDelta { text });
            self.coalesce_count = 0;
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

        actor.process_chunk(
            StreamChunk {
                index: 0,
                delta: ChunkDelta::Text { text: "hello".into() },
            },
            "test-model",
            "req-1",
        ).await;

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

        actor.process_chunk(
            StreamChunk {
                index: 0,
                delta: ChunkDelta::ToolUseStart {
                    call_id: "call-1".into(),
                    name: "search".into(),
                },
            },
            "test-model",
            "req-1",
        ).await;

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

        actor.process_chunk(
            StreamChunk::stop(0, StopReason::Completed),
            "test-model",
            "req-1",
        ).await;

        let event = handle.semantic_rx.try_recv();
        assert!(event.is_ok());
        assert!(matches!(
            event.unwrap(),
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
        actor.route_event(ModelEvent::Heartbeat { sequence: 1 }).await;
        // This should be silently dropped (Droppable policy + full channel)
        actor.route_event(ModelEvent::Heartbeat { sequence: 2 }).await;
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

        // Send multiple text deltas — should coalesce
        for i in 0..3 {
            actor.process_chunk(
                StreamChunk {
                    index: i,
                    delta: ChunkDelta::Text { text: format!("part{i}") },
                },
                "test-model",
                "req-1",
            ).await;
        }

        // The coalesced text should be a single event
        let event = handle.ui_rx.try_recv();
        assert!(event.is_ok());
        if let ModelEvent::TextDelta { text } = event.unwrap() {
            assert!(text.contains("part0"));
            assert!(text.contains("part1"));
            assert!(text.contains("part2"));
        } else {
            panic!("Expected TextDelta");
        }
    }
}
