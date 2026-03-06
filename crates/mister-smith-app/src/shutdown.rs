//! Signal handling and graceful/forced shutdown.
//!
//! Handles SIGTERM and SIGINT for orderly process termination.
//! First signal triggers graceful shutdown; second signal forces immediate exit.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal::unix::{signal, SignalKind};
use tracing::{info, warn};

/// Waits for a termination signal (SIGTERM or SIGINT).
///
/// Returns the signal name that was received.
pub async fn wait_for_signal() -> &'static str {
    let mut sigterm =
        signal(SignalKind::terminate()).expect("failed to register SIGTERM handler");
    let mut sigint =
        signal(SignalKind::interrupt()).expect("failed to register SIGINT handler");

    tokio::select! {
        _ = sigterm.recv() => "SIGTERM",
        _ = sigint.recv() => "SIGINT",
    }
}

/// Waits for a second termination signal during graceful shutdown.
///
/// If a second signal arrives while graceful shutdown is in progress,
/// this returns immediately to trigger forced shutdown.
pub async fn wait_for_forced_signal(shutdown_in_progress: Arc<AtomicBool>) {
    if !shutdown_in_progress.load(Ordering::SeqCst) {
        return;
    }

    let mut sigterm =
        signal(SignalKind::terminate()).expect("failed to register SIGTERM handler");
    let mut sigint =
        signal(SignalKind::interrupt()).expect("failed to register SIGINT handler");

    tokio::select! {
        _ = sigterm.recv() => {
            warn!("Second SIGTERM received during graceful shutdown — forcing immediate exit");
        }
        _ = sigint.recv() => {
            warn!("Second SIGINT received during graceful shutdown — forcing immediate exit");
        }
    }
}

/// Shutdown coordinator that manages the graceful/forced shutdown flow.
pub struct ShutdownCoordinator {
    /// Whether a graceful shutdown is currently in progress.
    pub shutdown_in_progress: Arc<AtomicBool>,
}

impl ShutdownCoordinator {
    pub fn new() -> Self {
        Self {
            shutdown_in_progress: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Mark that graceful shutdown has started.
    pub fn begin_graceful_shutdown(&self) {
        self.shutdown_in_progress.store(true, Ordering::SeqCst);
        info!("Graceful shutdown initiated");
    }

    /// Check if shutdown is in progress.
    pub fn is_shutting_down(&self) -> bool {
        self.shutdown_in_progress.load(Ordering::SeqCst)
    }
}
