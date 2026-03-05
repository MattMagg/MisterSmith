//! Health check implementations for persistence backends.
//!
//! Provides standalone health check functions for each backend and a
//! [`PersistenceHealthChecker`] that aggregates results across all configured
//! stores, returning a composite [`HealthStatus`].
//!
//! # Feature gates
//!
//! - The PostgreSQL health check is only available when the `sqlx` feature is
//!   enabled (default).
//! - The KV health check uses the JetStream `Context` directly, querying
//!   account info as a lightweight connectivity probe.

use mister_smith_core::HealthStatus;
use tracing::{debug, warn};

// ---------------------------------------------------------------------------
// Standalone health check: PostgreSQL
// ---------------------------------------------------------------------------

/// Check health of a PostgreSQL connection pool.
///
/// Executes `SELECT 1` against the pool. Returns [`HealthStatus::Healthy`] on
/// success, [`HealthStatus::Unhealthy`] on any error (connection timeout, pool
/// closed, query failure, etc.).
#[cfg(feature = "sqlx")]
pub async fn check_postgres_health(pool: &sqlx::PgPool) -> HealthStatus {
    match sqlx::query("SELECT 1").execute(pool).await {
        Ok(_) => {
            debug!("postgres health check passed");
            HealthStatus::Healthy
        }
        Err(err) => {
            warn!(error = %err, "postgres health check failed");
            HealthStatus::Unhealthy
        }
    }
}

// ---------------------------------------------------------------------------
// Standalone health check: JetStream KV
// ---------------------------------------------------------------------------

/// Check health of the JetStream KV backend.
///
/// Calls [`query_account`](async_nats::jetstream::Context::query_account) on
/// the JetStream context as a lightweight connectivity probe. This verifies
/// that the NATS connection is alive and the JetStream subsystem is reachable,
/// without requiring any specific bucket to exist.
///
/// Returns [`HealthStatus::Healthy`] on success, [`HealthStatus::Unhealthy`]
/// on any error.
pub async fn check_kv_health(context: &async_nats::jetstream::Context) -> HealthStatus {
    match context.query_account().await {
        Ok(_) => {
            debug!("kv health check passed");
            HealthStatus::Healthy
        }
        Err(err) => {
            warn!(error = %err, "kv health check failed");
            HealthStatus::Unhealthy
        }
    }
}

// ---------------------------------------------------------------------------
// PersistenceHealthChecker
// ---------------------------------------------------------------------------

/// Aggregated health checker for all configured persistence backends.
///
/// Stores optional handles to the PostgreSQL pool and JetStream context, then
/// probes whichever backends are configured when [`check_all`](Self::check_all)
/// is called.
///
/// # Composite status logic
///
/// | PG configured | KV configured | Result                                       |
/// |:--------------|:--------------|:---------------------------------------------|
/// | No            | No            | `Unknown` (nothing to check)                 |
/// | Yes (only)    | No            | PG status directly                           |
/// | No            | Yes (only)    | KV status directly                           |
/// | Yes           | Yes           | `Healthy` if both healthy, `Degraded` if one |
/// |               |               | healthy and one not, `Unhealthy` if none      |
pub struct PersistenceHealthChecker {
    /// PostgreSQL connection pool (PgPool is Clone — Arc internally).
    #[cfg(feature = "sqlx")]
    pg_pool: Option<sqlx::PgPool>,

    /// JetStream context for KV health probing (Context is Clone).
    kv_context: Option<async_nats::jetstream::Context>,
}

impl PersistenceHealthChecker {
    /// Create a new checker with both backends optionally configured.
    ///
    /// Pass `None` for any backend that is not in use.
    #[cfg(feature = "sqlx")]
    pub fn new(
        pg_pool: Option<sqlx::PgPool>,
        kv_context: Option<async_nats::jetstream::Context>,
    ) -> Self {
        Self {
            pg_pool,
            kv_context,
        }
    }

    /// Create a new checker without PostgreSQL support (sqlx feature disabled).
    #[cfg(not(feature = "sqlx"))]
    pub fn new(kv_context: Option<async_nats::jetstream::Context>) -> Self {
        Self { kv_context }
    }

    /// Check all configured backends and return a composite [`HealthStatus`].
    ///
    /// - If no backends are configured, returns [`HealthStatus::Unknown`].
    /// - If only one backend is configured, returns its status directly.
    /// - If both are configured, returns:
    ///   - [`HealthStatus::Healthy`] when all are healthy,
    ///   - [`HealthStatus::Degraded`] when at least one is healthy but not all,
    ///   - [`HealthStatus::Unhealthy`] when none are healthy.
    pub async fn check_all(&self) -> HealthStatus {
        let mut results: Vec<HealthStatus> = Vec::new();

        #[cfg(feature = "sqlx")]
        if let Some(ref pool) = self.pg_pool {
            results.push(check_postgres_health(pool).await);
        }

        if let Some(ref ctx) = self.kv_context {
            results.push(check_kv_health(ctx).await);
        }

        composite_status(&results)
    }
}

/// Compute a composite [`HealthStatus`] from a slice of individual results.
///
/// - Empty slice => `Unknown`
/// - All `Healthy` => `Healthy`
/// - None `Healthy` => `Unhealthy`
/// - Mixed => `Degraded`
fn composite_status(results: &[HealthStatus]) -> HealthStatus {
    if results.is_empty() {
        return HealthStatus::Unknown;
    }

    let healthy_count = results
        .iter()
        .filter(|s| **s == HealthStatus::Healthy)
        .count();

    if healthy_count == results.len() {
        HealthStatus::Healthy
    } else if healthy_count == 0 {
        HealthStatus::Unhealthy
    } else {
        HealthStatus::Degraded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // composite_status unit tests (no external dependencies)
    // -----------------------------------------------------------------------

    #[test]
    fn composite_empty_is_unknown() {
        assert_eq!(composite_status(&[]), HealthStatus::Unknown);
    }

    #[test]
    fn composite_single_healthy() {
        assert_eq!(
            composite_status(&[HealthStatus::Healthy]),
            HealthStatus::Healthy
        );
    }

    #[test]
    fn composite_single_unhealthy() {
        assert_eq!(
            composite_status(&[HealthStatus::Unhealthy]),
            HealthStatus::Unhealthy
        );
    }

    #[test]
    fn composite_all_healthy() {
        assert_eq!(
            composite_status(&[HealthStatus::Healthy, HealthStatus::Healthy]),
            HealthStatus::Healthy
        );
    }

    #[test]
    fn composite_all_unhealthy() {
        assert_eq!(
            composite_status(&[HealthStatus::Unhealthy, HealthStatus::Unhealthy]),
            HealthStatus::Unhealthy
        );
    }

    #[test]
    fn composite_mixed_is_degraded() {
        assert_eq!(
            composite_status(&[HealthStatus::Healthy, HealthStatus::Unhealthy]),
            HealthStatus::Degraded
        );
    }

    #[test]
    fn composite_degraded_input_counts_as_not_healthy() {
        // A Degraded input is neither Healthy nor strictly Unhealthy, but the
        // composite logic counts only Healthy vs. not-Healthy.
        assert_eq!(
            composite_status(&[HealthStatus::Healthy, HealthStatus::Degraded]),
            HealthStatus::Degraded
        );
    }

    #[test]
    fn composite_all_degraded_inputs() {
        assert_eq!(
            composite_status(&[HealthStatus::Degraded, HealthStatus::Degraded]),
            HealthStatus::Unhealthy
        );
    }

    #[test]
    fn composite_unknown_input_counts_as_not_healthy() {
        assert_eq!(
            composite_status(&[HealthStatus::Healthy, HealthStatus::Unknown]),
            HealthStatus::Degraded
        );
    }

    // -----------------------------------------------------------------------
    // PersistenceHealthChecker with no backends configured
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn checker_no_backends_returns_unknown() {
        #[cfg(feature = "sqlx")]
        let checker = PersistenceHealthChecker::new(None, None);
        #[cfg(not(feature = "sqlx"))]
        let checker = PersistenceHealthChecker::new(None);

        assert_eq!(checker.check_all().await, HealthStatus::Unknown);
    }
}
