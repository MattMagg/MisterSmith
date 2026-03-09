//! Phi accrual failure detector.
//!
//! Implements the phi accrual failure detection algorithm, which computes a
//! suspicion level (phi) based on the statistical distribution of heartbeat
//! inter-arrival times. Higher phi values indicate greater suspicion that a
//! node has failed.
//!
//! Reference: Hayashibara et al., "The Phi Accrual Failure Detector" (2004).

use std::collections::{HashMap, VecDeque};
use std::time::Instant;
use tracing::debug;

/// Phi accrual failure detector.
///
/// Tracks heartbeat arrivals per node and computes a phi value representing
/// how suspicious it is that the node has failed. A phi above the configured
/// threshold marks the node as unavailable.
pub struct PhiAccrualFailureDetector {
    /// Per-node heartbeat arrival timestamps (most recent at the back).
    heartbeats: HashMap<String, VecDeque<Instant>>,
    /// Maximum number of heartbeats to retain per node.
    window_size: usize,
    /// Phi threshold above which a node is considered failed.
    phi_threshold: f64,
}

impl PhiAccrualFailureDetector {
    /// Create a new failure detector.
    ///
    /// * `phi_threshold` — phi value above which a node is considered
    ///   unavailable (commonly 8.0).
    /// * `window_size` — number of heartbeat samples to keep per node.
    pub fn new(phi_threshold: f64, window_size: usize) -> Self {
        Self {
            heartbeats: HashMap::new(),
            window_size,
            phi_threshold,
        }
    }

    /// Record a heartbeat from `node_id` at the current instant.
    pub fn record_heartbeat(&mut self, node_id: &str) {
        let entry = self
            .heartbeats
            .entry(node_id.to_string())
            .or_insert_with(|| VecDeque::with_capacity(self.window_size + 1));
        entry.push_back(Instant::now());

        // Keep only the most recent `window_size` entries.
        while entry.len() > self.window_size {
            entry.pop_front();
        }
    }

    /// Compute the phi value for `node_id`.
    ///
    /// Returns `None` if there are fewer than 2 heartbeats recorded (the
    /// algorithm needs at least one inter-arrival interval).
    ///
    /// The phi value is defined as:
    ///
    /// ```text
    /// phi = -log10(1 - F(t_now - t_last))
    /// ```
    ///
    /// where `F` is the CDF of a normal distribution fitted to observed
    /// inter-arrival times.
    pub fn phi(&self, node_id: &str) -> Option<f64> {
        let arrivals = self.heartbeats.get(node_id)?;
        if arrivals.len() < 2 {
            return None;
        }

        // Compute inter-arrival intervals in seconds.
        let intervals: Vec<f64> = arrivals
            .iter()
            .zip(arrivals.iter().skip(1))
            .map(|(a, b)| b.duration_since(*a).as_secs_f64())
            .collect();

        let n = intervals.len() as f64;
        let mean = intervals.iter().sum::<f64>() / n;

        // Variance with a small floor to avoid division by zero.
        let variance = intervals.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
        let stddev = variance.sqrt().max(1e-9);

        // Time elapsed since the last heartbeat.
        let last = arrivals.back()?;
        let elapsed = last.elapsed().as_secs_f64();

        // Normal CDF approximation: P(X <= elapsed).
        let y = (elapsed - mean) / stddev;
        let cdf = normal_cdf(y);

        // phi = -log10(1 - cdf), clamped so we don't produce infinity.
        let p_later = (1.0 - cdf).max(1e-15);
        let phi = -p_later.log10();

        debug!(
            node = node_id,
            phi,
            elapsed_ms = (elapsed * 1000.0) as u64,
            mean_ms = (mean * 1000.0) as u64,
            stddev_ms = (stddev * 1000.0) as u64,
            "Computed phi"
        );

        Some(phi)
    }

    /// Returns `true` if `node_id` is considered available (phi < threshold).
    ///
    /// If no heartbeats have been recorded for the node, returns `false`.
    pub fn is_available(&self, node_id: &str) -> bool {
        match self.phi(node_id) {
            Some(phi) => phi < self.phi_threshold,
            None => {
                // No data — consider unavailable if we've never heard from them,
                // but available if there's exactly one heartbeat (just started).
                self.heartbeats
                    .get(node_id)
                    .is_some_and(|arrivals| !arrivals.is_empty())
            }
        }
    }

    /// Returns the configured phi threshold.
    pub fn phi_threshold(&self) -> f64 {
        self.phi_threshold
    }

    /// Returns the configured window size.
    pub fn window_size(&self) -> usize {
        self.window_size
    }
}

/// Approximate the standard normal CDF using the logistic function.
///
/// This is a fast approximation accurate to ~0.001:
///
/// ```text
/// Phi(x) ~ 1 / (1 + exp(-1.7155 * x))
/// ```
fn normal_cdf(x: f64) -> f64 {
    1.0 / (1.0 + (-1.7155277699214135 * x).exp())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn no_heartbeats_returns_none() {
        let detector = PhiAccrualFailureDetector::new(8.0, 100);
        assert!(detector.phi("node-1").is_none());
    }

    #[test]
    fn single_heartbeat_returns_none() {
        let mut detector = PhiAccrualFailureDetector::new(8.0, 100);
        detector.record_heartbeat("node-1");
        // Need at least 2 heartbeats for an interval.
        assert!(detector.phi("node-1").is_none());
    }

    #[test]
    fn recent_heartbeats_yield_low_phi() {
        let mut detector = PhiAccrualFailureDetector::new(8.0, 100);
        // Record several heartbeats in quick succession.
        for i in 0..5 {
            detector.record_heartbeat("node-1");
            if i < 4 {
                thread::sleep(Duration::from_millis(10));
            }
        }

        let phi = detector.phi("node-1").expect("should have phi");
        // Phi should be low since we just heard from the node.
        assert!(phi < 8.0, "phi={phi} should be < 8.0");
    }

    #[test]
    fn is_available_with_recent_heartbeats() {
        let mut detector = PhiAccrualFailureDetector::new(8.0, 100);
        for i in 0..5 {
            detector.record_heartbeat("node-1");
            if i < 4 {
                thread::sleep(Duration::from_millis(10));
            }
        }
        assert!(detector.is_available("node-1"));
    }

    #[test]
    fn is_available_unknown_node() {
        let detector = PhiAccrualFailureDetector::new(8.0, 100);
        assert!(!detector.is_available("unknown"));
    }

    #[test]
    fn window_size_respected() {
        let mut detector = PhiAccrualFailureDetector::new(8.0, 5);
        for _ in 0..20 {
            detector.record_heartbeat("node-1");
        }
        let arrivals = detector.heartbeats.get("node-1").unwrap();
        assert_eq!(arrivals.len(), 5);
    }

    #[test]
    fn phi_increases_with_silence() {
        let mut detector = PhiAccrualFailureDetector::new(8.0, 100);
        // Record heartbeats at ~10ms intervals.
        for i in 0..10 {
            detector.record_heartbeat("node-1");
            if i < 9 {
                thread::sleep(Duration::from_millis(10));
            }
        }
        let phi_soon = detector.phi("node-1").unwrap();

        // Wait a bit longer without heartbeats.
        thread::sleep(Duration::from_millis(200));
        let phi_later = detector.phi("node-1").unwrap();

        assert!(
            phi_later > phi_soon,
            "phi should increase: soon={phi_soon}, later={phi_later}"
        );
    }

    #[test]
    fn normal_cdf_sanity() {
        // CDF(0) should be close to 0.5.
        let mid = normal_cdf(0.0);
        assert!((mid - 0.5).abs() < 0.01);

        // CDF(-inf) -> 0, CDF(+inf) -> 1.
        assert!(normal_cdf(-10.0) < 0.01);
        assert!(normal_cdf(10.0) > 0.99);
    }

    #[test]
    fn accessors() {
        let detector = PhiAccrualFailureDetector::new(10.0, 50);
        assert!((detector.phi_threshold() - 10.0).abs() < f64::EPSILON);
        assert_eq!(detector.window_size(), 50);
    }
}
