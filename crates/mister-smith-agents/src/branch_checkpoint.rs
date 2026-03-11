//! Branch-local checkpoint capture, resume, and reassignment helpers.

use std::sync::Arc;

use chrono::Utc;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use mister_smith_core::{
    AgentId, BranchRecoveryStrategy, BranchState, CheckpointId, ExecutionBranchId, ExecutionNodeId,
    PersistenceError, TaskId,
};
use mister_smith_persistence::repository::task::TaskRepository;
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
}

/// Repository-backed branch checkpoint persistence over the Phase 6 dual-store substrate.
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
        self.hybrid
            .write_branch_checkpoint(
                *workflow_id.as_ref(),
                *checkpoint.branch_id.as_ref(),
                &checkpoint_value,
            )
            .await?;
        self.task_repository
            .persist_branch_recovery_metadata(
                *workflow_id.as_ref(),
                &[checkpoint_record(checkpoint)],
                &[],
            )
            .await?;
        Ok(())
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
        self.hybrid
            .write_branch_resume_history(
                *resume.workflow_id.as_ref(),
                *resume.branch_id.as_ref(),
                &history_value,
            )
            .await?;
        self.task_repository
            .persist_branch_recovery_metadata(
                *resume.workflow_id.as_ref(),
                &[],
                &[resume_record(resume)],
            )
            .await?;
        Ok(())
    }

    async fn latest_branch_checkpoint(
        &self,
        workflow_id: TaskId,
        branch_id: ExecutionBranchId,
    ) -> Result<Option<BranchCheckpoint>, PersistenceError> {
        if let Some(value) = self
            .hybrid
            .read_branch_checkpoint(*workflow_id.as_ref(), *branch_id.as_ref())
            .await?
        {
            return deserialize_value(value).map(Some);
        }

        self.task_repository
            .load_latest_branch_checkpoint(*workflow_id.as_ref(), branch_id)
            .await
            .map(|record| record.map(checkpoint_from_record))
    }

    async fn branch_resume_history(
        &self,
        workflow_id: TaskId,
        branch_id: ExecutionBranchId,
    ) -> Result<Vec<BranchResumeMetadata>, PersistenceError> {
        if let Some(value) = self
            .hybrid
            .read_branch_resume_history(*workflow_id.as_ref(), *branch_id.as_ref())
            .await?
        {
            return deserialize_value(value);
        }

        self.task_repository
            .load_branch_resume_history(*workflow_id.as_ref(), branch_id)
            .await
            .map(|records| records.into_iter().map(resume_from_record).collect())
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
        graph.checkpoint_lineage.push(checkpoint);
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
            BranchRecoveryStrategy::Resume,
            assigned_agent,
            None,
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
            BranchRecoveryStrategy::Reassign,
            Some(assigned_agent),
            Some(BranchState::Reassigned),
        )
        .await
    }

    async fn recovery_plan<S: BranchCheckpointStore + ?Sized>(
        &self,
        store: &S,
        workflow_id: TaskId,
        graph: &mut ExecutionGraph,
        branch_id: ExecutionBranchId,
        strategy: BranchRecoveryStrategy,
        assigned_agent: Option<AgentId>,
        state_override: Option<BranchState>,
    ) -> Result<BranchRecoveryPlan, AgentSystemError> {
        let checkpoint = latest_checkpoint(store, workflow_id, graph, branch_id).await?;
        let recovery_node_ids = graph.recovery_node_ids(&branch_id);
        let branch = graph.branch_mut(&branch_id).ok_or_else(|| {
            AgentSystemError::OrchestrationError(format!("No branch {branch_id} found for resume"))
        })?;
        let previous_assigned_agents = branch.assigned_agents.clone();

        if let Some(agent_id) = assigned_agent {
            branch.assigned_agents = vec![agent_id];
        }

        branch.state = state_override.unwrap_or(BranchState::Checkpointed);

        let resume_metadata = BranchResumeMetadata {
            workflow_id,
            branch_id,
            checkpoint_id: checkpoint.checkpoint_id,
            recovery_strategy: strategy,
            recovery_node_ids: recovery_node_ids.clone(),
            completed_nodes: checkpoint.completed_nodes.clone(),
            pending_nodes: checkpoint.pending_nodes.clone(),
            previous_assigned_agents,
            assigned_agent,
            notes: vec![match strategy {
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

        store
            .persist_branch_resume(&resume_metadata)
            .await
            .map_err(map_persistence_error)?;

        Ok(BranchRecoveryPlan {
            checkpoint,
            recovery_node_ids,
            resume_metadata,
        })
    }
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
        notes: record.notes,
        resumed_at: record.resumed_at,
    }
}

fn serialize_value<T: Serialize>(value: &T) -> Result<serde_json::Value, PersistenceError> {
    serde_json::to_value(value)
        .map_err(|error| PersistenceError::SerializationFailed(error.to_string()))
}

fn deserialize_value<T: DeserializeOwned>(value: serde_json::Value) -> Result<T, PersistenceError> {
    serde_json::from_value(value)
        .map_err(|error| PersistenceError::SerializationFailed(error.to_string()))
}
