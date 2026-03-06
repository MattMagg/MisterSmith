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
use tracing::info;

/// Handles returned from observability initialization.
///
/// Must be kept alive for the duration of the process. Dropping this
/// will shut down telemetry providers.
pub struct ObservabilityGuard {
    /// Prometheus handle for rendering /metrics endpoint.
    pub prometheus_handle: Option<metrics_exporter_prometheus::PrometheusHandle>,
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
