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
    ProvenanceChain, ProvenanceLink, RevocationState, TaskId, TaskShapeClassification,
    TaskShapeKind, TeamSizingDecision, TopologyKind, TopologyRationale,
};
use mister_smith_events::{
    AutonomyEvent, AutonomyEventEnvelope, AutonomyStatusView, BranchSummary, CapabilitySummary,
    CheckpointRecordSummary, ContextPressureSummary, DelegationAlert, ExecutionGraphSummary,
    ResumeProvenanceSummary, RoutingDecisionSummary, StepRoutingDecisionSummary,
    TopologyPlanSummary,
};

fn sample_task_shape(kind: TaskShapeKind) -> TaskShapeClassification {
    TaskShapeClassification {
        kind,
        root_count: 1,
        max_parallel_width: 1,
        max_depth: 2,
        has_join: false,
        has_fanout: false,
        structural_signals: vec![
            "roots:1".to_string(),
            "max_parallel_width:1".to_string(),
            "max_depth:2".to_string(),
        ],
    }
}

fn sample_team_sizing(workflow_id: TaskId, graph_id: ExecutionGraphId) -> TeamSizingDecision {
    TeamSizingDecision {
        workflow_id,
        graph_id,
        decision_phase: "initial".to_string(),
        desired_workers: 1,
        selected_workers: 1,
        available_workers: 1,
        branch_frontier_width: 1,
        dependency_depth: 2,
        conservative_mode: true,
        budget_pressure: Some(88),
        cap_reason: None,
        rationale_lines: vec![
            "task shape strict-chain with frontier width 1".to_string(),
            "selected 1 worker from the available pool".to_string(),
        ],
        decided_at: chrono::Utc::now(),
    }
}

fn sample_view() -> (AutonomyStatusView, GuardDecisionId, ExecutionBranchId) {
    let workflow_id = TaskId::new();
    let graph_id = ExecutionGraphId::new();
    let branch_id = ExecutionBranchId::new();
    let checkpoint_id = CheckpointId::new();
    let decision_id = GuardDecisionId::new();
    let capability_id = mister_smith_core::CapabilityId::new();
    let view = AutonomyStatusView {
        session_id: None,
        turn_index: None,
        coordinator_agent_id: None,
        resume_provenance: None,
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
            task_shape: sample_task_shape(TaskShapeKind::StrictChain),
            coordination_policy: CoordinationPolicy::Barrier,
            rationale: TopologyRationale {
                dependency_shape: "single branch".to_string(),
                operational_signals: vec!["degraded stream".to_string()],
                selected_for: "minimize restart blast radius".to_string(),
                fallback_reason: Some("conservative fallback to sequential".to_string()),
            },
            fallback_topology: Some(TopologyKind::Sequential),
        },
        team_sizing: Some(sample_team_sizing(workflow_id, graph_id)),
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
        step_routing_history: vec![StepRoutingDecisionSummary {
            step_id: "planner.step.2".to_string(),
            step_index: Some(2),
            step_kind: Some("planner".to_string()),
            model_id: "gpt-5.4".to_string(),
            tier: "llm-tier".to_string(),
            reason: "accepted at llm-tier after previous confidence review".to_string(),
            previous_step_id: Some("planner.step.1".to_string()),
            previous_action: Some("escalate".to_string()),
            previous_tier: Some("slm-tier".to_string()),
            action: "continue".to_string(),
            action_changed: true,
            preferred_tier_after: Some("llm-tier".to_string()),
            estimated_cost_tokens: Some(128),
            confidence_score: Some(0.92),
            triggered_checkpoints: vec![],
            change_rationale: vec![
                "previous step planner.step.1 ended with action=escalate tier=slm-tier".to_string(),
                "action changed from escalate to continue".to_string(),
                "preferred tier updated from slm-tier to llm-tier".to_string(),
            ],
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

fn sample_step_routing_history(
    step_id: &str,
    previous_step_id: Option<&str>,
    action: &str,
    action_changed: bool,
) -> StepRoutingDecisionSummary {
    StepRoutingDecisionSummary {
        step_id: step_id.to_string(),
        step_index: Some(2),
        step_kind: Some("planner".to_string()),
        model_id: "gpt-5.4".to_string(),
        tier: "llm-tier".to_string(),
        reason: "accepted at llm-tier after previous confidence review".to_string(),
        previous_step_id: previous_step_id.map(str::to_string),
        previous_action: Some("escalate".to_string()),
        previous_tier: Some("slm-tier".to_string()),
        action: action.to_string(),
        action_changed,
        preferred_tier_after: Some("llm-tier".to_string()),
        estimated_cost_tokens: Some(128),
        confidence_score: Some(0.92),
        triggered_checkpoints: vec![],
        change_rationale: vec![format!("action changed to {action}")],
    }
}

#[test]
fn render_status_surfaces_operator_rationale_and_history() {
    let (view, _, branch_id) = sample_view();
    let rendered = autonomy::render_status(&view);

    assert!(rendered.contains("minimize restart blast radius"));
    assert!(rendered.contains("shape=strict-chain"));
    assert!(rendered.contains("structure=roots:1 | max_parallel_width:1 | max_depth:2"));
    assert!(rendered.contains("dependency=single branch"));
    assert!(rendered.contains("signals=degraded stream"));
    assert!(rendered.contains("team sizing: phase=initial desired=1 selected=1"));
    assert!(rendered.contains("task shape strict-chain with frontier width 1"));
    assert!(rendered.contains(&branch_id.to_string()));
    assert!(rendered.contains("checkpoint scope narrowed resume"));
    assert!(rendered.contains("step routing:"));
    assert!(rendered.contains("planner.step.2#2"));
    assert!(rendered.contains("action changed from escalate to continue"));
    assert!(rendered.contains("preferred=llm-tier"));
    assert!(rendered.contains("applied retry for targeted recovery"));
    assert!(rendered.contains("delegation:"));
    assert!(rendered.contains("lineage="));
    assert!(rendered.contains("delegation revoked before tool execution"));
    assert!(rendered.contains("control-plane state unavailable"));
}

#[test]
fn render_status_surfaces_restart_resume_provenance() {
    let (mut view, _, _) = sample_view();
    let resumed_from_workflow_id = TaskId::new();
    view.session_id = Some(mister_smith_core::SessionId::new());
    view.turn_index = Some(2);
    view.coordinator_agent_id = Some(AgentId::new());
    view.resume_provenance = Some(ResumeProvenanceSummary {
        recovered_after_restart: true,
        resumed_after_restart: true,
        recovered_at: Some(chrono::Utc::now()),
        recovery_reason: Some(
            "workflow interrupted by runtime restart before session sync".to_string(),
        ),
        resumed_from_workflow_id: Some(resumed_from_workflow_id),
        resumed_from_turn_index: Some(1),
    });

    let rendered = autonomy::render_status(&view);

    assert!(rendered.contains("resume provenance:"));
    assert!(rendered.contains("recovered_after_restart=true"));
    assert!(rendered.contains("resumed_after_restart=true"));
    assert!(rendered.contains("resumed_from_turn=1"));
    assert!(rendered.contains(&format!("resumed_from_workflow={resumed_from_workflow_id}")));
    assert!(rendered.contains("runtime restart before session sync"));
}

#[test]
fn enrich_step_routing_history_preserves_live_history_over_stale_metadata() {
    let (mut view, _, _) = sample_view();
    let live_history = view.step_routing_history.clone();
    let metadata = serde_json::json!({
        "step_routing_history": [
            sample_step_routing_history("planner.step.1", None, "escalate", false)
        ]
    });

    autonomy::enrich_step_routing_history(&mut view, &metadata);

    assert_eq!(view.step_routing_history, live_history);
    assert_eq!(view.step_routing_history[0].step_id, "planner.step.2");
}

#[test]
fn render_status_surfaces_capped_parallel_team_decisions() {
    let (mut view, _, _) = sample_view();
    let workflow_id = view.graph.workflow_id;
    let graph_id = view.graph.graph_id;

    view.topology.topology_kind = TopologyKind::Parallel;
    view.topology.parallelism_width = 3;
    view.topology.task_shape = TaskShapeClassification {
        kind: TaskShapeKind::ParallelFanout,
        root_count: 1,
        max_parallel_width: 3,
        max_depth: 2,
        has_join: false,
        has_fanout: true,
        structural_signals: vec![
            "roots:1".to_string(),
            "max_parallel_width:3".to_string(),
            "max_depth:2".to_string(),
        ],
    };
    view.topology.rationale = TopologyRationale {
        dependency_shape: "independent branches".to_string(),
        operational_signals: vec![
            "budget pressure".to_string(),
            "conservative mode".to_string(),
        ],
        selected_for: "maximize safe concurrency".to_string(),
        fallback_reason: Some("budget pressure capped the active team".to_string()),
    };
    view.team_sizing = Some(TeamSizingDecision {
        workflow_id,
        graph_id,
        decision_phase: "frontier_rebalance".to_string(),
        desired_workers: 3,
        selected_workers: 1,
        available_workers: 3,
        branch_frontier_width: 3,
        dependency_depth: 2,
        conservative_mode: true,
        budget_pressure: Some(88),
        cap_reason: Some("budget pressure capped the active team".to_string()),
        rationale_lines: vec![
            "parallel fanout exposed three ready branches".to_string(),
            "budget pressure forced the active team to stay sequential".to_string(),
        ],
        decided_at: chrono::Utc::now(),
    });

    let rendered = autonomy::render_status(&view);

    assert!(rendered.contains("shape=parallel-fanout"));
    assert!(rendered.contains("structure=roots:1 | max_parallel_width:3 | max_depth:2"));
    assert!(rendered.contains("dependency=independent branches"));
    assert!(rendered.contains("signals=budget pressure | conservative mode"));
    assert!(rendered.contains("team sizing: phase=frontier_rebalance desired=3 selected=1"));
    assert!(rendered.contains("cap=budget pressure capped the active team"));
    assert!(rendered.contains("parallel fanout exposed three ready branches"));
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
        operation.name == "mistersmith_autonomy_topology_info"
            && operation
                .labels
                .iter()
                .any(|(key, value)| key == "task_shape" && value == "strict-chain")
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
