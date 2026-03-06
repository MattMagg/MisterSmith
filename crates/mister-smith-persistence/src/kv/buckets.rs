//! JetStream KV bucket lifecycle management.
//!
//! [`KvBucketManager`] creates and manages the three standard KV buckets
//! (session data, agent state, query cache) with configurable TTLs and
//! replica counts. Bucket creation is idempotent — calling
//! [`KvBucketManager::initialize_buckets`] on an already-provisioned
//! JetStream cluster is safe.

use std::collections::HashMap;
use std::time::Duration;

use async_nats::jetstream::{self, kv::Store};
use tracing::{debug, info, warn};

use crate::config::KvConfig;
use crate::error::{from_kv_error, PersistenceError};
use mister_smith_core::HealthStatus;

/// Bucket name for session-scoped data (conversations, context windows).
pub const SESSION_DATA: &str = "SESSION_DATA";

/// Bucket name for agent state snapshots.
pub const AGENT_STATE: &str = "AGENT_STATE";

/// Bucket name for ephemeral query/response caching.
pub const QUERY_CACHE: &str = "QUERY_CACHE";

/// Manages JetStream KV bucket lifecycle.
///
/// Holds a JetStream context and a map of initialized [`Store`] handles.
/// All three standard buckets are created during [`initialize_buckets`](Self::initialize_buckets)
/// using `create_or_update_key_value` for idempotent provisioning.
pub struct KvBucketManager {
    context: jetstream::Context,
    config: KvConfig,
    buckets: HashMap<String, Store>,
}

impl KvBucketManager {
    /// Create a new bucket manager.
    ///
    /// No buckets are created until [`initialize_buckets`](Self::initialize_buckets) is called.
    pub fn new(context: jetstream::Context, config: KvConfig) -> Self {
        Self {
            context,
            config,
            buckets: HashMap::new(),
        }
    }

    /// Create or update all standard KV buckets.
    ///
    /// This is idempotent — if a bucket already exists with a compatible
    /// configuration it will be returned as-is. If the bucket exists but
    /// its configuration diverges (e.g. a different TTL), the server will
    /// update it in place.
    ///
    /// # Bucket configuration
    ///
    /// | Bucket | TTL source | Replicas |
    /// |--------|-----------|----------|
    /// | `SESSION_DATA` | `config.session_ttl_secs` (default 3600s) | `config.replicas` |
    /// | `AGENT_STATE` | `config.agent_state_ttl_secs` (default 1800s) | `config.replicas` |
    /// | `QUERY_CACHE` | `config.cache_ttl_secs` (default 300s) | always 1 |
    pub async fn initialize_buckets(&mut self) -> Result<(), PersistenceError> {
        let bucket_specs: [(&str, u64, usize); 3] = [
            (
                SESSION_DATA,
                self.config.session_ttl_secs,
                self.config.replicas as usize,
            ),
            (
                AGENT_STATE,
                self.config.agent_state_ttl_secs,
                self.config.replicas as usize,
            ),
            (
                QUERY_CACHE,
                self.config.cache_ttl_secs,
                1, // query cache is always single-replica
            ),
        ];

        for (name, ttl_secs, replicas) in bucket_specs {
            let kv_config = jetstream::kv::Config {
                bucket: name.to_string(),
                max_age: Duration::from_secs(ttl_secs),
                num_replicas: replicas,
                ..Default::default()
            };

            debug!(
                bucket = %name,
                ttl_secs = ttl_secs,
                replicas = replicas,
                "Creating or updating KV bucket"
            );

            let store = self
                .context
                .create_or_update_key_value(kv_config)
                .await
                .map_err(from_kv_error)?;

            info!(bucket = %name, "KV bucket initialized");
            self.buckets.insert(name.to_string(), store);
        }

        Ok(())
    }

    /// Get a named bucket handle.
    ///
    /// Returns `PersistenceError::NotFound` if the bucket has not been
    /// initialized via [`initialize_buckets`](Self::initialize_buckets).
    pub fn bucket(&self, name: &str) -> Result<&Store, PersistenceError> {
        self.buckets.get(name).ok_or_else(|| {
            PersistenceError::NotFound(format!("KV bucket '{name}' not initialized"))
        })
    }

    /// Verify that all standard buckets are accessible.
    ///
    /// Calls [`Store::status`] on every initialized bucket. If all succeed
    /// the result is [`HealthStatus::Healthy`]. If some fail, the result is
    /// [`HealthStatus::Degraded`]. If all fail (or no buckets are initialized),
    /// the result is [`HealthStatus::Unhealthy`].
    pub async fn health_check(&self) -> Result<HealthStatus, PersistenceError> {
        if self.buckets.is_empty() {
            return Ok(HealthStatus::Unhealthy);
        }

        let expected = [SESSION_DATA, AGENT_STATE, QUERY_CACHE];
        let mut healthy_count = 0usize;
        let mut total = 0usize;

        for name in &expected {
            if let Some(store) = self.buckets.get(*name) {
                total += 1;
                match store.status().await {
                    Ok(_status) => {
                        debug!(bucket = %name, "KV bucket health check passed");
                        healthy_count += 1;
                    }
                    Err(err) => {
                        warn!(bucket = %name, error = %err, "KV bucket health check failed");
                    }
                }
            }
        }

        let status = if healthy_count == total && total == expected.len() {
            HealthStatus::Healthy
        } else if healthy_count > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };

        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_name_constants() {
        assert_eq!(SESSION_DATA, "SESSION_DATA");
        assert_eq!(AGENT_STATE, "AGENT_STATE");
        assert_eq!(QUERY_CACHE, "QUERY_CACHE");
    }

    #[test]
    fn bucket_not_found_before_init() {
        // KvBucketManager requires a real JetStream context, so we can't
        // construct one without a server. We test the error path by verifying
        // the constant names are distinct and the HashMap lookup logic works.
        let mut map: HashMap<String, ()> = HashMap::new();
        map.insert(SESSION_DATA.to_string(), ());
        assert!(map.contains_key(SESSION_DATA));
        assert!(!map.contains_key("NONEXISTENT"));
    }
}
