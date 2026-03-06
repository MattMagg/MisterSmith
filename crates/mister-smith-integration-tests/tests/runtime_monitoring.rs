//! Integration test: RuntimeManager + MonitoringSystem health check (T165).
//!
//! Verifies that a `RuntimeManager` starts, a `RuntimeHealthCheck` is
//! registered with a `HealthMonitor`, and the health check executes
//! returning `Healthy`.
//!
//! `RuntimeManager::initialize` builds a new multi-thread Tokio runtime
//! internally, and `graceful_shutdown` calls `block_on`. Both operations
//! are forbidden from within an async context, so they are performed on
//! a blocking thread via `spawn_blocking`.

use std::sync::Arc;
use std::time::Duration;

use mister_smith_config::RuntimeConfig;
use mister_smith_monitoring::types::{ComponentId, Status};
use mister_smith_monitoring::{
    HealthMonitor, MetricsCollector, MonitoringSystem, RuntimeHealthCheck,
};
use mister_smith_runtime::manager::RuntimeManager;

#[tokio::test]
async fn runtime_health_check_returns_healthy() {
    // 1. Create a RuntimeConfig with a small worker pool.
    let config = RuntimeConfig {
        worker_threads: Some(2),
        ..RuntimeConfig::default()
    };

    // 2. Initialize the RuntimeManager on a blocking thread (it builds a
    //    new Tokio runtime internally, which cannot be done inside an
    //    existing async runtime).
    let manager = tokio::task::spawn_blocking(move || {
        RuntimeManager::initialize(&config).expect("RuntimeManager should initialize")
    })
    .await
    .expect("spawn_blocking should not panic");

    // 3. Create a RuntimeHealthCheck targeting the managed runtime.
    let handle = manager.runtime().handle().clone();
    let health_check = Arc::new(RuntimeHealthCheck::new(handle, Duration::from_secs(5)));

    // 4. Create a HealthMonitor and register the health check.
    let health_monitor = Arc::new(HealthMonitor::new(Duration::from_secs(30)));
    health_monitor.register_check(health_check).await;

    // 5. Create a MonitoringSystem (requires both HealthMonitor and MetricsCollector).
    let metrics_collector = Arc::new(MetricsCollector::new(Duration::from_secs(60)));
    let _monitoring_system =
        MonitoringSystem::new(Arc::clone(&health_monitor), Arc::clone(&metrics_collector));

    // 6. Run health checks.
    health_monitor.perform_health_checks().await;

    // 7. Assert the runtime health check returned Healthy.
    let component_id = ComponentId::new("tokio-runtime");
    let status = health_monitor
        .get_status(&component_id)
        .await
        .expect("RuntimeHealthCheck status should be cached after perform_health_checks");
    assert_eq!(
        status.status,
        Status::Healthy,
        "Tokio runtime should be healthy"
    );

    // 8. Assert the overall system is healthy.
    assert!(
        health_monitor.is_system_healthy().await,
        "System should be healthy when all checks pass"
    );

    // 9. Graceful shutdown of the RuntimeManager on a blocking thread
    //    (it calls block_on internally).
    tokio::task::spawn_blocking(move || {
        manager
            .graceful_shutdown()
            .expect("graceful_shutdown should succeed");
    })
    .await
    .expect("shutdown spawn_blocking should not panic");
}
