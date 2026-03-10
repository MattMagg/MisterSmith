//! RuntimePerformanceMonitor for collecting Tokio runtime metrics.
//!
//! Uses the [`metrics`] crate macros (`gauge!`, `counter!`) to record
//! runtime statistics from the Tokio runtime handle. A downstream
//! exporter (e.g. `metrics-exporter-prometheus`) must be installed for
//! the recorded values to be observable.
//!
//! # Unstable metrics
//!
//! Many Tokio runtime metrics are gated behind `tokio_unstable`. When
//! the crate is compiled with `RUSTFLAGS="--cfg tokio_unstable"`, this
//! module records a richer set of per-worker and blocking-thread
//! metrics. Without that flag, only the stable subset is collected.

use metrics::SharedString;
use std::sync::OnceLock;
use tokio::runtime::Handle;

static WORKER_LABELS: OnceLock<Vec<SharedString>> = OnceLock::new();

fn get_worker_label(i: usize) -> SharedString {
    let labels =
        WORKER_LABELS.get_or_init(|| (0..1024).map(|idx| idx.to_string().into()).collect());
    if i < labels.len() {
        labels[i].clone()
    } else {
        i.to_string().into()
    }
}

/// Collects performance metrics from the Tokio runtime.
///
/// Wraps a [`Handle`] and reads its [`tokio::runtime::RuntimeMetrics`]
/// on each call to [`collect_metrics`](Self::collect_metrics), recording
/// the values through the `metrics` crate facade.
///
/// # Stable metric names
///
/// | Name | Type | Description |
/// |------|------|-------------|
/// | `runtime.workers_count` | gauge | Number of worker threads |
/// | `runtime.alive_tasks` | gauge | Currently alive tasks |
/// | `runtime.global_queue_depth` | gauge | Global injection queue depth |
///
/// # Metrics requiring `target_has_atomic = "64"` (most 64-bit targets)
///
/// | Name | Type | Description |
/// |------|------|-------------|
/// | `runtime.worker.busy_duration_secs` | gauge | Per-worker cumulative busy time (label: `worker`) |
/// | `runtime.worker.park_count` | counter | Per-worker park events (label: `worker`) |
///
/// # Metrics requiring `tokio_unstable`
///
/// | Name | Type | Description |
/// |------|------|-------------|
/// | `runtime.blocking_threads` | gauge | Active blocking threads |
/// | `runtime.idle_blocking_threads` | gauge | Idle blocking threads |
/// | `runtime.active_tasks` | gauge | Active tasks (unstable variant) |
/// | `runtime.injection_queue_depth` | gauge | Injection queue depth |
/// | `runtime.blocking_queue_depth` | gauge | Blocking task queue depth |
/// | `runtime.worker.local_queue_depth` | gauge | Per-worker local queue depth (label: `worker`) |
/// | `runtime.worker.noop_count` | counter | Per-worker no-op polls (label: `worker`) |
/// | `runtime.worker.steal_count` | counter | Per-worker task steals (label: `worker`) |
/// | `runtime.budget_forced_yield_count` | counter | Budget-forced yield events |
pub struct RuntimePerformanceMonitor {
    handle: Handle,
}

impl RuntimePerformanceMonitor {
    /// Create a new monitor for the given runtime handle.
    pub fn new(handle: Handle) -> Self {
        Self { handle }
    }

    /// Collect and record current runtime metrics.
    ///
    /// Reads a snapshot from the Tokio runtime and pushes every metric
    /// through the `metrics` facade. Counters use [`absolute`] so they
    /// reflect the runtime's monotonic totals regardless of collection
    /// cadence.
    ///
    /// [`absolute`]: metrics::Counter::absolute
    pub fn collect_metrics(&self) {
        let m = self.handle.metrics();

        // ----- Stable metrics (always available) -----
        metrics::gauge!("runtime.workers_count").set(m.num_workers() as f64);
        metrics::gauge!("runtime.alive_tasks").set(m.num_alive_tasks() as f64);
        metrics::gauge!("runtime.global_queue_depth").set(m.global_queue_depth() as f64);

        // ----- 64-bit atomic metrics (available on all 64-bit targets) -----
        #[cfg(target_has_atomic = "64")]
        {
            for i in 0..m.num_workers() {
                let worker = get_worker_label(i);

                metrics::gauge!(
                    "runtime.worker.busy_duration_secs",
                    "worker" => worker.clone()
                )
                .set(m.worker_total_busy_duration(i).as_secs_f64());

                metrics::counter!("runtime.worker.park_count", "worker" => worker)
                    .absolute(m.worker_park_count(i));
            }
        }

        // ----- Unstable metrics (require `--cfg tokio_unstable`) -----
        #[cfg(tokio_unstable)]
        self.collect_unstable_metrics(&m);

        tracing::trace!("Runtime metrics collected");
    }

    /// Collect the extended set of metrics gated behind `tokio_unstable`.
    #[cfg(tokio_unstable)]
    fn collect_unstable_metrics(&self, m: &tokio::runtime::RuntimeMetrics) {
        // Blocking thread pool
        metrics::gauge!("runtime.blocking_threads").set(m.num_blocking_threads() as f64);
        metrics::gauge!("runtime.idle_blocking_threads").set(m.num_idle_blocking_threads() as f64);
        metrics::gauge!("runtime.active_tasks").set(m.active_tasks_count() as f64);

        // Queue depths
        metrics::gauge!("runtime.injection_queue_depth").set(m.injection_queue_depth() as f64);
        metrics::gauge!("runtime.blocking_queue_depth").set(m.blocking_queue_depth() as f64);

        // Per-worker extended metrics
        for i in 0..m.num_workers() {
            let worker = get_worker_label(i);

            metrics::gauge!(
                "runtime.worker.local_queue_depth",
                "worker" => worker.clone()
            )
            .set(m.worker_local_queue_depth(i) as f64);

            metrics::counter!("runtime.worker.noop_count", "worker" => worker.clone())
                .absolute(m.worker_noop_count(i));

            metrics::counter!("runtime.worker.steal_count", "worker" => worker)
                .absolute(m.worker_steal_count(i));
        }

        // Budget forced yield
        metrics::counter!("runtime.budget_forced_yield_count")
            .absolute(m.budget_forced_yield_count());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn monitor_creation_and_collect() {
        let handle = Handle::current();
        let monitor = RuntimePerformanceMonitor::new(handle);
        // Without an installed metrics recorder the gauge!/counter! calls
        // are no-ops, but this validates that all method calls resolve and
        // the code does not panic.
        monitor.collect_metrics();
    }
}
