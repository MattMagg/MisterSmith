#![deny(missing_docs, unsafe_code)]

//! Reusable async building blocks for the Mister Smith multi-agent orchestration framework.
//!
//! This crate provides the fundamental async primitives used throughout the system:
//!
//! - **task** — [`TaskPriority`], [`TaskError`], and the [`AsyncTask`] trait.
//! - **retry** — [`RetryPolicy`] with configurable exponential backoff and jitter.
//! - **circuit_breaker** — [`CircuitBreaker`] with Closed/Open/HalfOpen state machine.
//! - **executor** — [`TaskExecutor`] with semaphore-based concurrency control, metrics, and retry support.
//! - **stream** — [`StreamProcessor`] pipeline with backpressure configuration.
//! - **sync** — [`DeadlockPreventingMutex`], [`AsyncBarrier`], [`CountdownLatch`].
//! - **guard** — [`TaskGuard`] RAII wrapper for task lifecycle management.

pub mod circuit_breaker;
pub mod executor;
pub mod guard;
pub mod retry;
pub mod stream;
pub mod sync;
pub mod task;

// Re-export key types at crate root for convenience.

// Task types
pub use task::{AsyncTask, TaskError, TaskPriority};

// Retry
pub use retry::RetryPolicy;

// Circuit breaker
pub use circuit_breaker::{CircuitBreaker, CircuitState};

// Executor
pub use executor::{ErrorStrategy, TaskExecutor, TaskMetrics, TaskMetricsSnapshot};

// Stream processing
pub use stream::{
    BackpressureConfig, BackpressureStrategy, Processor, StreamMetrics, StreamProcessor,
};

// Synchronization
pub use sync::{AsyncBarrier, CountdownLatch, DeadlockPreventingMutex};

// Guard
pub use guard::TaskGuard;
