//! Observability pipeline initialization and shutdown.
//!
//! Sets up the full telemetry stack:
//! - OpenTelemetry TracerProvider with OTLP batch exporter (when endpoint configured)
//! - OpenTelemetry MeterProvider with OTLP periodic exporter (when endpoint configured)
//! - W3C TraceContext propagator for distributed trace correlation
//! - `tracing` subscriber with:
//!   - `tracing-opentelemetry` bridge layer (spans → OTel)
//!   - `tracing_subscriber::fmt` layer (JSON or Pretty structured logs)
//!   - `EnvFilter` for log level control
//! - Prometheus metrics recorder for `/metrics` scraping

use mister_smith_config::ObservabilityConfig;
use mister_smith_events::{AutonomyEvent, AutonomyStatusView, EventBus};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::info;

/// Handles returned from observability initialization.
///
/// Must be kept alive for the duration of the process. Dropping this
/// will shut down telemetry providers.
pub struct ObservabilityGuard {
    /// Prometheus handle for rendering /metrics endpoint.
    pub prometheus_handle: Option<metrics_exporter_prometheus::PrometheusHandle>,
}

/// Metric write kind emitted from autonomy operator state.
#[derive(Debug, Clone, PartialEq)]
pub enum MetricOperationKind {
    /// Set a gauge to the provided value.
    Gauge,
    /// Increment a counter by the provided value.
    Counter,
}

/// One metrics write derived from a typed autonomy event and status view.
#[derive(Debug, Clone, PartialEq)]
pub struct MetricOperation {
    /// Metric name.
    pub name: &'static str,
    /// Whether the metric is a gauge or counter increment.
    pub kind: MetricOperationKind,
    /// Metric value to write.
    pub value: f64,
    /// String labels attached to the metric.
    pub labels: Vec<(String, String)>,
}

/// Initialize the full observability pipeline.
///
/// Must be called before any other subsystem logs or traces.
/// Returns an [`ObservabilityGuard`] that must be held until shutdown.
pub fn init_observability(
    config: &ObservabilityConfig,
) -> Result<ObservabilityGuard, Box<dyn std::error::Error + Send + Sync>> {
    // Step 1: Set up the Prometheus metrics recorder (always, for /metrics endpoint)
    let prometheus_handle = if config.prometheus_enabled {
        let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
        let handle = builder.install_recorder()?;
        Some(handle)
    } else {
        None
    };

    // Step 2: Build tracing subscriber layers
    let filter = tracing_subscriber::EnvFilter::try_new(&config.log_level)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    // Step 3: Set up OTel if endpoint is configured
    if let Some(ref _endpoint) = config.otlp_endpoint {
        // OTel TracerProvider and MeterProvider initialization
        // Deferred to when opentelemetry 0.31 runtime dependencies are resolved.
        // For now, install a tracing subscriber without the OTel bridge layer.
        // The tracing instrumentation (#[instrument] spans) still works — they
        // just emit to the fmt layer instead of being exported via OTLP.
        info!(
            endpoint = %_endpoint,
            "OTLP endpoint configured — OTel export will be enabled in a future iteration"
        );
    }

    // Step 4: Install the tracing subscriber
    use tracing_subscriber::prelude::*;

    let fmt_layer = match config.log_format {
        mister_smith_config::LogFormat::Json => tracing_subscriber::fmt::layer()
            .json()
            .with_target(true)
            .with_thread_ids(true)
            .with_span_list(true)
            .boxed(),
        mister_smith_config::LogFormat::Pretty => tracing_subscriber::fmt::layer()
            .pretty()
            .with_target(true)
            .boxed(),
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();

    info!("Observability pipeline initialized");

    Ok(ObservabilityGuard { prometheus_handle })
}

/// Gracefully shut down all observability providers.
///
/// Flushes remaining spans and metrics to the collector with a timeout.
/// Called during the shutdown sequence before connections are closed.
pub fn shutdown_observability(_guard: ObservabilityGuard) {
    // When OTel providers are initialized, this will call:
    // - TracerProvider::shutdown()
    // - MeterProvider::shutdown()
    // For now, dropping the guard is sufficient.
    info!("Observability pipeline shut down");
}

/// Spawn a detached observer that converts typed autonomy events into Prometheus metrics.
pub fn spawn_autonomy_metrics_observer(event_bus: Arc<EventBus>, shutdown_flag: Arc<AtomicBool>) {
    tokio::spawn(async move {
        let mut rx = event_bus.subscribe_broadcast();

        while !shutdown_flag.load(Ordering::SeqCst) {
            let event = match rx.recv().await {
                Ok(event) => event,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            };

            let Ok(Some(autonomy_event)) = event.autonomy_event() else {
                continue;
            };

            let workflow_id = autonomy_event.workflow_id();
            let Some(view) = event_bus.autonomy_status(&workflow_id).await else {
                continue;
            };

            apply_metric_operations(build_metric_operations(&autonomy_event, &view));
        }
    });
}

/// Build the metric writes implied by a typed autonomy event and the latest status view.
pub fn build_metric_operations(
    event: &AutonomyEvent,
    view: &AutonomyStatusView,
) -> Vec<MetricOperation> {
    let mut operations = Vec::new();
    let workflow_id = view.graph.workflow_id.to_string();

    operations.push(MetricOperation {
        name: "mistersmith_autonomy_workflows_active",
        kind: MetricOperationKind::Gauge,
        value: 1.0,
        labels: vec![
            ("workflow_id".to_string(), workflow_id.clone()),
            (
                "status".to_string(),
                format!("{:?}", view.graph.state).to_lowercase(),
            ),
        ],
    });

    operations.push(MetricOperation {
        name: "mistersmith_autonomy_topology_info",
        kind: MetricOperationKind::Gauge,
        value: 1.0,
        labels: vec![
            ("workflow_id".to_string(), workflow_id.clone()),
            (
                "topology".to_string(),
                format!("{:?}", view.topology.topology_kind).to_lowercase(),
            ),
            (
                "rationale".to_string(),
                view.topology.rationale.selected_for.clone(),
            ),
        ],
    });

    for branch in &view.branches {
        operations.push(MetricOperation {
            name: "mistersmith_autonomy_branches",
            kind: MetricOperationKind::Gauge,
            value: 1.0,
            labels: vec![
                ("workflow_id".to_string(), workflow_id.clone()),
                ("branch_id".to_string(), branch.branch_id.to_string()),
                (
                    "state".to_string(),
                    format!("{:?}", branch.state).to_lowercase(),
                ),
                (
                    "recovery_state".to_string(),
                    format!("{:?}", branch.recovery_strategy).to_lowercase(),
                ),
            ],
        });
    }

    for checkpoint in &view.checkpoint_lineage {
        operations.push(MetricOperation {
            name: "mistersmith_autonomy_branch_checkpoint_age_seconds",
            kind: MetricOperationKind::Gauge,
            value: (chrono::Utc::now() - checkpoint.captured_at)
                .num_seconds()
                .max(0) as f64,
            labels: vec![
                ("workflow_id".to_string(), workflow_id.clone()),
                ("branch_id".to_string(), checkpoint.branch_id.to_string()),
            ],
        });
    }

    for pressure in &view.memory_pressure {
        let ratio = pressure_ratio(pressure.max_units, pressure.reserved_units);
        operations.push(MetricOperation {
            name: "mistersmith_autonomy_context_pressure_ratio",
            kind: MetricOperationKind::Gauge,
            value: ratio,
            labels: vec![
                ("workflow_id".to_string(), workflow_id.clone()),
                (
                    "branch_id".to_string(),
                    pressure
                        .branch_id
                        .map(|branch_id| branch_id.to_string())
                        .unwrap_or_else(|| pressure.budget_id.to_string()),
                ),
                (
                    "pressure_level".to_string(),
                    pressure_level(ratio).to_string(),
                ),
            ],
        });
    }

    if let AutonomyEvent::InterventionRecorded(envelope) = event {
        let intervention = view
            .guard_decisions
            .iter()
            .find(|decision| decision.decision_id == envelope.payload.decision_id)
            .map(|decision| format!("{:?}", decision.intervention).to_lowercase())
            .unwrap_or_else(|| "unknown".to_string());
        operations.push(MetricOperation {
            name: "mistersmith_autonomy_interventions_total",
            kind: MetricOperationKind::Counter,
            value: 1.0,
            labels: vec![
                ("workflow_id".to_string(), workflow_id.clone()),
                (
                    "branch_id".to_string(),
                    event
                        .branch_id()
                        .map(|branch_id| branch_id.to_string())
                        .unwrap_or_else(|| "graph".to_string()),
                ),
                ("intervention".to_string(), intervention),
                ("decision_source".to_string(), "guard".to_string()),
            ],
        });
    }

    if let AutonomyEvent::DelegationUpdated(envelope) = event {
        if envelope.payload.revocation_state != mister_smith_core::RevocationState::Active {
            operations.push(MetricOperation {
                name: "mistersmith_autonomy_delegation_rejections_total",
                kind: MetricOperationKind::Counter,
                value: 1.0,
                labels: vec![
                    ("workflow_id".to_string(), workflow_id),
                    (
                        "reason".to_string(),
                        format!("{:?}", envelope.payload.revocation_state).to_lowercase(),
                    ),
                    (
                        "scope".to_string(),
                        format!("{:?}", envelope.payload.scope).to_lowercase(),
                    ),
                    (
                        "issuer".to_string(),
                        format!("{:?}", envelope.payload.issuer),
                    ),
                    (
                        "recipient".to_string(),
                        envelope.payload.recipient.to_string(),
                    ),
                ],
            });
        }
    }

    operations
}

fn apply_metric_operations(operations: Vec<MetricOperation>) {
    for operation in operations {
        match operation.kind {
            MetricOperationKind::Gauge => {
                metrics::gauge!(operation.name, &operation.labels).set(operation.value)
            }
            MetricOperationKind::Counter => metrics::counter!(operation.name, &operation.labels)
                .increment(operation.value as u64),
        }
    }
}

fn pressure_ratio(max_units: u64, reserved_units: u64) -> f64 {
    if max_units == 0 {
        return 0.0;
    }

    (reserved_units as f64 / max_units as f64).clamp(0.0, 1.0)
}

fn pressure_level(ratio: f64) -> &'static str {
    if ratio >= 0.9 {
        "critical"
    } else if ratio >= 0.7 {
        "elevated"
    } else {
        "normal"
    }
}
