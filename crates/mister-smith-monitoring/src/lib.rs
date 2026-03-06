//! Health check registration, metrics collection, failure detection, and observability.
//!
//! This crate provides the monitoring subsystem for the Mister Smith framework:
//!
//! - **types** — `ComponentId`, `Status`, `HealthStatus` data types.
//! - **health** — `HealthCheck` trait, `HealthMonitor`, `RuntimeHealthCheck`.
//! - **failure_detector** — Phi accrual failure detection.
//! - **metrics** — Buffered `MetricsCollector` with pluggable `MetricsBackend`s.
//! - **registry** — Lock-free `MetricsRegistry` backed by `DashMap`.
//! - **system** — `MonitoringSystem` coordinator.

pub mod failure_detector;
pub mod health;
pub mod metrics;
pub mod prometheus;
pub mod registry;
pub mod system;
pub mod types;

// Re-export key types at crate root for convenience.
pub use failure_detector::PhiAccrualFailureDetector;
pub use health::{HealthCheck, HealthMonitor, RuntimeHealthCheck};
pub use metrics::{Metric, MetricValue, MetricsBackend, MetricsCollector};
pub use registry::{MetricsRegistry, OverheadMonitor};
pub use prometheus::PrometheusBackend;
pub use system::MonitoringSystem;
pub use types::{ComponentId, HealthStatus, Status};
