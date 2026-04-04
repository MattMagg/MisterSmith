#![allow(dead_code)]

#[path = "../src/auth.rs"]
mod auth;
#[path = "../src/autonomy.rs"]
mod autonomy;
#[path = "../src/conversation.rs"]
mod conversation;
#[path = "../src/execution.rs"]
mod execution;
#[path = "../src/observability.rs"]
mod observability;

use chrono::Utc;
use mister_smith_core::{
    AgentId, BranchRecoveryStrategy, BranchState, CoordinationPolicy,
    DurableWorkflowLifecycleState, DurableWorkflowLifecycleVerb, ExecutionBranchId,
    ExecutionGraphId, GraphState, LifecycleDecisionOutcome, SessionId, TaskId,
    TaskShapeClassification, TaskShapeKind, TopologyKind, TopologyRationale,
};
use mister_smith_events::{
    AutonomyStatusView, BranchSummary, ExecutionGraphSummary, TopologyPlanSummary,
};
use mister_smith_persistence::postgres::queries::TaskRecord;
use mister_smith_persistence::{merge_lifecycle_decision_metadata, LifecycleDecisionRecord};
use serde_json::json;
use uuid::Uuid;

fn sample_task_record(status: &str, metadata: serde_json::Value) -> TaskRecord {
    TaskRecord {
        task_id: *TaskId::new().as_ref(),
        task_type: "workflow".to_string(),
        agent_id: Some(*AgentId::new().as_ref()),
        payload: json!({ "description": "packet 022 lifecycle test" }),
        result: None,
        metadata,
        status: status.to_string(),
        priority: 2,
        correlation_id: None,
        parent_task_id: None,
        created_at: Utc::now(),
        started_at: Some(Utc::now()),
        completed_at: None,
        expires_at: None,
    }
}

fn sample_autonomy_view(workflow_id: TaskId) -> AutonomyStatusView {
    let graph_id = ExecutionGraphId::from_uuid(*workflow_id.as_ref());
    let branch_id = ExecutionBranchId::new();
    AutonomyStatusView {
        session_id: Some(SessionId::new()),
        turn_index: Some(1),
        coordinator_agent_id: Some(AgentId::new()),
        resume_provenance: None,
        lifecycle_state: Some(DurableWorkflowLifecycleState::Failed),
        graph: ExecutionGraphSummary {
            graph_id,
            workflow_id,
            state: GraphState::Failed,
            branch_count: 1,
            node_count: 1,
            active_topology: Some(TopologyKind::Sequential),
        },
        topology: TopologyPlanSummary {
            graph_id,
            topology_kind: TopologyKind::Sequential,
            parallelism_width: 1,
            task_shape: TaskShapeClassification {
                kind: TaskShapeKind::StrictChain,
                root_count: 1,
                max_parallel_width: 1,
                max_depth: 1,
                has_join: false,
                has_fanout: false,
                structural_signals: vec!["single_lane".to_string()],
            },
            coordination_policy: CoordinationPolicy::StrictSequence,
            rationale: TopologyRationale {
                selected_for: "lifecycle projection coverage".to_string(),
                dependency_shape: "single lane".to_string(),
                operational_signals: vec![],
                fallback_reason: None,
            },
            fallback_topology: None,
        },
        team_sizing: None,
        branches: vec![BranchSummary {
            branch_id,
            graph_id,
            state: BranchState::Failed,
            assigned_agents: vec![AgentId::new()],
            checkpoint_id: None,
            recovery_strategy: BranchRecoveryStrategy::Resume,
        }],
        checkpoint_lineage: vec![],
        memory_pressure: vec![],
        routing_history: vec![],
        step_routing_history: vec![],
        result_preview: None,
        interventions: vec![],
        delegation_capabilities: vec![],
        delegation_alerts: vec![],
        delegation_records: vec![],
        subordinate_inbox: vec![],
        subagent_states: vec![],
        delegated_work_evidence: vec![],
        coordinator_decisions: vec![],
        coordinator_runtime_proof: None,
        external_capability_decisions: vec![],
        profiles: vec![],
        guard_decisions: vec![],
        supervision_evidence: None,
        runtime_truth: None,
        step_policy: None,
        conservative_reasons: vec![],
    }
}

#[test]
fn terminated_lifecycle_projects_the_same_meaning_across_surfaces() {
    let workflow_id = TaskId::new();
    let command_id = Uuid::new_v4();
    let mut metadata = json!({
        "session_id": SessionId::new(),
        "turn_index": 3,
        "coordinator_agent_id": AgentId::new(),
    });
    let decision = LifecycleDecisionRecord {
        workflow_id,
        command_id,
        verb: DurableWorkflowLifecycleVerb::Terminate,
        requested_by_agent_id: None,
        source: Some("test".to_string()),
        requested_at: Utc::now(),
        reason: Some("operator stopped the run".to_string()),
        outcome: LifecycleDecisionOutcome::Applied,
        resulting_state: DurableWorkflowLifecycleState::Terminated,
        decided_at: Utc::now(),
        note: None,
    };
    merge_lifecycle_decision_metadata(&mut metadata, std::slice::from_ref(&decision))
        .expect("lifecycle metadata should merge");

    let mut record = sample_task_record("failed", metadata.clone());
    record.task_id = *workflow_id.as_ref();

    let task_view = execution::build_task_summary_view(&record);
    assert_eq!(
        task_view.lifecycle_state,
        DurableWorkflowLifecycleState::Terminated
    );
    assert_eq!(task_view.status, "failed");

    let turn_lifecycle = conversation::lifecycle_state_for_turn(Some(&metadata), &record.status);
    assert_eq!(turn_lifecycle, DurableWorkflowLifecycleState::Terminated);

    let mut autonomy_view = sample_autonomy_view(workflow_id);
    autonomy::enrich_lifecycle_state(&mut autonomy_view, &metadata);
    assert_eq!(
        autonomy_view.lifecycle_state,
        Some(DurableWorkflowLifecycleState::Terminated)
    );

    let rendered = autonomy::render_status(&autonomy_view);
    assert!(rendered.contains("lifecycle: terminated"));
}
