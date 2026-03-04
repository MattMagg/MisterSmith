//! MetricsRegistry with DashMap-based storage.
//!
//! Provides lock-free concurrent access to counters and gauges, suitable for
//! high-throughput instrumentation where contention on a single `RwLock` would
//! be a bottleneck.

use dashmap::DashMap;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::time::Duration;

// ---------------------------------------------------------------------------
// MetricsRegistry
// ---------------------------------------------------------------------------

/// Lock-free metrics registry backed by [`DashMap`].
///
/// Counters are stored as `AtomicU64`; gauges are stored as `AtomicI64`
/// holding the bit-pattern of an `f64` (via `f64::to_bits` / `from_bits`).
pub struct MetricsRegistry {
    /// Monotonic counters.
    counters: DashMap<String, AtomicU64>,
    /// Point-in-time gauges (stored as i64 bits of f64).
    gauges: DashMap<String, AtomicI64>,
}

impl MetricsRegistry {
    /// Create a new, empty registry.
    pub fn new() -> Self {
        Self {
            counters: DashMap::new(),
            gauges: DashMap::new(),
        }
    }

    // -- Counters -----------------------------------------------------------

    /// Increment a counter by `delta`. Creates the counter if it does not
    /// exist.
    pub fn increment_counter(&self, name: &str, delta: u64) {
        self.counters
            .entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(delta, Ordering::SeqCst);
    }

    /// Read the current value of a counter. Returns 0 if the counter has not
    /// been created.
    pub fn get_counter(&self, name: &str) -> u64 {
        self.counters
            .get(name)
            .map_or(0, |c| c.load(Ordering::SeqCst))
    }

    // -- Gauges -------------------------------------------------------------

    /// Set a gauge to `value`. Creates the gauge if it does not exist.
    ///
    /// The `f64` value is stored as its raw bit representation in an
    /// `AtomicI64` so that we can use lock-free atomic operations.
    pub fn set_gauge(&self, name: &str, value: f64) {
        let bits = value.to_bits() as i64;
        self.gauges
            .entry(name.to_string())
            .or_insert_with(|| AtomicI64::new(0))
            .store(bits, Ordering::SeqCst);
    }

    /// Read the current value of a gauge. Returns 0.0 if the gauge has not
    /// been created.
    pub fn get_gauge(&self, name: &str) -> f64 {
        self.gauges
            .get(name)
            .map_or(0.0, |g| f64::from_bits(g.load(Ordering::SeqCst) as u64))
    }

    /// Returns the number of registered counters.
    pub fn counter_count(&self) -> usize {
        self.counters.len()
    }

    /// Returns the number of registered gauges.
    pub fn gauge_count(&self) -> usize {
        self.gauges.len()
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// OverheadMonitor
// ---------------------------------------------------------------------------

/// Configuration for monitoring the overhead of metrics collection itself.
///
/// This is a data-only struct used to enforce that instrumentation stays within
/// acceptable performance bounds.
#[derive(Debug, Clone)]
pub struct OverheadMonitor {
    /// Maximum allowed duration for a single metrics collection pass.
    pub max_collection_time: Duration,
    /// Fraction of observations to actually record (0.0..=1.0).
    pub sampling_rate: f64,
    /// Number of metrics to collect per batch before yielding.
    pub batch_size: usize,
}

impl OverheadMonitor {
    /// Create a new `OverheadMonitor` with the given parameters.
    pub fn new(max_collection_time: Duration, sampling_rate: f64, batch_size: usize) -> Self {
        Self {
            max_collection_time,
            sampling_rate: sampling_rate.clamp(0.0, 1.0),
            batch_size,
        }
    }
}

impl Default for OverheadMonitor {
    fn default() -> Self {
        Self {
            max_collection_time: Duration::from_millis(100),
            sampling_rate: 1.0,
            batch_size: 1000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increment_counter_creates_and_increments() {
        let registry = MetricsRegistry::new();
        assert_eq!(registry.get_counter("requests"), 0);

        registry.increment_counter("requests", 1);
        assert_eq!(registry.get_counter("requests"), 1);

        registry.increment_counter("requests", 5);
        assert_eq!(registry.get_counter("requests"), 6);
    }

    #[test]
    fn set_and_get_gauge() {
        let registry = MetricsRegistry::new();
        assert!((registry.get_gauge("cpu") - 0.0).abs() < f64::EPSILON);

        registry.set_gauge("cpu", 0.75);
        assert!((registry.get_gauge("cpu") - 0.75).abs() < f64::EPSILON);

        registry.set_gauge("cpu", 0.50);
        assert!((registry.get_gauge("cpu") - 0.50).abs() < f64::EPSILON);
    }

    #[test]
    fn gauge_negative_values() {
        let registry = MetricsRegistry::new();
        registry.set_gauge("temp", -40.0);
        assert!((registry.get_gauge("temp") - (-40.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn multiple_counters() {
        let registry = MetricsRegistry::new();
        registry.increment_counter("a", 1);
        registry.increment_counter("b", 2);
        registry.increment_counter("a", 3);

        assert_eq!(registry.get_counter("a"), 4);
        assert_eq!(registry.get_counter("b"), 2);
        assert_eq!(registry.counter_count(), 2);
    }

    #[test]
    fn multiple_gauges() {
        let registry = MetricsRegistry::new();
        registry.set_gauge("x", 1.0);
        registry.set_gauge("y", 2.0);

        assert_eq!(registry.gauge_count(), 2);
    }

    #[test]
    fn default_registry() {
        let registry = MetricsRegistry::default();
        assert_eq!(registry.counter_count(), 0);
        assert_eq!(registry.gauge_count(), 0);
    }

    #[test]
    fn concurrent_counter_increments() {
        use std::sync::Arc;
        use std::thread;

        let registry = Arc::new(MetricsRegistry::new());
        let mut handles = Vec::new();

        for _ in 0..10 {
            let reg = Arc::clone(&registry);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    reg.increment_counter("concurrent", 1);
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(registry.get_counter("concurrent"), 1000);
    }

    #[test]
    fn overhead_monitor_defaults() {
        let om = OverheadMonitor::default();
        assert_eq!(om.max_collection_time, Duration::from_millis(100));
        assert!((om.sampling_rate - 1.0).abs() < f64::EPSILON);
        assert_eq!(om.batch_size, 1000);
    }

    #[test]
    fn overhead_monitor_clamps_sampling_rate() {
        let om = OverheadMonitor::new(Duration::from_millis(50), 2.0, 500);
        assert!((om.sampling_rate - 1.0).abs() < f64::EPSILON);

        let om2 = OverheadMonitor::new(Duration::from_millis(50), -1.0, 500);
        assert!((om2.sampling_rate - 0.0).abs() < f64::EPSILON);
    }
}
