//! Tests for config type defaults and serde roundtrip.

use mister_smith_config::*;

#[test]
fn default_framework_config_is_valid() {
    let config = FrameworkConfig::default();
    config.validate().expect("default config should be valid");
}

#[test]
fn default_runtime_config_values() {
    let config = RuntimeConfig::default();
    assert_eq!(config.worker_threads, None);
    assert_eq!(config.blocking_threads, 512);
    assert_eq!(config.max_memory, 0);
    assert!(config.enable_all);
    assert!(config.enable_time);
    assert!(config.enable_io);
}

#[test]
fn default_supervision_config_values() {
    let config = SupervisionConfig::default();
    assert_eq!(config.max_restart_attempts, 3);
    assert_eq!(config.restart_window, std::time::Duration::from_secs(60));
    assert_eq!(
        config.escalation_timeout,
        std::time::Duration::from_secs(30)
    );
}

#[test]
fn default_monitoring_config_values() {
    let config = MonitoringConfig::default();
    assert_eq!(
        config.health_check_interval,
        std::time::Duration::from_secs(30)
    );
    assert_eq!(
        config.metrics_export_interval,
        std::time::Duration::from_secs(60)
    );
    assert_eq!(config.log_level, "info");
}

#[test]
fn serde_roundtrip_full_config() {
    let config = FrameworkConfig::default();
    let toml_str = toml::to_string(&config).expect("serialize");
    let deserialized: FrameworkConfig = toml::from_str(&toml_str).expect("deserialize");
    assert_eq!(deserialized.agent.runtime.blocking_threads, 512);
    assert_eq!(deserialized.agent.monitoring.log_level, "info");
}

#[test]
fn partial_toml_uses_defaults() {
    let toml_str = r#"
[agent.runtime]
blocking_threads = 256
"#;
    let config: FrameworkConfig = toml::from_str(toml_str).expect("deserialize partial");
    assert_eq!(config.agent.runtime.blocking_threads, 256);
    assert_eq!(config.agent.runtime.worker_threads, None); // default
    assert_eq!(config.agent.monitoring.log_level, "info"); // default
    assert!(!config.security.enabled); // default
}

#[test]
fn empty_toml_uses_all_defaults() {
    let config: FrameworkConfig = toml::from_str("").expect("deserialize empty");
    config
        .validate()
        .expect("empty config should be valid with defaults");
}
