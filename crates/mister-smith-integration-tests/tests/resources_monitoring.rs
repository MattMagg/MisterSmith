//! Integration test: ConnectionPool + ResourceManager + HealthMonitor.
//!
//! Covers T168: ConnectionPool with mock Resource acquires/releases through
//! ResourceManager, HealthMonitor checks pool health.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use mister_smith_monitoring::types::{ComponentId, Status};
use mister_smith_monitoring::{HealthCheck, HealthMonitor};
use mister_smith_resources::{ConnectionPool, PoolConfig, PoolHealthReport, ResourceManager};

/// Simple mock resource for testing.
#[derive(Debug)]
struct MockResource {
    id: usize,
}

/// Health check that inspects a `ConnectionPool<MockResource>` and reports
/// [`Status::Healthy`] when the pool has at least one managed resource.
struct PoolHealthCheck {
    pool: Arc<ConnectionPool<MockResource>>,
}

#[async_trait]
impl HealthCheck for PoolHealthCheck {
    async fn check(&self) -> Result<Status, Box<dyn std::error::Error + Send + Sync>> {
        let report: PoolHealthReport = self.pool.health_report();
        if report.total_resources > 0 {
            Ok(Status::Healthy)
        } else {
            Ok(Status::Unhealthy)
        }
    }

    fn component_id(&self) -> ComponentId {
        ComponentId::new("mock_pool")
    }
}

#[tokio::test]
async fn connection_pool_resource_manager_health_monitor() {
    // 1. Create a ConnectionPool<MockResource> with default PoolConfig and an
    //    incrementing-id factory.
    let counter = Arc::new(AtomicUsize::new(0));
    let factory_counter = Arc::clone(&counter);
    let pool = Arc::new(ConnectionPool::new(PoolConfig::default(), move || {
        let id = factory_counter.fetch_add(1, Ordering::SeqCst);
        async move { Ok(MockResource { id }) }
    }));

    // 2. Acquire a resource and verify it was created with id 0.
    let resource = pool.acquire().await.expect("first acquire should succeed");
    assert_eq!(resource.id, 0, "first resource should have id 0");
    assert_eq!(pool.active(), 1, "one resource should be active");

    // 3. Drop the resource so it returns to the pool.
    let first_id = resource.id;
    drop(resource);
    assert_eq!(pool.active(), 0, "no resources should be active after drop");
    assert_eq!(pool.size(), 1, "one idle resource should be in the pool");

    // 4. Acquire again — should reuse the same resource (same id, no new creation).
    let resource = pool.acquire().await.expect("second acquire should succeed");
    assert_eq!(
        resource.id, first_id,
        "pool should reuse the idle resource (same id)"
    );
    assert_eq!(
        pool.total_created(),
        1,
        "factory should only have been called once"
    );
    drop(resource);

    // 5. Register the pool in a ResourceManager under name "mock_pool".
    let manager = ResourceManager::new();
    manager.register_pool("mock_pool", Arc::clone(&pool));

    // 6. Verify pool_count is 1 and pool_names contains "mock_pool".
    assert_eq!(
        manager.pool_count(),
        1,
        "ResourceManager should have 1 pool"
    );
    let names = manager.pool_names();
    assert!(
        names.contains(&"mock_pool".to_string()),
        "pool_names should contain 'mock_pool', got: {names:?}"
    );

    // 7. Retrieve the pool from the manager and get a health report.
    let retrieved_pool = manager
        .get_pool::<Arc<ConnectionPool<MockResource>>>("mock_pool")
        .expect("should retrieve the registered pool by type");
    let report = retrieved_pool.health_report();
    assert!(
        report.total_resources > 0,
        "pool should have at least one resource, got total_resources={}",
        report.total_resources
    );

    // 8. Create a HealthMonitor and a custom PoolHealthCheck.
    let monitor = HealthMonitor::new(Duration::from_secs(60));
    let health_check = Arc::new(PoolHealthCheck {
        pool: Arc::clone(&pool),
    });

    // 9. Register the check, perform checks, and verify the system is healthy.
    monitor.register_check(health_check).await;
    monitor.perform_health_checks().await;
    assert!(
        monitor.is_system_healthy().await,
        "system should be healthy when pool has resources"
    );

    // Verify the specific component status was cached correctly.
    let status = monitor
        .get_status(&ComponentId::new("mock_pool"))
        .await
        .expect("mock_pool status should be cached after check");
    assert_eq!(
        status.status,
        Status::Healthy,
        "mock_pool should report Healthy"
    );
}
