//! Signal handling and graceful/forced shutdown.
//!
//! Handles SIGTERM and SIGINT for orderly process termination.
//! First signal triggers graceful shutdown; second signal forces immediate exit.
//!
//! Shutdown sequence (per contracts/process-lifecycle.md):
//! 1. Set state to Draining
//! 2. Signal HTTP server to stop accepting connections
//! 3. Stop background monitoring loops
//! 4. Disconnect NATS
//! 5. Stop HTTP server (awaits in-flight requests)
//! 6. Set state to Stopped

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use mister_smith_config::FrameworkConfig;
use mister_smith_core::ProcessLifecycle;
use tokio::signal::unix::{signal, SignalKind};
use tracing::{error, info, warn};

use crate::bootstrap::BootstrapContext;
use crate::ProcessStateTracker;

/// Waits for a termination signal (SIGTERM or SIGINT).
///
/// Returns the signal name that was received.
pub async fn wait_for_signal() -> &'static str {
    let mut sigterm = signal(SignalKind::terminate()).expect("failed to register SIGTERM handler");
    let mut sigint = signal(SignalKind::interrupt()).expect("failed to register SIGINT handler");

    tokio::select! {
        _ = sigterm.recv() => "SIGTERM",
        _ = sigint.recv() => "SIGINT",
    }
}

/// Waits for a second termination signal during graceful shutdown.
///
/// Returns immediately if shutdown is not in progress.
/// When a second signal arrives, this returns to trigger forced shutdown.
pub async fn wait_for_forced_signal(shutdown_in_progress: Arc<AtomicBool>) {
    if !shutdown_in_progress.load(Ordering::SeqCst) {
        return;
    }

    let mut sigterm = signal(SignalKind::terminate()).expect("failed to register SIGTERM handler");
    let mut sigint = signal(SignalKind::interrupt()).expect("failed to register SIGINT handler");

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

impl Default for ShutdownCoordinator {
    fn default() -> Self {
        Self::new()
    }
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
    #[allow(dead_code)]
    pub fn is_shutting_down(&self) -> bool {
        self.shutdown_in_progress.load(Ordering::SeqCst)
    }
}

/// Execute the full graceful shutdown sequence.
///
/// Tears down subsystems in reverse initialization order with a
/// configurable timeout. Returns `Ok(())` on clean shutdown.
pub async fn graceful_shutdown(
    ctx: BootstrapContext,
    state_tracker: &ProcessStateTracker,
    config: &FrameworkConfig,
) {
    let start = Instant::now();
    let shutdown_timeout = Duration::from_secs(config.observability.shutdown_timeout_secs);

    info!("Graceful shutdown sequence starting");

    // Step 1: Already in Draining state (set by caller)

    // Step 2: Signal HTTP server to stop accepting new connections
    // Sending on the broadcast channel triggers the graceful shutdown future
    let _ = ctx.shutdown_tx.send(());
    info!("HTTP server signaled to stop");

    // Step 3: Stop background monitoring loops
    ctx.shutdown_flag.store(true, Ordering::SeqCst);
    if let Some(handle) = ctx.monitor_handle {
        let _ = tokio::time::timeout(Duration::from_secs(5), handle).await;
    }
    if let Some(handle) = ctx.metrics_handle {
        let _ = tokio::time::timeout(Duration::from_secs(5), handle).await;
    }
    info!("Background monitors stopped");

    // Step 4: Wait for HTTP server to finish draining
    if let Some(handle) = ctx.http_handle {
        let remaining = shutdown_timeout.saturating_sub(start.elapsed());
        match tokio::time::timeout(remaining, handle).await {
            Ok(Ok(())) => info!("HTTP server stopped"),
            Ok(Err(e)) => warn!(error = ?e, "HTTP server task error"),
            Err(_) => warn!("HTTP server drain timed out"),
        }
    }

    // Step 5: Shut down the supervised actor system and supervision loop.
    match tokio::time::timeout(Duration::from_secs(5), ctx.supervised_system.shutdown()).await {
        Ok(Ok(())) => info!("Supervised actor system stopped"),
        Ok(Err(e)) => warn!(error = %e, "Supervised actor system shutdown error"),
        Err(_) => warn!("Supervised actor system shutdown timed out"),
    }
    if let Some(handle) = ctx.supervision_handle {
        let _ = tokio::time::timeout(Duration::from_secs(5), handle).await;
    }

    // Step 6: Disconnect NATS
    if let Some(ref nats) = ctx.nats_transport {
        match tokio::time::timeout(Duration::from_secs(5), nats.disconnect()).await {
            Ok(Ok(())) => info!("NATS disconnected"),
            Ok(Err(e)) => warn!(error = %e, "NATS disconnect error"),
            Err(_) => warn!("NATS disconnect timed out"),
        }
    }

    // Step 7: Mark stopped
    state_tracker.set(ProcessLifecycle::Stopped);
    let shutdown_duration = start.elapsed();
    info!(
        duration_ms = shutdown_duration.as_millis() as u64,
        "Mister Smith stopped"
    );
}

/// Execute the forced shutdown path (second signal received).
///
/// Skips message drain and monitoring flush. Immediately closes
/// connections and exits with code 2.
#[allow(dead_code)]
pub async fn forced_shutdown(ctx: BootstrapContext, state_tracker: &ProcessStateTracker) {
    warn!("Forced shutdown — skipping drain, closing connections");

    // Signal servers to stop
    let _ = ctx.shutdown_tx.send(());
    ctx.shutdown_flag.store(true, Ordering::SeqCst);

    let _ = ctx.supervised_system.shutdown().await;
    if let Some(handle) = ctx.supervision_handle {
        handle.abort();
    }

    // Close NATS immediately (no drain wait)
    if let Some(ref nats) = ctx.nats_transport {
        let _ = nats.disconnect().await;
    }

    // Abort server tasks
    if let Some(handle) = ctx.http_handle {
        handle.abort();
    }

    state_tracker.set(ProcessLifecycle::Stopped);
    error!("Forced shutdown complete — exiting with code 2");
}
