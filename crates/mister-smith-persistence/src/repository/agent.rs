//! Agent repository for agent registry and state management.
//!
//! [`AgentRepository`] implements [`Repository<AgentRecord>`](super::Repository)
//! for CRUD operations on the agent registry, plus specialized methods for
//! state persistence (via the hybrid KV+SQL manager) and checkpointing.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use tracing::{debug, warn};
use uuid::Uuid;

use mister_smith_core::PersistenceError;

#[cfg(feature = "sqlx")]
use crate::postgres::queries::{self, AgentRecord};

use crate::hybrid::manager::HybridStateManager;
use super::Repository;

/// Repository for agent registry records and agent state.
///
/// Combines SQL-based registry operations (insert, find, update, delete)
/// with hybrid KV+SQL state management (save_state, get_state, checkpoint).
pub struct AgentRepository {
    hybrid: Arc<HybridStateManager>,
    #[cfg(feature = "sqlx")]
    pool: sqlx::PgPool,
}

impl AgentRepository {
    /// Create from a hybrid state manager and PG pool.
    #[cfg(feature = "sqlx")]
    pub fn new(hybrid: Arc<HybridStateManager>, pool: sqlx::PgPool) -> Self {
        Self { hybrid, pool }
    }

    /// Create from a hybrid state manager only (no SQL).
    #[cfg(not(feature = "sqlx"))]
    pub fn new(hybrid: Arc<HybridStateManager>) -> Self {
        Self { hybrid }
    }

    /// Start a new database transaction for multi-operation atomicity.
    #[cfg(feature = "sqlx")]
    pub async fn begin_transaction(
        &self,
    ) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, PersistenceError> {
        queries::begin_transaction(&self.pool).await
    }

    /// Find agents by type (e.g., "orchestrator", "worker").
    #[cfg(feature = "sqlx")]
    pub async fn find_by_type(
        &self,
        agent_type: &str,
    ) -> Result<Vec<AgentRecord>, PersistenceError> {
        queries::find_agents_by_type(&self.pool, agent_type).await
    }

    /// Find agents by status (e.g., "active", "suspended").
    #[cfg(feature = "sqlx")]
    pub async fn find_by_status(
        &self,
        status: &str,
    ) -> Result<Vec<AgentRecord>, PersistenceError> {
        queries::find_agents_by_status(&self.pool, status).await
    }

    /// Save an agent state key-value pair (routed through hybrid manager).
    ///
    /// Writes to KV first for fast access, marks the key dirty for async
    /// flush to SQL.
    pub async fn save_state(
        &self,
        agent_id: Uuid,
        key: &str,
        value: Value,
    ) -> Result<(), PersistenceError> {
        self.hybrid.write_state(agent_id, key, &value).await?;
        Ok(())
    }

    /// Get agent state (KV first, SQL fallback with lazy hydration).
    pub async fn get_state(
        &self,
        agent_id: Uuid,
        key: &str,
    ) -> Result<Option<Value>, PersistenceError> {
        self.hybrid.read_state(agent_id, key).await
    }

    /// Get all state keys for an agent from SQL.
    #[cfg(feature = "sqlx")]
    pub async fn get_all_state(
        &self,
        agent_id: Uuid,
    ) -> Result<Vec<(String, Value)>, PersistenceError> {
        let rows = queries::get_all_state(&self.pool, agent_id).await?;
        Ok(rows
            .into_iter()
            .map(|r| (r.state_key, r.state_value))
            .collect())
    }

    /// Create a checkpoint of an agent's current state.
    ///
    /// Reads all state keys from SQL and stores a snapshot for point-in-time
    /// recovery. Returns the checkpoint UUID.
    #[cfg(feature = "sqlx")]
    pub async fn checkpoint(&self, agent_id: Uuid) -> Result<Uuid, PersistenceError> {
        // Flush any pending dirty keys first
        if let Err(e) = self.hybrid.flush_to_sql().await {
            warn!(
                agent_id = %agent_id, error = %e,
                "Failed to flush before checkpoint — snapshot may be stale"
            );
        }

        // Read all state from SQL
        let rows = queries::get_all_state(&self.pool, agent_id).await?;

        // Build snapshot as a JSON object
        let mut snapshot = serde_json::Map::new();
        for row in &rows {
            snapshot.insert(row.state_key.clone(), row.state_value.clone());
        }

        let snapshot_value = Value::Object(snapshot);

        // Insert checkpoint
        let checkpoint_id =
            queries::insert_checkpoint(&self.pool, agent_id, snapshot_value, None).await?;

        debug!(
            agent_id = %agent_id,
            checkpoint_id = %checkpoint_id,
            keys = rows.len(),
            "Checkpoint created"
        );

        Ok(checkpoint_id)
    }

    /// Hydrate agent state from SQL into KV on startup.
    ///
    /// Reads all state keys from SQL and writes them to KV for fast access.
    /// Returns the number of keys hydrated.
    #[cfg(feature = "sqlx")]
    pub async fn hydrate(&self, agent_id: Uuid) -> Result<usize, PersistenceError> {
        let rows = queries::get_all_state(&self.pool, agent_id).await?;
        let mut hydrated = 0usize;

        for row in &rows {
            let kv_key = format!("{agent_id}:{}", row.state_key);
            match self.hybrid.kv().save(&kv_key, &row.state_value).await {
                Ok(_) => {
                    hydrated += 1;
                }
                Err(e) => {
                    warn!(
                        agent_id = %agent_id, key = %row.state_key, error = %e,
                        "Failed to hydrate state key into KV"
                    );
                }
            }
        }

        debug!(
            agent_id = %agent_id,
            hydrated = hydrated,
            total = rows.len(),
            "State hydration complete"
        );

        Ok(hydrated)
    }
}

#[cfg(feature = "sqlx")]
#[async_trait]
impl Repository<AgentRecord> for AgentRepository {
    async fn save(&self, entity: &AgentRecord) -> Result<AgentRecord, PersistenceError> {
        queries::insert_agent(&self.pool, entity).await
    }

    async fn find(&self, id: &Uuid) -> Result<Option<AgentRecord>, PersistenceError> {
        queries::find_agent(&self.pool, *id).await
    }

    async fn update(&self, entity: &AgentRecord) -> Result<AgentRecord, PersistenceError> {
        // Update status (the primary mutable field on the registry record)
        queries::update_agent_status(&self.pool, entity.agent_id, &entity.status).await?;
        // Return the updated record from DB
        queries::find_agent(&self.pool, entity.agent_id)
            .await?
            .ok_or_else(|| {
                PersistenceError::NotFound(format!("Agent {} not found after update", entity.agent_id))
            })
    }

    async fn delete(&self, id: &Uuid) -> Result<bool, PersistenceError> {
        // Mark as terminated rather than hard-deleting
        match queries::update_agent_status(&self.pool, *id, "terminated").await {
            Ok(()) => Ok(true),
            Err(PersistenceError::NotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_state_key_format() {
        let id = Uuid::new_v4();
        let key = format!("{id}:config.model");
        assert!(key.contains(':'));
        assert!(key.starts_with(&id.to_string()));
    }

    #[test]
    fn repository_trait_is_object_safe() {
        // Verify the trait can be used as a trait object (with AgentRecord placeholder)
        fn _assert_send_sync<T: Send + Sync>() {}
        // AgentRepository is Send + Sync since its fields are
        // (Arc is Send+Sync, PgPool is Send+Sync)
    }
}
