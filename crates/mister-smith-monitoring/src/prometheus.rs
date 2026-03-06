//! Prometheus metrics backend.
//!
//! Implements `MetricsBackend` by forwarding flushed `Metric` observations to
//! the global `metrics` crate recorder, which is expected to be a
//! `metrics-exporter-prometheus` instance installed during observability init.
//!
//! Standard Mister Smith framework metrics:
//! - Counters: `mistersmith_messages_sent_total`, `mistersmith_messages_received_total`,
//!   `mistersmith_tasks_completed_total`, `mistersmith_tasks_failed_total`,
//!   `mistersmith_agent_restarts_total`
//! - Gauges: `mistersmith_active_agents`, `mistersmith_message_queue_depth`
//! - Histograms: `mistersmith_task_duration_seconds`, `mistersmith_message_latency_seconds`,
//!   `mistersmith_health_check_duration_seconds`

use async_trait::async_trait;

use crate::metrics::{Metric, MetricValue, MetricsBackend};

/// A `MetricsBackend` that forwards observations to the `metrics` crate.
///
/// The `metrics` crate uses a global recorder — when
/// `metrics-exporter-prometheus` is installed, all counter/gauge/histogram
/// calls are automatically exposed via the Prometheus text format.
pub struct PrometheusBackend;

impl PrometheusBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PrometheusBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MetricsBackend for PrometheusBackend {
    async fn send_metrics(
        &self,
        metrics: Vec<Metric>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for metric in metrics {
            let labels: Vec<(String, String)> = metric.tags.into_iter().collect();
            let name = metric.name.clone();
            match metric.value {
                MetricValue::Counter(val) => {
                    metrics::counter!(name, &labels).absolute(val);
                }
                MetricValue::Gauge(val) => {
                    metrics::gauge!(name, &labels).set(val);
                }
                MetricValue::Histogram(val) => {
                    metrics::histogram!(name, &labels).record(val);
                }
                MetricValue::Summary { quantiles } => {
                    // Summaries are emitted as individual gauges per quantile
                    for (quantile, value) in quantiles {
                        let mut q_labels = labels.clone();
                        q_labels.push(("quantile".to_string(), format!("{quantile}")));
                        metrics::gauge!(name.clone(), &q_labels).set(value);
                    }
                }
            }
        }
        Ok(())
    }
}

/// Register standard framework metrics with initial zero values.
///
/// This pre-registers metric names so they appear in `/metrics` output
/// even before any observations are recorded.
pub fn register_standard_metrics() {
    // Counters
    metrics::counter!("mistersmith_messages_sent_total").absolute(0);
    metrics::counter!("mistersmith_messages_received_total").absolute(0);
    metrics::counter!("mistersmith_tasks_completed_total").absolute(0);
    metrics::counter!("mistersmith_tasks_failed_total").absolute(0);
    metrics::counter!("mistersmith_agent_restarts_total").absolute(0);

    // Gauges
    metrics::gauge!("mistersmith_active_agents").set(0.0);
    metrics::gauge!("mistersmith_message_queue_depth").set(0.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::SystemTime;

    #[tokio::test]
    async fn prometheus_backend_accepts_counters() {
        // Without an installed recorder, metrics calls are no-ops but should not panic
        let backend = PrometheusBackend::new();
        let metrics = vec![Metric {
            name: "test_counter".to_string(),
            value: MetricValue::Counter(42),
            timestamp: SystemTime::now(),
            tags: HashMap::new(),
        }];
        assert!(backend.send_metrics(metrics).await.is_ok());
    }

    #[tokio::test]
    async fn prometheus_backend_accepts_gauges() {
        let backend = PrometheusBackend::new();
        let metrics = vec![Metric {
            name: "test_gauge".to_string(),
            value: MetricValue::Gauge(3.14),
            timestamp: SystemTime::now(),
            tags: HashMap::new(),
        }];
        assert!(backend.send_metrics(metrics).await.is_ok());
    }

    #[tokio::test]
    async fn prometheus_backend_accepts_histograms() {
        let backend = PrometheusBackend::new();
        let metrics = vec![Metric {
            name: "test_histogram".to_string(),
            value: MetricValue::Histogram(0.042),
            timestamp: SystemTime::now(),
            tags: HashMap::new(),
        }];
        assert!(backend.send_metrics(metrics).await.is_ok());
    }

    #[tokio::test]
    async fn prometheus_backend_accepts_tagged_metrics() {
        let backend = PrometheusBackend::new();
        let mut tags = HashMap::new();
        tags.insert("agent_type".to_string(), "planner".to_string());
        let metrics = vec![Metric {
            name: "test_tagged".to_string(),
            value: MetricValue::Counter(1),
            timestamp: SystemTime::now(),
            tags,
        }];
        assert!(backend.send_metrics(metrics).await.is_ok());
    }

    #[tokio::test]
    async fn prometheus_backend_accepts_summaries() {
        let backend = PrometheusBackend::new();
        let metrics = vec![Metric {
            name: "test_summary".to_string(),
            value: MetricValue::Summary {
                quantiles: vec![(0.5, 100.0), (0.9, 200.0), (0.99, 500.0)],
            },
            timestamp: SystemTime::now(),
            tags: HashMap::new(),
        }];
        assert!(backend.send_metrics(metrics).await.is_ok());
    }

    #[tokio::test]
    async fn prometheus_backend_handles_empty_batch() {
        let backend = PrometheusBackend::new();
        assert!(backend.send_metrics(vec![]).await.is_ok());
    }
}
