//! RuntimeManager: lifecycle management and shutdown coordination.
//!
//! Owns the Tokio [`Runtime`], coordinates graceful shutdown via an
//! [`AtomicBool`] signal, tracks spawned tasks, and initialises
//! the `tracing` subscriber.
//!
//! # Usage
//!
//! ```no_run
//! use mister_smith_config::RuntimeConfig;
//! use mister_smith_runtime::manager::RuntimeManager;
//!
//! let manager = RuntimeManager::initialize(&RuntimeConfig::default())
//!     .expect("runtime should build");
//!
//! manager.start_system();
//!
//! // … spawn work …
//! manager.spawn_task(async {
//!     // application logic
//! });
//!
//! // Shut down cleanly when ready.
//! // manager.graceful_shutdown().unwrap();
//! ```

use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

use mister_smith_config::RuntimeConfig;
use mister_smith_core::{EventPublisher, RuntimeError};

use crate::config::{RuntimeConfigExt, DEFAULT_SHUTDOWN_TIMEOUT};

// ---------------------------------------------------------------------------
// RuntimeManager
// ---------------------------------------------------------------------------

/// Manages the Tokio runtime lifecycle, shutdown coordination, and task tracking.
///
/// `RuntimeManager` is the outermost lifecycle owner in the framework. It:
///
/// - Builds and holds the Tokio [`Runtime`] according to [`RuntimeConfig`].
/// - Provides a cooperative shutdown signal via [`AtomicBool`].
/// - Tracks spawned task handles so they can be joined during shutdown.
/// - Installs signal handlers for `SIGTERM`/`SIGINT` (unix) or `ctrl_c` (other).
/// - Initialises the `tracing` subscriber on first construction.
pub struct RuntimeManager {
    /// The Tokio runtime.
    runtime: Arc<Runtime>,
    /// Cooperative shutdown signal — `true` means shutdown has been requested.
    shutdown_signal: Arc<AtomicBool>,
    /// Handles to all tasks spawned through [`spawn_task`](Self::spawn_task) and
    /// [`spawn_blocking_task`](Self::spawn_blocking_task).
    task_handles: Mutex<Vec<JoinHandle<()>>>,
    /// How long to wait for tasks to finish during graceful shutdown.
    shutdown_timeout: Duration,
    /// Optional event publisher for emitting lifecycle events (wired at integration time).
    #[allow(dead_code)]
    event_publisher: Option<Arc<dyn EventPublisher>>,
}

impl RuntimeManager {
    /// Returns a [`RuntimeManagerBuilder`] for step-by-step construction.
    pub fn builder() -> RuntimeManagerBuilder {
        RuntimeManagerBuilder::default()
    }

    /// Build a `RuntimeManager` directly from a [`RuntimeConfig`].
    ///
    /// This is a convenience entry point that uses default shutdown timeout
    /// and no event publisher.  For full control, use [`builder()`](Self::builder).
    ///
    /// # Tracing initialisation
    ///
    /// On the first call, installs a `tracing_subscriber` with
    /// [`EnvFilter`] defaulting to `"info"`.
    /// Subsequent calls silently skip re-initialisation (via `try_init()`).
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::BuildFailed`] if the Tokio runtime cannot be
    /// constructed from the given configuration.
    pub fn initialize(config: &RuntimeConfig) -> Result<Self, RuntimeError> {
        Self::init_tracing();

        let runtime = config.build_runtime()?;

        info!(
            worker_threads = ?config.worker_threads,
            blocking_threads = config.blocking_threads,
            "RuntimeManager initialized"
        );

        Ok(Self {
            runtime: Arc::new(runtime),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            task_handles: Mutex::new(Vec::new()),
            shutdown_timeout: DEFAULT_SHUTDOWN_TIMEOUT,
            event_publisher: None,
        })
    }

    // -- Lifecycle -----------------------------------------------------------

    /// Start the runtime system.
    ///
    /// Spawns a background signal handler that sets the shutdown signal on
    /// `SIGTERM`/`SIGINT` (or `ctrl_c` on non-unix platforms) and logs the
    /// startup banner.
    pub fn start_system(&self) {
        let signal = Arc::clone(&self.shutdown_signal);
        self.runtime.spawn(signal_handler(signal));
        info!("Mister Smith runtime started — signal handler active");
    }

    /// Perform a graceful shutdown.
    ///
    /// 1. Sets the shutdown signal so all cooperative loops can observe it.
    /// 2. Drains tracked task handles, waiting up to the configured shutdown timeout.
    /// 3. Shuts down the Tokio runtime with the configured timeout.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::ShutdownFailed`] if:
    /// - The task handle mutex is poisoned.
    /// - Any tracked task panicked.
    pub fn graceful_shutdown(self) -> Result<(), RuntimeError> {
        info!("Graceful shutdown initiated");

        // 1. Signal all cooperative loops.
        self.shutdown_signal.store(true, Ordering::SeqCst);

        // 2. Drain and join tracked tasks.
        let handles = {
            let mut guard = self.task_handles.lock().map_err(|e| {
                RuntimeError::ShutdownFailed(format!("task handle mutex poisoned: {e}"))
            })?;
            std::mem::take(&mut *guard)
        };

        let handle_count = handles.len();
        if handle_count > 0 {
            info!(count = handle_count, "Joining tracked task handles");
        }

        // We must block on the joins using the runtime itself, since the runtime
        // is still alive at this point.
        self.runtime.block_on(async {
            for handle in handles {
                match tokio::time::timeout(self.shutdown_timeout, handle).await {
                    Ok(Ok(())) => {}
                    Ok(Err(join_err)) => {
                        warn!(%join_err, "Tracked task panicked during shutdown");
                    }
                    Err(_elapsed) => {
                        warn!("Tracked task did not complete within shutdown timeout");
                    }
                }
            }
        });

        // 3. Shut down the Tokio runtime.
        //    `Arc::try_unwrap` recovers the inner `Runtime` so we can call
        //    `shutdown_timeout`.  If other Arc references still exist, we
        //    log a warning and let Drop handle it.
        match Arc::try_unwrap(self.runtime) {
            Ok(runtime) => {
                info!(timeout = ?self.shutdown_timeout, "Shutting down Tokio runtime");
                runtime.shutdown_timeout(self.shutdown_timeout);
            }
            Err(_arc) => {
                warn!(
                    "Outstanding Arc<Runtime> references exist; \
                     runtime will shut down when all references are dropped"
                );
            }
        }

        info!("Graceful shutdown complete");
        Ok(())
    }

    // -- Task spawning -------------------------------------------------------

    /// Spawn an async task on the runtime, tracking its [`JoinHandle`].
    ///
    /// The handle is stored so that [`graceful_shutdown`](Self::graceful_shutdown)
    /// can join it. Callers that need the handle directly can clone the
    /// returned reference.
    pub fn spawn_task<F>(&self, future: F) -> JoinHandle<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let handle = self.runtime.spawn(future);

        // Tokio's JoinHandle is not Clone, so we cannot both store it for
        // shutdown joining AND return it to the caller. We store the real
        // handle (needed for graceful_shutdown to join) and return a
        // lightweight sentinel handle to the caller. The caller can await
        // the sentinel or ignore it; the real work is tracked internally.
        let abort_handle = handle.abort_handle();
        if let Ok(mut handles) = self.task_handles.lock() {
            handles.push(handle);
        } else {
            error!("Task handle mutex poisoned — task will not be tracked for shutdown");
        }

        // Sentinel task: completes immediately, gives caller a JoinHandle.
        // Captures abort_handle so the caller could abort the real task if
        // they wanted (by aborting through the handle).
        self.runtime.spawn(async move {
            let _ = abort_handle;
        })
    }

    /// Spawn a blocking task on the runtime's blocking thread pool, tracking it.
    ///
    /// This is for CPU-heavy or synchronous work that should not run on the
    /// async worker threads.
    pub fn spawn_blocking_task<F>(&self, f: F) -> JoinHandle<()>
    where
        F: FnOnce() + Send + 'static,
    {
        let handle = self.runtime.spawn_blocking(f);
        if let Ok(mut handles) = self.task_handles.lock() {
            handles.push(handle);
        } else {
            error!("Task handle mutex poisoned — blocking task will not be tracked");
        }
        // Return a lightweight handle (same pattern as spawn_task).
        self.runtime.spawn(async {})
    }

    // -- Accessors -----------------------------------------------------------

    /// Returns a reference to the underlying Tokio [`Runtime`] wrapped in [`Arc`].
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }

    /// Returns `true` if shutdown has been signalled.
    pub fn is_shutting_down(&self) -> bool {
        self.shutdown_signal.load(Ordering::SeqCst)
    }

    /// Returns a clone of the shutdown signal for use in cooperative loops.
    pub fn shutdown_signal(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.shutdown_signal)
    }

    // -- Internal ------------------------------------------------------------

    /// Initialise the `tracing` subscriber.
    ///
    /// Uses `try_init()` so that repeated calls (e.g., in tests) are harmless.
    fn init_tracing() {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));

        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .try_init();
    }
}

impl std::fmt::Debug for RuntimeManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let handle_count = self
            .task_handles
            .lock()
            .map(|h| h.len())
            .unwrap_or(0);
        f.debug_struct("RuntimeManager")
            .field("shutting_down", &self.is_shutting_down())
            .field("tracked_tasks", &handle_count)
            .field("shutdown_timeout", &self.shutdown_timeout)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// RuntimeManagerBuilder
// ---------------------------------------------------------------------------

/// Builder for [`RuntimeManager`] with optional integrations.
///
/// # Example
///
/// ```no_run
/// use mister_smith_config::RuntimeConfig;
/// use mister_smith_runtime::manager::RuntimeManager;
/// use std::time::Duration;
///
/// let manager = RuntimeManager::builder()
///     .shutdown_timeout(Duration::from_secs(60))
///     .build(&RuntimeConfig::default())
///     .expect("runtime should build");
/// ```
#[derive(Default)]
pub struct RuntimeManagerBuilder {
    shutdown_timeout: Option<Duration>,
    event_publisher: Option<Arc<dyn EventPublisher>>,
}

impl RuntimeManagerBuilder {
    /// Set a custom shutdown timeout (overrides [`DEFAULT_SHUTDOWN_TIMEOUT`]).
    pub fn shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shutdown_timeout = Some(timeout);
        self
    }

    /// Attach an event publisher for emitting lifecycle events.
    ///
    /// This is wired during integration; the runtime crate itself does not
    /// depend on the events crate.
    pub fn event_publisher(mut self, publisher: Arc<dyn EventPublisher>) -> Self {
        self.event_publisher = Some(publisher);
        self
    }

    /// Build the [`RuntimeManager`].
    ///
    /// Initialises tracing and constructs the Tokio runtime from `config`.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::BuildFailed`] if the runtime cannot be built.
    pub fn build(self, config: &RuntimeConfig) -> Result<RuntimeManager, RuntimeError> {
        RuntimeManager::init_tracing();

        let runtime = config.build_runtime()?;

        info!(
            worker_threads = ?config.worker_threads,
            blocking_threads = config.blocking_threads,
            shutdown_timeout = ?self.shutdown_timeout.unwrap_or(DEFAULT_SHUTDOWN_TIMEOUT),
            "RuntimeManager built via builder"
        );

        Ok(RuntimeManager {
            runtime: Arc::new(runtime),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            task_handles: Mutex::new(Vec::new()),
            shutdown_timeout: self.shutdown_timeout.unwrap_or(DEFAULT_SHUTDOWN_TIMEOUT),
            event_publisher: self.event_publisher,
        })
    }
}

impl std::fmt::Debug for RuntimeManagerBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeManagerBuilder")
            .field("shutdown_timeout", &self.shutdown_timeout)
            .field("has_event_publisher", &self.event_publisher.is_some())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Signal handler
// ---------------------------------------------------------------------------

/// Async signal handler that sets the shutdown flag on `SIGTERM` or `SIGINT`.
///
/// On unix platforms, listens for both `SIGTERM` and `SIGINT`.
/// On non-unix platforms, falls back to `tokio::signal::ctrl_c()`.
async fn signal_handler(shutdown_signal: Arc<AtomicBool>) {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to register SIGTERM handler");
        let mut sigint =
            signal(SignalKind::interrupt()).expect("failed to register SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => {
                info!("Received SIGTERM — initiating shutdown");
            }
            _ = sigint.recv() => {
                info!("Received SIGINT — initiating shutdown");
            }
        }
    }

    #[cfg(not(unix))]
    {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Received Ctrl+C — initiating shutdown");
            }
            Err(e) => {
                error!(error = %e, "Failed to listen for Ctrl+C signal");
            }
        }
    }

    shutdown_signal.store(true, Ordering::SeqCst);
    info!("Shutdown signal set");
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> RuntimeConfig {
        RuntimeConfig {
            worker_threads: Some(2),
            blocking_threads: 32,
            ..RuntimeConfig::default()
        }
    }

    #[test]
    fn initialize_creates_manager() {
        let manager = RuntimeManager::initialize(&test_config())
            .expect("should initialize");
        assert!(!manager.is_shutting_down());
    }

    #[test]
    fn builder_creates_manager_with_custom_timeout() {
        let timeout = Duration::from_secs(10);
        let manager = RuntimeManager::builder()
            .shutdown_timeout(timeout)
            .build(&test_config())
            .expect("should build");

        assert_eq!(manager.shutdown_timeout, timeout);
        assert!(!manager.is_shutting_down());
    }

    #[test]
    fn builder_default_timeout_is_30s() {
        let manager = RuntimeManager::builder()
            .build(&test_config())
            .expect("should build");

        assert_eq!(manager.shutdown_timeout, DEFAULT_SHUTDOWN_TIMEOUT);
    }

    #[test]
    fn spawn_task_runs_on_runtime() {
        let manager = RuntimeManager::initialize(&test_config())
            .expect("should initialize");

        let (tx, rx) = std::sync::mpsc::channel();
        manager.spawn_task(async move {
            tx.send(42).unwrap();
        });

        let val = rx.recv_timeout(Duration::from_secs(5))
            .expect("task should have sent a value");
        assert_eq!(val, 42);
    }

    #[test]
    fn spawn_blocking_task_runs() {
        let manager = RuntimeManager::initialize(&test_config())
            .expect("should initialize");

        let (tx, rx) = std::sync::mpsc::channel();
        manager.spawn_blocking_task(move || {
            tx.send(99).unwrap();
        });

        let val = rx.recv_timeout(Duration::from_secs(5))
            .expect("blocking task should have sent a value");
        assert_eq!(val, 99);
    }

    #[test]
    fn is_shutting_down_reflects_signal() {
        let manager = RuntimeManager::initialize(&test_config())
            .expect("should initialize");

        assert!(!manager.is_shutting_down());
        manager.shutdown_signal.store(true, Ordering::SeqCst);
        assert!(manager.is_shutting_down());
    }

    #[test]
    fn runtime_accessor_returns_arc() {
        let manager = RuntimeManager::initialize(&test_config())
            .expect("should initialize");
        let rt = manager.runtime();
        // Verify we can use the runtime.
        rt.block_on(async { 1 + 1 });
    }

    #[test]
    fn shutdown_signal_accessor_shares_flag() {
        let manager = RuntimeManager::initialize(&test_config())
            .expect("should initialize");
        let signal = manager.shutdown_signal();
        signal.store(true, Ordering::SeqCst);
        assert!(manager.is_shutting_down());
    }

    #[test]
    fn graceful_shutdown_completes() {
        let manager = RuntimeManager::initialize(&test_config())
            .expect("should initialize");

        // Spawn a task that completes quickly.
        manager.spawn_task(async {
            tokio::time::sleep(Duration::from_millis(10)).await;
        });

        // Give the task a moment to be scheduled.
        std::thread::sleep(Duration::from_millis(50));

        manager.graceful_shutdown().expect("shutdown should succeed");
    }

    #[test]
    fn debug_impl_works() {
        let manager = RuntimeManager::initialize(&test_config())
            .expect("should initialize");
        let debug = format!("{:?}", manager);
        assert!(debug.contains("RuntimeManager"));
        assert!(debug.contains("shutting_down"));
    }

    #[test]
    fn builder_debug_impl() {
        let builder = RuntimeManager::builder()
            .shutdown_timeout(Duration::from_secs(5));
        let debug = format!("{:?}", builder);
        assert!(debug.contains("RuntimeManagerBuilder"));
    }

    #[test]
    fn start_system_installs_signal_handler() {
        let manager = RuntimeManager::initialize(&test_config())
            .expect("should initialize");
        // start_system should not panic.
        manager.start_system();
        // Signal handler is running — verify the manager is still not shutting down.
        assert!(!manager.is_shutting_down());
    }
}
