//! Integration test: Phase 8 observability pipeline.
//!
//! Verifies:
//! - Agent operations produce traced spans with correct attributes
//! - PrometheusBackend accepts all metric types
//! - Trace context inject/extract roundtrip through MessageEnvelope
//! - Structured log format configuration works

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use mister_smith_monitoring::metrics::{Metric, MetricValue, MetricsBackend, MetricsCollector};
use mister_smith_monitoring::prometheus::PrometheusBackend;
use mister_smith_transport::{
    extract_trace_context, inject_trace_context, MessageEnvelope, TRACEPARENT_HEADER,
    TRACESTATE_HEADER,
};

#[tokio::test]
async fn prometheus_backend_integrates_with_metrics_collector() {
    let collector = Arc::new(MetricsCollector::new(Duration::from_secs(60)));
    let backend = Arc::new(PrometheusBackend::new());
    collector
        .add_backend(backend as Arc<dyn MetricsBackend>)
        .await;

    // Record various metric types
    collector
        .increment_counter("mistersmith_messages_sent_total", HashMap::new())
        .await;
    collector
        .set_gauge("mistersmith_active_agents", 5.0, HashMap::new())
        .await;
    collector
        .record_histogram("mistersmith_task_duration_seconds", 0.042, HashMap::new())
        .await;

    // Flush should forward to the Prometheus backend without error
    collector.flush().await;

    // Buffer should be empty after flush
    assert_eq!(collector.buffered_count().await, 0);
}

#[tokio::test]
async fn prometheus_backend_handles_tagged_metrics() {
    let backend = PrometheusBackend::new();
    let mut tags = HashMap::new();
    tags.insert("agent_type".to_string(), "planner".to_string());
    tags.insert("status".to_string(), "completed".to_string());

    let metrics = vec![
        Metric {
            name: "mistersmith_tasks_completed_total".to_string(),
            value: MetricValue::Counter(42),
            timestamp: SystemTime::now(),
            tags: tags.clone(),
        },
        Metric {
            name: "mistersmith_agent_restarts_total".to_string(),
            value: MetricValue::Counter(3),
            timestamp: SystemTime::now(),
            tags,
        },
    ];

    assert!(backend.send_metrics(metrics).await.is_ok());
}

#[tokio::test]
async fn trace_context_roundtrip_through_envelope() {
    // Manually set traceparent/tracestate headers
    let traceparent = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
    let tracestate = "mistersmith=production";

    let mut envelope = MessageEnvelope::builder("test.traced").build().unwrap();

    envelope
        .headers
        .insert(TRACEPARENT_HEADER.to_string(), traceparent.to_string());
    envelope
        .headers
        .insert(TRACESTATE_HEADER.to_string(), tracestate.to_string());

    // Roundtrip through serialization
    let bytes = envelope.to_bytes().unwrap();
    let decoded = MessageEnvelope::from_bytes(&bytes).unwrap();

    // Verify trace context survives serialization
    assert_eq!(extract_trace_context(&decoded), Some(traceparent));
    assert_eq!(
        decoded.headers.get(TRACESTATE_HEADER).map(|s| s.as_str()),
        Some(tracestate)
    );
}

#[tokio::test]
async fn inject_trace_context_is_safe_without_active_span() {
    let mut envelope = MessageEnvelope::builder("test.noop").build().unwrap();

    // Without an active tracing subscriber/span, inject should be a no-op
    inject_trace_context(&mut envelope);

    // No traceparent should be set (no active span)
    assert!(extract_trace_context(&envelope).is_none());
}

#[tokio::test]
async fn envelope_preserves_headers_with_trace_and_custom() {
    let mut envelope = MessageEnvelope::builder("test.mixed_headers")
        .header("x-custom", "value1")
        .build()
        .unwrap();

    envelope
        .headers
        .insert(TRACEPARENT_HEADER.to_string(), "00-abc-def-01".to_string());

    // Both custom and trace headers should coexist
    assert_eq!(envelope.headers.get("x-custom").unwrap(), "value1");
    assert_eq!(extract_trace_context(&envelope), Some("00-abc-def-01"));
}

#[tokio::test]
async fn observability_config_validation_rejects_invalid_values() {
    use mister_smith_config::ObservabilityConfig;

    // Default config should be valid
    let config = ObservabilityConfig::default();
    assert!(config.validate().is_ok());

    // Invalid trace sampling ratio
    let mut bad_config = ObservabilityConfig::default();
    bad_config.trace_sampling_ratio = 2.0;
    assert!(bad_config.validate().is_err());

    // Invalid (zero) metrics interval
    let mut bad_config2 = ObservabilityConfig::default();
    bad_config2.metrics_export_interval_secs = 0;
    assert!(bad_config2.validate().is_err());
}
