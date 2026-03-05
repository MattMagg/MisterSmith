//! Message repository for message record persistence.
//!
//! [`MessageRepository`] implements [`Repository<MessageRecord>`](super::Repository)
//! for CRUD operations on the message store, plus specialized methods for
//! querying messages by correlation ID and sender.

use async_trait::async_trait;
use uuid::Uuid;

use mister_smith_core::PersistenceError;

#[cfg(feature = "sqlx")]
use crate::postgres::queries::{self, MessageRecord};

use super::Repository;

/// Repository for message records.
///
/// Provides SQL-based CRUD operations (insert, find, update, delete)
/// plus specialized query methods for message retrieval by various criteria.
pub struct MessageRepository {
    #[cfg(feature = "sqlx")]
    pool: sqlx::PgPool,
}

impl MessageRepository {
    /// Create from a PG pool.
    #[cfg(feature = "sqlx")]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Start a new database transaction for multi-operation atomicity.
    #[cfg(feature = "sqlx")]
    pub async fn begin_transaction(
        &self,
    ) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, PersistenceError> {
        queries::begin_transaction(&self.pool).await
    }

    /// Find messages by correlation ID.
    #[cfg(feature = "sqlx")]
    pub async fn find_by_correlation(
        &self,
        correlation_id: Uuid,
    ) -> Result<Vec<MessageRecord>, PersistenceError> {
        queries::find_messages_by_correlation(&self.pool, correlation_id).await
    }

    /// Find messages sent by a given agent within a time range.
    #[cfg(feature = "sqlx")]
    pub async fn find_by_sender(
        &self,
        agent_id: Uuid,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<MessageRecord>, PersistenceError> {
        queries::find_messages_by_sender(&self.pool, agent_id, start, end).await
    }
}

#[cfg(feature = "sqlx")]
#[async_trait]
impl Repository<MessageRecord> for MessageRepository {
    async fn save(&self, entity: &MessageRecord) -> Result<MessageRecord, PersistenceError> {
        queries::insert_message(&self.pool, entity).await
    }

    async fn find(&self, id: &Uuid) -> Result<Option<MessageRecord>, PersistenceError> {
        queries::find_message(&self.pool, *id).await
    }

    async fn update(&self, entity: &MessageRecord) -> Result<MessageRecord, PersistenceError> {
        // Update status (the primary mutable field on the message record)
        queries::update_message_status(&self.pool, entity.id, &entity.status).await?;
        // Return the updated record from DB
        queries::find_message(&self.pool, entity.id)
            .await?
            .ok_or_else(|| {
                PersistenceError::NotFound(format!(
                    "Message {} not found after update",
                    entity.id
                ))
            })
    }

    async fn delete(&self, id: &Uuid) -> Result<bool, PersistenceError> {
        // Soft delete: mark as cancelled rather than hard-deleting
        match queries::update_message_status(&self.pool, *id, "cancelled").await {
            Ok(()) => Ok(true),
            Err(PersistenceError::NotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn message_repository_struct_is_constructible() {
        // Verify the struct definition compiles and fields are correct.
        // Actual DB tests are in postgres_tests.rs (T040).
        #[cfg(feature = "sqlx")]
        {
            // Type-level check: MessageRepository must be Send + Sync
            fn _assert_send_sync<T: Send + Sync>() {}
            _assert_send_sync::<super::MessageRepository>();
        }
    }
}
