//! Pool sizing algorithms using Little's Law.
//!
//! Provides [`ConnectionPoolSizer`] for calculating optimal pool sizes based on
//! throughput, latency, and concurrency parameters, plus pre-built
//! [`PoolSizeTemplate`]s for common deployment environments.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Recommendation
// ---------------------------------------------------------------------------

/// Result of a pool sizing calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolSizeRecommendation {
    /// Calculated optimal pool size.
    pub recommended_size: usize,
    /// Suggested minimum connections (floor).
    pub min_connections: usize,
    /// Suggested maximum connections (ceiling).
    pub max_connections: usize,
    /// Human-readable explanation of how the values were derived.
    pub reasoning: String,
}

// ---------------------------------------------------------------------------
// Sizer
// ---------------------------------------------------------------------------

/// Stateless calculator for optimal connection pool sizes.
///
/// Uses Little's Law: `L = lambda * W` where
/// - `L` = average number of concurrent connections needed,
/// - `lambda` = operations per second,
/// - `W` = average operation duration.
///
/// The result is adjusted for target utilization and an agent concurrency factor.
pub struct ConnectionPoolSizer;

impl ConnectionPoolSizer {
    /// Calculate an optimal pool size.
    ///
    /// # Arguments
    ///
    /// - `ops_per_sec` — expected throughput in operations/second.
    /// - `avg_duration` — average duration of a single operation.
    /// - `target_utilization` — desired pool utilization ratio (0.0, 1.0].
    ///   Values outside this range are clamped.
    /// - `agent_count` — number of agents sharing this pool.
    ///
    /// # Returns
    ///
    /// A [`PoolSizeRecommendation`] with `recommended_size`, `min_connections`,
    /// and `max_connections`.
    pub fn calculate_optimal_pool_size(
        ops_per_sec: f64,
        avg_duration: std::time::Duration,
        target_utilization: f64,
        agent_count: usize,
    ) -> PoolSizeRecommendation {
        let utilization = target_utilization.clamp(0.01, 1.0);
        let avg_secs = avg_duration.as_secs_f64();
        let concurrency_factor = Self::calculate_agent_concurrency_factor(agent_count);

        // Little's Law: L = lambda * W, adjusted for utilization and concurrency.
        let raw = (ops_per_sec * avg_secs) / utilization * concurrency_factor;
        let pool_size = (raw.ceil() as usize).max(1);

        let min_connections = (pool_size / 4).max(1);
        let max_connections = pool_size * 2;

        let reasoning = format!(
            "Little's Law: ceil(({ops_per_sec:.1} ops/s * {avg_secs:.3}s) / {utilization:.2} \
             * {concurrency_factor:.2}) = {pool_size}  \
             [agents={agent_count}, factor={concurrency_factor:.2}]"
        );

        PoolSizeRecommendation {
            recommended_size: pool_size,
            min_connections,
            max_connections,
            reasoning,
        }
    }

    /// Agent concurrency factor: a multiplier that accounts for contention
    /// as the number of agents sharing a pool increases.
    ///
    /// - 1..=5 agents  -> 1.0 (no contention overhead)
    /// - 6..=20 agents -> 0.8
    /// - 21+ agents    -> 0.6
    pub fn calculate_agent_concurrency_factor(agent_count: usize) -> f64 {
        match agent_count {
            0..=5 => 1.0,
            6..=20 => 0.8,
            _ => 0.6,
        }
    }
}

// ---------------------------------------------------------------------------
// Templates
// ---------------------------------------------------------------------------

/// Pre-built pool sizing template for a deployment environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolSizeTemplate {
    /// Template name (e.g. "development", "staging", "production").
    pub name: String,
    /// Minimum idle connections.
    pub min_size: usize,
    /// Maximum total connections.
    pub max_size: usize,
    /// Idle timeout in seconds.
    pub idle_timeout_secs: u64,
}

/// Deployment environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    /// Local development.
    Development,
    /// Pre-production staging.
    Staging,
    /// Production deployment.
    Production,
}

/// Return a sensible [`PoolSizeTemplate`] for the given environment.
pub fn get_environment_template(env: Environment) -> PoolSizeTemplate {
    match env {
        Environment::Development => PoolSizeTemplate {
            name: "development".to_string(),
            min_size: 1,
            max_size: 5,
            idle_timeout_secs: 60,
        },
        Environment::Staging => PoolSizeTemplate {
            name: "staging".to_string(),
            min_size: 2,
            max_size: 20,
            idle_timeout_secs: 300,
        },
        Environment::Production => PoolSizeTemplate {
            name: "production".to_string(),
            min_size: 5,
            max_size: 50,
            idle_timeout_secs: 600,
        },
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn concurrency_factor_low_agent_count() {
        assert_eq!(
            ConnectionPoolSizer::calculate_agent_concurrency_factor(1),
            1.0
        );
        assert_eq!(
            ConnectionPoolSizer::calculate_agent_concurrency_factor(5),
            1.0
        );
    }

    #[test]
    fn concurrency_factor_medium_agent_count() {
        assert_eq!(
            ConnectionPoolSizer::calculate_agent_concurrency_factor(6),
            0.8
        );
        assert_eq!(
            ConnectionPoolSizer::calculate_agent_concurrency_factor(20),
            0.8
        );
    }

    #[test]
    fn concurrency_factor_high_agent_count() {
        assert_eq!(
            ConnectionPoolSizer::calculate_agent_concurrency_factor(21),
            0.6
        );
        assert_eq!(
            ConnectionPoolSizer::calculate_agent_concurrency_factor(100),
            0.6
        );
    }

    #[test]
    fn concurrency_factor_zero_agents() {
        assert_eq!(
            ConnectionPoolSizer::calculate_agent_concurrency_factor(0),
            1.0
        );
    }

    #[test]
    fn basic_pool_sizing() {
        // 100 ops/s, 50ms avg, 80% utilization, 3 agents.
        let rec = ConnectionPoolSizer::calculate_optimal_pool_size(
            100.0,
            Duration::from_millis(50),
            0.8,
            3,
        );
        // L = (100 * 0.05) / 0.8 * 1.0 = 6.25 -> ceil = 7
        assert_eq!(rec.recommended_size, 7);
        assert_eq!(rec.min_connections, 1); // 7/4 = 1
        assert_eq!(rec.max_connections, 14); // 7*2 = 14
    }

    #[test]
    fn pool_sizing_with_many_agents() {
        // 100 ops/s, 50ms avg, 80% utilization, 30 agents.
        let rec = ConnectionPoolSizer::calculate_optimal_pool_size(
            100.0,
            Duration::from_millis(50),
            0.8,
            30,
        );
        // L = (100 * 0.05) / 0.8 * 0.6 = 3.75 -> ceil = 4
        assert_eq!(rec.recommended_size, 4);
        assert_eq!(rec.min_connections, 1);
        assert_eq!(rec.max_connections, 8);
    }

    #[test]
    fn pool_sizing_never_below_one() {
        let rec =
            ConnectionPoolSizer::calculate_optimal_pool_size(0.1, Duration::from_millis(1), 1.0, 1);
        assert!(rec.recommended_size >= 1);
        assert!(rec.min_connections >= 1);
    }

    #[test]
    fn pool_sizing_clamps_utilization() {
        // Zero utilization gets clamped to 0.01.
        let rec = ConnectionPoolSizer::calculate_optimal_pool_size(
            100.0,
            Duration::from_millis(50),
            0.0,
            1,
        );
        // (100 * 0.05) / 0.01 * 1.0 = 500
        assert_eq!(rec.recommended_size, 500);
    }

    #[test]
    fn reasoning_is_populated() {
        let rec = ConnectionPoolSizer::calculate_optimal_pool_size(
            50.0,
            Duration::from_millis(100),
            0.75,
            10,
        );
        assert!(!rec.reasoning.is_empty());
        assert!(rec.reasoning.contains("Little's Law"));
    }

    #[test]
    fn environment_template_development() {
        let tpl = get_environment_template(Environment::Development);
        assert_eq!(tpl.name, "development");
        assert_eq!(tpl.min_size, 1);
        assert_eq!(tpl.max_size, 5);
        assert_eq!(tpl.idle_timeout_secs, 60);
    }

    #[test]
    fn environment_template_staging() {
        let tpl = get_environment_template(Environment::Staging);
        assert_eq!(tpl.name, "staging");
        assert_eq!(tpl.min_size, 2);
        assert_eq!(tpl.max_size, 20);
        assert_eq!(tpl.idle_timeout_secs, 300);
    }

    #[test]
    fn environment_template_production() {
        let tpl = get_environment_template(Environment::Production);
        assert_eq!(tpl.name, "production");
        assert_eq!(tpl.min_size, 5);
        assert_eq!(tpl.max_size, 50);
        assert_eq!(tpl.idle_timeout_secs, 600);
    }
}
