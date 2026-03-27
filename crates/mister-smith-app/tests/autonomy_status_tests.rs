#![allow(dead_code)]

#[path = "../src/autonomy.rs"]
mod autonomy;
#[path = "../src/observability.rs"]
mod observability;

use mister_smith_core::{
    AgentId, BranchRecoveryStrategy, BranchState, BudgetPolicy, BudgetScope, CheckpointId,
    CoordinationPolicy, ExecutionBranchId, ExecutionGraphId, FailureClass,
    FailureContextCheckpoint, GraphState, GuardDecision, GuardDecisionId, GuardEvidence,
    HandoffClarificationRequest, HealthState, InterventionRecord, InterventionRecordId,
    InterventionType, MemorySnapshotId, OrchestrationQualityView, ProfileSnapshotId, ProfileTarget,
    ProofOutcomeClassification, ProvenanceChain, ProvenanceLink, RepairDirective,
    RepairDirectiveAction, RevocationState, StepEvaluationRecord, TaskId, TaskShapeClassification,
    TaskShapeKind, TeamSizingDecision, TopologyKind, TopologyRationale, VerifierVerdict,
};
use mister_smith_events::{
    AutonomyEvent, AutonomyEventEnvelope, AutonomyStatusView, BranchSummary, CapabilitySummary,
    CheckpointRecordSummary, ContextPressureSummary, DelegationAlert, ExecutionGraphSummary,
    ExternalCapabilityDecisionOutcome, ExternalCapabilityDecisionSummary,
    ExternalCapabilityDecisionSurface, ResumeProvenanceSummary, RoutingDecisionSummary,
    StepRoutingDecisionSummary, TopologyPlanSummary,
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
        result_preview: None,
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
            descriptor_id: None,
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
                    descriptor_id: None,
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
        external_capability_decisions: vec![ExternalCapabilityDecisionSummary {
            boundary_surface: Some(ExternalCapabilityDecisionSurface::ToolBus),
            branch_id: Some(branch_id),
            capability_id: Some(capability_id),
            capability_descriptor_id: Some("tool:agent.echo".to_string()),
            action_descriptor_id: Some("tool:agent.echo".to_string()),
            action_id: Some("tool:agent.echo#execute".to_string()),
            action_title: Some("execute agent.echo".to_string()),
            scope: Some(mister_smith_core::DelegationScope::InvokeTool),
            required_scope: Some(mister_smith_core::DelegationScope::InvokeTool),
            policy_action: Some("execute".to_string()),
            policy_resource: Some("echo".to_string()),
            policy_scope: Some("agent".to_string()),
            policy_resource_id: Some("agent.echo".to_string()),
            revocation_state: Some(RevocationState::Active),
            chain_depth: 1,
            outcome: ExternalCapabilityDecisionOutcome::Allowed,
            observed_at: None,
            rationale: vec![
                "descriptor 'tool:agent.echo' matched the requested external action".to_string(),
                "required scope InvokeTool matched capability scope InvokeTool".to_string(),
            ],
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

fn sample_accepted_task_ingress_metadata(
    capability_id: mister_smith_core::CapabilityId,
) -> serde_json::Value {
    serde_json::json!({
        "accepted_task_ingress": {
            "request_surface": "POST /api/v1/tasks",
            "source_metadata_key": "external_delegation",
            "capability_id": capability_id,
            "capability_descriptor_id": "tool:agent.echo",
            "action_descriptor_id": "tool:agent.echo",
            "action_id": "tool:agent.echo#execute",
            "action_title": "execute agent.echo",
            "scope": "InvokeTool",
            "required_scope": "InvokeTool",
            "policy_action": "execute",
            "policy_resource": "echo",
            "policy_scope": "agent",
            "policy_resource_id": "agent.echo",
            "revocation_state": "Active",
            "chain_depth": 1
        }
    })
}

fn sample_step_evaluation_record(
    workflow_id: TaskId,
    step_id: &str,
    verdict: VerifierVerdict,
    repair_action: Option<RepairDirectiveAction>,
    clarification_attempt_count: u32,
    checkpoint_ref: Option<&str>,
    last_stable_step_id: Option<&str>,
    failure_context_ref: Option<&str>,
) -> StepEvaluationRecord {
    let repair_directive = if verdict == VerifierVerdict::Rejected {
        repair_action.map(|action| RepairDirective {
            action,
            issued_by: "verifier.runtime".to_string(),
            failure_context_ref: failure_context_ref
                .unwrap_or("draft-outline/failure")
                .to_string(),
            retry_budget_remaining: 1,
        })
    } else {
        None
    };

    let clarification_request = if clarification_attempt_count > 0 {
        Some(HandoffClarificationRequest {
            source_step_id: step_id.to_string(),
            target_step_id: "write-brief".to_string(),
            missing_constraints: vec!["budget ceiling".to_string()],
            attempt_count: clarification_attempt_count,
            expires_at: Some(chrono::Utc::now()),
        })
    } else {
        None
    };

    let failure_context_checkpoint = if failure_context_ref.is_some()
        || checkpoint_ref.is_some()
        || last_stable_step_id.is_some()
    {
        Some(FailureContextCheckpoint {
            failed_step_id: step_id.to_string(),
            last_stable_step_id: last_stable_step_id.map(ToOwned::to_owned),
            checkpoint_ref: checkpoint_ref.map(ToOwned::to_owned),
            failure_context_ref: failure_context_ref
                .unwrap_or("draft-outline/failure")
                .to_string(),
            failure_code: Some("missing_constraint".to_string()),
            reason: "missing context".to_string(),
            attempt_count: clarification_attempt_count.max(1),
        })
    } else {
        None
    };

    StepEvaluationRecord {
        workflow_id,
        step_id: step_id.to_string(),
        verdict,
        confidence: Some(0.91),
        reason: match verdict {
            VerifierVerdict::Accepted => "accepted after bounded repair".to_string(),
            VerifierVerdict::Rejected => "missing context".to_string(),
        },
        failure_code: if verdict == VerifierVerdict::Rejected {
            Some("missing_constraint".to_string())
        } else {
            None
        },
        checkpoint_ref: checkpoint_ref.map(ToOwned::to_owned),
        repair_directive,
        clarification_request,
        failure_context_checkpoint,
    }
}

fn sample_step_result_with_evaluation(
    step_id: &str,
    step_evaluation: StepEvaluationRecord,
) -> serde_json::Value {
    serde_json::json!({
        "task_id": TaskId::new(),
        "step_id": step_id,
        "result": {
            "summary": "bounded answer preview"
        },
        "step_evaluation": step_evaluation
    })
}

fn sample_step_result_with_task_payload(
    step_id: &str,
    action: &str,
    description: &str,
    dependencies: &[&str],
) -> serde_json::Value {
    serde_json::json!({
        "task_id": TaskId::new(),
        "result": {
            "task": {
                "step_id": step_id,
                "action": action,
                "description": description,
                "dependencies": dependencies,
            }
        }
    })
}

fn sample_task_result_payload_with_details(
    workflow_id: TaskId,
    status: &str,
    execution_steps: Vec<serde_json::Value>,
    step_results: Vec<serde_json::Value>,
    proof_outcome: Option<ProofOutcomeClassification>,
) -> serde_json::Value {
    let proof_outcome = proof_outcome.map(ProofOutcomeClassification::as_str);

    serde_json::json!({
        "workflow_id": workflow_id,
        "status": status,
        "proof_outcome": proof_outcome,
        "result": {
            "workflow_id": workflow_id,
            "provider_kind": "openai_chatgpt",
            "model_id": "gpt-5.4",
            "description": "freeze the result contract",
            "runtime_execution_mode": {
                "execution_boundary": "tool_bus",
                "workflow_runner": "tokio_task",
                "routing_policy": "round_robin",
                "registered_provider_count": 1,
                "budget_root": "disabled"
            },
            "planner_output": {
                "steps": 1
            },
            "execution_plan": {
                "steps": execution_steps
            },
            "step_results": step_results,
            "aggregated_result": {
                "summary": "bounded answer preview"
            },
            "proof_outcome": proof_outcome
        }
    })
}

fn sample_task_result_view(
    workflow_id: TaskId,
    proof_outcome: ProofOutcomeClassification,
) -> serde_json::Value {
    sample_task_result_payload(workflow_id, "completed", 1, 1, Some(proof_outcome))
}

fn sample_task_result_payload(
    workflow_id: TaskId,
    status: &str,
    execution_step_count: usize,
    step_result_count: usize,
    proof_outcome: Option<ProofOutcomeClassification>,
) -> serde_json::Value {
    let execution_steps = (1..=execution_step_count)
        .map(|index| serde_json::json!({ "id": format!("step-{index}") }))
        .collect::<Vec<_>>();
    let step_results = (0..step_result_count)
        .map(|_| {
            serde_json::json!({
                "task_id": TaskId::new(),
                "result": {
                    "summary": "bounded answer preview"
                }
            })
        })
        .collect::<Vec<_>>();
    sample_task_result_payload_with_details(
        workflow_id,
        status,
        execution_steps,
        step_results,
        proof_outcome,
    )
}

fn sample_canonical_result(
    workflow_id: TaskId,
    status: &str,
    execution_steps: Vec<serde_json::Value>,
    step_results: Vec<serde_json::Value>,
) -> mister_smith_core::UnifiedResultEnvelope {
    autonomy::build_canonical_result_envelope(autonomy::CanonicalResultEnvelopeInput {
        workflow_id,
        provider_kind: "openai_chatgpt",
        model_id: "gpt-5.4",
        description: "freeze the result contract",
        runtime_execution_mode: serde_json::json!({
            "execution_boundary": "tool_bus",
            "workflow_runner": "tokio_task",
            "routing_policy": "round_robin",
            "registered_provider_count": 1,
            "budget_root": "disabled"
        }),
        planner_output: serde_json::json!({ "steps": execution_steps.len() }),
        execution_plan: serde_json::json!({ "steps": execution_steps }),
        step_results,
        aggregated_result: serde_json::json!({
            "summary": "bounded answer preview"
        }),
        status,
    })
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
    assert!(rendered.contains("external capability decisions:"));
    assert!(rendered.contains("surface=tool_bus"));
    assert!(rendered.contains("outcome=allowed"));
    assert!(rendered.contains(&format!("branch={branch_id}")));
    assert!(rendered.contains("tool:agent.echo#execute"));
    assert!(rendered.contains("required scope InvokeTool matched capability scope InvokeTool"));
    assert!(rendered.contains("control-plane state unavailable"));
}

#[test]
fn enrich_accepted_task_ingress_continuity_surfaces_task_ingress_decision() {
    let (mut view, _, _) = sample_view();
    let capability_id = view
        .external_capability_decisions
        .first()
        .and_then(|decision| decision.capability_id)
        .expect("sample view should include one capability decision");
    view.external_capability_decisions.clear();

    autonomy::enrich_accepted_task_ingress_continuity(
        &mut view,
        &sample_accepted_task_ingress_metadata(capability_id),
    );

    let summary = view
        .external_capability_decisions
        .first()
        .expect("accepted task ingress should project one boundary decision");
    assert_eq!(
        summary.boundary_surface,
        Some(ExternalCapabilityDecisionSurface::TaskIngress)
    );
    assert_eq!(summary.outcome, ExternalCapabilityDecisionOutcome::Allowed);
    assert_eq!(summary.branch_id, None);
    assert_eq!(
        summary.action_id.as_deref(),
        Some("tool:agent.echo#execute")
    );
    assert!(summary
        .rationale
        .iter()
        .any(|line| line.contains("POST /api/v1/tasks")));
    assert!(summary
        .rationale
        .iter()
        .any(|line| { line.contains("accepted_task_ingress sourced from external_delegation") }));

    let rendered = autonomy::render_status(&view);
    assert!(rendered.contains("surface=task_ingress"));
    assert!(rendered.contains("branch=none"));
}

#[test]
fn enrich_result_preview_prefers_task_result_projection() {
    let (mut view, _, _) = sample_view();
    view.graph.state = GraphState::Completed;
    view.step_routing_history[0].triggered_checkpoints =
        vec!["budget_policy".to_string(), "confidence_review".to_string()];
    let task_result = sample_task_result_view(
        view.graph.workflow_id,
        ProofOutcomeClassification::CollapsedToSequential,
    );
    let metadata = serde_json::json!({
        "final_result": task_result["result"].clone(),
    });

    autonomy::enrich_result_preview(&mut view, &metadata, Some(&task_result));

    let preview = view
        .result_preview
        .expect("task result should produce an operator preview");
    assert_eq!(preview.payload_location, "task.result");
    assert_eq!(
        preview.proof_outcome,
        ProofOutcomeClassification::CollapsedToSequential
    );
    assert_eq!(
        preview.preview_text.as_deref(),
        Some("bounded answer preview")
    );
    assert!(preview
        .provenance_lines
        .iter()
        .any(|line| line.contains("planner emitted one sequential step")));
    assert!(preview
        .provenance_lines
        .iter()
        .any(|line| line.contains("provider=openai_chatgpt model=gpt-5.4")));
    assert!(preview.provenance_lines.iter().any(|line| {
        line.contains("runtime execution mode boundary=tool_bus runner=tokio_task")
            && line.contains("routing_policy=round_robin")
            && line.contains("registered_providers=1")
            && line.contains("budget_root=disabled")
    }));
    assert!(preview
        .provenance_lines
        .iter()
        .any(|line| { line.contains("graph state Completed with topology Sequential") }));
    assert!(preview
        .provenance_lines
        .iter()
        .any(|line| line.contains("routing history retained 1 decision(s)")));
    assert!(preview.provenance_lines.iter().any(|line| {
        line.contains("latest step routing tier=llm-tier action=continue")
            && line.contains("budget_policy")
            && line.contains("confidence_review")
    }));
    assert!(preview
        .provenance_lines
        .iter()
        .any(|line| line
            .contains("session assistant_result derives from the canonical result object")));
}

#[test]
fn enrich_result_preview_falls_back_to_metadata_final_result() {
    let (mut view, _, _) = sample_view();
    view.graph.state = GraphState::Completed;
    let task_result = sample_task_result_view(
        view.graph.workflow_id,
        ProofOutcomeClassification::GraphFormedAndCompleted,
    );
    let metadata = serde_json::json!({
        "final_result": task_result["result"].clone(),
    });

    autonomy::enrich_result_preview(&mut view, &metadata, None);

    let preview = view
        .result_preview
        .expect("metadata.final_result should still produce an operator preview");
    assert_eq!(preview.payload_location, "metadata.final_result");
    assert_eq!(
        preview.proof_outcome,
        ProofOutcomeClassification::GraphFormedAndCompleted
    );
    assert_eq!(
        preview.preview_text.as_deref(),
        Some("bounded answer preview")
    );
}

#[test]
fn enrich_result_preview_prefers_stored_proof_outcome_over_structural_inference() {
    let (mut view, _, _) = sample_view();
    view.graph.state = GraphState::Completed;
    view.topology.topology_kind = TopologyKind::Hybrid;
    view.topology.parallelism_width = 2;
    view.topology.task_shape.kind = TaskShapeKind::FanoutJoin;
    view.topology.task_shape.max_parallel_width = 2;
    view.graph.branch_count = 2;
    view.graph.node_count = 2;

    let task_result = sample_task_result_payload(
        view.graph.workflow_id,
        "completed",
        2,
        2,
        Some(ProofOutcomeClassification::CollapsedToSequential),
    );
    let metadata = serde_json::json!({
        "final_result": task_result["result"].clone(),
    });

    autonomy::enrich_result_preview(&mut view, &metadata, Some(&task_result));

    let preview = view
        .result_preview
        .expect("stored task result should produce an operator preview");
    assert_eq!(
        preview.proof_outcome,
        ProofOutcomeClassification::CollapsedToSequential
    );
    assert!(preview
        .provenance_lines
        .iter()
        .any(|line| line.contains("planner emitted one sequential step")));
}

#[test]
fn classify_proof_outcome_keeps_failed_runs_in_the_single_failure_class() {
    let execution_plan = serde_json::json!({
        "steps": [
            {
                "id": "step-1"
            },
            {
                "id": "step-2"
            }
        ]
    });
    let step_results = vec![serde_json::json!({
        "task_id": TaskId::new(),
        "result": {
            "summary": "partial branch output"
        }
    })];

    assert_eq!(
        autonomy::classify_proof_outcome(
            "failed",
            Some(&execution_plan),
            Some(step_results.as_slice()),
        ),
        ProofOutcomeClassification::FailedBeforeGraph
    );
}

#[test]
fn classify_proof_outcome_covers_success_collapse_and_failure_visible_matrix() {
    let success_plan = serde_json::json!({
        "steps": [
            { "id": "step-1" },
            { "id": "step-2" }
        ]
    });
    let success_steps = vec![
        serde_json::json!({
            "task_id": TaskId::new(),
            "result": { "summary": "parallel branch alpha" }
        }),
        serde_json::json!({
            "task_id": TaskId::new(),
            "result": { "summary": "parallel branch beta" }
        }),
    ];
    let collapse_plan = serde_json::json!({
        "steps": [
            { "id": "step-1" }
        ]
    });
    let collapse_steps = vec![serde_json::json!({
        "task_id": TaskId::new(),
        "result": { "summary": "single sequential branch" }
    })];
    let failure_plan = serde_json::json!({
        "steps": [
            { "id": "step-1" },
            { "id": "step-2" }
        ]
    });
    let failure_steps = vec![serde_json::json!({
        "task_id": TaskId::new(),
        "result": { "summary": "partial branch output" }
    })];

    let cases = [
        (
            "success",
            "completed",
            &success_plan,
            success_steps.as_slice(),
            ProofOutcomeClassification::GraphFormedAndCompleted,
        ),
        (
            "collapse",
            "completed",
            &collapse_plan,
            collapse_steps.as_slice(),
            ProofOutcomeClassification::CollapsedToSequential,
        ),
        (
            "failure_visible",
            "failed",
            &failure_plan,
            failure_steps.as_slice(),
            ProofOutcomeClassification::FailedBeforeGraph,
        ),
    ];

    for (label, status, execution_plan, step_results, expected) in cases {
        assert_eq!(
            autonomy::classify_proof_outcome(status, Some(execution_plan), Some(step_results)),
            expected,
            "unexpected proof outcome for {label}"
        );
    }
}

#[test]
fn build_task_result_view_surfaces_accepted_without_repair_orchestration_quality() {
    let workflow_id = TaskId::new();
    let canonical_result = sample_canonical_result(
        workflow_id,
        "completed",
        vec![serde_json::json!({ "id": "draft-outline" })],
        vec![sample_step_result_with_evaluation(
            "draft-outline",
            sample_step_evaluation_record(
                workflow_id,
                "draft-outline",
                VerifierVerdict::Accepted,
                None,
                0,
                None,
                None,
                None,
            ),
        )],
    );

    let summary = autonomy::build_task_result_view("completed", canonical_result)
        .orchestration_quality
        .expect("accepted evaluation should surface orchestration quality");

    assert_eq!(
        summary,
        OrchestrationQualityView {
            step_id: "draft-outline".to_string(),
            verdict: VerifierVerdict::Accepted,
            repair_action: None,
            clarification_attempt_count: 0,
            checkpoint_ref: None,
            last_stable_step_id: None,
            failure_context_ref: None,
            outcome_summary: "accepted_without_repair".to_string(),
        }
    );
}

#[test]
fn build_task_result_view_surfaces_rejected_retry_orchestration_quality() {
    let workflow_id = TaskId::new();
    let canonical_result = sample_canonical_result(
        workflow_id,
        "failed",
        vec![serde_json::json!({ "id": "draft-outline" })],
        vec![sample_step_result_with_evaluation(
            "draft-outline",
            sample_step_evaluation_record(
                workflow_id,
                "draft-outline",
                VerifierVerdict::Rejected,
                Some(RepairDirectiveAction::RetryStep),
                0,
                Some("checkpoint-retry"),
                Some("collect-evidence"),
                Some("draft-outline/retry"),
            ),
        )],
    );

    let summary = autonomy::build_task_result_view("failed", canonical_result)
        .orchestration_quality
        .expect("rejected evaluation should surface orchestration quality");

    assert_eq!(
        summary,
        OrchestrationQualityView {
            step_id: "draft-outline".to_string(),
            verdict: VerifierVerdict::Rejected,
            repair_action: Some(RepairDirectiveAction::RetryStep),
            clarification_attempt_count: 0,
            checkpoint_ref: Some("checkpoint-retry".to_string()),
            last_stable_step_id: Some("collect-evidence".to_string()),
            failure_context_ref: Some("draft-outline/retry".to_string()),
            outcome_summary: "rejected_with_retry_step".to_string(),
        }
    );
}

#[test]
fn build_task_result_view_infers_planner_repair_orchestration_quality_without_verifier_policy() {
    let workflow_id = TaskId::new();
    let canonical_result = sample_canonical_result(
        workflow_id,
        "completed",
        vec![
            serde_json::json!({ "id": "s1" }),
            serde_json::json!({ "id": "s2" }),
            serde_json::json!({ "id": "s3" }),
        ],
        vec![
            sample_step_result_with_task_payload(
                "s1",
                "inspect_live_runtime",
                "Capture directly observed runtime evidence.",
                &[],
            ),
            sample_step_result_with_task_payload(
                "s2",
                "resolve_missing_context",
                "Request clarification or perform bounded local repair instead of guessing.",
                &["s1"],
            ),
            sample_step_result_with_task_payload(
                "s3",
                "deliver_evidence_grounded_answer",
                "Return the final answer grounded in observed evidence.",
                &["s2"],
            ),
        ],
    );

    let summary = autonomy::build_task_result_view("completed", canonical_result)
        .orchestration_quality
        .expect("planner repair step should surface orchestration quality");

    assert_eq!(
        summary,
        OrchestrationQualityView {
            step_id: "s2".to_string(),
            verdict: VerifierVerdict::Accepted,
            repair_action: Some(RepairDirectiveAction::ClarifyHandoff),
            clarification_attempt_count: 1,
            checkpoint_ref: Some("planner-step:s1".to_string()),
            last_stable_step_id: Some("s1".to_string()),
            failure_context_ref: Some("planner:s2/clarify_handoff".to_string()),
            outcome_summary: "accepted_after_clarify_handoff".to_string(),
        }
    );
}

#[test]
fn enrich_result_preview_surfaces_accepted_after_clarify_orchestration_quality() {
    let (mut view, _, _) = sample_view();
    view.graph.state = GraphState::Completed;
    view.step_routing_history[0].triggered_checkpoints =
        vec!["budget_policy".to_string(), "confidence_review".to_string()];
    let workflow_id = view.graph.workflow_id;
    let task_result = sample_task_result_payload_with_details(
        workflow_id,
        "completed",
        vec![serde_json::json!({ "id": "draft-outline" })],
        vec![
            sample_step_result_with_evaluation(
                "draft-outline",
                sample_step_evaluation_record(
                    workflow_id,
                    "draft-outline",
                    VerifierVerdict::Accepted,
                    None,
                    2,
                    Some("checkpoint-clarify"),
                    Some("collect-evidence"),
                    Some("draft-outline/clarify"),
                ),
            ),
            sample_step_result_with_evaluation(
                "draft-outline",
                sample_step_evaluation_record(
                    workflow_id,
                    "draft-outline",
                    VerifierVerdict::Rejected,
                    Some(RepairDirectiveAction::ClarifyHandoff),
                    1,
                    Some("checkpoint-clarify"),
                    Some("collect-evidence"),
                    Some("draft-outline/clarify"),
                ),
            ),
        ],
        Some(ProofOutcomeClassification::CollapsedToSequential),
    );

    autonomy::enrich_result_preview(&mut view, &serde_json::json!({}), Some(&task_result));

    let preview = view
        .result_preview
        .clone()
        .expect("accepted clarify path should produce a preview");
    assert_eq!(
        preview.orchestration_quality,
        Some(OrchestrationQualityView {
            step_id: "draft-outline".to_string(),
            verdict: VerifierVerdict::Accepted,
            repair_action: Some(RepairDirectiveAction::ClarifyHandoff),
            clarification_attempt_count: 2,
            checkpoint_ref: Some("checkpoint-clarify".to_string()),
            last_stable_step_id: Some("collect-evidence".to_string()),
            failure_context_ref: Some("draft-outline/clarify".to_string()),
            outcome_summary: "accepted_after_clarify_handoff".to_string(),
        })
    );
    assert!(preview.provenance_lines.iter().any(|line| {
        line.contains("latest step routing tier=llm-tier action=continue")
            && line.contains("budget_policy")
            && line.contains("confidence_review")
    }));
    assert!(preview.provenance_lines.iter().any(|line| {
        line.contains("orchestration quality step=draft-outline verdict=accepted")
            && line.contains("repair=clarify_handoff")
            && line.contains("clarification_attempts=2")
            && line.contains("checkpoint=checkpoint-clarify")
            && line.contains("last_stable_step=collect-evidence")
            && line.contains("failure_context=draft-outline/clarify")
            && line.contains("outcome=accepted_after_clarify_handoff")
    }));

    let rendered = autonomy::render_status(&view);
    assert!(rendered.contains("orchestration_quality:"));
    assert!(rendered.contains("step=draft-outline verdict=accepted repair=clarify_handoff"));
    assert!(rendered.contains(
        "clarification_attempts=2 checkpoint=checkpoint-clarify last_stable_step=collect-evidence"
    ));
    assert!(rendered
        .contains("failure_context=draft-outline/clarify outcome=accepted_after_clarify_handoff"));
}

#[test]
fn enrich_result_preview_marks_planner_repair_inference_when_verifier_surface_is_absent() {
    let (mut view, _, _) = sample_view();
    view.graph.state = GraphState::Completed;
    let workflow_id = view.graph.workflow_id;
    let task_result = sample_task_result_payload_with_details(
        workflow_id,
        "completed",
        vec![
            serde_json::json!({ "id": "s1" }),
            serde_json::json!({ "id": "s2" }),
            serde_json::json!({ "id": "s3" }),
        ],
        vec![
            sample_step_result_with_task_payload(
                "s1",
                "inspect_live_runtime",
                "Capture directly observed runtime evidence.",
                &[],
            ),
            sample_step_result_with_task_payload(
                "s2",
                "resolve_missing_context",
                "Request clarification or perform bounded local repair instead of guessing.",
                &["s1"],
            ),
            sample_step_result_with_task_payload(
                "s3",
                "deliver_evidence_grounded_answer",
                "Return the final answer grounded in observed evidence.",
                &["s2"],
            ),
        ],
        Some(ProofOutcomeClassification::GraphFormedAndCompleted),
    );

    autonomy::enrich_result_preview(&mut view, &serde_json::json!({}), Some(&task_result));

    let preview = view
        .result_preview
        .expect("planner repair inference should still produce an operator preview");
    let orchestration_quality = preview
        .orchestration_quality
        .expect("planner repair inference should surface orchestration quality");
    assert_eq!(orchestration_quality.step_id, "s2");
    assert_eq!(
        orchestration_quality.repair_action,
        Some(RepairDirectiveAction::ClarifyHandoff)
    );
    assert!(preview.provenance_lines.iter().any(|line| {
        line.contains("inferred from planner repair step 's2' without verifier_policy")
    }));
}

#[test]
fn enrich_result_preview_recovers_replan_checkpoint_lineage() {
    let (mut view, _, _) = sample_view();
    view.graph.state = GraphState::Completed;
    let workflow_id = view.graph.workflow_id;
    let task_result = sample_task_result_payload_with_details(
        workflow_id,
        "completed",
        vec![
            serde_json::json!({ "id": "collect-evidence" }),
            serde_json::json!({ "id": "draft-outline" }),
        ],
        vec![
            sample_step_result_with_evaluation(
                "draft-outline",
                sample_step_evaluation_record(
                    workflow_id,
                    "draft-outline",
                    VerifierVerdict::Rejected,
                    Some(RepairDirectiveAction::ReplanFromCheckpoint),
                    1,
                    Some("checkpoint-replan"),
                    Some("collect-evidence"),
                    Some("draft-outline/replan"),
                ),
            ),
            sample_step_result_with_evaluation(
                "draft-outline",
                sample_step_evaluation_record(
                    workflow_id,
                    "draft-outline",
                    VerifierVerdict::Accepted,
                    None,
                    2,
                    Some("checkpoint-replan"),
                    Some("collect-evidence"),
                    Some("draft-outline/replan"),
                ),
            ),
        ],
        Some(ProofOutcomeClassification::GraphFormedAndCompleted),
    );

    autonomy::enrich_result_preview(&mut view, &serde_json::json!({}), Some(&task_result));

    let summary = view
        .result_preview
        .expect("accepted replan path should produce a preview")
        .orchestration_quality
        .expect("accepted replan path should retain orchestration quality");
    assert_eq!(
        summary,
        OrchestrationQualityView {
            step_id: "draft-outline".to_string(),
            verdict: VerifierVerdict::Accepted,
            repair_action: Some(RepairDirectiveAction::ReplanFromCheckpoint),
            clarification_attempt_count: 2,
            checkpoint_ref: Some("checkpoint-replan".to_string()),
            last_stable_step_id: Some("collect-evidence".to_string()),
            failure_context_ref: Some("draft-outline/replan".to_string()),
            outcome_summary: "accepted_after_replan_from_checkpoint".to_string(),
        }
    );
}

#[test]
fn enrich_result_preview_uses_execution_plan_order_over_step_result_position() {
    let (mut view, _, _) = sample_view();
    view.graph.state = GraphState::Completed;
    let workflow_id = view.graph.workflow_id;
    let task_result = sample_task_result_payload_with_details(
        workflow_id,
        "completed",
        vec![
            serde_json::json!({ "id": "collect-evidence" }),
            serde_json::json!({ "id": "draft-outline" }),
            serde_json::json!({ "id": "publish" }),
        ],
        vec![
            sample_step_result_with_evaluation(
                "publish",
                sample_step_evaluation_record(
                    workflow_id,
                    "publish",
                    VerifierVerdict::Accepted,
                    None,
                    0,
                    None,
                    None,
                    None,
                ),
            ),
            sample_step_result_with_evaluation(
                "draft-outline",
                sample_step_evaluation_record(
                    workflow_id,
                    "draft-outline",
                    VerifierVerdict::Accepted,
                    None,
                    3,
                    Some("checkpoint-clarify"),
                    Some("collect-evidence"),
                    Some("draft-outline/clarify"),
                ),
            ),
        ],
        Some(ProofOutcomeClassification::GraphFormedAndCompleted),
    );

    autonomy::enrich_result_preview(&mut view, &serde_json::json!({}), Some(&task_result));

    let summary = view
        .result_preview
        .expect("preview should be retained")
        .orchestration_quality
        .expect("terminal evaluation should be projected");
    assert_eq!(summary.step_id, "publish");
    assert_eq!(summary.verdict, VerifierVerdict::Accepted);
    assert_eq!(summary.repair_action, None);
    assert_eq!(summary.outcome_summary, "accepted_without_repair");
}

#[test]
fn render_status_surfaces_result_preview_block() {
    let (mut view, _, _) = sample_view();
    view.result_preview = Some(mister_smith_core::OperatorResultPreview {
        workflow_id: view.graph.workflow_id,
        proof_outcome: ProofOutcomeClassification::GraphFormedAndCompleted,
        preview_text: Some("bounded answer preview".to_string()),
        payload_location: "task.result".to_string(),
        orchestration_quality: None,
        provenance_lines: vec![
            "graph formed and completed before final result publication".to_string(),
            "provider=openai_chatgpt model=gpt-5.4".to_string(),
            "canonical result stored in metadata.final_result".to_string(),
            "session assistant_result derives from the canonical result object".to_string(),
        ],
    });

    let rendered = autonomy::render_status(&view);

    assert!(rendered.contains("result preview:"));
    assert!(rendered.contains("proof=graph_formed_and_completed"));
    assert!(rendered.contains("location=task.result"));
    assert!(rendered.contains("preview=bounded answer preview"));
    assert!(rendered.contains("provenance:"));
    assert!(rendered.contains("  - graph formed and completed before final result publication"));
    assert!(rendered.contains("  - provider=openai_chatgpt model=gpt-5.4"));
    assert!(rendered.contains("canonical result stored in metadata.final_result"));
    assert!(rendered.contains("session assistant_result derives from the canonical result object"));
}

#[test]
fn enrich_result_preview_merges_existing_structural_provenance() {
    let (mut view, _, _) = sample_view();
    view.graph.state = GraphState::Completed;
    view.result_preview = Some(mister_smith_core::OperatorResultPreview {
        workflow_id: view.graph.workflow_id,
        proof_outcome: ProofOutcomeClassification::GraphFormedAndCompleted,
        preview_text: Some("structural preview only".to_string()),
        payload_location: "task.result".to_string(),
        orchestration_quality: None,
        provenance_lines: vec![
            "projection observed graph state Completed with topology Sequential (1 branch(es), 3 node(s))".to_string(),
            "routing history retained 1 decision(s)".to_string(),
        ],
    });
    let task_result = sample_task_result_view(
        view.graph.workflow_id,
        ProofOutcomeClassification::GraphFormedAndCompleted,
    );
    let metadata = serde_json::json!({
        "final_result": task_result["result"].clone(),
    });

    autonomy::enrich_result_preview(&mut view, &metadata, Some(&task_result));

    let preview = view
        .result_preview
        .expect("canonical preview should remain present");
    assert_eq!(
        preview.preview_text.as_deref(),
        Some("bounded answer preview")
    );
    assert!(preview
        .provenance_lines
        .iter()
        .any(|line| line.contains("provider=openai_chatgpt model=gpt-5.4")));
    assert!(preview.provenance_lines.iter().any(|line| {
        line.contains("projection observed graph state Completed with topology Sequential")
    }));
}

#[test]
fn enrich_and_render_result_preview_cover_success_collapse_and_failure_visible_matrix() {
    let cases = [
        (
            "success",
            GraphState::Completed,
            TopologyKind::Hybrid,
            2usize,
            TaskShapeKind::FanoutJoin,
            2usize,
            "completed",
            4usize,
            4usize,
            ProofOutcomeClassification::GraphFormedAndCompleted,
            "graph formed and completed before final result publication",
            "topology: Hybrid width=2 shape=fanout-join",
        ),
        (
            "collapse",
            GraphState::Completed,
            TopologyKind::Sequential,
            1usize,
            TaskShapeKind::ParallelFanout,
            3usize,
            "completed",
            1usize,
            1usize,
            ProofOutcomeClassification::CollapsedToSequential,
            "planner emitted one sequential step",
            "topology: Sequential width=1 shape=parallel-fanout",
        ),
        (
            "failure_visible",
            GraphState::Failed,
            TopologyKind::Hybrid,
            2usize,
            TaskShapeKind::FanoutJoin,
            2usize,
            "failed",
            2usize,
            1usize,
            ProofOutcomeClassification::FailedBeforeGraph,
            "workflow failed before usable graph formation",
            "topology: Hybrid width=2 shape=fanout-join",
        ),
    ];

    for (
        label,
        graph_state,
        topology_kind,
        parallelism_width,
        task_shape_kind,
        max_parallel_width,
        status,
        execution_step_count,
        step_result_count,
        expected_outcome,
        expected_provenance,
        expected_topology,
    ) in cases
    {
        let (mut view, _, _) = sample_view();
        view.graph.state = graph_state;
        view.graph.branch_count = max_parallel_width;
        view.graph.node_count = execution_step_count;
        view.graph.active_topology = Some(topology_kind);
        view.topology.topology_kind = topology_kind;
        view.topology.parallelism_width = parallelism_width;
        view.topology.task_shape.kind = task_shape_kind;
        view.topology.task_shape.max_parallel_width = max_parallel_width;
        view.topology.task_shape.has_fanout = max_parallel_width > 1;
        view.topology.task_shape.has_join = matches!(task_shape_kind, TaskShapeKind::FanoutJoin);
        view.topology.task_shape.structural_signals = vec![
            "roots:1".to_string(),
            format!("max_parallel_width:{max_parallel_width}"),
            "max_depth:2".to_string(),
        ];

        let task_result = sample_task_result_payload(
            view.graph.workflow_id,
            status,
            execution_step_count,
            step_result_count,
            None,
        );

        autonomy::enrich_result_preview(&mut view, &serde_json::json!({}), Some(&task_result));

        let preview = view
            .result_preview
            .clone()
            .expect("proof matrix case should infer a result preview");
        let rendered = autonomy::render_status(&view);

        assert_eq!(
            preview.proof_outcome, expected_outcome,
            "unexpected preview proof outcome for {label}"
        );
        assert!(
            rendered.contains(&format!("proof={}", expected_outcome.as_str())),
            "missing proof outcome in rendered status for {label}"
        );
        assert!(
            rendered.contains(expected_provenance),
            "missing provenance in rendered status for {label}"
        );
        assert!(
            rendered.contains(expected_topology),
            "missing topology summary in rendered status for {label}"
        );
    }
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
fn synthesize_failed_before_graph_status_uses_frontier_width_for_single_root_fanout() {
    let workflow_id = TaskId::new();
    let canonical_result =
        autonomy::build_canonical_result_envelope(autonomy::CanonicalResultEnvelopeInput {
            workflow_id,
            provider_kind: "openai_chatgpt",
            model_id: "gpt-5.4",
            description: "single-root fanout failure",
            runtime_execution_mode: serde_json::json!({}),
            planner_output: serde_json::json!({
                "goal": "single root fanout",
            }),
            execution_plan: serde_json::json!({
                "steps": [
                    {"id": "root", "depends_on": []},
                    {"id": "branch-a", "depends_on": ["root"]},
                    {"id": "branch-b", "depends_on": ["root"]},
                ]
            }),
            step_results: vec![],
            aggregated_result: serde_json::json!({
                "error": "planner execution failed: Ask operation timed out",
            }),
            status: "failed",
        });
    let metadata = serde_json::json!({
        "final_result": serde_json::to_value(canonical_result).expect("final result should serialize")
    });

    let view = autonomy::synthesize_failed_before_graph_status(workflow_id, &metadata, None)
        .expect("single-root fanout should synthesize a bounded autonomy status");

    assert_eq!(view.graph.branch_count, 1);
    assert_eq!(view.graph.node_count, 3);
    assert_eq!(view.topology.parallelism_width, 2);
    assert_eq!(view.topology.topology_kind, TopologyKind::Parallel);
    assert_eq!(view.topology.task_shape.kind, TaskShapeKind::ParallelFanout);
    assert_eq!(view.topology.task_shape.max_parallel_width, 2);
}

#[test]
fn synthesize_failed_before_graph_status_preserves_hybrid_fanout_join_width() {
    let workflow_id = TaskId::new();
    let canonical_result = autonomy::build_canonical_result_envelope(
        autonomy::CanonicalResultEnvelopeInput {
            workflow_id,
            provider_kind: "openai_chatgpt",
            model_id: "gpt-5.4",
            description: "fanout join failure",
            runtime_execution_mode: serde_json::json!({}),
            planner_output: serde_json::json!({
                "goal": "fanout join",
            }),
            execution_plan: serde_json::json!({
                "steps": [
                    {"id": "root", "depends_on": []},
                    {"id": "branch-a", "depends_on": ["root"]},
                    {"id": "branch-b", "depends_on": ["root"]},
                    {"id": "join", "depends_on": ["branch-a", "branch-b"]},
                ]
            }),
            step_results: vec![],
            aggregated_result: serde_json::json!({
                "error": "execution graph compile failed: Unsupported topology contract: unsupported planner role 'joiner'",
            }),
            status: "failed",
        },
    );
    let metadata = serde_json::json!({
        "final_result": serde_json::to_value(canonical_result).expect("final result should serialize")
    });

    let view = autonomy::synthesize_failed_before_graph_status(workflow_id, &metadata, None)
        .expect("fanout-join should synthesize a bounded autonomy status");

    assert_eq!(view.graph.branch_count, 1);
    assert_eq!(view.graph.node_count, 4);
    assert_eq!(view.topology.parallelism_width, 2);
    assert_eq!(view.topology.topology_kind, TopologyKind::Hybrid);
    assert_eq!(view.topology.task_shape.kind, TaskShapeKind::FanoutJoin);
    assert_eq!(view.topology.task_shape.max_parallel_width, 2);
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
