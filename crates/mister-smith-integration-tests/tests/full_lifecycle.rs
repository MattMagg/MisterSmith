//! Integration test: Full Phase 2 startup -> operate -> shutdown lifecycle (T169).
//!
//! Exercises the complete Phase 2 startup sequence:
//!   RuntimeManager -> MonitoringSystem -> EventBus -> shutdown
//!
//! Verifies:
//! - All components start without error.
//! - Health checks report healthy while the runtime is alive.
//! - Metrics are recorded and buffered correctly.
//! - The shutdown signal stops monitoring background tasks.
//! - `RuntimeManager::graceful_shutdown` completes cleanly.
//! - No resource leaks (all join handles resolve, no panics).
//!
//! Note: `RuntimeManager::graceful_shutdown` calls `block_on` internally,
//! which cannot be called from within an async runtime. It is therefore
//! dispatched to a blocking thread via `spawn_blocking`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use mister_smith_config::RuntimeConfig;
use mister_smith_core::EventPublisher;
use mister_smith_events::{Event, EventBus, EventType, SystemEventType};
use mister_smith_monitoring::types::{ComponentId, Status};
use mister_smith_monitoring::{HealthMonitor, MetricsCollector, MonitoringSystem, RuntimeHealthCheck};
use mister_smith_runtime::manager::RuntimeManager;

#[tokio::test]
async fn full_phase2_startup_shutdown_lifecycle() {
    // -----------------------------------------------------------------------
    // 1. Create a RuntimeConfig with a small worker pool.
    // -----------------------------------------------------------------------
    let config = RuntimeConfig {
        worker_threads: Some(2),
        blocking_threads: 32,
        ..RuntimeConfig::default()
    };

    // -----------------------------------------------------------------------
    // 2. Create an EventBus (capacity 32) wrapped in Arc.
    //    EventBus implements EventPublisher, so it can be shared with both the
    //    RuntimeManager builder and the MonitoringSystem.
    // -----------------------------------------------------------------------
    let event_bus = Arc::new(EventBus::new(32));

    // -----------------------------------------------------------------------
    // 3. Build a RuntimeManager using the builder pattern.
    //    - Attach the EventBus as the event publisher.
    //    - Set a 5-second shutdown timeout.
    // -----------------------------------------------------------------------
    let manager = RuntimeManager::builder()
        .shutdown_timeout(Duration::from_secs(5))
        .event_publisher(Arc::clone(&event_bus) as Arc<dyn EventPublisher>)
        .build(&config)
        .expect("RuntimeManager should build successfully");

    // -----------------------------------------------------------------------
    // 4. Create a HealthMonitor (1-second check interval) and register a
    //    RuntimeHealthCheck targeting the managed runtime's handle.
    // -----------------------------------------------------------------------
    let health_monitor = Arc::new(HealthMonitor::new(Duration::from_secs(1)));

    let runtime_handle = manager.runtime().handle().clone();
    let runtime_health_check = Arc::new(RuntimeHealthCheck::new(
        runtime_handle,
        Duration::from_secs(5),
    ));
    health_monitor.register_check(runtime_health_check).await;

    // -----------------------------------------------------------------------
    // 5. Create a MetricsCollector (1-second flush interval).
    // -----------------------------------------------------------------------
    let metrics_collector = Arc::new(MetricsCollector::new(Duration::from_secs(1)));

    // -----------------------------------------------------------------------
    // 6. Create a MonitoringSystem with the health monitor and metrics
    //    collector, and attach the event publisher.
    // -----------------------------------------------------------------------
    let monitoring_system = MonitoringSystem::new(
        Arc::clone(&health_monitor),
        Arc::clone(&metrics_collector),
    )
    .with_event_publisher(Arc::clone(&event_bus) as Arc<dyn EventPublisher>);

    // -----------------------------------------------------------------------
    // 7. Create a shutdown signal (Arc<AtomicBool>).
    //    This signal will be used to stop the monitoring background tasks.
    // -----------------------------------------------------------------------
    let shutdown_signal = Arc::new(AtomicBool::new(false));

    // -----------------------------------------------------------------------
    // 8. Start the monitoring system.
    //    This spawns two background tasks: the health monitor loop and the
    //    metrics collector flush loop.
    // -----------------------------------------------------------------------
    let (health_handle, metrics_handle) = monitoring_system.start(Arc::clone(&shutdown_signal));

    // -----------------------------------------------------------------------
    // 9. Publish a SystemEventType::Started event through the EventBus.
    //    This simulates the framework announcing its startup.
    // -----------------------------------------------------------------------
    let started_event = Event::new(
        "mister-smith",
        EventType::System(SystemEventType::Started),
    );
    event_bus
        .publish(started_event)
        .await
        .expect("publishing Started event should succeed");

    // -----------------------------------------------------------------------
    // 10. Run one round of health checks and verify the system is healthy.
    //     The RuntimeHealthCheck spawns a yield_now() task on the managed
    //     runtime and expects it to complete within the timeout -- proving
    //     the runtime is responsive.
    // -----------------------------------------------------------------------
    health_monitor.perform_health_checks().await;

    let runtime_status = health_monitor
        .get_status(&ComponentId::new("tokio-runtime"))
        .await
        .expect("RuntimeHealthCheck status should be cached after perform_health_checks");
    assert_eq!(
        runtime_status.status,
        Status::Healthy,
        "Tokio runtime should report Healthy after a successful health check"
    );
    assert!(
        health_monitor.is_system_healthy().await,
        "Overall system should be healthy when all checks pass"
    );

    // -----------------------------------------------------------------------
    // 11. Record an event metric on the MetricsCollector and verify the
    //     buffer contains at least one entry.
    // -----------------------------------------------------------------------
    metrics_collector.record_event_published().await;
    let buffered = metrics_collector.buffered_count().await;
    assert!(
        buffered > 0,
        "MetricsCollector should have at least one buffered metric after record_event_published (got {buffered})"
    );

    // -----------------------------------------------------------------------
    // 12. Set the shutdown signal to true, stopping monitoring background
    //     tasks.
    // -----------------------------------------------------------------------
    shutdown_signal.store(true, Ordering::SeqCst);

    // -----------------------------------------------------------------------
    // 13. Wait briefly for the monitoring tasks to observe the shutdown signal
    //     and exit cleanly, then join the handles to confirm they stopped.
    // -----------------------------------------------------------------------
    tokio::time::sleep(Duration::from_millis(100)).await;

    tokio::time::timeout(Duration::from_secs(5), health_handle)
        .await
        .expect("health monitor task should stop within timeout")
        .expect("health monitor task should not panic");

    tokio::time::timeout(Duration::from_secs(5), metrics_handle)
        .await
        .expect("metrics collector task should stop within timeout")
        .expect("metrics collector task should not panic");

    // -----------------------------------------------------------------------
    // 14. Graceful shutdown of the RuntimeManager.
    //     `graceful_shutdown` calls `block_on` internally to join tracked
    //     tasks, so it must run on a blocking thread to avoid the
    //     "cannot block_on inside a runtime" panic.
    // -----------------------------------------------------------------------
    tokio::task::spawn_blocking(move || {
        manager
            .graceful_shutdown()
            .expect("RuntimeManager graceful_shutdown should succeed");
    })
    .await
    .expect("shutdown spawn_blocking should not panic");

    // -----------------------------------------------------------------------
    // 15. If we reached this point, the full startup -> operate -> shutdown
    //     lifecycle completed without panics or resource leaks.
    // -----------------------------------------------------------------------
    // (No explicit assert needed -- reaching here proves the lifecycle
    //  completed successfully. The prior asserts verified correctness at
    //  each stage.)
}
