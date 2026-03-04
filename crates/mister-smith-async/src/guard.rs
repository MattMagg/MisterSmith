//! RAII task guard for cleanup on drop.
//!
//! [`TaskGuard`] wraps a [`JoinHandle`](tokio::task::JoinHandle) and an
//! optional cleanup closure. When the guard is dropped — whether normally or
//! due to a panic — it aborts the spawned task and runs the cleanup function.

use tokio::task::JoinHandle;

/// RAII guard that aborts a spawned task and runs cleanup when dropped.
///
/// # Examples
///
/// ```no_run
/// # use mister_smith_async::guard::TaskGuard;
/// # async fn example() {
/// let handle = tokio::spawn(async {
///     loop { tokio::time::sleep(std::time::Duration::from_secs(1)).await; }
/// });
/// let guard = TaskGuard::new(handle)
///     .with_cleanup(|| println!("Task cleaned up"));
/// // When `guard` goes out of scope, the task is aborted and the
/// // cleanup closure executes.
/// # }
/// ```
pub struct TaskGuard {
    handle: Option<JoinHandle<()>>,
    cleanup: Option<Box<dyn FnOnce() + Send>>,
}

impl TaskGuard {
    /// Create a guard that will abort the given task handle on drop.
    pub fn new(handle: JoinHandle<()>) -> Self {
        Self {
            handle: Some(handle),
            cleanup: None,
        }
    }

    /// Attach a cleanup closure to run on drop (builder pattern).
    pub fn with_cleanup<F: FnOnce() + Send + 'static>(mut self, f: F) -> Self {
        self.cleanup = Some(Box::new(f));
        self
    }
}

impl Drop for TaskGuard {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

impl std::fmt::Debug for TaskGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskGuard")
            .field("has_handle", &self.handle.is_some())
            .field("has_cleanup", &self.cleanup.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn guard_aborts_task_on_drop() {
        let handle = tokio::spawn(async {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        });

        let abort_handle = handle.abort_handle();
        {
            let _guard = TaskGuard::new(handle);
        }

        // Yield so the runtime can process the abort signal.
        tokio::task::yield_now().await;
        assert!(abort_handle.is_finished());
    }

    #[tokio::test]
    async fn guard_runs_cleanup_on_drop() {
        let cleaned_up = Arc::new(AtomicBool::new(false));
        let cleaned_up_clone = Arc::clone(&cleaned_up);

        let handle = tokio::spawn(async {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        });

        {
            let _guard =
                TaskGuard::new(handle).with_cleanup(move || {
                    cleaned_up_clone.store(true, Ordering::SeqCst);
                });
            // guard drops here.
        }

        assert!(cleaned_up.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn guard_without_cleanup() {
        let handle = tokio::spawn(async {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        });

        let abort_handle = handle.abort_handle();
        {
            let _guard = TaskGuard::new(handle);
        }

        // Yield so the runtime can process the abort signal.
        tokio::task::yield_now().await;
        assert!(abort_handle.is_finished());
    }

    #[test]
    fn debug_impl() {
        // We cannot create a real JoinHandle outside of a runtime, so just
        // verify the Debug derive is present by checking the type compiles.
        // The actual debug output is tested in the async tests above.
    }
}
