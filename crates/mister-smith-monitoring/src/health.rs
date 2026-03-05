//! Health check trait and HealthMonitor.
//!
//! [`HealthCheck`] is the extension point — downstream crates implement it for
//! each component they want monitored. [`HealthMonitor`] runs periodic checks,
//! caches results, and optionally publishes status-change events via the core
//! [`EventPublisher`] trait.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use mister_smith_core::{EventPublisher, SystemEvent};

use crate::types::{ComponentId, HealthStatus, Status};

// ---------------------------------------------------------------------------
// HealthCheck trait
// ---------------------------------------------------------------------------

/// Async health check interface.
///
/// Implementors define how to probe a particular component and how often.
#[async_trait]
pub trait HealthCheck: Send + Sync + 'static {
    /// Perform the health check, returning the observed status.
    async fn check(&self) -> Result<Status, Box<dyn std::error::Error + Send + Sync>>;

    /// Return the component ID this check belongs to.
    fn component_id(&self) -> ComponentId;

    /// Suggested interval between checks (default: 30 s).
    fn check_interval(&self) -> Duration {
        Duration::from_secs(30)
    }
}

// ---------------------------------------------------------------------------
// HealthMonitor
// ---------------------------------------------------------------------------

/// Periodically runs registered [`HealthCheck`]s and caches the results.
///
/// When a component's status changes between check cycles, an event is
/// published through the optional [`EventPublisher`].
pub struct HealthMonitor {
    /// Interval between full check cycles.
    check_interval: Duration,
    /// Registered health checks.
    health_checks: RwLock<Vec<Arc<dyn HealthCheck>>>,
    /// Cached results keyed by component ID.
    status_cache: RwLock<HashMap<ComponentId, HealthStatus>>,
    /// Optional event publisher for status-change notifications.
    event_publisher: Option<Arc<dyn EventPublisher>>,
}

impl HealthMonitor {
    /// Create a new `HealthMonitor` that checks at the given interval.
    pub fn new(check_interval: Duration) -> Self {
        Self {
            check_interval,
            health_checks: RwLock::new(Vec::new()),
            status_cache: RwLock::new(HashMap::new()),
            event_publisher: None,
        }
    }

    /// Attach an event publisher for status-change notifications.
    pub fn with_event_publisher(mut self, publisher: Arc<dyn EventPublisher>) -> Self {
        self.event_publisher = Some(publisher);
        self
    }

    /// Register a new health check.
    pub async fn register_check(&self, check: Arc<dyn HealthCheck>) {
        let mut checks = self.health_checks.write().await;
        checks.push(check);
    }

    /// Run the monitor loop until `shutdown` is set to `true`.
    ///
    /// Each iteration performs all registered checks, then sleeps for
    /// `check_interval`.
    pub async fn run(&self, shutdown: Arc<AtomicBool>) {
        info!(
            interval_ms = self.check_interval.as_millis() as u64,
            "Health monitor started"
        );

        while !shutdown.load(Ordering::SeqCst) {
            self.perform_health_checks().await;

            // Use tokio::select! so we wake promptly on shutdown.
            tokio::select! {
                _ = tokio::time::sleep(self.check_interval) => {}
                _ = async {
                    while !shutdown.load(Ordering::SeqCst) {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                } => {
                    break;
                }
            }
        }

        info!("Health monitor stopped");
    }

    /// Execute every registered health check, update the cache, and publish
    /// events for any status changes.
    pub async fn perform_health_checks(&self) {
        let checks = {
            let checks = self.health_checks.read().await;
            checks.clone()
        };

        for check in checks {
            let component_id = check.component_id();
            let result = check.check().await;

            let (new_status, message) = match result {
                Ok(status) => (status, None),
                Err(e) => {
                    let msg = format!("Health check failed: {e}");
                    error!(component = %component_id, error = %e, "Health check error");
                    (Status::Unhealthy, Some(msg))
                }
            };

            let mut hs = HealthStatus::new(component_id.clone(), new_status);
            if let Some(msg) = message {
                hs = hs.with_message(msg);
            }

            // Detect status change while holding the cache lock as briefly as possible.
            let status_change = {
                let mut cache = self.status_cache.write().await;
                let previous_status = cache.get(&component_id).map(|h| h.status);
                let status_changed = previous_status != Some(new_status);
                cache.insert(component_id.clone(), hs);

                if status_changed {
                    Some((previous_status, new_status))
                } else {
                    None
                }
            };

            // Publish only after releasing status_cache lock.
            if let Some((previous_status, current_status)) = status_change {
                debug!(
                    component = %component_id,
                    previous = ?previous_status,
                    current = ?current_status,
                    "Component status changed"
                );

                if let Some(ref publisher) = self.event_publisher {
                    let event = SystemEvent {
                        event_type: "health.status_changed".to_string(),
                        payload: serde_json::json!({
                            "component_id": component_id.as_str(),
                            "previous_status": previous_status.map(|s| format!("{s:?}")),
                            "new_status": format!("{current_status:?}"),
                        }),
                    };
                    if let Err(e) = publisher.publish(event).await {
                        warn!(
                            component = %component_id,
                            error = %e,
                            "Failed to publish health status change event"
                        );
                    }
                }
            }
        }
    }

    /// Get the cached health status for a specific component.
    pub async fn get_status(&self, component_id: &ComponentId) -> Option<HealthStatus> {
        let cache = self.status_cache.read().await;
        cache.get(component_id).cloned()
    }

    /// Get all cached health statuses.
    pub async fn get_all_statuses(&self) -> HashMap<ComponentId, HealthStatus> {
        let cache = self.status_cache.read().await;
        cache.clone()
    }

    /// Returns `true` if no component is currently `Unhealthy`.
    pub async fn is_system_healthy(&self) -> bool {
        let cache = self.status_cache.read().await;
        !cache.values().any(|hs| hs.status == Status::Unhealthy)
    }
}

// ---------------------------------------------------------------------------
// RuntimeHealthCheck
// ---------------------------------------------------------------------------

/// Health check that verifies the Tokio runtime is responsive.
///
/// Spawns a `yield_now()` task on the given runtime handle and expects it to
/// complete within the configured timeout.
pub struct RuntimeHealthCheck {
    handle: tokio::runtime::Handle,
    timeout: Duration,
}

impl RuntimeHealthCheck {
    /// Create a new `RuntimeHealthCheck`.
    ///
    /// * `handle` — runtime handle to probe.
    /// * `timeout` — maximum time to wait for the spawned task.
    pub fn new(handle: tokio::runtime::Handle, timeout: Duration) -> Self {
        Self { handle, timeout }
    }
}

#[async_trait]
impl HealthCheck for RuntimeHealthCheck {
    async fn check(&self) -> Result<Status, Box<dyn std::error::Error + Send + Sync>> {
        let task = self.handle.spawn(async {
            tokio::task::yield_now().await;
        });

        match tokio::time::timeout(self.timeout, task).await {
            Ok(Ok(())) => Ok(Status::Healthy),
            Ok(Err(e)) => {
                warn!(error = %e, "Runtime health check task panicked");
                Ok(Status::Unhealthy)
            }
            Err(_) => {
                warn!(
                    timeout_ms = self.timeout.as_millis() as u64,
                    "Runtime health check timed out"
                );
                Ok(Status::Degraded)
            }
        }
    }

    fn component_id(&self) -> ComponentId {
        ComponentId::new("tokio-runtime")
    }

    fn check_interval(&self) -> Duration {
        Duration::from_secs(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mister_smith_core::EventError;
    use std::sync::OnceLock;
    use tokio::sync::{Barrier, Notify};

    /// A trivial health check that always returns a fixed status.
    struct FixedHealthCheck {
        id: ComponentId,
        status: Status,
    }

    impl FixedHealthCheck {
        fn new(id: &str, status: Status) -> Self {
            Self {
                id: ComponentId::new(id),
                status,
            }
        }
    }

    #[async_trait]
    impl HealthCheck for FixedHealthCheck {
        async fn check(&self) -> Result<Status, Box<dyn std::error::Error + Send + Sync>> {
            Ok(self.status)
        }

        fn component_id(&self) -> ComponentId {
            self.id.clone()
        }
    }

    /// A health check that always errors.
    struct FailingHealthCheck;

    #[async_trait]
    impl HealthCheck for FailingHealthCheck {
        async fn check(&self) -> Result<Status, Box<dyn std::error::Error + Send + Sync>> {
            Err("connection refused".into())
        }

        fn component_id(&self) -> ComponentId {
            ComponentId::new("failing")
        }
    }

    struct CallbackEventPublisher {
        callback: Arc<
            dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
                + Send
                + Sync,
        >,
    }

    impl CallbackEventPublisher {
        fn new<F, Fut>(callback: F) -> Self
        where
            F: Fn() -> Fut + Send + Sync + 'static,
            Fut: std::future::Future<Output = ()> + Send + 'static,
        {
            Self {
                callback: Arc::new(move || Box::pin(callback())),
            }
        }
    }

    #[async_trait]
    impl EventPublisher for CallbackEventPublisher {
        async fn publish(&self, _event: SystemEvent) -> Result<(), EventError> {
            (self.callback)().await;
            Ok(())
        }
    }

    struct BlockingHealthCheck {
        id: ComponentId,
        started: Arc<Barrier>,
        release: Arc<Notify>,
        should_block: AtomicBool,
    }

    impl BlockingHealthCheck {
        fn new(id: &str, started: Arc<Barrier>, release: Arc<Notify>) -> Self {
            Self {
                id: ComponentId::new(id),
                started,
                release,
                should_block: AtomicBool::new(true),
            }
        }
    }

    #[async_trait]
    impl HealthCheck for BlockingHealthCheck {
        async fn check(&self) -> Result<Status, Box<dyn std::error::Error + Send + Sync>> {
            if self.should_block.swap(false, Ordering::SeqCst) {
                self.started.wait().await;
                self.release.notified().await;
            }
            Ok(Status::Healthy)
        }

        fn component_id(&self) -> ComponentId {
            self.id.clone()
        }
    }

    #[tokio::test]
    async fn register_and_check() {
        let monitor = HealthMonitor::new(Duration::from_secs(60));
        let check = Arc::new(FixedHealthCheck::new("db", Status::Healthy));
        monitor.register_check(check).await;

        monitor.perform_health_checks().await;

        let status = monitor
            .get_status(&ComponentId::new("db"))
            .await
            .expect("status should be cached");
        assert_eq!(status.status, Status::Healthy);
    }

    #[tokio::test]
    async fn failing_check_produces_unhealthy() {
        let monitor = HealthMonitor::new(Duration::from_secs(60));
        monitor.register_check(Arc::new(FailingHealthCheck)).await;

        monitor.perform_health_checks().await;

        let status = monitor
            .get_status(&ComponentId::new("failing"))
            .await
            .expect("status should be cached");
        assert_eq!(status.status, Status::Unhealthy);
        assert!(status.message.is_some());
    }

    #[tokio::test]
    async fn is_system_healthy_no_unhealthy() {
        let monitor = HealthMonitor::new(Duration::from_secs(60));
        monitor
            .register_check(Arc::new(FixedHealthCheck::new("a", Status::Healthy)))
            .await;
        monitor
            .register_check(Arc::new(FixedHealthCheck::new("b", Status::Degraded)))
            .await;

        monitor.perform_health_checks().await;
        assert!(monitor.is_system_healthy().await);
    }

    #[tokio::test]
    async fn is_system_healthy_with_unhealthy() {
        let monitor = HealthMonitor::new(Duration::from_secs(60));
        monitor
            .register_check(Arc::new(FixedHealthCheck::new("a", Status::Healthy)))
            .await;
        monitor
            .register_check(Arc::new(FixedHealthCheck::new("b", Status::Unhealthy)))
            .await;

        monitor.perform_health_checks().await;
        assert!(!monitor.is_system_healthy().await);
    }

    #[tokio::test]
    async fn get_all_statuses() {
        let monitor = HealthMonitor::new(Duration::from_secs(60));
        monitor
            .register_check(Arc::new(FixedHealthCheck::new("x", Status::Healthy)))
            .await;
        monitor
            .register_check(Arc::new(FixedHealthCheck::new("y", Status::Degraded)))
            .await;

        monitor.perform_health_checks().await;

        let all = monitor.get_all_statuses().await;
        assert_eq!(all.len(), 2);
        assert_eq!(all[&ComponentId::new("x")].status, Status::Healthy);
        assert_eq!(all[&ComponentId::new("y")].status, Status::Degraded);
    }

    #[tokio::test]
    async fn run_loop_stops_on_shutdown() {
        let monitor = Arc::new(HealthMonitor::new(Duration::from_millis(50)));
        let shutdown = Arc::new(AtomicBool::new(false));

        let monitor_clone = Arc::clone(&monitor);
        let shutdown_clone = Arc::clone(&shutdown);
        let handle = tokio::spawn(async move {
            monitor_clone.run(shutdown_clone).await;
        });

        // Let it run briefly, then signal shutdown.
        tokio::time::sleep(Duration::from_millis(150)).await;
        shutdown.store(true, Ordering::SeqCst);

        // Should complete within a reasonable time.
        tokio::time::timeout(Duration::from_secs(2), handle)
            .await
            .expect("monitor should stop within timeout")
            .expect("monitor task should not panic");
    }

    #[tokio::test]
    async fn runtime_health_check_healthy() {
        let handle = tokio::runtime::Handle::current();
        let check = RuntimeHealthCheck::new(handle, Duration::from_secs(1));
        let status = check.check().await.expect("check should succeed");
        assert_eq!(status, Status::Healthy);
    }

    #[tokio::test]
    async fn default_check_interval() {
        let check = FixedHealthCheck::new("test", Status::Healthy);
        assert_eq!(check.check_interval(), Duration::from_secs(30));
    }

    #[tokio::test]
    async fn event_publisher_can_read_monitor_without_deadlock() {
        let monitor_ref: Arc<OnceLock<Arc<HealthMonitor>>> = Arc::new(OnceLock::new());
        let monitor_ref_for_publisher = Arc::clone(&monitor_ref);
        let publisher = Arc::new(CallbackEventPublisher::new(move || {
            let monitor = Arc::clone(
                monitor_ref_for_publisher
                    .get()
                    .expect("monitor should be initialized before publishing"),
            );
            async move {
                let statuses = monitor.get_all_statuses().await;
                let status = statuses
                    .get(&ComponentId::new("svc"))
                    .map(|entry| entry.status);
                assert_eq!(status, Some(Status::Healthy));
            }
        }));

        let monitor =
            Arc::new(HealthMonitor::new(Duration::from_secs(60)).with_event_publisher(publisher));
        assert!(
            monitor_ref.set(Arc::clone(&monitor)).is_ok(),
            "monitor should only be set once"
        );
        monitor
            .register_check(Arc::new(FixedHealthCheck::new("svc", Status::Healthy)))
            .await;

        tokio::time::timeout(Duration::from_secs(1), monitor.perform_health_checks())
            .await
            .expect("health checks should complete without deadlock");
    }

    #[tokio::test]
    async fn registration_during_checks_does_not_starve_writers() {
        let monitor = Arc::new(HealthMonitor::new(Duration::from_secs(60)));
        let started = Arc::new(Barrier::new(2));
        let release = Arc::new(Notify::new());

        monitor
            .register_check(Arc::new(BlockingHealthCheck::new(
                "slow",
                Arc::clone(&started),
                Arc::clone(&release),
            )))
            .await;

        let monitor_for_checks = Arc::clone(&monitor);
        let checks_task = tokio::spawn(async move {
            monitor_for_checks.perform_health_checks().await;
        });

        started.wait().await;

        let registration_result = tokio::time::timeout(
            Duration::from_secs(1),
            monitor.register_check(Arc::new(FixedHealthCheck::new("new", Status::Degraded))),
        )
        .await;

        release.notify_waiters();
        registration_result.expect("register_check should not block while checks are executing");
        tokio::time::timeout(Duration::from_secs(1), checks_task)
            .await
            .expect("health-check task should finish")
            .expect("health-check task should not panic");

        monitor.perform_health_checks().await;

        let all = monitor.get_all_statuses().await;
        assert_eq!(all[&ComponentId::new("slow")].status, Status::Healthy);
        assert_eq!(all[&ComponentId::new("new")].status, Status::Degraded);
    }
}
