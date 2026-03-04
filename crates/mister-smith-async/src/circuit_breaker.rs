//! Circuit breaker pattern implementation.
//!
//! The circuit breaker prevents cascading failures by tracking error rates and
//! temporarily rejecting calls when a failure threshold is exceeded.
//!
//! ## State Machine
//!
//! ```text
//! ┌────────┐  failure_threshold   ┌──────┐  recovery_timeout  ┌──────────┐
//! │ Closed │ ─────────────────> │ Open │ ─────────────────> │ HalfOpen │
//! └────────┘                     └──────┘                     └──────────┘
//!      ^                                                          │
//!      │ ←── success ────────────────────────────────────────────┘
//!      │                                                          │
//!      └── ←── failure (back to Open) ───────────────────────────┘
//! ```
//!
//! Uses `std::sync` primitives (not `tokio::sync`) so the circuit breaker can
//! be queried from both sync and async contexts without requiring `.await`.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, RwLock};
use std::time::{Duration, Instant};

/// Circuit breaker states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed — calls pass through normally.
    Closed,
    /// Circuit is open — calls are rejected immediately.
    Open,
    /// Circuit is half-open — a limited number of probe calls are allowed.
    HalfOpen,
}

/// Thread-safe circuit breaker with automatic state transitions.
///
/// All internal state is guarded by `std::sync` primitives, making the breaker
/// usable from both synchronous and asynchronous code paths.
pub struct CircuitBreaker {
    failure_count: AtomicU32,
    last_failure_time: Mutex<Option<Instant>>,
    state: RwLock<CircuitState>,
    failure_threshold: u32,
    recovery_timeout: Duration,
    half_open_max_calls: u32,
    half_open_calls: AtomicU32,
}

impl std::fmt::Debug for CircuitBreaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CircuitBreaker")
            .field("failure_count", &self.failure_count.load(Ordering::Relaxed))
            .field("state", &*self.state.read().unwrap())
            .field("failure_threshold", &self.failure_threshold)
            .field("recovery_timeout", &self.recovery_timeout)
            .field("half_open_max_calls", &self.half_open_max_calls)
            .field(
                "half_open_calls",
                &self.half_open_calls.load(Ordering::Relaxed),
            )
            .finish()
    }
}

impl CircuitBreaker {
    /// Create a new circuit breaker.
    ///
    /// # Arguments
    ///
    /// * `failure_threshold` — Number of consecutive failures before opening the circuit.
    /// * `recovery_timeout` — How long to stay open before transitioning to half-open.
    /// * `half_open_max_calls` — Maximum probe calls allowed in half-open state.
    pub fn new(
        failure_threshold: u32,
        recovery_timeout: Duration,
        half_open_max_calls: u32,
    ) -> Self {
        Self {
            failure_count: AtomicU32::new(0),
            last_failure_time: Mutex::new(None),
            state: RwLock::new(CircuitState::Closed),
            failure_threshold,
            recovery_timeout,
            half_open_max_calls,
            half_open_calls: AtomicU32::new(0),
        }
    }

    /// Check whether a call should be allowed through.
    ///
    /// Returns `true` if the circuit is closed or half-open (and under the
    /// probe call limit). Automatically transitions from open to half-open
    /// once the recovery timeout has elapsed.
    pub fn can_proceed(&self) -> bool {
        let current_state = *self.state.read().unwrap();
        match current_state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if recovery timeout has elapsed.
                let last_failure = self.last_failure_time.lock().unwrap();
                if let Some(t) = *last_failure {
                    if t.elapsed() >= self.recovery_timeout {
                        drop(last_failure); // release mutex before write lock
                        let mut state = self.state.write().unwrap();
                        // Double-check: another thread may have transitioned already.
                        if *state == CircuitState::Open {
                            *state = CircuitState::HalfOpen;
                            self.half_open_calls.store(0, Ordering::SeqCst);
                        }
                        // Now in half-open — allow the probe call.
                        self.half_open_calls.fetch_add(1, Ordering::SeqCst)
                            < self.half_open_max_calls
                    } else {
                        false
                    }
                } else {
                    // No failure recorded — shouldn't be open. Recover gracefully.
                    false
                }
            }
            CircuitState::HalfOpen => {
                self.half_open_calls.fetch_add(1, Ordering::SeqCst) < self.half_open_max_calls
            }
        }
    }

    /// Record a successful call. Resets failure count and closes the circuit.
    pub fn record_success(&self) {
        self.failure_count.store(0, Ordering::SeqCst);
        self.half_open_calls.store(0, Ordering::SeqCst);
        let mut state = self.state.write().unwrap();
        *state = CircuitState::Closed;
    }

    /// Record a failed call. Increments the failure counter and opens the
    /// circuit if the threshold is reached.
    pub fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        *self.last_failure_time.lock().unwrap() = Some(Instant::now());

        if count >= self.failure_threshold {
            let mut state = self.state.write().unwrap();
            *state = CircuitState::Open;
            self.half_open_calls.store(0, Ordering::SeqCst);
        }
    }

    /// Current state of the circuit breaker.
    pub fn state(&self) -> CircuitState {
        *self.state.read().unwrap()
    }

    /// Current failure count.
    pub fn failure_count(&self) -> u32 {
        self.failure_count.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_closed() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(5), 1);
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.failure_count(), 0);
        assert!(cb.can_proceed());
    }

    #[test]
    fn opens_after_threshold() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(5), 1);
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.can_proceed());

        cb.record_failure(); // 3rd failure — reaches threshold.
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.can_proceed());
    }

    #[test]
    fn success_resets() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(5), 1);
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.failure_count(), 2);

        cb.record_success();
        assert_eq!(cb.failure_count(), 0);
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn transitions_to_half_open_after_timeout() {
        let cb = CircuitBreaker::new(1, Duration::from_millis(10), 1);
        cb.record_failure(); // opens
        assert_eq!(cb.state(), CircuitState::Open);

        // Wait for recovery timeout to elapse.
        std::thread::sleep(Duration::from_millis(20));

        // can_proceed should transition to HalfOpen.
        assert!(cb.can_proceed());
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn half_open_limits_calls() {
        let cb = CircuitBreaker::new(1, Duration::from_millis(10), 2);
        cb.record_failure(); // opens
        std::thread::sleep(Duration::from_millis(20));

        // First two calls in half-open should succeed.
        assert!(cb.can_proceed());
        assert!(cb.can_proceed());
        // Third call should be rejected (half_open_max_calls = 2).
        assert!(!cb.can_proceed());
    }

    #[test]
    fn half_open_success_closes() {
        let cb = CircuitBreaker::new(1, Duration::from_millis(10), 1);
        cb.record_failure(); // opens
        std::thread::sleep(Duration::from_millis(20));

        assert!(cb.can_proceed()); // transitions to HalfOpen
        cb.record_success(); // back to Closed
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.can_proceed());
    }

    #[test]
    fn half_open_failure_reopens() {
        let cb = CircuitBreaker::new(1, Duration::from_millis(10), 1);
        cb.record_failure(); // opens
        std::thread::sleep(Duration::from_millis(20));

        assert!(cb.can_proceed()); // transitions to HalfOpen
        cb.record_failure(); // back to Open
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.can_proceed());
    }

    #[test]
    fn debug_impl() {
        let cb = CircuitBreaker::new(5, Duration::from_secs(30), 2);
        let debug = format!("{cb:?}");
        assert!(debug.contains("CircuitBreaker"));
        assert!(debug.contains("failure_threshold: 5"));
    }
}
