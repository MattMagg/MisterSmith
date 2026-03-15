use std::sync::Arc;

use chrono::Utc;
use mister_smith_agents::config::TaskState;
use mister_smith_agents::orchestrator::Orchestrator;
use mister_smith_agents::scheduler::{
    ArrayAggregator, IdentityDecomposer, TaskAssignment, TaskScheduler,
};
use mister_smith_agents::{
    BranchCheckpoint, GuardContext, ProfileAssessment, TopologyCompiler, TopologySignals,
};
use mister_smith_core::{
    AgentId, BranchState, CheckpointId, ExecutionBranchId, FailureClass, GuardTarget,
    HealthState, InterventionType, MemorySnapshotId, NodeState, ProfileSnapshot,
    ProfileSnapshotId, ProfileTarget, SemanticSignal, SemanticSignalKind, TaskId,
};
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

struct MixedDependencyFixture {
    scheduler: Arc<TaskScheduler>,
    orchestrator: Orchestrator,
    workflow_id: TaskId,
    right_branch: ExecutionBranchId,
    checkpoint_id: CheckpointId,
}

fn checkpointed_fixture() -> MixedDependencyFixture {
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
    let checkpoint = BranchCheckpoint::new(
        right_branch,
        vec![right_1],
        vec![right_2],
        MemorySnapshotId::new(),
    );
    let checkpoint_id = checkpoint.checkpoint_id;
    graph.checkpoint_lineage.push(checkpoint);
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

    MixedDependencyFixture {
        scheduler,
        orchestrator,
        workflow_id,
        right_branch,
        checkpoint_id,
    }
}

fn assessed_profile(
    branch_id: ExecutionBranchId,
    health_state: HealthState,
    semantic_signals: Vec<SemanticSignal>,
    notes: Vec<&str>,
) -> ProfileAssessment {
    ProfileAssessment::new(
        Some(ProfileSnapshot {
            profile_id: ProfileSnapshotId::new(),
            target: ProfileTarget::Branch,
            health_state,
            latency_window: None,
            error_window: None,
            semantic_signals,
            updated_at: Utc::now(),
        }),
        notes.into_iter().map(str::to_string).collect(),
    )
    .with_target(GuardTarget::Branch(branch_id))
}

#[tokio::test]
async fn gate10_mixed_dependency_resume_preserves_completed_branches() {
    let fixture = checkpointed_fixture();
    let graph = fixture
        .orchestrator
        .execution_graph(&fixture.workflow_id)
        .expect("execution graph should be registered");
    let left = node_id_for(&graph, "left");
    let right_1 = node_id_for(&graph, "right-1");
    let right_2 = node_id_for(&graph, "right-2");
    let join = node_id_for(&graph, "join");

    let worker = AgentId::new();
    let decisions = fixture
        .orchestrator
        .route_ready_branches(&fixture.workflow_id, &[worker])
        .expect("Gate 10 resume should route only the failed branch scope");

    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].branch_id, fixture.right_branch);
    assert_eq!(
        decisions[0].task_ids,
        vec![TaskId::from_uuid(*right_2.as_ref())]
    );
    assert_eq!(
        fixture
            .scheduler
            .get(&TaskId::from_uuid(*left.as_ref()))
            .unwrap()
            .state,
        TaskState::Completed
    );
    assert_eq!(
        fixture
            .scheduler
            .get(&TaskId::from_uuid(*right_1.as_ref()))
            .unwrap()
            .state,
        TaskState::Completed
    );
    assert_eq!(
        fixture
            .scheduler
            .get(&TaskId::from_uuid(*join.as_ref()))
            .unwrap()
            .state,
        TaskState::Pending
    );
    let status = fixture
        .orchestrator
        .autonomy_status(&fixture.workflow_id)
        .expect("operator-visible autonomy status should exist");
    assert_eq!(status.checkpoint_lineage.len(), 1);
    assert_eq!(status.routing_history.len(), 1);
    assert_eq!(status.checkpoint_lineage[0].checkpoint_id, fixture.checkpoint_id);
    assert!(status.routing_history[0]
        .rationale
        .iter()
        .any(|reason| reason.contains("checkpoint")));
    assert!(fixture
        .orchestrator
        .autonomy_events(&fixture.workflow_id)
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

#[tokio::test]
async fn gate10_transient_retry_remains_operator_visible() {
    let fixture = checkpointed_fixture();
    let context = GuardContext::new(GuardTarget::Branch(fixture.right_branch))
        .with_profile(assessed_profile(
            fixture.right_branch,
            HealthState::Degraded,
            Vec::new(),
            vec!["transient provider timeout"],
        ))
        .with_checkpoints(
            fixture
                .orchestrator
                .execution_graph(&fixture.workflow_id)
                .unwrap()
                .checkpoint_lineage,
        );

    let (decision, _) = fixture
        .orchestrator
        .supervise(&fixture.workflow_id, context)
        .await
        .expect("transient supervision should succeed");

    assert_eq!(decision.failure_class, FailureClass::Transient);
    assert_eq!(decision.intervention, InterventionType::Retry);

    let status = fixture
        .orchestrator
        .autonomy_status(&fixture.workflow_id)
        .expect("operator status should exist after transient retry");
    assert_eq!(status.guard_decisions.len(), 1);
    assert_eq!(status.interventions.len(), 1);
    assert_eq!(status.checkpoint_lineage[0].checkpoint_id, fixture.checkpoint_id);
}

#[tokio::test]
async fn gate10_streaming_retry_remains_operator_visible() {
    let fixture = checkpointed_fixture();
    let context = GuardContext::new(GuardTarget::Branch(fixture.right_branch))
        .with_profile(assessed_profile(
            fixture.right_branch,
            HealthState::Degraded,
            vec![SemanticSignal {
                signal_kind: SemanticSignalKind::Stalled,
                severity: 70,
                detail: "stream stalled before branch completion".to_string(),
            }],
            vec!["stream monitor observed stall"],
        ))
        .with_checkpoints(
            fixture
                .orchestrator
                .execution_graph(&fixture.workflow_id)
                .unwrap()
                .checkpoint_lineage,
        );

    let (decision, record) = fixture
        .orchestrator
        .supervise(&fixture.workflow_id, context)
        .await
        .expect("streaming supervision should succeed");

    assert_eq!(decision.failure_class, FailureClass::Streaming);
    assert_eq!(decision.intervention, InterventionType::Retry);
    assert!(record.rationale.contains("targeted recovery"));

    let status = fixture
        .orchestrator
        .autonomy_status(&fixture.workflow_id)
        .expect("operator status should exist after streaming retry");
    assert!(status.guard_decisions[0]
        .evidence
        .signal_descriptions
        .iter()
        .any(|detail| detail.contains("stalled")));
}

#[tokio::test]
async fn gate10_semantic_branch_isolation_remains_operator_visible() {
    let fixture = checkpointed_fixture();
    let context = GuardContext::new(GuardTarget::Branch(fixture.right_branch))
        .with_profile(assessed_profile(
            fixture.right_branch,
            HealthState::Unhealthy,
            vec![SemanticSignal {
                signal_kind: SemanticSignalKind::Repetitive,
                severity: 95,
                detail: "branch entered repetitive low-value loop".to_string(),
            }],
            vec!["semantic degradation exceeded safe threshold"],
        ))
        .with_checkpoints(
            fixture
                .orchestrator
                .execution_graph(&fixture.workflow_id)
                .unwrap()
                .checkpoint_lineage,
        );

    let (decision, _) = fixture
        .orchestrator
        .supervise(&fixture.workflow_id, context)
        .await
        .expect("semantic supervision should succeed");

    assert_eq!(decision.failure_class, FailureClass::Semantic);
    assert_eq!(decision.intervention, InterventionType::BranchIsolation);

    let status = fixture
        .orchestrator
        .autonomy_status(&fixture.workflow_id)
        .expect("operator status should exist after isolation");
    let branch = status
        .branches
        .iter()
        .find(|branch| branch.branch_id == fixture.right_branch)
        .expect("isolated branch should remain visible");
    assert_eq!(branch.state, BranchState::Isolated);
    assert_eq!(status.interventions.len(), 1);
}

#[tokio::test]
async fn gate10_missing_input_fallback_remains_operator_visible() {
    let fixture = checkpointed_fixture();
    let context = GuardContext::new(GuardTarget::Branch(fixture.right_branch))
        .with_control_plane_fresh(false)
        .with_memory_metadata_available(false)
        .with_checkpoints(
            fixture
                .orchestrator
                .execution_graph(&fixture.workflow_id)
                .unwrap()
                .checkpoint_lineage,
        );

    let (decision, record) = fixture
        .orchestrator
        .supervise(&fixture.workflow_id, context)
        .await
        .expect("missing-input fallback should escalate visibly");

    assert_eq!(decision.failure_class, FailureClass::Structural);
    assert_eq!(decision.intervention, InterventionType::Escalation);
    assert!(record.rationale.contains("conservative fallback"));

    let status = fixture
        .orchestrator
        .autonomy_status(&fixture.workflow_id)
        .expect("operator status should exist after conservative fallback");
    assert!(status
        .conservative_reasons
        .iter()
        .any(|reason| reason.contains("control-plane state unavailable")));
    assert!(status
        .conservative_reasons
        .iter()
        .any(|reason| reason.contains("memory metadata unavailable")));
}
