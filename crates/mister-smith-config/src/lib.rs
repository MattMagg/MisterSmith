#![deny(missing_docs, unsafe_code)]

//! Typed configuration loading, validation, and environment overlay for the Mister Smith framework.

mod error;
mod loader;
mod types;
mod validation;

pub use error::ConfigValidationError;
pub use loader::{apply_env_overlay, discover_config_paths, load_config, load_from_file};
pub use types::{
    AgentConfig, FrameworkConfig, LlmConfig, LogFormat, MonitoringConfig, ObservabilityConfig,
    OtlpProtocol, RuntimeConfig, RuntimeProviderTier, RuntimeRoutingPolicy,
    RuntimeRoutingProfile, SecurityConfig, SupervisionConfig, TransportConfig,
};
