//! Mister Smith — Multi-agent orchestration framework binary entry point.
//!
//! Orchestrates process lifecycle: deterministic startup, graceful shutdown,
//! signal handling, observability initialization, and cross-phase integration wiring.

mod config;
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

    // Initialize basic tracing (will be enhanced in Phase 4/US2 with OTel)
    let filter = tracing_subscriber::EnvFilter::try_new(&config.observability.log_level)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true);

    match config.observability.log_format {
        LogFormat::Json => {
            subscriber.json().init();
        }
        LogFormat::Pretty => {
            subscriber.pretty().init();
        }
    }

    // Initialize process state tracker
    let state_tracker = ProcessStateTracker::new();
    let shutdown_coordinator = shutdown::ShutdownCoordinator::new();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        "Mister Smith starting"
    );

    // TODO: Phase 3 (US1) — implement full bootstrap sequence
    // For now, just mark as ready and wait for signal
    state_tracker.set(ProcessLifecycle::Ready);
    info!("Mister Smith ready (stub — full bootstrap pending US1 implementation)");

    // Wait for termination signal
    let signal_name = shutdown::wait_for_signal().await;
    info!(signal = signal_name, "Received termination signal");

    // Begin graceful shutdown
    shutdown_coordinator.begin_graceful_shutdown();
    state_tracker.set(ProcessLifecycle::Draining);

    // Spawn forced shutdown listener
    let forced_flag = shutdown_coordinator.shutdown_in_progress.clone();
    let forced_handle = tokio::spawn(async move {
        shutdown::wait_for_forced_signal(forced_flag).await;
        error!("Forced shutdown — exiting immediately");
        std::process::exit(2);
    });

    // TODO: Phase 3 (US1) — implement full graceful shutdown sequence
    // For now, just mark as stopped
    state_tracker.set(ProcessLifecycle::Stopped);
    info!("Mister Smith stopped");

    forced_handle.abort();
    std::process::exit(0);
}
