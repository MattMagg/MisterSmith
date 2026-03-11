//! Heuristic stream monitor for step boundaries and degradation signals.

use mister_smith_core::{SemanticSignal, SemanticSignalKind};

use crate::model_event::{ModelEvent, StepBoundary};

/// Runtime heuristics used by the stream monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamMonitorConfig {
    /// Number of consecutive idle heartbeats allowed before reporting a stall.
    pub max_idle_heartbeats: u64,
    /// Number of repeated text deltas allowed before reporting repetition.
    pub repetitive_delta_threshold: usize,
}

impl Default for StreamMonitorConfig {
    fn default() -> Self {
        Self {
            max_idle_heartbeats: 3,
            repetitive_delta_threshold: 3,
        }
    }
}

/// Signals surfaced after ingesting a single model event.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StreamObservation {
    /// Step boundaries exposed by the latest event.
    pub step_boundaries: Vec<StepBoundary>,
    /// Degradation signals inferred from the latest event and local history.
    pub degradation_signals: Vec<SemanticSignal>,
}

/// Stateful stream monitor consuming canonical `ModelEvent` items.
#[derive(Debug, Clone)]
pub struct StreamMonitor {
    config: StreamMonitorConfig,
    idle_heartbeats: u64,
    last_text_delta: Option<String>,
    repeated_text_deltas: usize,
    last_step_boundary: Option<StepBoundary>,
}

impl StreamMonitor {
    /// Create a monitor with the given runtime heuristics.
    pub fn new(config: StreamMonitorConfig) -> Self {
        Self {
            config,
            idle_heartbeats: 0,
            last_text_delta: None,
            repeated_text_deltas: 0,
            last_step_boundary: None,
        }
    }

    /// Observe a single stream event and emit boundaries plus degradation signals.
    pub fn observe(&mut self, event: &ModelEvent) -> StreamObservation {
        let mut observation = StreamObservation::default();

        if let Some(boundary) = event.step_boundary() {
            self.last_step_boundary = Some(boundary.clone());
            observation.step_boundaries.push(boundary);
            self.reset_progress_state();
        }

        if let Some(signal) = event.degradation_signal() {
            observation.degradation_signals.push(signal);
        }

        match event {
            ModelEvent::Heartbeat { .. } => {
                self.idle_heartbeats += 1;
                if self.idle_heartbeats >= self.config.max_idle_heartbeats {
                    observation.degradation_signals.push(SemanticSignal {
                        signal_kind: SemanticSignalKind::Stalled,
                        severity: 75,
                        detail: format!(
                            "stream observed {} idle heartbeats without progress",
                            self.idle_heartbeats
                        ),
                    });
                    self.idle_heartbeats = 0;
                }
            }
            ModelEvent::TextDelta { text } => {
                self.idle_heartbeats = 0;
                if self.last_text_delta.as_deref() == Some(text.as_str()) {
                    self.repeated_text_deltas += 1;
                } else {
                    self.last_text_delta = Some(text.clone());
                    self.repeated_text_deltas = 1;
                }

                if self.repeated_text_deltas >= self.config.repetitive_delta_threshold {
                    observation.degradation_signals.push(SemanticSignal {
                        signal_kind: SemanticSignalKind::Repetitive,
                        severity: 78,
                        detail: format!(
                            "stream repeated the same text delta {} times",
                            self.repeated_text_deltas
                        ),
                    });
                    self.repeated_text_deltas = 0;
                }
            }
            ModelEvent::TextCompleted { .. }
            | ModelEvent::TextAnnotation { .. }
            | ModelEvent::ToolCallStart { .. }
            | ModelEvent::ToolCallDelta { .. }
            | ModelEvent::ToolCallCompleted { .. }
            | ModelEvent::ToolResult { .. }
            | ModelEvent::UsageUpdate { .. }
            | ModelEvent::LatencyMarker { .. }
            | ModelEvent::RoutingDecision { .. } => {
                self.idle_heartbeats = 0;
            }
            ModelEvent::StreamStarted { .. }
            | ModelEvent::StreamCompleted { .. }
            | ModelEvent::StreamFailed { .. }
            | ModelEvent::StreamCancelled { .. }
            | ModelEvent::StreamResumed { .. } => {
                self.reset_progress_state();
            }
            ModelEvent::Error { .. } | ModelEvent::Unknown => {}
        }

        if !observation.degradation_signals.is_empty() && observation.step_boundaries.is_empty() {
            if let Some(boundary) = self.last_step_boundary.clone() {
                observation.step_boundaries.push(boundary);
            }
        }

        observation
    }

    fn reset_progress_state(&mut self) {
        self.idle_heartbeats = 0;
        self.last_text_delta = None;
        self.repeated_text_deltas = 0;
    }
}
