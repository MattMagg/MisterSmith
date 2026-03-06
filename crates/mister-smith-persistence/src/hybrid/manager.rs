//! HybridStateManager — coordinates reads/writes between KV and SQL stores.
//!
//! For [`KvPrimary`](super::router::StorageLayer::KvPrimary) data types:
//! - **Writes** go to KV first, then the key is marked dirty for async flush to SQL.
//! - **Reads** try KV first; on miss, fall back to SQL and lazily hydrate the KV entry.
//!
//! ## Dirty-Key Flush (US2)
//!
//! Dirty keys are flushed to SQL when either:
//! 1. The dirty count exceeds the configured threshold, OR
//! 2. The time since the oldest dirty key exceeds the flush deadline.
//!
//! The flush deadline is clamped to `min(configured_deadline, kv_ttl - safety_margin)`
//! to prevent data loss from TTL expiration before flush.
//!
//! ## Graceful Degradation (US2)
//!
//! - **KV unreachable**: Falls back to SQL with warning logs.
//! - **SQL unreachable**: Continues KV writes with dirty tracking, retries flushes.
//! - **Both unreachable**: Returns `PersistenceError::ConnectionFailed`.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::Value;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::config::FlushConfig;
use crate::error::PersistenceError;
use crate::kv::state::StateManager;

/// Shared mutable state for dirty-key tracking.
struct DirtyState {
    /// Keys that have been written to KV but not yet flushed to SQL.
    keys: HashSet<String>,
    /// When the first key was marked dirty (since last flush).
    oldest_dirty_at: Option<Instant>,
}

impl DirtyState {
    fn new() -> Self {
        Self {
            keys: HashSet::new(),
            oldest_dirty_at: None,
        }
    }

    fn mark_dirty(&mut self, key: String) {
        if self.keys.is_empty() {
            self.oldest_dirty_at = Some(Instant::now());
        }
        self.keys.insert(key);
    }

    /// Drain all dirty keys, returning them and the saved oldest timestamp.
    fn drain(&mut self) -> (Vec<String>, Option<Instant>) {
        let saved_oldest = self.oldest_dirty_at.take();
        let keys = self.keys.drain().collect();
        (keys, saved_oldest)
    }

    /// Re-mark a key as dirty, preserving a saved timestamp if available.
    fn re_mark(&mut self, key: String, saved_oldest: Option<Instant>) {
        self.keys.insert(key);
        if self.oldest_dirty_at.is_none() {
            self.oldest_dirty_at = saved_oldest.or_else(|| Some(Instant::now()));
        }
    }

    fn count(&self) -> usize {
        self.keys.len()
    }

    fn time_since_oldest(&self) -> Option<Duration> {
        self.oldest_dirty_at.map(|t| t.elapsed())
    }
}

/// Coordinates reads and writes between the KV and SQL stores for agent state.
///
/// The basic flow for `KvPrimary` data:
/// - `write_state()`: write to KV, mark key dirty, auto-flush if threshold exceeded
/// - `read_state()`: read from KV; on miss, fall back to SQL and hydrate KV
/// - `flush_to_sql()`: batch-upsert all dirty keys to PostgreSQL
/// - `start_background_flush()`: spawns a periodic check task
pub struct HybridStateManager {
    /// KV state manager for fast reads/writes.
    kv: StateManager,

    /// PostgreSQL connection pool for durable storage.
    #[cfg(feature = "sqlx")]
    pool: sqlx::PgPool,

    /// Dirty-key tracking state.
    dirty: Arc<Mutex<DirtyState>>,

    /// Flush configuration.
    flush_config: FlushConfig,

    /// Effective flush deadline (clamped by KV TTL - safety margin).
    effective_deadline: Duration,
}

impl HybridStateManager {
    /// Create a new hybrid manager from KV and SQL backends.
    ///
    /// The `kv_ttl` parameter is used to compute the effective flush deadline:
    /// `min(flush_config.deadline_secs, kv_ttl - safety_margin)`.
    #[cfg(feature = "sqlx")]
    pub fn new(
        kv: StateManager,
        pool: sqlx::PgPool,
        flush_config: FlushConfig,
        kv_ttl: Duration,
    ) -> Self {
        let effective_deadline = compute_effective_deadline(&flush_config, kv_ttl);
        Self {
            kv,
            pool,
            dirty: Arc::new(Mutex::new(DirtyState::new())),
            flush_config,
            effective_deadline,
        }
    }

    /// Create a new hybrid manager with KV only (no SQL backend).
    #[cfg(not(feature = "sqlx"))]
    pub fn new(kv: StateManager, flush_config: FlushConfig, kv_ttl: Duration) -> Self {
        let effective_deadline = compute_effective_deadline(&flush_config, kv_ttl);
        Self {
            kv,
            dirty: Arc::new(Mutex::new(DirtyState::new())),
            flush_config,
            effective_deadline,
        }
    }

    /// Write agent state to the KV layer and mark the key dirty for flush.
    ///
    /// If the KV write fails and SQL is available, writes directly to SQL
    /// as a graceful degradation path.
    ///
    /// Returns the KV revision number on success, or 0 if fallback to SQL was used.
    pub async fn write_state(
        &self,
        agent_id: Uuid,
        key: &str,
        value: &Value,
    ) -> Result<u64, PersistenceError> {
        let kv_key = format!("{agent_id}:{key}");

        // Try KV first
        match self.kv.save(&kv_key, value).await {
            Ok(revision) => {
                debug!(agent_id = %agent_id, key = %key, revision = revision, "State written to KV");

                // Mark key dirty for future flush
                let should_flush = {
                    let mut dirty = self.dirty.lock().await;
                    dirty.mark_dirty(kv_key);
                    dirty.count() >= self.flush_config.threshold
                };

                // Auto-flush if threshold exceeded
                #[cfg(feature = "sqlx")]
                if should_flush {
                    debug!("Dirty count threshold reached, triggering flush");
                    if let Err(e) = self.flush_to_sql().await {
                        warn!(error = %e, "Auto-flush on threshold failed");
                    }
                }

                Ok(revision)
            }
            Err(kv_err) => {
                // Graceful degradation: KV unreachable, fall back to SQL
                warn!(
                    agent_id = %agent_id, key = %key, error = %kv_err,
                    "KV write failed, falling back to SQL"
                );

                #[cfg(feature = "sqlx")]
                {
                    crate::postgres::queries::upsert_state(
                        &self.pool,
                        agent_id,
                        key,
                        value.clone(),
                        None,
                    )
                    .await?;
                    info!(agent_id = %agent_id, key = %key, "State written directly to SQL (KV degraded)");
                    Ok(0) // No KV revision when writing directly to SQL
                }

                #[cfg(not(feature = "sqlx"))]
                Err(kv_err)
            }
        }
    }

    /// Read agent state, trying KV first with SQL fallback.
    ///
    /// If both KV and SQL are unreachable, returns `ConnectionFailed`.
    pub async fn read_state(
        &self,
        agent_id: Uuid,
        key: &str,
    ) -> Result<Option<Value>, PersistenceError> {
        let kv_key = format!("{agent_id}:{key}");
        let mut kv_failed = false;

        // Try KV first
        match self.kv.get::<Value>(&kv_key).await {
            Ok(Some(value)) => {
                debug!(agent_id = %agent_id, key = %key, "State read from KV (cache hit)");
                return Ok(Some(value));
            }
            Ok(None) => {
                debug!(agent_id = %agent_id, key = %key, "KV miss, trying SQL fallback");
            }
            Err(e) => {
                warn!(
                    agent_id = %agent_id, key = %key, error = %e,
                    "KV read failed, trying SQL fallback"
                );
                kv_failed = true;
            }
        }

        // SQL fallback
        #[cfg(feature = "sqlx")]
        {
            match crate::postgres::queries::get_state(&self.pool, agent_id, key).await {
                Ok(Some(row)) => {
                    debug!(agent_id = %agent_id, key = %key, "State read from SQL (fallback)");

                    // Lazy hydration: write back to KV for next read (only if KV is up)
                    if !kv_failed {
                        if let Err(e) = self.kv.save(&kv_key, &row.state_value).await {
                            warn!(
                                agent_id = %agent_id, key = %key, error = %e,
                                "Failed to hydrate KV from SQL fallback"
                            );
                        }
                    }

                    Ok(Some(row.state_value))
                }
                Ok(None) => Ok(None),
                Err(sql_err) => {
                    if kv_failed {
                        // Both backends failed
                        Err(PersistenceError::ConnectionFailed(format!(
                            "Both KV and SQL backends failed for agent {agent_id} key {key}: {sql_err}"
                        )))
                    } else {
                        warn!(agent_id = %agent_id, key = %key, error = %sql_err, "SQL fallback failed");
                        Err(sql_err)
                    }
                }
            }
        }

        #[cfg(not(feature = "sqlx"))]
        {
            if kv_failed {
                Err(PersistenceError::ConnectionFailed(
                    "KV read failed and no SQL backend available".to_string(),
                ))
            } else {
                Ok(None)
            }
        }
    }

    /// Flush all dirty keys from KV to PostgreSQL.
    ///
    /// Reads each dirty key from KV and upserts it into the SQL agent state table
    /// via [`queries::upsert_state`]. Returns the number of keys flushed.
    ///
    /// Keys that fail to read from KV are re-marked dirty for retry.
    /// On SQL error, all remaining unprocessed keys are re-marked with the
    /// original dirty timestamp preserved.
    #[cfg(feature = "sqlx")]
    pub async fn flush_to_sql(&self) -> Result<usize, PersistenceError> {
        let (keys, saved_oldest) = {
            let mut dirty = self.dirty.lock().await;
            dirty.drain()
        };

        if keys.is_empty() {
            return Ok(0);
        }

        debug!(count = keys.len(), "Flushing dirty keys to SQL");

        let mut flushed = 0usize;

        for (i, kv_key) in keys.iter().enumerate() {
            let (agent_id, state_key) = match parse_kv_key(kv_key) {
                Some(parts) => parts,
                None => {
                    warn!(key = %kv_key, "Skipping dirty key with invalid format");
                    continue;
                }
            };

            match self.kv.get::<Value>(kv_key).await {
                Ok(Some(value)) => {
                    match crate::postgres::queries::upsert_state(
                        &self.pool, agent_id, state_key, value, None,
                    )
                    .await
                    {
                        Ok(_) => {
                            flushed += 1;
                        }
                        Err(e) => {
                            warn!(key = %kv_key, error = %e, "SQL upsert failed during flush");
                            // Re-mark this key and all remaining unprocessed keys
                            let mut dirty = self.dirty.lock().await;
                            dirty.re_mark(kv_key.clone(), saved_oldest);
                            for remaining_key in &keys[i + 1..] {
                                dirty.re_mark(remaining_key.clone(), saved_oldest);
                            }
                            return Err(e);
                        }
                    }
                }
                Ok(None) => {
                    debug!(key = %kv_key, "Dirty key no longer in KV (TTL expired?), skipping");
                }
                Err(e) => {
                    warn!(key = %kv_key, error = %e, "Failed to read dirty key from KV during flush");
                    self.dirty
                        .lock()
                        .await
                        .re_mark(kv_key.clone(), saved_oldest);
                }
            }
        }

        debug!(flushed = flushed, "Flush to SQL complete");
        Ok(flushed)
    }

    /// Start a background task that periodically checks if a flush is needed.
    ///
    /// The task checks every second whether:
    /// - The dirty count exceeds the threshold, OR
    /// - The time since the oldest dirty key exceeds the effective deadline.
    ///
    /// Returns a handle that can be used to abort the task.
    #[cfg(feature = "sqlx")]
    pub fn start_background_flush(self: &Arc<Self>) -> tokio::task::JoinHandle<()> {
        let manager = Arc::clone(self);
        let check_interval = Duration::from_secs(1);

        tokio::spawn(async move {
            let mut retries = 0u32;
            loop {
                tokio::time::sleep(check_interval).await;

                let should_flush = {
                    let dirty = manager.dirty.lock().await;
                    let count_exceeded = dirty.count() >= manager.flush_config.threshold;
                    let deadline_exceeded = dirty
                        .time_since_oldest()
                        .is_some_and(|elapsed| elapsed >= manager.effective_deadline);
                    count_exceeded || deadline_exceeded
                };

                if should_flush {
                    match manager.flush_to_sql().await {
                        Ok(n) if n > 0 => {
                            debug!(flushed = n, "Background flush completed");
                            retries = 0;
                        }
                        Ok(_) => {
                            retries = 0;
                        }
                        Err(e) => {
                            retries += 1;
                            warn!(
                                error = %e, retries = retries,
                                max_retries = manager.flush_config.max_flush_retries,
                                "Background flush failed"
                            );
                            if retries >= manager.flush_config.max_flush_retries {
                                warn!(
                                    "Max flush retries exceeded, pausing background flush for 30s"
                                );
                                tokio::time::sleep(Duration::from_secs(30)).await;
                                retries = 0;
                            }
                        }
                    }
                }
            }
        })
    }

    /// Get the current number of dirty keys pending flush.
    pub async fn dirty_count(&self) -> usize {
        self.dirty.lock().await.count()
    }

    /// Check if the deadline-based flush trigger has been exceeded.
    pub async fn deadline_exceeded(&self) -> bool {
        self.dirty
            .lock()
            .await
            .time_since_oldest()
            .is_some_and(|elapsed| elapsed >= self.effective_deadline)
    }

    /// Get the effective flush deadline.
    pub fn effective_deadline(&self) -> Duration {
        self.effective_deadline
    }

    /// Get a reference to the underlying KV state manager.
    pub fn kv(&self) -> &StateManager {
        &self.kv
    }

    /// Get a reference to the underlying PostgreSQL pool.
    #[cfg(feature = "sqlx")]
    pub fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}

/// Compute the effective flush deadline clamped by KV TTL.
///
/// `effective = min(configured_deadline, kv_ttl - safety_margin)`
///
/// If `kv_ttl - safety_margin` would be zero or negative, uses 10 seconds
/// as a minimum to prevent immediate continuous flushing.
fn compute_effective_deadline(config: &FlushConfig, kv_ttl: Duration) -> Duration {
    let configured = Duration::from_secs(config.deadline_secs);
    let ttl_based = kv_ttl.saturating_sub(Duration::from_secs(config.safety_margin_secs));
    let ttl_based = if ttl_based.is_zero() {
        Duration::from_secs(10) // minimum floor
    } else {
        ttl_based
    };
    configured.min(ttl_based)
}

/// Parse a KV key in `{agent_id}:{state_key}` format.
fn parse_kv_key(key: &str) -> Option<(Uuid, &str)> {
    let colon_pos = key.find(':')?;
    let agent_id = Uuid::parse_str(&key[..colon_pos]).ok()?;
    let state_key = &key[colon_pos + 1..];
    Some((agent_id, state_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_kv_key_valid() {
        let id = Uuid::new_v4();
        let key = format!("{id}:my_state_key");
        let (parsed_id, parsed_key) = parse_kv_key(&key).unwrap();
        assert_eq!(parsed_id, id);
        assert_eq!(parsed_key, "my_state_key");
    }

    #[test]
    fn parse_kv_key_nested() {
        let id = Uuid::new_v4();
        let key = format!("{id}:nested:key:with:colons");
        let (parsed_id, parsed_key) = parse_kv_key(&key).unwrap();
        assert_eq!(parsed_id, id);
        assert_eq!(parsed_key, "nested:key:with:colons");
    }

    #[test]
    fn parse_kv_key_invalid_uuid() {
        assert!(parse_kv_key("not-a-uuid:key").is_none());
    }

    #[test]
    fn parse_kv_key_no_colon() {
        assert!(parse_kv_key("nocolonhere").is_none());
    }

    #[test]
    fn parse_kv_key_empty_state_key() {
        let id = Uuid::new_v4();
        let key = format!("{id}:");
        let (parsed_id, parsed_key) = parse_kv_key(&key).unwrap();
        assert_eq!(parsed_id, id);
        assert_eq!(parsed_key, "");
    }

    #[test]
    fn effective_deadline_uses_configured_when_smaller() {
        let config = FlushConfig {
            threshold: 50,
            deadline_secs: 10,
            safety_margin_secs: 60,
            max_flush_retries: 3,
        };
        let kv_ttl = Duration::from_secs(1800); // 30 min
        let effective = compute_effective_deadline(&config, kv_ttl);
        // configured=10s, ttl_based=1800-60=1740s → min=10s
        assert_eq!(effective, Duration::from_secs(10));
    }

    #[test]
    fn effective_deadline_uses_ttl_based_when_smaller() {
        let config = FlushConfig {
            threshold: 50,
            deadline_secs: 600,
            safety_margin_secs: 60,
            max_flush_retries: 3,
        };
        let kv_ttl = Duration::from_secs(120); // 2 min
        let effective = compute_effective_deadline(&config, kv_ttl);
        // configured=600s, ttl_based=120-60=60s → min=60s
        assert_eq!(effective, Duration::from_secs(60));
    }

    #[test]
    fn effective_deadline_floor_when_ttl_tiny() {
        let config = FlushConfig {
            threshold: 50,
            deadline_secs: 600,
            safety_margin_secs: 60,
            max_flush_retries: 3,
        };
        let kv_ttl = Duration::from_secs(30); // less than safety margin
        let effective = compute_effective_deadline(&config, kv_ttl);
        // ttl_based = 30-60 = saturating_sub → 0 → floor 10s
        // configured = 600s → min(600, 10) = 10s
        assert_eq!(effective, Duration::from_secs(10));
    }

    #[test]
    fn dirty_state_tracking() {
        let mut state = DirtyState::new();
        assert_eq!(state.count(), 0);
        assert!(state.time_since_oldest().is_none());

        state.mark_dirty("key1".to_string());
        assert_eq!(state.count(), 1);
        assert!(state.time_since_oldest().is_some());

        state.mark_dirty("key2".to_string());
        assert_eq!(state.count(), 2);

        let (drained, saved_oldest) = state.drain();
        assert_eq!(drained.len(), 2);
        assert!(saved_oldest.is_some());
        assert_eq!(state.count(), 0);
        assert!(state.time_since_oldest().is_none());
    }

    #[test]
    fn dirty_state_re_mark_preserves_timestamp() {
        let mut state = DirtyState::new();
        let saved = Some(Instant::now() - Duration::from_secs(30));
        state.re_mark("retry_key".to_string(), saved);
        assert_eq!(state.count(), 1);
        // Timestamp should be the saved one, not a fresh Instant::now()
        let elapsed = state.time_since_oldest().unwrap();
        assert!(elapsed >= Duration::from_secs(29));
    }

    #[test]
    fn dirty_state_re_mark_no_saved_timestamp() {
        let mut state = DirtyState::new();
        state.re_mark("retry_key".to_string(), None);
        assert_eq!(state.count(), 1);
        assert!(state.time_since_oldest().is_some());
    }
}
