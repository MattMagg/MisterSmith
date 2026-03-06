//! Mister Smith — Multi-agent orchestration framework binary entry point.
//!
//! Orchestrates process lifecycle: deterministic startup, graceful shutdown,
//! signal handling, observability initialization, and cross-phase integration wiring.

mod bootstrap;
#[allow(dead_code)]
mod bridges;
mod config;
mod observability;
mod shutdown;

use clap::Parser;
use mister_smith_config::LogFormat;
use mister_smith_core::ProcessLifecycle;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use tracing::{error, info};

/// Mister Smith — Multi-agent orchestration framework.
#[derive(Parser, Debug)]
#[command(name = "mister-smith", version, about)]
struct Cli {
    /// Path to the configuration file.
    #[arg(short, long)]
    config: Option<String>,

    /// Override log level (trace, debug, info, warn, error).
    #[arg(long)]
    log_level: Option<String>,

    /// Override log format (json, pretty).
    #[arg(long)]
    log_format: Option<String>,
}

/// Thread-safe process state tracker.
///
/// Uses `AtomicU8` for lock-free reads from health probe endpoints.
#[derive(Clone)]
pub struct ProcessStateTracker {
    state: Arc<AtomicU8>,
}

impl Default for ProcessStateTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessStateTracker {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicU8::new(ProcessLifecycle::Starting as u8)),
        }
    }

    pub fn set(&self, lifecycle: ProcessLifecycle) {
        self.state.store(lifecycle as u8, Ordering::SeqCst);
    }

    pub fn get(&self) -> ProcessLifecycle {
        match self.state.load(Ordering::SeqCst) {
            0 => ProcessLifecycle::Starting,
            1 => ProcessLifecycle::Ready,
            2 => ProcessLifecycle::Draining,
            3 => ProcessLifecycle::Stopped,
            _ => ProcessLifecycle::Failed,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.get() == ProcessLifecycle::Ready
    }
}

fn parse_log_format(s: &str) -> Option<LogFormat> {
    match s.to_lowercase().as_str() {
        "json" => Some(LogFormat::Json),
        "pretty" => Some(LogFormat::Pretty),
        _ => None,
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Build CLI overrides
    let overrides = config::CliOverrides {
        config_path: cli.config,
        log_level: cli.log_level,
        log_format: cli.log_format.as_deref().and_then(parse_log_format),
    };

    // Load configuration
    let config = match config::load_framework_config(&overrides) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ERROR: Failed to load configuration: {e}");
            std::process::exit(1);
        }
    };

    // Initialize the observability pipeline (tracing, metrics, OTel)
    // Must happen before any logging.
    let otel_guard = match observability::init_observability(&config.observability) {
        Ok(guard) => guard,
        Err(e) => {
            eprintln!("ERROR: Failed to initialize observability: {e}");
            std::process::exit(1);
        }
    };

    // Initialize process state tracker and shutdown coordinator
    let state_tracker = ProcessStateTracker::new();
    let shutdown_coordinator = shutdown::ShutdownCoordinator::new();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        "Mister Smith starting"
    );

    // Run the deterministic bootstrap sequence
    let ctx = match bootstrap::bootstrap(&config, &state_tracker, &otel_guard).await {
        Ok(ctx) => ctx,
        Err(e) => {
            error!(error = %e, "Bootstrap failed");
            state_tracker.set(ProcessLifecycle::Failed);
            std::process::exit(1);
        }
    };

    // Wait for termination signal
    let signal_name = shutdown::wait_for_signal().await;
    info!(signal = signal_name, "Received termination signal");

    // Begin graceful shutdown
    shutdown_coordinator.begin_graceful_shutdown();
    state_tracker.set(ProcessLifecycle::Draining);

    // Spawn forced shutdown listener (second signal → immediate exit)
    let forced_flag = shutdown_coordinator.shutdown_in_progress.clone();
    let forced_handle = tokio::spawn(async move {
        shutdown::wait_for_forced_signal(forced_flag).await;
        error!("Forced shutdown — exiting immediately");
        std::process::exit(2);
    });

    // Execute the graceful shutdown sequence
    shutdown::graceful_shutdown(ctx, &state_tracker, &config).await;

    // Flush and shut down observability providers
    observability::shutdown_observability(otel_guard);

    // Cancel the forced shutdown listener (we're done gracefully)
    forced_handle.abort();
    std::process::exit(0);
}
