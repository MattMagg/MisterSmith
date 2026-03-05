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

/// Look up the embedded down-migration SQL for a given version.
///
/// Returns `None` if no down-migration exists for the version.
fn down_migration_sql(version: i64) -> Option<&'static str> {
    match version {
        1 => Some(DOWN_00001),
        2 => Some(DOWN_00002),
        3 => Some(DOWN_00003),
        4 => Some(DOWN_00004),
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
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT version FROM _sqlx_migrations ORDER BY version DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PersistenceError::MigrationFailed(e.to_string()))?;

        Ok(row.map(|(v,)| v))
    }

    /// List all migrations and their applied status.
    pub async fn status(&self) -> Result<Vec<MigrationStatus>, PersistenceError> {
        let migrator = sqlx::migrate!("./migrations");
        let applied: Vec<(i64, String, bool, Vec<u8>)> = sqlx::query_as(
            "SELECT version, description, success, checksum FROM _sqlx_migrations ORDER BY version",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PersistenceError::MigrationFailed(e.to_string()))?;

        let mut statuses = Vec::new();
        for migration in migrator.iter() {
            let applied_info = applied.iter().find(|(v, _, _, _)| *v == migration.version);
            statuses.push(MigrationStatus {
                version: migration.version,
                description: migration.description.to_string(),
                applied: applied_info.is_some_and(|(_, _, s, _)| *s),
                applied_at: None, // sqlx doesn't expose applied_at directly in this query
                checksum: hex::encode(&migration.checksum),
            });
        }

        Ok(statuses)
    }

    /// Verify all migrations have been applied (health check use).
    ///
    /// Returns `true` if every embedded migration has a corresponding
    /// successful entry in the `_sqlx_migrations` table.
    pub async fn verify(&self) -> Result<bool, PersistenceError> {
        let migrator = sqlx::migrate!("./migrations");
        let applied_count = self.applied_count().await?;
        Ok(applied_count >= migrator.iter().count())
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
        let version = self
            .current_version()
            .await?
            .ok_or_else(|| {
                PersistenceError::MigrationFailed(
                    "No migrations to revert".to_string(),
                )
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

        sqlx::query(sql)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
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
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM _sqlx_migrations WHERE success = true",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| PersistenceError::MigrationFailed(e.to_string()))?;

        Ok(row.0 as usize)
    }
}
