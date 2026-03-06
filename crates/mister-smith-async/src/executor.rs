//! Task executor with concurrency control, metrics tracking, and retry support.
//!
//! [`TaskExecutor`] manages the lifecycle of submitted [`AsyncTask`] instances,
//! enforcing a concurrency limit via a [`tokio::sync::Semaphore`] and tracking
//! operational metrics through atomic counters.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use tracing::{debug, error, warn};

use crate::circuit_breaker::CircuitBreaker;
use crate::retry::RetryPolicy;
use crate::task::{AsyncTask, TaskError};

/// Strategy for handling errors during execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorStrategy {
    /// Stop processing on the first error.
    StopOnError,
    /// Log the error and continue with remaining tasks.
    LogAndContinue,
    /// Retry the failed task with exponential backoff.
    RetryWithBackoff,
    /// Use a circuit breaker to prevent cascading failures.
    CircuitBreaker,
}

/// Live atomic counters for task executor metrics.
pub struct TaskMetrics {
    /// Total number of tasks submitted.
    pub total_submitted: AtomicU64,
    /// Tasks that completed successfully.
    pub completed: AtomicU64,
    /// Tasks that failed.
    pub failed: AtomicU64,
    /// Tasks currently executing.
    pub currently_running: AtomicU64,
    /// Tasks whose panics were caught and recovered.
    pub panics_recovered: AtomicU64,
    /// Number of times the circuit breaker tripped.
    pub circuit_breaker_trips: AtomicU64,
}

impl TaskMetrics {
    /// Create a new zeroed metrics instance.
    fn new() -> Self {
        Self {
            total_submitted: AtomicU64::new(0),
            completed: AtomicU64::new(0),
            failed: AtomicU64::new(0),
            currently_running: AtomicU64::new(0),
            panics_recovered: AtomicU64::new(0),
            circuit_breaker_trips: AtomicU64::new(0),
        }
    }

    /// Take a consistent point-in-time snapshot of all counters.
    pub fn snapshot(&self) -> TaskMetricsSnapshot {
        TaskMetricsSnapshot {
            total_submitted: self.total_submitted.load(Ordering::Relaxed),
            completed: self.completed.load(Ordering::Relaxed),
            failed: self.failed.load(Ordering::Relaxed),
            currently_running: self.currently_running.load(Ordering::Relaxed),
            panics_recovered: self.panics_recovered.load(Ordering::Relaxed),
            circuit_breaker_trips: self.circuit_breaker_trips.load(Ordering::Relaxed),
        }
    }
}

impl std::fmt::Debug for TaskMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskMetrics")
            .field(
                "total_submitted",
                &self.total_submitted.load(Ordering::Relaxed),
            )
            .field("completed", &self.completed.load(Ordering::Relaxed))
            .field("failed", &self.failed.load(Ordering::Relaxed))
            .field(
                "currently_running",
                &self.currently_running.load(Ordering::Relaxed),
            )
            .field(
                "panics_recovered",
                &self.panics_recovered.load(Ordering::Relaxed),
            )
            .field(
                "circuit_breaker_trips",
                &self.circuit_breaker_trips.load(Ordering::Relaxed),
            )
            .finish()
    }
}

/// Plain-data snapshot of [`TaskMetrics`] at a point in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskMetricsSnapshot {
    /// Total number of tasks submitted.
    pub total_submitted: u64,
    /// Tasks that completed successfully.
    pub completed: u64,
    /// Tasks that failed.
    pub failed: u64,
    /// Tasks currently executing.
    pub currently_running: u64,
    /// Tasks whose panics were caught and recovered.
    pub panics_recovered: u64,
    /// Number of times the circuit breaker tripped.
    pub circuit_breaker_trips: u64,
}

/// Concurrent task executor with semaphore-based concurrency control.
pub struct TaskExecutor {
    semaphore: Arc<Semaphore>,
    metrics: Arc<TaskMetrics>,
    #[allow(dead_code)]
    max_concurrent: usize,
    #[allow(dead_code)]
    error_strategy: ErrorStrategy,
    circuit_breaker: Option<Arc<CircuitBreaker>>,
}

impl std::fmt::Debug for TaskExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskExecutor")
            .field("max_concurrent", &self.max_concurrent)
            .field("error_strategy", &self.error_strategy)
            .field("has_circuit_breaker", &self.circuit_breaker.is_some())
            .finish()
    }
}

impl TaskExecutor {
    /// Create a new executor with the given concurrency limit.
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            metrics: Arc::new(TaskMetrics::new()),
            max_concurrent,
            error_strategy: ErrorStrategy::StopOnError,
            circuit_breaker: None,
        }
    }

    /// Set the error strategy (builder pattern).
    pub fn with_error_strategy(mut self, strategy: ErrorStrategy) -> Self {
        self.error_strategy = strategy;
        self
    }

    /// Attach a circuit breaker (builder pattern).
    pub fn with_circuit_breaker(mut self, breaker: Arc<CircuitBreaker>) -> Self {
        self.circuit_breaker = Some(breaker);
        self
    }

    /// Submit a task for execution.
    ///
    /// Acquires a semaphore permit before spawning the task on the Tokio
    /// runtime. The task is wrapped with its configured timeout. Returns a
    /// `JoinHandle` that resolves to the task result.
    pub fn submit(
        &self,
        task: Arc<dyn AsyncTask>,
    ) -> Result<JoinHandle<Result<serde_json::Value, TaskError>>, TaskError> {
        // Check circuit breaker first.
        if let Some(ref cb) = self.circuit_breaker {
            if !cb.can_proceed() {
                self.metrics
                    .circuit_breaker_trips
                    .fetch_add(1, Ordering::Relaxed);
                return Err(TaskError::CircuitBreakerOpen);
            }
        }

        self.metrics.total_submitted.fetch_add(1, Ordering::Relaxed);

        let semaphore = Arc::clone(&self.semaphore);
        let metrics = Arc::clone(&self.metrics);
        let cb = self.circuit_breaker.clone();
        let timeout = task.timeout();
        let task_id = task.task_id();

        let handle = tokio::spawn(async move {
            // Acquire semaphore permit (blocks until a slot is available).
            let _permit = semaphore
                .acquire()
                .await
                .map_err(|_| TaskError::SpawnFailed("semaphore closed".into()))?;

            metrics.currently_running.fetch_add(1, Ordering::Relaxed);
            debug!(task_id = %task_id, "Task started");

            let result = match tokio::time::timeout(timeout, task.execute()).await {
                Ok(Ok(value)) => {
                    if let Some(ref cb) = cb {
                        cb.record_success();
                    }
                    metrics.completed.fetch_add(1, Ordering::Relaxed);
                    debug!(task_id = %task_id, "Task completed successfully");
                    Ok(value)
                }
                Ok(Err(e)) => {
                    if let Some(ref cb) = cb {
                        cb.record_failure();
                    }
                    metrics.failed.fetch_add(1, Ordering::Relaxed);
                    error!(task_id = %task_id, error = %e, "Task failed");
                    Err(e)
                }
                Err(_) => {
                    if let Some(ref cb) = cb {
                        cb.record_failure();
                    }
                    metrics.failed.fetch_add(1, Ordering::Relaxed);
                    warn!(task_id = %task_id, "Task timed out");
                    Err(TaskError::Timeout(format!(
                        "Task {task_id} exceeded timeout of {timeout:?}"
                    )))
                }
            };

            metrics.currently_running.fetch_sub(1, Ordering::Relaxed);
            result
        });

        Ok(handle)
    }

    /// Execute a task with retry logic according to the given policy.
    ///
    /// Runs the task up to `policy.max_attempts` times with exponential
    /// backoff between attempts. If a circuit breaker is attached and trips,
    /// returns [`TaskError::CircuitBreakerOpen`] immediately.
    pub async fn execute_with_retry(
        &self,
        task: Arc<dyn AsyncTask>,
        policy: &RetryPolicy,
    ) -> Result<serde_json::Value, TaskError> {
        let mut last_error = String::new();

        for attempt in 0..policy.max_attempts {
            // Check circuit breaker.
            if let Some(ref cb) = self.circuit_breaker {
                if !cb.can_proceed() {
                    self.metrics
                        .circuit_breaker_trips
                        .fetch_add(1, Ordering::Relaxed);
                    return Err(TaskError::CircuitBreakerOpen);
                }
            }

            match self.submit(Arc::clone(&task)) {
                Ok(handle) => match handle.await {
                    Ok(Ok(value)) => return Ok(value),
                    Ok(Err(e)) => {
                        last_error = e.to_string();
                        warn!(
                            task_id = %task.task_id(),
                            attempt = attempt + 1,
                            max_attempts = policy.max_attempts,
                            error = %e,
                            "Task attempt failed"
                        );
                    }
                    Err(join_err) => {
                        // JoinError means the task panicked or was cancelled.
                        last_error = join_err.to_string();
                        self.metrics
                            .panics_recovered
                            .fetch_add(1, Ordering::Relaxed);
                        warn!(
                            task_id = %task.task_id(),
                            attempt = attempt + 1,
                            "Task panicked or was cancelled"
                        );
                    }
                },
                Err(e) => {
                    last_error = e.to_string();
                    if matches!(e, TaskError::CircuitBreakerOpen) {
                        return Err(e);
                    }
                }
            }

            // Sleep before retrying (skip sleep on the last attempt).
            if attempt + 1 < policy.max_attempts {
                let delay = policy.delay_for_attempt(attempt);
                debug!(
                    task_id = %task.task_id(),
                    delay = ?delay,
                    "Waiting before retry"
                );
                tokio::time::sleep(delay).await;
            }
        }

        Err(TaskError::RetryExhausted {
            attempts: policy.max_attempts,
            last_error,
        })
    }

    /// Access the live metrics.
    pub fn metrics(&self) -> &Arc<TaskMetrics> {
        &self.metrics
    }

    /// Graceful shutdown placeholder.
    ///
    /// Currently a no-op. Future versions may drain in-flight tasks and close
    /// the semaphore.
    pub fn shutdown(&self) {
        debug!("TaskExecutor shutdown requested (no-op)");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{AsyncTask, TaskError};
    use async_trait::async_trait;
    use serde_json::json;
    use std::time::Duration;
    use uuid::Uuid;

    struct SuccessTask {
        id: Uuid,
    }

    #[async_trait]
    impl AsyncTask for SuccessTask {
        async fn execute(&self) -> Result<serde_json::Value, TaskError> {
            Ok(json!({"result": "ok"}))
        }
        fn task_id(&self) -> Uuid {
            self.id
        }
    }

    struct FailTask {
        id: Uuid,
    }

    #[async_trait]
    impl AsyncTask for FailTask {
        async fn execute(&self) -> Result<serde_json::Value, TaskError> {
            Err(TaskError::ExecutionFailed("always fails".into()))
        }
        fn task_id(&self) -> Uuid {
            self.id
        }
    }

    struct SlowTask {
        id: Uuid,
        duration: Duration,
    }

    #[async_trait]
    impl AsyncTask for SlowTask {
        async fn execute(&self) -> Result<serde_json::Value, TaskError> {
            tokio::time::sleep(self.duration).await;
            Ok(json!({"result": "slow_done"}))
        }
        fn timeout(&self) -> Duration {
            Duration::from_millis(50)
        }
        fn task_id(&self) -> Uuid {
            self.id
        }
    }

    #[tokio::test]
    async fn submit_success() {
        let executor = TaskExecutor::new(4);
        let task: Arc<dyn AsyncTask> = Arc::new(SuccessTask { id: Uuid::new_v4() });
        let handle = executor.submit(task).unwrap();
        let result = handle.await.unwrap().unwrap();
        assert_eq!(result["result"], "ok");

        let snap = executor.metrics().snapshot();
        assert_eq!(snap.total_submitted, 1);
        assert_eq!(snap.completed, 1);
        assert_eq!(snap.failed, 0);
    }

    #[tokio::test]
    async fn submit_failure() {
        let executor = TaskExecutor::new(4);
        let task: Arc<dyn AsyncTask> = Arc::new(FailTask { id: Uuid::new_v4() });
        let handle = executor.submit(task).unwrap();
        let result = handle.await.unwrap();
        assert!(result.is_err());

        let snap = executor.metrics().snapshot();
        assert_eq!(snap.failed, 1);
    }

    #[tokio::test]
    async fn submit_timeout() {
        let executor = TaskExecutor::new(4);
        let task: Arc<dyn AsyncTask> = Arc::new(SlowTask {
            id: Uuid::new_v4(),
            duration: Duration::from_secs(5),
        });
        let handle = executor.submit(task).unwrap();
        let result = handle.await.unwrap();
        assert!(matches!(result, Err(TaskError::Timeout(_))));
    }

    #[tokio::test]
    async fn concurrency_limit() {
        let executor = TaskExecutor::new(1);
        // Submit two tasks — only one can run at a time.
        let t1: Arc<dyn AsyncTask> = Arc::new(SuccessTask { id: Uuid::new_v4() });
        let t2: Arc<dyn AsyncTask> = Arc::new(SuccessTask { id: Uuid::new_v4() });

        let h1 = executor.submit(t1).unwrap();
        let h2 = executor.submit(t2).unwrap();

        let r1 = h1.await.unwrap().unwrap();
        let r2 = h2.await.unwrap().unwrap();
        assert_eq!(r1["result"], "ok");
        assert_eq!(r2["result"], "ok");

        let snap = executor.metrics().snapshot();
        assert_eq!(snap.total_submitted, 2);
        assert_eq!(snap.completed, 2);
    }

    #[tokio::test]
    async fn retry_exhausted() {
        let executor = TaskExecutor::new(4);
        let policy = RetryPolicy {
            max_attempts: 2,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            backoff_multiplier: 1.0,
        };
        let task: Arc<dyn AsyncTask> = Arc::new(FailTask { id: Uuid::new_v4() });
        let result = executor.execute_with_retry(task, &policy).await;
        assert!(matches!(result, Err(TaskError::RetryExhausted { .. })));
    }

    #[tokio::test]
    async fn circuit_breaker_rejects() {
        use crate::circuit_breaker::CircuitBreaker;

        let cb = Arc::new(CircuitBreaker::new(1, Duration::from_secs(60), 1));
        cb.record_failure(); // trip the breaker

        let executor = TaskExecutor::new(4).with_circuit_breaker(cb);
        let task: Arc<dyn AsyncTask> = Arc::new(SuccessTask { id: Uuid::new_v4() });
        let result = executor.submit(task);
        assert!(matches!(result, Err(TaskError::CircuitBreakerOpen)));

        let snap = executor.metrics().snapshot();
        assert_eq!(snap.circuit_breaker_trips, 1);
    }

    #[tokio::test]
    async fn error_strategy_builder() {
        let executor = TaskExecutor::new(4).with_error_strategy(ErrorStrategy::LogAndContinue);
        // Just verify it builds.
        let _ = format!("{executor:?}");
    }

    #[test]
    fn metrics_snapshot_serde() {
        let snap = TaskMetricsSnapshot {
            total_submitted: 10,
            completed: 8,
            failed: 2,
            currently_running: 0,
            panics_recovered: 1,
            circuit_breaker_trips: 0,
        };
        let json = serde_json::to_string(&snap).unwrap();
        let deserialized: TaskMetricsSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snap, deserialized);
    }
}
