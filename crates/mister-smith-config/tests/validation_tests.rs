//! Tests for config validation constraints.

use mister_smith_config::*;
use std::time::Duration;

fn invalid_observability_configs() -> Vec<(&'static str, ObservabilityConfig)> {
    let mut invalid_trace_sampling_ratio = ObservabilityConfig::default();
    invalid_trace_sampling_ratio.trace_sampling_ratio = 1.5;

    let mut invalid_metrics_export_interval_secs = ObservabilityConfig::default();
    invalid_metrics_export_interval_secs.metrics_export_interval_secs = 4;

    let mut invalid_buffer_size = ObservabilityConfig::default();
    invalid_buffer_size.buffer_size = 1000;

    vec![
        ("trace_sampling_ratio", invalid_trace_sampling_ratio),
        (
            "metrics_export_interval_secs",
            invalid_metrics_export_interval_secs,
        ),
        ("buffer_size", invalid_buffer_size),
    ]
}

#[test]
fn valid_config_passes() {
    let config = FrameworkConfig::default();
    assert!(config.validate().is_ok());
}

#[test]
fn worker_threads_zero_rejected() {
    let mut config = RuntimeConfig::default();
    config.worker_threads = Some(0);
    let err = config.validate().unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("worker_threads"),
        "error should mention field: {msg}"
    );
}

#[test]
fn worker_threads_over_1024_rejected() {
    let mut config = RuntimeConfig::default();
    config.worker_threads = Some(2000);
    assert!(config.validate().is_err());
}

#[test]
fn worker_threads_none_passes() {
    let mut config = RuntimeConfig::default();
    config.worker_threads = None;
    assert!(config.validate().is_ok());
}

#[test]
fn blocking_threads_zero_rejected() {
    let mut config = RuntimeConfig::default();
    config.blocking_threads = 0;
    assert!(config.validate().is_err());
}

#[test]
fn blocking_threads_over_512_rejected() {
    let mut config = RuntimeConfig::default();
    config.blocking_threads = 1000;
    assert!(config.validate().is_err());
}

#[test]
fn max_restart_attempts_over_100_rejected() {
    let mut config = SupervisionConfig::default();
    config.max_restart_attempts = 200;
    assert!(config.validate().is_err());
}

#[test]
fn restart_window_too_small_rejected() {
    let mut config = SupervisionConfig::default();
    config.restart_window = Duration::from_millis(500);
    assert!(config.validate().is_err());
}

#[test]
fn restart_window_too_large_rejected() {
    let mut config = SupervisionConfig::default();
    config.restart_window = Duration::from_secs(7200);
    assert!(config.validate().is_err());
}

#[test]
fn escalation_timeout_too_small_rejected() {
    let mut config = SupervisionConfig::default();
    config.escalation_timeout = Duration::from_millis(100);
    assert!(config.validate().is_err());
}

#[test]
fn escalation_timeout_too_large_rejected() {
    let mut config = SupervisionConfig::default();
    config.escalation_timeout = Duration::from_secs(600);
    assert!(config.validate().is_err());
}

#[test]
fn health_check_interval_too_small_rejected() {
    let mut config = MonitoringConfig::default();
    config.health_check_interval = Duration::from_millis(100);
    assert!(config.validate().is_err());
}

#[test]
fn health_check_interval_too_large_rejected() {
    let mut config = MonitoringConfig::default();
    config.health_check_interval = Duration::from_secs(600);
    assert!(config.validate().is_err());
}

#[test]
fn metrics_interval_too_large_rejected() {
    let mut config = MonitoringConfig::default();
    config.metrics_export_interval = Duration::from_secs(1200);
    assert!(config.validate().is_err());
}

#[test]
fn invalid_log_level_rejected() {
    let mut config = MonitoringConfig::default();
    config.log_level = "verbose".to_string();
    let err = config.validate().unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("log_level"),
        "error should mention field: {msg}"
    );
}

#[test]
fn valid_log_levels_accepted() {
    for level in &["trace", "debug", "info", "warn", "error"] {
        let mut config = MonitoringConfig::default();
        config.log_level = level.to_string();
        assert!(config.validate().is_ok(), "level '{level}' should be valid");
    }
}

#[test]
fn framework_config_validates_nested() {
    let mut config = FrameworkConfig::default();
    config.agent.runtime.blocking_threads = 0; // invalid
    assert!(config.validate().is_err());
}

#[test]
fn framework_config_rejects_empty_llm_model_id() {
    let mut config = FrameworkConfig::default();
    config.llm.model_id = "   ".to_string();

    let err = config
        .validate()
        .expect_err("empty llm model id should fail");

    match err {
        ConfigValidationError::InvalidValue { field, reason } => {
            assert_eq!(field, "llm.model_id");
            assert_eq!(reason, "must not be empty");
        }
        other => panic!("expected invalid value error, got {other}"),
    }
}

#[test]
fn error_messages_are_actionable() {
    let mut config = RuntimeConfig::default();
    config.worker_threads = Some(0);
    let err = config.validate().unwrap_err();
    let msg = err.to_string();
    // Should contain field name and valid range
    assert!(msg.contains("worker_threads"));
    assert!(msg.contains("1..=1024"));
}

#[test]
fn observability_boundary_values_accepted() {
    let mut min_config = ObservabilityConfig::default();
    min_config.trace_sampling_ratio = 0.0;
    min_config.metrics_export_interval_secs = 5;
    min_config.buffer_size = 1024;
    assert!(min_config.validate().is_ok());

    let mut max_config = ObservabilityConfig::default();
    max_config.trace_sampling_ratio = 1.0;
    max_config.buffer_size = 65536;
    assert!(max_config.validate().is_ok());
}

#[test]
fn observability_invalid_values_rejected() {
    for (field, config) in invalid_observability_configs() {
        let err = config.validate().unwrap_err();
        match err {
            ConfigValidationError::InvalidValue {
                field: actual_field,
                ..
            } => assert_eq!(actual_field, field),
            other => panic!("expected invalid value error for {field}, got {other}"),
        }
    }
}

#[test]
fn framework_config_rejects_invalid_observability() {
    for (field, observability) in invalid_observability_configs() {
        let mut config = FrameworkConfig::default();
        config.observability = observability;

        let err = config
            .validate()
            .expect_err(&format!("expected invalid {field} to fail"));

        match err {
            ConfigValidationError::InvalidValue {
                field: actual_field,
                ..
            } => assert_eq!(actual_field, field),
            other => panic!("expected invalid value error for {field}, got {other}"),
        }
    }
}
