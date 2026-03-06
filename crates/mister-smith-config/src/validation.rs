//! Configuration validation logic.
//!
//! Each config struct has a `validate()` method that checks value constraints
//! and returns actionable error messages on failure.

use crate::error::ConfigValidationError;
use crate::types::{
    AgentConfig, FrameworkConfig, MonitoringConfig, RuntimeConfig, SecurityConfig,
    SupervisionConfig, TransportConfig,
};
use std::time::Duration;

impl RuntimeConfig {
    /// Validate runtime configuration values.
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        if let Some(threads) = self.worker_threads {
            if threads == 0 || threads > 1024 {
                return Err(ConfigValidationError::InvalidValue {
                    field: "worker_threads".to_string(),
                    reason: format!("must be 1..=1024, got {threads}"),
                });
            }
        }
        if self.blocking_threads == 0 || self.blocking_threads > 512 {
            let blocking_threads = self.blocking_threads;
            return Err(ConfigValidationError::InvalidValue {
                field: "blocking_threads".to_string(),
                reason: format!("must be 1..=512, got {blocking_threads}"),
            });
        }
        Ok(())
    }
}

impl SupervisionConfig {
    /// Validate supervision configuration values.
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        if self.max_restart_attempts > 100 {
            let max_restart_attempts = self.max_restart_attempts;
            return Err(ConfigValidationError::InvalidValue {
                field: "max_restart_attempts".to_string(),
                reason: format!("must be 0..=100, got {max_restart_attempts}"),
            });
        }
        if self.restart_window < Duration::from_secs(1)
            || self.restart_window > Duration::from_secs(3600)
        {
            return Err(ConfigValidationError::InvalidValue {
                field: "restart_window".to_string(),
                reason: format!("must be 1s..=3600s, got {:?}", self.restart_window),
            });
        }
        if self.escalation_timeout < Duration::from_secs(1)
            || self.escalation_timeout > Duration::from_secs(300)
        {
            return Err(ConfigValidationError::InvalidValue {
                field: "escalation_timeout".to_string(),
                reason: format!("must be 1s..=300s, got {:?}", self.escalation_timeout),
            });
        }
        Ok(())
    }
}

impl MonitoringConfig {
    /// Validate monitoring configuration values.
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        if self.health_check_interval < Duration::from_secs(1)
            || self.health_check_interval > Duration::from_secs(300)
        {
            return Err(ConfigValidationError::InvalidValue {
                field: "health_check_interval".to_string(),
                reason: format!("must be 1s..=300s, got {:?}", self.health_check_interval),
            });
        }
        if self.metrics_export_interval < Duration::from_secs(1)
            || self.metrics_export_interval > Duration::from_secs(600)
        {
            return Err(ConfigValidationError::InvalidValue {
                field: "metrics_export_interval".to_string(),
                reason: format!("must be 1s..=600s, got {:?}", self.metrics_export_interval),
            });
        }
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.log_level.as_str()) {
            return Err(ConfigValidationError::InvalidValue {
                field: "log_level".to_string(),
                reason: format!(
                    "must be one of trace/debug/info/warn/error, got '{}'",
                    self.log_level
                ),
            });
        }
        Ok(())
    }
}

impl AgentConfig {
    /// Validate all nested agent configuration.
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        self.runtime.validate()?;
        self.supervision.validate()?;
        self.monitoring.validate()?;
        Ok(())
    }
}

impl TransportConfig {
    /// Validate transport configuration (placeholder — expanded in Phase 4).
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        Ok(())
    }
}

impl SecurityConfig {
    /// Validate security configuration (placeholder — expanded in Phase 5).
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        Ok(())
    }
}

impl FrameworkConfig {
    /// Validate the entire framework configuration.
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        self.agent.validate()?;
        self.transport.validate()?;
        self.security.validate()?;
        self.persistence.validate()?;
        Ok(())
    }
}
