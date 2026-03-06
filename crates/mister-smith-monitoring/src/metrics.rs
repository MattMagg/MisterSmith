//! MetricsCollector and MetricsBackend trait.
//!
//! Provides buffered metrics collection with pluggable backends. Metrics are
//! accumulated in memory and periodically flushed to registered backends
//! (e.g., Prometheus, StatsD, logging).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

// ---------------------------------------------------------------------------
// Metric value types
// ---------------------------------------------------------------------------

/// The value of a single metric observation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    /// Monotonically increasing counter.
    Counter(u64),
    /// Point-in-time gauge.
    Gauge(f64),
    /// Histogram observation (a single sample).
    Histogram(f64),
    /// Pre-computed quantile summary.
    Summary {
        /// (quantile, value) pairs, e.g. (0.99, 42.0).
        quantiles: Vec<(f64, f64)>,
    },
}

/// A single metric observation with name, value, timestamp, and tags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name (e.g. `http_requests_total`).
    pub name: String,
    /// Observed value.
    pub value: MetricValue,
    /// When the observation was recorded.
    pub timestamp: SystemTime,
    /// Dimensional tags (e.g. `method=GET`, `status=200`).
    pub tags: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// MetricsBackend trait
// ---------------------------------------------------------------------------

/// Async trait for backends that receive flushed metrics.
#[async_trait]
pub trait MetricsBackend: Send + Sync + 'static {
    /// Send a batch of metrics to the backend.
    async fn send_metrics(
        &self,
        metrics: Vec<Metric>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

// ---------------------------------------------------------------------------
// MetricsCollector
// ---------------------------------------------------------------------------

/// Buffers metric observations and periodically flushes them to backends.
pub struct MetricsCollector {
    /// Buffered metrics keyed by metric name.
    buffer: RwLock<HashMap<String, Vec<Metric>>>,
    /// How often to flush buffered metrics.
    flush_interval: Duration,
    /// Registered output backends.
    backends: RwLock<Vec<Arc<dyn MetricsBackend>>>,
}

impl MetricsCollector {
    /// Create a new `MetricsCollector` with the given flush interval.
    pub fn new(flush_interval: Duration) -> Self {
        Self {
            buffer: RwLock::new(HashMap::new()),
            flush_interval,
            backends: RwLock::new(Vec::new()),
        }
    }

    /// Register a metrics backend.
    pub async fn add_backend(&self, backend: Arc<dyn MetricsBackend>) {
        let mut backends = self.backends.write().await;
        backends.push(backend);
    }

    // -- Recording helpers --------------------------------------------------

    /// Increment a counter metric.
    pub async fn increment_counter(&self, name: &str, tags: HashMap<String, String>) {
        let metric = Metric {
            name: name.to_string(),
            value: MetricValue::Counter(1),
            timestamp: SystemTime::now(),
            tags,
        };
        self.push_metric(metric).await;
    }

    /// Set a gauge metric.
    pub async fn set_gauge(&self, name: &str, value: f64, tags: HashMap<String, String>) {
        let metric = Metric {
            name: name.to_string(),
            value: MetricValue::Gauge(value),
            timestamp: SystemTime::now(),
            tags,
        };
        self.push_metric(metric).await;
    }

    /// Record a histogram observation.
    pub async fn record_histogram(&self, name: &str, value: f64, tags: HashMap<String, String>) {
        let metric = Metric {
            name: name.to_string(),
            value: MetricValue::Histogram(value),
            timestamp: SystemTime::now(),
            tags,
        };
        self.push_metric(metric).await;
    }

    /// Convenience: increment the `events_published_total` counter.
    pub async fn record_event_published(&self) {
        self.increment_counter("events_published_total", HashMap::new())
            .await;
    }

    /// Convenience: increment the `handler_errors_total` counter.
    pub async fn record_handler_error(&self) {
        self.increment_counter("handler_errors_total", HashMap::new())
            .await;
    }

    // -- Flush loop ---------------------------------------------------------

    /// Run the periodic flush loop until `shutdown` is set.
    pub async fn run(&self, shutdown: Arc<AtomicBool>) {
        info!(
            interval_ms = self.flush_interval.as_millis() as u64,
            "Metrics collector started"
        );

        while !shutdown.load(Ordering::SeqCst) {
            tokio::select! {
                _ = tokio::time::sleep(self.flush_interval) => {
                    self.flush().await;
                }
                _ = async {
                    while !shutdown.load(Ordering::SeqCst) {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                } => {
                    break;
                }
            }
        }

        // Final flush before stopping.
        self.flush().await;
        info!("Metrics collector stopped");
    }

    /// Flush all buffered metrics to every registered backend, then clear
    /// the buffer.
    pub async fn flush(&self) {
        let metrics = {
            let mut buffer = self.buffer.write().await;
            let all: Vec<Metric> = buffer.values().flatten().cloned().collect();
            buffer.clear();
            all
        };

        if metrics.is_empty() {
            return;
        }

        debug!(count = metrics.len(), "Flushing metrics to backends");

        let backends = self.backends.read().await;
        for backend in backends.iter() {
            if let Err(e) = backend.send_metrics(metrics.clone()).await {
                error!(error = %e, "Failed to send metrics to backend");
            }
        }
    }

    // -- Internal -----------------------------------------------------------

    async fn push_metric(&self, metric: Metric) {
        let mut buffer = self.buffer.write().await;
        buffer.entry(metric.name.clone()).or_default().push(metric);
    }

    /// Returns the number of buffered metric observations (across all names).
    pub async fn buffered_count(&self) -> usize {
        let buffer = self.buffer.read().await;
        buffer.values().map(|v| v.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// A test backend that captures flushed metrics.
    struct CapturingBackend {
        captured: Mutex<Vec<Metric>>,
    }

    impl CapturingBackend {
        fn new() -> Self {
            Self {
                captured: Mutex::new(Vec::new()),
            }
        }

        fn captured_count(&self) -> usize {
            self.captured.lock().unwrap().len()
        }
    }

    #[async_trait]
    impl MetricsBackend for CapturingBackend {
        async fn send_metrics(
            &self,
            metrics: Vec<Metric>,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let mut captured = self.captured.lock().unwrap();
            captured.extend(metrics);
            Ok(())
        }
    }

    /// A backend that always fails.
    struct FailingBackend;

    #[async_trait]
    impl MetricsBackend for FailingBackend {
        async fn send_metrics(
            &self,
            _metrics: Vec<Metric>,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Err("backend unavailable".into())
        }
    }

    #[tokio::test]
    async fn increment_counter_buffers() {
        let collector = MetricsCollector::new(Duration::from_secs(60));
        collector
            .increment_counter("requests", HashMap::new())
            .await;
        assert_eq!(collector.buffered_count().await, 1);
    }

    #[tokio::test]
    async fn set_gauge_buffers() {
        let collector = MetricsCollector::new(Duration::from_secs(60));
        collector.set_gauge("cpu_usage", 0.75, HashMap::new()).await;
        assert_eq!(collector.buffered_count().await, 1);
    }

    #[tokio::test]
    async fn record_histogram_buffers() {
        let collector = MetricsCollector::new(Duration::from_secs(60));
        collector
            .record_histogram("latency", 42.5, HashMap::new())
            .await;
        assert_eq!(collector.buffered_count().await, 1);
    }

    #[tokio::test]
    async fn flush_sends_to_backend() {
        let collector = MetricsCollector::new(Duration::from_secs(60));
        let backend = Arc::new(CapturingBackend::new());
        collector
            .add_backend(Arc::clone(&backend) as Arc<dyn MetricsBackend>)
            .await;

        collector.increment_counter("a", HashMap::new()).await;
        collector.set_gauge("b", 1.0, HashMap::new()).await;

        collector.flush().await;

        assert_eq!(backend.captured_count(), 2);
        assert_eq!(collector.buffered_count().await, 0);
    }

    #[tokio::test]
    async fn flush_empty_buffer_is_noop() {
        let collector = MetricsCollector::new(Duration::from_secs(60));
        let backend = Arc::new(CapturingBackend::new());
        collector
            .add_backend(Arc::clone(&backend) as Arc<dyn MetricsBackend>)
            .await;

        collector.flush().await;
        assert_eq!(backend.captured_count(), 0);
    }

    #[tokio::test]
    async fn failing_backend_does_not_panic() {
        let collector = MetricsCollector::new(Duration::from_secs(60));
        collector
            .add_backend(Arc::new(FailingBackend) as Arc<dyn MetricsBackend>)
            .await;

        collector.increment_counter("x", HashMap::new()).await;

        // Should log an error but not panic.
        collector.flush().await;
    }

    #[tokio::test]
    async fn record_event_published() {
        let collector = MetricsCollector::new(Duration::from_secs(60));
        collector.record_event_published().await;
        assert_eq!(collector.buffered_count().await, 1);
    }

    #[tokio::test]
    async fn record_handler_error() {
        let collector = MetricsCollector::new(Duration::from_secs(60));
        collector.record_handler_error().await;
        assert_eq!(collector.buffered_count().await, 1);
    }

    #[tokio::test]
    async fn metrics_have_tags() {
        let collector = MetricsCollector::new(Duration::from_secs(60));
        let backend = Arc::new(CapturingBackend::new());
        collector
            .add_backend(Arc::clone(&backend) as Arc<dyn MetricsBackend>)
            .await;

        let mut tags = HashMap::new();
        tags.insert("method".to_string(), "GET".to_string());
        collector.increment_counter("http_requests", tags).await;

        collector.flush().await;

        let captured = backend.captured.lock().unwrap();
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].tags.get("method").unwrap(), "GET");
    }

    #[tokio::test]
    async fn run_loop_stops_on_shutdown() {
        let collector = Arc::new(MetricsCollector::new(Duration::from_millis(50)));
        let shutdown = Arc::new(AtomicBool::new(false));

        let collector_clone = Arc::clone(&collector);
        let shutdown_clone = Arc::clone(&shutdown);
        let handle = tokio::spawn(async move {
            collector_clone.run(shutdown_clone).await;
        });

        tokio::time::sleep(Duration::from_millis(150)).await;
        shutdown.store(true, Ordering::SeqCst);

        tokio::time::timeout(Duration::from_secs(2), handle)
            .await
            .expect("collector should stop within timeout")
            .expect("collector task should not panic");
    }
}
