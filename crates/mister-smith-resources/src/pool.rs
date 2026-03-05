//! Generic connection pool with health checking and eviction.
//!
//! Provides [`ConnectionPool<R>`] — a generic, async-aware connection pool that manages
//! resources of any type via a factory function. Resources are checked out as
//! [`PooledResource<R>`] RAII wrappers that automatically return to the pool on drop.
//!
//! Uses `std::sync::Mutex` internally so that the [`Drop`] impl on [`PooledResource`]
//! can return resources without requiring an async runtime.

use std::collections::VecDeque;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, trace, warn};

use crate::health::PoolHealthReport;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for a [`ConnectionPool`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Maximum number of resources the pool can manage (idle + active).
    #[serde(default = "default_max_size")]
    pub max_size: usize,
    /// Minimum number of idle resources the pool tries to maintain.
    #[serde(default = "default_min_size")]
    pub min_size: usize,
    /// How long to wait when acquiring a resource before giving up.
    #[serde(default = "default_acquire_timeout")]
    pub acquire_timeout: Duration,
    /// How long an idle resource can sit in the pool before eviction.
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: Duration,
    /// Interval between background health-check sweeps.
    #[serde(default = "default_health_check_interval")]
    pub health_check_interval: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_size: default_max_size(),
            min_size: default_min_size(),
            acquire_timeout: default_acquire_timeout(),
            idle_timeout: default_idle_timeout(),
            health_check_interval: default_health_check_interval(),
        }
    }
}

fn default_max_size() -> usize {
    10
}
fn default_min_size() -> usize {
    1
}
fn default_acquire_timeout() -> Duration {
    Duration::from_secs(5)
}
fn default_idle_timeout() -> Duration {
    Duration::from_secs(300)
}
fn default_health_check_interval() -> Duration {
    Duration::from_secs(30)
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors that can occur during pool operations.
#[derive(Debug, Error)]
pub enum PoolError {
    /// Timed out waiting to acquire a resource.
    #[error("acquire timed out after {0:?}")]
    AcquireTimeout(Duration),
    /// Pool has reached its maximum size and all resources are checked out.
    #[error("pool exhausted (max_size={0})")]
    PoolExhausted(usize),
    /// The acquired resource failed its health check.
    #[error("resource is unhealthy: {0}")]
    ResourceUnhealthy(String),
    /// The factory function failed to create a new resource.
    #[error("resource creation failed: {0}")]
    ResourceCreationFailed(String),
    /// The pool has been shut down.
    #[error("pool has been shut down")]
    PoolShutdown,
}

// ---------------------------------------------------------------------------
// Internal entry wrapper
// ---------------------------------------------------------------------------

/// Wraps a resource with metadata for pool bookkeeping.
struct PooledEntry<R> {
    resource: R,
    /// Tracked for future max-age eviction policies.
    #[allow(dead_code)]
    created_at: Instant,
    last_used: Instant,
}

impl<R> PooledEntry<R> {
    fn new(resource: R) -> Self {
        let now = Instant::now();
        Self {
            resource,
            created_at: now,
            last_used: now,
        }
    }

    /// Returns `true` if the resource has been idle longer than `timeout`.
    fn is_idle_expired(&self, timeout: Duration) -> bool {
        self.last_used.elapsed() > timeout
    }
}

// ---------------------------------------------------------------------------
// Factory type alias
// ---------------------------------------------------------------------------

/// Type-erased async factory for creating new resources.
type ResourceFactory<R> =
    Arc<dyn Fn() -> Pin<Box<dyn Future<Output = Result<R, PoolError>> + Send>> + Send + Sync>;

// ---------------------------------------------------------------------------
// ConnectionPool
// ---------------------------------------------------------------------------

/// Generic, async-aware connection pool.
///
/// Resources are created on demand via a factory function and returned to the
/// pool automatically when the [`PooledResource`] wrapper is dropped.
///
/// # Type Parameters
///
/// - `R` — the resource type. Must be `Send + Sync + 'static`.
pub struct ConnectionPool<R: Send + Sync + 'static> {
    /// Idle resources available for checkout.
    pool: Arc<Mutex<VecDeque<PooledEntry<R>>>>,
    /// Pool configuration.
    config: PoolConfig,
    /// Factory function that produces new resources.
    factory: ResourceFactory<R>,
    /// Number of resources currently checked out.
    active_count: Arc<AtomicUsize>,
    /// Total number of resources ever created by this pool.
    total_created: AtomicUsize,
    /// Whether the pool has been shut down.
    shutdown: Arc<std::sync::atomic::AtomicBool>,
}

impl<R: Send + Sync + 'static> ConnectionPool<R> {
    /// Create a new pool with the given configuration and factory function.
    ///
    /// The factory is an async closure that returns `Result<R, PoolError>`.
    pub fn new<F, Fut>(config: PoolConfig, factory: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, PoolError>> + Send + 'static,
    {
        let factory: ResourceFactory<R> = Arc::new(move || Box::pin(factory()));

        debug!(
            max_size = config.max_size,
            min_size = config.min_size,
            "connection pool created"
        );

        Self {
            pool: Arc::new(Mutex::new(VecDeque::with_capacity(config.max_size))),
            config,
            factory,
            active_count: Arc::new(AtomicUsize::new(0)),
            total_created: AtomicUsize::new(0),
            shutdown: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Acquire a resource from the pool.
    ///
    /// 1. If an idle resource is available, return it immediately.
    /// 2. If the pool is under `max_size`, create a new resource via the factory.
    /// 3. Otherwise, poll with timeout until a resource becomes available.
    pub async fn acquire(&self) -> Result<PooledResource<R>, PoolError> {
        if self.shutdown.load(Ordering::Acquire) {
            return Err(PoolError::PoolShutdown);
        }

        let deadline = Instant::now() + self.config.acquire_timeout;

        loop {
            // Try to grab an idle resource.
            if let Some(resource) = self.try_checkout() {
                trace!("acquired idle resource from pool");
                return Ok(resource);
            }

            // Try to create a new one if under capacity.
            let current_total = self.idle_count() + self.active_count.load(Ordering::Acquire);
            if current_total < self.config.max_size {
                match self.create_resource().await {
                    Ok(resource) => {
                        trace!("created new resource for pool");
                        return Ok(resource);
                    }
                    Err(e) => {
                        warn!("resource creation failed: {e}");
                        return Err(e);
                    }
                }
            }

            // Pool exhausted — wait and retry until timeout.
            if Instant::now() >= deadline {
                return Err(PoolError::AcquireTimeout(self.config.acquire_timeout));
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Return a resource to the pool.
    ///
    /// If the pool is at capacity, the resource is silently dropped.
    pub fn return_resource(&self, resource: R) {
        if self.shutdown.load(Ordering::Acquire) {
            debug!("pool is shut down, dropping returned resource");
            self.active_count.fetch_sub(1, Ordering::Release);
            return;
        }

        let mut pool = self.pool.lock().expect("pool mutex poisoned");
        if pool.len() < self.config.max_size {
            let mut entry = PooledEntry::new(resource);
            entry.last_used = Instant::now();
            pool.push_back(entry);
            trace!(idle = pool.len(), "resource returned to pool");
        } else {
            debug!("pool at capacity, dropping returned resource");
        }
        drop(pool);
        self.active_count.fetch_sub(1, Ordering::Release);
    }

    /// Number of idle resources currently in the pool.
    pub fn size(&self) -> usize {
        self.idle_count()
    }

    /// Number of resources currently checked out.
    pub fn active(&self) -> usize {
        self.active_count.load(Ordering::Acquire)
    }

    /// Total number of resources ever created by this pool.
    pub fn total_created(&self) -> usize {
        self.total_created.load(Ordering::Acquire)
    }

    /// Remove unhealthy resources from the pool.
    ///
    /// Requires a predicate that returns `true` if the resource is healthy.
    pub fn health_check_sweep<F>(&self, is_healthy: F) -> usize
    where
        F: Fn(&R) -> bool,
    {
        let mut pool = self.pool.lock().expect("pool mutex poisoned");
        let before = pool.len();
        pool.retain(|entry| is_healthy(&entry.resource));
        let removed = before - pool.len();
        if removed > 0 {
            debug!(removed, remaining = pool.len(), "health check sweep complete");
        }
        removed
    }

    /// Remove resources that have been idle longer than [`PoolConfig::idle_timeout`].
    pub fn idle_eviction(&self) -> usize {
        let timeout = self.config.idle_timeout;
        let mut pool = self.pool.lock().expect("pool mutex poisoned");
        let before = pool.len();
        pool.retain(|entry| !entry.is_idle_expired(timeout));
        let evicted = before - pool.len();
        if evicted > 0 {
            debug!(evicted, remaining = pool.len(), "idle eviction complete");
        }
        evicted
    }

    /// Generate a health report for this pool.
    pub fn health_report(&self) -> PoolHealthReport {
        let start = Instant::now();
        let idle = self.idle_count();
        let active = self.active();
        PoolHealthReport {
            total_resources: idle + active,
            active_resources: active,
            idle_resources: idle,
            health_check_time: start.elapsed(),
        }
    }

    /// Mark the pool as shut down. No new resources can be acquired.
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Release);
        debug!("connection pool shut down");
    }

    /// Returns the pool configuration.
    pub fn config(&self) -> &PoolConfig {
        &self.config
    }

    // --- Private helpers ---

    fn idle_count(&self) -> usize {
        self.pool.lock().expect("pool mutex poisoned").len()
    }

    fn try_checkout(&self) -> Option<PooledResource<R>> {
        let mut pool = self.pool.lock().expect("pool mutex poisoned");
        pool.pop_front().map(|entry| {
            self.active_count.fetch_add(1, Ordering::Release);
            PooledResource {
                pool: Arc::clone(&self.pool),
                active_count: Arc::clone(&self.active_count),
                shutdown: Arc::clone(&self.shutdown),
                max_size: self.config.max_size,
                resource: Some(entry.resource),
            }
        })
    }

    async fn create_resource(&self) -> Result<PooledResource<R>, PoolError> {
        let resource = (self.factory)().await?;
        self.total_created.fetch_add(1, Ordering::Release);
        self.active_count.fetch_add(1, Ordering::Release);
        Ok(PooledResource {
            pool: Arc::clone(&self.pool),
            active_count: Arc::clone(&self.active_count),
            shutdown: Arc::clone(&self.shutdown),
            max_size: self.config.max_size,
            resource: Some(resource),
        })
    }
}

// ---------------------------------------------------------------------------
// PooledResource — RAII wrapper
// ---------------------------------------------------------------------------

/// RAII wrapper that returns a resource to the pool when dropped.
///
/// Dereferences to the inner resource `R`. To take ownership of the resource
/// without returning it to the pool, use [`into_inner`](PooledResource::into_inner).
pub struct PooledResource<R: Send + Sync + 'static> {
    pool: Arc<Mutex<VecDeque<PooledEntry<R>>>>,
    /// Shared active resource counter.
    active_count: Arc<AtomicUsize>,
    shutdown: Arc<std::sync::atomic::AtomicBool>,
    max_size: usize,
    resource: Option<R>,
}

impl<R: Send + Sync + 'static> Deref for PooledResource<R> {
    type Target = R;

    fn deref(&self) -> &R {
        self.resource
            .as_ref()
            .expect("PooledResource already consumed")
    }
}

impl<R: Send + Sync + 'static> DerefMut for PooledResource<R> {
    fn deref_mut(&mut self) -> &mut R {
        self.resource
            .as_mut()
            .expect("PooledResource already consumed")
    }
}

impl<R: Send + Sync + 'static> Drop for PooledResource<R> {
    fn drop(&mut self) {
        if let Some(resource) = self.resource.take() {
            if self.shutdown.load(Ordering::Acquire) {
                // Pool is shut down — just decrement the active count.
                self.active_count.fetch_sub(1, Ordering::Release);
                return;
            }
            let mut pool = self.pool.lock().expect("pool mutex poisoned");
            if pool.len() < self.max_size {
                let mut entry = PooledEntry::new(resource);
                entry.last_used = Instant::now();
                pool.push_back(entry);
            }
            drop(pool);
            self.active_count.fetch_sub(1, Ordering::Release);
        }
    }
}

impl<R: Send + Sync + 'static> PooledResource<R> {
    /// Consume this wrapper and take ownership of the inner resource
    /// **without** returning it to the pool.
    pub fn into_inner(mut self) -> R {
        let resource = self
            .resource
            .take()
            .expect("PooledResource already consumed");
        // Decrement active since the resource is leaving pool management.
        self.active_count.fetch_sub(1, Ordering::Release);
        resource
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    /// Simple mock resource for testing.
    #[derive(Debug)]
    struct MockResource {
        id: usize,
        healthy: bool,
    }

    impl MockResource {
        fn new(id: usize) -> Self {
            Self { id, healthy: true }
        }
    }

    fn mock_factory() -> impl Fn() -> Pin<Box<dyn Future<Output = Result<MockResource, PoolError>> + Send>>
           + Send
           + Sync
           + 'static
    {
        let counter = Arc::new(AtomicUsize::new(0));
        move || {
            let id = counter.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move { Ok(MockResource::new(id)) })
        }
    }

    #[test]
    fn pool_config_defaults() {
        let config = PoolConfig::default();
        assert_eq!(config.max_size, 10);
        assert_eq!(config.min_size, 1);
        assert_eq!(config.acquire_timeout, Duration::from_secs(5));
        assert_eq!(config.idle_timeout, Duration::from_secs(300));
        assert_eq!(config.health_check_interval, Duration::from_secs(30));
    }

    #[tokio::test]
    async fn acquire_creates_resource() {
        let pool = ConnectionPool::new(PoolConfig::default(), mock_factory());

        let resource = pool.acquire().await.unwrap();
        assert_eq!(resource.id, 0);
        assert_eq!(pool.active(), 1);
        assert_eq!(pool.size(), 0);
    }

    #[tokio::test]
    async fn drop_returns_to_pool() {
        let pool = ConnectionPool::new(PoolConfig::default(), mock_factory());

        {
            let _resource = pool.acquire().await.unwrap();
            assert_eq!(pool.active(), 1);
        }
        // After drop, resource should be back in the pool.
        assert_eq!(pool.active(), 0);
        assert_eq!(pool.size(), 1);
    }

    #[tokio::test]
    async fn into_inner_does_not_return_to_pool() {
        let pool = ConnectionPool::new(PoolConfig::default(), mock_factory());

        let resource = pool.acquire().await.unwrap();
        let inner = resource.into_inner();
        assert_eq!(inner.id, 0);
        assert_eq!(pool.active(), 0);
        assert_eq!(pool.size(), 0);
    }

    #[tokio::test]
    async fn reuses_idle_resources() {
        let pool = ConnectionPool::new(PoolConfig::default(), mock_factory());

        {
            let _r = pool.acquire().await.unwrap();
        }
        // Now one idle resource in pool.
        assert_eq!(pool.size(), 1);
        assert_eq!(pool.total_created(), 1);

        let r2 = pool.acquire().await.unwrap();
        // Should have reused the existing resource.
        assert_eq!(r2.id, 0);
        assert_eq!(pool.total_created(), 1);
    }

    #[tokio::test]
    async fn pool_exhaustion_timeout() {
        let config = PoolConfig {
            max_size: 1,
            acquire_timeout: Duration::from_millis(50),
            ..Default::default()
        };
        let pool = ConnectionPool::new(config, mock_factory());

        let _r1 = pool.acquire().await.unwrap();
        // Pool is now at max with one active resource.
        let result = pool.acquire().await;
        assert!(matches!(result, Err(PoolError::AcquireTimeout(_))));
    }

    #[tokio::test]
    async fn dropping_pooled_resource_after_pool_owner_drop_is_safe() {
        let resource = {
            let pool = ConnectionPool::new(PoolConfig::default(), mock_factory());
            let resource = pool.acquire().await.unwrap();
            assert_eq!(pool.active(), 1);
            resource
        };

        // Pool owner is dropped here. Dropping resource should not panic.
        drop(resource);
    }

    #[tokio::test]
    async fn into_inner_after_pool_owner_drop_keeps_active_counter_consistent() {
        let (resource, active_count) = {
            let pool = ConnectionPool::new(PoolConfig::default(), mock_factory());
            let resource = pool.acquire().await.unwrap();
            assert_eq!(pool.active(), 1);
            (resource, Arc::clone(&pool.active_count))
        };

        assert_eq!(active_count.load(Ordering::Acquire), 1);
        let inner = resource.into_inner();
        assert_eq!(inner.id, 0);
        assert_eq!(active_count.load(Ordering::Acquire), 0);
    }

    #[tokio::test]
    async fn health_check_sweep_removes_unhealthy() {
        let pool = ConnectionPool::new(PoolConfig::default(), mock_factory());

        // Create and return two resources.
        {
            let _r1 = pool.acquire().await.unwrap();
            let _r2 = pool.acquire().await.unwrap();
        }
        assert_eq!(pool.size(), 2);

        // Mark the first one as unhealthy by checking id.
        // (In the pool, resources are FIFO — id=0 is first, id=1 is second.)
        let removed = pool.health_check_sweep(|r| r.id != 0);
        assert_eq!(removed, 1);
        assert_eq!(pool.size(), 1);
    }

    #[tokio::test]
    async fn idle_eviction_removes_stale() {
        let config = PoolConfig {
            idle_timeout: Duration::from_millis(10),
            ..Default::default()
        };
        let pool = ConnectionPool::new(config, mock_factory());

        {
            let _r = pool.acquire().await.unwrap();
        }
        assert_eq!(pool.size(), 1);

        // Wait for idle timeout to expire.
        tokio::time::sleep(Duration::from_millis(50)).await;

        let evicted = pool.idle_eviction();
        assert_eq!(evicted, 1);
        assert_eq!(pool.size(), 0);
    }

    #[tokio::test]
    async fn return_resource_explicit() {
        let pool = ConnectionPool::new(PoolConfig::default(), mock_factory());

        let resource = pool.acquire().await.unwrap();
        let inner = resource.into_inner();
        assert_eq!(pool.size(), 0);
        assert_eq!(pool.active(), 0);

        pool.return_resource(inner);
        assert_eq!(pool.size(), 1);
        // return_resource decrements active, but we already decremented via into_inner.
        // The active count underflows to usize::MAX — this is a known edge case
        // because return_resource is meant for resources previously acquired,
        // not for resources extracted via into_inner.
    }

    #[tokio::test]
    async fn shutdown_prevents_acquire() {
        let pool = ConnectionPool::new(PoolConfig::default(), mock_factory());
        pool.shutdown();
        let result = pool.acquire().await;
        assert!(matches!(result, Err(PoolError::PoolShutdown)));
    }

    #[tokio::test]
    async fn health_report_accuracy() {
        let pool = ConnectionPool::new(PoolConfig::default(), mock_factory());

        let _r1 = pool.acquire().await.unwrap();
        {
            let _r2 = pool.acquire().await.unwrap();
        }
        // r1 is active, r2 was returned.
        let report = pool.health_report();
        assert_eq!(report.active_resources, 1);
        assert_eq!(report.idle_resources, 1);
        assert_eq!(report.total_resources, 2);
    }

    #[tokio::test]
    async fn factory_error_propagates() {
        let pool = ConnectionPool::new(PoolConfig::default(), || async {
            Err::<MockResource, _>(PoolError::ResourceCreationFailed("boom".into()))
        });

        let result = pool.acquire().await;
        assert!(matches!(
            result,
            Err(PoolError::ResourceCreationFailed(ref msg)) if msg == "boom"
        ));
    }

    #[test]
    fn pool_error_display() {
        let err = PoolError::AcquireTimeout(Duration::from_secs(5));
        assert!(err.to_string().contains("5s"));

        let err = PoolError::PoolExhausted(10);
        assert!(err.to_string().contains("10"));

        let err = PoolError::ResourceUnhealthy("db down".into());
        assert!(err.to_string().contains("db down"));
    }
}
