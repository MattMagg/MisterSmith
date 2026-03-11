use mister_smith_core::SemanticSignalKind;
use mister_smith_llm::{
    ModelEvent, StepBoundary, StopReason, StreamMonitor, StreamMonitorConfig, Usage,
};

#[test]
fn stream_monitor_emits_boundaries_for_tooling_and_completion() {
    let mut monitor = StreamMonitor::new(StreamMonitorConfig::default());

    let started = monitor.observe(&ModelEvent::StreamStarted {
        model_id: "gpt-test".to_string(),
        request_id: "req-1".to_string(),
    });
    assert_eq!(started.step_boundaries, vec![StepBoundary::StreamStarted]);

    let tool = monitor.observe(&ModelEvent::ToolCallCompleted {
        call_id: "call-1".to_string(),
        name: "search".to_string(),
        input: serde_json::json!({"q": "guard"}),
    });
    assert_eq!(
        tool.step_boundaries,
        vec![StepBoundary::ToolCallCompleted {
            call_id: "call-1".to_string(),
            name: "search".to_string(),
        }]
    );

    let completed = monitor.observe(&ModelEvent::StreamCompleted {
        usage: Usage::new(10, 5),
        stop_reason: StopReason::Completed,
    });
    assert_eq!(
        completed.step_boundaries,
        vec![StepBoundary::StreamCompleted]
    );
}

#[test]
fn stream_monitor_detects_stall_from_idle_heartbeats() {
    let mut monitor = StreamMonitor::new(StreamMonitorConfig {
        max_idle_heartbeats: 2,
        repetitive_delta_threshold: 3,
    });

    monitor.observe(&ModelEvent::StreamStarted {
        model_id: "gpt-test".to_string(),
        request_id: "req-2".to_string(),
    });
    monitor.observe(&ModelEvent::Heartbeat { sequence: 1 });
    let stalled = monitor.observe(&ModelEvent::Heartbeat { sequence: 2 });

    assert_eq!(stalled.degradation_signals.len(), 1);
    assert_eq!(
        stalled.degradation_signals[0].signal_kind,
        SemanticSignalKind::Stalled
    );
}

#[test]
fn stream_monitor_carries_forward_last_step_boundary_when_stall_is_detected() {
    let mut monitor = StreamMonitor::new(StreamMonitorConfig {
        max_idle_heartbeats: 2,
        repetitive_delta_threshold: 3,
    });

    let started = monitor.observe(&ModelEvent::StreamStarted {
        model_id: "gpt-test".to_string(),
        request_id: "req-3".to_string(),
    });
    assert_eq!(started.step_boundaries, vec![StepBoundary::StreamStarted]);

    monitor.observe(&ModelEvent::Heartbeat { sequence: 1 });
    let stalled = monitor.observe(&ModelEvent::Heartbeat { sequence: 2 });

    assert_eq!(stalled.step_boundaries, vec![StepBoundary::StreamStarted]);
    assert_eq!(stalled.degradation_signals.len(), 1);
    assert_eq!(
        stalled.degradation_signals[0].signal_kind,
        SemanticSignalKind::Stalled
    );
}

#[test]
fn stream_monitor_detects_repetitive_text_deltas() {
    let mut monitor = StreamMonitor::new(StreamMonitorConfig {
        max_idle_heartbeats: 3,
        repetitive_delta_threshold: 2,
    });

    monitor.observe(&ModelEvent::TextDelta {
        text: "same answer".to_string(),
    });
    let repetitive = monitor.observe(&ModelEvent::TextDelta {
        text: "same answer".to_string(),
    });

    assert_eq!(repetitive.degradation_signals.len(), 1);
    assert_eq!(
        repetitive.degradation_signals[0].signal_kind,
        SemanticSignalKind::Repetitive
    );
}
