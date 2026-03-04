//! MonitoringSystem coordinator wiring health and metrics.
//!
//! [`MonitoringSystem`] owns the [`HealthMonitor`] and [`MetricsCollector`],
//! spawns their background loops on `start`, and provides accessor methods for
//! the rest of the framework.

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tracing::info;

use mister_smith_core::EventPublisher;

use crate::health::HealthMonitor;
use crate::metrics::MetricsCollector;

/// Top-level coordinator that owns and starts the monitoring subsystem.
pub struct MonitoringSystem {
    /// The health monitor instance.
    health_monitor: Arc<HealthMonitor>,
    /// The metrics collector instance.
    metrics_collector: Arc<MetricsCollector>,
    /// Optional event publisher for health-change events.
    event_publisher: Option<Arc<dyn EventPublisher>>,
}

impl MonitoringSystem {
    /// Create a new `MonitoringSystem` with the given monitor and collector.
    pub fn new(
        health_monitor: Arc<HealthMonitor>,
        metrics_collector: Arc<MetricsCollector>,
    ) -> Self {
        Self {
            health_monitor,
            metrics_collector,
            event_publisher: None,
        }
    }

    /// Attach an event publisher. This is informational on the system level;
    /// the [`HealthMonitor`] should be constructed with
    /// [`HealthMonitor::with_event_publisher`] to actually publish events.
    pub fn with_event_publisher(mut self, publisher: Arc<dyn EventPublisher>) -> Self {
        self.event_publisher = Some(publisher);
        self
    }

    /// Spawn background tasks for both the health monitor and the metrics
    /// collector. Both tasks run until `shutdown` is set to `true`.
    ///
    /// Returns join handles so the caller can await orderly termination.
    pub fn start(
        &self,
        shutdown: Arc<AtomicBool>,
    ) -> (
        tokio::task::JoinHandle<()>,
        tokio::task::JoinHandle<()>,
    ) {
        info!("Starting monitoring system");

        let health_monitor = Arc::clone(&self.health_monitor);
        let health_shutdown = Arc::clone(&shutdown);
        let health_handle = tokio::spawn(async move {
            health_monitor.run(health_shutdown).await;
        });

        let metrics_collector = Arc::clone(&self.metrics_collector);
        let metrics_shutdown = Arc::clone(&shutdown);
        let metrics_handle = tokio::spawn(async move {
            metrics_collector.run(metrics_shutdown).await;
        });

        (health_handle, metrics_handle)
    }

    /// Returns a reference to the shared [`HealthMonitor`].
    pub fn health_monitor(&self) -> &Arc<HealthMonitor> {
        &self.health_monitor
    }

    /// Returns a reference to the shared [`MetricsCollector`].
    pub fn metrics_collector(&self) -> &Arc<MetricsCollector> {
        &self.metrics_collector
    }

    /// Returns a reference to the event publisher, if one was attached.
    pub fn event_publisher(&self) -> Option<&Arc<dyn EventPublisher>> {
        self.event_publisher.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::HealthMonitor;
    use crate::metrics::MetricsCollector;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    #[tokio::test]
    async fn start_and_stop() {
        let hm = Arc::new(HealthMonitor::new(Duration::from_millis(50)));
        let mc = Arc::new(MetricsCollector::new(Duration::from_millis(50)));
        let system = MonitoringSystem::new(Arc::clone(&hm), Arc::clone(&mc));

        let shutdown = Arc::new(AtomicBool::new(false));
        let (health_handle, metrics_handle) = system.start(Arc::clone(&shutdown));

        // Let it run for a bit.
        tokio::time::sleep(Duration::from_millis(150)).await;
        shutdown.store(true, Ordering::SeqCst);

        tokio::time::timeout(Duration::from_secs(2), health_handle)
            .await
            .expect("health monitor should stop")
            .expect("health monitor should not panic");

        tokio::time::timeout(Duration::from_secs(2), metrics_handle)
            .await
            .expect("metrics collector should stop")
            .expect("metrics collector should not panic");
    }

    #[test]
    fn accessors() {
        let hm = Arc::new(HealthMonitor::new(Duration::from_secs(30)));
        let mc = Arc::new(MetricsCollector::new(Duration::from_secs(10)));
        let system = MonitoringSystem::new(Arc::clone(&hm), Arc::clone(&mc));

        // Should return the same Arc instances.
        assert!(Arc::ptr_eq(system.health_monitor(), &hm));
        assert!(Arc::ptr_eq(system.metrics_collector(), &mc));
        assert!(system.event_publisher().is_none());
    }
}
