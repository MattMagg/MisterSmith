//! Tests for config loading, environment overlay, and file discovery.

use mister_smith_config::*;
use mister_smith_llm::ProviderKind;
use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvGuard {
    _lock: MutexGuard<'static, ()>,
    previous: Vec<(String, Option<OsString>)>,
}

impl EnvGuard {
    fn new(entries: &[(&str, Option<&str>)]) -> Self {
        let lock = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = entries
            .iter()
            .map(|(name, value)| {
                let previous = ((*name).to_string(), std::env::var_os(name));
                match value {
                    Some(value) => std::env::set_var(name, value),
                    None => std::env::remove_var(name),
                }
                previous
            })
            .collect();

        Self {
            _lock: lock,
            previous,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (name, previous) in self.previous.iter().rev() {
            match previous {
                Some(value) => std::env::set_var(name, value),
                None => std::env::remove_var(name),
            }
        }
    }
}

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
fn load_from_toml_file_parses_runtime_routing_profile() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let mut file = std::fs::File::create(&path).unwrap();
    write!(
        file,
        r#"
[llm]
provider_kind = "openai_chatgpt"
model_id = "gpt-5.4"

[llm.runtime_routing_profile]
policy = "cascade"
budget_root = "runtime.task_path"

[[llm.runtime_routing_profile.tiers]]
label = "primary"
provider_kind = "openai_chatgpt"
model_id = "gpt-5.4"
metadata = {{ preferred_tier = "primary" }}

[[llm.runtime_routing_profile.tiers]]
label = "fallback"
provider_kind = "claude_subscription"
model_id = "claude-sonnet"
"#
    )
    .unwrap();

    let config = load_from_file(&path).unwrap();
    let profile = config
        .llm
        .runtime_routing_profile
        .expect("runtime routing profile should parse");

    assert_eq!(profile.policy, RuntimeRoutingPolicy::Cascade);
    assert_eq!(profile.budget_root, "runtime.task_path");
    assert_eq!(profile.tiers.len(), 2);
    assert_eq!(profile.tiers[0].provider_kind, ProviderKind::OpenAiChatGpt);
    assert_eq!(profile.tiers[1].provider_kind, ProviderKind::ClaudeSubscription);
    assert_eq!(
        profile.tiers[0].metadata["preferred_tier"],
        serde_json::json!("primary")
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
fn env_overlay_runtime() {
    let mut config = FrameworkConfig::default();
    let _env = EnvGuard::new(&[
        ("TEST_PREFIX_AGENT__RUNTIME__WORKER_THREADS", Some("16")),
        ("TEST_PREFIX_AGENT__RUNTIME__BLOCKING_THREADS", Some("128")),
        ("TEST_PREFIX_AGENT__RUNTIME__MAX_MEMORY", Some("2048")),
    ]);

    apply_env_overlay(&mut config, "TEST_PREFIX").unwrap();

    assert_eq!(config.agent.runtime.worker_threads, Some(16));
    assert_eq!(config.agent.runtime.blocking_threads, 128);
    assert_eq!(config.agent.runtime.max_memory, 2048);
}

#[test]
fn env_overlay_supervision() {
    let mut config = FrameworkConfig::default();
    let _env = EnvGuard::new(&[(
        "TEST_PREFIX2_AGENT__SUPERVISION__MAX_RESTART_ATTEMPTS",
        Some("9"),
    )]);

    apply_env_overlay(&mut config, "TEST_PREFIX2").unwrap();

    assert_eq!(config.agent.supervision.max_restart_attempts, 9);
}

#[test]
fn env_overlay_log_level() {
    let mut config = FrameworkConfig::default();
    let _env = EnvGuard::new(&[("TEST_PREFIX3_AGENT__MONITORING__LOG_LEVEL", Some("debug"))]);
    apply_env_overlay(&mut config, "TEST_PREFIX3").unwrap();

    assert_eq!(config.agent.monitoring.log_level, "debug");
}

#[test]
fn env_overlay_transport() {
    let mut config = FrameworkConfig::default();
    let _env = EnvGuard::new(&[
        (
            "TEST_PREFIX4_TRANSPORT__NATS_URL",
            Some("nats://remote:4222"),
        ),
        ("TEST_PREFIX4_TRANSPORT__HTTP_PORT", Some("8080")),
        ("TEST_PREFIX4_TRANSPORT__GRPC_PORT", Some("9090")),
    ]);
    apply_env_overlay(&mut config, "TEST_PREFIX4").unwrap();

    assert_eq!(
        config.transport.nats_url,
        Some("nats://remote:4222".to_string())
    );
    assert_eq!(config.transport.http_port, Some(8080));
    assert_eq!(config.transport.grpc_port, Some(9090));
}

#[test]
fn env_overlay_security() {
    let mut config = FrameworkConfig::default();
    let _env = EnvGuard::new(&[
        ("TEST_PREFIX5_SECURITY__ENABLED", Some("true")),
        ("TEST_PREFIX5_SECURITY__TLS_ENABLED", Some("true")),
        ("TEST_PREFIX5_SECURITY__AUTH_ENABLED", Some("true")),
    ]);
    apply_env_overlay(&mut config, "TEST_PREFIX5").unwrap();

    assert!(config.security.enabled);
    assert!(config.security.tls.enabled);
    assert!(config.security.auth.enabled);
}

#[test]
fn env_overlay_security_auth_fields() {
    let mut config = FrameworkConfig::default();
    let _env = EnvGuard::new(&[
        ("TEST_PREFIX6_SECURITY__AUTH__ALGORITHM", Some("HS512")),
        (
            "TEST_PREFIX6_SECURITY__AUTH__ACCESS_TOKEN_TTL_SECS",
            Some("120"),
        ),
        (
            "TEST_PREFIX6_SECURITY__AUTH__REFRESH_TOKEN_TTL_SECS",
            Some("7200"),
        ),
        (
            "TEST_PREFIX6_SECURITY__AUTH__ISSUER",
            Some("mister-smith-tests"),
        ),
        ("TEST_PREFIX6_SECURITY__AUTH__AUDIENCE", Some("ops,proof")),
        (
            "TEST_PREFIX6_SECURITY__AUTH__HMAC_SECRET",
            Some("shared-secret"),
        ),
    ]);

    apply_env_overlay(&mut config, "TEST_PREFIX6").unwrap();

    assert_eq!(config.security.auth.algorithm, "HS512");
    assert_eq!(config.security.auth.access_token_ttl_secs, 120);
    assert_eq!(config.security.auth.refresh_token_ttl_secs, 7200);
    assert_eq!(
        config.security.auth.issuer.as_deref(),
        Some("mister-smith-tests")
    );
    assert_eq!(
        config.security.auth.audience,
        vec!["ops".to_string(), "proof".to_string()]
    );
    assert_eq!(
        config.security.auth.hmac_secret.as_deref(),
        Some("shared-secret")
    );
}

#[test]
fn env_overlay_invalid_llm_provider_fails() {
    let mut config = FrameworkConfig::default();
    let _env = EnvGuard::new(&[("TEST_PREFIX7_LLM__PROVIDER_KIND", Some("mockk"))]);

    let err = apply_env_overlay(&mut config, "TEST_PREFIX7")
        .expect_err("invalid provider should fail explicitly");

    match err {
        ConfigValidationError::InvalidValue { field, reason } => {
            assert_eq!(field, "llm.provider_kind");
            assert!(reason.contains("mockk"));
        }
        other => panic!("expected invalid value error, got {other}"),
    }
}

#[test]
fn discover_config_paths_without_home_or_environment() {
    let _env = EnvGuard::new(&[("HOME", None), ("MS_ENVIRONMENT", None)]);
    let paths = discover_config_paths();

    assert_eq!(
        paths,
        vec![
            PathBuf::from("/etc/mister-smith/config.toml"),
            PathBuf::from("./mister-smith.toml"),
        ]
    );
}

#[test]
fn discover_config_paths_with_home_and_environment() {
    let home = tempfile::tempdir().unwrap();
    let _env = EnvGuard::new(&[
        ("HOME", Some(home.path().to_str().unwrap())),
        ("MS_ENVIRONMENT", Some("production")),
    ]);

    let paths = discover_config_paths();

    assert_eq!(
        paths,
        vec![
            PathBuf::from("/etc/mister-smith/config.toml"),
            home.path().join(".mister-smith/config.toml"),
            PathBuf::from("./mister-smith.toml"),
            PathBuf::from("./mister-smith.production.toml"),
        ]
    );
}
