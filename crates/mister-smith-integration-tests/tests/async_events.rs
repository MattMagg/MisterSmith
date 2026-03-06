//! T167: Integration test — TaskExecutor + CircuitBreaker + EventBus.
//!
//! Verifies that:
//! - TaskExecutor submits tasks and tracks metrics.
//! - CircuitBreaker trips to Open after reaching the failure threshold.
//! - A tripped breaker causes the executor to reject subsequent submissions.
//! - Metrics correctly reflect submitted tasks and circuit breaker trips.
//! - The EventBus integrates with the async subsystem by publishing and
//!   receiving events over a broadcast channel.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use uuid::Uuid;

use mister_smith_async::{AsyncTask, CircuitBreaker, CircuitState, TaskError, TaskExecutor};
use mister_smith_events::{Event, EventBus, EventType, SystemEventType};

// ---------------------------------------------------------------------------
// Test helper: a task that always fails.
// ---------------------------------------------------------------------------

struct FailingTask {
    id: Uuid,
}

impl FailingTask {
    fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }
}

#[async_trait]
impl AsyncTask for FailingTask {
    async fn execute(&self) -> Result<serde_json::Value, TaskError> {
        Err(TaskError::ExecutionFailed("boom".into()))
    }

    fn task_id(&self) -> Uuid {
        self.id
    }
}

// ---------------------------------------------------------------------------
// T167: TaskExecutor + CircuitBreaker + EventBus integration test.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn task_executor_circuit_breaker_and_event_bus() {
    // 1. Create a CircuitBreaker: trips after 3 failures, 60 s recovery, 1 half-open probe.
    let cb = Arc::new(CircuitBreaker::new(3, Duration::from_secs(60), 1));
    assert_eq!(cb.state(), CircuitState::Closed);

    // 2. Create a TaskExecutor with max_concurrent=4 and attach the circuit breaker.
    let executor = TaskExecutor::new(4).with_circuit_breaker(Arc::clone(&cb));

    // 3-4. Submit the failing task 3 times, awaiting each to completion.
    for i in 0..3 {
        let task: Arc<dyn AsyncTask> = Arc::new(FailingTask::new());
        let handle = executor
            .submit(task)
            .unwrap_or_else(|e| panic!("submit #{} should succeed but got: {e}", i + 1));

        // The JoinHandle resolves to the inner task result — which is an Err.
        let inner = handle.await.expect("JoinHandle should not panic");
        assert!(
            inner.is_err(),
            "task #{} should have returned an error",
            i + 1
        );
    }

    // 5. After 3 failures the circuit breaker must be Open.
    assert_eq!(
        cb.state(),
        CircuitState::Open,
        "circuit breaker should be Open after 3 failures"
    );

    // 6. A 4th submission must be rejected immediately by the breaker.
    let task_4: Arc<dyn AsyncTask> = Arc::new(FailingTask::new());
    let rejection = executor.submit(task_4);
    assert!(
        rejection.is_err(),
        "4th submit should be rejected by open circuit breaker"
    );
    assert!(
        matches!(rejection, Err(TaskError::CircuitBreakerOpen)),
        "rejection error should be CircuitBreakerOpen"
    );

    // 7. Verify metrics snapshot.
    let snap = executor.metrics().snapshot();
    assert!(
        snap.total_submitted >= 3,
        "total_submitted should be >= 3, got {}",
        snap.total_submitted
    );
    assert!(
        snap.circuit_breaker_trips >= 1,
        "circuit_breaker_trips should be >= 1, got {}",
        snap.circuit_breaker_trips
    );

    // 8. Demonstrate async + events integration: publish a system event through
    //    the EventBus and verify it is received on the broadcast channel.
    let bus = EventBus::new(128);
    let mut rx = bus.subscribe_broadcast();

    let event = Event::new(
        "integration-test",
        EventType::System(SystemEventType::CircuitBreakerOpen),
    );
    let event_id = event.id;

    bus.publish(event)
        .await
        .expect("EventBus publish should succeed");

    let received = rx
        .recv()
        .await
        .expect("broadcast receiver should get the event");

    assert_eq!(received.id, event_id);
    assert_eq!(
        received.event_type,
        EventType::System(SystemEventType::CircuitBreakerOpen),
    );
    assert_eq!(received.source, "integration-test");
}
