//! Conversation session repository for durable multi-turn state.

use mister_smith_core::{PersistenceError, SessionId, TaskId};

#[cfg(feature = "sqlx")]
use crate::postgres::queries::{self, SessionRecord, SessionTurnRecord};

/// Repository for conversation session rows and ordered turn rows.
pub struct SessionRepository {
    #[cfg(feature = "sqlx")]
    pool: sqlx::PgPool,
}

impl SessionRepository {
    /// Create from a PG pool.
    #[cfg(feature = "sqlx")]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Persist a new conversation session row.
    #[cfg(feature = "sqlx")]
    pub async fn create_session(
        &self,
        record: &SessionRecord,
    ) -> Result<SessionRecord, PersistenceError> {
        queries::insert_session(&self.pool, record).await
    }

    /// Find a conversation session by identifier.
    #[cfg(feature = "sqlx")]
    pub async fn find_session(
        &self,
        session_id: SessionId,
    ) -> Result<Option<SessionRecord>, PersistenceError> {
        queries::find_session(&self.pool, *session_id.as_ref()).await
    }

    /// Update a conversation session row.
    #[cfg(feature = "sqlx")]
    pub async fn update_session(
        &self,
        record: &SessionRecord,
    ) -> Result<SessionRecord, PersistenceError> {
        queries::update_session(&self.pool, record).await
    }

    /// Delete a session row. Used for compensation when startup fails before launch.
    #[cfg(feature = "sqlx")]
    pub async fn delete_session(&self, session_id: SessionId) -> Result<bool, PersistenceError> {
        queries::delete_session(&self.pool, *session_id.as_ref()).await
    }

    /// Persist a new ordered session turn.
    #[cfg(feature = "sqlx")]
    pub async fn create_turn(
        &self,
        record: &SessionTurnRecord,
    ) -> Result<SessionTurnRecord, PersistenceError> {
        queries::insert_session_turn(&self.pool, record).await
    }

    /// Find the turn row linked to a workflow root.
    #[cfg(feature = "sqlx")]
    pub async fn find_turn_by_workflow(
        &self,
        workflow_id: TaskId,
    ) -> Result<Option<SessionTurnRecord>, PersistenceError> {
        queries::find_session_turn_by_workflow(&self.pool, *workflow_id.as_ref()).await
    }

    /// Update a session turn row.
    #[cfg(feature = "sqlx")]
    pub async fn update_turn(
        &self,
        record: &SessionTurnRecord,
    ) -> Result<SessionTurnRecord, PersistenceError> {
        queries::update_session_turn(&self.pool, record).await
    }

    /// Delete a turn row by workflow identifier.
    #[cfg(feature = "sqlx")]
    pub async fn delete_turn_by_workflow(
        &self,
        workflow_id: TaskId,
    ) -> Result<bool, PersistenceError> {
        queries::delete_session_turn_by_workflow(&self.pool, *workflow_id.as_ref()).await
    }

    /// List ordered turns for a session.
    #[cfg(feature = "sqlx")]
    pub async fn list_turns(
        &self,
        session_id: SessionId,
    ) -> Result<Vec<SessionTurnRecord>, PersistenceError> {
        queries::list_session_turns(&self.pool, *session_id.as_ref()).await
    }
}
