//! Configuration file loading and environment overlay.
//!
//! Implements the 6-step config pipeline: defaults → file → env overlay →
//! build → validate → return.

use crate::error::ConfigValidationError;
use crate::types::FrameworkConfig;
use std::path::{Path, PathBuf};

/// Load configuration from a TOML file and validate it.
pub fn load_from_file(path: &Path) -> Result<FrameworkConfig, ConfigValidationError> {
    let content = std::fs::read_to_string(path)?;
    let config: FrameworkConfig = toml::from_str(&content)
        .map_err(|e| ConfigValidationError::DeserializationError(e.to_string()))?;
    config.validate()?;
    Ok(config)
}

/// Apply environment variable overlays to a configuration.
///
/// Reads env vars with the given prefix and `__` as nested field separator.
/// Example: `MISTER_SMITH_AGENT__RUNTIME__WORKER_THREADS=8` sets
/// `agent.runtime.worker_threads` to `8`.
pub fn apply_env_overlay(
    config: &mut FrameworkConfig,
    prefix: &str,
) -> Result<(), ConfigValidationError> {
    // Agent runtime fields
    if let Ok(val) = std::env::var(format!("{prefix}_AGENT__RUNTIME__WORKER_THREADS")) {
        if let Ok(n) = val.parse() {
            config.agent.runtime.worker_threads = Some(n);
        }
    }
    if let Ok(val) = std::env::var(format!("{prefix}_AGENT__RUNTIME__BLOCKING_THREADS")) {
        if let Ok(n) = val.parse() {
            config.agent.runtime.blocking_threads = n;
        }
    }
    if let Ok(val) = std::env::var(format!("{prefix}_AGENT__RUNTIME__MAX_MEMORY")) {
        if let Ok(n) = val.parse() {
            config.agent.runtime.max_memory = n;
        }
    }

    // Agent supervision fields
    if let Ok(val) = std::env::var(format!("{prefix}_AGENT__SUPERVISION__MAX_RESTART_ATTEMPTS")) {
        if let Ok(n) = val.parse() {
            config.agent.supervision.max_restart_attempts = n;
        }
    }

    // Agent monitoring fields
    if let Ok(val) = std::env::var(format!("{prefix}_AGENT__MONITORING__LOG_LEVEL")) {
        config.agent.monitoring.log_level = val;
    }

    // Transport fields
    if let Ok(val) = std::env::var(format!("{prefix}_TRANSPORT__NATS_URL")) {
        config.transport.nats_url = Some(val);
    }
    if let Ok(val) = std::env::var(format!("{prefix}_TRANSPORT__HTTP_PORT")) {
        if let Ok(n) = val.parse() {
            config.transport.http_port = Some(n);
        }
    }
    if let Ok(val) = std::env::var(format!("{prefix}_TRANSPORT__GRPC_PORT")) {
        if let Ok(n) = val.parse() {
            config.transport.grpc_port = Some(n);
        }
    }

    // LLM fields
    if let Ok(val) = std::env::var(format!("{prefix}_LLM__PROVIDER_KIND")) {
        let provider_kind = serde_json::from_value(serde_json::Value::String(val.clone()))
            .map_err(|_| ConfigValidationError::InvalidValue {
                field: "llm.provider_kind".to_string(),
                reason: format!(
                    "invalid environment value '{val}'; expected one of: anthropic, openai, openai_chatgpt, claude_subscription, mock"
                ),
            })?;
        config.llm.provider_kind = provider_kind;
    }
    if let Ok(val) = std::env::var(format!("{prefix}_LLM__MODEL_ID")) {
        config.llm.model_id = val;
    }

    // Security fields
    if let Ok(val) = std::env::var(format!("{prefix}_SECURITY__ENABLED")) {
        if let Ok(b) = val.parse() {
            config.security.enabled = b;
        }
    }
    if let Ok(val) = std::env::var(format!("{prefix}_SECURITY__TLS_ENABLED")) {
        if let Ok(b) = val.parse() {
            config.security.tls.enabled = b;
        }
    }
    if let Ok(val) = std::env::var(format!("{prefix}_SECURITY__AUTH_ENABLED")) {
        if let Ok(b) = val.parse() {
            config.security.auth.enabled = b;
        }
    }
    if let Ok(val) = std::env::var(format!("{prefix}_SECURITY__AUTH__ALGORITHM")) {
        config.security.auth.algorithm = val;
    }
    if let Ok(val) = std::env::var(format!("{prefix}_SECURITY__AUTH__ACCESS_TOKEN_TTL_SECS")) {
        if let Ok(n) = val.parse() {
            config.security.auth.access_token_ttl_secs = n;
        }
    }
    if let Ok(val) = std::env::var(format!("{prefix}_SECURITY__AUTH__REFRESH_TOKEN_TTL_SECS")) {
        if let Ok(n) = val.parse() {
            config.security.auth.refresh_token_ttl_secs = n;
        }
    }
    if let Ok(val) = std::env::var(format!("{prefix}_SECURITY__AUTH__ISSUER")) {
        config.security.auth.issuer = Some(val);
    }
    if let Ok(val) = std::env::var(format!("{prefix}_SECURITY__AUTH__AUDIENCE")) {
        config.security.auth.audience = val
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .collect();
    }
    if let Ok(val) = std::env::var(format!("{prefix}_SECURITY__AUTH__HMAC_SECRET")) {
        config.security.auth.hmac_secret = Some(val);
    }

    Ok(())
}

/// Discover configuration file paths in priority order.
///
/// Returns paths in ascending priority (later entries override earlier ones):
/// 1. `/etc/mister-smith/config.toml`
/// 2. `~/.mister-smith/config.toml`
/// 3. `./mister-smith.toml`
/// 4. Environment-specific via `MS_ENVIRONMENT` (e.g., `./mister-smith.production.toml`)
pub fn discover_config_paths() -> Vec<PathBuf> {
    let mut paths = vec![PathBuf::from("/etc/mister-smith/config.toml")];

    if let Some(home) = home_dir() {
        paths.push(home.join(".mister-smith/config.toml"));
    }

    paths.push(PathBuf::from("./mister-smith.toml"));

    if let Ok(env) = std::env::var("MS_ENVIRONMENT") {
        paths.push(PathBuf::from(format!("./mister-smith.{env}.toml")));
    }

    paths
}

fn load_config_from_paths(paths: &[PathBuf]) -> Result<FrameworkConfig, ConfigValidationError> {
    let mut config = FrameworkConfig::default();

    // Load from first existing config file
    for path in paths {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            config = toml::from_str(&content)
                .map_err(|e| ConfigValidationError::DeserializationError(e.to_string()))?;
            break;
        }
    }

    // Apply environment variable overlays
    apply_env_overlay(&mut config, "MISTER_SMITH")?;

    // Validate final configuration
    config.validate()?;

    Ok(config)
}

/// Load configuration using the full pipeline:
/// discover paths → load first existing file → apply env overlay → validate.
pub fn load_config() -> Result<FrameworkConfig, ConfigValidationError> {
    load_config_from_paths(&discover_config_paths())
}

/// Get the user's home directory.
fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
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
    fn load_config_from_paths_returns_defaults_when_no_files_exist() {
        let _env = EnvGuard::new(&[
            ("MISTER_SMITH_AGENT__RUNTIME__WORKER_THREADS", None),
            ("MISTER_SMITH_AGENT__RUNTIME__BLOCKING_THREADS", None),
            ("MISTER_SMITH_AGENT__RUNTIME__MAX_MEMORY", None),
            (
                "MISTER_SMITH_AGENT__SUPERVISION__MAX_RESTART_ATTEMPTS",
                None,
            ),
            ("MISTER_SMITH_AGENT__MONITORING__LOG_LEVEL", None),
            ("MISTER_SMITH_TRANSPORT__NATS_URL", None),
            ("MISTER_SMITH_TRANSPORT__HTTP_PORT", None),
            ("MISTER_SMITH_TRANSPORT__GRPC_PORT", None),
            ("MISTER_SMITH_LLM__PROVIDER_KIND", None),
            ("MISTER_SMITH_LLM__MODEL_ID", None),
            ("MISTER_SMITH_SECURITY__ENABLED", None),
            ("MISTER_SMITH_SECURITY__TLS_ENABLED", None),
            ("MISTER_SMITH_SECURITY__AUTH_ENABLED", None),
        ]);
        let temp_dir = tempfile::tempdir().unwrap();
        let paths = vec![
            temp_dir.path().join("missing-system.toml"),
            temp_dir.path().join("missing-home.toml"),
            temp_dir.path().join("missing-local.toml"),
        ];

        let config = load_config_from_paths(&paths).unwrap();

        assert_eq!(config.agent.runtime.blocking_threads, 512);
        assert_eq!(
            config.llm.provider_kind,
            mister_smith_llm::ProviderKind::OpenAiChatGpt
        );
        assert_eq!(config.llm.model_id, "gpt-5.4");
    }

    #[test]
    fn apply_env_overlay_updates_llm_selection() {
        let _env = EnvGuard::new(&[
            ("MISTER_SMITH_LLM__PROVIDER_KIND", Some("mock")),
            ("MISTER_SMITH_LLM__MODEL_ID", Some("mock-ops")),
        ]);
        let temp_dir = tempfile::tempdir().unwrap();
        let paths = vec![temp_dir.path().join("missing-local.toml")];

        let config = load_config_from_paths(&paths).unwrap();

        assert_eq!(
            config.llm.provider_kind,
            mister_smith_llm::ProviderKind::Mock
        );
        assert_eq!(config.llm.model_id, "mock-ops");
    }

    #[test]
    fn apply_env_overlay_rejects_invalid_llm_provider_selection() {
        let _env = EnvGuard::new(&[("MISTER_SMITH_LLM__PROVIDER_KIND", Some("mockk"))]);
        let temp_dir = tempfile::tempdir().unwrap();
        let paths = vec![temp_dir.path().join("missing-local.toml")];

        let error = load_config_from_paths(&paths).expect_err("invalid provider should fail");

        match error {
            ConfigValidationError::InvalidValue { field, reason } => {
                assert_eq!(field, "llm.provider_kind");
                assert!(reason.contains("mockk"));
            }
            other => panic!("expected invalid value error, got {other}"),
        }
    }
}
