use chrono::Utc;
use mister_smith_agents::scheduler::{ArrayAggregator, IdentityDecomposer, TaskScheduler};
use mister_smith_agents::{Orchestrator, TopologyCompiler, TopologySignals};
use mister_smith_core::{
    BranchState, DurableWorkflowEventKind, GraphState, MemorySnapshotId, NodeState, TaskId,
};
use mister_smith_persistence::WorkflowHistoryEventRecord;
use serde_json::json;
use uuid::Uuid;

fn history_event(
    workflow_id: TaskId,
    replay_position: u64,
    event_kind: DurableWorkflowEventKind,
    payload: serde_json::Value,
) -> WorkflowHistoryEventRecord {
    WorkflowHistoryEventRecord {
        workflow_id,
        event_id: Uuid::new_v4(),
        replay_position,
        event_kind,
        recorded_at: Utc::now(),
        actor_agent_id: None,
        source: Some("test".to_string()),
        branch_id: None,
        node_id: None,
        lifecycle_state: None,
        effect_boundary_id: None,
        compaction_id: None,
        parent_event_id: None,
        payload,
    }
}

fn compile_graph(workflow_id: TaskId) -> mister_smith_agents::ExecutionGraph {
    TopologyCompiler::default()
        .compile(
            workflow_id,
            &json!({
                "steps": [
                    {
                        "id": "root",
                        "step": 1,
                        "action": "collect",
                        "description": "Collect context",
                        "branch": "branch-a"
                    },
                    {
                        "id": "branch-a",
                        "step": 2,
                        "action": "draft",
                        "description": "Draft answer",
                        "branch": "branch-a",
                        "depends_on": ["root"]
                    }
                ]
            }),
            &TopologySignals::default(),
        )
        .expect("graph should compile")
}

fn canonical_node_states(graph: &mister_smith_agents::ExecutionGraph) -> Vec<(String, NodeState)> {
    let mut states = graph
        .nodes
        .iter()
        .map(|node| (node.step_key.clone(), node.state))
        .collect::<Vec<_>>();
    states.sort_by(|left, right| left.0.cmp(&right.0));
    states
}

fn canonical_checkpoints(
    graph: &mister_smith_agents::ExecutionGraph,
) -> Vec<(Vec<String>, Vec<String>, BranchState)> {
    let mut checkpoints = graph
        .checkpoint_lineage
        .iter()
        .map(|checkpoint| {
            let mut completed = checkpoint
                .completed_nodes
                .iter()
                .filter_map(|node_id| {
                    graph
                        .nodes
                        .iter()
                        .find(|node| node.node_id == *node_id)
                        .map(|node| node.step_key.clone())
                })
                .collect::<Vec<_>>();
            let mut pending = checkpoint
                .pending_nodes
                .iter()
                .filter_map(|node_id| {
                    graph
                        .nodes
                        .iter()
                        .find(|node| node.node_id == *node_id)
                        .map(|node| node.step_key.clone())
                })
                .collect::<Vec<_>>();
            completed.sort();
            pending.sort();
            let branch_state = graph
                .branch(&checkpoint.branch_id)
                .map_or(BranchState::Pending, |branch| branch.state);
            (completed, pending, branch_state)
        })
        .collect::<Vec<_>>();
    checkpoints.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    checkpoints
}

#[test]
fn replayed_history_rebuilds_the_same_graph_projection_twice() {
    let workflow_id = TaskId::new();
    let history = vec![
        history_event(
            workflow_id,
            1,
            DurableWorkflowEventKind::LifecycleChanged,
            json!({
                "graph_state": GraphState::Running,
            }),
        ),
        history_event(
            workflow_id,
            2,
            DurableWorkflowEventKind::NodeStateChanged,
            json!({
                "step_key": "root",
                "node_state": NodeState::Completed,
                "branch_state": BranchState::Running,
                "graph_state": GraphState::Running,
            }),
        ),
        history_event(
            workflow_id,
            3,
            DurableWorkflowEventKind::NodeStateChanged,
            json!({
                "step_key": "branch-a",
                "node_state": NodeState::Running,
                "branch_state": BranchState::Running,
                "graph_state": GraphState::Running,
            }),
        ),
    ];
    let orchestrator = Orchestrator::new(
        std::sync::Arc::new(IdentityDecomposer),
        std::sync::Arc::new(ArrayAggregator),
        std::sync::Arc::new(TaskScheduler::new()),
    );

    let first = orchestrator
        .replay_execution_graph_from_history(compile_graph(workflow_id), &history)
        .expect("first replay should succeed");
    let second = orchestrator
        .replay_execution_graph_from_history(compile_graph(workflow_id), &history)
        .expect("second replay should succeed");

    assert_eq!(first.state, GraphState::Running);
    assert_eq!(
        canonical_node_states(&first),
        canonical_node_states(&second)
    );
    assert_eq!(
        canonical_checkpoints(&first),
        canonical_checkpoints(&second)
    );
}

#[test]
fn replayed_history_restores_checkpoint_scope_from_stable_step_keys() {
    let workflow_id = TaskId::new();
    let checkpoint_id = mister_smith_core::CheckpointId::new();
    let memory_snapshot_id = MemorySnapshotId::new();
    let history = vec![history_event(
        workflow_id,
        1,
        DurableWorkflowEventKind::BranchStateChanged,
        json!({
            "branch_anchor_step_key": "branch-a",
            "branch_state": BranchState::Running,
            "recovery_strategy": mister_smith_core::BranchRecoveryStrategy::Resume,
            "assigned_agent_ids": [],
            "checkpoint_id": checkpoint_id,
            "memory_snapshot_id": memory_snapshot_id,
            "captured_at": Utc::now(),
            "completed_step_keys": ["root"],
            "pending_step_keys": ["branch-a"],
            "recovery_step_keys": ["branch-a"],
            "failure_context": {"reason": "resume from durable history"},
        }),
    )];
    let orchestrator = Orchestrator::new(
        std::sync::Arc::new(IdentityDecomposer),
        std::sync::Arc::new(ArrayAggregator),
        std::sync::Arc::new(TaskScheduler::new()),
    );

    let replayed = orchestrator
        .replay_execution_graph_from_history(compile_graph(workflow_id), &history)
        .expect("checkpoint replay should succeed");

    assert_eq!(replayed.checkpoint_lineage.len(), 1);
    assert_eq!(
        canonical_checkpoints(&replayed),
        vec![(
            vec!["root".to_string()],
            vec!["branch-a".to_string()],
            BranchState::Running
        )]
    );
    let pending_node = replayed
        .nodes
        .iter()
        .find(|node| node.step_key == "branch-a")
        .map(|node| node.node_id)
        .expect("branch step should exist");
    assert_eq!(
        replayed.checkpoint_lineage[0].pending_nodes,
        vec![pending_node],
        "checkpoint lineage should track reconstructed node ids from stable step keys"
    );
}

#[test]
fn compaction_snapshot_replay_restores_bounded_lineage_state() {
    let workflow_id = TaskId::new();
    let history = vec![
        history_event(
            workflow_id,
            9,
            DurableWorkflowEventKind::HistoryCompacted,
            json!({
                "graph_state": GraphState::Running,
                "lifecycle_state": "active",
                "branch_states": [
                    {
                        "branch_key": "branch-a",
                        "branch_state": BranchState::Running,
                        "recovery_strategy": mister_smith_core::BranchRecoveryStrategy::Resume,
                        "assigned_agent_ids": []
                    }
                ],
                "node_states": [
                    {
                        "step_key": "root",
                        "node_state": NodeState::Completed,
                        "branch_id": "branch-a"
                    },
                    {
                        "step_key": "branch-a",
                        "node_state": NodeState::Running,
                        "branch_id": "branch-a"
                    }
                ],
                "source_replay_start": 1,
                "source_replay_end": 8,
                "replay_start_position": 9,
                "preserved_lineage_note": "compacted replay positions 1-8 into snapshot 9"
            }),
        ),
        history_event(
            workflow_id,
            10,
            DurableWorkflowEventKind::NodeStateChanged,
            json!({
                "step_key": "branch-a",
                "node_state": NodeState::Completed,
                "branch_state": BranchState::Completed,
                "graph_state": GraphState::Completed,
            }),
        ),
    ];
    let orchestrator = Orchestrator::new(
        std::sync::Arc::new(IdentityDecomposer),
        std::sync::Arc::new(ArrayAggregator),
        std::sync::Arc::new(TaskScheduler::new()),
    );

    let replayed = orchestrator
        .replay_execution_graph_from_history(compile_graph(workflow_id), &history)
        .expect("compacted replay should succeed");

    assert_eq!(replayed.state, GraphState::Completed);
    assert_eq!(
        canonical_node_states(&replayed),
        vec![
            ("branch-a".to_string(), NodeState::Completed),
            ("root".to_string(), NodeState::Completed),
        ]
    );
}

#[test]
fn lifecycle_terminal_event_preserves_terminal_graph_state_after_recompute() {
    let workflow_id = TaskId::new();
    let history = vec![
        history_event(
            workflow_id,
            1,
            DurableWorkflowEventKind::LifecycleChanged,
            json!({
                "lifecycle_state": "failed",
            }),
        ),
        history_event(
            workflow_id,
            2,
            DurableWorkflowEventKind::NodeStateChanged,
            json!({
                "step_key": "root",
                "node_state": NodeState::Completed,
            }),
        ),
    ];
    let orchestrator = Orchestrator::new(
        std::sync::Arc::new(IdentityDecomposer),
        std::sync::Arc::new(ArrayAggregator),
        std::sync::Arc::new(TaskScheduler::new()),
    );

    let replayed = orchestrator
        .replay_execution_graph_from_history(compile_graph(workflow_id), &history)
        .expect("terminal lifecycle replay should succeed");

    assert_eq!(replayed.state, GraphState::Failed);
}
