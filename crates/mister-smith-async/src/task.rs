//! Task types, priorities, and the [`AsyncTask`] trait.
//!
//! [`TaskPriority`] mirrors the ordering semantics of
//! [`MessagePriority`](mister_smith_core::MessagePriority) — lower discriminant
//! values represent higher priority (Critical = 0 is the highest).

use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Task priority levels with explicit discriminants.
///
/// Lower discriminant values represent higher priority.
/// `Critical` (0) is highest; `Low` (3) is lowest.
///
/// This mirrors [`MessagePriority`](mister_smith_core::MessagePriority) but
/// omits the `Bulk` level, which is not applicable to discrete tasks.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum TaskPriority {
    /// Highest priority — system-critical tasks.
    Critical = 0,
    /// High priority — time-sensitive tasks.
    High = 1,
    /// Normal priority — standard task processing.
    #[default]
    Normal = 2,
    /// Low priority — background tasks.
    Low = 3,
}

/// Errors that can occur during task execution.
#[derive(Debug, Error, Clone)]
pub enum TaskError {
    /// Task execution failed with a descriptive message.
    #[error("Task execution failed: {0}")]
    ExecutionFailed(String),
    /// Task exceeded its configured timeout.
    #[error("Task timed out: {0}")]
    Timeout(String),
    /// Task was cancelled before completion.
    #[error("Task cancelled: {0}")]
    Cancelled(String),
    /// All retry attempts exhausted.
    #[error("Retry exhausted after {attempts} attempts: {last_error}")]
    RetryExhausted {
        /// Number of attempts made.
        attempts: u32,
        /// Error from the final attempt.
        last_error: String,
    },
    /// Circuit breaker is open — calls are being rejected.
    #[error("Circuit breaker is open")]
    CircuitBreakerOpen,
    /// Task panicked and the panic was recovered.
    #[error("Panic recovered: {0}")]
    PanicRecovered(String),
    /// Failed to spawn the task onto the runtime.
    #[error("Spawn failed: {0}")]
    SpawnFailed(String),
}

/// Trait for asynchronous, priority-aware tasks.
///
/// Implementors define an [`execute`](AsyncTask::execute) method that returns a
/// JSON value on success. Priority, timeout, and identity have sensible defaults
/// but can be overridden.
#[async_trait]
pub trait AsyncTask: Send + Sync {
    /// Execute the task, returning a JSON result on success.
    async fn execute(&self) -> Result<serde_json::Value, TaskError>;

    /// The priority of this task. Defaults to [`TaskPriority::Normal`].
    fn priority(&self) -> TaskPriority {
        TaskPriority::Normal
    }

    /// Maximum duration this task is allowed to run. Defaults to 30 seconds.
    fn timeout(&self) -> Duration {
        Duration::from_secs(30)
    }

    /// Unique identifier for this task instance.
    fn task_id(&self) -> Uuid;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn task_priority_discriminants() {
        assert_eq!(TaskPriority::Critical as u8, 0);
        assert_eq!(TaskPriority::High as u8, 1);
        assert_eq!(TaskPriority::Normal as u8, 2);
        assert_eq!(TaskPriority::Low as u8, 3);
    }

    #[test]
    fn task_priority_default() {
        assert_eq!(TaskPriority::default(), TaskPriority::Normal);
    }

    #[test]
    fn task_priority_ordering() {
        // Lower discriminant = higher priority, and PartialOrd/Ord derive on
        // repr(u8) enums orders by discriminant.
        assert!(TaskPriority::Critical < TaskPriority::High);
        assert!(TaskPriority::High < TaskPriority::Normal);
        assert!(TaskPriority::Normal < TaskPriority::Low);
    }

    #[test]
    fn task_priority_serde_roundtrip() {
        let p = TaskPriority::High;
        let json_str = serde_json::to_string(&p).unwrap();
        let deserialized: TaskPriority = serde_json::from_str(&json_str).unwrap();
        assert_eq!(p, deserialized);
    }

    #[test]
    fn task_error_display() {
        let err = TaskError::ExecutionFailed("something broke".into());
        assert!(err.to_string().contains("something broke"));

        let err = TaskError::RetryExhausted {
            attempts: 3,
            last_error: "timeout".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("3 attempts"));
        assert!(msg.contains("timeout"));
    }

    struct DummyTask {
        id: Uuid,
    }

    #[async_trait]
    impl AsyncTask for DummyTask {
        async fn execute(&self) -> Result<serde_json::Value, TaskError> {
            Ok(json!({"status": "ok"}))
        }

        fn task_id(&self) -> Uuid {
            self.id
        }
    }

    #[tokio::test]
    async fn dummy_task_defaults() {
        let task = DummyTask { id: Uuid::new_v4() };
        assert_eq!(task.priority(), TaskPriority::Normal);
        assert_eq!(task.timeout(), Duration::from_secs(30));

        let result = task.execute().await.unwrap();
        assert_eq!(result["status"], "ok");
    }
}
