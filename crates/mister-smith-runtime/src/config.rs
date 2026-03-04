//! Runtime configuration presets and `build_runtime` extension.
//!
//! [`RuntimeConfig`] is defined in `mister-smith-config`. This module adds
//! preset constructors and the ability to materialise a Tokio [`Runtime`]
//! from a configuration value via the [`RuntimeConfigExt`] extension trait.

use std::time::Duration;

use mister_smith_config::RuntimeConfig;
use tokio::runtime::Runtime;

use crate::error::RuntimeError;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default worker thread count (number of logical CPUs).
/// A value of `0` is a sentinel meaning "detect at runtime via `num_cpus::get()`".
pub const DEFAULT_WORKER_THREADS: usize = 0;

/// Default maximum number of blocking threads.
pub const DEFAULT_MAX_BLOCKING_THREADS: usize = 512;

/// Default keep-alive duration for idle worker threads.
pub const DEFAULT_THREAD_KEEP_ALIVE: Duration = Duration::from_secs(60);

/// Default thread stack size (2 MiB).
pub const DEFAULT_THREAD_STACK_SIZE: usize = 2 * 1024 * 1024;

/// Default graceful-shutdown timeout.
pub const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

// ---------------------------------------------------------------------------
// Extension trait
// ---------------------------------------------------------------------------

/// Extension methods on [`RuntimeConfig`] that add preset constructors
/// and the ability to build a Tokio [`Runtime`].
pub trait RuntimeConfigExt {
    /// Create a configuration tuned for CPU-bound workloads.
    ///
    /// * Worker threads = physical CPU count
    /// * Low blocking-thread ceiling (64)
    /// * Short keep-alive (30 s)
    fn cpu_bound() -> RuntimeConfig;

    /// Create a configuration tuned for I/O-bound workloads.
    ///
    /// * Worker threads = 2 x CPU count
    /// * High blocking-thread ceiling (512)
    /// * Longer keep-alive (120 s)
    fn io_bound() -> RuntimeConfig;

    /// Create a configuration tuned for high-throughput message processing.
    ///
    /// * Worker threads = CPU count
    /// * High blocking-thread ceiling (512)
    /// * Larger stack size (4 MiB)
    fn high_throughput() -> RuntimeConfig;

    /// Build a Tokio [`Runtime`] from this configuration.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::BuildFailed`] if the underlying
    /// `tokio::runtime::Builder::build()` call fails.
    fn build_runtime(&self) -> Result<Runtime, RuntimeError>;
}

impl RuntimeConfigExt for RuntimeConfig {
    fn cpu_bound() -> RuntimeConfig {
        RuntimeConfig {
            worker_threads: Some(num_cpus::get()),
            blocking_threads: 64,
            thread_keep_alive: Duration::from_secs(30),
            thread_stack_size: Some(DEFAULT_THREAD_STACK_SIZE),
            enable_all: true,
            enable_time: true,
            enable_io: true,
            ..RuntimeConfig::default()
        }
    }

    fn io_bound() -> RuntimeConfig {
        RuntimeConfig {
            worker_threads: Some(num_cpus::get() * 2),
            blocking_threads: DEFAULT_MAX_BLOCKING_THREADS,
            thread_keep_alive: Duration::from_secs(120),
            thread_stack_size: Some(DEFAULT_THREAD_STACK_SIZE),
            enable_all: true,
            enable_time: true,
            enable_io: true,
            ..RuntimeConfig::default()
        }
    }

    fn high_throughput() -> RuntimeConfig {
        RuntimeConfig {
            worker_threads: Some(num_cpus::get()),
            blocking_threads: DEFAULT_MAX_BLOCKING_THREADS,
            thread_keep_alive: Duration::from_secs(60),
            thread_stack_size: Some(4 * 1024 * 1024), // 4 MiB
            enable_all: true,
            enable_time: true,
            enable_io: true,
            ..RuntimeConfig::default()
        }
    }

    fn build_runtime(&self) -> Result<Runtime, RuntimeError> {
        let mut builder = tokio::runtime::Builder::new_multi_thread();

        let workers = self.worker_threads.unwrap_or_else(num_cpus::get);
        builder.worker_threads(workers);
        builder.max_blocking_threads(self.blocking_threads);
        builder.thread_keep_alive(self.thread_keep_alive);

        if let Some(stack_size) = self.thread_stack_size {
            builder.thread_stack_size(stack_size);
        }

        if self.enable_all {
            builder.enable_all();
        } else {
            if self.enable_time {
                builder.enable_time();
            }
            if self.enable_io {
                builder.enable_io();
            }
        }

        // RuntimeError::BuildFailed has `#[from] std::io::Error`,
        // so the `?` operator converts automatically.
        Ok(builder.build()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_bound_preset_has_correct_blocking_ceiling() {
        let cfg = RuntimeConfig::cpu_bound();
        assert_eq!(cfg.blocking_threads, 64);
        assert_eq!(cfg.thread_keep_alive, Duration::from_secs(30));
        assert!(cfg.enable_all);
    }

    #[test]
    fn io_bound_preset_doubles_workers() {
        let cfg = RuntimeConfig::io_bound();
        let cpus = num_cpus::get();
        assert_eq!(cfg.worker_threads, Some(cpus * 2));
        assert_eq!(cfg.blocking_threads, DEFAULT_MAX_BLOCKING_THREADS);
        assert_eq!(cfg.thread_keep_alive, Duration::from_secs(120));
    }

    #[test]
    fn high_throughput_preset_uses_large_stack() {
        let cfg = RuntimeConfig::high_throughput();
        assert_eq!(cfg.thread_stack_size, Some(4 * 1024 * 1024));
    }

    #[test]
    fn build_runtime_from_default_config() {
        let cfg = RuntimeConfig::default();
        let rt = cfg.build_runtime().expect("default config should build");
        // Smoke-test: spawn a trivial task and verify the runtime works.
        rt.block_on(async { 1 + 1 });
    }

    #[test]
    fn build_runtime_from_cpu_bound_preset() {
        let cfg = RuntimeConfig::cpu_bound();
        let rt = cfg.build_runtime().expect("cpu_bound config should build");
        let val = rt.block_on(async { 42 });
        assert_eq!(val, 42);
    }

    #[test]
    fn build_runtime_selective_features() {
        let cfg = RuntimeConfig {
            enable_all: false,
            enable_time: true,
            enable_io: false,
            ..RuntimeConfig::default()
        };
        // Should still build — time-only is a valid configuration.
        let _rt = cfg.build_runtime().expect("time-only config should build");
    }
}
