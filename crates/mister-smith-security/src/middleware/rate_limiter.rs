//! Token-bucket rate limiter for security middleware.
//!
//! Tracks request counts per source (IP or agent ID) within a sliding time
//! window. Thread-safe via [`DashMap`].

use dashmap::DashMap;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Per-source token-bucket rate limiter.
///
/// Thread-safe (`Send + Sync`) — uses [`DashMap`] for concurrent access.
pub struct RateLimiter {
    /// Maximum requests per window per source.
    max_requests: u32,
    /// Time window for counting requests.
    window: Duration,
    /// Per-source request timestamps.
    entries: DashMap<String, VecDeque<Instant>>,
}

impl RateLimiter {
    /// Create a new rate limiter.
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            entries: DashMap::new(),
        }
    }

    /// Check if a request from the given source should be allowed.
    ///
    /// Returns `Ok(())` if allowed, or `Err(retry_after)` with the duration
    /// until the oldest request in the window expires.
    pub fn check(&self, source: &str) -> Result<(), Duration> {
        let now = Instant::now();
        let mut entry = self.entries.entry(source.to_string()).or_default();

        // Remove timestamps outside the window.
        while let Some(&front) = entry.front() {
            if now.duration_since(front) >= self.window {
                entry.pop_front();
            } else {
                break;
            }
        }

        if entry.len() >= self.max_requests as usize {
            // Calculate retry-after from the oldest entry in the window.
            let retry_after = self
                .window
                .checked_sub(now.duration_since(*entry.front().unwrap()))
                .unwrap_or(Duration::from_secs(1));
            return Err(retry_after);
        }

        entry.push_back(now);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_within_limit() {
        let limiter = RateLimiter::new(3, Duration::from_secs(60));
        assert!(limiter.check("src1").is_ok());
        assert!(limiter.check("src1").is_ok());
        assert!(limiter.check("src1").is_ok());
    }

    #[test]
    fn blocks_over_limit() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));
        assert!(limiter.check("src1").is_ok());
        assert!(limiter.check("src1").is_ok());
        assert!(limiter.check("src1").is_err());
    }

    #[test]
    fn per_source_isolation() {
        let limiter = RateLimiter::new(1, Duration::from_secs(60));
        assert!(limiter.check("a").is_ok());
        assert!(limiter.check("a").is_err());
        // Different source still allowed.
        assert!(limiter.check("b").is_ok());
    }
}
