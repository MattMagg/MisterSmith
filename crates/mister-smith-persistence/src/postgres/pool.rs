//! PostgreSQL connection pool wrapping sqlx::PgPool.
//!
//! Implements the core `Resource` trait for framework lifecycle management.

use std::time::Duration;

use async_trait::async_trait;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use mister_smith_core::{HealthStatus, Resource, ResourceId};

use crate::config::PostgresConfig;
use crate::error::{from_sqlx_error, PersistenceError};

/// PostgreSQL connection pool managed as a framework [`Resource`].
///
/// Wraps a [`sqlx::PgPool`] and exposes it through the core `Resource` trait
/// for lifecycle management (acquire, release, health checks).
///
/// # Examples
///
/// ```rust,no_run
/// use mister_smith_persistence::postgres::pool::PostgresConnection;
/// use mister_smith_core::Resource;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let conn = PostgresConnection::connect("postgres://localhost/mydb").await?;
/// assert!(conn.is_healthy());
/// let pool = conn.pool();
/// // Use pool for queries...
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct PostgresConnection {
    pool: PgPool,
    id: ResourceId,
}

impl PostgresConnection {
    /// Create a connection pool with default settings from a database URL.
    ///
    /// Uses `PostgresConfig::default()` for pool sizing and timeouts,
    /// overriding only the URL.
    pub async fn connect(url: &str) -> Result<Self, PersistenceError> {
        let config = PostgresConfig {
            url: Some(url.to_string()),
            ..PostgresConfig::default()
        };
        Self::acquire(config).await
    }

    /// Returns a reference to the underlying [`PgPool`].
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl Resource for PostgresConnection {
    type Config = PostgresConfig;
    type Error = PersistenceError;

    async fn acquire(config: Self::Config) -> Result<Self, Self::Error> {
        let url = config
            .url
            .or_else(|| std::env::var("DATABASE_URL").ok())
            .ok_or_else(|| {
                PersistenceError::ConnectionFailed(
                    "no database URL: set PostgresConfig.url or DATABASE_URL env var".to_string(),
                )
            })?;

        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_secs(config.connect_timeout_secs))
            .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
            .connect(&url)
            .await
            .map_err(from_sqlx_error)?;

        Ok(Self {
            pool,
            id: ResourceId::new(),
        })
    }

    async fn release(self) -> Result<(), Self::Error> {
        self.pool.close().await;
        Ok(())
    }

    fn is_healthy(&self) -> bool {
        !self.pool.is_closed()
    }

    async fn health_check(&self) -> Result<HealthStatus, Self::Error> {
        match self.pool.acquire().await {
            Ok(_conn) => Ok(HealthStatus::Healthy),
            Err(err) => {
                tracing::warn!(resource_id = %self.id.0, error = %err, "postgres health check failed");
                Ok(HealthStatus::Unhealthy)
            }
        }
    }

    fn resource_id(&self) -> ResourceId {
        self.id
    }
}
