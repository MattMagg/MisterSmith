//! Application configuration loading pipeline.
//!
//! Merges CLI arguments, config file, and environment variable overlays
//! into a validated `FrameworkConfig`.

use mister_smith_config::{
    apply_env_overlay, load_config, load_from_file, FrameworkConfig, LogFormat,
};
use std::path::Path;
use tracing::info;

/// CLI-overridable fields that take precedence over config file and env vars.
pub struct CliOverrides {
    pub config_path: Option<String>,
    pub log_level: Option<String>,
    pub log_format: Option<LogFormat>,
}

/// Load and validate the framework configuration.
///
/// Priority (highest to lowest):
/// 1. CLI overrides
/// 2. Environment variables (MISTER_SMITH_ prefix)
/// 3. Config file
/// 4. Default values
pub fn load_framework_config(
    overrides: &CliOverrides,
) -> Result<FrameworkConfig, Box<dyn std::error::Error>> {
    let mut config = if let Some(ref path) = overrides.config_path {
        info!(path = %path, "Loading configuration from specified file");
        load_from_file(Path::new(path))?
    } else {
        info!("Loading configuration from auto-discovered paths");
        load_config()?
    };

    // Apply environment variable overlay (MISTER_SMITH_ prefix)
    apply_env_overlay(&mut config, "MISTER_SMITH")?;

    // Apply CLI overrides (highest priority)
    if let Some(ref level) = overrides.log_level {
        config.observability.log_level = level.clone();
    }
    if let Some(format) = overrides.log_format {
        config.observability.log_format = format;
    }

    // Validate
    config.observability.validate()?;

    Ok(config)
}
