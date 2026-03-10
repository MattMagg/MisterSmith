use std::sync::Arc;

use chrono::Utc;
use mister_smith_agents::scheduler::{TaskAssignment, TaskScheduler};
use mister_smith_agents::{
    BranchCheckpoint, ExecutionGraph, Guard, GuardContext, GuardPolicy, InterventionEngine,
    ProfileAssessment, TopologyCompiler, TopologySignals,
};
use mister_smith_core::{
    BranchState, ExecutionBranchId, ExecutionNodeId, FailureClass, GraphState, GuardTarget,
    HealthState, InterventionType, SemanticSignal, SemanticSignalKind, TaskId,
};
use serde_json::json;

fn semantic_signal(kind: SemanticSignalKind, severity: u8, detail: &str) -> SemanticSignal {
    SemanticSignal {
        signal_kind: kind,
        severity,
        detail: detail.to_string(),
    }
}

fn stalled_profile_context(branch_id: ExecutionBranchId) -> GuardContext {
    let assessment = ProfileAssessment::new(
        Some(mister_smith_core::ProfileSnapshot {
            profile_id: mister_smith_core::ProfileSnapshotId::new(),
            target: mister_smith_core::ProfileTarget::Branch,
            health_state: HealthState::Degraded,
            latency_window: None,
            error_window: None,
            semantic_signals: vec![semantic_signal(
                SemanticSignalKind::Stalled,
                88,
                "stream stalled mid-branch",
            )],
            updated_at: Utc::now(),
        }),
        Vec::new(),
    );

    GuardContext::new(GuardTarget::Branch(branch_id))
        .with_profile(assessment)
        .with_checkpoints(vec![BranchCheckpoint {
            checkpoint_id: mister_smith_core::CheckpointId::new(),
            branch_id,
        }])
}

fn semantic_profile_context(branch_id: ExecutionBranchId) -> GuardContext {
    let assessment = ProfileAssessment::new(
        Some(mister_smith_core::ProfileSnapshot {
            profile_id: mister_smith_core::ProfileSnapshotId::new(),
            target: mister_smith_core::ProfileTarget::Branch,
            health_state: HealthState::Degraded,
            latency_window: None,
            error_window: None,
            semantic_signals: vec![semantic_signal(
                SemanticSignalKind::Repetitive,
                72,
                "analysis loop repeated the same conclusion",
            )],
            updated_at: Utc::now(),
        }),
        Vec::new(),
    );

    GuardContext::new(GuardTarget::Branch(branch_id)).with_profile(assessment)
}

fn branch_graph() -> (
    ExecutionGraph,
    Vec<TaskId>,
    ExecutionBranchId,
    ExecutionNodeId,
) {
    let compiler = TopologyCompiler::default();
    let workflow_id = TaskId::new();
    let graph = compiler
        .compile(
            workflow_id,
            &json!({
                "goal": "guard-branch",
                "steps": [
                    {
                        "id": "collect",
                        "step": 1,
                        "action": "collect",
                        "description": "Collect context"
                    },
                    {
                        "id": "branch-a",
                        "step": 2,
                        "action": "branch-a",
                        "description": "Branch A",
                        "depends_on": ["collect"],
                        "branch": "branch-a"
                    },
                    {
                        "id": "branch-b",
                        "step": 3,
                        "action": "branch-b",
                        "description": "Branch B",
                        "depends_on": ["collect"],
                        "branch": "branch-b"
                    }
                ]
            }),
            &TopologySignals::default(),
        )
        .expect("graph should compile");

    let node_id = graph
        .nodes
        .iter()
        .find(|node| node.step_key == "branch-a")
        .map(|node| node.node_id)
        .expect("branch-a node should exist");
    let branch_id = graph
        .nodes
        .iter()
        .find(|node| node.node_id == node_id)
        .map(|node| node.branch_id)
        .expect("branch-a branch should exist");
    let task_ids = graph
        .nodes
        .iter()
        .map(|node| TaskId::from_uuid(*node.node_id.as_ref()))
        .collect::<Vec<_>>();

    (graph, task_ids, branch_id, node_id)
}

fn scheduler_for_graph(task_ids: &[TaskId]) -> Arc<TaskScheduler> {
    let scheduler = Arc::new(TaskScheduler::new());
    for task_id in task_ids {
        let mut task = TaskAssignment::new("analysis", json!({"branch": "guard"}));
        task.task_id = *task_id;
        scheduler.submit(task);
    }
    scheduler
}

#[test]
fn guard_classifies_stalled_branch_as_streaming_retry() {
    let (_, _, branch_id, _) = branch_graph();
    let guard = Guard::new(GuardPolicy::default());
    let decision = guard
        .evaluate(&stalled_profile_context(branch_id))
        .expect("guard decision should succeed");

    assert_eq!(decision.failure_class, FailureClass::Streaming);
    assert_eq!(decision.intervention, InterventionType::Retry);
    assert_eq!(decision.target_scope, GuardTarget::Branch(branch_id));
    assert!(decision.operator_visibility);
}

#[test]
fn guard_classifies_semantic_drift_as_context_refresh() {
    let (_, _, branch_id, _) = branch_graph();
    let guard = Guard::new(GuardPolicy::default());
    let decision = guard
        .evaluate(&semantic_profile_context(branch_id))
        .expect("guard decision should succeed");

    assert_eq!(decision.failure_class, FailureClass::Semantic);
    assert_eq!(decision.intervention, InterventionType::ContextRefresh);
}

#[test]
fn guard_falls_back_conservatively_when_profile_or_control_plane_is_missing() {
    let (_, _, branch_id, _) = branch_graph();
    let guard = Guard::new(GuardPolicy::default());
    let decision = guard
        .evaluate(
            &GuardContext::new(GuardTarget::Branch(branch_id))
                .with_control_plane_fresh(false)
                .with_memory_metadata_available(false),
        )
        .expect("guard decision should succeed");

    assert_eq!(decision.intervention, InterventionType::Escalation);
    assert!(decision.operator_visibility);
    assert!(decision
        .evidence
        .notes
        .iter()
        .any(|note| note.contains("conservative fallback")));
}

#[test]
fn intervention_engine_isolates_branch_without_restarting_other_branches() {
    let (mut graph, task_ids, branch_id, node_id) = branch_graph();
    let scheduler = scheduler_for_graph(&task_ids);
    let task_id = TaskId::from_uuid(*node_id.as_ref());

    scheduler
        .assign(&task_id, mister_smith_core::AgentId::new())
        .unwrap();
    scheduler.start(&task_id).unwrap();
    scheduler.fail(&task_id, "semantic degradation").unwrap();

    let guard = Guard::new(GuardPolicy::default());
    let decision = guard
        .evaluate(
            &GuardContext::new(GuardTarget::Branch(branch_id)).with_profile(
                ProfileAssessment::new(
                    Some(mister_smith_core::ProfileSnapshot {
                        profile_id: mister_smith_core::ProfileSnapshotId::new(),
                        target: mister_smith_core::ProfileTarget::Branch,
                        health_state: HealthState::Unhealthy,
                        latency_window: None,
                        error_window: None,
                        semantic_signals: vec![semantic_signal(
                            SemanticSignalKind::Repetitive,
                            95,
                            "branch is trapped in a reasoning loop",
                        )],
                        updated_at: Utc::now(),
                    }),
                    Vec::new(),
                ),
            ),
        )
        .expect("guard decision should succeed");

    assert_eq!(decision.intervention, InterventionType::BranchIsolation);

    let record = InterventionEngine::default()
        .apply(&decision, &scheduler, &mut graph)
        .expect("intervention should apply");

    let isolated_branch = graph.branch(&branch_id).expect("branch should still exist");
    assert_eq!(isolated_branch.state, BranchState::Isolated);
    assert_eq!(graph.state, GraphState::Running);
    assert_eq!(record.decision_id, decision.decision_id);
    assert!(record.rationale.contains("branch isolation"));

    let branch_task = scheduler.get(&task_id).expect("task should still exist");
    assert_eq!(
        branch_task.state,
        mister_smith_agents::config::TaskState::Cancelled
    );

    let unaffected = task_ids
        .iter()
        .filter(|candidate| **candidate != task_id)
        .filter_map(|candidate| scheduler.get(candidate))
        .collect::<Vec<_>>();
    assert!(unaffected
        .iter()
        .all(|task| task.state == mister_smith_agents::config::TaskState::Pending));
}
