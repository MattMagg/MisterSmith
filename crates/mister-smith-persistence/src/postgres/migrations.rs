//! Database migration runner using sqlx's built-in migration framework.
//!
//! Wraps `sqlx::migrate!()` to provide a typed interface with status reporting,
//! health-check-compatible verification, and single-step rollback via embedded
//! down-migration SQL.

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use mister_smith_core::PersistenceError;

// ---------------------------------------------------------------------------
// Embedded down-migration SQL (sqlx::migrate!() only supports up-migrations)
// ---------------------------------------------------------------------------

/// Down-migration for 00001_initial_schema: drop tables, types, schemas.
const DOWN_00001: &str = include_str!("../../migrations/00001_initial_schema.down.sql");

/// Down-migration for 00002_indexes: drop performance indexes.
const DOWN_00002: &str = include_str!("../../migrations/00002_indexes.down.sql");

/// Down-migration for 00003_partitions: drop partition functions and indexes.
const DOWN_00003: &str = include_str!("../../migrations/00003_partitions.down.sql");

/// Down-migration for 00004_audit_schema: drop audit log table and indexes.
const DOWN_00004: &str = include_str!("../../migrations/00004_audit_schema.down.sql");

/// Down-migration for 00005_message_idempotency: remove dedup column and indexes.
const DOWN_00005: &str = include_str!("../../migrations/00005_message_idempotency.down.sql");

/// Look up the embedded down-migration SQL for a given version.
///
/// Returns `None` if no down-migration exists for the version.
fn down_migration_sql(version: i64) -> Option<&'static str> {
    match version {
        1 => Some(DOWN_00001),
        2 => Some(DOWN_00002),
        3 => Some(DOWN_00003),
        4 => Some(DOWN_00004),
        5 => Some(DOWN_00005),
        _ => None,
    }
}

/// Status of an individual migration.
#[derive(Debug, Clone)]
pub struct MigrationStatus {
    /// Migration version number.
    pub version: i64,
    /// Human-readable description extracted from filename.
    pub description: String,
    /// Whether this migration has been applied.
    pub applied: bool,
    /// When the migration was applied (if applied).
    pub applied_at: Option<DateTime<Utc>>,
    /// Checksum of the migration SQL for integrity verification.
    pub checksum: String,
}

/// Runs database migrations and reports status.
///
/// Uses sqlx's compile-time embedded migrations via `sqlx::migrate!()`.
/// Migrations are baked into the binary — no runtime file access needed.
#[derive(Clone)]
pub struct MigrationRunner {
    pool: PgPool,
}

impl MigrationRunner {
    /// Create from a PgPool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Run all pending migrations. Returns count of applied migrations.
    ///
    /// Idempotent — running on an already-migrated database is a no-op.
    pub async fn run(&self) -> Result<usize, PersistenceError> {
        let migrator = sqlx::migrate!("./migrations");
        let before = self.applied_count().await?;

        migrator
            .run(&self.pool)
            .await
            .map_err(|e| PersistenceError::MigrationFailed(e.to_string()))?;

        let after = self.applied_count().await?;
        Ok(after.saturating_sub(before))
    }

    /// Check current migration version without applying anything.
    ///
    /// Returns the highest applied migration version, or `None` if no
    /// migrations have been applied.
    pub async fn current_version(&self) -> Result<Option<i64>, PersistenceError> {
        if !self.migration_table_exists().await? {
            return Ok(None);
        }

        let row: Option<(i64,)> =
            sqlx::query_as("SELECT version FROM _sqlx_migrations ORDER BY version DESC LIMIT 1")
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| PersistenceError::MigrationFailed(e.to_string()))?;

        Ok(row.map(|(v,)| v))
    }

    /// List all migrations and their applied status.
    pub async fn status(&self) -> Result<Vec<MigrationStatus>, PersistenceError> {
        let migrator = sqlx::migrate!("./migrations");
        let applied: Vec<(i64, String, bool, Vec<u8>)> = if self.migration_table_exists().await? {
            sqlx::query_as(
                "SELECT version, description, success, checksum FROM _sqlx_migrations ORDER BY version",
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| PersistenceError::MigrationFailed(e.to_string()))?
        } else {
            Vec::new()
        };

        let applied_map: std::collections::HashMap<i64, bool> =
            applied.into_iter().map(|(v, _, s, _)| (v, s)).collect();

        let mut statuses = Vec::new();
        for migration in migrator.iter() {
            statuses.push(MigrationStatus {
                version: migration.version,
                description: migration.description.to_string(),
                applied: applied_map
                    .get(&migration.version)
                    .copied()
                    .unwrap_or(false),
                applied_at: None, // sqlx doesn't expose applied_at directly in this query
                checksum: hex::encode(&migration.checksum),
            });
        }

        Ok(statuses)
    }

    /// Verify all migrations have been applied (health check use).
    ///
    /// Returns `true` when the migration table exists, at least one migration
    /// has been applied successfully, and no recorded migration failed.
    pub async fn verify(&self) -> Result<bool, PersistenceError> {
        if !self.migration_table_exists().await? {
            return Ok(false);
        }

        let row: (i64, i64) = sqlx::query_as(
            "SELECT \
                COUNT(*) FILTER (WHERE success = true), \
                COUNT(*) FILTER (WHERE success = false) \
             FROM _sqlx_migrations",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| PersistenceError::MigrationFailed(e.to_string()))?;

        Ok(row.0 > 0 && row.1 == 0)
    }

    /// Revert the latest applied migration.
    ///
    /// Reads the highest-version row from `_sqlx_migrations`, finds the
    /// corresponding `.down.sql` content, executes it, then removes the row.
    ///
    /// Returns the version number that was reverted.
    ///
    /// # Errors
    ///
    /// - [`PersistenceError::MigrationFailed`] if no migrations are applied,
    ///   if no down-migration exists for the version, or if the SQL fails.
    pub async fn revert(&self) -> Result<i64, PersistenceError> {
        // Find the latest applied migration version
        let version = self.current_version().await?.ok_or_else(|| {
            PersistenceError::MigrationFailed("No migrations to revert".to_string())
        })?;

        // Look up the embedded down-migration SQL
        let sql = down_migration_sql(version).ok_or_else(|| {
            PersistenceError::MigrationFailed(format!(
                "No down-migration found for version {version}"
            ))
        })?;

        // Execute the down-migration in a transaction
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| PersistenceError::MigrationFailed(e.to_string()))?;

        sqlx::query(sql).execute(&mut *tx).await.map_err(|e| {
            PersistenceError::MigrationFailed(format!(
                "Down-migration for version {version} failed: {e}"
            ))
        })?;

        // Remove the migration record
        sqlx::query("DELETE FROM _sqlx_migrations WHERE version = $1")
            .bind(version)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                PersistenceError::MigrationFailed(format!(
                    "Failed to remove migration record for version {version}: {e}"
                ))
            })?;

        tx.commit()
            .await
            .map_err(|e| PersistenceError::MigrationFailed(e.to_string()))?;

        Ok(version)
    }

    /// Count successfully applied migrations.
    async fn applied_count(&self) -> Result<usize, PersistenceError> {
        if !self.migration_table_exists().await? {
            return Ok(0);
        }

        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM _sqlx_migrations WHERE success = true")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| PersistenceError::MigrationFailed(e.to_string()))?;

        Ok(row.0 as usize)
    }

    async fn migration_table_exists(&self) -> Result<bool, PersistenceError> {
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND table_name = '_sqlx_migrations')",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| PersistenceError::MigrationFailed(e.to_string()))?;

        Ok(row.0)
    }
}

#[cfg(test)]
mod perf_tests {
    // use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_migration_lookup() {
        // Mock 10,000 applied migrations
        let mut applied: Vec<(i64, String, bool, Vec<u8>)> = Vec::new();
        for i in 1..=10_000 {
            applied.push((i as i64, format!("migration_{}", i), true, vec![]));
        }

        // Mock 10,000 embedded migrations to look up
        let to_lookup: Vec<i64> = (1..=10_000).collect();

        let start = Instant::now();
        let mut found_count = 0;
        for version in &to_lookup {
            let applied_info = applied.iter().find(|(v, _, _, _)| *v == *version);
            if applied_info.is_some() {
                found_count += 1;
            }
        }
        let elapsed_linear = start.elapsed();
        println!(
            "Linear search found {} in {:?}",
            found_count, elapsed_linear
        );

        let start = Instant::now();
        let applied_map: std::collections::HashMap<i64, bool> = applied
            .clone()
            .into_iter()
            .map(|(v, _, s, _)| (v, s))
            .collect();

        let mut found_count_hashmap = 0;
        for version in &to_lookup {
            if applied_map.contains_key(version) {
                found_count_hashmap += 1;
            }
        }
        let elapsed_hashmap = start.elapsed();
        println!(
            "Hashmap search found {} in {:?}",
            found_count_hashmap, elapsed_hashmap
        );
    }
}
