use std::sync::Arc;

use chrono::Utc;
use mister_smith_agents::config::TaskState;
use mister_smith_agents::orchestrator::Orchestrator;
use mister_smith_agents::scheduler::{
    ArrayAggregator, IdentityDecomposer, TaskAssignment, TaskScheduler,
};
use mister_smith_agents::{BranchCheckpoint, TopologyCompiler, TopologySignals};
use mister_smith_core::{AgentId, BranchState, MemorySnapshotId, NodeState, TaskId};
use mister_smith_events::AutonomyEvent;
use serde_json::json;
use uuid::Uuid;

fn mixed_dependency_plan() -> serde_json::Value {
    json!({
        "goal": "gate10-mixed-dependencies",
        "steps": [
            {
                "id": "root",
                "step": 1,
                "action": "root",
                "description": "root"
            },
            {
                "id": "left",
                "step": 2,
                "action": "left",
                "description": "left",
                "depends_on": ["root"],
                "branch": "left"
            },
            {
                "id": "right-1",
                "step": 3,
                "action": "right-1",
                "description": "right-1",
                "depends_on": ["root"],
                "branch": "right"
            },
            {
                "id": "right-2",
                "step": 4,
                "action": "right-2",
                "description": "right-2",
                "depends_on": ["right-1"],
                "branch": "right"
            },
            {
                "id": "join",
                "step": 5,
                "action": "join",
                "description": "join",
                "depends_on": ["left", "right-2"],
                "branch": "join"
            }
        ]
    })
}

fn cyclic_plan() -> serde_json::Value {
    json!({
        "goal": "gate10-invalid-graph",
        "steps": [
            {
                "id": "first",
                "step": 1,
                "action": "first",
                "description": "first",
                "depends_on": ["second"]
            },
            {
                "id": "second",
                "step": 2,
                "action": "second",
                "description": "second",
                "depends_on": ["first"]
            }
        ]
    })
}

fn branch_id_for(
    graph: &mister_smith_agents::ExecutionGraph,
    step_key: &str,
) -> mister_smith_core::ExecutionBranchId {
    graph
        .nodes
        .iter()
        .find(|node| node.step_key == step_key)
        .map(|node| node.branch_id)
        .expect("expected branch to exist")
}

fn node_id_for(
    graph: &mister_smith_agents::ExecutionGraph,
    step_key: &str,
) -> mister_smith_core::ExecutionNodeId {
    graph
        .nodes
        .iter()
        .find(|node| node.step_key == step_key)
        .map(|node| node.node_id)
        .expect("expected node to exist")
}

fn submit_task(
    scheduler: &TaskScheduler,
    task_id: TaskId,
    task_type: &str,
    parent_task_id: TaskId,
    state: TaskState,
) {
    scheduler.submit(TaskAssignment {
        task_id,
        task_type: task_type.to_string(),
        priority: 128,
        deadline: None,
        input: json!({ "from": "gate10" }),
        output: None,
        state,
        assigned_to: None,
        parent_task_id: Some(parent_task_id),
        team_id: None,
        message_id: Uuid::new_v4(),
        created_at: Utc::now(),
        assigned_at: None,
        completed_at: (state == TaskState::Completed).then_some(Utc::now()),
        error_message: None,
    });
}

#[tokio::test]
async fn gate10_mixed_dependency_resume_preserves_completed_branches() {
    let scheduler = Arc::new(TaskScheduler::new());
    let orchestrator = Orchestrator::new(
        Arc::new(IdentityDecomposer),
        Arc::new(ArrayAggregator),
        scheduler.clone(),
    );
    let mut graph = TopologyCompiler::default()
        .compile(
            TaskId::new(),
            &mixed_dependency_plan(),
            &TopologySignals::default(),
        )
        .expect("mixed dependency graph should compile");
    let workflow_id = graph.workflow_id;
    let root = node_id_for(&graph, "root");
    let left = node_id_for(&graph, "left");
    let right_1 = node_id_for(&graph, "right-1");
    let right_2 = node_id_for(&graph, "right-2");
    let join = node_id_for(&graph, "join");
    let root_branch = branch_id_for(&graph, "root");
    let left_branch = branch_id_for(&graph, "left");
    let right_branch = branch_id_for(&graph, "right-1");

    graph.nodes.iter_mut().for_each(|node| {
        if node.node_id == root || node.node_id == left || node.node_id == right_1 {
            node.state = NodeState::Completed;
        }
    });
    graph.branch_mut(&root_branch).unwrap().state = BranchState::Completed;
    graph.branch_mut(&left_branch).unwrap().state = BranchState::Completed;
    graph.branch_mut(&right_branch).unwrap().state = BranchState::Checkpointed;
    graph.checkpoint_lineage.push(BranchCheckpoint::new(
        right_branch,
        vec![right_1],
        vec![right_2],
        MemorySnapshotId::new(),
    ));
    orchestrator.register_execution_graph(graph);

    submit_task(
        &scheduler,
        TaskId::from_uuid(*root.as_ref()),
        "root",
        workflow_id,
        TaskState::Completed,
    );
    submit_task(
        &scheduler,
        TaskId::from_uuid(*left.as_ref()),
        "left",
        workflow_id,
        TaskState::Completed,
    );
    submit_task(
        &scheduler,
        TaskId::from_uuid(*right_1.as_ref()),
        "right-1",
        workflow_id,
        TaskState::Completed,
    );
    submit_task(
        &scheduler,
        TaskId::from_uuid(*right_2.as_ref()),
        "right-2",
        workflow_id,
        TaskState::Failed,
    );
    submit_task(
        &scheduler,
        TaskId::from_uuid(*join.as_ref()),
        "join",
        workflow_id,
        TaskState::Pending,
    );

    let worker = AgentId::new();
    let decisions = orchestrator
        .route_ready_branches(&workflow_id, &[worker])
        .expect("Gate 10 resume should route only the failed branch scope");

    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].branch_id, right_branch);
    assert_eq!(
        decisions[0].task_ids,
        vec![TaskId::from_uuid(*right_2.as_ref())]
    );
    assert_eq!(
        scheduler
            .get(&TaskId::from_uuid(*left.as_ref()))
            .unwrap()
            .state,
        TaskState::Completed
    );
    assert_eq!(
        scheduler
            .get(&TaskId::from_uuid(*right_1.as_ref()))
            .unwrap()
            .state,
        TaskState::Completed
    );
    assert_eq!(
        scheduler
            .get(&TaskId::from_uuid(*join.as_ref()))
            .unwrap()
            .state,
        TaskState::Pending
    );
    assert!(orchestrator
        .autonomy_events(&workflow_id)
        .iter()
        .any(|event| matches!(event, AutonomyEvent::RoutingDecisionRecorded(_))));
}

#[test]
fn gate10_rejects_invalid_graph_before_dispatch() {
    let err = TopologyCompiler::default()
        .compile(TaskId::new(), &cyclic_plan(), &TopologySignals::default())
        .expect_err("invalid gate10 graph should be rejected before dispatch");

    assert!(matches!(
        err,
        mister_smith_core::TopologyError::CycleDetected { .. }
    ));
}
