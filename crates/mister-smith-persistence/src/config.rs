//! Persistence configuration types.
//!
//! Defines typed configuration for PostgreSQL, JetStream KV, flush behavior,
//! and checkpoint intervals. All structs use `#[serde(default)]` for partial
//! TOML deserialization.

use serde::{Deserialize, Serialize};

/// Top-level persistence configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersistenceConfig {
    /// Master switch — when `false`, persistence is disabled.
    #[serde(default)]
    pub enabled: bool,

    /// PostgreSQL connection configuration.
    #[serde(default)]
    pub postgres: PostgresConfig,

    /// JetStream KV configuration.
    #[serde(default)]
    pub kv: KvConfig,

    /// Flush behavior for dirty-key tracking.
    #[serde(default)]
    pub flush: FlushConfig,

    /// Checkpoint configuration.
    #[serde(default)]
    pub checkpoint: CheckpointConfig,
}

/// PostgreSQL connection pool configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig {
    /// Database connection URL.
    #[serde(default)]
    pub url: Option<String>,

    /// Maximum number of connections in the pool.
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Minimum number of connections to keep alive.
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    /// Connection timeout in seconds.
    #[serde(default = "default_connect_timeout_secs")]
    pub connect_timeout_secs: u64,

    /// Idle connection timeout in seconds.
    #[serde(default = "default_idle_timeout_secs")]
    pub idle_timeout_secs: u64,
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            url: None,
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            connect_timeout_secs: default_connect_timeout_secs(),
            idle_timeout_secs: default_idle_timeout_secs(),
        }
    }
}

/// JetStream KV bucket configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvConfig {
    /// Whether KV storage is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Session data bucket TTL in seconds.
    #[serde(default = "default_session_ttl")]
    pub session_ttl_secs: u64,

    /// Agent state bucket TTL in seconds.
    #[serde(default = "default_agent_state_ttl")]
    pub agent_state_ttl_secs: u64,

    /// Query cache bucket TTL in seconds.
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_secs: u64,

    /// Number of replicas for session and agent state buckets.
    #[serde(default = "default_replicas")]
    pub replicas: u32,
}

impl Default for KvConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            session_ttl_secs: default_session_ttl(),
            agent_state_ttl_secs: default_agent_state_ttl(),
            cache_ttl_secs: default_cache_ttl(),
            replicas: default_replicas(),
        }
    }
}

/// Flush configuration for dirty-key tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlushConfig {
    /// Number of dirty keys before triggering a flush.
    #[serde(default = "default_flush_threshold")]
    pub threshold: usize,

    /// Maximum time (seconds) before flushing the oldest dirty key.
    #[serde(default = "default_flush_deadline_secs")]
    pub deadline_secs: u64,

    /// Safety margin (seconds) subtracted from KV TTL for flush deadline.
    #[serde(default = "default_safety_margin_secs")]
    pub safety_margin_secs: u64,

    /// Maximum number of flush retries on failure.
    #[serde(default = "default_max_flush_retries")]
    pub max_flush_retries: u32,
}

impl Default for FlushConfig {
    fn default() -> Self {
        Self {
            threshold: default_flush_threshold(),
            deadline_secs: default_flush_deadline_secs(),
            safety_margin_secs: default_safety_margin_secs(),
            max_flush_retries: default_max_flush_retries(),
        }
    }
}

/// Checkpoint configuration for periodic state snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointConfig {
    /// Interval in seconds between automatic checkpoints.
    #[serde(default = "default_checkpoint_interval")]
    pub interval_secs: u64,

    /// Whether automatic checkpointing is enabled.
    #[serde(default)]
    pub enabled: bool,
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            interval_secs: default_checkpoint_interval(),
            enabled: false,
        }
    }
}

// Default value functions

fn default_true() -> bool {
    true
}

fn default_max_connections() -> u32 {
    10
}

fn default_min_connections() -> u32 {
    2
}

fn default_connect_timeout_secs() -> u64 {
    30
}

fn default_idle_timeout_secs() -> u64 {
    600
}

fn default_session_ttl() -> u64 {
    3600 // 1 hour
}

fn default_agent_state_ttl() -> u64 {
    1800 // 30 minutes
}

fn default_cache_ttl() -> u64 {
    300 // 5 minutes
}

fn default_replicas() -> u32 {
    1 // default to 1 for dev; production should use 3
}

fn default_flush_threshold() -> usize {
    50
}

fn default_flush_deadline_secs() -> u64 {
    60
}

fn default_safety_margin_secs() -> u64 {
    300 // 5 minutes
}

fn default_max_flush_retries() -> u32 {
    3
}

fn default_checkpoint_interval() -> u64 {
    300 // 5 minutes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_persistence_config() {
        let config = PersistenceConfig::default();
        assert!(!config.enabled);
        assert!(config.postgres.url.is_none());
        assert_eq!(config.postgres.max_connections, 10);
        assert!(config.kv.enabled);
        assert_eq!(config.kv.session_ttl_secs, 3600);
        assert_eq!(config.flush.threshold, 50);
        assert_eq!(config.flush.max_flush_retries, 3);
        assert_eq!(config.checkpoint.interval_secs, 300);
        assert!(!config.checkpoint.enabled);
    }

    #[test]
    fn deserialize_partial_config() {
        let toml_str = r#"
            enabled = true
            [postgres]
            url = "postgres://localhost/test"
            max_connections = 20
        "#;
        let config: PersistenceConfig = toml::from_str(toml_str).unwrap();
        assert!(config.enabled);
        assert_eq!(
            config.postgres.url.as_deref(),
            Some("postgres://localhost/test")
        );
        assert_eq!(config.postgres.max_connections, 20);
        // Defaults still apply for unset fields
        assert_eq!(config.postgres.min_connections, 2);
        assert!(config.kv.enabled);
    }
}
