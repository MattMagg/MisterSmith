//! Role-aware context assembly over persisted managed memory.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use mister_smith_core::{
    AgentType, ContextBudget, ExecutionBranchId, ExecutionNodeId, MemoryError, MemorySnapshotId,
};
use mister_smith_persistence::{
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
        let mut checkpoint =
            BranchCheckpoint::new(branch_id, completed_nodes, pending_nodes, snapshot_id);
        checkpoint.created_at = Utc::now();
        Ok(checkpoint)
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
