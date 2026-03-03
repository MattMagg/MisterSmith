# Implementation Configuration

## Overview

Concrete configuration structures and patterns for implementing agents in the Mister Smith Framework.
Focused on practical configuration, validation, and module organization.

## Navigation

- [System Architecture](system-architecture.md) - Complete architectural specifications
- [Type Definitions](type-definitions.md) - Core type system
- [Dependency Specifications](dependency-specifications.md) - External dependencies
- [Integration Patterns](./integration-patterns.md) - System integration approaches
- [Coding Standards](coding-standards.md) - Development guidelines

---

## 1. Agent Implementation Configuration

### 1.1 Core Agent Configuration

```rust
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use validator::Validate;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct AgentConfig {
    #[serde(default)]
    pub runtime: RuntimeConfig,
    
    #[serde(default)]
    pub supervision: SupervisionConfig,
    
    #[serde(default)]
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct RuntimeConfig {
    /// Number of worker threads for async tasks
    #[validate(range(min = 1, max = 1024))]
    #[serde(default = "default_worker_threads")]
    pub worker_threads: usize,
    
    /// Number of blocking threads for I/O operations
    #[validate(range(min = 1, max = 512))]
    #[serde(default = "default_blocking_threads")]
    pub blocking_threads: usize,
    
    /// Maximum memory in bytes (0 = unlimited)
    #[validate(range(min = 0))]
    #[serde(default = "default_max_memory")]
    pub max_memory: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct SupervisionConfig {
    /// Maximum restart attempts before escalation
    #[validate(range(min = 0, max = 100))]
    #[serde(default = "default_max_restart_attempts")]
    pub max_restart_attempts: u32,
    
    /// Time window for restart attempts (in seconds)
    #[validate(range(min = 1, max = 3600))]
    #[serde(with = "humantime_serde", default = "default_restart_window")]
    pub restart_window: Duration,
    
    /// Timeout before escalating to parent supervisor (in seconds)
    #[validate(range(min = 1, max = 300))]
    #[serde(with = "humantime_serde", default = "default_escalation_timeout")]
    pub escalation_timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct MonitoringConfig {
    /// Health check interval (in seconds)
    #[validate(range(min = 1, max = 300))]
    #[serde(with = "humantime_serde", default = "default_health_check_interval")]
    pub health_check_interval: Duration,
    
    /// Metrics export interval (in seconds)
    #[validate(range(min = 1, max = 600))]
    #[serde(with = "humantime_serde", default = "default_metrics_export_interval")]
    pub metrics_export_interval: Duration,
    
    /// Log level (trace, debug, info, warn, error)
    #[validate(regex = "^(trace|debug|info|warn|error)$")]
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

// Default value functions
fn default_worker_threads() -> usize {
    num_cpus::get() * 2
}

fn default_blocking_threads() -> usize {
    num_cpus::get()
}

fn default_max_memory() -> usize {
    0 // Unlimited
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
```

### 1.2 Agent Orchestration Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct OrchestrationConfig {
    /// Number of agent replicas
    #[validate(range(min = 1, max = 100))]
    #[serde(default = "default_replicas")]
    pub replicas: u32,
    
    #[serde(default)]
    pub resources: ResourceConfig,
    
    #[serde(default)]
    pub probes: ProbeConfig,
    
    #[serde(default)]
    pub autoscaling: AutoscalingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct ResourceConfig {
    #[serde(default)]
    pub requests: ResourceAllocation,
    
    #[serde(default)]
    pub limits: ResourceAllocation,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct ResourceAllocation {
    /// CPU cores (fractional values allowed)
    #[validate(range(min = 0.1, max = 64.0))]
    #[serde(default = "default_cpu")]
    pub cpu: f64,
    
    /// Memory in MB
    #[validate(range(min = 128, max = 65536))]
    #[serde(default = "default_memory_mb")]
    pub memory_mb: u32,
    
    /// Disk space in MB
    #[validate(range(min = 256, max = 1048576))]
    #[serde(default = "default_disk_mb")]
    pub disk_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct ProbeConfig {
    #[serde(default)]
    pub liveness: ProbeSettings,
    
    #[serde(default)]
    pub readiness: ProbeSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct ProbeSettings {
    /// Initial delay before probing (in seconds)
    #[validate(range(min = 0, max = 300))]
    #[serde(with = "humantime_serde", default = "default_initial_delay")]
    pub initial_delay: Duration,
    
    /// Probe interval (in seconds)
    #[validate(range(min = 1, max = 300))]
    #[serde(with = "humantime_serde", default = "default_probe_interval")]
    pub interval: Duration,
    
    /// Probe timeout (in seconds)
    #[validate(range(min = 1, max = 60))]
    #[serde(with = "humantime_serde", default = "default_probe_timeout")]
    pub timeout: Duration,
    
    /// Number of failures before marking unhealthy
    #[validate(range(min = 1, max = 10))]
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: u32,
    
    /// Number of successes before marking healthy
    #[validate(range(min = 1, max = 10))]
    #[serde(default = "default_success_threshold")]
    pub success_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct AutoscalingConfig {
    /// Minimum number of replicas
    #[validate(range(min = 1, max = 50))]
    #[serde(default = "default_min_replicas")]
    pub min_replicas: u32,
    
    /// Maximum number of replicas
    #[validate(range(min = 1, max = 100))]
    #[serde(default = "default_max_replicas")]
    pub max_replicas: u32,
    
    #[serde(default)]
    pub scaling_policy: ScalingPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct ScalingPolicy {
    /// Target CPU utilization percentage
    #[validate(range(min = 10, max = 95))]
    #[serde(default = "default_target_cpu_percent")]
    pub target_cpu_percent: u32,
    
    /// Target memory utilization percentage
    #[validate(range(min = 10, max = 95))]
    #[serde(default = "default_target_memory_percent")]
    pub target_memory_percent: u32,
    
    /// Scale up threshold (consecutive periods above target)
    #[validate(range(min = 1, max = 10))]
    #[serde(default = "default_scale_up_threshold")]
    pub scale_up_threshold: u32,
    
    /// Scale down threshold (consecutive periods below target)
    #[validate(range(min = 1, max = 20))]
    #[serde(default = "default_scale_down_threshold")]
    pub scale_down_threshold: u32,
    
    /// Cooldown period after scaling (in seconds)
    #[validate(range(min = 30, max = 600))]
    #[serde(with = "humantime_serde", default = "default_cooldown_period")]
    pub cooldown_period: Duration,
}

// Default value functions for orchestration
fn default_replicas() -> u32 {
    1
}

fn default_cpu() -> f64 {
    0.5
}

fn default_memory_mb() -> u32 {
    512
}

fn default_disk_mb() -> u32 {
    1024
}

fn default_initial_delay() -> Duration {
    Duration::from_secs(10)
}

fn default_probe_interval() -> Duration {
    Duration::from_secs(10)
}

fn default_probe_timeout() -> Duration {
    Duration::from_secs(5)
}

fn default_failure_threshold() -> u32 {
    3
}

fn default_success_threshold() -> u32 {
    1
}

fn default_min_replicas() -> u32 {
    1
}

fn default_max_replicas() -> u32 {
    10
}

fn default_target_cpu_percent() -> u32 {
    70
}

fn default_target_memory_percent() -> u32 {
    80
}

fn default_scale_up_threshold() -> u32 {
    3
}

fn default_scale_down_threshold() -> u32 {
    5
}

fn default_cooldown_period() -> Duration {
    Duration::from_secs(120)
}
```

### 1.3 Configuration Validation System

```rust
use std::env;
use std::path::PathBuf;
use config::{Config, ConfigError, Environment, File};
use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigValidationError {
    #[error("Configuration validation failed: {0}")]
    ValidationError(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Invalid value for {field}: {reason}")]
    InvalidValue { field: String, reason: String },
    
    #[error("Environment variable error: {0}")]
    EnvVarError(String),
    
    #[error("Configuration file error: {0}")]
    FileError(#[from] std::io::Error),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(#[from] serde_json::Error),
}

pub struct ConfigValidator {
    schema_validator: jsonschema::JSONSchema,
    environment_prefix: String,
}

impl ConfigValidator {
    pub fn new(environment_prefix: &str) -> Self {
        Self {
            schema_validator: Self::build_schema_validator(),
            environment_prefix: environment_prefix.to_string(),
        }
    }
    
    /// Load and validate configuration from multiple sources
    pub fn load_config<T>(&self) -> Result<T, ConfigValidationError>
    where
        T: DeserializeOwned + Validate + JsonSchema,
    {
        let mut builder = Config::builder();
        
        // 1. Load default configuration
        builder = builder.add_source(Config::try_from(&T::default())?);        
        
        // 2. Load from configuration files
        let config_paths = self.get_config_paths();
        for path in config_paths {
            if path.exists() {
                builder = builder.add_source(File::from(path));
            }
        }
        
        // 3. Override with environment variables
        builder = builder.add_source(
            Environment::with_prefix(&self.environment_prefix)
                .separator("__")
                .try_parsing(true)
        );
        
        // 4. Build and deserialize
        let config: T = builder
            .build()
            .map_err(|e| ConfigValidationError::ValidationError(e.to_string()))?
            .try_deserialize()
            .map_err(|e| ConfigValidationError::DeserializationError(e))?;
        
        // 5. Validate using validator crate
        config.validate()
            .map_err(|e| ConfigValidationError::ValidationError(e.to_string()))?;
        
        // 6. Validate against JSON schema
        self.validate_schema(&config)?;
        
        Ok(config)
    }
    
    /// Validate configuration against JSON schema
    fn validate_schema<T: Serialize>(&self, config: &T) -> Result<(), ConfigValidationError> {
        let value = serde_json::to_value(config)?;
        
        match self.schema_validator.validate(&value) {
            Ok(_) => Ok(()),
            Err(errors) => {
                let error_messages: Vec<String> = errors
                    .map(|e| format!("{}: {}", e.instance_path, e))
                    .collect();
                Err(ConfigValidationError::ValidationError(
                    error_messages.join("; ")
                ))
            }
        }
    }
    
    /// Get configuration file paths in priority order
    fn get_config_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        // 1. Default config in /etc
        paths.push(PathBuf::from("/etc/mister-smith/config.toml"));
        
        // 2. User config in home directory
        if let Ok(home) = env::var("HOME") {
            paths.push(PathBuf::from(home).join(".mister-smith/config.toml"));
        }
        
        // 3. Local config in current directory
        paths.push(PathBuf::from("./mister-smith.toml"));
        
        // 4. Environment-specific config
        if let Ok(env_name) = env::var("MS_ENVIRONMENT") {
            paths.push(PathBuf::from(format!("./config/{}.toml", env_name)));
        }
        
        paths
    }
    
    fn build_schema_validator() -> jsonschema::JSONSchema {
        // This would typically load the actual JSON schema
        // For now, returning a placeholder
        let schema = serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        });
        
        jsonschema::JSONSchema::compile(&schema)
            .expect("Failed to compile JSON schema")
    }
}

/// Environment variable mapping helper
pub struct EnvVarMapper {
    prefix: String,
}

impl EnvVarMapper {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }
    
    /// Map configuration field to environment variable name
    pub fn map_field(&self, field_path: &str) -> String {
        format!(
            "{}_{}",
            self.prefix,
            field_path.to_uppercase().replace('.', "__")
        )
    }
    
    /// Get all mapped environment variables with descriptions
    pub fn get_env_var_mappings() -> Vec<EnvVarMapping> {
        vec![
            EnvVarMapping {
                env_var: "MS_RUNTIME__WORKER_THREADS".to_string(),
                config_path: "runtime.worker_threads".to_string(),
                description: "Number of worker threads".to_string(),
                example: "8".to_string(),
            },
            EnvVarMapping {
                env_var: "MS_RUNTIME__MAX_MEMORY".to_string(),
                config_path: "runtime.max_memory".to_string(),
                description: "Maximum memory in bytes (0 = unlimited)".to_string(),
                example: "1073741824".to_string(),
            },
            EnvVarMapping {
                env_var: "MS_SUPERVISION__MAX_RESTART_ATTEMPTS".to_string(),
                config_path: "supervision.max_restart_attempts".to_string(),
                description: "Maximum restart attempts before escalation".to_string(),
                example: "5".to_string(),
            },
            EnvVarMapping {
                env_var: "MS_MONITORING__LOG_LEVEL".to_string(),
                config_path: "monitoring.log_level".to_string(),
                description: "Log level (trace|debug|info|warn|error)".to_string(),
                example: "info".to_string(),
            },
        ]
    }
}

#[derive(Debug, Clone)]
pub struct EnvVarMapping {
    pub env_var: String,
    pub config_path: String,
    pub description: String,
    pub example: String,
}
```

### 1.4 Configuration Migration

```rust
use semver::Version;
use std::collections::HashMap;

/// Configuration migration system for handling version upgrades
pub struct ConfigMigrator {
    migrations: HashMap<Version, Box<dyn ConfigMigration>>,
}

impl ConfigMigrator {
    pub fn new() -> Self {
        let mut migrations = HashMap::new();
        
        // Register migrations
        migrations.insert(
            Version::parse("1.0.0").unwrap(),
            Box::new(V1ToV2Migration) as Box<dyn ConfigMigration>,
        );
        
        Self { migrations }
    }
    
    /// Migrate configuration from one version to another
    pub fn migrate(
        &self,
        config: serde_json::Value,
        from_version: &Version,
        to_version: &Version,
    ) -> Result<serde_json::Value, ConfigValidationError> {
        let mut current_config = config;
        let mut current_version = from_version.clone();
        
        // Apply migrations in sequence
        while current_version < *to_version {
            if let Some(migration) = self.migrations.get(&current_version) {
                current_config = migration.migrate(current_config)?;
                current_version = migration.target_version();
            } else {
                return Err(ConfigValidationError::ValidationError(
                    format!("No migration path from {} to {}", current_version, to_version)
                ));
            }
        }
        
        Ok(current_config)
    }
}

/// Trait for configuration migrations
pub trait ConfigMigration: Send + Sync {
    fn migrate(&self, config: serde_json::Value) -> Result<serde_json::Value, ConfigValidationError>;
    fn target_version(&self) -> Version;
    fn description(&self) -> &str;
}

/// Example migration from v1 to v2
struct V1ToV2Migration;

impl ConfigMigration for V1ToV2Migration {
    fn migrate(&self, mut config: serde_json::Value) -> Result<serde_json::Value, ConfigValidationError> {
        // Example: Rename field
        if let Some(obj) = config.as_object_mut() {
            if let Some(old_value) = obj.remove("max_threads") {
                obj.insert("worker_threads".to_string(), old_value);
            }
        }
        
        Ok(config)
    }
    
    fn target_version(&self) -> Version {
        Version::parse("2.0.0").unwrap()
    }
    
    fn description(&self) -> &str {
        "Migrate configuration from v1 to v2: rename max_threads to worker_threads"
    }
}
```

### 1.5 Configuration Usage Examples

```rust
// Example 1: Agent-specific configuration
use mister_smith_core::config::*;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SearchAgentConfig {
    #[serde(flatten)]
    pub base: AgentConfig,
    
    /// Maximum search depth
    #[validate(range(min = 1, max = 10))]
    pub max_search_depth: u32,
    
    /// Search timeout in seconds
    #[serde(with = "humantime_serde")]
    pub search_timeout: Duration,
    
    /// Allowed tools for this agent
    pub allowed_tools: Vec<String>,
}

// Example 2: Loading agent configuration
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration with validation
    let validator = ConfigValidator::new("MS");
    let config: SearchAgentConfig = validator.load_config()?;
    
    // Create agent with config
    let agent = SearchAgent::builder()
        .id(AgentId::new())
        .config(config)
        .tool_registry(Arc::new(ToolRegistry::new()))
        .build()?;
    
    // Start agent actor
    let actor_ref = ActorSystem::spawn(agent).await?;
    
    Ok(())
}

// Example 3: Configuration file (search-agent.toml)
/*
# Base agent configuration
[runtime]
worker_threads = 4
blocking_threads = 2
max_memory = 1073741824  # 1GB

[supervision]
max_restart_attempts = 3
restart_window = "1m"
escalation_timeout = "15s"

[monitoring]
health_check_interval = "10s"
metrics_export_interval = "30s"
log_level = "info"

# Agent-specific configuration
max_search_depth = 5
search_timeout = "30s"
allowed_tools = ["web_search", "file_search", "database_query"]

# Resource limits for this agent
[orchestration.resources.limits]
cpu = 1.0
memory_mb = 1024
disk_mb = 512
*/

// Example 4: Environment variable configuration
// export MS_RUNTIME__WORKER_THREADS=8
// export MS_MAX_SEARCH_DEPTH=3
// export MS_SEARCH_TIMEOUT=60s
// export MS_ALLOWED_TOOLS=web_search,file_search

// Example 5: Dynamic configuration updates
impl SearchAgent {
    pub async fn update_config(&mut self, new_config: SearchAgentConfig) -> Result<()> {
        // Validate new configuration
        new_config.validate()?;
        
        // Check if restart required
        let restart_required = self.config.runtime != new_config.runtime;
        
        // Update non-critical settings immediately
        self.config.max_search_depth = new_config.max_search_depth;
        self.config.search_timeout = new_config.search_timeout;
        
        // Schedule restart if needed
        if restart_required {
            self.schedule_graceful_restart().await?;
        }
        
        Ok(())
    }
}
```

### 1.6 Configuration Error Handling

```rust
/// Comprehensive error handling for configuration issues
pub fn handle_config_error(error: ConfigValidationError) -> String {
    match error {
        ConfigValidationError::ValidationError(msg) => {
            format!("Configuration validation failed:\n  {}", msg)
        }
        ConfigValidationError::MissingField(field) => {
            format!(
                "Missing required configuration field: {}\n  \
                Hint: Set {} environment variable or add to config file",
                field,
                EnvVarMapper::new("MS").map_field(&field)
            )
        }
        ConfigValidationError::InvalidValue { field, reason } => {
            format!(
                "Invalid configuration value for {}:\n  {}\n  \
                Hint: Check the value constraints in the documentation",
                field, reason
            )
        }
        ConfigValidationError::EnvVarError(msg) => {
            format!("Environment variable error:\n  {}", msg)
        }
        ConfigValidationError::FileError(err) => {
            format!("Configuration file error:\n  {}", err)
        }
        ConfigValidationError::DeserializationError(err) => {
            format!(
                "Failed to parse configuration:\n  {}\n  \
                Hint: Check TOML syntax and field types",
                err
            )
        }
    }
}

/// Helper to print all available environment variables
pub fn print_env_var_help() {
    println!("Available environment variables:\n");
    
    for mapping in EnvVarMapper::get_env_var_mappings() {
        println!("  {}=", mapping.env_var);
        println!("    Description: {}", mapping.description);
        println!("    Config path: {}", mapping.config_path);
        println!("    Example: {}={}", mapping.env_var, mapping.example);
        println!();
    }
}
```

## 2. Module Organization Structure

```rust
// src/lib.rs
pub mod core;
pub mod actors;
pub mod supervision;
pub mod async_patterns;
pub mod events;
pub mod resources;
pub mod transport;
pub mod tools;
pub mod errors;
pub mod types;

// Re-export commonly used types
pub use errors::SystemError;
pub use types::*;

// Core system prelude
pub mod prelude {
    pub use crate::core::{RuntimeManager, RuntimeConfig};
    pub use crate::actors::{Actor, ActorSystem, ActorRef};
    pub use crate::async_patterns::{AsyncTask, TaskExecutor, StreamProcessor};
    pub use crate::tools::{Tool, ToolBus, ToolSchema};
    pub use crate::types::*;
    pub use crate::errors::*;
}
```

```text
src/
├── lib.rs                    // Main crate exports and prelude
├── core/                     // Core system components
│   ├── mod.rs               // Module exports
│   ├── runtime.rs           // RuntimeManager, RuntimeConfig  
│   ├── system.rs            // SystemCore, component wiring
│   └── config.rs            // Configuration management
├── actors/                   // Actor system implementation
│   ├── mod.rs               // Module exports
│   ├── actor.rs             // Actor trait, ActorRef
│   ├── system.rs            // ActorSystem
│   └── mailbox.rs           // Mailbox, message handling
├── supervision/              // Supervision tree
│   ├── mod.rs               // Module exports
│   ├── supervisor.rs        // Supervisor trait, strategies
│   ├── tree.rs              // SupervisionTree
│   └── failure.rs           // FailureDetector, CircuitBreaker
├── async_patterns/           // Async pattern implementations
│   ├── mod.rs               // Module exports
│   ├── tasks.rs             // TaskExecutor, AsyncTask trait
│   ├── streams.rs           // StreamProcessor
│   └── middleware.rs        // AgentMiddleware pattern
├── events/                   // Event-driven architecture
│   ├── mod.rs               // Module exports
│   ├── bus.rs               // EventBus
│   ├── handler.rs           // EventHandler trait
│   └── types.rs             // Event types, EventResult
├── resources/                // Resource management
│   ├── mod.rs               // Module exports
│   ├── pool.rs              // ConnectionPool
│   ├── manager.rs           // ResourceManager
│   └── health.rs            // HealthCheck, monitoring
├── transport/                // Communication layer
│   ├── mod.rs               // Module exports
│   ├── bridge.rs            // MessageBridge
│   └── routing.rs           // RoutingStrategy
├── tools/                    // Tool system
│   ├── mod.rs               // Module exports
│   ├── bus.rs               // ToolBus
│   └── agent_tool.rs        // AgentTool pattern
├── errors.rs                 // Central error types
└── types.rs                  // Core type definitions
```

## 3. Implementation Checklist

### Configuration Checklist

- [ ] **Validation**
  - [ ] All fields have validation rules
  - [ ] Default values are sensible
  - [ ] Environment variable mapping works
  - [ ] Schema validation passes

- [ ] **Error Handling**
  - [ ] Clear error messages for invalid config
  - [ ] Helpful hints for missing fields
  - [ ] Migration path for old configs
  - [ ] Validation errors are actionable

- [ ] **Testing**
  - [ ] Unit tests for validation logic
  - [ ] Integration tests for env vars
  - [ ] Migration tests for upgrades
  - [ ] Edge case coverage

### Module Organization Checklist

- [ ] **Structure**
  - [ ] Clear module boundaries
  - [ ] No circular dependencies
  - [ ] Proper re-exports in mod.rs
  - [ ] Test modules organized

- [ ] **Dependencies**
  - [ ] Minimal external dependencies
  - [ ] Version pinning for stability
  - [ ] Feature flags used appropriately
  - [ ] Security audit passing

## Key Implementation Notes

### Dependencies Required

```toml
[dependencies]
tokio = { version = "1.45.1", features = ["full"] }
futures = "0.3"
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
dashmap = "6.0"
num_cpus = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
metrics = "0.23"

# Configuration validation dependencies
config = "0.14"
validator = { version = "0.18", features = ["derive"] }
schemars = "0.8"
jsonschema = "0.18"
humantime-serde = "1.1"
semver = "1.0"
```

### Quick Start Example

```rust
use mister_smith_core::prelude::*;

#[tokio::main]
async fn main() -> Result<(), SystemError> {
    // 1. Load agent configuration
    let config: AgentConfig = ConfigValidator::new("MS").load_config()?;
    
    // 2. Initialize actor system
    let actor_system = ActorSystem::with_config(config.clone());
    
    // 3. Create tool registry
    let tool_registry = Arc::new(ToolRegistry::new());
    tool_registry.register(WebSearchTool::new()).await?;
    tool_registry.register(FileSystemTool::new()).await?;
    
    // 4. Create and spawn agent
    let agent = ExampleAgent::new(
        AgentId::new(),
        tool_registry.clone(),
        config
    );
    
    let agent_ref = actor_system.spawn_actor(agent).await?;
    
    // 5. Send message to agent
    let (tx, rx) = oneshot::channel();
    agent_ref.send(AgentMessage::ExecuteTool {
        tool_id: "web_search".to_string(),
        params: json!({ "query": "rust async patterns" }),
        respond_to: tx,
    }).await?;
    
    // 6. Wait for result
    let result = rx.await??;
    println!("Tool result: {:?}", result);
    
    // 7. Graceful shutdown
    actor_system.shutdown().await?;
    
    Ok(())
}
```

---

## Related Documents

- [System Architecture](system-architecture.md) - Core architectural patterns
- [Type Definitions](type-definitions.md) - Type system specifications
- [Coding Standards](coding-standards.md) - Development guidelines
- [Dependency Specifications](dependency-specifications.md) - External dependencies
