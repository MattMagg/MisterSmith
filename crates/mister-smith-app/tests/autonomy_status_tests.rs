#![allow(dead_code)]

#[path = "../src/autonomy.rs"]
mod autonomy;
#[path = "../src/observability.rs"]
mod observability;

use mister_smith_core::{
    AgentId, BranchRecoveryStrategy, BranchState, BudgetPolicy, BudgetScope, CheckpointId,
    CoordinationPolicy, ExecutionBranchId, ExecutionGraphId, FailureClass, GraphState,
    GuardDecision, GuardDecisionId, GuardEvidence, HealthState, InterventionRecord,
    InterventionRecordId, InterventionType, MemorySnapshotId, ProfileSnapshotId, ProfileTarget,
    ProvenanceChain, ProvenanceLink, RevocationState, TaskId, TopologyKind, TopologyRationale,
};
use mister_smith_events::{
    AutonomyEvent, AutonomyEventEnvelope, AutonomyStatusView, BranchSummary, CapabilitySummary,
    CheckpointRecordSummary, ContextPressureSummary, DelegationAlert, ExecutionGraphSummary,
    RoutingDecisionSummary, TopologyPlanSummary,
};

fn sample_view() -> (AutonomyStatusView, GuardDecisionId, ExecutionBranchId) {
    let workflow_id = TaskId::new();
    let graph_id = ExecutionGraphId::new();
    let branch_id = ExecutionBranchId::new();
    let checkpoint_id = CheckpointId::new();
    let decision_id = GuardDecisionId::new();
    let capability_id = mister_smith_core::CapabilityId::new();
    let view = AutonomyStatusView {
        graph: ExecutionGraphSummary {
            graph_id,
            workflow_id,
            state: GraphState::Running,
            branch_count: 1,
            node_count: 3,
            active_topology: Some(TopologyKind::Sequential),
        },
        topology: TopologyPlanSummary {
            graph_id,
            topology_kind: TopologyKind::Sequential,
            parallelism_width: 1,
            coordination_policy: CoordinationPolicy::Barrier,
            rationale: TopologyRationale {
                dependency_shape: "single branch".to_string(),
                operational_signals: vec!["degraded stream".to_string()],
                selected_for: "minimize restart blast radius".to_string(),
                fallback_reason: Some("conservative fallback to sequential".to_string()),
            },
            fallback_topology: Some(TopologyKind::Sequential),
        },
        branches: vec![BranchSummary {
            branch_id,
            graph_id,
            state: BranchState::Checkpointed,
            assigned_agents: vec![AgentId::new()],
            checkpoint_id: Some(checkpoint_id),
            recovery_strategy: BranchRecoveryStrategy::Resume,
        }],
        checkpoint_lineage: vec![CheckpointRecordSummary {
            graph_id,
            branch_id,
            checkpoint_id,
            captured_at: chrono::Utc::now(),
            memory_snapshot_id: MemorySnapshotId::new(),
            completed_nodes: vec![],
            pending_nodes: vec![],
            recovery_strategy: BranchRecoveryStrategy::Resume,
            failure_context: Some(serde_json::json!({"reason": "stalled stream"})),
        }],
        memory_pressure: vec![ContextPressureSummary {
            budget_id: mister_smith_core::ContextBudgetId::new(),
            branch_id: Some(branch_id),
            scope: BudgetScope::Branch,
            max_units: 4096,
            reserved_units: 3500,
            policy: BudgetPolicy::Summarize,
        }],
        routing_history: vec![RoutingDecisionSummary {
            graph_id,
            branch_id,
            selected_agent: AgentId::new(),
            task_ids: vec![TaskId::new()],
            recovery_strategy: BranchRecoveryStrategy::Resume,
            checkpoint_id: Some(checkpoint_id),
            dependency_depth: 1,
            budget_pressure: 88,
            health_state: HealthState::Degraded,
            profile_id: Some(ProfileSnapshotId::new()),
            rationale: vec!["checkpoint scope narrowed resume".to_string()],
        }],
        interventions: vec![InterventionRecord {
            record_id: InterventionRecordId::new(),
            decision_id,
            before_state: serde_json::json!({"state": "running"}),
            after_state: Some(serde_json::json!({"state": "checkpointed"})),
            rationale: "applied retry for targeted recovery".to_string(),
            emitted_at: chrono::Utc::now(),
        }],
        delegation_capabilities: vec![CapabilitySummary {
            capability_id,
            issuer: mister_smith_core::AuthorityPrincipal::Policy("operator".to_string()),
            recipient: AgentId::new(),
            scope: mister_smith_core::DelegationScope::InvokeTool,
            parent_capability: None,
            expires_at: chrono::Utc::now(),
            provenance: ProvenanceChain {
                root_issuer: mister_smith_core::AuthorityPrincipal::Policy("operator".to_string()),
                terminal_capability: capability_id,
                links: vec![ProvenanceLink {
                    issuer: mister_smith_core::AuthorityPrincipal::Policy("operator".to_string()),
                    recipient: AgentId::new(),
                    capability_id,
                    scope: mister_smith_core::DelegationScope::InvokeTool,
                    expires_at: chrono::Utc::now(),
                }],
            },
            revocation_state: RevocationState::Active,
            rejection_reason: None,
        }],
        delegation_alerts: vec![DelegationAlert {
            capability_id: Some(capability_id),
            scope: Some(mister_smith_core::DelegationScope::InvokeTool),
            revocation_state: Some(RevocationState::Revoked),
            parent_capability: None,
            expires_at: Some(chrono::Utc::now()),
            chain_depth: 1,
            rejection_reason: Some("delegation revoked before tool execution".to_string()),
            message: "operator review required".to_string(),
        }],
        profiles: vec![mister_smith_core::ProfileSnapshot {
            profile_id: ProfileSnapshotId::new(),
            target: ProfileTarget::Branch,
            health_state: HealthState::Degraded,
            latency_window: None,
            error_window: None,
            semantic_signals: vec![],
            updated_at: chrono::Utc::now(),
        }],
        guard_decisions: vec![GuardDecision {
            decision_id,
            failure_class: FailureClass::Streaming,
            intervention: InterventionType::Retry,
            evidence: GuardEvidence {
                profile_id: None,
                signal_descriptions: vec!["stream stalled before completion".to_string()],
                checkpoint_ids: vec![checkpoint_id],
                notes: vec!["conservative fallback: control-plane state unavailable".to_string()],
            },
            target_scope: mister_smith_core::GuardTarget::Branch(branch_id),
            operator_visibility: true,
        }],
        conservative_reasons: vec![
            "conservative fallback: control-plane state unavailable".to_string()
        ],
    };

    (view, decision_id, branch_id)
}

#[test]
fn render_status_surfaces_operator_rationale_and_history() {
    let (view, _, branch_id) = sample_view();
    let rendered = autonomy::render_status(&view);

    assert!(rendered.contains("minimize restart blast radius"));
    assert!(rendered.contains(&branch_id.to_string()));
    assert!(rendered.contains("checkpoint scope narrowed resume"));
    assert!(rendered.contains("applied retry for targeted recovery"));
    assert!(rendered.contains("delegation:"));
    assert!(rendered.contains("lineage="));
    assert!(rendered.contains("delegation revoked before tool execution"));
    assert!(rendered.contains("control-plane state unavailable"));
}

#[test]
fn metric_operations_cover_checkpoint_pressure_and_intervention_visibility() {
    let (view, decision_id, branch_id) = sample_view();
    let event = AutonomyEvent::InterventionRecorded(AutonomyEventEnvelope {
        workflow_id: view.graph.workflow_id,
        graph_id: Some(view.graph.graph_id),
        branch_id: Some(branch_id),
        payload: InterventionRecord {
            record_id: InterventionRecordId::new(),
            decision_id,
            before_state: serde_json::json!({"state": "running"}),
            after_state: Some(serde_json::json!({"state": "checkpointed"})),
            rationale: "applied retry for targeted recovery".to_string(),
            emitted_at: chrono::Utc::now(),
        },
        operator_visible: true,
    });

    let operations = observability::build_metric_operations(&event, &view);

    assert!(operations.iter().any(|operation| {
        operation.name == "mistersmith_autonomy_branch_checkpoint_age_seconds"
            && operation
                .labels
                .iter()
                .any(|(key, value)| key == "branch_id" && value == &branch_id.to_string())
    }));
    assert!(operations.iter().any(|operation| {
        operation.name == "mistersmith_autonomy_context_pressure_ratio"
            && operation
                .labels
                .iter()
                .any(|(key, value)| key == "pressure_level" && value == "elevated")
    }));
    assert!(operations.iter().any(|operation| {
        operation.name == "mistersmith_autonomy_delegation_chain_depth"
            && operation.kind == observability::MetricOperationKind::Gauge
    }));
    assert!(operations.iter().any(|operation| {
        operation.name == "mistersmith_autonomy_interventions_total"
            && operation.kind == observability::MetricOperationKind::Counter
            && operation
                .labels
                .iter()
                .any(|(key, value)| key == "intervention" && value == "retry")
    }));
    assert!(operations.iter().any(|operation| {
        operation.name == "mistersmith_autonomy_branches"
            && operation.value == 1.0
            && operation
                .labels
                .iter()
                .any(|(key, value)| key == "state" && value == "checkpointed")
    }));
    assert!(operations.iter().any(|operation| {
        operation.name == "mistersmith_autonomy_branches"
            && operation.value == 0.0
            && operation
                .labels
                .iter()
                .any(|(key, value)| key == "state" && value == "running")
            && operation
                .labels
                .iter()
                .any(|(key, value)| key == "branch_id" && value == &branch_id.to_string())
    }));
}

#[test]
fn delegation_rejection_metrics_use_operator_visible_reason() {
    let (view, _, branch_id) = sample_view();
    let mut rejected = view.delegation_capabilities[0].clone();
    rejected.revocation_state = RevocationState::Revoked;
    rejected.rejection_reason = Some("delegation revoked before tool execution".to_string());
    let event = AutonomyEvent::DelegationUpdated(AutonomyEventEnvelope {
        workflow_id: view.graph.workflow_id,
        graph_id: Some(view.graph.graph_id),
        branch_id: Some(branch_id),
        payload: rejected,
        operator_visible: true,
    });

    let operations = observability::build_metric_operations(&event, &view);

    assert!(operations.iter().any(|operation| {
        operation.name == "mistersmith_autonomy_delegation_rejections_total"
            && operation.kind == observability::MetricOperationKind::Counter
            && operation.labels.iter().any(|(key, value)| {
                key == "reason" && value == "delegation revoked before tool execution"
            })
    }));
}
