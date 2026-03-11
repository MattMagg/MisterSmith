use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use async_trait::async_trait;
use mister_smith_agents::branch_checkpoint::{
    BranchCheckpointCoordinator, BranchCheckpointStore, BranchRecoveryPlan, BranchResumeMetadata,
};
use mister_smith_agents::orchestrator::Orchestrator;
use mister_smith_agents::scheduler::{ArrayAggregator, IdentityDecomposer, TaskScheduler};
use mister_smith_agents::{BranchCheckpoint, ExecutionGraph, TopologyCompiler, TopologySignals};
use mister_smith_core::{
    AgentId, BranchRecoveryStrategy, BranchState, ExecutionBranchId, ExecutionNodeId,
    PersistenceError, TaskId,
};
use mister_smith_events::AutonomyEvent;
use serde_json::json;

fn branch_recovery_plan() -> serde_json::Value {
    json!({
        "goal": "branch-recovery",
        "steps": [
            {
                "id": "root",
                "step": 1,
                "action": "root",
                "description": "root"
            },
            {
                "id": "branch-a-1",
                "step": 2,
                "action": "branch-a-1",
                "description": "branch-a-1",
                "depends_on": ["root"],
                "branch": "branch-a"
            },
            {
                "id": "branch-a-2",
                "step": 3,
                "action": "branch-a-2",
                "description": "branch-a-2",
                "depends_on": ["branch-a-1"],
                "branch": "branch-a"
            },
            {
                "id": "branch-b-1",
                "step": 4,
                "action": "branch-b-1",
                "description": "branch-b-1",
                "depends_on": ["root"],
                "branch": "branch-b"
            }
        ]
    })
}

fn compile_graph() -> ExecutionGraph {
    TopologyCompiler::default()
        .compile(
            TaskId::new(),
            &branch_recovery_plan(),
            &TopologySignals::default(),
        )
        .expect("branch recovery graph should compile")
}

fn node_id_for(graph: &ExecutionGraph, step_key: &str) -> ExecutionNodeId {
    graph
        .nodes
        .iter()
        .find(|node| node.step_key == step_key)
        .map(|node| node.node_id)
        .expect("expected node to exist")
}

fn branch_id_for(graph: &ExecutionGraph, step_key: &str) -> ExecutionBranchId {
    graph
        .nodes
        .iter()
        .find(|node| node.step_key == step_key)
        .map(|node| node.branch_id)
        .expect("expected branch to exist")
}

#[derive(Default)]
struct InMemoryBranchCheckpointStore {
    checkpoints: Mutex<HashMap<(TaskId, ExecutionBranchId), Vec<BranchCheckpoint>>>,
    resumes: Mutex<HashMap<(TaskId, ExecutionBranchId), Vec<BranchResumeMetadata>>>,
}

impl InMemoryBranchCheckpointStore {
    fn checkpoint_count(&self, workflow_id: TaskId, branch_id: ExecutionBranchId) -> usize {
        self.checkpoints
            .lock()
            .unwrap()
            .get(&(workflow_id, branch_id))
            .map(Vec::len)
            .unwrap_or_default()
    }

    fn latest_resume(
        &self,
        workflow_id: TaskId,
        branch_id: ExecutionBranchId,
    ) -> Option<BranchResumeMetadata> {
        self.resumes
            .lock()
            .unwrap()
            .get(&(workflow_id, branch_id))
            .and_then(|entries| entries.last().cloned())
    }
}

#[async_trait]
impl BranchCheckpointStore for InMemoryBranchCheckpointStore {
    async fn persist_branch_checkpoint(
        &self,
        workflow_id: TaskId,
        checkpoint: &BranchCheckpoint,
    ) -> Result<(), PersistenceError> {
        self.checkpoints
            .lock()
            .unwrap()
            .entry((workflow_id, checkpoint.branch_id))
            .or_default()
            .push(checkpoint.clone());
        Ok(())
    }

    async fn persist_branch_resume(
        &self,
        resume: &BranchResumeMetadata,
    ) -> Result<(), PersistenceError> {
        self.resumes
            .lock()
            .unwrap()
            .entry((resume.workflow_id, resume.branch_id))
            .or_default()
            .push(resume.clone());
        Ok(())
    }

    async fn latest_branch_checkpoint(
        &self,
        workflow_id: TaskId,
        branch_id: ExecutionBranchId,
    ) -> Result<Option<BranchCheckpoint>, PersistenceError> {
        Ok(self
            .checkpoints
            .lock()
            .unwrap()
            .get(&(workflow_id, branch_id))
            .and_then(|entries| entries.last().cloned()))
    }

    async fn branch_resume_history(
        &self,
        workflow_id: TaskId,
        branch_id: ExecutionBranchId,
    ) -> Result<Vec<BranchResumeMetadata>, PersistenceError> {
        Ok(self
            .resumes
            .lock()
            .unwrap()
            .get(&(workflow_id, branch_id))
            .cloned()
            .unwrap_or_default())
    }
}

fn assert_recovery_scope(plan: &BranchRecoveryPlan, expected_nodes: Vec<ExecutionNodeId>) {
    assert_eq!(plan.recovery_node_ids, expected_nodes);
    assert_eq!(
        plan.resume_metadata.recovery_node_ids,
        plan.recovery_node_ids
    );
}

#[tokio::test]
async fn checkpoint_coordinator_records_checkpoint_and_resumes_pending_nodes_only() {
    let mut graph = compile_graph();
    let workflow_id = graph.workflow_id;
    let branch_id = branch_id_for(&graph, "branch-a-1");
    let completed = node_id_for(&graph, "branch-a-1");
    let pending = node_id_for(&graph, "branch-a-2");
    let store = InMemoryBranchCheckpointStore::default();
    let coordinator = BranchCheckpointCoordinator::default();

    let checkpoint = BranchCheckpoint::new(
        branch_id,
        vec![completed],
        vec![pending],
        mister_smith_core::MemorySnapshotId::new(),
    );

    coordinator
        .record_checkpoint(&store, workflow_id, &mut graph, checkpoint.clone())
        .await
        .expect("checkpoint should record");

    assert_eq!(
        graph.branch(&branch_id).unwrap().state,
        BranchState::Checkpointed
    );
    assert_eq!(
        graph
            .latest_checkpoint(&branch_id)
            .expect("latest checkpoint should exist")
            .checkpoint_id,
        checkpoint.checkpoint_id
    );
    assert_eq!(store.checkpoint_count(workflow_id, branch_id), 1);

    let agent_id = AgentId::new();
    let recovery = coordinator
        .resume_branch(&store, workflow_id, &mut graph, branch_id, Some(agent_id))
        .await
        .expect("resume should use latest checkpoint");

    assert_eq!(
        recovery.resume_metadata.recovery_strategy,
        BranchRecoveryStrategy::Resume
    );
    assert_eq!(recovery.resume_metadata.assigned_agent, Some(agent_id));
    assert_eq!(
        graph.branch(&branch_id).unwrap().assigned_agents,
        vec![agent_id]
    );
    assert_eq!(
        graph.branch(&branch_id).unwrap().state,
        BranchState::Checkpointed
    );
    assert_recovery_scope(&recovery, vec![pending]);
}

#[tokio::test]
async fn checkpoint_coordinator_hydrates_durable_checkpoint_when_graph_lineage_is_cold() {
    let mut graph = compile_graph();
    let workflow_id = graph.workflow_id;
    let branch_id = branch_id_for(&graph, "branch-a-1");
    let completed = node_id_for(&graph, "branch-a-1");
    let pending = node_id_for(&graph, "branch-a-2");
    let store = InMemoryBranchCheckpointStore::default();
    let coordinator = BranchCheckpointCoordinator::default();
    let checkpoint = BranchCheckpoint::new(
        branch_id,
        vec![completed],
        vec![pending],
        mister_smith_core::MemorySnapshotId::new(),
    );

    store
        .persist_branch_checkpoint(workflow_id, &checkpoint)
        .await
        .expect("durable checkpoint should persist without hydrating the graph");

    let recovery = coordinator
        .resume_branch(&store, workflow_id, &mut graph, branch_id, None)
        .await
        .expect("resume should recover from the durable checkpoint");

    assert_eq!(recovery.checkpoint.checkpoint_id, checkpoint.checkpoint_id);
    assert_recovery_scope(&recovery, vec![pending]);
    assert_eq!(
        graph
            .latest_checkpoint(&branch_id)
            .expect("durable checkpoint should hydrate graph lineage")
            .checkpoint_id,
        checkpoint.checkpoint_id
    );
}

#[tokio::test]
async fn checkpoint_coordinator_reassigns_failed_branch_without_touching_completed_siblings() {
    let mut graph = compile_graph();
    let workflow_id = graph.workflow_id;
    let branch_a = branch_id_for(&graph, "branch-a-1");
    let branch_b = branch_id_for(&graph, "branch-b-1");
    let completed = node_id_for(&graph, "branch-a-1");
    let pending = node_id_for(&graph, "branch-a-2");
    let store = InMemoryBranchCheckpointStore::default();
    let coordinator = BranchCheckpointCoordinator::default();

    graph.branch_mut(&branch_b).unwrap().state = BranchState::Completed;

    coordinator
        .record_checkpoint(
            &store,
            workflow_id,
            &mut graph,
            BranchCheckpoint::new(
                branch_a,
                vec![completed],
                vec![pending],
                mister_smith_core::MemorySnapshotId::new(),
            ),
        )
        .await
        .expect("checkpoint should record");

    let new_agent = AgentId::new();
    let recovery = coordinator
        .reassign_branch(&store, workflow_id, &mut graph, branch_a, new_agent)
        .await
        .expect("reassignment should plan recovery from latest checkpoint");

    assert_eq!(
        recovery.resume_metadata.recovery_strategy,
        BranchRecoveryStrategy::Reassign
    );
    assert_eq!(recovery.resume_metadata.assigned_agent, Some(new_agent));
    assert_eq!(
        graph.branch(&branch_a).unwrap().state,
        BranchState::Reassigned
    );
    assert_eq!(
        graph.branch(&branch_a).unwrap().recovery_strategy,
        BranchRecoveryStrategy::Reassign
    );
    assert_eq!(
        graph.branch(&branch_a).unwrap().assigned_agents,
        vec![new_agent]
    );
    assert_eq!(
        graph.branch(&branch_b).unwrap().state,
        BranchState::Completed
    );
    assert_recovery_scope(&recovery, vec![pending]);

    let persisted = store
        .latest_resume(workflow_id, branch_a)
        .expect("resume metadata should persist");
    assert_eq!(persisted.assigned_agent, Some(new_agent));
}

#[tokio::test]
async fn orchestrator_resume_branch_records_checkpoint_event_and_resume_history() {
    let scheduler = Arc::new(TaskScheduler::new());
    let orchestrator = Orchestrator::new(
        Arc::new(IdentityDecomposer),
        Arc::new(ArrayAggregator),
        scheduler,
    );
    let graph = compile_graph();
    let workflow_id = graph.workflow_id;
    let branch_id = branch_id_for(&graph, "branch-a-1");
    let completed = node_id_for(&graph, "branch-a-1");
    let pending = node_id_for(&graph, "branch-a-2");
    let store = InMemoryBranchCheckpointStore::default();
    let checkpoint = BranchCheckpoint::new(
        branch_id,
        vec![completed],
        vec![pending],
        mister_smith_core::MemorySnapshotId::new(),
    );
    orchestrator.register_execution_graph(graph);

    orchestrator
        .record_branch_checkpoint(&workflow_id, &store, checkpoint.clone())
        .await
        .expect("checkpoint should record through orchestrator");
    assert!(orchestrator
        .autonomy_events(&workflow_id)
        .iter()
        .any(|event| matches!(event, AutonomyEvent::CheckpointRecorded(_))));

    let resumed_agent = AgentId::new();
    let recovery = orchestrator
        .resume_branch(&workflow_id, &store, branch_id, Some(resumed_agent))
        .await
        .expect("resume should persist recovery metadata");

    assert_eq!(
        recovery.resume_metadata.recovery_strategy,
        BranchRecoveryStrategy::Resume
    );
    assert_eq!(recovery.resume_metadata.assigned_agent, Some(resumed_agent));
    assert_eq!(
        store
            .latest_resume(workflow_id, branch_id)
            .expect("resume history should persist")
            .assigned_agent,
        Some(resumed_agent)
    );
    assert_eq!(
        orchestrator
            .execution_graph(&workflow_id)
            .and_then(|graph| graph.branch(&branch_id).cloned())
            .expect("branch should remain registered")
            .assigned_agents,
        vec![resumed_agent]
    );
    assert!(orchestrator
        .autonomy_events(&workflow_id)
        .iter()
        .any(|event| matches!(event, AutonomyEvent::RoutingDecisionRecorded(_))));
}

#[tokio::test]
async fn orchestrator_reassign_branch_updates_branch_state_and_persists_history() {
    let scheduler = Arc::new(TaskScheduler::new());
    let orchestrator = Orchestrator::new(
        Arc::new(IdentityDecomposer),
        Arc::new(ArrayAggregator),
        scheduler,
    );
    let graph = compile_graph();
    let workflow_id = graph.workflow_id;
    let branch_id = branch_id_for(&graph, "branch-a-1");
    let completed = node_id_for(&graph, "branch-a-1");
    let pending = node_id_for(&graph, "branch-a-2");
    let store = InMemoryBranchCheckpointStore::default();
    orchestrator.register_execution_graph(graph);
    orchestrator
        .record_branch_checkpoint(
            &workflow_id,
            &store,
            BranchCheckpoint::new(
                branch_id,
                vec![completed],
                vec![pending],
                mister_smith_core::MemorySnapshotId::new(),
            ),
        )
        .await
        .expect("checkpoint should record before reassignment");

    let reassigned_agent = AgentId::new();
    let recovery = orchestrator
        .reassign_branch(&workflow_id, &store, branch_id, reassigned_agent)
        .await
        .expect("reassignment should persist recovery metadata");

    assert_eq!(
        recovery.resume_metadata.recovery_strategy,
        BranchRecoveryStrategy::Reassign
    );
    assert_eq!(
        recovery.resume_metadata.assigned_agent,
        Some(reassigned_agent)
    );
    assert_eq!(
        orchestrator
            .execution_graph(&workflow_id)
            .and_then(|graph| graph.branch(&branch_id).cloned())
            .expect("branch should remain registered")
            .state,
        BranchState::Reassigned
    );
    assert_eq!(
        orchestrator
            .execution_graph(&workflow_id)
            .and_then(|graph| graph.branch(&branch_id).cloned())
            .expect("branch should remain registered")
            .recovery_strategy,
        BranchRecoveryStrategy::Reassign
    );
    assert_eq!(
        store
            .latest_resume(workflow_id, branch_id)
            .expect("resume history should persist")
            .assigned_agent,
        Some(reassigned_agent)
    );
}
