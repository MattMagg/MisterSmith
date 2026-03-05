//! Task repository for task record persistence.
//!
//! [`TaskRepository`] implements [`Repository<TaskRecord>`](super::Repository)
//! for CRUD operations on the task registry, plus specialized methods for
//! querying tasks by agent, time range, and correlation ID.

use async_trait::async_trait;
use uuid::Uuid;

use mister_smith_core::PersistenceError;

#[cfg(feature = "sqlx")]
use crate::postgres::queries::{self, TaskRecord};

use super::Repository;

/// Repository for task records.
///
/// Provides SQL-based CRUD operations (insert, find, update, delete)
/// plus specialized query methods for task retrieval by various criteria.
pub struct TaskRepository {
    #[cfg(feature = "sqlx")]
    pool: sqlx::PgPool,
}

impl TaskRepository {
    /// Create from a PG pool.
    #[cfg(feature = "sqlx")]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Find tasks assigned to a given agent, optionally filtered by status.
    #[cfg(feature = "sqlx")]
    pub async fn find_by_agent(
        &self,
        agent_id: Uuid,
        status: Option<&str>,
    ) -> Result<Vec<TaskRecord>, PersistenceError> {
        let tasks = queries::find_tasks_by_agent(&self.pool, agent_id).await?;
        match status {
            Some(s) => Ok(tasks.into_iter().filter(|t| t.status == s).collect()),
            None => Ok(tasks),
        }
    }

    /// Find tasks within a time range (by `created_at`).
    #[cfg(feature = "sqlx")]
    pub async fn find_by_time_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<TaskRecord>, PersistenceError> {
        queries::find_tasks_by_time_range(&self.pool, start, end).await
    }

    /// Start a new database transaction for multi-operation atomicity.
    #[cfg(feature = "sqlx")]
    pub async fn begin_transaction(
        &self,
    ) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, PersistenceError> {
        queries::begin_transaction(&self.pool).await
    }

    /// Find tasks by correlation ID.
    #[cfg(feature = "sqlx")]
    pub async fn find_by_correlation(
        &self,
        correlation_id: Uuid,
    ) -> Result<Vec<TaskRecord>, PersistenceError> {
        queries::find_tasks_by_correlation(&self.pool, correlation_id).await
    }
}

#[cfg(feature = "sqlx")]
#[async_trait]
impl Repository<TaskRecord> for TaskRepository {
    async fn save(&self, entity: &TaskRecord) -> Result<TaskRecord, PersistenceError> {
        queries::insert_task(&self.pool, entity).await
    }

    async fn find(&self, id: &Uuid) -> Result<Option<TaskRecord>, PersistenceError> {
        queries::find_task(&self.pool, *id).await
    }

    async fn update(&self, entity: &TaskRecord) -> Result<TaskRecord, PersistenceError> {
        // Update status (the primary mutable field on the task record)
        queries::update_task_status(&self.pool, entity.task_id, &entity.status).await?;
        // Return the updated record from DB
        queries::find_task(&self.pool, entity.task_id)
            .await?
            .ok_or_else(|| {
                PersistenceError::NotFound(format!(
                    "Task {} not found after update",
                    entity.task_id
                ))
            })
    }

    async fn delete(&self, id: &Uuid) -> Result<bool, PersistenceError> {
        // Soft delete: mark as cancelled rather than hard-deleting
        match queries::update_task_status(&self.pool, *id, "cancelled").await {
            Ok(()) => Ok(true),
            Err(PersistenceError::NotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn task_repository_struct_is_constructible() {
        // Verify the struct definition compiles and fields are correct.
        // Actual DB tests are in postgres_tests.rs (T040).
        #[cfg(feature = "sqlx")]
        {
            // Type-level check: TaskRepository must be Send + Sync
            fn _assert_send_sync<T: Send + Sync>() {}
            _assert_send_sync::<super::TaskRepository>();
        }
    }
}
