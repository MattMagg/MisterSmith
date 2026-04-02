//! Branch-local checkpoint capture, resume, and reassignment helpers.

use std::collections::HashSet;
use std::future::Future;
use std::sync::Arc;

use chrono::Utc;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::warn;
use uuid::Uuid;

use mister_smith_core::{
    AgentId, BranchRecoveryStrategy, BranchState, CapabilityId, CheckpointId, DelegationScope,
    DurableWorkflowEventKind, ExecutionBranchId, ExecutionNodeId, MemorySnapshotId,
    PersistenceError, TaskId,
};
use mister_smith_events::CapabilitySummary;
use mister_smith_persistence::repository::task::{TaskRepository, WorkflowHistoryEventRecord};
use mister_smith_persistence::{BranchCheckpointRecord, BranchResumeRecord, HybridStateManager};

use crate::errors::AgentSystemError;
use crate::execution_graph::{BranchCheckpoint, ExecutionGraph};

/// Resume metadata persisted whenever a branch is resumed or reassigned.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchResumeMetadata {
    /// Workflow that owns the resumed branch.
    pub workflow_id: TaskId,
    /// Branch targeted for recovery.
    pub branch_id: ExecutionBranchId,
    /// Checkpoint selected for recovery.
    pub checkpoint_id: CheckpointId,
    /// Recovery strategy applied to the branch.
    pub recovery_strategy: BranchRecoveryStrategy,
    /// Checkpoint-safe node scope used for recovery.
    pub recovery_node_ids: Vec<ExecutionNodeId>,
    /// Nodes already completed before recovery started.
    pub completed_nodes: Vec<ExecutionNodeId>,
    /// Nodes still pending at the selected checkpoint.
    pub pending_nodes: Vec<ExecutionNodeId>,
    /// Agents previously assigned before recovery planning.
    pub previous_assigned_agents: Vec<AgentId>,
    /// Agent selected to resume the branch, when any.
    pub assigned_agent: Option<AgentId>,
    /// Capability that authorized the recovery action, when any.
    pub delegation_capability_id: Option<CapabilityId>,
    /// Delegation scope that authorized the recovery action, when any.
    pub delegation_scope: Option<DelegationScope>,
    /// Depth of the delegation provenance chain, when any.
    pub delegation_chain_depth: Option<usize>,
    /// Operator-visible rejection reason for denied delegated recovery, when any.
    pub delegation_rejection_reason: Option<String>,
    /// Operator-visible recovery notes.
    pub notes: Vec<String>,
    /// When the branch recovery plan was recorded.
    pub resumed_at: chrono::DateTime<chrono::Utc>,
}

/// Recovery plan reconstructed from the latest durable branch checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct BranchRecoveryPlan {
    /// Checkpoint selected for recovery.
    pub checkpoint: BranchCheckpoint,
    /// Checkpoint-safe node scope that still requires execution.
    pub recovery_node_ids: Vec<ExecutionNodeId>,
    /// Persistable resume metadata for the recovery action.
    pub resume_metadata: BranchResumeMetadata,
}

/// Stable replay payload for branch-scoped durable history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchReplayStatePayload {
    /// Stable step key used to find the branch after recompilation.
    pub branch_anchor_step_key: String,
    /// Branch state projected by this history event.
    pub branch_state: BranchState,
    /// Recovery strategy in effect when the event was accepted.
    pub recovery_strategy: BranchRecoveryStrategy,
    /// Agents assigned to the branch when the event was accepted.
    pub assigned_agent_ids: Vec<AgentId>,
    /// Durable checkpoint reference when the event carries checkpoint lineage.
    pub checkpoint_id: Option<CheckpointId>,
    /// Completed step keys covered by the replay-safe checkpoint scope.
    pub completed_step_keys: Vec<String>,
    /// Pending step keys covered by the replay-safe checkpoint scope.
    pub pending_step_keys: Vec<String>,
    /// Recovery step keys selected by the accepted resume plan.
    pub recovery_step_keys: Vec<String>,
    /// Managed-memory snapshot anchored to the checkpoint, when present.
    pub memory_snapshot_id: Option<MemorySnapshotId>,
    /// Failure or intervention context captured at the checkpoint boundary, when present.
    pub failure_context: Option<Value>,
    /// Timestamp recorded for the accepted checkpoint boundary, when present.
    pub captured_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Copy)]
struct RecoveryPlanRequest {
    strategy: BranchRecoveryStrategy,
    assigned_agent: Option<AgentId>,
    state_override: Option<BranchState>,
}

/// Durable storage surface for branch checkpoints and resume metadata.
#[async_trait::async_trait]
pub trait BranchCheckpointStore: Send + Sync {
    /// Persist a branch checkpoint for later recovery.
    async fn persist_branch_checkpoint(
        &self,
        workflow_id: TaskId,
        checkpoint: &BranchCheckpoint,
    ) -> Result<(), PersistenceError>;

    /// Persist resume or reassignment metadata.
    async fn persist_branch_resume(
        &self,
        resume: &BranchResumeMetadata,
    ) -> Result<(), PersistenceError>;

    /// Load the latest known checkpoint for a branch, when available.
    async fn latest_branch_checkpoint(
        &self,
        workflow_id: TaskId,
        branch_id: ExecutionBranchId,
    ) -> Result<Option<BranchCheckpoint>, PersistenceError>;

    /// Load historical resume records for a branch.
    async fn branch_resume_history(
        &self,
        workflow_id: TaskId,
        branch_id: ExecutionBranchId,
    ) -> Result<Vec<BranchResumeMetadata>, PersistenceError>;

    /// Persist one accepted durable-history event related to branch replay.
    async fn persist_workflow_history_event(
        &self,
        _workflow_id: TaskId,
        _event: WorkflowHistoryEventRecord,
    ) -> Result<(), PersistenceError> {
        Ok(())
    }
}

/// Repository-backed branch checkpoint persistence with SQL-authoritative metadata
/// and best-effort KV cache hydration.
pub struct RepositoryBranchCheckpointStore {
    hybrid: Arc<HybridStateManager>,
    task_repository: Arc<TaskRepository>,
}

impl RepositoryBranchCheckpointStore {
    /// Create a new repository-backed branch checkpoint store.
    pub fn new(hybrid: Arc<HybridStateManager>, task_repository: Arc<TaskRepository>) -> Self {
        Self {
            hybrid,
            task_repository,
        }
    }
}

#[async_trait::async_trait]
impl BranchCheckpointStore for RepositoryBranchCheckpointStore {
    async fn persist_branch_checkpoint(
        &self,
        workflow_id: TaskId,
        checkpoint: &BranchCheckpoint,
    ) -> Result<(), PersistenceError> {
        let checkpoint_value = serialize_value(checkpoint)?;
        persist_repository_then_cache(
            async {
                self.task_repository
                    .persist_branch_recovery_metadata(
                        *workflow_id.as_ref(),
                        &[checkpoint_record(checkpoint)],
                        &[],
                    )
                    .await
                    .map(|_| ())
            },
            async {
                self.hybrid
                    .write_branch_checkpoint(
                        *workflow_id.as_ref(),
                        *checkpoint.branch_id.as_ref(),
                        &checkpoint_value,
                    )
                    .await
                    .map(|_| ())
            },
            "branch checkpoint",
        )
        .await
    }

    async fn persist_branch_resume(
        &self,
        resume: &BranchResumeMetadata,
    ) -> Result<(), PersistenceError> {
        let mut history = self
            .branch_resume_history(resume.workflow_id, resume.branch_id)
            .await?;
        history.push(resume.clone());
        history.sort_by(|left, right| left.resumed_at.cmp(&right.resumed_at));

        let history_value = serialize_value(&history)?;
        persist_repository_then_cache(
            async {
                self.task_repository
                    .persist_branch_recovery_metadata(
                        *resume.workflow_id.as_ref(),
                        &[],
                        &[resume_record(resume)],
                    )
                    .await
                    .map(|_| ())
            },
            async {
                self.hybrid
                    .write_branch_resume_history(
                        *resume.workflow_id.as_ref(),
                        *resume.branch_id.as_ref(),
                        &history_value,
                    )
                    .await
                    .map(|_| ())
            },
            "branch resume history",
        )
        .await
    }

    async fn latest_branch_checkpoint(
        &self,
        workflow_id: TaskId,
        branch_id: ExecutionBranchId,
    ) -> Result<Option<BranchCheckpoint>, PersistenceError> {
        read_repository_then_cache(
            async {
                self.task_repository
                    .load_latest_branch_checkpoint(*workflow_id.as_ref(), branch_id)
                    .await
                    .map(|record| record.map(checkpoint_from_record))
            },
            async {
                self.hybrid
                    .read_branch_checkpoint(*workflow_id.as_ref(), *branch_id.as_ref())
                    .await?
                    .map_or(Ok(None), |value| deserialize_value(value).map(Some))
            },
            "branch checkpoint",
        )
        .await
    }

    async fn branch_resume_history(
        &self,
        workflow_id: TaskId,
        branch_id: ExecutionBranchId,
    ) -> Result<Vec<BranchResumeMetadata>, PersistenceError> {
        match self
            .task_repository
            .load_branch_resume_history(*workflow_id.as_ref(), branch_id)
            .await
        {
            Ok(records) if !records.is_empty() => {
                Ok(records.into_iter().map(resume_from_record).collect())
            }
            Ok(_) => self
                .hybrid
                .read_branch_resume_history(*workflow_id.as_ref(), *branch_id.as_ref())
                .await?
                .map_or(Ok(Vec::new()), deserialize_value),
            Err(repository_error) => {
                warn!(
                    cache = "branch resume history",
                    error = %repository_error,
                    "branch recovery repository read failed; falling back to cache"
                );
                match self
                    .hybrid
                    .read_branch_resume_history(*workflow_id.as_ref(), *branch_id.as_ref())
                    .await
                {
                    Ok(Some(value)) => deserialize_value(value),
                    Ok(None) => Ok(Vec::new()),
                    Err(cache_error) => Err(PersistenceError::ConnectionFailed(format!(
                        "repository read failed: {repository_error}; cache fallback failed: {cache_error}"
                    ))),
                }
            }
        }
    }

    async fn persist_workflow_history_event(
        &self,
        workflow_id: TaskId,
        mut event: WorkflowHistoryEventRecord,
    ) -> Result<(), PersistenceError> {
        let history = self
            .task_repository
            .load_workflow_history(*workflow_id.as_ref())
            .await?;
        event.replay_position = history
            .last()
            .map(|entry| entry.replay_position.saturating_add(1))
            .unwrap_or(1);

        self.task_repository
            .persist_durable_workflow_metadata(*workflow_id.as_ref(), &[event], &[], &[], &[])
            .await?;

        let refreshed = self
            .task_repository
            .load_workflow_history(*workflow_id.as_ref())
            .await?;
        let history_value = serialize_value(&refreshed)?;
        if let Err(error) = self
            .hybrid
            .write_workflow_history(*workflow_id.as_ref(), &history_value)
            .await
        {
            warn!(
                cache = "workflow history",
                error = %error,
                "workflow history cache write failed after durable repository persistence"
            );
        }

        Ok(())
    }
}

/// Coordinator for branch-local checkpoint recording and recovery planning.
#[derive(Debug, Default, Clone, Copy)]
pub struct BranchCheckpointCoordinator;

impl BranchCheckpointCoordinator {
    /// Record a branch checkpoint onto the in-memory graph and durable store.
    pub async fn record_checkpoint<S: BranchCheckpointStore + ?Sized>(
        &self,
        store: &S,
        workflow_id: TaskId,
        graph: &mut ExecutionGraph,
        checkpoint: BranchCheckpoint,
    ) -> Result<(), AgentSystemError> {
        let branch = graph.branch_mut(&checkpoint.branch_id).ok_or_else(|| {
            AgentSystemError::OrchestrationError(format!(
                "No branch {} found for checkpoint {}",
                checkpoint.branch_id, checkpoint.checkpoint_id
            ))
        })?;

        store
            .persist_branch_checkpoint(workflow_id, &checkpoint)
            .await
            .map_err(map_persistence_error)?;

        branch.state = BranchState::Checkpointed;
        graph.checkpoint_lineage.push(checkpoint.clone());
        store
            .persist_workflow_history_event(
                workflow_id,
                checkpoint_history_event(workflow_id, graph, &checkpoint),
            )
            .await
            .map_err(map_persistence_error)?;
        Ok(())
    }

    /// Plan a checkpoint-based branch resume without replaying completed siblings.
    pub async fn resume_branch<S: BranchCheckpointStore + ?Sized>(
        &self,
        store: &S,
        workflow_id: TaskId,
        graph: &mut ExecutionGraph,
        branch_id: ExecutionBranchId,
        assigned_agent: Option<AgentId>,
    ) -> Result<BranchRecoveryPlan, AgentSystemError> {
        self.recovery_plan(
            store,
            workflow_id,
            graph,
            branch_id,
            RecoveryPlanRequest {
                strategy: BranchRecoveryStrategy::Resume,
                assigned_agent,
                state_override: None,
            },
            None,
        )
        .await
    }

    /// Plan a checkpoint-based branch resume with delegation provenance attached.
    pub async fn resume_branch_with_delegation<S: BranchCheckpointStore + ?Sized>(
        &self,
        store: &S,
        workflow_id: TaskId,
        graph: &mut ExecutionGraph,
        branch_id: ExecutionBranchId,
        assigned_agent: Option<AgentId>,
        capability: &CapabilitySummary,
    ) -> Result<BranchRecoveryPlan, AgentSystemError> {
        self.recovery_plan(
            store,
            workflow_id,
            graph,
            branch_id,
            RecoveryPlanRequest {
                strategy: BranchRecoveryStrategy::Resume,
                assigned_agent,
                state_override: None,
            },
            Some(capability),
        )
        .await
    }

    /// Plan a reassignment from the latest durable checkpoint for a failed branch.
    pub async fn reassign_branch<S: BranchCheckpointStore + ?Sized>(
        &self,
        store: &S,
        workflow_id: TaskId,
        graph: &mut ExecutionGraph,
        branch_id: ExecutionBranchId,
        assigned_agent: AgentId,
    ) -> Result<BranchRecoveryPlan, AgentSystemError> {
        self.recovery_plan(
            store,
            workflow_id,
            graph,
            branch_id,
            RecoveryPlanRequest {
                strategy: BranchRecoveryStrategy::Reassign,
                assigned_agent: Some(assigned_agent),
                state_override: Some(BranchState::Reassigned),
            },
            None,
        )
        .await
    }

    /// Plan a delegated reassignment from the latest durable checkpoint.
    pub async fn reassign_branch_with_delegation<S: BranchCheckpointStore + ?Sized>(
        &self,
        store: &S,
        workflow_id: TaskId,
        graph: &mut ExecutionGraph,
        branch_id: ExecutionBranchId,
        assigned_agent: AgentId,
        capability: &CapabilitySummary,
    ) -> Result<BranchRecoveryPlan, AgentSystemError> {
        self.recovery_plan(
            store,
            workflow_id,
            graph,
            branch_id,
            RecoveryPlanRequest {
                strategy: BranchRecoveryStrategy::Reassign,
                assigned_agent: Some(assigned_agent),
                state_override: Some(BranchState::Reassigned),
            },
            Some(capability),
        )
        .await
    }

    async fn recovery_plan<S: BranchCheckpointStore + ?Sized>(
        &self,
        store: &S,
        workflow_id: TaskId,
        graph: &mut ExecutionGraph,
        branch_id: ExecutionBranchId,
        request: RecoveryPlanRequest,
        delegation: Option<&CapabilitySummary>,
    ) -> Result<BranchRecoveryPlan, AgentSystemError> {
        let mut checkpoint = latest_checkpoint(store, workflow_id, graph, branch_id).await?;
        if let Some(capability) = delegation {
            attach_checkpoint_delegation(&mut checkpoint, capability);
            store
                .persist_branch_checkpoint(workflow_id, &checkpoint)
                .await
                .map_err(map_persistence_error)?;
        }
        hydrate_checkpoint_lineage(graph, checkpoint.clone());
        let recovery_node_ids = recovery_node_ids_from_checkpoint(graph, branch_id, &checkpoint);
        let branch = graph.branch_mut(&branch_id).ok_or_else(|| {
            AgentSystemError::OrchestrationError(format!("No branch {branch_id} found for resume"))
        })?;
        let previous_assigned_agents = branch.assigned_agents.clone();

        if let Some(agent_id) = request.assigned_agent {
            branch.assigned_agents = vec![agent_id];
        }

        branch.state = request.state_override.unwrap_or(BranchState::Checkpointed);
        branch.recovery_strategy = request.strategy;

        let mut resume_metadata = BranchResumeMetadata {
            workflow_id,
            branch_id,
            checkpoint_id: checkpoint.checkpoint_id,
            recovery_strategy: request.strategy,
            recovery_node_ids: recovery_node_ids.clone(),
            completed_nodes: checkpoint.completed_nodes.clone(),
            pending_nodes: checkpoint.pending_nodes.clone(),
            previous_assigned_agents,
            assigned_agent: request.assigned_agent,
            delegation_capability_id: None,
            delegation_scope: None,
            delegation_chain_depth: None,
            delegation_rejection_reason: None,
            notes: vec![match request.strategy {
                BranchRecoveryStrategy::Resume => {
                    "resume planned from latest branch checkpoint".to_string()
                }
                BranchRecoveryStrategy::Reassign => {
                    "branch reassigned from latest branch checkpoint".to_string()
                }
                BranchRecoveryStrategy::Isolate => {
                    "branch isolated from latest branch checkpoint".to_string()
                }
                BranchRecoveryStrategy::Escalate => {
                    "branch escalated from latest branch checkpoint".to_string()
                }
            }],
            resumed_at: Utc::now(),
        };
        if let Some(capability) = delegation {
            attach_delegation_summary(&mut resume_metadata, capability);
        }

        store
            .persist_branch_resume(&resume_metadata)
            .await
            .map_err(map_persistence_error)?;
        store
            .persist_workflow_history_event(
                workflow_id,
                recovery_plan_history_event(workflow_id, graph, &checkpoint, &resume_metadata),
            )
            .await
            .map_err(map_persistence_error)?;

        Ok(BranchRecoveryPlan {
            checkpoint,
            recovery_node_ids,
            resume_metadata,
        })
    }
}

fn hydrate_checkpoint_lineage(graph: &mut ExecutionGraph, checkpoint: BranchCheckpoint) {
    let branch_id = checkpoint.branch_id;
    let checkpoint_id = checkpoint.checkpoint_id;

    graph.checkpoint_lineage.retain(|existing| {
        existing.branch_id != branch_id || existing.checkpoint_id != checkpoint_id
    });

    if graph
        .latest_checkpoint(&branch_id)
        .is_some_and(|existing| existing.checkpoint_id == checkpoint_id)
    {
        return;
    }

    graph.checkpoint_lineage.push(checkpoint);
}

fn step_keys_for_node_ids(graph: &ExecutionGraph, node_ids: &[ExecutionNodeId]) -> Vec<String> {
    node_ids
        .iter()
        .filter_map(|node_id| {
            graph
                .nodes
                .iter()
                .find(|node| node.node_id == *node_id)
                .map(|node| node.step_key.clone())
        })
        .collect()
}

fn node_ids_for_step_keys(graph: &ExecutionGraph, step_keys: &[String]) -> Vec<ExecutionNodeId> {
    step_keys
        .iter()
        .filter_map(|step_key| {
            graph
                .nodes
                .iter()
                .find(|node| node.step_key == *step_key)
                .map(|node| node.node_id)
        })
        .collect()
}

/// Build a stable checkpoint replay payload that survives graph recompilation.
pub fn checkpoint_replay_payload(graph: &ExecutionGraph, checkpoint: &BranchCheckpoint) -> Value {
    let Some(branch) = graph.branch(&checkpoint.branch_id) else {
        return Value::Null;
    };
    let payload = BranchReplayStatePayload {
        branch_anchor_step_key: branch
            .node_ids
            .first()
            .and_then(|node_id| {
                graph
                    .nodes
                    .iter()
                    .find(|node| node.node_id == *node_id)
                    .map(|node| node.step_key.clone())
            })
            .unwrap_or_else(|| checkpoint.checkpoint_id.to_string()),
        branch_state: BranchState::Checkpointed,
        recovery_strategy: branch.recovery_strategy,
        assigned_agent_ids: branch.assigned_agents.clone(),
        checkpoint_id: Some(checkpoint.checkpoint_id),
        completed_step_keys: step_keys_for_node_ids(graph, &checkpoint.completed_nodes),
        pending_step_keys: step_keys_for_node_ids(graph, &checkpoint.pending_nodes),
        recovery_step_keys: step_keys_for_node_ids(graph, &checkpoint.pending_nodes),
        memory_snapshot_id: Some(checkpoint.memory_snapshot_id),
        failure_context: checkpoint.failure_context.clone(),
        captured_at: Some(checkpoint.created_at),
    };
    serde_json::to_value(payload).unwrap_or(Value::Null)
}

/// Rebuild one branch checkpoint from a stable replay payload and a freshly compiled graph.
pub fn checkpoint_from_replay_payload(
    graph: &ExecutionGraph,
    payload: &Value,
) -> Option<BranchCheckpoint> {
    let payload = serde_json::from_value::<BranchReplayStatePayload>(payload.clone()).ok()?;
    let completed_nodes = node_ids_for_step_keys(graph, &payload.completed_step_keys);
    let pending_nodes = node_ids_for_step_keys(graph, &payload.pending_step_keys);
    let branch_id = pending_nodes
        .first()
        .copied()
        .or_else(|| completed_nodes.first().copied())
        .and_then(|node_id| graph.nodes.iter().find(|node| node.node_id == node_id))
        .map(|node| node.branch_id)?;

    Some(BranchCheckpoint {
        checkpoint_id: payload.checkpoint_id?,
        branch_id,
        completed_nodes,
        pending_nodes,
        memory_snapshot_id: payload.memory_snapshot_id?,
        failure_context: payload.failure_context.filter(|value| !value.is_null()),
        created_at: payload.captured_at?,
    })
}

fn checkpoint_history_event(
    workflow_id: TaskId,
    graph: &ExecutionGraph,
    checkpoint: &BranchCheckpoint,
) -> WorkflowHistoryEventRecord {
    WorkflowHistoryEventRecord {
        workflow_id,
        event_id: Uuid::new_v4(),
        replay_position: 0,
        event_kind: DurableWorkflowEventKind::BranchStateChanged,
        recorded_at: Utc::now(),
        actor_agent_id: None,
        source: Some("branch_checkpoint_coordinator".to_string()),
        branch_id: Some(checkpoint.branch_id),
        node_id: None,
        lifecycle_state: None,
        effect_boundary_id: None,
        compaction_id: None,
        parent_event_id: None,
        payload: checkpoint_replay_payload(graph, checkpoint),
    }
}

fn recovery_plan_history_event(
    workflow_id: TaskId,
    graph: &ExecutionGraph,
    checkpoint: &BranchCheckpoint,
    resume_metadata: &BranchResumeMetadata,
) -> WorkflowHistoryEventRecord {
    let branch = graph
        .branch(&resume_metadata.branch_id)
        .expect("branch exists while recording recovery history");
    let payload = BranchReplayStatePayload {
        branch_anchor_step_key: branch
            .node_ids
            .first()
            .and_then(|node_id| {
                graph
                    .nodes
                    .iter()
                    .find(|node| node.node_id == *node_id)
                    .map(|node| node.step_key.clone())
            })
            .unwrap_or_else(|| resume_metadata.branch_id.to_string()),
        branch_state: branch.state,
        recovery_strategy: resume_metadata.recovery_strategy,
        assigned_agent_ids: branch.assigned_agents.clone(),
        checkpoint_id: Some(checkpoint.checkpoint_id),
        completed_step_keys: step_keys_for_node_ids(graph, &resume_metadata.completed_nodes),
        pending_step_keys: step_keys_for_node_ids(graph, &resume_metadata.pending_nodes),
        recovery_step_keys: step_keys_for_node_ids(graph, &resume_metadata.recovery_node_ids),
        memory_snapshot_id: Some(checkpoint.memory_snapshot_id),
        failure_context: checkpoint.failure_context.clone(),
        captured_at: Some(checkpoint.created_at),
    };

    WorkflowHistoryEventRecord {
        workflow_id,
        event_id: Uuid::new_v4(),
        replay_position: 0,
        event_kind: DurableWorkflowEventKind::BranchStateChanged,
        recorded_at: resume_metadata.resumed_at,
        actor_agent_id: resume_metadata.assigned_agent,
        source: Some("branch_checkpoint_recovery".to_string()),
        branch_id: Some(resume_metadata.branch_id),
        node_id: None,
        lifecycle_state: None,
        effect_boundary_id: None,
        compaction_id: None,
        parent_event_id: None,
        payload: serde_json::to_value(payload).unwrap_or(Value::Null),
    }
}

fn recovery_node_ids_from_checkpoint(
    graph: &ExecutionGraph,
    branch_id: ExecutionBranchId,
    checkpoint: &BranchCheckpoint,
) -> Vec<ExecutionNodeId> {
    if !checkpoint.pending_nodes.is_empty() {
        return checkpoint.pending_nodes.clone();
    }

    let Some(branch) = graph.branch(&branch_id) else {
        return Vec::new();
    };

    if !checkpoint.completed_nodes.is_empty() {
        let completed: HashSet<_> = checkpoint.completed_nodes.iter().copied().collect();
        return branch
            .node_ids
            .iter()
            .copied()
            .filter(|node_id| !completed.contains(node_id))
            .collect();
    }

    branch.node_ids.clone()
}

async fn latest_checkpoint<S: BranchCheckpointStore + ?Sized>(
    store: &S,
    workflow_id: TaskId,
    graph: &ExecutionGraph,
    branch_id: ExecutionBranchId,
) -> Result<BranchCheckpoint, AgentSystemError> {
    if let Some(checkpoint) = store
        .latest_branch_checkpoint(workflow_id, branch_id)
        .await
        .map_err(map_persistence_error)?
    {
        return Ok(checkpoint);
    }

    graph.latest_checkpoint(&branch_id).cloned().ok_or_else(|| {
        AgentSystemError::OrchestrationError(format!(
            "No checkpoint available for branch {branch_id}"
        ))
    })
}

fn map_persistence_error(error: PersistenceError) -> AgentSystemError {
    AgentSystemError::OrchestrationError(format!("branch checkpoint persistence failed: {error}"))
}

fn checkpoint_record(checkpoint: &BranchCheckpoint) -> BranchCheckpointRecord {
    BranchCheckpointRecord {
        checkpoint_id: checkpoint.checkpoint_id,
        branch_id: checkpoint.branch_id,
        completed_nodes: checkpoint.completed_nodes.clone(),
        pending_nodes: checkpoint.pending_nodes.clone(),
        memory_snapshot_id: checkpoint.memory_snapshot_id,
        failure_context: checkpoint.failure_context.clone(),
        created_at: checkpoint.created_at,
    }
}

fn checkpoint_from_record(record: BranchCheckpointRecord) -> BranchCheckpoint {
    BranchCheckpoint {
        checkpoint_id: record.checkpoint_id,
        branch_id: record.branch_id,
        completed_nodes: record.completed_nodes,
        pending_nodes: record.pending_nodes,
        memory_snapshot_id: record.memory_snapshot_id,
        failure_context: record.failure_context,
        created_at: record.created_at,
    }
}

fn resume_record(resume: &BranchResumeMetadata) -> BranchResumeRecord {
    BranchResumeRecord {
        workflow_id: resume.workflow_id,
        branch_id: resume.branch_id,
        checkpoint_id: resume.checkpoint_id,
        recovery_strategy: resume.recovery_strategy,
        recovery_node_ids: resume.recovery_node_ids.clone(),
        completed_nodes: resume.completed_nodes.clone(),
        pending_nodes: resume.pending_nodes.clone(),
        previous_assigned_agents: resume.previous_assigned_agents.clone(),
        assigned_agent: resume.assigned_agent,
        delegation_capability_id: resume.delegation_capability_id,
        delegation_scope: resume.delegation_scope,
        delegation_chain_depth: resume.delegation_chain_depth,
        delegation_rejection_reason: resume.delegation_rejection_reason.clone(),
        notes: resume.notes.clone(),
        resumed_at: resume.resumed_at,
    }
}

fn resume_from_record(record: BranchResumeRecord) -> BranchResumeMetadata {
    BranchResumeMetadata {
        workflow_id: record.workflow_id,
        branch_id: record.branch_id,
        checkpoint_id: record.checkpoint_id,
        recovery_strategy: record.recovery_strategy,
        recovery_node_ids: record.recovery_node_ids,
        completed_nodes: record.completed_nodes,
        pending_nodes: record.pending_nodes,
        previous_assigned_agents: record.previous_assigned_agents,
        assigned_agent: record.assigned_agent,
        delegation_capability_id: record.delegation_capability_id,
        delegation_scope: record.delegation_scope,
        delegation_chain_depth: record.delegation_chain_depth,
        delegation_rejection_reason: record.delegation_rejection_reason,
        notes: record.notes,
        resumed_at: record.resumed_at,
    }
}

/// Attach delegation provenance to a durable resume record.
pub fn attach_delegation_summary(
    resume_metadata: &mut BranchResumeMetadata,
    capability: &CapabilitySummary,
) {
    resume_metadata.delegation_capability_id = Some(capability.capability_id);
    resume_metadata.delegation_scope = Some(capability.scope);
    resume_metadata.delegation_chain_depth = Some(capability.chain_depth());
    resume_metadata.delegation_rejection_reason = capability.rejection_reason.clone();
    resume_metadata.notes.push(format!(
        "delegation capability {} {:?} depth={}",
        capability.capability_id,
        capability.scope,
        capability.chain_depth()
    ));
}

/// Attach delegation provenance to a checkpoint failure context for operator replay.
pub fn attach_checkpoint_delegation(
    checkpoint: &mut BranchCheckpoint,
    capability: &CapabilitySummary,
) {
    let delegation_context = json!({
        "delegation_capability_id": capability.capability_id,
        "delegation_scope": format!("{:?}", capability.scope),
        "delegation_chain_depth": capability.chain_depth(),
        "delegation_rejection_reason": capability.rejection_reason,
        "provenance": capability.provenance,
    });

    checkpoint.failure_context = Some(match checkpoint.failure_context.take() {
        Some(Value::Object(mut existing)) => {
            let Value::Object(delegation_fields) = delegation_context else {
                unreachable!("delegation context is always an object");
            };
            existing.extend(delegation_fields);
            Value::Object(existing)
        }
        Some(existing) => {
            let Value::Object(mut delegation_fields) = delegation_context else {
                unreachable!("delegation context is always an object");
            };
            delegation_fields.insert("existing_failure_context".to_string(), existing);
            Value::Object(delegation_fields)
        }
        None => delegation_context,
    });
}

fn serialize_value<T: Serialize>(value: &T) -> Result<serde_json::Value, PersistenceError> {
    serde_json::to_value(value)
        .map_err(|error| PersistenceError::SerializationFailed(error.to_string()))
}

fn deserialize_value<T: DeserializeOwned>(value: serde_json::Value) -> Result<T, PersistenceError> {
    serde_json::from_value(value)
        .map_err(|error| PersistenceError::SerializationFailed(error.to_string()))
}

async fn persist_repository_then_cache<RepositoryWrite, CacheWrite>(
    repository_write: RepositoryWrite,
    cache_write: CacheWrite,
    cache_label: &str,
) -> Result<(), PersistenceError>
where
    RepositoryWrite: Future<Output = Result<(), PersistenceError>>,
    CacheWrite: Future<Output = Result<(), PersistenceError>>,
{
    repository_write.await?;
    if let Err(error) = cache_write.await {
        warn!(
            cache = cache_label,
            error = %error,
            "branch recovery cache write failed after durable repository persistence"
        );
    }
    Ok(())
}

async fn read_repository_then_cache<T, RepositoryRead, CacheRead>(
    repository_read: RepositoryRead,
    cache_read: CacheRead,
    cache_label: &str,
) -> Result<Option<T>, PersistenceError>
where
    RepositoryRead: Future<Output = Result<Option<T>, PersistenceError>>,
    CacheRead: Future<Output = Result<Option<T>, PersistenceError>>,
{
    match repository_read.await {
        Ok(Some(value)) => Ok(Some(value)),
        Ok(None) => cache_read.await,
        Err(repository_error) => {
            warn!(
                cache = cache_label,
                error = %repository_error,
                "branch recovery repository read failed; falling back to cache"
            );
            match cache_read.await {
                Ok(value) => Ok(value),
                Err(cache_error) => Err(PersistenceError::ConnectionFailed(format!(
                    "repository read failed: {repository_error}; cache fallback failed: {cache_error}"
                ))),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{persist_repository_then_cache, read_repository_then_cache};
    use mister_smith_core::PersistenceError;

    #[tokio::test]
    async fn repository_persist_remains_authoritative_when_cache_write_fails() {
        let result = persist_repository_then_cache(
            async { Ok(()) },
            async {
                Err(PersistenceError::ConnectionFailed(
                    "kv unavailable".to_string(),
                ))
            },
            "branch checkpoint",
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn repository_read_falls_back_to_cache_when_repository_is_degraded() {
        let result = read_repository_then_cache(
            async {
                Err(PersistenceError::ConnectionFailed(
                    "repository unavailable".to_string(),
                ))
            },
            async { Ok(Some("cached-checkpoint")) },
            "branch checkpoint",
        )
        .await
        .unwrap();

        assert_eq!(result, Some("cached-checkpoint"));
    }

    #[tokio::test]
    async fn repository_read_prefers_durable_value_when_available() {
        let result = read_repository_then_cache(
            async { Ok(Some("durable-checkpoint")) },
            async { Ok(Some("cached-checkpoint")) },
            "branch checkpoint",
        )
        .await
        .unwrap();

        assert_eq!(result, Some("durable-checkpoint"));
    }
}
