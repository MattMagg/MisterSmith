//! Task repository for task record persistence.
//!
//! [`TaskRepository`] implements [`Repository<TaskRecord>`](super::Repository)
//! for CRUD operations on the task registry, plus specialized methods for
//! querying tasks by agent, time range, and correlation ID.

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use uuid::Uuid;

use mister_smith_core::PersistenceError;

use crate::memory::{
    MemoryFragmentMetadata, MemoryMetadataPage, MemoryMetadataPageRequest, MemorySnapshotMetadata,
};
#[cfg(feature = "sqlx")]
use crate::postgres::queries::{self, TaskRecord};

use super::Repository;

const MANAGED_MEMORY_KEY: &str = "managed_memory";
const FRAGMENT_INDEX_KEY: &str = "fragments";
const SNAPSHOT_INDEX_KEY: &str = "snapshots";

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

    /// Persist managed-memory indexes onto the task metadata document.
    #[cfg(feature = "sqlx")]
    pub async fn persist_managed_memory_metadata(
        &self,
        task_id: Uuid,
        fragments: &[MemoryFragmentMetadata],
        snapshots: &[MemorySnapshotMetadata],
    ) -> Result<TaskRecord, PersistenceError> {
        let mut record = queries::find_task(&self.pool, task_id)
            .await?
            .ok_or_else(|| PersistenceError::NotFound(format!("task {task_id} not found")))?;

        merge_fragment_metadata(&mut record.metadata, fragments)?;
        merge_snapshot_metadata(&mut record.metadata, snapshots)?;

        queries::update_task_metadata(&self.pool, task_id, record.metadata.clone()).await
    }

    /// Page persisted fragment metadata for a task.
    #[cfg(feature = "sqlx")]
    pub async fn page_fragment_metadata(
        &self,
        task_id: Uuid,
        request: MemoryMetadataPageRequest,
    ) -> Result<MemoryMetadataPage<MemoryFragmentMetadata>, PersistenceError> {
        let record = queries::find_task(&self.pool, task_id)
            .await?
            .ok_or_else(|| PersistenceError::NotFound(format!("task {task_id} not found")))?;

        page_fragment_metadata(&record.metadata, request)
    }

    /// Page persisted snapshot metadata for a task.
    #[cfg(feature = "sqlx")]
    pub async fn page_snapshot_metadata(
        &self,
        task_id: Uuid,
        request: MemoryMetadataPageRequest,
    ) -> Result<MemoryMetadataPage<MemorySnapshotMetadata>, PersistenceError> {
        let record = queries::find_task(&self.pool, task_id)
            .await?
            .ok_or_else(|| PersistenceError::NotFound(format!("task {task_id} not found")))?;

        page_snapshot_metadata(&record.metadata, request)
    }
}

/// Merge fragment metadata into a task metadata document, replacing duplicate IDs in place.
pub fn merge_fragment_metadata(
    metadata: &mut Value,
    fragments: &[MemoryFragmentMetadata],
) -> Result<(), PersistenceError> {
    if fragments.is_empty() {
        return Ok(());
    }

    let existing = load_index_entries::<MemoryFragmentMetadata>(metadata, FRAGMENT_INDEX_KEY)?;
    let merged = merge_entries_by_key(existing, fragments.to_vec(), |entry| entry.fragment_id);
    store_index_entries(metadata, FRAGMENT_INDEX_KEY, &merged)
}

/// Merge snapshot metadata into a task metadata document, replacing duplicate IDs in place.
pub fn merge_snapshot_metadata(
    metadata: &mut Value,
    snapshots: &[MemorySnapshotMetadata],
) -> Result<(), PersistenceError> {
    if snapshots.is_empty() {
        return Ok(());
    }

    let existing = load_index_entries::<MemorySnapshotMetadata>(metadata, SNAPSHOT_INDEX_KEY)?;
    let merged = merge_entries_by_key(existing, snapshots.to_vec(), |entry| entry.snapshot_id);
    store_index_entries(metadata, SNAPSHOT_INDEX_KEY, &merged)
}

/// Page persisted fragment metadata from a task metadata document.
pub fn page_fragment_metadata(
    metadata: &Value,
    request: MemoryMetadataPageRequest,
) -> Result<MemoryMetadataPage<MemoryFragmentMetadata>, PersistenceError> {
    let mut entries =
        load_index_entries_from_value::<MemoryFragmentMetadata>(metadata, FRAGMENT_INDEX_KEY)?;

    if let Some(scope) = request.scope.as_ref() {
        entries.retain(|entry| &entry.scope == scope);
    }
    if let Some(role) = request.role {
        entries.retain(|entry| entry.source_role == role);
    }

    Ok(page_entries(entries, request))
}

/// Page persisted snapshot metadata from a task metadata document.
pub fn page_snapshot_metadata(
    metadata: &Value,
    request: MemoryMetadataPageRequest,
) -> Result<MemoryMetadataPage<MemorySnapshotMetadata>, PersistenceError> {
    let mut entries =
        load_index_entries_from_value::<MemorySnapshotMetadata>(metadata, SNAPSHOT_INDEX_KEY)?;

    if let Some(scope) = request.scope.as_ref() {
        entries.retain(|entry| &entry.target_scope == scope);
    }
    if let Some(role) = request.role {
        entries.retain(|entry| entry.role == role);
    }

    Ok(page_entries(entries, request))
}

fn merge_entries_by_key<T, K, F>(mut existing: Vec<T>, incoming: Vec<T>, key_fn: F) -> Vec<T>
where
    T: Clone,
    K: PartialEq,
    F: Fn(&T) -> K,
{
    for entry in incoming {
        let key = key_fn(&entry);
        if let Some(slot) = existing.iter_mut().find(|current| key_fn(current) == key) {
            *slot = entry;
        } else {
            existing.push(entry);
        }
    }

    existing
}

fn page_entries<T: Clone>(
    entries: Vec<T>,
    request: MemoryMetadataPageRequest,
) -> MemoryMetadataPage<T> {
    let total_entries = entries.len();
    let offset = request.offset.min(total_entries);
    let limit = if request.limit == 0 {
        total_entries.saturating_sub(offset)
    } else {
        request.limit
    };
    let end = offset.saturating_add(limit).min(total_entries);

    MemoryMetadataPage {
        entries: entries[offset..end].to_vec(),
        offset,
        limit,
        total_entries,
        next_offset: (end < total_entries).then_some(end),
    }
}

fn load_index_entries<T>(metadata: &mut Value, index_key: &str) -> Result<Vec<T>, PersistenceError>
where
    T: DeserializeOwned,
{
    let root = ensure_root_object(metadata)?;
    let managed_memory = ensure_managed_memory_object(root)?;
    match managed_memory.get(index_key) {
        Some(value) => deserialize_index_entries(value),
        None => Ok(Vec::new()),
    }
}

fn load_index_entries_from_value<T>(
    metadata: &Value,
    index_key: &str,
) -> Result<Vec<T>, PersistenceError>
where
    T: DeserializeOwned,
{
    let Some(root) = metadata.as_object() else {
        if metadata.is_null() {
            return Ok(Vec::new());
        }
        return Err(PersistenceError::DataCorrupted(
            "task metadata must be a JSON object".to_string(),
        ));
    };

    let Some(managed_memory) = root.get(MANAGED_MEMORY_KEY) else {
        return Ok(Vec::new());
    };
    let Some(managed_memory) = managed_memory.as_object() else {
        return Err(PersistenceError::DataCorrupted(
            "task metadata managed_memory index must be a JSON object".to_string(),
        ));
    };

    match managed_memory.get(index_key) {
        Some(value) => deserialize_index_entries(value),
        None => Ok(Vec::new()),
    }
}

fn deserialize_index_entries<T>(value: &Value) -> Result<Vec<T>, PersistenceError>
where
    T: DeserializeOwned,
{
    match value {
        Value::Array(_) => serde_json::from_value(value.clone())
            .map_err(|error| PersistenceError::SerializationFailed(error.to_string())),
        Value::Null => Ok(Vec::new()),
        _ => Err(PersistenceError::DataCorrupted(
            "managed-memory index must be a JSON array".to_string(),
        )),
    }
}

fn store_index_entries<T>(
    metadata: &mut Value,
    index_key: &str,
    entries: &[T],
) -> Result<(), PersistenceError>
where
    T: Serialize,
{
    let root = ensure_root_object(metadata)?;
    let managed_memory = ensure_managed_memory_object(root)?;
    managed_memory.insert(
        index_key.to_string(),
        serde_json::to_value(entries)
            .map_err(|error| PersistenceError::SerializationFailed(error.to_string()))?,
    );
    Ok(())
}

fn ensure_root_object(
    metadata: &mut Value,
) -> Result<&mut serde_json::Map<String, Value>, PersistenceError> {
    if metadata.is_null() {
        *metadata = Value::Object(serde_json::Map::new());
    }

    metadata.as_object_mut().ok_or_else(|| {
        PersistenceError::DataCorrupted("task metadata must be a JSON object".to_string())
    })
}

fn ensure_managed_memory_object(
    root: &mut serde_json::Map<String, Value>,
) -> Result<&mut serde_json::Map<String, Value>, PersistenceError> {
    let entry = root
        .entry(MANAGED_MEMORY_KEY.to_string())
        .or_insert_with(|| Value::Object(serde_json::Map::new()));

    entry.as_object_mut().ok_or_else(|| {
        PersistenceError::DataCorrupted(
            "task metadata managed_memory index must be a JSON object".to_string(),
        )
    })
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
    use chrono::Utc;
    use serde_json::json;

    use mister_smith_core::{
        AgentId, AgentType, ContextBudgetId, ExecutionBranchId, MemoryFragmentId, MemorySnapshotId,
        TaskId,
    };

    use crate::memory::{
        FragmentClass, MemoryFragmentMetadata, MemoryMetadataPageRequest, MemorySnapshotMetadata,
        SnapshotScope,
    };

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

    #[test]
    fn merge_and_page_fragment_metadata_preserves_existing_fields() {
        let branch_id = ExecutionBranchId::new();
        let recorded_at = Utc::now();
        let mut metadata = json!({
            "status_reason": "queued",
        });
        let fragments = vec![
            MemoryFragmentMetadata {
                fragment_id: MemoryFragmentId::new(),
                scope: SnapshotScope::Branch(branch_id),
                source_agent_id: AgentId::new(),
                source_role: AgentType::Planner,
                source_key: "planner.context".to_string(),
                fragment_class: FragmentClass::Working,
                units: 3,
                version: 1,
                recorded_at,
            },
            MemoryFragmentMetadata {
                fragment_id: MemoryFragmentId::new(),
                scope: SnapshotScope::Branch(branch_id),
                source_agent_id: AgentId::new(),
                source_role: AgentType::Executor,
                source_key: "executor.trace".to_string(),
                fragment_class: FragmentClass::Episodic,
                units: 6,
                version: 2,
                recorded_at,
            },
        ];

        super::merge_fragment_metadata(&mut metadata, &fragments).expect("metadata should merge");

        assert_eq!(metadata["status_reason"], "queued");

        let page = super::page_fragment_metadata(
            &metadata,
            MemoryMetadataPageRequest {
                offset: 0,
                limit: 5,
                scope: Some(SnapshotScope::Branch(branch_id)),
                role: Some(AgentType::Planner),
            },
        )
        .expect("fragment metadata should page");

        assert_eq!(page.total_entries, 1);
        assert_eq!(page.entries[0], fragments[0]);
        assert_eq!(page.next_offset, None);
    }

    #[test]
    fn merge_snapshot_metadata_replaces_duplicate_ids_and_pages_results() {
        let workflow_id = TaskId::new();
        let snapshot_id = MemorySnapshotId::new();
        let target_scope = SnapshotScope::Workflow(workflow_id);
        let mut metadata = json!({});
        let first = MemorySnapshotMetadata {
            snapshot_id,
            target_scope: target_scope.clone(),
            role: AgentType::Planner,
            delivered_units: 4,
            total_candidate_units: 9,
            checkpoint_fragment_id: None,
            budget_id: ContextBudgetId::new(),
            fragment_count: 2,
            has_summary: false,
            created_at: Utc::now(),
        };
        let replacement = MemorySnapshotMetadata {
            snapshot_id,
            target_scope: target_scope.clone(),
            role: AgentType::Planner,
            delivered_units: 3,
            total_candidate_units: 9,
            checkpoint_fragment_id: Some(MemoryFragmentId::new()),
            budget_id: ContextBudgetId::new(),
            fragment_count: 1,
            has_summary: true,
            created_at: Utc::now(),
        };

        super::merge_snapshot_metadata(&mut metadata, std::slice::from_ref(&first))
            .expect("initial snapshot metadata should merge");
        super::merge_snapshot_metadata(&mut metadata, std::slice::from_ref(&replacement))
            .expect("duplicate snapshot metadata should replace");

        let page = super::page_snapshot_metadata(
            &metadata,
            MemoryMetadataPageRequest {
                offset: 0,
                limit: 10,
                scope: Some(target_scope),
                role: Some(AgentType::Planner),
            },
        )
        .expect("snapshot metadata should page");

        assert_eq!(page.total_entries, 1);
        assert_eq!(page.entries, vec![replacement]);
        assert_eq!(page.next_offset, None);
    }
}
