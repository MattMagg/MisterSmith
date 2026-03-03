# Configuration Management Specifications

## Complete Configuration Schemas and Management Framework

### Cross-Reference Integration

**Related Documentation**:

- [Configuration & Deployment Specifications](configuration-deployment-specifications.md) - Deployment patterns and infrastructure integration
- [System Architecture](../core-architecture/system-architecture.md) - Overall framework architecture
- [Agent Lifecycle](../data-management/agent-lifecycle.md) - Agent initialization and configuration loading

### Technical Overview

This specification defines the complete configuration management system for multi-tier agent framework deployment. It integrates with deployment patterns defined in [Configuration & Deployment Specifications](configuration-deployment-specifications.md) to provide:

- **4-Layer Configuration Hierarchy** with clear precedence rules
- **Tier-based Feature Sets** (tier_1/tier_2/tier_3) with progressive enablement  
- **Advanced Type System** including Duration, Size, Array, and Map types
- **Hot-Reload Capabilities** with safety mechanisms for development
- **Container Optimization** patterns with build caching strategies

### Overview

This document provides comprehensive configuration management specifications for the Mister Smith AI Agent Framework. It defines complete configuration file schemas,
environment variable specifications, validation rules, default value hierarchies, and configuration override patterns that support the framework's multi-tier
deployment architecture and feature-based modularity.

For deployment implementation patterns that utilize these configurations, see [Configuration & Deployment Specifications](configuration-deployment-specifications.md).

---

## 1. Configuration System Architecture

### 1.1 Configuration Management Principles

- **Hierarchical Configuration**: Layer-based configuration with clear precedence rules
- **Environment Tier Support**: Native support for tier_1 (experimental), tier_2 (validation), and tier_3 (operational) environments
- **Feature-Driven Configuration**: Configuration schemas that align with feature flag architecture
- **Security-First Approach**: Secure secret management and configuration validation
- **Agent-Optimized Design**: Configuration structure optimized for programmatic agent consumption

### 1.2 Configuration Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                    CONFIGURATION LAYERS                    │
├─────────────────────────────────────────────────────────────┤
│ Layer 4: Runtime Overrides (ENV vars, CLI args)           │
├─────────────────────────────────────────────────────────────┤
│ Layer 3: Environment Configuration (tier-specific)         │
├─────────────────────────────────────────────────────────────┤
│ Layer 2: Feature Configuration (feature flag dependent)    │
├─────────────────────────────────────────────────────────────┤
│ Layer 1: Base Configuration (framework defaults)           │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Configuration File Schemas

### 2.1 Base Framework Configuration Schema

#### 2.1.1 Core Framework Settings (framework.toml)

```toml
# framework.toml - Base framework configuration
[framework]
name = "mister-smith-framework"
version = "0.1.0"
environment_tier = "tier_2"  # tier_1, tier_2, tier_3
log_level = "info"           # trace, debug, info, warn, error
config_validation = true
hot_reload = false           # Development feature

[framework.runtime]
# Tokio runtime configuration
worker_threads = "auto"      # auto, or specific number
max_blocking_threads = 512
thread_stack_size = "2MB"
enable_io = true
enable_time = true
enable_rt = true

[framework.features]
# Feature flag control
actors = true
tools = true
monitoring = true
supervision = true
config = true
security = false             # Enabled in higher tiers
encryption = false           # Requires security
persistence = false          # Optional feature
clustering = false           # Optional feature
http_client = false          # Optional feature
metrics = false              # Optional feature
tracing = false              # Optional feature

[framework.agent]
# Default agent configuration
agent_type = "worker"        # orchestrator, worker, messaging
agent_tier = "standard"      # minimal, standard, premium
agent_id = "${HOSTNAME}-${PID}" # Auto-generated if not specified
heartbeat_interval = 30      # seconds
health_check_interval = 15   # seconds
graceful_shutdown_timeout = 60 # seconds
max_concurrent_tasks = 100
```

#### 2.1.2 Agent-Specific Configuration Schemas

```toml
# orchestrator.toml - Orchestrator agent configuration
[orchestrator]
enabled = true
bind_address = "0.0.0.0"
http_port = 8080
grpc_port = 9090
max_connections = 1000
connection_timeout = 30

[orchestrator.coordination]
consensus_timeout = 5000     # milliseconds
election_timeout = 150       # milliseconds
heartbeat_interval = 50      # milliseconds
max_entries_per_batch = 100
snapshot_threshold = 10000

[orchestrator.task_management]
max_queued_tasks = 10000
task_timeout = 300           # seconds
retry_attempts = 3
retry_backoff = "exponential" # linear, exponential
dead_letter_enabled = true

# worker.toml - Worker agent configuration
[worker]
enabled = true
bind_address = "0.0.0.0"
http_port = 8081
concurrency = 4              # Number of parallel tasks
workspace_dir = "/app/workspace"
temp_dir = "/tmp"
max_task_duration = 300      # seconds

[worker.resources]
max_memory_mb = 512
max_cpu_percent = 80
max_disk_mb = 1024
cleanup_interval = 3600      # seconds

[worker.claude_cli]
# Claude CLI integration settings
parallel_default = 4
parallel_max_agents = 50
parallel_cpu_budget = "4.0"
parallel_memory_budget = "4Gi"
hook_system_enabled = true
output_parsing_enabled = true

# messaging.toml - Messaging agent configuration
[messaging]
enabled = true
bind_address = "0.0.0.0"
client_port = 4222
cluster_port = 6222
http_port = 8222
jetstream_enabled = true
clustering_enabled = false

[messaging.limits]
max_payload = "1MB"
max_connections = 64000
max_subscriptions = 0        # unlimited
max_channels = 0             # unlimited
```

### 2.2 Runtime Configuration Schema

#### 2.2.1 Security Configuration (security.toml)

```toml
# security.toml - Security and authentication configuration
[security]
enabled = true
encryption_enabled = true
auth_required = true
audit_enabled = true
tls_enabled = true
mtls_enabled = false         # Mutual TLS

[security.encryption]
algorithm = "chacha20poly1305" # aes-gcm, chacha20poly1305
key_rotation_interval = 86400  # seconds (24 hours)
key_derivation = "argon2id"
salt_length = 32

[security.authentication]
method = "jwt"               # jwt, oauth2, ldap
jwt_secret = "${JWT_SECRET}" # Environment variable reference
jwt_expiry = 3600           # seconds (1 hour)
refresh_enabled = true
refresh_expiry = 604800     # seconds (7 days)

[security.authorization]
rbac_enabled = true
default_role = "user"
admin_role = "admin"
role_inheritance = true
policy_file = "rbac_policy.yaml"

[security.audit]
log_level = "info"
log_format = "json"
log_destination = "file"     # file, syslog, remote
log_file = "/var/log/mister-smith/audit.log"
log_rotation = true
max_log_size = "100MB"
max_log_files = 10
```

#### 2.2.2 Messaging Configuration (messaging.toml)

```toml
# messaging.toml - NATS messaging configuration
[messaging.nats]
url = "nats://localhost:4222"
cluster_id = "mister-smith-cluster"
client_id = "${AGENT_TYPE}-${AGENT_ID}"
max_reconnect = -1           # unlimited
reconnect_wait = 2           # seconds
ping_interval = 120          # seconds
max_pending_msgs = 65536

[messaging.nats.tls]
enabled = false
cert_file = "/etc/tls/certs/client.crt"
key_file = "/etc/tls/private/client.key"
ca_file = "/etc/tls/certs/ca.crt"
verify = true

[messaging.jetstream]
enabled = true
max_memory = "256MB"
max_storage = "1GB"
replicas = 1
retention = "limits"         # limits, interest, workqueue
max_age = 86400             # seconds (24 hours)

[messaging.subjects]
prefix = "mister-smith"
agent_heartbeat = "${prefix}.agent.heartbeat.${agent_type}"
task_queue = "${prefix}.task.queue.${queue_name}"
task_result = "${prefix}.task.result.${task_id}"
agent_command = "${prefix}.agent.command.${agent_id}"
system_event = "${prefix}.system.event.${event_type}"
```

#### 2.2.3 Persistence Configuration (persistence.toml)

```toml
# persistence.toml - Database and storage configuration
[persistence.database]
enabled = false
driver = "postgresql"        # postgresql, sqlite, mysql
url = "${DATABASE_URL}"
max_connections = 10
min_connections = 1
connection_timeout = 30      # seconds
idle_timeout = 600          # seconds
max_lifetime = 3600         # seconds

[persistence.database.migrations]
enabled = true
auto_migrate = false        # Only in development
migration_dir = "migrations"
migration_table = "_migrations"

[persistence.redis]
enabled = false
url = "${REDIS_URL}"
pool_size = 10
connection_timeout = 5       # seconds
command_timeout = 1         # seconds
reconnect_delay = 1         # seconds
max_retries = 3

[persistence.redis.cluster]
enabled = false
nodes = ["redis-1:6379", "redis-2:6379", "redis-3:6379"]
read_from_replicas = true
max_redirections = 16

[persistence.embedded]
enabled = true              # Embedded database (sled)
data_dir = "/app/data"
cache_size = "64MB"
compression = "zstd"        # lz4, zstd, none
fsync = true
```

### 2.3 Monitoring Configuration Schema

#### 2.3.1 Metrics Configuration (metrics.toml)

```toml
# metrics.toml - Metrics and monitoring configuration
[metrics]
enabled = false
collection_interval = 15    # seconds
export_interval = 30       # seconds
retention_period = 86400   # seconds (24 hours)

[metrics.prometheus]
enabled = false
bind_address = "0.0.0.0"
port = 9090
metrics_path = "/metrics"
registry_namespace = "mister_smith"

[metrics.statsd]
enabled = false
address = "localhost:8125"
prefix = "mister_smith"
tags_enabled = true
sample_rate = 1.0

[metrics.custom]
# Custom metrics configuration
agent_task_count = true
agent_response_time = true
agent_error_rate = true
system_resource_usage = true
message_queue_depth = true
```

#### 2.3.2 Tracing Configuration (tracing.toml)

```toml
# tracing.toml - Distributed tracing configuration
[tracing]
enabled = false
service_name = "mister-smith-agent"
service_version = "0.1.0"
environment = "${ENVIRONMENT_TIER}"
sample_rate = 1.0           # 0.0 to 1.0

[tracing.jaeger]
enabled = false
agent_endpoint = "http://localhost:14268"
collector_endpoint = "http://localhost:14268/api/traces"
max_packet_size = 65000
queue_size = 100

[tracing.zipkin]
enabled = false
endpoint = "http://localhost:9411/api/v2/spans"
timeout = 5                 # seconds

[tracing.otlp]
enabled = false
endpoint = "http://localhost:4317"
timeout = 10               # seconds
compression = "gzip"       # gzip, none
headers = {}
```

---

## 3. Environment Variable Specifications

### 3.1 Environment Variable Naming Conventions

#### 3.1.1 Hierarchical Naming Structure

```bash
# Pattern: MISTER_SMITH_{CATEGORY}_{SUBCATEGORY}_{SETTING}
# All environment variables use SCREAMING_SNAKE_CASE

# Framework Core Variables
MISTER_SMITH_ENVIRONMENT_TIER=tier_2          # tier_1, tier_2, tier_3
MISTER_SMITH_LOG_LEVEL=info                   # trace, debug, info, warn, error
MISTER_SMITH_CONFIG_PATH=/app/config          # Configuration directory
MISTER_SMITH_CONFIG_VALIDATION=true           # Enable configuration validation

# Agent Configuration Variables
MISTER_SMITH_AGENT_TYPE=worker                # orchestrator, worker, messaging
MISTER_SMITH_AGENT_TIER=standard              # minimal, standard, premium
MISTER_SMITH_AGENT_ID=${HOSTNAME}-${PID}      # Unique agent identifier
MISTER_SMITH_AGENT_HEARTBEAT_INTERVAL=30      # Heartbeat interval in seconds
MISTER_SMITH_AGENT_HEALTH_CHECK_INTERVAL=15   # Health check interval

# Runtime Configuration Variables
MISTER_SMITH_RUNTIME_WORKER_THREADS=auto      # Tokio worker thread count
MISTER_SMITH_RUNTIME_MAX_BLOCKING_THREADS=512 # Max blocking threads
MISTER_SMITH_RUNTIME_THREAD_STACK_SIZE=2MB    # Thread stack size
```

#### 3.1.2 Feature Toggle Variables

```bash
# Feature Control Variables
MISTER_SMITH_FEATURE_ACTORS=true              # Enable actor system
MISTER_SMITH_FEATURE_TOOLS=true               # Enable tool integration
MISTER_SMITH_FEATURE_MONITORING=true          # Enable monitoring
MISTER_SMITH_FEATURE_SUPERVISION=true         # Enable supervision trees
MISTER_SMITH_FEATURE_SECURITY=false           # Enable security features
MISTER_SMITH_FEATURE_ENCRYPTION=false         # Enable encryption
MISTER_SMITH_FEATURE_PERSISTENCE=false        # Enable persistence
MISTER_SMITH_FEATURE_CLUSTERING=false         # Enable clustering
MISTER_SMITH_FEATURE_HTTP_CLIENT=false        # Enable HTTP client
MISTER_SMITH_FEATURE_METRICS=false            # Enable metrics collection
MISTER_SMITH_FEATURE_TRACING=false            # Enable distributed tracing
```

#### 3.1.3 Service Integration Variables

```bash
# NATS Messaging Variables
MISTER_SMITH_NATS_URL=nats://localhost:4222   # NATS server URL
MISTER_SMITH_NATS_CLUSTER_ID=mister-smith-cluster # Cluster identifier
MISTER_SMITH_NATS_CLIENT_ID=${AGENT_TYPE}-${AGENT_ID} # Client identifier
MISTER_SMITH_NATS_MAX_RECONNECT=-1            # Max reconnection attempts
MISTER_SMITH_NATS_RECONNECT_WAIT=2            # Reconnection wait time
MISTER_SMITH_NATS_PING_INTERVAL=120           # Ping interval
MISTER_SMITH_NATS_TLS_ENABLED=false           # Enable TLS
MISTER_SMITH_NATS_JETSTREAM_ENABLED=true      # Enable JetStream

# Database Variables
MISTER_SMITH_DB_ENABLED=false                 # Enable database
MISTER_SMITH_DB_DRIVER=postgresql             # Database driver
MISTER_SMITH_DB_URL=${DATABASE_URL}           # Database connection URL
MISTER_SMITH_DB_MAX_CONNECTIONS=10            # Max connection pool size
MISTER_SMITH_DB_CONNECTION_TIMEOUT=30         # Connection timeout
MISTER_SMITH_DB_AUTO_MIGRATE=false            # Auto-run migrations

# Redis Variables
MISTER_SMITH_REDIS_ENABLED=false              # Enable Redis
MISTER_SMITH_REDIS_URL=${REDIS_URL}           # Redis connection URL
MISTER_SMITH_REDIS_POOL_SIZE=10               # Connection pool size
MISTER_SMITH_REDIS_CONNECTION_TIMEOUT=5       # Connection timeout
MISTER_SMITH_REDIS_COMMAND_TIMEOUT=1          # Command timeout
```

#### 3.1.4 Claude CLI Integration Variables

```bash
# Claude CLI Configuration Variables
MISTER_SMITH_CLAUDE_PARALLEL_DEFAULT=4        # Default parallel agents
MISTER_SMITH_CLAUDE_PARALLEL_MAX_AGENTS=50    # Maximum parallel agents
MISTER_SMITH_CLAUDE_PARALLEL_CPU_BUDGET=4.0   # CPU budget for parallel execution
MISTER_SMITH_CLAUDE_PARALLEL_MEMORY_BUDGET=4Gi # Memory budget for parallel execution
MISTER_SMITH_CLAUDE_HOOK_SYSTEM_ENABLED=true  # Enable hook system
MISTER_SMITH_CLAUDE_OUTPUT_PARSING_ENABLED=true # Enable output parsing
MISTER_SMITH_CLAUDE_NATS_SUBJECT_MAPPING=true # Enable NATS subject mapping
MISTER_SMITH_CLAUDE_OBSERVABILITY_INTEGRATION=true # Enable observability
MISTER_SMITH_CLAUDE_SPAN_TAGGING_PER_AGENT=true # Enable span tagging
```

#### 3.1.5 Security Variables

```bash
# Security Configuration Variables
MISTER_SMITH_SECURITY_ENABLED=true            # Enable security features
MISTER_SMITH_SECURITY_ENCRYPTION_ENABLED=true # Enable encryption
MISTER_SMITH_SECURITY_AUTH_REQUIRED=true      # Require authentication
MISTER_SMITH_SECURITY_AUDIT_ENABLED=true      # Enable audit logging
MISTER_SMITH_SECURITY_TLS_ENABLED=true        # Enable TLS
MISTER_SMITH_SECURITY_MTLS_ENABLED=false      # Enable mutual TLS

# Authentication Variables
MISTER_SMITH_AUTH_METHOD=jwt                  # Authentication method
MISTER_SMITH_AUTH_JWT_SECRET=${JWT_SECRET}    # JWT signing secret
MISTER_SMITH_AUTH_JWT_EXPIRY=3600             # JWT token expiry
MISTER_SMITH_AUTH_REFRESH_ENABLED=true       # Enable refresh tokens
MISTER_SMITH_AUTH_REFRESH_EXPIRY=604800      # Refresh token expiry

# Encryption Variables
MISTER_SMITH_ENCRYPTION_ALGORITHM=chacha20poly1305 # Encryption algorithm
MISTER_SMITH_ENCRYPTION_KEY_ROTATION_INTERVAL=86400 # Key rotation interval
MISTER_SMITH_ENCRYPTION_KEY_DERIVATION=argon2id # Key derivation function
```

#### 3.1.6 Monitoring Variables

```bash
# Monitoring Configuration Variables
MISTER_SMITH_MONITORING_ENABLED=false         # Enable monitoring
MISTER_SMITH_MONITORING_METRICS_ENABLED=false # Enable metrics
MISTER_SMITH_MONITORING_TRACING_ENABLED=false # Enable tracing
MISTER_SMITH_MONITORING_HEALTH_CHECKS_ENABLED=true # Enable health checks

# Metrics Variables
MISTER_SMITH_METRICS_COLLECTION_INTERVAL=15   # Collection interval
MISTER_SMITH_METRICS_EXPORT_INTERVAL=30       # Export interval
MISTER_SMITH_METRICS_PROMETHEUS_ENABLED=false # Enable Prometheus
MISTER_SMITH_METRICS_PROMETHEUS_PORT=9090     # Prometheus port
MISTER_SMITH_METRICS_STATSD_ENABLED=false     # Enable StatsD
MISTER_SMITH_METRICS_STATSD_ADDRESS=localhost:8125 # StatsD address

# Tracing Variables
MISTER_SMITH_TRACING_SERVICE_NAME=mister-smith-agent # Service name
MISTER_SMITH_TRACING_SERVICE_VERSION=0.1.0    # Service version
MISTER_SMITH_TRACING_SAMPLE_RATE=1.0          # Sampling rate
MISTER_SMITH_TRACING_JAEGER_ENABLED=false     # Enable Jaeger
MISTER_SMITH_TRACING_JAEGER_ENDPOINT=http://localhost:14268 # Jaeger endpoint
```

### 3.2 Environment Variable Type System

#### 3.2.1 Type Coercion Rules

```rust
// Type coercion specification for environment variables
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Duration(Duration),
    Size(u64),           // Memory/disk sizes with units
    Array(Vec<String>),  // Comma-separated values
    Map(HashMap<String, String>), // Key=value pairs
}

// Parsing rules:
// Boolean: "true", "false", "1", "0", "yes", "no", "on", "off"
// Duration: "30s", "5m", "1h", "2d" (supports s, m, h, d units)
// Size: "1KB", "512MB", "2GB", "1TB" (supports KB, MB, GB, TB units)
// Array: "item1,item2,item3" (comma-separated)
// Map: "key1=value1,key2=value2" (comma-separated key=value pairs)
```

#### 3.2.2 Validation Rules

```toml
# validation.toml - Environment variable validation rules
[validation.agent_type]
type = "string"
required = true
allowed_values = ["orchestrator", "worker", "messaging"]
default = "worker"

[validation.agent_tier]
type = "string"
required = true
allowed_values = ["minimal", "standard", "premium"]
default = "standard"

[validation.environment_tier]
type = "string"
required = true
allowed_values = ["tier_1", "tier_2", "tier_3"]
default = "tier_2"

[validation.log_level]
type = "string"
required = false
allowed_values = ["trace", "debug", "info", "warn", "error"]
default = "info"

[validation.worker_threads]
type = "string_or_integer"
required = false
min_value = 1
max_value = 256
default = "auto"

[validation.heartbeat_interval]
type = "duration"
required = false
min_value = "5s"
max_value = "300s"
default = "30s"

[validation.max_memory]
type = "size"
required = false
min_value = "64MB"
max_value = "32GB"
default = "512MB"
```

---

## 4. Configuration Validation Rules

### 4.1 Schema Validation Framework

#### 4.1.1 Validation Types

```rust
// Configuration validation framework specification
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    Required,
    Type(ConfigType),
    Range { min: Option<f64>, max: Option<f64> },
    Length { min: Option<usize>, max: Option<usize> },
    Pattern(String),  // Regex pattern
    OneOf(Vec<String>),
    Custom(String),   // Custom validation function name
    Dependency { field: String, value: String }, // Field dependency
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigType {
    String,
    Integer,
    Float,
    Boolean,
    Duration,
    Size,
    Array,
    Object,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValidation {
    pub rules: Vec<ValidationRule>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchema {
    pub fields: HashMap<String, FieldValidation>,
    pub global_rules: Vec<ValidationRule>,
}
```

#### 4.1.2 Validation Rule Definitions

```yaml
# validation_rules.yaml - Complete validation rule set
agent_configuration:
  agent_type:
    rules:
      - Required
      - OneOf: ["orchestrator", "worker", "messaging"]
    error_message: "agent_type must be one of: orchestrator, worker, messaging"
  
  agent_tier:
    rules:
      - Required
      - OneOf: ["minimal", "standard", "premium"]
    error_message: "agent_tier must be one of: minimal, standard, premium"
  
  agent_id:
    rules:
      - Required
      - Length: { min: 3, max: 64 }
      - Pattern: "^[a-zA-Z0-9_-]+$"
    error_message: "agent_id must be 3-64 characters, alphanumeric, underscore, or dash"

runtime_configuration:
  worker_threads:
    rules:
      - Type: String
      - Custom: "validate_worker_threads"
    error_message: "worker_threads must be 'auto' or integer between 1-256"
  
  max_blocking_threads:
    rules:
      - Type: Integer
      - Range: { min: 1, max: 1024 }
    error_message: "max_blocking_threads must be between 1-1024"

security_configuration:
  encryption_enabled:
    rules:
      - Type: Boolean
      - Dependency: { field: "security_enabled", value: "true" }
    error_message: "encryption_enabled requires security_enabled=true"
  
  jwt_secret:
    rules:
      - Required
      - Length: { min: 32 }
      - Custom: "validate_jwt_secret_strength"
    error_message: "jwt_secret must be at least 32 characters and cryptographically strong"

messaging_configuration:
  nats_url:
    rules:
      - Required
      - Pattern: "^nats://[^\\s]+$"
    error_message: "nats_url must be a valid NATS URL"
  
  max_reconnect:
    rules:
      - Type: Integer
      - Range: { min: -1, max: 1000 }
    error_message: "max_reconnect must be -1 (unlimited) or 0-1000"

resource_limits:
  max_memory:
    rules:
      - Type: Size
      - Range: { min: 64000000, max: 34359738368 } # 64MB to 32GB in bytes
    error_message: "max_memory must be between 64MB and 32GB"
  
  max_connections:
    rules:
      - Type: Integer
      - Range: { min: 1, max: 65535 }
    error_message: "max_connections must be between 1-65535"
```

### 4.2 Cross-Field Validation Rules

#### 4.2.1 Feature Dependency Validation

```yaml
# feature_dependencies.yaml - Cross-field validation rules
feature_dependencies:
  encryption:
    requires:
      - field: "security_enabled"
        value: true
    message: "encryption features require security to be enabled"
  
  clustering:
    requires:
      - field: "messaging_enabled"
        value: true
    message: "clustering requires messaging to be enabled"
  
  metrics:
    requires:
      - field: "monitoring_enabled"
        value: true
    message: "metrics collection requires monitoring to be enabled"
  
  tracing:
    requires:
      - field: "monitoring_enabled"
        value: true
    message: "distributed tracing requires monitoring to be enabled"

tier_compatibility:
  tier_1:
    allowed_features:
      - actors
      - tools
      - config
    forbidden_features:
      - security
      - encryption
      - clustering
      - persistence
    message: "tier_1 environments only support basic features"
  
  tier_2:
    allowed_features:
      - actors
      - tools
      - config
      - monitoring
      - security
      - persistence
    restricted_features:
      - clustering: false
      - encryption: false
    message: "tier_2 environments support standard features but not clustering/encryption"
  
  tier_3:
    allowed_features: "all"
    message: "tier_3 environments support all features"

resource_constraints:
  minimal_tier:
    max_memory: "256MB"
    max_cpu_percent: 50
    max_connections: 100
    message: "minimal tier has strict resource constraints"
  
  standard_tier:
    max_memory: "1GB"
    max_cpu_percent: 80
    max_connections: 1000
    message: "standard tier has moderate resource constraints"
  
  premium_tier:
    max_memory: "4GB"
    max_cpu_percent: 95
    max_connections: 10000
    message: "premium tier has relaxed resource constraints"
```

### 4.3 Custom Validation Functions

#### 4.3.1 Validation Function Specifications

```rust
// custom_validators.rs - Custom validation function implementations
use std::time::Duration;
use url::Url;

pub fn validate_worker_threads(value: &str) -> Result<(), String> {
    if value == "auto" {
        return Ok(());
    }
    
    match value.parse::<usize>() {
        Ok(n) if n >= 1 && n <= 256 => Ok(()),
        Ok(_) => Err("worker_threads must be between 1-256".to_string()),
        Err(_) => Err("worker_threads must be 'auto' or a valid integer".to_string()),
    }
}

pub fn validate_jwt_secret_strength(secret: &str) -> Result<(), String> {
    if secret.len() < 32 {
        return Err("JWT secret must be at least 32 characters".to_string());
    }
    
    let has_upper = secret.chars().any(|c| c.is_uppercase());
    let has_lower = secret.chars().any(|c| c.is_lowercase());
    let has_digit = secret.chars().any(|c| c.is_numeric());
    let has_special = secret.chars().any(|c| !c.is_alphanumeric());
    
    if !(has_upper && has_lower && has_digit && has_special) {
        return Err("JWT secret must contain uppercase, lowercase, digit, and special characters".to_string());
    }
    
    Ok(())
}

pub fn validate_nats_url(url: &str) -> Result<(), String> {
    match Url::parse(url) {
        Ok(parsed) => {
            if parsed.scheme() != "nats" {
                return Err("URL must use nats:// scheme".to_string());
            }
            if parsed.host().is_none() {
                return Err("URL must specify a host".to_string());
            }
            Ok(())
        }
        Err(e) => Err(format!("Invalid URL: {}", e)),
    }
}

pub fn validate_duration_range(value: &str, min: Duration, max: Duration) -> Result<(), String> {
    match parse_duration(value) {
        Ok(duration) => {
            if duration < min || duration > max {
                return Err(format!("Duration must be between {}s and {}s", 
                    min.as_secs(), max.as_secs()));
            }
            Ok(())
        }
        Err(e) => Err(format!("Invalid duration format: {}", e)),
    }
}

pub fn validate_size_range(value: &str, min_bytes: u64, max_bytes: u64) -> Result<(), String> {
    match parse_size(value) {
        Ok(bytes) => {
            if bytes < min_bytes || bytes > max_bytes {
                return Err(format!("Size must be between {} and {} bytes", 
                    min_bytes, max_bytes));
            }
            Ok(())
        }
        Err(e) => Err(format!("Invalid size format: {}", e)),
    }
}
```

---

## 5. Default Value Definitions

### 5.1 Environment Tier Defaults

#### 5.1.1 Tier 1 (Experimental) Defaults

```toml
# tier_1_defaults.toml - Experimental environment defaults
[framework]
environment_tier = "tier_1"
log_level = "debug"
config_validation = true
hot_reload = true

[framework.runtime]
worker_threads = 2
max_blocking_threads = 32
thread_stack_size = "1MB"
enable_io = true
enable_time = true
enable_rt = true

[framework.features]
actors = true
tools = true
monitoring = false
supervision = true
config = true
security = false
encryption = false
persistence = false
clustering = false
http_client = false
metrics = false
tracing = false

[framework.agent]
agent_tier = "minimal"
heartbeat_interval = 60
health_check_interval = 30
graceful_shutdown_timeout = 30
max_concurrent_tasks = 10

[resources]
max_memory = "128MB"
max_cpu_percent = 50
max_connections = 10
```

#### 5.1.2 Tier 2 (Validation) Defaults

```toml
# tier_2_defaults.toml - Validation environment defaults
[framework]
environment_tier = "tier_2"
log_level = "info"
config_validation = true
hot_reload = false

[framework.runtime]
worker_threads = "auto"
max_blocking_threads = 128
thread_stack_size = "2MB"
enable_io = true
enable_time = true
enable_rt = true

[framework.features]
actors = true
tools = true
monitoring = true
supervision = true
config = true
security = true
encryption = false
persistence = true
clustering = false
http_client = true
metrics = true
tracing = false

[framework.agent]
agent_tier = "standard"
heartbeat_interval = 30
health_check_interval = 15
graceful_shutdown_timeout = 60
max_concurrent_tasks = 100

[resources]
max_memory = "512MB"
max_cpu_percent = 80
max_connections = 1000
```

#### 5.1.3 Tier 3 (Operational) Defaults

```toml
# tier_3_defaults.toml - Operational environment defaults
[framework]
environment_tier = "tier_3"
log_level = "warn"
config_validation = true
hot_reload = false

[framework.runtime]
worker_threads = "auto"
max_blocking_threads = 512
thread_stack_size = "2MB"
enable_io = true
enable_time = true
enable_rt = true

[framework.features]
actors = true
tools = true
monitoring = true
supervision = true
config = true
security = true
encryption = true
persistence = true
clustering = true
http_client = true
metrics = true
tracing = true

[framework.agent]
agent_tier = "premium"
heartbeat_interval = 30
health_check_interval = 15
graceful_shutdown_timeout = 120
max_concurrent_tasks = 1000

[resources]
max_memory = "2GB"
max_cpu_percent = 95
max_connections = 10000
```

### 5.2 Agent Type Defaults

#### 5.2.1 Orchestrator Agent Defaults

```toml
# orchestrator_defaults.toml
[orchestrator]
bind_address = "0.0.0.0"
http_port = 8080
grpc_port = 9090
max_connections = 1000
connection_timeout = 30

[orchestrator.coordination]
consensus_timeout = 5000
election_timeout = 150
heartbeat_interval = 50
max_entries_per_batch = 100
snapshot_threshold = 10000

[orchestrator.task_management]
max_queued_tasks = 10000
task_timeout = 300
retry_attempts = 3
retry_backoff = "exponential"
dead_letter_enabled = true

[orchestrator.clustering]
enabled = false
cluster_size = 3
replication_factor = 2
sync_timeout = 1000
```

#### 5.2.2 Worker Agent Defaults

```toml
# worker_defaults.toml
[worker]
bind_address = "0.0.0.0"
http_port = 8081
concurrency = 4
workspace_dir = "/app/workspace"
temp_dir = "/tmp"
max_task_duration = 300

[worker.resources]
max_memory_mb = 512
max_cpu_percent = 80
max_disk_mb = 1024
cleanup_interval = 3600

[worker.claude_cli]
parallel_default = 4
parallel_max_agents = 50
parallel_cpu_budget = "4.0"
parallel_memory_budget = "4Gi"
hook_system_enabled = true
output_parsing_enabled = true
```

#### 5.2.3 Messaging Agent Defaults

```toml
# messaging_defaults.toml
[messaging]
bind_address = "0.0.0.0"
client_port = 4222
cluster_port = 6222
http_port = 8222
jetstream_enabled = true
clustering_enabled = false

[messaging.limits]
max_payload = "1MB"
max_connections = 64000
max_subscriptions = 0
max_channels = 0

[messaging.performance]
write_deadline = "2s"
max_pending = "256MB"
max_control_line = 4096
```

---

## 6. Configuration Override Hierarchies

### 6.1 Override Precedence Rules

#### 6.1.1 Precedence Order (Highest to Lowest)

```text
1. Command Line Arguments (--config-key=value)
2. Environment Variables (MISTER_SMITH_*)
3. Environment-Specific Config Files (tier_1.toml, tier_2.toml, tier_3.toml)
4. Feature-Specific Config Files (security.toml, messaging.toml, etc.)
5. Agent-Specific Config Files (orchestrator.toml, worker.toml, messaging.toml)
6. Base Configuration File (framework.toml)
7. Built-in Defaults (compiled into binary)
```

#### 6.1.2 Configuration Merge Strategy

```rust
// Configuration merge strategy specification
pub struct ConfigurationMerger {
    layers: Vec<ConfigLayer>,
}

#[derive(Debug, Clone)]
pub enum ConfigLayer {
    Defaults,
    BaseConfig(PathBuf),
    AgentConfig(PathBuf),
    FeatureConfig(PathBuf),
    EnvironmentConfig(PathBuf),
    EnvironmentVariables(HashMap<String, String>),
    CommandLineArgs(HashMap<String, String>),
}

impl ConfigurationMerger {
    pub fn merge(&self) -> Result<FinalConfig, ConfigError> {
        let mut config = FinalConfig::default();
        
        // Apply layers in precedence order (lowest to highest)
        for layer in &self.layers {
            match layer {
                ConfigLayer::Defaults => {
                    // Built-in defaults are already in config
                }
                ConfigLayer::BaseConfig(path) => {
                    let base_config = load_toml_config(path)?;
                    config = config.merge_with(base_config);
                }
                ConfigLayer::AgentConfig(path) => {
                    let agent_config = load_toml_config(path)?;
                    config = config.merge_with(agent_config);
                }
                ConfigLayer::FeatureConfig(path) => {
                    let feature_config = load_toml_config(path)?;
                    config = config.merge_with(feature_config);
                }
                ConfigLayer::EnvironmentConfig(path) => {
                    let env_config = load_toml_config(path)?;
                    config = config.merge_with(env_config);
                }
                ConfigLayer::EnvironmentVariables(vars) => {
                    config = config.merge_with_env_vars(vars);
                }
                ConfigLayer::CommandLineArgs(args) => {
                    config = config.merge_with_cli_args(args);
                }
            }
        }
        
        // Validate final configuration
        config.validate()?;
        
        Ok(config)
    }
}
```

### 6.2 Configuration File Discovery

#### 6.2.1 File Discovery Strategy

```rust
// Configuration file discovery specification
pub struct ConfigDiscovery {
    search_paths: Vec<PathBuf>,
    environment_tier: String,
    agent_type: String,
    enabled_features: Vec<String>,
}

impl ConfigDiscovery {
    pub fn discover_configs(&self) -> Result<Vec<PathBuf>, ConfigError> {
        let mut configs = Vec::new();
        
        // 1. Base configuration (always loaded)
        if let Some(base) = self.find_file("framework.toml")? {
            configs.push(base);
        }
        
        // 2. Agent-specific configuration
        let agent_file = format!("{}.toml", self.agent_type);
        if let Some(agent) = self.find_file(&agent_file)? {
            configs.push(agent);
        }
        
        // 3. Feature-specific configurations
        for feature in &self.enabled_features {
            let feature_file = format!("{}.toml", feature);
            if let Some(feature_config) = self.find_file(&feature_file)? {
                configs.push(feature_config);
            }
        }
        
        // 4. Environment tier configuration
        let tier_file = format!("{}.toml", self.environment_tier);
        if let Some(tier) = self.find_file(&tier_file)? {
            configs.push(tier);
        }
        
        // 5. Local overrides (highest precedence file)
        if let Some(local) = self.find_file("local.toml")? {
            configs.push(local);
        }
        
        Ok(configs)
    }
    
    fn find_file(&self, filename: &str) -> Result<Option<PathBuf>, ConfigError> {
        for search_path in &self.search_paths {
            let full_path = search_path.join(filename);
            if full_path.exists() && full_path.is_file() {
                return Ok(Some(full_path));
            }
        }
        Ok(None)
    }
}

// Default search paths (in order)
pub fn default_search_paths() -> Vec<PathBuf> {
    vec![
        PathBuf::from("./config"),           // Local config directory
        PathBuf::from("./"),                 // Current directory
        PathBuf::from("/app/config"),        // Container config directory
        PathBuf::from("/etc/mister-smith"),  // System config directory
        dirs::config_dir().unwrap_or_default().join("mister-smith"), // User config
    ]
}
```

### 6.3 Dynamic Configuration Updates

#### 6.3.1 Hot Reload Support

```rust
// Hot reload configuration specification
use notify::{Watcher, RecursiveMode, Event};
use tokio::sync::broadcast;

pub struct ConfigWatcher {
    watcher: notify::RecommendedWatcher,
    config_paths: Vec<PathBuf>,
    update_sender: broadcast::Sender<ConfigUpdate>,
}

#[derive(Debug, Clone)]
pub struct ConfigUpdate {
    pub changed_files: Vec<PathBuf>,
    pub new_config: FinalConfig,
    pub validation_errors: Vec<ConfigError>,
}

impl ConfigWatcher {
    pub fn new(
        config_paths: Vec<PathBuf>,
        hot_reload_enabled: bool,
    ) -> Result<(Self, broadcast::Receiver<ConfigUpdate>), ConfigError> {
        let (update_sender, update_receiver) = broadcast::channel(100);
        
        if !hot_reload_enabled {
            // Return a watcher that never sends updates
            return Ok((
                Self {
                    watcher: notify::null_watcher()?,
                    config_paths,
                    update_sender,
                },
                update_receiver,
            ));
        }
        
        let watcher = notify::watcher(Duration::from_secs(1))?;
        
        // Watch all configuration files
        for path in &config_paths {
            if let Some(parent) = path.parent() {
                watcher.watch(parent, RecursiveMode::NonRecursive)?;
            }
        }
        
        Ok((
            Self {
                watcher,
                config_paths,
                update_sender,
            },
            update_receiver,
        ))
    }
    
    async fn handle_file_event(&mut self, event: Event) -> Result<(), ConfigError> {
        // Filter events to only configuration files
        let changed_configs: Vec<PathBuf> = event.paths.into_iter()
            .filter(|path| self.config_paths.contains(path))
            .filter(|path| path.extension().map_or(false, |ext| ext == "toml"))
            .collect();
        
        if changed_configs.is_empty() {
            return Ok(());
        }
        
        // Reload configuration
        let merger = ConfigurationMerger::new(self.config_paths.clone());
        match merger.merge() {
            Ok(new_config) => {
                let update = ConfigUpdate {
                    changed_files: changed_configs,
                    new_config,
                    validation_errors: Vec::new(),
                };
                
                if let Err(_) = self.update_sender.send(update) {
                    // No receivers, ignore
                }
            }
            Err(validation_errors) => {
                let update = ConfigUpdate {
                    changed_files: changed_configs,
                    new_config: FinalConfig::default(),
                    validation_errors: vec![validation_errors],
                };
                
                if let Err(_) = self.update_sender.send(update) {
                    // No receivers, ignore
                }
            }
        }
        
        Ok(())
    }
}
```

---

## 7. Configuration Examples

### 7.1 Complete Configuration Examples

#### 7.1.1 Development Environment Example

```toml
# config/framework.toml - Development environment
[framework]
name = "mister-smith-dev"
version = "0.1.0"
environment_tier = "tier_1"
log_level = "debug"
config_validation = true
hot_reload = true

[framework.runtime]
worker_threads = 2
max_blocking_threads = 32
thread_stack_size = "1MB"

[framework.features]
actors = true
tools = true
monitoring = false
supervision = true
config = true
# Security disabled for development
security = false
encryption = false
persistence = false

[framework.agent]
agent_type = "worker"
agent_tier = "minimal"
heartbeat_interval = 60
health_check_interval = 30
max_concurrent_tasks = 5

# config/worker.toml - Worker-specific development config
[worker]
bind_address = "127.0.0.1"
http_port = 8081
concurrency = 2
workspace_dir = "./workspace"
temp_dir = "/tmp"
max_task_duration = 60

[worker.resources]
max_memory_mb = 256
max_cpu_percent = 50
max_disk_mb = 512
cleanup_interval = 1800

[worker.claude_cli]
parallel_default = 2
parallel_max_agents = 5
parallel_cpu_budget = "2.0"
parallel_memory_budget = "1Gi"
hook_system_enabled = true
output_parsing_enabled = true
```

#### 7.1.2 Production Environment Example

```toml
# config/framework.toml - Production environment
[framework]
name = "mister-smith-prod"
version = "0.1.0"
environment_tier = "tier_3"
log_level = "warn"
config_validation = true
hot_reload = false

[framework.runtime]
worker_threads = "auto"
max_blocking_threads = 512
thread_stack_size = "2MB"

[framework.features]
actors = true
tools = true
monitoring = true
supervision = true
config = true
security = true
encryption = true
persistence = true
clustering = true
http_client = true
metrics = true
tracing = true

[framework.agent]
agent_type = "orchestrator"
agent_tier = "premium"
heartbeat_interval = 30
health_check_interval = 15
max_concurrent_tasks = 1000

# config/orchestrator.toml - Orchestrator production config
[orchestrator]
bind_address = "0.0.0.0"
http_port = 8080
grpc_port = 9090
max_connections = 10000
connection_timeout = 30

[orchestrator.coordination]
consensus_timeout = 3000
election_timeout = 100
heartbeat_interval = 25
max_entries_per_batch = 500
snapshot_threshold = 50000

[orchestrator.clustering]
enabled = true
cluster_size = 5
replication_factor = 3
sync_timeout = 500

# config/security.toml - Production security config
[security]
enabled = true
encryption_enabled = true
auth_required = true
audit_enabled = true
tls_enabled = true
mtls_enabled = true

[security.authentication]
method = "jwt"
jwt_secret = "${JWT_SECRET}"
jwt_expiry = 3600
refresh_enabled = true
refresh_expiry = 604800

[security.authorization]
rbac_enabled = true
default_role = "user"
admin_role = "admin"
role_inheritance = true
policy_file = "/app/config/rbac_policy.yaml"

# config/messaging.toml - Production messaging config
[messaging.nats]
url = "${NATS_URL}"
cluster_id = "mister-smith-prod-cluster"
client_id = "${AGENT_TYPE}-${AGENT_ID}"
max_reconnect = 10
reconnect_wait = 2
ping_interval = 60

[messaging.nats.tls]
enabled = true
cert_file = "/etc/tls/certs/client.crt"
key_file = "/etc/tls/private/client.key"
ca_file = "/etc/tls/certs/ca.crt"
verify = true

[messaging.jetstream]
enabled = true
max_memory = "1GB"
max_storage = "10GB"
replicas = 3
retention = "limits"
max_age = 604800
```

#### 7.1.3 Environment Variables Example

```bash
#!/bin/bash
# production.env - Production environment variables

# Core Framework Settings
export MISTER_SMITH_ENVIRONMENT_TIER=tier_3
export MISTER_SMITH_LOG_LEVEL=warn
export MISTER_SMITH_CONFIG_PATH=/app/config
export MISTER_SMITH_CONFIG_VALIDATION=true

# Agent Configuration
export MISTER_SMITH_AGENT_TYPE=orchestrator
export MISTER_SMITH_AGENT_TIER=premium
export MISTER_SMITH_AGENT_ID=$(hostname)-$(date +%s)

# Security Settings
export MISTER_SMITH_SECURITY_ENABLED=true
export MISTER_SMITH_SECURITY_ENCRYPTION_ENABLED=true
export MISTER_SMITH_SECURITY_TLS_ENABLED=true
export MISTER_SMITH_SECURITY_MTLS_ENABLED=true

# Authentication
export MISTER_SMITH_AUTH_METHOD=jwt
export MISTER_SMITH_AUTH_JWT_SECRET="$(cat /run/secrets/jwt_secret)"
export MISTER_SMITH_AUTH_JWT_EXPIRY=3600

# NATS Messaging
export MISTER_SMITH_NATS_URL="nats://nats-1:4222,nats-2:4222,nats-3:4222"
export MISTER_SMITH_NATS_CLUSTER_ID=mister-smith-prod-cluster
export MISTER_SMITH_NATS_TLS_ENABLED=true

# Database
export MISTER_SMITH_DB_ENABLED=true
export MISTER_SMITH_DB_DRIVER=postgresql
export MISTER_SMITH_DB_URL="$(cat /run/secrets/database_url)"
export MISTER_SMITH_DB_MAX_CONNECTIONS=20

# Redis
export MISTER_SMITH_REDIS_ENABLED=true
export MISTER_SMITH_REDIS_URL="$(cat /run/secrets/redis_url)"
export MISTER_SMITH_REDIS_POOL_SIZE=20

# Monitoring
export MISTER_SMITH_MONITORING_ENABLED=true
export MISTER_SMITH_METRICS_ENABLED=true
export MISTER_SMITH_METRICS_PROMETHEUS_ENABLED=true
export MISTER_SMITH_METRICS_PROMETHEUS_PORT=9090
export MISTER_SMITH_TRACING_ENABLED=true
export MISTER_SMITH_TRACING_JAEGER_ENABLED=true
export MISTER_SMITH_TRACING_JAEGER_ENDPOINT="http://jaeger:14268"

# Claude CLI Integration
export MISTER_SMITH_CLAUDE_PARALLEL_DEFAULT=8
export MISTER_SMITH_CLAUDE_PARALLEL_MAX_AGENTS=100
export MISTER_SMITH_CLAUDE_PARALLEL_CPU_BUDGET=8.0
export MISTER_SMITH_CLAUDE_PARALLEL_MEMORY_BUDGET=16Gi
```

---

## 8. Implementation Guidelines

### 8.1 Configuration Loading Implementation

#### 8.1.1 Configuration Manager Structure

```rust
// src/config/mod.rs - Configuration management implementation
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MisterSmithConfig {
    pub framework: FrameworkConfig,
    pub agent: AgentConfig,
    pub security: Option<SecurityConfig>,
    pub messaging: Option<MessagingConfig>,
    pub persistence: Option<PersistenceConfig>,
    pub monitoring: Option<MonitoringConfig>,
}

pub struct ConfigManager {
    config: Arc<RwLock<MisterSmithConfig>>,
    watcher: Option<ConfigWatcher>,
    discovery: ConfigDiscovery,
}

impl ConfigManager {
    pub async fn new(
        config_path: Option<PathBuf>,
        environment_tier: String,
        agent_type: String,
    ) -> Result<Self, ConfigError> {
        let discovery = ConfigDiscovery::new(config_path, environment_tier, agent_type);
        let config_files = discovery.discover_configs()?;
        
        // Load and merge all configuration files
        let merger = ConfigurationMerger::new(config_files);
        let final_config = merger.merge()?;
        
        // Set up file watcher for hot reload (if enabled)
        let watcher = if final_config.framework.hot_reload {
            Some(ConfigWatcher::new(merger.config_files(), true)?)
        } else {
            None
        };
        
        Ok(Self {
            config: Arc::new(RwLock::new(final_config)),
            watcher,
            discovery,
        })
    }
    
    pub async fn get_config(&self) -> MisterSmithConfig {
        self.config.read().await.clone()
    }
    
    pub async fn reload_config(&self) -> Result<(), ConfigError> {
        let config_files = self.discovery.discover_configs()?;
        let merger = ConfigurationMerger::new(config_files);
        let new_config = merger.merge()?;
        
        *self.config.write().await = new_config;
        Ok(())
    }
    
    pub async fn validate_config(&self) -> Result<(), Vec<ConfigError>> {
        let config = self.config.read().await;
        config.validate()
    }
}
```

### 8.2 Configuration Lifecycle Example

#### 8.2.1 Complete Configuration Lifecycle

```rust
// Configuration lifecycle implementation pattern
pub struct ConfigurationLifecycle {
    discovery: ConfigDiscovery,
    manager: ConfigManager,
    watcher: Option<ConfigWatcher>,
}

impl ConfigurationLifecycle {
    pub async fn initialize() -> Result<Self, ConfigError> {
        // Step 1: Discover configuration files
        let discovery = ConfigDiscovery::new(
            env::var("MISTER_SMITH_CONFIG_PATH").ok().map(PathBuf::from),
            env::var("MISTER_SMITH_ENVIRONMENT_TIER")
                .unwrap_or_else(|_| "tier_2".to_string()),
            env::var("MISTER_SMITH_AGENT_TYPE")
                .unwrap_or_else(|_| "worker".to_string()),
        );
        
        // Step 2: Load and merge configurations
        let config_files = discovery.discover_configs()?;
        let manager = ConfigManager::load_from_files(config_files).await?;
        
        // Step 3: Validate final configuration
        manager.validate_config().await?;
        
        // Step 4: Set up hot reload (if enabled)
        let watcher = if manager.hot_reload_enabled() {
            Some(ConfigWatcher::new(
                manager.get_config_files(),
                true,
            )?)
        } else {
            None
        };
        
        Ok(Self {
            discovery,
            manager,
            watcher,
        })
    }
    
    pub async fn deploy_with_patterns(&self) -> Result<(), ConfigError> {
        let config = self.manager.get_config().await;
        
        // Apply deployment patterns from configuration-deployment-specifications.md
        match config.framework.environment_tier.as_str() {
            "tier_1" => self.apply_experimental_deployment(&config).await?,
            "tier_2" => self.apply_validation_deployment(&config).await?,
            "tier_3" => self.apply_operational_deployment(&config).await?,
            _ => return Err(ConfigError::InvalidTier),
        }
        
        Ok(())
    }
    
    async fn apply_experimental_deployment(&self, config: &MisterSmithConfig) -> Result<(), ConfigError> {
        // Implement tier_1 deployment pattern
        // - Minimal resource allocation
        // - Limited feature set
        // - Development-focused configuration
        Ok(())
    }
    
    async fn apply_validation_deployment(&self, config: &MisterSmithConfig) -> Result<(), ConfigError> {
        // Implement tier_2 deployment pattern  
        // - Standard resource allocation
        // - Validation feature set
        // - Integration testing configuration
        Ok(())
    }
    
    async fn apply_operational_deployment(&self, config: &MisterSmithConfig) -> Result<(), ConfigError> {
        // Implement tier_3 deployment pattern
        // - Full resource allocation
        // - Complete feature set
        // - Production-grade configuration
        Ok(())
    }
}
```

#### 8.2.2 Deployment Integration Pattern

```rust
// Integration with deployment patterns from configuration-deployment-specifications.md
pub struct DeploymentIntegration {
    config_lifecycle: ConfigurationLifecycle,
    deployment_strategy: DeploymentStrategy,
}

impl DeploymentIntegration {
    pub async fn execute_progressive_deployment(&self) -> Result<(), DeploymentError> {
        let config = self.config_lifecycle.manager.get_config().await;
        
        // Apply progressive deployment pattern based on configuration
        match self.deployment_strategy {
            DeploymentStrategy::BlueGreen => {
                self.execute_blue_green_deployment(&config).await?;
            }
            DeploymentStrategy::Canary => {
                self.execute_canary_deployment(&config).await?;
            }
            DeploymentStrategy::Rolling => {
                self.execute_rolling_deployment(&config).await?;
            }
        }
        
        Ok(())
    }
}
```

### 8.3 Integration with Framework Components

#### 8.3.1 Agent Initialization with Configuration

```rust
// src/agent/mod.rs - Agent initialization with configuration
use crate::config::{ConfigManager, MisterSmithConfig};

pub struct Agent {
    config_manager: ConfigManager,
    runtime_handle: tokio::runtime::Handle,
    agent_type: AgentType,
}

impl Agent {
    pub async fn new(
        config_path: Option<PathBuf>,
        agent_type: AgentType,
    ) -> Result<Self, AgentError> {
        // Initialize configuration manager
        let config_manager = ConfigManager::new(
            config_path,
            std::env::var("MISTER_SMITH_ENVIRONMENT_TIER")
                .unwrap_or_else(|_| "tier_2".to_string()),
            agent_type.to_string(),
        ).await?;
        
        // Validate configuration
        config_manager.validate_config().await?;
        
        // Get current configuration
        let config = config_manager.get_config().await;
        
        // Initialize runtime based on configuration
        let runtime = Self::build_runtime(&config.framework.runtime)?;
        
        Ok(Self {
            config_manager,
            runtime_handle: runtime.handle().clone(),
            agent_type,
        })
    }
    
    fn build_runtime(
        runtime_config: &RuntimeConfig,
    ) -> Result<tokio::runtime::Runtime, AgentError> {
        let mut builder = tokio::runtime::Builder::new_multi_thread();
        
        match runtime_config.worker_threads.as_str() {
            "auto" => {
                builder.enable_all();
            }
            threads => {
                let thread_count = threads.parse::<usize>()
                    .map_err(|_| AgentError::InvalidConfiguration(
                        "Invalid worker_threads value".to_string()
                    ))?;
                builder.worker_threads(thread_count);
            }
        }
        
        builder
            .max_blocking_threads(runtime_config.max_blocking_threads)
            .thread_stack_size(runtime_config.thread_stack_size_bytes())
            .enable_io()
            .enable_time()
            .build()
            .map_err(AgentError::RuntimeInitialization)
    }
    
    pub async fn start(&mut self) -> Result<(), AgentError> {
        let config = self.config_manager.get_config().await;
        
        // Start components based on configuration
        if config.messaging.is_some() {
            self.start_messaging(&config).await?;
        }
        
        if config.security.as_ref().map_or(false, |s| s.enabled) {
            self.start_security(&config).await?;
        }
        
        if config.monitoring.as_ref().map_or(false, |m| m.enabled) {
            self.start_monitoring(&config).await?;
        }
        
        // Start agent-specific components
        match self.agent_type {
            AgentType::Orchestrator => self.start_orchestrator(&config).await?,
            AgentType::Worker => self.start_worker(&config).await?,
            AgentType::Messaging => self.start_messaging_agent(&config).await?,
        }
        
        Ok(())
    }
}
```

---

## 9. Best Practices and Recommendations

### 9.1 Configuration Security Best Practices

1. **Secret Management**
   - Never store secrets in configuration files
   - Use environment variables for secret references
   - Implement proper secret rotation
   - Use dedicated secret management systems in production

2. **Validation**
   - Validate all configuration at startup
   - Provide clear error messages for invalid configuration
   - Implement type-safe configuration parsing
   - Use schema validation for complex configurations

3. **Environment Separation**
   - Use different configuration files for different environments
   - Implement tier-based feature restrictions
   - Use environment variables for environment-specific settings
   - Never share configuration between environments

### 9.2 Performance Considerations

1. **Configuration Loading**
   - Cache parsed configuration in memory
   - Use lazy loading for optional features
   - Minimize configuration file I/O
   - Implement efficient configuration merging

2. **Hot Reload**
   - Only enable hot reload in development environments
   - Use efficient file watching
   - Implement graceful configuration updates
   - Validate configuration before applying changes

### 9.3 Operational Guidelines

1. **Monitoring**
   - Monitor configuration changes
   - Log configuration validation errors
   - Track configuration reload events
   - Implement configuration drift detection

2. **Documentation**
   - Document all configuration options
   - Provide examples for common scenarios
   - Maintain up-to-date default values
   - Include validation rules in documentation

---

## 10. Infrastructure-as-Code Integration

**Integration with Deployment Patterns**: This section implements the Infrastructure-as-Code patterns defined in [Configuration & Deployment Specifications](configuration-deployment-specifications.md) Section 5.

### 10.1 Terraform Integration

```hcl
# terraform/modules/mister-smith-config/main.tf
resource "kubernetes_config_map" "mister_smith_base" {
  metadata {
    name      = "mister-smith-base-config"
    namespace = var.namespace
  }

  data = {
    "framework.toml" = templatefile("${path.module}/templates/framework.toml.tpl", {
      environment_tier = var.environment_tier
      log_level       = var.log_level
      feature_flags   = var.feature_flags
    })
    
    "orchestrator.toml" = templatefile("${path.module}/templates/orchestrator.toml.tpl", {
      http_port = var.orchestrator_http_port
      grpc_port = var.orchestrator_grpc_port
      max_connections = var.max_connections
    })
  }
}

resource "kubernetes_secret" "mister_smith_secrets" {
  metadata {
    name      = "mister-smith-secrets"
    namespace = var.namespace
  }

  data = {
    database_url = var.database_url
    nats_url     = var.nats_url
    redis_url    = var.redis_url
  }
}

# Output configuration values for other modules
output "config_map_name" {
  value = kubernetes_config_map.mister_smith_base.metadata[0].name
}

output "secret_name" {
  value = kubernetes_secret.mister_smith_secrets.metadata[0].name
}
```

### 10.2 CloudFormation Integration

```yaml
# cloudformation/mister-smith-config.yaml
AWSTemplateFormatVersion: '2010-09-09'
Description: 'Mister Smith Framework Configuration Stack'

Parameters:
  EnvironmentTier:
    Type: String
    AllowedValues:
      - tier_1
      - tier_2
      - tier_3
    Default: tier_2
    
  ClusterName:
    Type: String
    Description: EKS cluster name

Resources:
  ConfigParameterStore:
    Type: AWS::SSM::Parameter
    Properties:
      Name: /mister-smith/${EnvironmentTier}/framework-config
      Type: String
      Value: !Sub |
        [framework]
        environment_tier = "${EnvironmentTier}"
        log_level = "${LogLevel}"
        
        [framework.features]
        security = ${SecurityEnabled}
        encryption = ${EncryptionEnabled}
        clustering = ${ClusteringEnabled}
  
  SecretsManager:
    Type: AWS::SecretsManager::Secret
    Properties:
      Name: !Sub mister-smith-${EnvironmentTier}-secrets
      SecretString: !Sub |
        {
          "database_url": "${DatabaseUrl}",
          "nats_url": "${NatsUrl}",
          "redis_url": "${RedisUrl}",
          "api_keys": {
            "claude": "${ClaudeApiKey}"
          }
        }
      
  ConfigMapManifest:
    Type: AWS::SSM::Parameter
    Properties:
      Name: !Sub /mister-smith/${EnvironmentTier}/k8s-configmap
      Type: String
      Value: !Sub |
        apiVersion: v1
        kind: ConfigMap
        metadata:
          name: mister-smith-config
          namespace: mister-smith-${EnvironmentTier}
        data:
          framework.toml: |
            ${ConfigParameterStore}
```

### 10.3 Helm Values Integration

```yaml
# helm/mister-smith/values.yaml
configuration:
  # Base configuration settings
  framework:
    environmentTier: tier_2
    logLevel: info
    configValidation: true
    hotReload: false
    
  # Feature flags by tier
  features:
    tier_1:
      - actors
      - tools
      - monitoring
    tier_2:
      - actors
      - tools
      - monitoring
      - security
      - persistence
      - metrics
    tier_3:
      - actors
      - tools
      - monitoring
      - security
      - persistence
      - metrics
      - encryption
      - clustering
      - tracing
      
  # Agent-specific configurations
  agents:
    orchestrator:
      httpPort: 8080
      grpcPort: 9090
      replicas: 3
      resources:
        requests:
          memory: "512Mi"
          cpu: "500m"
        limits:
          memory: "1Gi"
          cpu: "1000m"
          
    worker:
      replicas: 5
      maxConcurrentTasks: 100
      resources:
        requests:
          memory: "256Mi"
          cpu: "250m"
        limits:
          memory: "512Mi"
          cpu: "500m"

# ConfigMap template
configMapTemplate: |
  {{- range $key, $value := .Values.configuration.framework }}
  {{ $key }} = {{ $value | quote }}
  {{- end }}
```

### 10.4 Dynamic Scaling Configuration

```yaml
# autoscaling-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: mister-smith-autoscaling-config
data:
  scaling-policies.yaml: |
    policies:
      worker_scaling:
        metrics:
          - type: cpu
            target: 70
          - type: memory
            target: 80
          - type: custom
            metric: pending_tasks
            target: 50
        behavior:
          scaleUp:
            stabilizationWindow: 60s
            policies:
            - type: Percent
              value: 100
              periodSeconds: 15
          scaleDown:
            stabilizationWindow: 300s
            policies:
            - type: Percent
              value: 25
              periodSeconds: 60
              
      orchestrator_scaling:
        metrics:
          - type: custom
            metric: coordination_queue_depth
            target: 100
        minReplicas: 3
        maxReplicas: 10
```

### 10.5 Troubleshooting Common Configuration Issues

#### Issue: Configuration Not Loading

```bash
# Check configuration file syntax
mister-smith config validate --file framework.toml

# Verify environment variables
mister-smith config env --show-sources

# Debug configuration merge order
mister-smith config debug --show-merge-order
```

#### Issue: Feature Not Available

```yaml
# Verify feature dependencies
troubleshooting:
  encryption_not_working:
    check:
      - security feature is enabled
      - environment tier is tier_3
      - encryption keys are configured
    solution: |
      Ensure all dependencies are met:
      framework.features.security = true
      framework.environment_tier = "tier_3"
      MISTER_SMITH_ENCRYPTION_KEY is set
```

#### Issue: Secret Access Denied

```bash
# Verify secret permissions
kubectl auth can-i get secrets -n mister-smith-system

# Check secret references
mister-smith config secrets --verify

# Test secret rotation
mister-smith config rotate-secrets --dry-run
```

---

This comprehensive configuration management specification provides the foundation for a flexible, secure, and maintainable configuration system
that supports the Mister Smith AI Agent Framework's multi-tier architecture and feature-based modularity while maintaining strong security
and operational practices. The Infrastructure-as-Code integration ensures seamless deployment across various cloud platforms and orchestration systems.

**Implementation Integration**: Use these configuration specifications with the deployment patterns defined in [Configuration & Deployment Specifications](configuration-deployment-specifications.md) for complete system deployment.

**Related Specifications**:

- [Container Bootstrap Patterns](configuration-deployment-specifications.md#54-container-bootstrap-sequence-pattern) - Container initialization using these configurations
- [Resource Management Patterns](configuration-deployment-specifications.md#6-scaling-and-resource-management-patterns) - Scaling configurations implementation
- [Secret Management Patterns](configuration-deployment-specifications.md#2-secret-management-patterns) - Secret handling in deployment contexts
