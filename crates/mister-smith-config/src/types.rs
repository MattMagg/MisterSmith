//! Configuration type definitions.
//!
//! Defines typed configuration structs for all framework domains.
//! All structs implement `Default` with sensible production defaults
//! and use `#[serde(default)]` for partial TOML deserialization.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Runtime configuration.
///
/// This is the SINGLE canonical `RuntimeConfig` definition. Phase 2's
/// `mister-smith-runtime` crate adds behavior (preset constructors,
/// `build_runtime()`) via extension methods — no duplicate struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Number of Tokio worker threads. `None` uses Tokio's default.
    #[serde(default)]
    pub worker_threads: Option<usize>,
    /// Maximum number of blocking threads.
    #[serde(default = "default_blocking_threads")]
    pub blocking_threads: usize,
    /// Maximum memory in bytes (0 = unlimited).
    #[serde(default)]
    pub max_memory: usize,
    /// Thread stack size override.
    #[serde(default)]
    pub thread_stack_size: Option<usize>,
    /// Keep-alive duration for idle threads.
    #[serde(default = "default_thread_keep_alive")]
    pub thread_keep_alive: Duration,
    /// Enable all Tokio features.
    #[serde(default = "default_true")]
    pub enable_all: bool,
    /// Enable Tokio time driver.
    #[serde(default = "default_true")]
    pub enable_time: bool,
    /// Enable Tokio I/O driver.
    #[serde(default = "default_true")]
    pub enable_io: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: None,
            blocking_threads: default_blocking_threads(),
            max_memory: 0,
            thread_stack_size: None,
            thread_keep_alive: default_thread_keep_alive(),
            enable_all: true,
            enable_time: true,
            enable_io: true,
        }
    }
}

/// Supervision configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionConfig {
    /// Maximum restart attempts before escalation.
    #[serde(default = "default_max_restart_attempts")]
    pub max_restart_attempts: u32,
    /// Time window for counting restart attempts.
    #[serde(default = "default_restart_window")]
    pub restart_window: Duration,
    /// Timeout before escalating to parent supervisor.
    #[serde(default = "default_escalation_timeout")]
    pub escalation_timeout: Duration,
}

impl Default for SupervisionConfig {
    fn default() -> Self {
        Self {
            max_restart_attempts: default_max_restart_attempts(),
            restart_window: default_restart_window(),
            escalation_timeout: default_escalation_timeout(),
        }
    }
}

/// Monitoring configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Interval between health checks.
    #[serde(default = "default_health_check_interval")]
    pub health_check_interval: Duration,
    /// Interval between metrics exports.
    #[serde(default = "default_metrics_export_interval")]
    pub metrics_export_interval: Duration,
    /// Log level (trace, debug, info, warn, error).
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            health_check_interval: default_health_check_interval(),
            metrics_export_interval: default_metrics_export_interval(),
            log_level: default_log_level(),
        }
    }
}

/// Agent configuration combining runtime, supervision, and monitoring.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Runtime settings.
    #[serde(default)]
    pub runtime: RuntimeConfig,
    /// Supervision settings.
    #[serde(default)]
    pub supervision: SupervisionConfig,
    /// Monitoring settings.
    #[serde(default)]
    pub monitoring: MonitoringConfig,
}

/// Transport configuration (minimal placeholder — full definition in Phase 4).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// NATS server URL.
    #[serde(default)]
    pub nats_url: Option<String>,
    /// HTTP server port.
    #[serde(default)]
    pub http_port: Option<u16>,
    /// gRPC server port.
    #[serde(default)]
    pub grpc_port: Option<u16>,
}

/// Security configuration (minimal placeholder — full definition in Phase 5).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Whether security features are enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Whether TLS is enabled.
    #[serde(default)]
    pub tls_enabled: bool,
    /// Whether authentication is required.
    #[serde(default)]
    pub auth_required: bool,
}

/// Top-level framework configuration.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FrameworkConfig {
    /// Agent configuration.
    #[serde(default)]
    pub agent: AgentConfig,
    /// Transport configuration.
    #[serde(default)]
    pub transport: TransportConfig,
    /// Security configuration.
    #[serde(default)]
    pub security: SecurityConfig,
}

// ---------------------------------------------------------------------------
// Default value functions
// ---------------------------------------------------------------------------

fn default_blocking_threads() -> usize {
    512
}

fn default_thread_keep_alive() -> Duration {
    Duration::from_secs(60)
}

fn default_true() -> bool {
    true
}

fn default_max_restart_attempts() -> u32 {
    3
}

fn default_restart_window() -> Duration {
    Duration::from_secs(60)
}

fn default_escalation_timeout() -> Duration {
    Duration::from_secs(30)
}

fn default_health_check_interval() -> Duration {
    Duration::from_secs(30)
}

fn default_metrics_export_interval() -> Duration {
    Duration::from_secs(60)
}

fn default_log_level() -> String {
    "info".to_string()
}
