//! Audit repository for immutable audit log persistence.
//!
//! [`AuditRepository`] provides append-only operations for the audit log.
//! It does **not** implement [`Repository<T>`](super::Repository) because
//! audit entries are immutable — no updates or deletes are permitted.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use mister_smith_core::PersistenceError;

#[cfg(feature = "sqlx")]
use crate::postgres::queries::{self, AuditEntry};

/// Repository for audit log entries.
///
/// Audit entries are append-only. This repository intentionally omits
/// update and delete operations to preserve the immutable audit trail.
pub struct AuditRepository {
    #[cfg(feature = "sqlx")]
    pool: sqlx::PgPool,
}

impl AuditRepository {
    /// Create from a PG pool.
    #[cfg(feature = "sqlx")]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Append a single audit entry.
    #[cfg(feature = "sqlx")]
    pub async fn append(&self, entry: &AuditEntry) -> Result<(), PersistenceError> {
        queries::insert_audit_entry(&self.pool, entry).await
    }

    /// Batch append audit entries in a single transaction.
    ///
    /// Returns the number of entries inserted. Used by `AuditPersister`
    /// to flush the Phase 5 ring buffer to PostgreSQL.
    #[cfg(feature = "sqlx")]
    pub async fn append_batch(&self, entries: &[AuditEntry]) -> Result<usize, PersistenceError> {
        queries::insert_audit_batch(&self.pool, entries).await
    }

    /// Query audit entries by agent within a time range.
    #[cfg(feature = "sqlx")]
    pub async fn find_by_agent(
        &self,
        agent_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>, PersistenceError> {
        queries::find_audit_by_agent(&self.pool, agent_id, start, end).await
    }

    /// Start a new database transaction for multi-operation atomicity.
    #[cfg(feature = "sqlx")]
    pub async fn begin_transaction(
        &self,
    ) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, PersistenceError> {
        queries::begin_transaction(&self.pool).await
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn audit_repository_struct_is_constructible() {
        #[cfg(feature = "sqlx")]
        {
            fn _assert_send_sync<T: Send + Sync>() {}
            _assert_send_sync::<super::AuditRepository>();
        }
    }
}
