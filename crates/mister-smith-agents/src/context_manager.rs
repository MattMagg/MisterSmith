//! Role-aware context assembly over persisted managed memory.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use mister_smith_core::{
    AgentId, AgentType, ContextBudget, ExecutionBranchId, ExecutionNodeId, MemoryError,
    MemorySnapshotId, PersistenceError, TaskId,
};
use mister_smith_persistence::{
    repository::{agent::AgentRepository, task::TaskRepository},
    ManagedMemoryManager, MaterializedSnapshot, MemoryFragment, MemorySnapshot, ResumeSource,
    SnapshotScope,
};

use crate::execution_graph::BranchCheckpoint;

/// Materialized context delivered to a role from a bounded managed-memory snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleContext {
    /// Snapshot used for reconstruction.
    pub snapshot: MemorySnapshot,
    /// Fragments delivered for the role.
    pub fragments: Vec<MemoryFragment>,
    /// Resume source used during reconstruction.
    pub resume_source: ResumeSource,
    /// JSON payload convenient for prompt/context injection.
    pub payload: Value,
}

impl RoleContext {
    fn from_materialized(materialized: MaterializedSnapshot) -> Self {
        let payload = build_payload(&materialized);
        Self {
            snapshot: materialized.snapshot,
            fragments: materialized.fragments,
            resume_source: materialized.resume_source,
            payload,
        }
    }

    fn as_materialized_snapshot(&self) -> MaterializedSnapshot {
        MaterializedSnapshot {
            snapshot: self.snapshot.clone(),
            fragments: self.fragments.clone(),
            resume_source: self.resume_source,
        }
    }
}

/// Runtime request to assemble managed context for a role.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManagedContextRequest {
    /// Workflow whose task metadata should receive snapshot indexes.
    pub workflow_id: TaskId,
    /// Scope to assemble context for.
    pub scope: SnapshotScope,
    /// Budget that bounds delivered context.
    pub budget: ContextBudget,
}

/// Runtime request to checkpoint branch context and persist the resulting snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManagedCheckpointRequest {
    /// Workflow whose task metadata should receive snapshot indexes.
    pub workflow_id: TaskId,
    /// Branch to checkpoint.
    pub branch_id: ExecutionBranchId,
    /// Role the checkpoint is assembled for.
    pub role: AgentType,
    /// Budget that bounds delivered context.
    pub budget: ContextBudget,
    /// Nodes completed at checkpoint time.
    pub completed_nodes: Vec<ExecutionNodeId>,
    /// Nodes still pending at checkpoint time.
    pub pending_nodes: Vec<ExecutionNodeId>,
}

/// Managed-context input resolved at runtime or passed directly from a helper.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ManagedContextInput {
    /// A pre-materialized managed-context payload.
    Payload(Value),
    /// A runtime request that requires the actor's managed-context runtime.
    Request(ManagedContextRequest),
}

/// Persistence hook for managed-context snapshots and metadata.
#[async_trait::async_trait]
pub trait ManagedContextStore: Send + Sync {
    /// Persist a materialized snapshot and its lightweight metadata indexes.
    async fn persist_materialized_snapshot(
        &self,
        agent_id: AgentId,
        workflow_id: TaskId,
        snapshot: &MaterializedSnapshot,
    ) -> Result<(), PersistenceError>;

    /// Load a previously persisted materialized snapshot by agent and snapshot ID.
    async fn load_materialized_snapshot(
        &self,
        agent_id: AgentId,
        snapshot_id: MemorySnapshotId,
    ) -> Result<Option<MaterializedSnapshot>, PersistenceError>;
}

/// Repository-backed managed-context persistence over the Phase 6 persistence substrate.
pub struct RepositoryManagedContextStore {
    agent_repository: Arc<AgentRepository>,
    task_repository: Arc<TaskRepository>,
}

impl RepositoryManagedContextStore {
    /// Create a new repository-backed managed-context store.
    pub fn new(
        agent_repository: Arc<AgentRepository>,
        task_repository: Arc<TaskRepository>,
    ) -> Self {
        Self {
            agent_repository,
            task_repository,
        }
    }
}

#[async_trait::async_trait]
impl ManagedContextStore for RepositoryManagedContextStore {
    async fn persist_materialized_snapshot(
        &self,
        agent_id: AgentId,
        workflow_id: TaskId,
        snapshot: &MaterializedSnapshot,
    ) -> Result<(), PersistenceError> {
        self.agent_repository
            .persist_materialized_snapshot(*agent_id.as_ref(), snapshot)
            .await?;
        self.task_repository
            .persist_managed_memory_metadata(
                *workflow_id.as_ref(),
                &snapshot
                    .fragments
                    .iter()
                    .map(MemoryFragment::metadata)
                    .collect::<Vec<_>>(),
                &[snapshot.snapshot.metadata()],
            )
            .await?;
        Ok(())
    }

    async fn load_materialized_snapshot(
        &self,
        agent_id: AgentId,
        snapshot_id: MemorySnapshotId,
    ) -> Result<Option<MaterializedSnapshot>, PersistenceError> {
        self.agent_repository
            .get_materialized_snapshot(*agent_id.as_ref(), snapshot_id)
            .await
    }
}

/// Agent-facing facade over the persistence managed-memory manager.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextManager {
    memory: ManagedMemoryManager,
}

impl ContextManager {
    /// Create a context manager from an existing managed-memory manager.
    pub fn new(memory: ManagedMemoryManager) -> Self {
        Self { memory }
    }

    /// Record a managed-memory fragment for later role-aware assembly.
    pub fn record_fragment(&mut self, fragment: MemoryFragment) {
        self.memory.record_fragment(fragment);
    }

    /// Restore a persisted materialized snapshot into the in-memory managed-memory store.
    pub fn restore_materialized_snapshot(&mut self, materialized: &MaterializedSnapshot) {
        self.memory.restore_materialized_snapshot(materialized);
    }

    /// Assemble bounded role context for the provided scope.
    pub async fn assemble_role_context(
        &mut self,
        scope: SnapshotScope,
        role: AgentType,
        budget: ContextBudget,
    ) -> Result<RoleContext, MemoryError> {
        let snapshot = self.memory.assemble_snapshot(scope, role, budget).await?;
        self.materialize_snapshot(snapshot.snapshot_id)
    }

    /// Create a checkpoint-ready branch checkpoint backed by a managed-memory snapshot.
    pub async fn checkpoint_branch(
        &mut self,
        branch_id: ExecutionBranchId,
        role: AgentType,
        budget: ContextBudget,
        completed_nodes: Vec<ExecutionNodeId>,
        pending_nodes: Vec<ExecutionNodeId>,
    ) -> Result<BranchCheckpoint, MemoryError> {
        let snapshot_id = self.memory.checkpoint(branch_id, role, budget).await?;
        Ok(BranchCheckpoint::new(
            branch_id,
            completed_nodes,
            pending_nodes,
            snapshot_id,
        ))
    }

    /// Resume role context from a previously captured branch checkpoint.
    pub fn resume_from_checkpoint(
        &self,
        checkpoint: &BranchCheckpoint,
    ) -> Result<RoleContext, MemoryError> {
        self.materialize_snapshot(checkpoint.memory_snapshot_id)
    }

    /// Materialize a stored snapshot into a role context payload.
    pub fn materialize_snapshot(
        &self,
        snapshot_id: MemorySnapshotId,
    ) -> Result<RoleContext, MemoryError> {
        let materialized = self.memory.materialize_snapshot(snapshot_id)?;
        Ok(RoleContext::from_materialized(materialized))
    }

    /// Get a stored snapshot by identifier.
    pub fn snapshot(&self, snapshot_id: MemorySnapshotId) -> Option<MemorySnapshot> {
        self.memory.snapshot(snapshot_id)
    }
}

/// Actor-owned runtime that wires managed context into live message handling.
pub struct ManagedContextRuntime {
    context_manager: ContextManager,
    store: Option<Arc<dyn ManagedContextStore>>,
}

impl ManagedContextRuntime {
    /// Create a runtime with in-memory managed-memory state only.
    pub fn new(context_manager: ContextManager) -> Self {
        Self {
            context_manager,
            store: None,
        }
    }

    /// Attach a persistence-backed managed-context store.
    pub fn with_store(mut self, store: Arc<dyn ManagedContextStore>) -> Self {
        self.store = Some(store);
        self
    }

    /// Resolve and optionally persist a runtime managed-context request.
    pub async fn assemble_role_context(
        &mut self,
        agent_id: AgentId,
        role: AgentType,
        request: ManagedContextRequest,
    ) -> Result<RoleContext, MemoryError> {
        let role_context = self
            .context_manager
            .assemble_role_context(request.scope, role, request.budget)
            .await?;
        self.persist_role_context(agent_id, request.workflow_id, &role_context)
            .await?;
        Ok(role_context)
    }

    /// Checkpoint branch context and persist the materialized snapshot when configured.
    pub async fn checkpoint_branch(
        &mut self,
        agent_id: AgentId,
        request: ManagedCheckpointRequest,
    ) -> Result<BranchCheckpoint, MemoryError> {
        let checkpoint = self
            .context_manager
            .checkpoint_branch(
                request.branch_id,
                request.role,
                request.budget,
                request.completed_nodes,
                request.pending_nodes,
            )
            .await?;
        let role_context = self
            .context_manager
            .materialize_snapshot(checkpoint.memory_snapshot_id)?;
        self.persist_role_context(agent_id, request.workflow_id, &role_context)
            .await?;
        Ok(checkpoint)
    }

    /// Resume from checkpoint, falling back to persisted materialized snapshots when needed.
    pub async fn resume_from_checkpoint(
        &mut self,
        agent_id: AgentId,
        checkpoint: &BranchCheckpoint,
    ) -> Result<RoleContext, MemoryError> {
        match self.context_manager.resume_from_checkpoint(checkpoint) {
            Ok(role_context) => Ok(role_context),
            Err(MemoryError::SnapshotUnavailable { .. }) => {
                let Some(store) = self.store.as_ref() else {
                    return self.context_manager.resume_from_checkpoint(checkpoint);
                };
                let Some(materialized) = store
                    .load_materialized_snapshot(agent_id, checkpoint.memory_snapshot_id)
                    .await
                    .map_err(map_persistence_error)?
                else {
                    return Err(MemoryError::SnapshotUnavailable {
                        snapshot_id: Some(checkpoint.memory_snapshot_id),
                        message: "persisted snapshot not found for checkpoint resume".to_string(),
                    });
                };

                self.context_manager
                    .restore_materialized_snapshot(&materialized);
                Ok(RoleContext::from_materialized(materialized))
            }
            Err(error) => Err(error),
        }
    }

    async fn persist_role_context(
        &self,
        agent_id: AgentId,
        workflow_id: TaskId,
        role_context: &RoleContext,
    ) -> Result<(), MemoryError> {
        let Some(store) = self.store.as_ref() else {
            return Ok(());
        };
        store
            .persist_materialized_snapshot(
                agent_id,
                workflow_id,
                &role_context.as_materialized_snapshot(),
            )
            .await
            .map_err(map_persistence_error)
    }
}

/// Resolve a managed-context input into a payload for live message handling.
pub async fn resolve_managed_context_input(
    runtime: Option<&mut ManagedContextRuntime>,
    agent_id: AgentId,
    role: AgentType,
    managed_context: Option<ManagedContextInput>,
) -> Result<Option<Value>, MemoryError> {
    match managed_context {
        None => Ok(None),
        Some(ManagedContextInput::Payload(payload)) => Ok(Some(payload)),
        Some(ManagedContextInput::Request(request)) => {
            let runtime = runtime.ok_or_else(|| MemoryError::SnapshotUnavailable {
                snapshot_id: None,
                message: format!("managed-context runtime is not configured for role {role:?}"),
            })?;
            let role_context = runtime
                .assemble_role_context(agent_id, role, request)
                .await?;
            Ok(Some(role_context.payload))
        }
    }
}

/// Attach a managed-context payload to an arbitrary JSON value.
pub fn attach_managed_context(target: Value, managed_context: Value) -> Value {
    match target {
        Value::Object(mut object) => {
            object.insert("managed_context".to_string(), managed_context);
            Value::Object(object)
        }
        other => json!({
            "value": other,
            "managed_context": managed_context,
        }),
    }
}

fn build_payload(materialized: &MaterializedSnapshot) -> Value {
    json!({
        "snapshot_id": materialized.snapshot.snapshot_id,
        "target_scope": materialized.snapshot.target_scope,
        "role": materialized.snapshot.role,
        "resume_source": materialized.resume_source,
        "summary": materialized.snapshot.summary,
        "fragments": materialized.fragments.iter().map(|fragment| json!({
            "fragment_id": fragment.fragment_id,
            "scope": fragment.scope,
            "class": fragment.fragment_class,
            "units": fragment.units,
            "source_role": fragment.provenance.source_role,
            "source_key": fragment.provenance.source_key,
            "content": fragment.content,
        })).collect::<Vec<_>>(),
    })
}

fn map_persistence_error(error: PersistenceError) -> MemoryError {
    MemoryError::SnapshotUnavailable {
        snapshot_id: None,
        message: error.to_string(),
    }
}
