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
pub fn apply_env_overlay(config: &mut FrameworkConfig, prefix: &str) {
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

    // Security fields
    if let Ok(val) = std::env::var(format!("{prefix}_SECURITY__ENABLED")) {
        if let Ok(b) = val.parse() {
            config.security.enabled = b;
        }
    }
    if let Ok(val) = std::env::var(format!("{prefix}_SECURITY__TLS_ENABLED")) {
        if let Ok(b) = val.parse() {
            config.security.tls_enabled = b;
        }
    }
    if let Ok(val) = std::env::var(format!("{prefix}_SECURITY__AUTH_REQUIRED")) {
        if let Ok(b) = val.parse() {
            config.security.auth_required = b;
        }
    }
}

/// Discover configuration file paths in priority order.
///
/// Returns paths in ascending priority (later entries override earlier ones):
/// 1. `/etc/mister-smith/config.toml`
/// 2. `~/.mister-smith/config.toml`
/// 3. `./mister-smith.toml`
/// 4. Environment-specific via `MS_ENVIRONMENT` (e.g., `./mister-smith.production.toml`)
pub fn discover_config_paths() -> Vec<PathBuf> {
    let mut paths = vec![
        PathBuf::from("/etc/mister-smith/config.toml"),
    ];

    if let Some(home) = home_dir() {
        paths.push(home.join(".mister-smith/config.toml"));
    }

    paths.push(PathBuf::from("./mister-smith.toml"));

    if let Ok(env) = std::env::var("MS_ENVIRONMENT") {
        paths.push(PathBuf::from(format!("./mister-smith.{env}.toml")));
    }

    paths
}

/// Load configuration using the full pipeline:
/// discover paths → load first existing file → apply env overlay → validate.
pub fn load_config() -> Result<FrameworkConfig, ConfigValidationError> {
    let paths = discover_config_paths();
    let mut config = FrameworkConfig::default();

    // Load from first existing config file
    for path in &paths {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            config = toml::from_str(&content)
                .map_err(|e| ConfigValidationError::DeserializationError(e.to_string()))?;
            break;
        }
    }

    // Apply environment variable overlays
    apply_env_overlay(&mut config, "MISTER_SMITH");

    // Validate final configuration
    config.validate()?;

    Ok(config)
}

/// Get the user's home directory.
fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(PathBuf::from)
}
