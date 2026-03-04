//! Tests for config loading, environment overlay, and file discovery.

use mister_smith_config::*;
use std::io::Write;

#[test]
fn load_from_toml_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let mut file = std::fs::File::create(&path).unwrap();
    write!(
        file,
        r#"
[agent.runtime]
blocking_threads = 256

[agent.monitoring]
log_level = "debug"

[transport]
nats_url = "nats://localhost:4222"
"#
    )
    .unwrap();

    let config = load_from_file(&path).unwrap();
    assert_eq!(config.agent.runtime.blocking_threads, 256);
    assert_eq!(config.agent.monitoring.log_level, "debug");
    assert_eq!(
        config.transport.nats_url,
        Some("nats://localhost:4222".to_string())
    );
}

#[test]
fn load_from_file_validates() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.toml");
    let mut file = std::fs::File::create(&path).unwrap();
    write!(
        file,
        r#"
[agent.runtime]
blocking_threads = 0
"#
    )
    .unwrap();

    let result = load_from_file(&path);
    assert!(result.is_err(), "invalid config should fail validation");
}

#[test]
fn load_from_missing_file_errors() {
    let result = load_from_file(std::path::Path::new("/nonexistent/config.toml"));
    assert!(result.is_err());
}

#[test]
fn env_overlay_worker_threads() {
    let mut config = FrameworkConfig::default();
    std::env::set_var("TEST_PREFIX_AGENT__RUNTIME__WORKER_THREADS", "16");
    apply_env_overlay(&mut config, "TEST_PREFIX");
    std::env::remove_var("TEST_PREFIX_AGENT__RUNTIME__WORKER_THREADS");

    assert_eq!(config.agent.runtime.worker_threads, Some(16));
}

#[test]
fn env_overlay_log_level() {
    let mut config = FrameworkConfig::default();
    std::env::set_var("TEST_PREFIX2_AGENT__MONITORING__LOG_LEVEL", "debug");
    apply_env_overlay(&mut config, "TEST_PREFIX2");
    std::env::remove_var("TEST_PREFIX2_AGENT__MONITORING__LOG_LEVEL");

    assert_eq!(config.agent.monitoring.log_level, "debug");
}

#[test]
fn env_overlay_transport() {
    let mut config = FrameworkConfig::default();
    std::env::set_var("TEST_PREFIX3_TRANSPORT__NATS_URL", "nats://remote:4222");
    std::env::set_var("TEST_PREFIX3_TRANSPORT__HTTP_PORT", "8080");
    apply_env_overlay(&mut config, "TEST_PREFIX3");
    std::env::remove_var("TEST_PREFIX3_TRANSPORT__NATS_URL");
    std::env::remove_var("TEST_PREFIX3_TRANSPORT__HTTP_PORT");

    assert_eq!(
        config.transport.nats_url,
        Some("nats://remote:4222".to_string())
    );
    assert_eq!(config.transport.http_port, Some(8080));
}

#[test]
fn env_overlay_security() {
    let mut config = FrameworkConfig::default();
    std::env::set_var("TEST_PREFIX4_SECURITY__ENABLED", "true");
    apply_env_overlay(&mut config, "TEST_PREFIX4");
    std::env::remove_var("TEST_PREFIX4_SECURITY__ENABLED");

    assert!(config.security.enabled);
}

#[test]
fn discover_config_paths_includes_local() {
    let paths = discover_config_paths();
    assert!(paths.iter().any(|p| p.ends_with("mister-smith.toml")));
}

#[test]
fn load_config_returns_defaults_when_no_file() {
    // In test environment, none of the standard config paths should exist
    let config = load_config().unwrap();
    assert_eq!(config.agent.runtime.blocking_threads, 512);
}
