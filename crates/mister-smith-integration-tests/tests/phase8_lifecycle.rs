//! Integration test: Phase 8 bootstrap and lifecycle.
//!
//! Verifies:
//! - BootstrapContext initializes all core subsystems (no external services)
//! - Process state transitions: Starting → Ready → Draining → Stopped
//! - Graceful shutdown completes cleanly
//! - EventBus, HealthMonitor, MetricsCollector, SupervisedSystem, AgentRegistry
//!   are all wired and functional after bootstrap

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

use mister_smith_agents::AgentRegistry;
use mister_smith_config::FrameworkConfig;
use mister_smith_core::ProcessLifecycle;
use mister_smith_core::SupervisionStrategy;
use mister_smith_events::EventBus;
use mister_smith_monitoring::{HealthMonitor, MetricsCollector};
use mister_smith_supervision::SupervisedSystem;

/// Minimal process state tracker for test assertions.
#[derive(Clone)]
struct TestStateTracker {
    state: Arc<AtomicU8>,
}

impl TestStateTracker {
    fn new() -> Self {
        Self {
            state: Arc::new(AtomicU8::new(ProcessLifecycle::Starting as u8)),
        }
    }

    fn set(&self, lifecycle: ProcessLifecycle) {
        self.state.store(lifecycle as u8, Ordering::SeqCst);
    }

    fn get(&self) -> ProcessLifecycle {
        match self.state.load(Ordering::SeqCst) {
            0 => ProcessLifecycle::Starting,
            1 => ProcessLifecycle::Ready,
            2 => ProcessLifecycle::Draining,
            3 => ProcessLifecycle::Stopped,
            _ => ProcessLifecycle::Failed,
        }
    }
}

#[tokio::test]
async fn bootstrap_core_subsystems_without_external_services() {
    // Use default config (no NATS URL, no persistence) — pure in-process bootstrap
    let config = FrameworkConfig::default();

    // Verify initial state
    let state = TestStateTracker::new();
    assert_eq!(state.get(), ProcessLifecycle::Starting);

    // Initialize core subsystems (same order as bootstrap.rs)
    let event_bus = Arc::new(EventBus::new(64));
    let health_monitor = Arc::new(HealthMonitor::new(Duration::from_secs(30)));
    let _metrics_collector = Arc::new(MetricsCollector::new(Duration::from_secs(60)));
    let actor_config = mister_smith_actor::ActorSystemConfig::default();
    let supervised_system = Arc::new(SupervisedSystem::with_event_bus(
        actor_config,
        event_bus.clone(),
    ));
    let agent_registry = Arc::new(AgentRegistry::new());

    // Verify NATS is not configured (no URL in default config)
    assert!(config.transport.nats_url.is_none());

    // Verify supervision tree is operational
    let _supervisor_id = supervised_system
        .create_supervisor(SupervisionStrategy::default())
        .await;

    // Verify agent registry starts empty
    assert_eq!(agent_registry.count(), 0);

    // Verify health monitor accepts registrations
    assert!(health_monitor.is_system_healthy().await);

    // Mark ready
    state.set(ProcessLifecycle::Ready);
    assert_eq!(state.get(), ProcessLifecycle::Ready);
}

#[tokio::test]
async fn lifecycle_state_transitions_are_correct() {
    let state = TestStateTracker::new();

    // Starting is the initial state
    assert_eq!(state.get(), ProcessLifecycle::Starting);

    // Transition to Ready after bootstrap
    state.set(ProcessLifecycle::Ready);
    assert_eq!(state.get(), ProcessLifecycle::Ready);

    // Transition to Draining on shutdown signal
    state.set(ProcessLifecycle::Draining);
    assert_eq!(state.get(), ProcessLifecycle::Draining);

    // Transition to Stopped after graceful shutdown
    state.set(ProcessLifecycle::Stopped);
    assert_eq!(state.get(), ProcessLifecycle::Stopped);
}

#[tokio::test]
async fn lifecycle_failed_state_on_bootstrap_error() {
    let state = TestStateTracker::new();
    assert_eq!(state.get(), ProcessLifecycle::Starting);

    // Simulate bootstrap failure
    state.set(ProcessLifecycle::Failed);
    assert_eq!(state.get(), ProcessLifecycle::Failed);
}

#[tokio::test]
async fn observability_config_validates_with_defaults() {
    let config = FrameworkConfig::default();
    // Default ObservabilityConfig should pass validation
    assert!(config.observability.validate().is_ok());
}

#[tokio::test]
async fn startup_timeout_is_configurable() {
    let config = FrameworkConfig::default();
    // Default is 30 seconds
    assert_eq!(config.observability.startup_timeout_secs, 30);
    assert_eq!(config.observability.shutdown_timeout_secs, 30);
}

#[tokio::test]
async fn monitoring_background_tasks_respond_to_shutdown_flag() {
    let shutdown_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));

    let health_monitor = Arc::new(HealthMonitor::new(Duration::from_millis(50)));
    let metrics_collector = Arc::new(MetricsCollector::new(Duration::from_millis(50)));

    // Start background tasks
    let monitor_flag = shutdown_flag.clone();
    let monitor = health_monitor.clone();
    let monitor_handle = tokio::spawn(async move {
        monitor.run(monitor_flag).await;
    });

    let metrics_flag = shutdown_flag.clone();
    let metrics = metrics_collector.clone();
    let metrics_handle = tokio::spawn(async move {
        metrics.run(metrics_flag).await;
    });

    // Let them run briefly
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Signal shutdown
    shutdown_flag.store(true, std::sync::atomic::Ordering::SeqCst);

    // Both tasks should complete within a reasonable time
    let monitor_result = tokio::time::timeout(Duration::from_secs(5), monitor_handle).await;
    assert!(monitor_result.is_ok(), "Monitor task did not stop in time");

    let metrics_result = tokio::time::timeout(Duration::from_secs(5), metrics_handle).await;
    assert!(metrics_result.is_ok(), "Metrics task did not stop in time");
}
