use std::sync::Arc;

use chrono::Utc;
#[cfg(feature = "llm")]
use mister_smith_agents::orchestrator::{LlmSupervision, LlmSupervisionConfig};
use mister_smith_agents::scheduler::{TaskAssignment, TaskScheduler};
use mister_smith_agents::{
    BranchCheckpoint, ExecutionGraph, Guard, GuardContext, GuardPolicy, InterventionEngine,
    ProfileAssessment, TopologyCompiler, TopologySignals,
};
use mister_smith_core::{
    BranchState, ExecutionBranchId, ExecutionNodeId, FailureClass, GraphState, GuardTarget,
    HealthState, InterventionType, ProfileFingerprint, ProfileFingerprintId, ProfileTarget,
    SemanticSignal, SemanticSignalKind, SupervisionDecisionBasis, TaskId,
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
            fingerprint_ref: None,
            updated_at: Utc::now(),
        }),
        Vec::new(),
    );

    GuardContext::new(GuardTarget::Branch(branch_id))
        .with_profile(assessment)
        .with_checkpoints(vec![BranchCheckpoint {
            checkpoint_id: mister_smith_core::CheckpointId::new(),
            branch_id,
            completed_nodes: Vec::new(),
            pending_nodes: Vec::new(),
            memory_snapshot_id: mister_smith_core::MemorySnapshotId::new(),
            failure_context: None,
            created_at: Utc::now(),
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
            fingerprint_ref: None,
            updated_at: Utc::now(),
        }),
        Vec::new(),
    );

    GuardContext::new(GuardTarget::Branch(branch_id)).with_profile(assessment)
}

fn profile_fingerprint(
    branch_id: ExecutionBranchId,
    dominant_failure_modes: Vec<&str>,
    preferred_interventions: Vec<InterventionType>,
) -> ProfileFingerprint {
    ProfileFingerprint {
        fingerprint_id: ProfileFingerprintId::new(),
        target_kind: "branch".to_string(),
        target_selector: branch_id.to_string(),
        source_refs: vec!["workflow:test".to_string()],
        summary_payload: json!({
            "health_state": "degraded",
            "signal_count": 1,
        }),
        dominant_failure_modes: dominant_failure_modes
            .into_iter()
            .map(str::to_string)
            .collect(),
        preferred_interventions,
        confidence: 0.84,
        updated_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::hours(6),
    }
}

fn branch_graph() -> (
    ExecutionGraph,
    Vec<TaskId>,
    ExecutionBranchId,
    ExecutionNodeId,
) {
    let compiler = TopologyCompiler;
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

fn checkpointed_branch_graph() -> (
    ExecutionGraph,
    Vec<TaskId>,
    ExecutionBranchId,
    ExecutionNodeId,
    ExecutionNodeId,
    ExecutionNodeId,
    ExecutionNodeId,
) {
    let compiler = TopologyCompiler;
    let workflow_id = TaskId::new();
    let mut graph = compiler
        .compile(
            workflow_id,
            &json!({
                "goal": "guard-checkpoint-scope",
                "steps": [
                    {
                        "id": "collect",
                        "step": 1,
                        "action": "collect",
                        "description": "Collect context"
                    },
                    {
                        "id": "branch-a-1",
                        "step": 2,
                        "action": "branch-a-1",
                        "description": "Branch A step 1",
                        "depends_on": ["collect"],
                        "branch": "branch-a"
                    },
                    {
                        "id": "branch-a-2",
                        "step": 3,
                        "action": "branch-a-2",
                        "description": "Branch A step 2",
                        "depends_on": ["branch-a-1"],
                        "branch": "branch-a"
                    },
                    {
                        "id": "branch-b",
                        "step": 4,
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

    let collect_node = graph
        .nodes
        .iter()
        .find(|node| node.step_key == "collect")
        .map(|node| node.node_id)
        .expect("collect node should exist");
    let branch_a_node_1 = graph
        .nodes
        .iter()
        .find(|node| node.step_key == "branch-a-1")
        .map(|node| node.node_id)
        .expect("branch-a-1 node should exist");
    let branch_a_node_2 = graph
        .nodes
        .iter()
        .find(|node| node.step_key == "branch-a-2")
        .map(|node| node.node_id)
        .expect("branch-a-2 node should exist");
    let branch_b_node = graph
        .nodes
        .iter()
        .find(|node| node.step_key == "branch-b")
        .map(|node| node.node_id)
        .expect("branch-b node should exist");
    let branch_id = graph
        .nodes
        .iter()
        .find(|node| node.node_id == branch_a_node_1)
        .map(|node| node.branch_id)
        .expect("branch-a branch should exist");
    graph.checkpoint_lineage.push(BranchCheckpoint {
        checkpoint_id: mister_smith_core::CheckpointId::new(),
        branch_id,
        completed_nodes: vec![branch_a_node_1],
        pending_nodes: vec![branch_a_node_2],
        memory_snapshot_id: mister_smith_core::MemorySnapshotId::new(),
        failure_context: None,
        created_at: Utc::now(),
    });

    let task_ids = graph
        .nodes
        .iter()
        .map(|node| TaskId::from_uuid(*node.node_id.as_ref()))
        .collect::<Vec<_>>();

    (
        graph,
        task_ids,
        branch_id,
        collect_node,
        branch_a_node_1,
        branch_a_node_2,
        branch_b_node,
    )
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

fn branch_decision(
    branch_id: ExecutionBranchId,
    intervention: InterventionType,
) -> mister_smith_core::GuardDecision {
    mister_smith_core::GuardDecision {
        decision_id: mister_smith_core::GuardDecisionId::new(),
        failure_class: FailureClass::Streaming,
        intervention,
        evidence: mister_smith_core::GuardEvidence {
            profile_id: None,
            decision_basis: SupervisionDecisionBasis::LiveSignalsOnly,
            signal_descriptions: vec!["stream stalled".to_string()],
            checkpoint_ids: Vec::new(),
            notes: Vec::new(),
        },
        target_scope: GuardTarget::Branch(branch_id),
        operator_visibility: true,
    }
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
fn guard_marks_decision_as_fingerprint_reinforced_when_advisory_context_matches() {
    let (_, _, branch_id, _) = branch_graph();
    let guard = Guard::new(GuardPolicy::default());
    let decision = guard
        .evaluate(
            &GuardContext::new(GuardTarget::Branch(branch_id)).with_profile(
                ProfileAssessment::new(
                    Some(mister_smith_core::ProfileSnapshot {
                        profile_id: mister_smith_core::ProfileSnapshotId::new(),
                        target: ProfileTarget::Branch,
                        health_state: HealthState::Degraded,
                        latency_window: None,
                        error_window: None,
                        semantic_signals: vec![semantic_signal(
                            SemanticSignalKind::MissingContext,
                            75,
                            "missing branch-local repair context",
                        )],
                        fingerprint_ref: None,
                        updated_at: Utc::now(),
                    }),
                    Vec::new(),
                )
                .with_fingerprint(profile_fingerprint(
                    branch_id,
                    vec!["missing_context"],
                    vec![InterventionType::ContextRefresh],
                )),
            ),
        )
        .expect("guard decision should succeed");

    assert_eq!(decision.intervention, InterventionType::ContextRefresh);
    assert_eq!(
        decision.evidence.decision_basis,
        SupervisionDecisionBasis::FingerprintReinforced
    );
    assert!(decision
        .evidence
        .notes
        .iter()
        .any(|note| note.contains("fingerprint reinforced")));
}

#[test]
fn guard_ignores_non_matching_fingerprint_context() {
    let (_, _, branch_id, _) = branch_graph();
    let guard = Guard::new(GuardPolicy::default());
    let decision = guard
        .evaluate(
            &GuardContext::new(GuardTarget::Branch(branch_id)).with_profile(
                ProfileAssessment::new(
                    Some(mister_smith_core::ProfileSnapshot {
                        profile_id: mister_smith_core::ProfileSnapshotId::new(),
                        target: ProfileTarget::Branch,
                        health_state: HealthState::Degraded,
                        latency_window: None,
                        error_window: None,
                        semantic_signals: vec![semantic_signal(
                            SemanticSignalKind::MissingContext,
                            75,
                            "missing branch-local repair context",
                        )],
                        fingerprint_ref: None,
                        updated_at: Utc::now(),
                    }),
                    Vec::new(),
                )
                .with_fingerprint(profile_fingerprint(
                    branch_id,
                    vec!["policy_conflict"],
                    vec![InterventionType::Retry],
                )),
            ),
        )
        .expect("guard decision should succeed");

    assert_eq!(decision.intervention, InterventionType::ContextRefresh);
    assert_eq!(
        decision.evidence.decision_basis,
        SupervisionDecisionBasis::LiveSignalsOnly
    );
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
                        fingerprint_ref: None,
                        updated_at: Utc::now(),
                    }),
                    Vec::new(),
                ),
            ),
        )
        .expect("guard decision should succeed");

    assert_eq!(decision.intervention, InterventionType::BranchIsolation);

    let record = InterventionEngine
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

#[cfg(feature = "llm")]
#[tokio::test]
async fn orchestrator_supervises_stream_degradation_and_forwards_messages() {
    use mister_smith_llm::ModelEvent;

    let (graph, task_ids, branch_id, node_id) = branch_graph();
    let workflow_id = graph.workflow_id;
    let task_id = TaskId::from_uuid(*node_id.as_ref());
    let scheduler = scheduler_for_graph(&task_ids);
    let agent = AgentId::new();

    scheduler.assign(&task_id, agent).unwrap();
    scheduler.start(&task_id).unwrap();
    scheduler.fail(&task_id, "stalled branch").unwrap();

    let orchestrator = Arc::new(Orchestrator::new(
        Arc::new(IdentityDecomposer),
        Arc::new(ArrayAggregator),
        scheduler.clone(),
    ));
    orchestrator.register_execution_graph(graph);
    let supervision = LlmSupervision::new(
        orchestrator.clone(),
        workflow_id,
        LlmSupervisionConfig::new(GuardTarget::Branch(branch_id)),
    );

    assert!(supervision
        .observe_model_event(&ModelEvent::StreamStarted {
            model_id: "gpt-test".to_string(),
            request_id: "req-1".to_string(),
        },)
        .await
        .expect("stream start should be observed")
        .is_none());
    assert!(supervision
        .observe_model_event(&ModelEvent::Heartbeat { sequence: 1 },)
        .await
        .expect("heartbeat should be observed")
        .is_none());
    assert!(supervision
        .observe_model_event(&ModelEvent::Heartbeat { sequence: 2 },)
        .await
        .expect("heartbeat should be observed")
        .is_none());

    let (guard_decision, record) = supervision
        .observe_model_event(&ModelEvent::Heartbeat { sequence: 3 })
        .await
        .expect("degradation should be observed")
        .expect("third idle heartbeat should trigger supervision");

    assert_eq!(guard_decision.failure_class, FailureClass::Streaming);
    assert_eq!(guard_decision.intervention, InterventionType::Retry);
    assert_eq!(record.decision_id, guard_decision.decision_id);

    let monitor_state = orchestrator
        .monitor_state(&workflow_id)
        .expect("monitor state should exist");
    assert_eq!(monitor_state.guard_decisions.len(), 1);
    assert_eq!(monitor_state.interventions.len(), 1);

    let supervisor_state = orchestrator
        .supervisor_state(&workflow_id)
        .expect("supervisor state should exist");
    assert_eq!(supervisor_state.guard_decisions.len(), 1);
    assert_eq!(supervisor_state.interventions.len(), 1);

    assert_eq!(
        scheduler.get(&task_id).expect("task should exist").state,
        mister_smith_agents::config::TaskState::Pending
    );

    let status = orchestrator
        .autonomy_status(&workflow_id)
        .expect("status should be available");
    assert_eq!(status.guard_decisions.len(), 1);
    assert_eq!(status.interventions.len(), 1);
    let supervision_evidence = status
        .supervision_evidence
        .expect("supervision evidence should be projected");
    assert_eq!(
        supervision_evidence.target_scope.kind,
        mister_smith_core::SupervisionTargetKind::Branch
    );
    assert_eq!(supervision_evidence.target_scope.branch_id, Some(branch_id));
    assert!(supervision_evidence.target_scope.node_id.is_none());
    assert!(supervision_evidence.fingerprint_ref.is_none());
    assert_eq!(
        supervision_evidence.decision_basis.as_deref(),
        Some(SupervisionDecisionBasis::LiveSignalsOnly.as_str())
    );
    assert!(status.guard_decisions[0]
        .evidence
        .notes
        .iter()
        .any(|note| note.contains("step boundary")));
}

#[test]
fn intervention_engine_retries_only_pending_nodes_from_latest_checkpoint() {
    let (
        mut graph,
        task_ids,
        branch_id,
        collect_node,
        branch_a_node_1,
        branch_a_node_2,
        branch_b_node,
    ) = checkpointed_branch_graph();
    let scheduler = scheduler_for_graph(&task_ids);
    let collect_task_id = TaskId::from_uuid(*collect_node.as_ref());
    let branch_a_task_1 = TaskId::from_uuid(*branch_a_node_1.as_ref());
    let branch_a_task_2 = TaskId::from_uuid(*branch_a_node_2.as_ref());
    let branch_b_task_id = TaskId::from_uuid(*branch_b_node.as_ref());

    scheduler
        .assign(&collect_task_id, mister_smith_core::AgentId::new())
        .unwrap();
    scheduler.start(&collect_task_id).unwrap();
    scheduler
        .complete(&collect_task_id, json!({"step": "collect"}))
        .unwrap();

    scheduler
        .assign(&branch_a_task_1, mister_smith_core::AgentId::new())
        .unwrap();
    scheduler.start(&branch_a_task_1).unwrap();
    scheduler
        .complete(&branch_a_task_1, json!({"step": "branch-a-1"}))
        .unwrap();

    scheduler
        .assign(&branch_a_task_2, mister_smith_core::AgentId::new())
        .unwrap();
    scheduler.start(&branch_a_task_2).unwrap();
    scheduler.fail(&branch_a_task_2, "stalled").unwrap();

    let record = InterventionEngine
        .apply(
            &branch_decision(branch_id, InterventionType::Retry),
            &scheduler,
            &mut graph,
        )
        .expect("retry intervention should apply");

    assert!(record.rationale.contains("retry"));
    assert_eq!(
        scheduler.get(&branch_a_task_1).expect("task exists").state,
        mister_smith_agents::config::TaskState::Completed
    );
    assert_eq!(
        scheduler.get(&branch_a_task_2).expect("task exists").state,
        mister_smith_agents::config::TaskState::Pending
    );
    assert_eq!(
        scheduler.get(&branch_b_task_id).expect("task exists").state,
        mister_smith_agents::config::TaskState::Pending
    );
}

#[test]
fn intervention_engine_propagates_branch_scheduler_failures() {
    let (mut graph, _task_ids, branch_id, ..) = checkpointed_branch_graph();
    let scheduler = Arc::new(TaskScheduler::new());

    let error = InterventionEngine
        .apply(
            &branch_decision(branch_id, InterventionType::Retry),
            &scheduler,
            &mut graph,
        )
        .expect_err("missing scheduler tasks should fail the intervention");

    assert!(matches!(
        error,
        mister_smith_core::GuardError::InvalidTarget(_)
    ));
}

#[test]
fn intervention_engine_escalation_cancels_branch_recovery_scope() {
    let (mut graph, task_ids, branch_id, collect_node, branch_a_node_1, branch_a_node_2, _) =
        checkpointed_branch_graph();
    let scheduler = scheduler_for_graph(&task_ids);
    let collect_task_id = TaskId::from_uuid(*collect_node.as_ref());
    let branch_a_task_1 = TaskId::from_uuid(*branch_a_node_1.as_ref());
    let branch_a_task_2 = TaskId::from_uuid(*branch_a_node_2.as_ref());

    scheduler
        .assign(&collect_task_id, mister_smith_core::AgentId::new())
        .unwrap();
    scheduler.start(&collect_task_id).unwrap();
    scheduler
        .complete(&collect_task_id, json!({"step": "collect"}))
        .unwrap();
    scheduler
        .assign(&branch_a_task_1, mister_smith_core::AgentId::new())
        .unwrap();
    scheduler.start(&branch_a_task_1).unwrap();
    scheduler
        .complete(&branch_a_task_1, json!({"step": "branch-a-1"}))
        .unwrap();
    scheduler
        .assign(&branch_a_task_2, mister_smith_core::AgentId::new())
        .unwrap();
    scheduler.start(&branch_a_task_2).unwrap();

    InterventionEngine
        .apply(
            &branch_decision(branch_id, InterventionType::Escalation),
            &scheduler,
            &mut graph,
        )
        .expect("escalation should apply");

    assert_eq!(
        scheduler.get(&branch_a_task_1).expect("task exists").state,
        mister_smith_agents::config::TaskState::Completed
    );
    assert_eq!(
        scheduler.get(&branch_a_task_2).expect("task exists").state,
        mister_smith_agents::config::TaskState::Cancelled
    );
}

#[test]
fn intervention_engine_graph_abort_fails_all_scheduled_work() {
    let (mut graph, task_ids, ..) = checkpointed_branch_graph();
    let scheduler = scheduler_for_graph(&task_ids);
    let running_task_id = task_ids[0];

    scheduler
        .assign(&running_task_id, mister_smith_core::AgentId::new())
        .unwrap();
    scheduler.start(&running_task_id).unwrap();

    let decision = mister_smith_core::GuardDecision {
        decision_id: mister_smith_core::GuardDecisionId::new(),
        failure_class: FailureClass::Structural,
        intervention: InterventionType::Abort,
        evidence: mister_smith_core::GuardEvidence {
            profile_id: None,
            decision_basis: SupervisionDecisionBasis::LiveSignalsOnly,
            signal_descriptions: vec!["operator abort".to_string()],
            checkpoint_ids: Vec::new(),
            notes: Vec::new(),
        },
        target_scope: GuardTarget::Graph(graph.graph_id),
        operator_visibility: true,
    };

    InterventionEngine
        .apply(&decision, &scheduler, &mut graph)
        .expect("graph abort should apply");

    assert_eq!(graph.state, GraphState::Aborted);
    for task_id in task_ids {
        assert_eq!(
            scheduler.get(&task_id).expect("task exists").state,
            mister_smith_agents::config::TaskState::Failed
        );
    }
}
