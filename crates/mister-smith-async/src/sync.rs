//! Synchronization primitives: deadlock-preventing mutex, async barrier, countdown latch.
//!
//! These primitives build on top of `tokio::sync` types, adding timeout-based
//! acquisition and coordination patterns commonly needed in multi-agent systems.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::sync::Notify;

use crate::task::TaskError;

/// A mutex wrapper that enforces a timeout on lock acquisition to prevent deadlocks.
///
/// Each mutex is assigned an `acquisition_order` number. While not enforced at
/// runtime (this would require global tracking), the order value serves as
/// documentation for developers to acquire locks in a consistent order.
pub struct DeadlockPreventingMutex<T> {
    inner: tokio::sync::Mutex<T>,
    #[allow(dead_code)]
    acquisition_order: u64,
    default_timeout: Duration,
}

impl<T> DeadlockPreventingMutex<T> {
    /// Create a new mutex wrapping `value`.
    ///
    /// # Arguments
    ///
    /// * `value` — The data to protect.
    /// * `order` — Acquisition order hint (lower = acquire first).
    /// * `timeout` — Maximum time to wait for the lock before returning an error.
    pub fn new(value: T, order: u64, timeout: Duration) -> Self {
        Self {
            inner: tokio::sync::Mutex::new(value),
            acquisition_order: order,
            default_timeout: timeout,
        }
    }

    /// Attempt to acquire the lock within the configured timeout.
    ///
    /// Returns the guard on success, or a [`TaskError::Timeout`] if the lock
    /// could not be acquired within the deadline.
    pub async fn lock_with_timeout(
        &self,
    ) -> Result<tokio::sync::MutexGuard<'_, T>, TaskError> {
        match tokio::time::timeout(self.default_timeout, self.inner.lock()).await {
            Ok(guard) => Ok(guard),
            Err(_) => Err(TaskError::Timeout(format!(
                "Mutex acquisition timed out after {:?}",
                self.default_timeout
            ))),
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for DeadlockPreventingMutex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeadlockPreventingMutex")
            .field("acquisition_order", &self.acquisition_order)
            .field("default_timeout", &self.default_timeout)
            .finish()
    }
}

/// Async barrier — wraps [`tokio::sync::Barrier`].
///
/// All `count` participants must call [`wait`](AsyncBarrier::wait) before any
/// of them proceed.
pub struct AsyncBarrier {
    inner: tokio::sync::Barrier,
}

impl AsyncBarrier {
    /// Create a barrier that requires `count` participants.
    pub fn new(count: usize) -> Self {
        Self {
            inner: tokio::sync::Barrier::new(count),
        }
    }

    /// Wait at the barrier until all participants have arrived.
    pub async fn wait(&self) -> tokio::sync::BarrierWaitResult {
        self.inner.wait().await
    }
}

impl std::fmt::Debug for AsyncBarrier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncBarrier").finish()
    }
}

/// A countdown latch that blocks waiters until the count reaches zero.
///
/// Each call to [`count_down`](CountdownLatch::count_down) decrements the
/// internal counter. Waiters block on [`wait`](CountdownLatch::wait) until the
/// counter reaches zero.
pub struct CountdownLatch {
    count: AtomicUsize,
    notify: Notify,
}

impl CountdownLatch {
    /// Create a latch with the given initial count.
    pub fn new(count: usize) -> Self {
        Self {
            count: AtomicUsize::new(count),
            notify: Notify::new(),
        }
    }

    /// Decrement the count by one. If this brings the count to zero, all
    /// waiters are notified.
    pub fn count_down(&self) {
        let prev = self.count.fetch_sub(1, Ordering::SeqCst);
        if prev == 1 {
            // Count just reached zero — notify all waiters.
            self.notify.notify_waiters();
        }
    }

    /// Wait until the count reaches zero.
    pub async fn wait(&self) {
        // Fast path: already zero.
        while self.count.load(Ordering::SeqCst) > 0 {
            self.notify.notified().await;
        }
    }

    /// Current remaining count.
    pub fn remaining(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }
}

impl std::fmt::Debug for CountdownLatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CountdownLatch")
            .field("remaining", &self.count.load(Ordering::Relaxed))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // ---------- DeadlockPreventingMutex ----------

    #[tokio::test]
    async fn mutex_lock_success() {
        let m = DeadlockPreventingMutex::new(42, 1, Duration::from_secs(1));
        let guard = m.lock_with_timeout().await.unwrap();
        assert_eq!(*guard, 42);
    }

    #[tokio::test]
    async fn mutex_lock_timeout() {
        let m = Arc::new(DeadlockPreventingMutex::new(0, 1, Duration::from_millis(50)));
        // Hold the lock in the background.
        let m2 = Arc::clone(&m);
        let _hold = tokio::spawn(async move {
            let _guard = m2.lock_with_timeout().await.unwrap();
            tokio::time::sleep(Duration::from_secs(5)).await;
        });
        // Give the spawned task time to acquire.
        tokio::time::sleep(Duration::from_millis(10)).await;

        // This should timeout.
        let result = m.lock_with_timeout().await;
        assert!(result.is_err());
    }

    // ---------- AsyncBarrier ----------

    #[tokio::test]
    async fn barrier_releases_all() {
        let barrier = Arc::new(AsyncBarrier::new(3));
        let mut handles = Vec::new();
        for _ in 0..3 {
            let b = Arc::clone(&barrier);
            handles.push(tokio::spawn(async move {
                b.wait().await;
                true
            }));
        }
        for h in handles {
            assert!(h.await.unwrap());
        }
    }

    // ---------- CountdownLatch ----------

    #[tokio::test]
    async fn latch_count_down_to_zero() {
        let latch = Arc::new(CountdownLatch::new(3));
        assert_eq!(latch.remaining(), 3);

        latch.count_down();
        assert_eq!(latch.remaining(), 2);
        latch.count_down();
        assert_eq!(latch.remaining(), 1);
        latch.count_down();
        assert_eq!(latch.remaining(), 0);
    }

    #[tokio::test]
    async fn latch_wait_blocks_until_zero() {
        let latch = Arc::new(CountdownLatch::new(2));
        let latch2 = Arc::clone(&latch);

        let waiter = tokio::spawn(async move {
            latch2.wait().await;
            true
        });

        // Count down from another context.
        latch.count_down();
        // Small delay to ensure the waiter is actually waiting.
        tokio::time::sleep(Duration::from_millis(10)).await;
        latch.count_down();

        assert!(waiter.await.unwrap());
    }

    #[tokio::test]
    async fn latch_zero_count_returns_immediately() {
        let latch = CountdownLatch::new(0);
        // Should not hang.
        latch.wait().await;
        assert_eq!(latch.remaining(), 0);
    }

    #[test]
    fn debug_impls() {
        let m = DeadlockPreventingMutex::new(0i32, 1, Duration::from_secs(1));
        assert!(format!("{m:?}").contains("DeadlockPreventingMutex"));

        let b = AsyncBarrier::new(2);
        assert!(format!("{b:?}").contains("AsyncBarrier"));

        let l = CountdownLatch::new(5);
        let debug = format!("{l:?}");
        assert!(debug.contains("CountdownLatch"));
        assert!(debug.contains("5"));
    }
}
