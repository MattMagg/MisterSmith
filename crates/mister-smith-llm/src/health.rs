use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Per-provider health snapshot maintained in the data-plane routing table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub provider_id: String,
    pub circuit_state: CircuitState,
    pub consecutive_failures: u32,
    pub rolling_error_rate: f64,
    pub p95_latency_ms: u64,
    pub last_success: Option<u64>,
    pub rate_limit_until: Option<u64>,
}

/// Circuit breaker state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum CircuitState {
    #[default]
    Closed,
    Open,
    HalfOpen,
}

/// Configuration for the circuit breaker.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub error_rate_threshold: f64,
    pub recovery_timeout: Duration,
    pub half_open_max_probes: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            error_rate_threshold: 0.5,
            recovery_timeout: Duration::from_secs(30),
            half_open_max_probes: 1,
        }
    }
}

/// Circuit breaker managing provider health transitions.
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitState,
    consecutive_failures: u32,
    // Sliding window for error rate - store (timestamp_ms, is_error) tuples
    window: Vec<(u64, bool)>,
    window_duration: Duration,
    last_state_change: Instant,
    half_open_probes: u32,
    last_success_epoch_ms: Option<u64>,
    rate_limit_until_epoch_ms: Option<u64>,
    p95_latencies: Vec<u64>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: CircuitState::Closed,
            consecutive_failures: 0,
            window: Vec::new(),
            window_duration: Duration::from_secs(60),
            last_state_change: Instant::now(),
            half_open_probes: 0,
            last_success_epoch_ms: None,
            rate_limit_until_epoch_ms: None,
            p95_latencies: Vec::new(),
        }
    }

    pub fn state(&self) -> CircuitState {
        self.state
    }

    /// Check if requests should be allowed through.
    pub fn is_allowed(&self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if recovery timeout has elapsed
                self.last_state_change.elapsed() >= self.config.recovery_timeout
            }
            CircuitState::HalfOpen => self.half_open_probes < self.config.half_open_max_probes,
        }
    }

    /// Should auto-transition from Open to HalfOpen when recovery timeout elapsed.
    pub fn maybe_transition_to_half_open(&mut self) {
        if self.state == CircuitState::Open
            && self.last_state_change.elapsed() >= self.config.recovery_timeout
        {
            self.state = CircuitState::HalfOpen;
            self.half_open_probes = 0;
            self.last_state_change = Instant::now();
        }
    }

    /// Record a successful response.
    pub fn record_success(&mut self, latency_ms: u64) {
        let now_ms = epoch_ms();
        self.window.push((now_ms, false));
        self.prune_window(now_ms);
        self.consecutive_failures = 0;
        self.last_success_epoch_ms = Some(now_ms);
        self.rate_limit_until_epoch_ms = None;
        self.p95_latencies.push(latency_ms);
        if self.p95_latencies.len() > 100 {
            self.p95_latencies.remove(0);
        }

        if self.state == CircuitState::HalfOpen {
            // Probe succeeded — close the circuit
            self.state = CircuitState::Closed;
            self.last_state_change = Instant::now();
        }
    }

    /// Record a failed response.
    pub fn record_failure(&mut self, retry_after_secs: Option<u64>) {
        let now_ms = epoch_ms();
        self.window.push((now_ms, true));
        self.prune_window(now_ms);
        self.consecutive_failures += 1;

        if let Some(secs) = retry_after_secs {
            self.rate_limit_until_epoch_ms = Some(now_ms + secs * 1000);
        }

        match self.state {
            CircuitState::Closed => {
                if self.should_open() {
                    self.state = CircuitState::Open;
                    self.last_state_change = Instant::now();
                }
            }
            CircuitState::HalfOpen => {
                // Probe failed — reopen
                self.state = CircuitState::Open;
                self.last_state_change = Instant::now();
            }
            CircuitState::Open => {}
        }
    }

    fn should_open(&self) -> bool {
        self.consecutive_failures >= self.config.failure_threshold
            || self.rolling_error_rate() >= self.config.error_rate_threshold
    }

    fn rolling_error_rate(&self) -> f64 {
        if self.window.is_empty() {
            return 0.0;
        }
        let errors = self.window.iter().filter(|(_, is_error)| *is_error).count();
        errors as f64 / self.window.len() as f64
    }

    fn prune_window(&mut self, now_ms: u64) {
        let cutoff = now_ms.saturating_sub(self.window_duration.as_millis() as u64);
        self.window.retain(|(ts, _)| *ts >= cutoff);
    }

    /// Build a snapshot of current health.
    pub fn health_status(&self, provider_id: impl Into<String>) -> HealthStatus {
        let mut latencies = self.p95_latencies.clone();
        latencies.sort_unstable();
        let p95 = if latencies.is_empty() {
            0
        } else {
            let idx = (latencies.len() as f64 * 0.95) as usize;
            latencies[idx.min(latencies.len() - 1)]
        };

        HealthStatus {
            provider_id: provider_id.into(),
            circuit_state: self.state,
            consecutive_failures: self.consecutive_failures,
            rolling_error_rate: self.rolling_error_rate(),
            p95_latency_ms: p95,
            last_success: self.last_success_epoch_ms,
            rate_limit_until: self.rate_limit_until_epoch_ms,
        }
    }

    /// Check if this provider is rate-limited.
    pub fn is_rate_limited(&self) -> bool {
        self.rate_limit_until_epoch_ms
            .map(|until| epoch_ms() < until)
            .unwrap_or(false)
    }
}

fn epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circuit_starts_closed() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.is_allowed());
    }

    #[test]
    fn opens_after_threshold_failures() {
        let mut cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            error_rate_threshold: 1.1, // Disable error-rate trigger for this test
            ..Default::default()
        });
        cb.record_failure(None);
        cb.record_failure(None);
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure(None);
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn half_open_on_recovery_timeout() {
        let mut cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_millis(1),
            ..Default::default()
        });
        cb.record_failure(None);
        assert_eq!(cb.state(), CircuitState::Open);
        std::thread::sleep(Duration::from_millis(5));
        cb.maybe_transition_to_half_open();
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn half_open_closes_on_success() {
        let mut cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_millis(1),
            ..Default::default()
        });
        cb.record_failure(None);
        std::thread::sleep(Duration::from_millis(5));
        cb.maybe_transition_to_half_open();
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        cb.record_success(10);
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn half_open_reopens_on_failure() {
        let mut cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_millis(1),
            ..Default::default()
        });
        cb.record_failure(None);
        std::thread::sleep(Duration::from_millis(5));
        cb.maybe_transition_to_half_open();
        cb.record_failure(None);
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn retry_after_sets_rate_limit() {
        let mut cb = CircuitBreaker::new(CircuitBreakerConfig::default());
        cb.record_failure(Some(60));
        assert!(cb.is_rate_limited());
    }

    #[test]
    fn health_status_snapshot() {
        let mut cb = CircuitBreaker::new(CircuitBreakerConfig::default());
        cb.record_success(50);
        cb.record_success(100);
        let status = cb.health_status("test-provider");
        assert_eq!(status.provider_id, "test-provider");
        assert_eq!(status.circuit_state, CircuitState::Closed);
        assert_eq!(status.consecutive_failures, 0);
    }
}
