//! Configuration type definitions.
//!
//! Defines typed configuration structs for all framework domains.
//! All structs implement `Default` with sensible production defaults
//! and use `#[serde(default)]` for partial TOML deserialization.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use mister_smith_llm::ProviderKind;

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

/// Runtime LLM provider/model selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Provider kind used by the runtime-backed task path.
    #[serde(default = "default_llm_provider_kind")]
    pub provider_kind: ProviderKind,
    /// Model identifier to pass to the selected provider.
    #[serde(default = "default_llm_model_id")]
    pub model_id: String,
    /// Optional multi-provider routing profile for the runtime-backed task path.
    #[serde(default)]
    pub runtime_routing_profile: Option<RuntimeRoutingProfile>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider_kind: default_llm_provider_kind(),
            model_id: default_llm_model_id(),
            runtime_routing_profile: None,
        }
    }
}

/// Bounded routing policies supported by the runtime profile config.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeRoutingPolicy {
    /// Multi-tier cascade routing with bounded fallback.
    #[default]
    Cascade,
}

/// One provider tier declared inside the runtime routing profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeProviderTier {
    /// Operator-visible tier label surfaced on routing evidence.
    pub label: String,
    /// Shipped provider kind to register for this tier.
    pub provider_kind: ProviderKind,
    /// Model identifier for this tier.
    pub model_id: String,
    /// Optional provider-tier metadata used by later runtime wiring.
    #[serde(default = "default_runtime_tier_metadata")]
    pub metadata: serde_json::Value,
}

/// Typed runtime routing profile for bounded multi-provider boot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeRoutingProfile {
    /// Routing policy for the runtime-backed task path.
    #[serde(default)]
    pub policy: RuntimeRoutingPolicy,
    /// Canonical budget root used by the runtime task path.
    #[serde(default = "default_runtime_budget_root")]
    pub budget_root: String,
    /// Ordered provider tiers registered into the runtime router.
    #[serde(default)]
    pub tiers: Vec<RuntimeProviderTier>,
}

impl Default for RuntimeRoutingProfile {
    fn default() -> Self {
        Self {
            policy: RuntimeRoutingPolicy::Cascade,
            budget_root: default_runtime_budget_root(),
            tiers: Vec::new(),
        }
    }
}

/// Security configuration with independent subsystem toggles.
///
/// The top-level `enabled` field acts as a master switch — when `false`, all
/// subsystems are disabled regardless of their individual flags.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Master switch — disables all security subsystems when `false`.
    #[serde(default)]
    pub enabled: bool,
    /// Authentication (JWT) configuration.
    #[serde(default)]
    pub auth: AuthConfig,
    /// Authorization (RBAC) configuration.
    #[serde(default)]
    pub authz: AuthzConfig,
    /// TLS / mTLS configuration.
    #[serde(default)]
    pub tls: TlsSecurityConfig,
    /// Audit logging configuration.
    #[serde(default)]
    pub audit: AuditSecurityConfig,
}

/// Authentication (JWT) configuration section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Whether authentication is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Signing algorithm (e.g., "RS256", "ES256", "HS256").
    #[serde(default = "default_algorithm")]
    pub algorithm: String,
    /// Access token TTL in seconds.
    #[serde(default = "default_access_token_ttl")]
    pub access_token_ttl_secs: u64,
    /// Refresh token TTL in seconds.
    #[serde(default = "default_refresh_token_ttl")]
    pub refresh_token_ttl_secs: u64,
    /// Token issuer claim.
    #[serde(default)]
    pub issuer: Option<String>,
    /// Required audience claims.
    #[serde(default)]
    pub audience: Vec<String>,
    /// Path to the private key PEM file (for RSA/EC/Ed algorithms).
    #[serde(default)]
    pub private_key_path: Option<String>,
    /// Path to the public key PEM file (for RSA/EC/Ed algorithms).
    #[serde(default)]
    pub public_key_path: Option<String>,
    /// HMAC secret (for HS* algorithms). Base64-encoded.
    #[serde(default)]
    pub hmac_secret: Option<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            algorithm: default_algorithm(),
            access_token_ttl_secs: default_access_token_ttl(),
            refresh_token_ttl_secs: default_refresh_token_ttl(),
            issuer: None,
            audience: Vec::new(),
            private_key_path: None,
            public_key_path: None,
            hmac_secret: None,
        }
    }
}

/// Authorization (RBAC) configuration section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthzConfig {
    /// Whether authorization is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Default role for unauthenticated requests (when auth is disabled).
    #[serde(default)]
    pub default_role: Option<String>,
}

/// TLS / mTLS configuration section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsSecurityConfig {
    /// Whether TLS is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Path to the server certificate PEM file.
    #[serde(default)]
    pub cert_path: Option<String>,
    /// Path to the server private key PEM file.
    #[serde(default)]
    pub key_path: Option<String>,
    /// Path to the CA certificate PEM for client verification.
    #[serde(default)]
    pub ca_path: Option<String>,
    /// Whether mutual TLS (client certificates) is required.
    #[serde(default)]
    pub mtls_enabled: bool,
    /// Auto-generate self-signed certificates for dev/test.
    #[serde(default)]
    pub generate_self_signed: bool,
    /// Certificate reload check interval in seconds.
    #[serde(default)]
    pub reload_interval_secs: Option<u64>,
    /// Days before expiry to emit warnings.
    #[serde(default = "default_expiry_warning_days")]
    pub expiry_warning_days: u32,
}

impl Default for TlsSecurityConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_path: None,
            key_path: None,
            ca_path: None,
            mtls_enabled: false,
            generate_self_signed: false,
            reload_interval_secs: None,
            expiry_warning_days: default_expiry_warning_days(),
        }
    }
}

/// Audit logging configuration section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSecurityConfig {
    /// Whether audit logging is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Maximum number of audit events to retain in memory.
    #[serde(default = "default_max_audit_events")]
    pub max_events: usize,
    /// Auth failure threshold per source per minute before alert.
    #[serde(default = "default_auth_failure_threshold")]
    pub auth_failure_alert_threshold: u32,
}

impl Default for AuditSecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_events: default_max_audit_events(),
            auth_failure_alert_threshold: default_auth_failure_threshold(),
        }
    }
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
    /// Persistence configuration.
    #[serde(default)]
    pub persistence: PersistenceConfig,
    /// Runtime LLM selection.
    #[serde(default)]
    pub llm: LlmConfig,
    /// Observability configuration (Phase 8).
    #[serde(default)]
    pub observability: ObservabilityConfig,
}

/// Log output format.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogFormat {
    /// JSON-structured log output (production default).
    #[default]
    Json,
    /// Human-readable pretty-printed output (development).
    Pretty,
}

/// OTLP export protocol.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OtlpProtocol {
    /// gRPC transport (default).
    #[default]
    Grpc,
    /// HTTP/protobuf transport.
    Http,
}

fn default_trace_sampling_ratio() -> f64 {
    1.0
}
fn default_metrics_export_interval_secs() -> u64 {
    60
}
fn default_buffer_size() -> usize {
    8192
}
fn default_startup_timeout_secs() -> u64 {
    30
}
fn default_shutdown_timeout_secs() -> u64 {
    30
}

/// Observability and telemetry configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// OTLP collector endpoint (e.g., "http://localhost:4317").
    /// If None, OTLP export is disabled (logs and metrics still available locally).
    #[serde(default)]
    pub otlp_endpoint: Option<String>,

    /// OTLP export protocol.
    #[serde(default)]
    pub otlp_protocol: OtlpProtocol,

    /// Trace sampling ratio (0.0 to 1.0).
    #[serde(default = "default_trace_sampling_ratio")]
    pub trace_sampling_ratio: f64,

    /// How often to push metrics via OTLP (seconds).
    #[serde(default = "default_metrics_export_interval_secs")]
    pub metrics_export_interval_secs: u64,

    /// Log output format.
    #[serde(default)]
    pub log_format: LogFormat,

    /// Tracing filter directive (e.g., "info", "mister_smith=debug").
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Local telemetry buffer size when collector is unreachable.
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,

    /// Expose Prometheus /metrics endpoint.
    #[serde(default = "default_true")]
    pub prometheus_enabled: bool,

    /// Startup timeout in seconds before the process exits with failure.
    #[serde(default = "default_startup_timeout_secs")]
    pub startup_timeout_secs: u64,

    /// Graceful shutdown timeout in seconds.
    #[serde(default = "default_shutdown_timeout_secs")]
    pub shutdown_timeout_secs: u64,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            otlp_endpoint: None,
            otlp_protocol: OtlpProtocol::default(),
            trace_sampling_ratio: default_trace_sampling_ratio(),
            metrics_export_interval_secs: default_metrics_export_interval_secs(),
            log_format: LogFormat::default(),
            log_level: default_log_level(),
            buffer_size: default_buffer_size(),
            prometheus_enabled: true,
            startup_timeout_secs: default_startup_timeout_secs(),
            shutdown_timeout_secs: default_shutdown_timeout_secs(),
        }
    }
}

impl ObservabilityConfig {
    /// Validate observability configuration values.
    pub fn validate(&self) -> Result<(), crate::error::ConfigValidationError> {
        if !(0.0..=1.0).contains(&self.trace_sampling_ratio) {
            return Err(crate::error::ConfigValidationError::InvalidValue {
                field: "trace_sampling_ratio".to_string(),
                reason: "must be between 0.0 and 1.0".to_string(),
            });
        }
        if self.metrics_export_interval_secs < 5 {
            return Err(crate::error::ConfigValidationError::InvalidValue {
                field: "metrics_export_interval_secs".to_string(),
                reason: "must be >= 5".to_string(),
            });
        }
        if !(1024..=65536).contains(&self.buffer_size) {
            return Err(crate::error::ConfigValidationError::InvalidValue {
                field: "buffer_size".to_string(),
                reason: "must be between 1024 and 65536".to_string(),
            });
        }
        Ok(())
    }
}

/// Persistence configuration (re-exported from mister-smith-persistence).
///
/// This is a minimal placeholder to avoid a circular dependency.
/// The full implementation lives in `mister-smith-persistence::config`.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    /// Master switch — when `false`, persistence is disabled.
    #[serde(default)]
    pub enabled: bool,
}

impl PersistenceConfig {
    /// Validate persistence configuration values.
    pub fn validate(&self) -> Result<(), crate::error::ConfigValidationError> {
        Ok(())
    }
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

fn default_llm_provider_kind() -> ProviderKind {
    ProviderKind::OpenAiChatGpt
}

fn default_llm_model_id() -> String {
    "gpt-5.4".to_string()
}

fn default_runtime_budget_root() -> String {
    "runtime.task_path".to_string()
}

fn default_runtime_tier_metadata() -> serde_json::Value {
    serde_json::json!({})
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

fn default_algorithm() -> String {
    "RS256".to_string()
}

fn default_access_token_ttl() -> u64 {
    900 // 15 minutes
}

fn default_refresh_token_ttl() -> u64 {
    86400 // 24 hours
}

fn default_expiry_warning_days() -> u32 {
    30
}

fn default_max_audit_events() -> usize {
    10_000
}

fn default_auth_failure_threshold() -> u32 {
    5
}
