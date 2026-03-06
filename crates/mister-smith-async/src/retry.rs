//! Retry policies with configurable exponential backoff and jitter.
//!
//! [`RetryPolicy`] computes per-attempt delays using:
//!
//! ```text
//! delay = min(base_delay * multiplier^attempt, max_delay) +/- 10% jitter
//! ```

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Configurable retry policy with exponential backoff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of attempts (including the initial attempt).
    pub max_attempts: u32,
    /// Base delay between retries.
    pub base_delay: Duration,
    /// Maximum delay cap.
    pub max_delay: Duration,
    /// Multiplier applied to the delay for each subsequent attempt.
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryPolicy {
    /// Policy tuned for database operations: more attempts, moderate delay.
    pub fn for_database() -> Self {
        Self {
            max_attempts: 5,
            base_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }

    /// Policy tuned for network operations: fewer attempts, longer max delay.
    pub fn for_network() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }

    /// Compute the delay for a given attempt number (0-indexed).
    ///
    /// Applies exponential backoff capped at `max_delay`, then adds deterministic
    /// jitter of +/-10% based on the attempt number. The jitter is deterministic
    /// to keep behaviour reproducible in tests while still spreading retries in
    /// practice (different attempt numbers produce different jitter fractions).
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base_nanos = self.base_delay.as_nanos() as f64;
        let multiplied = base_nanos * self.backoff_multiplier.powi(attempt as i32);
        let max_nanos = self.max_delay.as_nanos() as f64;
        let capped = multiplied.min(max_nanos);

        // Deterministic jitter: map attempt to a value in [-0.10, +0.10].
        // Uses a simple hash-like approach: ((attempt * 7 + 3) % 20) / 20 maps
        // to [0.0, 0.95], then scale to [-0.10, +0.10].
        let jitter_input = ((attempt as u64).wrapping_mul(7).wrapping_add(3) % 20) as f64;
        let jitter_fraction = (jitter_input / 20.0) * 0.2 - 0.1; // range [-0.10, +0.10)
        let jittered = capped * (1.0 + jitter_fraction);

        // Ensure non-negative
        Duration::from_nanos(jittered.max(0.0) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy() {
        let p = RetryPolicy::default();
        assert_eq!(p.max_attempts, 3);
        assert_eq!(p.base_delay, Duration::from_millis(100));
        assert_eq!(p.max_delay, Duration::from_secs(30));
        assert!((p.backoff_multiplier - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn database_policy() {
        let p = RetryPolicy::for_database();
        assert_eq!(p.max_attempts, 5);
        assert_eq!(p.base_delay, Duration::from_millis(200));
        assert_eq!(p.max_delay, Duration::from_secs(10));
    }

    #[test]
    fn network_policy() {
        let p = RetryPolicy::for_network();
        assert_eq!(p.max_attempts, 3);
        assert_eq!(p.base_delay, Duration::from_millis(100));
        assert_eq!(p.max_delay, Duration::from_secs(30));
    }

    #[test]
    fn exponential_backoff_increases() {
        let p = RetryPolicy::default();
        let d0 = p.delay_for_attempt(0);
        let d1 = p.delay_for_attempt(1);
        let d2 = p.delay_for_attempt(2);

        // Each successive attempt should produce a larger delay (ignoring jitter noise).
        // With base=100ms and multiplier=2.0: ~100ms, ~200ms, ~400ms (before jitter).
        assert!(d1 > d0, "d1={d1:?} should be > d0={d0:?}");
        assert!(d2 > d1, "d2={d2:?} should be > d1={d1:?}");
    }

    #[test]
    fn delay_respects_max_cap() {
        let p = RetryPolicy {
            max_attempts: 10,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 10.0,
        };
        // Attempt 5: 1s * 10^5 = 100_000s, should be capped to ~5s +/- jitter.
        let d = p.delay_for_attempt(5);
        // With +/- 10% jitter of 5s, result should be in [4.5s, 5.5s].
        assert!(
            d >= Duration::from_millis(4500),
            "d={d:?} should be >= 4.5s"
        );
        assert!(
            d <= Duration::from_millis(5500),
            "d={d:?} should be <= 5.5s"
        );
    }

    #[test]
    fn jitter_is_deterministic() {
        let p = RetryPolicy::default();
        let d1 = p.delay_for_attempt(1);
        let d2 = p.delay_for_attempt(1);
        assert_eq!(d1, d2);
    }

    #[test]
    fn jitter_varies_across_attempts() {
        let p = RetryPolicy {
            max_attempts: 10,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 1.0, // constant base so we isolate jitter effect
        };
        // Different attempts should (usually) produce different jitter.
        // Not guaranteed for all pairs, but across several attempts we expect variation.
        let delays: Vec<Duration> = (0..10).map(|a| p.delay_for_attempt(a)).collect();
        let unique_count = {
            let mut v = delays.clone();
            v.dedup();
            v.len()
        };
        assert!(
            unique_count > 1,
            "Expected jitter variation, got {delays:?}"
        );
    }

    #[test]
    fn serde_roundtrip() {
        let p = RetryPolicy::for_database();
        let json = serde_json::to_string(&p).unwrap();
        let deserialized: RetryPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.max_attempts, p.max_attempts);
        assert_eq!(deserialized.base_delay, p.base_delay);
        assert_eq!(deserialized.max_delay, p.max_delay);
        assert!((deserialized.backoff_multiplier - p.backoff_multiplier).abs() < f64::EPSILON);
    }
}
