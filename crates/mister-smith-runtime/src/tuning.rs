//! Runtime tuning presets for different workload profiles.
//!
//! This module provides:
//!
//! * [`WorkloadType`] — a classification enum used to select tuning parameters.
//! * [`RuntimeBestPractices`] — static helper methods that compute optimal
//!   thread counts and other parameters based on workload type.
//! * [`RuntimeTuning`] — named preset constructors for common deployment
//!   scenarios (WebSocket servers, data pipelines, agent systems).

use mister_smith_config::RuntimeConfig;
use serde::{Deserialize, Serialize};

use crate::config::RuntimeConfigExt;

// ---------------------------------------------------------------------------
// Workload classification
// ---------------------------------------------------------------------------

/// Workload-type classification used to drive runtime optimisation decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkloadType {
    /// CPU-intensive computation (model inference, crypto, compression).
    CpuBound,
    /// I/O-intensive operations (network calls, disk reads).
    IoBound,
    /// Mixed CPU and I/O workload.
    Mixed,
    /// High-throughput message processing (NATS streams, event buses).
    HighThroughput,
}

impl std::fmt::Display for WorkloadType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkloadType::CpuBound => write!(f, "CpuBound"),
            WorkloadType::IoBound => write!(f, "IoBound"),
            WorkloadType::Mixed => write!(f, "Mixed"),
            WorkloadType::HighThroughput => write!(f, "HighThroughput"),
        }
    }
}

// ---------------------------------------------------------------------------
// Best-practices calculations
// ---------------------------------------------------------------------------

/// Static helper methods that compute optimal runtime parameters based on
/// the hardware profile and workload classification.
pub struct RuntimeBestPractices;

impl RuntimeBestPractices {
    /// Returns the recommended number of Tokio worker threads for `workload`.
    ///
    /// | Workload | Formula |
    /// |----------|---------|
    /// | `CpuBound` | `num_cpus` |
    /// | `IoBound` | `num_cpus * 2` |
    /// | `Mixed` | `num_cpus * 1.5` (rounded down) |
    /// | `HighThroughput` | `num_cpus` |
    pub fn optimal_worker_threads(workload: WorkloadType) -> usize {
        let cpus = num_cpus::get();
        match workload {
            WorkloadType::CpuBound => cpus,
            WorkloadType::IoBound => cpus * 2,
            WorkloadType::Mixed => (cpus as f64 * 1.5) as usize,
            WorkloadType::HighThroughput => cpus,
        }
    }

    /// Returns the recommended maximum blocking-thread count for `workload`.
    ///
    /// CPU-bound profiles use a lower ceiling to avoid thread over-subscription.
    pub fn optimal_blocking_threads(workload: WorkloadType) -> usize {
        match workload {
            WorkloadType::CpuBound => 64,
            WorkloadType::IoBound => 512,
            WorkloadType::Mixed => 256,
            WorkloadType::HighThroughput => 512,
        }
    }

    /// Returns the recommended thread stack size in bytes.
    ///
    /// High-throughput workloads receive a larger stack (4 MiB) to accommodate
    /// deep call chains common in pipeline processing.
    pub fn optimal_stack_size(workload: WorkloadType) -> usize {
        match workload {
            WorkloadType::HighThroughput => 4 * 1024 * 1024, // 4 MiB
            _ => 2 * 1024 * 1024,                            // 2 MiB
        }
    }
}

// ---------------------------------------------------------------------------
// Named deployment presets
// ---------------------------------------------------------------------------

/// Named runtime presets for common deployment scenarios.
///
/// Each method returns a [`RuntimeConfig`] that has been tuned for the
/// scenario's expected workload profile.
pub struct RuntimeTuning;

impl RuntimeTuning {
    /// Configuration tuned for WebSocket/long-poll server workloads.
    ///
    /// Delegates to [`RuntimeConfigExt::io_bound`] — many concurrent
    /// connections, each performing lightweight I/O.
    pub fn websocket_server() -> RuntimeConfig {
        RuntimeConfig::io_bound()
    }

    /// Configuration tuned for data-pipeline / stream-processing workloads.
    ///
    /// Delegates to [`RuntimeConfigExt::high_throughput`] — high message
    /// volume with larger stack requirements.
    pub fn data_pipeline() -> RuntimeConfig {
        RuntimeConfig::high_throughput()
    }

    /// Configuration tuned for multi-agent orchestration workloads.
    ///
    /// Delegates to [`RuntimeConfigExt::cpu_bound`] — inference and
    /// orchestration logic are CPU-intensive.
    pub fn agent_system() -> RuntimeConfig {
        RuntimeConfig::cpu_bound()
    }

    /// Build a [`RuntimeConfig`] from a [`WorkloadType`] classification.
    ///
    /// This is useful when the workload type is determined dynamically
    /// (e.g., via configuration or auto-detection).
    pub fn from_workload(workload: WorkloadType) -> RuntimeConfig {
        match workload {
            WorkloadType::CpuBound => RuntimeConfig::cpu_bound(),
            WorkloadType::IoBound => RuntimeConfig::io_bound(),
            WorkloadType::Mixed => {
                // Mixed uses a blend: 1.5x CPU count with moderate blocking.
                RuntimeConfig {
                    worker_threads: Some(RuntimeBestPractices::optimal_worker_threads(workload)),
                    blocking_threads: RuntimeBestPractices::optimal_blocking_threads(workload),
                    thread_stack_size: Some(RuntimeBestPractices::optimal_stack_size(workload)),
                    enable_all: true,
                    enable_time: true,
                    enable_io: true,
                    ..RuntimeConfig::default()
                }
            }
            WorkloadType::HighThroughput => RuntimeConfig::high_throughput(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimal_workers_cpu_bound_equals_cpus() {
        let cpus = num_cpus::get();
        assert_eq!(
            RuntimeBestPractices::optimal_worker_threads(WorkloadType::CpuBound),
            cpus
        );
    }

    #[test]
    fn optimal_workers_io_bound_doubles_cpus() {
        let cpus = num_cpus::get();
        assert_eq!(
            RuntimeBestPractices::optimal_worker_threads(WorkloadType::IoBound),
            cpus * 2
        );
    }

    #[test]
    fn optimal_workers_mixed_is_between() {
        let cpus = num_cpus::get();
        let mixed = RuntimeBestPractices::optimal_worker_threads(WorkloadType::Mixed);
        assert!(mixed >= cpus, "mixed ({mixed}) should be >= cpus ({cpus})");
        assert!(
            mixed <= cpus * 2,
            "mixed ({mixed}) should be <= cpus*2 ({})",
            cpus * 2
        );
    }

    #[test]
    fn optimal_blocking_threads_cpu_bound_is_low() {
        assert_eq!(
            RuntimeBestPractices::optimal_blocking_threads(WorkloadType::CpuBound),
            64
        );
    }

    #[test]
    fn optimal_stack_high_throughput_is_4mib() {
        assert_eq!(
            RuntimeBestPractices::optimal_stack_size(WorkloadType::HighThroughput),
            4 * 1024 * 1024
        );
    }

    #[test]
    fn websocket_server_preset_is_io_bound() {
        let ws = RuntimeTuning::websocket_server();
        let io = RuntimeConfig::io_bound();
        assert_eq!(ws.worker_threads, io.worker_threads);
        assert_eq!(ws.blocking_threads, io.blocking_threads);
    }

    #[test]
    fn agent_system_preset_is_cpu_bound() {
        let agent = RuntimeTuning::agent_system();
        let cpu = RuntimeConfig::cpu_bound();
        assert_eq!(agent.worker_threads, cpu.worker_threads);
        assert_eq!(agent.blocking_threads, cpu.blocking_threads);
    }

    #[test]
    fn from_workload_round_trips_all_variants() {
        for workload in [
            WorkloadType::CpuBound,
            WorkloadType::IoBound,
            WorkloadType::Mixed,
            WorkloadType::HighThroughput,
        ] {
            let cfg = RuntimeTuning::from_workload(workload);
            // Every preset must produce a non-zero worker count.
            assert!(
                cfg.worker_threads.unwrap_or(1) > 0,
                "workload {workload} produced zero workers"
            );
        }
    }

    #[test]
    fn workload_type_display() {
        assert_eq!(WorkloadType::CpuBound.to_string(), "CpuBound");
        assert_eq!(WorkloadType::IoBound.to_string(), "IoBound");
        assert_eq!(WorkloadType::Mixed.to_string(), "Mixed");
        assert_eq!(WorkloadType::HighThroughput.to_string(), "HighThroughput");
    }

    #[test]
    fn workload_type_serde_round_trip() {
        for workload in [
            WorkloadType::CpuBound,
            WorkloadType::IoBound,
            WorkloadType::Mixed,
            WorkloadType::HighThroughput,
        ] {
            let json = serde_json::to_string(&workload).expect("serialize");
            let back: WorkloadType = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(workload, back);
        }
    }
}
