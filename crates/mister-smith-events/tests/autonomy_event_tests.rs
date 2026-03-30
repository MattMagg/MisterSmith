use mister_smith_core::{
    AgentId, AuthorityPrincipal, BranchRecoveryStrategy, BranchState, BudgetPolicy, BudgetScope,
    CapabilityId, CheckpointId, ContextBudgetId, CoordinationPolicy, DelegationScope,
    ExecutionBranchId, ExecutionGraphId, ExecutionNodeId, FailureClass, GraphState, GuardDecision,
    GuardDecisionId, GuardEvidence, HealthState, InterventionRecord, InterventionRecordId,
    InterventionType, OperatorResultPreview, OrchestrationQualityView, ProfileFingerprintId,
    ProfileFingerprintRef, ProfileSnapshot, ProfileSnapshotId, ProfileTarget,
    ProofOutcomeClassification, ProvenanceChain, ProvenanceLink, RepairDirectiveAction,
    RepairLineageRef, ResultProvenanceSummary, RevocationState, SupervisionDecisionBasis,
    SupervisionEvidenceView, SupervisionTargetKind, SupervisionTargetScope, TaskId,
    TaskShapeClassification, TaskShapeKind, TeamSizingDecision, TopologyKind, TopologyRationale,
    VerifierVerdict,
};
use mister_smith_events::autonomy::{
    infer_proof_outcome_from_projection, merge_operator_result_preview,
};
use mister_smith_events::{
    AutonomyEvent, AutonomyEventEnvelope, AutonomyEventType, AutonomyStatusView, BranchSummary,
    CapabilitySummary, CheckpointRecordSummary, ContextPressureSummary, DelegationAlert, EventBus,
    EventType, ExecutionGraphSummary, ExternalCapabilityDecisionOutcome,
    ExternalCapabilityDecisionSummary, ExternalCapabilityDecisionSurface, ResumeProvenanceSummary,
    RoutingDecisionSummary, StepRoutingDecisionSummary, TopologyPlanSummary,
};
use serde::de::DeserializeOwned;

fn assert_event_traits<T>()
where
    T: Clone + Send + Sync + std::fmt::Debug + serde::Serialize + DeserializeOwned + 'static,
{
}

fn sample_provenance(
    issuer: AuthorityPrincipal,
    recipient: AgentId,
    capability_id: CapabilityId,
    scope: DelegationScope,
    expires_at: chrono::DateTime<chrono::Utc>,
) -> ProvenanceChain {
    ProvenanceChain {
        root_issuer: issuer.clone(),
        links: vec![ProvenanceLink {
            issuer,
            recipient,
            capability_id,
            scope,
            expires_at,
            descriptor_id: None,
        }],
        terminal_capability: capability_id,
    }
}

fn sample_task_shape(kind: TaskShapeKind) -> TaskShapeClassification {
    let (max_parallel_width, max_depth, has_join, has_fanout) = match kind {
        TaskShapeKind::StrictChain => (1, 2, false, false),
        TaskShapeKind::ParallelFanout => (2, 1, false, true),
        TaskShapeKind::FanoutJoin => (2, 2, true, true),
        TaskShapeKind::HierarchicalFanout => (3, 3, false, true),
        TaskShapeKind::MixedGraph => (2, 2, true, true),
    };
    TaskShapeClassification {
        kind,
        root_count: 1,
        max_parallel_width,
        max_depth,
        has_join,
        has_fanout,
        structural_signals: vec![
            "roots:1".to_string(),
            format!("max_parallel_width:{max_parallel_width}"),
            format!("max_depth:{max_depth}"),
        ],
    }
}

fn sample_team_sizing(
    workflow_id: TaskId,
    graph_id: ExecutionGraphId,
    kind: TaskShapeKind,
) -> TeamSizingDecision {
    let task_shape = sample_task_shape(kind);
    let desired_workers = task_shape.max_parallel_width.max(1);
    TeamSizingDecision {
        workflow_id,
        graph_id,
        decision_phase: "initial".to_string(),
        desired_workers,
        selected_workers: desired_workers,
        available_workers: desired_workers,
        branch_frontier_width: task_shape.max_parallel_width.max(1),
        dependency_depth: task_shape.max_depth,
        conservative_mode: false,
        budget_pressure: Some(35),
        cap_reason: None,
        rationale_lines: vec![
            format!(
                "task shape {} with frontier width {}",
                task_shape.kind.as_str(),
                task_shape.max_parallel_width.max(1)
            ),
            format!(
                "selected {} workers from the available pool",
                desired_workers
            ),
        ],
        decided_at: chrono::Utc::now(),
    }
}

fn sample_external_capability_decision(
    capability_id: CapabilityId,
    scope: DelegationScope,
) -> ExternalCapabilityDecisionSummary {
    ExternalCapabilityDecisionSummary {
        boundary_surface: Some(ExternalCapabilityDecisionSurface::ToolBus),
        branch_id: None,
        capability_id: Some(capability_id),
        capability_descriptor_id: Some("tool:agent.echo".to_string()),
        action_descriptor_id: Some("tool:agent.echo".to_string()),
        action_id: Some("tool:agent.echo#execute".to_string()),
        action_title: Some("execute agent.echo".to_string()),
        scope: Some(scope),
        required_scope: Some(scope),
        policy_action: Some("execute".to_string()),
        policy_resource: Some("echo".to_string()),
        policy_scope: Some("agent".to_string()),
        policy_resource_id: Some("agent.echo".to_string()),
        revocation_state: Some(RevocationState::Active),
        chain_depth: 1,
        outcome: ExternalCapabilityDecisionOutcome::Allowed,
        observed_at: Some(chrono::Utc::now()),
        rationale: vec![
            "descriptor 'tool:agent.echo' matched the requested external action".to_string(),
            "required scope InvokeTool matched capability scope InvokeTool".to_string(),
        ],
    }
}

fn sample_task_ingress_decision(
    capability_id: CapabilityId,
    scope: DelegationScope,
) -> ExternalCapabilityDecisionSummary {
    ExternalCapabilityDecisionSummary {
        boundary_surface: Some(ExternalCapabilityDecisionSurface::TaskIngress),
        branch_id: None,
        capability_id: Some(capability_id),
        capability_descriptor_id: Some("tool:agent.echo".to_string()),
        action_descriptor_id: Some("tool:agent.echo".to_string()),
        action_id: Some("tool:agent.echo#execute".to_string()),
        action_title: Some("execute agent.echo".to_string()),
        scope: Some(scope),
        required_scope: Some(scope),
        policy_action: Some("execute".to_string()),
        policy_resource: Some("echo".to_string()),
        policy_scope: Some("agent".to_string()),
        policy_resource_id: Some("agent.echo".to_string()),
        revocation_state: Some(RevocationState::Active),
        chain_depth: 1,
        outcome: ExternalCapabilityDecisionOutcome::Allowed,
        observed_at: None,
        rationale: vec![
            "accepted delegated task ingress remained authorized at POST /api/v1/tasks".to_string(),
            "continuity projected from workflow metadata accepted_task_ingress sourced from external_delegation".to_string(),
        ],
    }
}

fn sample_result_preview(
    workflow_id: TaskId,
    proof_outcome: ProofOutcomeClassification,
) -> OperatorResultPreview {
    OperatorResultPreview {
        workflow_id,
        proof_outcome,
        preview_text: Some("bounded answer preview".to_string()),
        payload_location: "task.result".to_string(),
        orchestration_quality: None,
        provenance_lines: vec![
            "canonical result stored in metadata.final_result".to_string(),
            "aggregated payload nested under metadata.aggregated_result".to_string(),
            "session assistant_result derives from the canonical result object".to_string(),
        ],
    }
}

#[test]
fn merge_operator_result_preview_preserves_orchestration_quality_from_fallback() {
    let workflow_id = TaskId::new();
    let preferred = OperatorResultPreview {
        workflow_id,
        proof_outcome: ProofOutcomeClassification::GraphFormedAndCompleted,
        preview_text: Some("preferred preview".to_string()),
        payload_location: "task.result".to_string(),
        orchestration_quality: None,
        provenance_lines: vec!["preferred provenance".to_string()],
    };
    let fallback = OperatorResultPreview {
        workflow_id,
        proof_outcome: ProofOutcomeClassification::GraphFormedAndCompleted,
        preview_text: Some("fallback preview".to_string()),
        payload_location: "task.result".to_string(),
        orchestration_quality: Some(OrchestrationQualityView {
            step_id: "draft-outline".to_string(),
            verdict: VerifierVerdict::Accepted,
            repair_action: Some(RepairDirectiveAction::ClarifyHandoff),
            clarification_attempt_count: 2,
            checkpoint_ref: Some("checkpoint-clarify".to_string()),
            last_stable_step_id: Some("collect-evidence".to_string()),
            failure_context_ref: Some("draft-outline/clarify".to_string()),
            outcome_summary: "accepted_after_clarify_handoff".to_string(),
        }),
        provenance_lines: vec!["fallback provenance".to_string()],
    };

    let merged = merge_operator_result_preview(&preferred, &fallback);

    assert_eq!(
        merged.orchestration_quality,
        fallback.orchestration_quality.clone()
    );
    assert!(merged
        .provenance_lines
        .contains(&"preferred provenance".to_string()));
    assert!(merged
        .provenance_lines
        .contains(&"fallback provenance".to_string()));
}

fn sample_graph_summary(
    workflow_id: TaskId,
    graph_id: ExecutionGraphId,
    state: GraphState,
    branch_count: usize,
    node_count: usize,
    active_topology: Option<TopologyKind>,
) -> ExecutionGraphSummary {
    ExecutionGraphSummary {
        graph_id,
        workflow_id,
        state,
        branch_count,
        node_count,
        active_topology,
    }
}

fn sample_topology_summary(
    graph_id: ExecutionGraphId,
    topology_kind: TopologyKind,
    parallelism_width: usize,
    task_shape_kind: TaskShapeKind,
) -> TopologyPlanSummary {
    TopologyPlanSummary {
        graph_id,
        topology_kind,
        parallelism_width,
        task_shape: sample_task_shape(task_shape_kind),
        coordination_policy: CoordinationPolicy::Mixed,
        rationale: TopologyRationale {
            dependency_shape: "proof matrix".to_string(),
            operational_signals: vec!["packet-015".to_string()],
            selected_for: "freeze proof outcome matrix".to_string(),
            fallback_reason: None,
        },
        fallback_topology: Some(TopologyKind::Sequential),
    }
}

fn sample_branch_summary(graph_id: ExecutionGraphId) -> BranchSummary {
    BranchSummary {
        branch_id: ExecutionBranchId::new(),
        graph_id,
        state: BranchState::Running,
        assigned_agents: vec![AgentId::new()],
        checkpoint_id: None,
        recovery_strategy: BranchRecoveryStrategy::Resume,
    }
}

fn sample_routing_summary(
    workflow_id: TaskId,
    graph_id: ExecutionGraphId,
    branch_id: ExecutionBranchId,
) -> RoutingDecisionSummary {
    RoutingDecisionSummary {
        graph_id,
        branch_id,
        selected_agent: AgentId::new(),
        task_ids: vec![workflow_id],
        recovery_strategy: BranchRecoveryStrategy::Resume,
        checkpoint_id: None,
        dependency_depth: 2,
        budget_pressure: 35,
        health_state: HealthState::Healthy,
        profile_id: None,
        rationale: vec!["multi-branch routing remained visible".to_string()],
    }
}

async fn publish_projection(
    event_bus: &EventBus,
    graph: ExecutionGraphSummary,
    topology: TopologyPlanSummary,
    branches: Vec<BranchSummary>,
    routing_history: Vec<RoutingDecisionSummary>,
) {
    event_bus
        .publish(
            AutonomyEvent::GraphUpdated(AutonomyEventEnvelope {
                workflow_id: graph.workflow_id,
                graph_id: Some(graph.graph_id),
                branch_id: None,
                payload: graph.clone(),
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::TopologySelected(AutonomyEventEnvelope {
                workflow_id: graph.workflow_id,
                graph_id: Some(graph.graph_id),
                branch_id: None,
                payload: topology,
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();

    for branch in branches {
        event_bus
            .publish(
                AutonomyEvent::BranchUpdated(AutonomyEventEnvelope {
                    workflow_id: graph.workflow_id,
                    graph_id: Some(graph.graph_id),
                    branch_id: Some(branch.branch_id),
                    payload: branch,
                    operator_visible: true,
                })
                .into_event("autonomy-test"),
            )
            .await
            .unwrap();
    }

    for routing in routing_history {
        event_bus
            .publish(
                AutonomyEvent::RoutingDecisionRecorded(AutonomyEventEnvelope {
                    workflow_id: graph.workflow_id,
                    graph_id: Some(graph.graph_id),
                    branch_id: Some(routing.branch_id),
                    payload: routing,
                    operator_visible: true,
                })
                .into_event("autonomy-test"),
            )
            .await
            .unwrap();
    }
}

#[test]
fn autonomy_event_surfaces_compile_with_shared_trait_bounds() {
    assert_event_traits::<OperatorResultPreview>();
    assert_event_traits::<ExecutionGraphSummary>();
    assert_event_traits::<TopologyPlanSummary>();
    assert_event_traits::<BranchSummary>();
    assert_event_traits::<ContextPressureSummary>();
    assert_event_traits::<CapabilitySummary>();
    assert_event_traits::<DelegationAlert>();
    assert_event_traits::<StepRoutingDecisionSummary>();
    assert_event_traits::<AutonomyStatusView>();
    assert_event_traits::<AutonomyEventEnvelope<ExecutionGraphSummary>>();
    assert_event_traits::<AutonomyEvent>();
}

#[test]
fn autonomy_event_type_is_exposed_through_event_type() {
    assert_eq!(
        EventType::Autonomy(AutonomyEventType::TopologySelected).to_string(),
        "autonomy.TopologySelected"
    );
}

#[test]
fn autonomy_event_roundtrips_and_converts_to_generic_event() {
    let workflow_id = TaskId::new();
    let graph_id = ExecutionGraphId::new();
    let topology = TopologyPlanSummary {
        graph_id,
        topology_kind: TopologyKind::Parallel,
        parallelism_width: 3,
        task_shape: sample_task_shape(TaskShapeKind::ParallelFanout),
        coordination_policy: CoordinationPolicy::Barrier,
        rationale: TopologyRationale {
            dependency_shape: "independent branches".to_string(),
            operational_signals: vec!["healthy profile".to_string()],
            selected_for: "maximize safe concurrency".to_string(),
            fallback_reason: Some("degrade to sequential when budgets tighten".to_string()),
        },
        fallback_topology: Some(TopologyKind::Sequential),
    };
    let autonomy_event = AutonomyEvent::TopologySelected(AutonomyEventEnvelope {
        workflow_id,
        graph_id: Some(graph_id),
        branch_id: None,
        payload: topology.clone(),
        operator_visible: true,
    });

    let json = serde_json::to_string(&autonomy_event).unwrap();
    let roundtrip: AutonomyEvent = serde_json::from_str(&json).unwrap();
    let generic = autonomy_event.clone().into_event("autonomy-test");

    assert_eq!(roundtrip, autonomy_event);
    assert_eq!(
        generic.event_type,
        EventType::Autonomy(AutonomyEventType::TopologySelected)
    );
    assert!(generic.payload.is_object());
}

#[test]
fn proof_outcome_classification_freezes_the_three_packet_labels() {
    assert_eq!(
        ProofOutcomeClassification::ALL.map(ProofOutcomeClassification::as_str),
        [
            "graph_formed_and_completed",
            "collapsed_to_sequential",
            "failed_before_graph",
        ]
    );
}

#[test]
fn autonomy_status_view_serializes_with_typed_summaries() {
    let workflow_id = TaskId::new();
    let graph_id = ExecutionGraphId::new();
    let branch_id = ExecutionBranchId::new();
    let decision_id = GuardDecisionId::new();
    let fingerprint_ref = ProfileFingerprintRef {
        fingerprint_id: ProfileFingerprintId::new(),
        fingerprint_key: "executor:branch".to_string(),
        confidence: 0.81,
        expires_at: chrono::Utc::now(),
    };
    let view = AutonomyStatusView {
        session_id: None,
        turn_index: None,
        coordinator_agent_id: None,
        resume_provenance: Some(ResumeProvenanceSummary {
            recovered_after_restart: true,
            resumed_after_restart: true,
            recovered_at: Some(chrono::Utc::now()),
            recovery_reason: Some(
                "workflow interrupted by runtime restart before session sync".to_string(),
            ),
            resumed_from_workflow_id: Some(TaskId::new()),
            resumed_from_turn_index: Some(1),
        }),
        graph: ExecutionGraphSummary {
            graph_id,
            workflow_id,
            state: GraphState::Running,
            branch_count: 2,
            node_count: 5,
            active_topology: Some(TopologyKind::Hybrid),
        },
        topology: TopologyPlanSummary {
            graph_id,
            topology_kind: TopologyKind::Hybrid,
            parallelism_width: 2,
            task_shape: sample_task_shape(TaskShapeKind::FanoutJoin),
            coordination_policy: CoordinationPolicy::Mixed,
            rationale: TopologyRationale {
                dependency_shape: "mixed graph".to_string(),
                operational_signals: vec!["context pressure".to_string()],
                selected_for: "balance concurrency and recovery".to_string(),
                fallback_reason: None,
            },
            fallback_topology: Some(TopologyKind::Sequential),
        },
        team_sizing: Some(sample_team_sizing(
            workflow_id,
            graph_id,
            TaskShapeKind::FanoutJoin,
        )),
        branches: vec![BranchSummary {
            branch_id,
            graph_id,
            state: BranchState::Checkpointed,
            assigned_agents: vec![AgentId::new()],
            checkpoint_id: Some(CheckpointId::new()),
            recovery_strategy: BranchRecoveryStrategy::Resume,
        }],
        checkpoint_lineage: vec![CheckpointRecordSummary {
            graph_id,
            branch_id,
            checkpoint_id: CheckpointId::new(),
            captured_at: chrono::Utc::now(),
            memory_snapshot_id: mister_smith_core::MemorySnapshotId::new(),
            completed_nodes: vec![],
            pending_nodes: vec![],
            recovery_strategy: BranchRecoveryStrategy::Resume,
            failure_context: None,
        }],
        memory_pressure: vec![ContextPressureSummary {
            budget_id: ContextBudgetId::new(),
            branch_id: Some(branch_id),
            scope: BudgetScope::Branch,
            max_units: 4096,
            reserved_units: 3072,
            policy: BudgetPolicy::Summarize,
        }],
        routing_history: vec![RoutingDecisionSummary {
            graph_id,
            branch_id,
            selected_agent: AgentId::new(),
            task_ids: vec![TaskId::new()],
            recovery_strategy: BranchRecoveryStrategy::Resume,
            checkpoint_id: Some(CheckpointId::new()),
            dependency_depth: 2,
            budget_pressure: 75,
            health_state: HealthState::Degraded,
            profile_id: Some(ProfileSnapshotId::new()),
            rationale: vec!["checkpoint resume preserved completed siblings".to_string()],
        }],
        step_routing_history: vec![StepRoutingDecisionSummary {
            step_id: "critic.step.2".to_string(),
            step_index: Some(2),
            step_kind: Some("critic".to_string()),
            model_id: "gpt-5.4".to_string(),
            tier: "llm-tier".to_string(),
            reason: "accepted at llm-tier after provider fallback".to_string(),
            previous_step_id: Some("critic.step.1".to_string()),
            previous_action: Some("fallback".to_string()),
            previous_tier: Some("slm-tier".to_string()),
            action: "continue".to_string(),
            action_changed: true,
            preferred_tier_after: Some("llm-tier".to_string()),
            estimated_cost_tokens: Some(96),
            confidence_score: Some(0.88),
            triggered_checkpoints: vec![],
            change_rationale: vec!["action changed from fallback to continue".to_string()],
        }],
        result_preview: Some(sample_result_preview(
            workflow_id,
            ProofOutcomeClassification::GraphFormedAndCompleted,
        )),
        interventions: vec![mister_smith_core::InterventionRecord {
            record_id: InterventionRecordId::new(),
            decision_id: GuardDecisionId::new(),
            before_state: serde_json::json!({"state": "running"}),
            after_state: Some(serde_json::json!({"state": "isolated"})),
            rationale: "branch isolation".to_string(),
            emitted_at: chrono::Utc::now(),
        }],
        delegation_capabilities: vec![],
        delegation_alerts: vec![
            DelegationAlert {
                capability_id: Some(CapabilityId::new()),
                scope: Some(DelegationScope::ManageBranch),
                revocation_state: Some(RevocationState::Revoked),
                parent_capability: None,
                expires_at: None,
                chain_depth: 1,
                rejection_reason: Some("Delegation capability revoked".to_string()),
                message: "delegation revoked before branch resume".to_string(),
            },
            DelegationAlert {
                capability_id: None,
                scope: None,
                revocation_state: None,
                parent_capability: None,
                expires_at: None,
                chain_depth: 0,
                rejection_reason: Some(
                    "operator review required for widened authority".to_string(),
                ),
                message: "operator review required for widened authority".to_string(),
            },
        ],
        external_capability_decisions: vec![
            sample_external_capability_decision(CapabilityId::new(), DelegationScope::InvokeTool),
            sample_task_ingress_decision(CapabilityId::new(), DelegationScope::InvokeTool),
        ],
        profiles: vec![ProfileSnapshot {
            profile_id: ProfileSnapshotId::new(),
            target: ProfileTarget::Branch,
            health_state: HealthState::Degraded,
            latency_window: None,
            error_window: None,
            semantic_signals: vec![],
            fingerprint_ref: Some(fingerprint_ref.clone()),
            updated_at: chrono::Utc::now(),
        }],
        guard_decisions: vec![GuardDecision {
            decision_id,
            failure_class: FailureClass::Semantic,
            intervention: InterventionType::ContextRefresh,
            evidence: GuardEvidence {
                profile_id: None,
                decision_basis: SupervisionDecisionBasis::FingerprintReinforced,
                signal_descriptions: vec!["loop detected".to_string()],
                checkpoint_ids: vec![],
                notes: vec!["operator review available".to_string()],
            },
            target_scope: mister_smith_core::GuardTarget::Branch(branch_id),
            operator_visibility: true,
        }],
        supervision_evidence: Some(SupervisionEvidenceView {
            target_scope: SupervisionTargetScope {
                kind: SupervisionTargetKind::Branch,
                provider: None,
                graph_id: Some(graph_id),
                branch_id: Some(branch_id),
                node_id: None,
            },
            fingerprint_ref: Some(fingerprint_ref),
            profile_snapshot: None,
            guard_decision: None,
            intervention_record: None,
            decision_basis: Some(
                SupervisionDecisionBasis::FingerprintReinforced
                    .as_str()
                    .to_string(),
            ),
            repair_lineage_ref: Some(RepairLineageRef {
                source: "packet-020".to_string(),
                checkpoint_ref: Some("last-stable-checkpoint".to_string()),
            }),
            proof_boundary: Some("deterministic-only".to_string()),
        }),
        conservative_reasons: vec!["control-plane state unavailable".to_string()],
    };

    let json = serde_json::to_string(&view).unwrap();
    let roundtrip: AutonomyStatusView = serde_json::from_str(&json).unwrap();

    assert_eq!(roundtrip, view);
    assert!(
        roundtrip
            .resume_provenance
            .as_ref()
            .expect("resume provenance should round-trip")
            .resumed_after_restart
    );
}

#[tokio::test]
async fn event_bus_synthesizes_supervision_evidence_from_runtime_events() {
    let event_bus = EventBus::default();
    let workflow_id = TaskId::new();
    let graph_id = ExecutionGraphId::new();
    let branch_id = ExecutionBranchId::new();
    let other_branch_id = ExecutionBranchId::new();
    let profile_id = ProfileSnapshotId::new();
    let other_profile_id = ProfileSnapshotId::new();
    let decision_id = GuardDecisionId::new();
    let fingerprint_ref = ProfileFingerprintRef {
        fingerprint_id: ProfileFingerprintId::new(),
        fingerprint_key: "executor:branch".to_string(),
        confidence: 0.67,
        expires_at: chrono::Utc::now(),
    };

    event_bus
        .publish(
            AutonomyEvent::GraphUpdated(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: None,
                payload: ExecutionGraphSummary {
                    graph_id,
                    workflow_id,
                    state: GraphState::Running,
                    branch_count: 1,
                    node_count: 2,
                    active_topology: Some(TopologyKind::Sequential),
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::TopologySelected(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: None,
                payload: TopologyPlanSummary {
                    graph_id,
                    topology_kind: TopologyKind::Sequential,
                    parallelism_width: 1,
                    task_shape: sample_task_shape(TaskShapeKind::StrictChain),
                    coordination_policy: CoordinationPolicy::Barrier,
                    rationale: TopologyRationale {
                        dependency_shape: "single branch".to_string(),
                        operational_signals: vec!["predictive supervision".to_string()],
                        selected_for: "bounded recovery".to_string(),
                        fallback_reason: None,
                    },
                    fallback_topology: Some(TopologyKind::Sequential),
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::BranchUpdated(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(branch_id),
                payload: BranchSummary {
                    branch_id,
                    graph_id,
                    state: BranchState::Running,
                    assigned_agents: vec![AgentId::new()],
                    checkpoint_id: None,
                    recovery_strategy: BranchRecoveryStrategy::Resume,
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::ProfileSnapshotRecorded(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(branch_id),
                payload: ProfileSnapshot {
                    profile_id,
                    target: ProfileTarget::Branch,
                    health_state: HealthState::Degraded,
                    latency_window: None,
                    error_window: None,
                    semantic_signals: vec![],
                    fingerprint_ref: Some(fingerprint_ref.clone()),
                    updated_at: chrono::Utc::now(),
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::GuardDecisionEvaluated(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(branch_id),
                payload: GuardDecision {
                    decision_id,
                    failure_class: FailureClass::Semantic,
                    intervention: InterventionType::ContextRefresh,
                    evidence: GuardEvidence {
                        profile_id: Some(profile_id),
                        decision_basis: SupervisionDecisionBasis::FingerprintReinforced,
                        signal_descriptions: vec!["loop detected".to_string()],
                        checkpoint_ids: vec![],
                        notes: vec!["fingerprint reinforced local recovery".to_string()],
                    },
                    target_scope: mister_smith_core::GuardTarget::Branch(branch_id),
                    operator_visibility: true,
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::InterventionRecorded(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(branch_id),
                payload: mister_smith_core::InterventionRecord {
                    record_id: InterventionRecordId::new(),
                    decision_id,
                    before_state: serde_json::json!({"state": "running"}),
                    after_state: Some(serde_json::json!({"state": "refreshed"})),
                    rationale: "context refresh".to_string(),
                    emitted_at: chrono::Utc::now(),
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::BranchUpdated(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(other_branch_id),
                payload: BranchSummary {
                    branch_id: other_branch_id,
                    graph_id,
                    state: BranchState::Running,
                    assigned_agents: vec![AgentId::new()],
                    checkpoint_id: None,
                    recovery_strategy: BranchRecoveryStrategy::Resume,
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::ProfileSnapshotRecorded(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(other_branch_id),
                payload: ProfileSnapshot {
                    profile_id: other_profile_id,
                    target: ProfileTarget::Branch,
                    health_state: HealthState::Healthy,
                    latency_window: None,
                    error_window: None,
                    semantic_signals: vec![],
                    fingerprint_ref: Some(ProfileFingerprintRef {
                        fingerprint_id: ProfileFingerprintId::new(),
                        fingerprint_key: "executor:other-branch".to_string(),
                        confidence: 0.31,
                        expires_at: chrono::Utc::now(),
                    }),
                    updated_at: chrono::Utc::now(),
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();

    let status = event_bus
        .autonomy_status(&workflow_id)
        .await
        .expect("status should be available");
    let supervision_evidence = status
        .supervision_evidence
        .expect("supervision evidence should be synthesized");

    assert_eq!(
        supervision_evidence.target_scope.kind,
        SupervisionTargetKind::Branch
    );
    assert_eq!(supervision_evidence.target_scope.graph_id, Some(graph_id));
    assert_eq!(supervision_evidence.target_scope.branch_id, Some(branch_id));
    assert_eq!(
        supervision_evidence
            .profile_snapshot
            .as_ref()
            .map(|profile| profile.profile_id),
        Some(profile_id)
    );
    assert_eq!(
        supervision_evidence
            .fingerprint_ref
            .as_ref()
            .map(|reference| reference.fingerprint_key.as_str()),
        Some("executor:branch")
    );
    assert_eq!(
        supervision_evidence.decision_basis.as_deref(),
        Some(SupervisionDecisionBasis::FingerprintReinforced.as_str())
    );
    assert!(supervision_evidence.repair_lineage_ref.is_none());
    assert_eq!(
        supervision_evidence.proof_boundary.as_deref(),
        Some("supported task path")
    );
}

#[tokio::test]
async fn event_bus_synthesizes_node_scoped_lineage_from_guard_branch_hint() {
    let event_bus = EventBus::default();
    let workflow_id = TaskId::new();
    let graph_id = ExecutionGraphId::new();
    let branch_id = ExecutionBranchId::new();
    let node_id = ExecutionNodeId::new();
    let checkpoint_id = CheckpointId::new();
    let decision_id = GuardDecisionId::new();

    event_bus
        .publish(
            AutonomyEvent::GraphUpdated(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: None,
                payload: ExecutionGraphSummary {
                    graph_id,
                    workflow_id,
                    state: GraphState::Running,
                    branch_count: 1,
                    node_count: 1,
                    active_topology: Some(TopologyKind::Sequential),
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::TopologySelected(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: None,
                payload: TopologyPlanSummary {
                    graph_id,
                    topology_kind: TopologyKind::Sequential,
                    parallelism_width: 1,
                    task_shape: sample_task_shape(TaskShapeKind::StrictChain),
                    coordination_policy: CoordinationPolicy::Barrier,
                    rationale: TopologyRationale {
                        dependency_shape: "single node branch".to_string(),
                        operational_signals: vec!["node-scoped recovery".to_string()],
                        selected_for: "bounded checkpoint replay".to_string(),
                        fallback_reason: None,
                    },
                    fallback_topology: Some(TopologyKind::Sequential),
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::BranchUpdated(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(branch_id),
                payload: BranchSummary {
                    branch_id,
                    graph_id,
                    state: BranchState::Running,
                    assigned_agents: vec![AgentId::new()],
                    checkpoint_id: None,
                    recovery_strategy: BranchRecoveryStrategy::Resume,
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::CheckpointRecorded(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(branch_id),
                payload: CheckpointRecordSummary {
                    graph_id,
                    branch_id,
                    checkpoint_id,
                    captured_at: chrono::Utc::now(),
                    memory_snapshot_id: mister_smith_core::MemorySnapshotId::new(),
                    completed_nodes: vec![],
                    pending_nodes: vec![],
                    recovery_strategy: BranchRecoveryStrategy::Resume,
                    failure_context: Some(serde_json::json!({"reason": "node retry"})),
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::GuardDecisionEvaluated(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(branch_id),
                payload: GuardDecision {
                    decision_id,
                    failure_class: FailureClass::Structural,
                    intervention: InterventionType::Retry,
                    evidence: GuardEvidence {
                        profile_id: None,
                        decision_basis: SupervisionDecisionBasis::LiveSignalsOnly,
                        signal_descriptions: vec!["node retry checkpoint".to_string()],
                        checkpoint_ids: vec![],
                        notes: vec!["node-scoped runtime supervision".to_string()],
                    },
                    target_scope: mister_smith_core::GuardTarget::Node(node_id),
                    operator_visibility: true,
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();

    let status = event_bus
        .autonomy_status(&workflow_id)
        .await
        .expect("status should be available");
    let supervision_evidence = status
        .supervision_evidence
        .expect("supervision evidence should be synthesized");
    let expected_checkpoint_ref = checkpoint_id.to_string();

    assert_eq!(
        supervision_evidence.target_scope.kind,
        SupervisionTargetKind::Node
    );
    assert_eq!(supervision_evidence.target_scope.graph_id, Some(graph_id));
    assert_eq!(supervision_evidence.target_scope.branch_id, Some(branch_id));
    assert_eq!(supervision_evidence.target_scope.node_id, Some(node_id));
    assert_eq!(
        supervision_evidence
            .repair_lineage_ref
            .as_ref()
            .and_then(|reference| reference.checkpoint_ref.as_deref()),
        Some(expected_checkpoint_ref.as_str())
    );
    assert_eq!(
        supervision_evidence.proof_boundary.as_deref(),
        Some("supported task path")
    );
}

#[test]
fn autonomy_status_updated_event_roundtrips_with_boxed_payload() {
    let workflow_id = TaskId::new();
    let graph_id = ExecutionGraphId::new();
    let branch_id = ExecutionBranchId::new();
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
                fallback_reason: Some("stay sequential until supervision stabilizes".to_string()),
            },
            fallback_topology: Some(TopologyKind::Sequential),
        },
        team_sizing: Some(sample_team_sizing(
            workflow_id,
            graph_id,
            TaskShapeKind::StrictChain,
        )),
        branches: vec![BranchSummary {
            branch_id,
            graph_id,
            state: BranchState::Isolated,
            assigned_agents: vec![AgentId::new()],
            checkpoint_id: Some(CheckpointId::new()),
            recovery_strategy: BranchRecoveryStrategy::Resume,
        }],
        checkpoint_lineage: vec![CheckpointRecordSummary {
            graph_id,
            branch_id,
            checkpoint_id: CheckpointId::new(),
            captured_at: chrono::Utc::now(),
            memory_snapshot_id: mister_smith_core::MemorySnapshotId::new(),
            completed_nodes: vec![],
            pending_nodes: vec![],
            recovery_strategy: BranchRecoveryStrategy::Resume,
            failure_context: Some(serde_json::json!({"reason": "stalled stream"})),
        }],
        memory_pressure: vec![],
        routing_history: vec![RoutingDecisionSummary {
            graph_id,
            branch_id,
            selected_agent: AgentId::new(),
            task_ids: vec![TaskId::new()],
            recovery_strategy: BranchRecoveryStrategy::Resume,
            checkpoint_id: Some(CheckpointId::new()),
            dependency_depth: 1,
            budget_pressure: 20,
            health_state: HealthState::Degraded,
            profile_id: None,
            rationale: vec!["sequential routing avoided restart blast radius".to_string()],
        }],
        step_routing_history: vec![],
        result_preview: Some(sample_result_preview(
            workflow_id,
            ProofOutcomeClassification::CollapsedToSequential,
        )),
        interventions: vec![],
        delegation_capabilities: vec![],
        delegation_alerts: vec![],
        external_capability_decisions: vec![],
        profiles: vec![],
        guard_decisions: vec![],
        supervision_evidence: None,
        conservative_reasons: vec!["control-plane freshness unavailable".to_string()],
    };
    let event = AutonomyEvent::StatusUpdated(Box::new(AutonomyEventEnvelope {
        workflow_id,
        graph_id: Some(graph_id),
        branch_id: Some(branch_id),
        payload: view,
        operator_visible: true,
    }));

    let json = serde_json::to_string(&event).unwrap();
    let roundtrip: AutonomyEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(roundtrip, event);
}

#[tokio::test]
async fn event_bus_preserves_supervision_evidence_from_status_updated() {
    let event_bus = EventBus::default();
    let workflow_id = TaskId::new();
    let graph_id = ExecutionGraphId::new();
    let branch_id = ExecutionBranchId::new();
    let profile_id = ProfileSnapshotId::new();
    let decision_id = GuardDecisionId::new();
    let record_id = InterventionRecordId::new();
    let fingerprint_ref = ProfileFingerprintRef {
        fingerprint_id: ProfileFingerprintId::new(),
        fingerprint_key: "executor:branch".to_string(),
        confidence: 0.67,
        expires_at: chrono::Utc::now(),
    };
    let supervision_evidence = SupervisionEvidenceView {
        target_scope: SupervisionTargetScope {
            kind: SupervisionTargetKind::Branch,
            provider: None,
            graph_id: Some(graph_id),
            branch_id: Some(branch_id),
            node_id: None,
        },
        fingerprint_ref: Some(fingerprint_ref.clone()),
        profile_snapshot: Some(ProfileSnapshot {
            profile_id,
            target: ProfileTarget::Branch,
            health_state: HealthState::Degraded,
            latency_window: None,
            error_window: None,
            semantic_signals: vec![],
            fingerprint_ref: Some(fingerprint_ref.clone()),
            updated_at: chrono::Utc::now(),
        }),
        guard_decision: Some(GuardDecision {
            decision_id,
            failure_class: FailureClass::Semantic,
            intervention: InterventionType::ContextRefresh,
            evidence: GuardEvidence {
                profile_id: Some(profile_id),
                decision_basis: SupervisionDecisionBasis::FingerprintReinforced,
                signal_descriptions: vec!["loop detected".to_string()],
                checkpoint_ids: vec![],
                notes: vec!["fingerprint reinforced local recovery".to_string()],
            },
            target_scope: mister_smith_core::GuardTarget::Branch(branch_id),
            operator_visibility: true,
        }),
        intervention_record: Some(InterventionRecord {
            record_id,
            decision_id,
            before_state: serde_json::json!({"state": "running"}),
            after_state: Some(serde_json::json!({"state": "refreshed"})),
            rationale: "context refresh".to_string(),
            emitted_at: chrono::Utc::now(),
        }),
        decision_basis: Some(
            SupervisionDecisionBasis::FingerprintReinforced
                .as_str()
                .to_string(),
        ),
        repair_lineage_ref: Some(RepairLineageRef {
            source: "packet-020".to_string(),
            checkpoint_ref: Some("last-stable-checkpoint".to_string()),
        }),
        proof_boundary: Some("deterministic-only".to_string()),
    };
    let view = AutonomyStatusView {
        session_id: None,
        turn_index: None,
        coordinator_agent_id: None,
        resume_provenance: None,
        graph: sample_graph_summary(
            workflow_id,
            graph_id,
            GraphState::Running,
            1,
            3,
            Some(TopologyKind::Sequential),
        ),
        topology: sample_topology_summary(
            graph_id,
            TopologyKind::Sequential,
            1,
            TaskShapeKind::StrictChain,
        ),
        team_sizing: None,
        branches: vec![BranchSummary {
            branch_id,
            graph_id,
            state: BranchState::Running,
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
        external_capability_decisions: vec![],
        profiles: vec![],
        guard_decisions: vec![],
        supervision_evidence: Some(supervision_evidence.clone()),
        conservative_reasons: vec![],
    };

    event_bus
        .publish(
            AutonomyEvent::StatusUpdated(Box::new(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(branch_id),
                payload: view,
                operator_visible: true,
            }))
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();

    let status = event_bus
        .autonomy_status(&workflow_id)
        .await
        .expect("status should be available");

    assert_eq!(status.supervision_evidence, Some(supervision_evidence));
}

#[test]
fn capability_summary_preserves_policy_issuers() {
    let capability_id = CapabilityId::new();
    let recipient = AgentId::new();
    let expires_at = chrono::Utc::now();
    let issuer = AuthorityPrincipal::Policy("bootstrap-policy".to_string());
    let summary = CapabilitySummary {
        capability_id,
        descriptor_id: None,
        issuer: issuer.clone(),
        recipient,
        scope: DelegationScope::ApplyIntervention,
        parent_capability: None,
        expires_at,
        provenance: sample_provenance(
            issuer,
            recipient,
            capability_id,
            DelegationScope::ApplyIntervention,
            expires_at,
        ),
        revocation_state: RevocationState::Active,
        rejection_reason: None,
    };

    let json = serde_json::to_string(&summary).unwrap();
    let roundtrip: CapabilitySummary = serde_json::from_str(&json).unwrap();

    assert_eq!(roundtrip, summary);
}

#[tokio::test]
async fn event_bus_assembles_operator_visible_autonomy_projection() {
    let workflow_id = TaskId::new();
    let graph_id = ExecutionGraphId::new();
    let branch_id = ExecutionBranchId::new();
    let checkpoint_id = CheckpointId::new();
    let decision_id = GuardDecisionId::new();
    let event_bus = EventBus::default();

    event_bus
        .publish(
            AutonomyEvent::GraphUpdated(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: None,
                payload: ExecutionGraphSummary {
                    graph_id,
                    workflow_id,
                    state: GraphState::Running,
                    branch_count: 1,
                    node_count: 3,
                    active_topology: Some(TopologyKind::Sequential),
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::TopologySelected(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: None,
                payload: TopologyPlanSummary {
                    graph_id,
                    topology_kind: TopologyKind::Sequential,
                    parallelism_width: 1,
                    task_shape: sample_task_shape(TaskShapeKind::StrictChain),
                    coordination_policy: CoordinationPolicy::Barrier,
                    rationale: TopologyRationale {
                        dependency_shape: "single branch".to_string(),
                        operational_signals: vec!["stalled stream".to_string()],
                        selected_for: "minimize restart blast radius".to_string(),
                        fallback_reason: Some("conservative fallback to sequential".to_string()),
                    },
                    fallback_topology: Some(TopologyKind::Sequential),
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::BranchUpdated(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(branch_id),
                payload: BranchSummary {
                    branch_id,
                    graph_id,
                    state: BranchState::Checkpointed,
                    assigned_agents: vec![AgentId::new()],
                    checkpoint_id: Some(checkpoint_id),
                    recovery_strategy: BranchRecoveryStrategy::Resume,
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::CheckpointRecorded(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(branch_id),
                payload: CheckpointRecordSummary {
                    graph_id,
                    branch_id,
                    checkpoint_id,
                    captured_at: chrono::Utc::now(),
                    memory_snapshot_id: mister_smith_core::MemorySnapshotId::new(),
                    completed_nodes: vec![],
                    pending_nodes: vec![],
                    recovery_strategy: BranchRecoveryStrategy::Resume,
                    failure_context: Some(serde_json::json!({"reason": "provider retry"})),
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::ContextPressureObserved(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(branch_id),
                payload: ContextPressureSummary {
                    budget_id: ContextBudgetId::new(),
                    branch_id: Some(branch_id),
                    scope: BudgetScope::Branch,
                    max_units: 4096,
                    reserved_units: 3500,
                    policy: BudgetPolicy::Summarize,
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::RoutingDecisionRecorded(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(branch_id),
                payload: RoutingDecisionSummary {
                    graph_id,
                    branch_id,
                    selected_agent: AgentId::new(),
                    task_ids: vec![TaskId::new()],
                    recovery_strategy: BranchRecoveryStrategy::Resume,
                    checkpoint_id: Some(checkpoint_id),
                    dependency_depth: 1,
                    budget_pressure: 88,
                    health_state: HealthState::Degraded,
                    profile_id: None,
                    rationale: vec!["checkpoint scope narrowed resume".to_string()],
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::GuardDecisionEvaluated(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(branch_id),
                payload: GuardDecision {
                    decision_id,
                    failure_class: FailureClass::Structural,
                    intervention: InterventionType::Escalation,
                    evidence: GuardEvidence {
                        profile_id: None,
                        decision_basis: SupervisionDecisionBasis::ConservativeFallback,
                        signal_descriptions: vec!["missing profile".to_string()],
                        checkpoint_ids: vec![checkpoint_id],
                        notes: vec![
                            "conservative fallback: control-plane state unavailable".to_string()
                        ],
                    },
                    target_scope: mister_smith_core::GuardTarget::Branch(branch_id),
                    operator_visibility: true,
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::InterventionRecorded(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(branch_id),
                payload: mister_smith_core::InterventionRecord {
                    record_id: InterventionRecordId::new(),
                    decision_id,
                    before_state: serde_json::json!({"state": "running"}),
                    after_state: Some(serde_json::json!({"state": "escalated"})),
                    rationale: "operator escalation remained visible".to_string(),
                    emitted_at: chrono::Utc::now(),
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    let capability_id = CapabilityId::new();
    let mut first_decision =
        sample_external_capability_decision(capability_id, DelegationScope::InvokeTool);
    first_decision.branch_id = Some(branch_id);
    event_bus
        .publish(
            AutonomyEvent::DelegationDecisionRecorded(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(branch_id),
                payload: first_decision,
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    let retry_branch_id = ExecutionBranchId::new();
    let mut retry_decision =
        sample_external_capability_decision(capability_id, DelegationScope::InvokeTool);
    retry_decision.branch_id = Some(retry_branch_id);
    retry_decision.outcome = ExternalCapabilityDecisionOutcome::Rejected;
    retry_decision.observed_at = Some(chrono::Utc::now() + chrono::Duration::seconds(1));
    retry_decision.rationale.insert(
        0,
        "delegation descriptor 'tool:agent.echo' was revoked before retry".to_string(),
    );
    event_bus
        .publish(
            AutonomyEvent::DelegationDecisionRecorded(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(retry_branch_id),
                payload: retry_decision,
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();

    let view = event_bus
        .autonomy_status(&workflow_id)
        .await
        .expect("autonomy projection should assemble from typed events");

    assert_eq!(
        view.topology.rationale.selected_for,
        "minimize restart blast radius"
    );
    let team_sizing = view
        .team_sizing
        .expect("typed event projection should infer team sizing");
    assert_eq!(team_sizing.decision_phase, "event_projection");
    assert_eq!(team_sizing.desired_workers, 1);
    assert_eq!(team_sizing.selected_workers, 1);
    assert_eq!(team_sizing.available_workers, 1);
    assert_eq!(team_sizing.budget_pressure, Some(88));
    assert!(team_sizing.conservative_mode);
    assert!(team_sizing
        .rationale_lines
        .iter()
        .any(|line| line.contains("minimize restart blast radius")));
    assert_eq!(view.checkpoint_lineage.len(), 1);
    let supervision_evidence = view
        .supervision_evidence
        .expect("checkpoint-backed projection should include supervision evidence");
    let repair_lineage = supervision_evidence
        .repair_lineage_ref
        .as_ref()
        .expect("checkpoint-backed supervision should project packet-020 lineage");
    assert_eq!(repair_lineage.source, "packet-020");
    assert_eq!(
        repair_lineage.checkpoint_ref.as_ref(),
        Some(&checkpoint_id.to_string())
    );
    assert_eq!(
        supervision_evidence.proof_boundary.as_deref(),
        Some("supported task path")
    );
    assert_eq!(view.routing_history.len(), 1);
    assert_eq!(view.external_capability_decisions.len(), 2);
    assert_eq!(
        view.external_capability_decisions[0].branch_id,
        Some(branch_id)
    );
    assert_eq!(
        view.external_capability_decisions[0].outcome,
        ExternalCapabilityDecisionOutcome::Allowed
    );
    assert_eq!(
        view.external_capability_decisions[0].boundary_surface,
        Some(ExternalCapabilityDecisionSurface::ToolBus)
    );
    assert!(view.external_capability_decisions[0]
        .rationale
        .iter()
        .any(|line| line.contains("matched the requested external action")));
    assert_eq!(
        view.external_capability_decisions[1].branch_id,
        Some(retry_branch_id)
    );
    assert_eq!(
        view.external_capability_decisions[1].outcome,
        ExternalCapabilityDecisionOutcome::Rejected
    );
    assert_eq!(
        view.external_capability_decisions[1].boundary_surface,
        Some(ExternalCapabilityDecisionSurface::ToolBus)
    );
    assert_eq!(
        view.interventions[0].rationale,
        "operator escalation remained visible"
    );
    assert!(view
        .conservative_reasons
        .iter()
        .any(|reason| reason.contains("control-plane state unavailable")));
}

#[tokio::test]
async fn event_bus_aggregates_the_frozen_proof_outcome_matrix() {
    let expected_labels = ProofOutcomeClassification::ALL.map(ProofOutcomeClassification::as_str);
    assert_eq!(
        expected_labels,
        [
            "graph_formed_and_completed",
            "collapsed_to_sequential",
            "failed_before_graph",
        ]
    );

    let success_bus = EventBus::default();
    let success_workflow_id = TaskId::new();
    let success_graph_id = ExecutionGraphId::new();
    let success_branch = sample_branch_summary(success_graph_id);
    let success_graph = sample_graph_summary(
        success_workflow_id,
        success_graph_id,
        GraphState::Completed,
        2,
        4,
        Some(TopologyKind::Hybrid),
    );
    let success_topology = sample_topology_summary(
        success_graph_id,
        TopologyKind::Hybrid,
        2,
        TaskShapeKind::FanoutJoin,
    );
    let success_routing = sample_routing_summary(
        success_workflow_id,
        success_graph_id,
        success_branch.branch_id,
    );

    publish_projection(
        &success_bus,
        success_graph,
        success_topology.clone(),
        vec![success_branch.clone()],
        vec![success_routing.clone()],
    )
    .await;

    let success_view = success_bus
        .autonomy_status(&success_workflow_id)
        .await
        .expect("success projection should assemble");
    assert_eq!(
        infer_proof_outcome_from_projection(
            &success_view.graph,
            &success_view.topology,
            &success_view.branches,
            &success_view.routing_history,
        ),
        Some(ProofOutcomeClassification::GraphFormedAndCompleted)
    );
    assert_eq!(
        success_view
            .result_preview
            .expect("success case should infer a result preview"),
        OperatorResultPreview {
            workflow_id: success_workflow_id,
            proof_outcome: ProofOutcomeClassification::GraphFormedAndCompleted,
            preview_text: Some("workflow completed with 2 branch(es) across 4 node(s)".to_string()),
            payload_location: "task.result".to_string(),
            orchestration_quality: None,
            provenance_lines: vec![
                "graph formed and completed before final result publication".to_string(),
                format!(
                    "projection observed graph state {:?} with topology {:?} ({} branch(es), {} node(s))",
                    GraphState::Completed,
                    TopologyKind::Hybrid,
                    2,
                    4
                ),
                "canonical result stored in metadata.final_result".to_string(),
                "aggregated payload nested under metadata.aggregated_result".to_string(),
                "full payload remains recoverable from task.result".to_string(),
                "projection retained 1 branch detail record(s)".to_string(),
                "routing history retained 1 decision(s)".to_string(),
            ],
        }
    );

    let collapse_bus = EventBus::default();
    let collapse_workflow_id = TaskId::new();
    let collapse_graph_id = ExecutionGraphId::new();
    let collapse_graph = sample_graph_summary(
        collapse_workflow_id,
        collapse_graph_id,
        GraphState::Completed,
        1,
        1,
        Some(TopologyKind::Sequential),
    );
    let collapse_topology = sample_topology_summary(
        collapse_graph_id,
        TopologyKind::Sequential,
        1,
        TaskShapeKind::FanoutJoin,
    );

    publish_projection(
        &collapse_bus,
        collapse_graph,
        collapse_topology,
        vec![],
        vec![],
    )
    .await;

    let collapse_view = collapse_bus
        .autonomy_status(&collapse_workflow_id)
        .await
        .expect("collapse projection should assemble");
    assert_eq!(
        infer_proof_outcome_from_projection(
            &collapse_view.graph,
            &collapse_view.topology,
            &collapse_view.branches,
            &collapse_view.routing_history,
        ),
        Some(ProofOutcomeClassification::CollapsedToSequential)
    );
    let collapse_preview = collapse_view
        .result_preview
        .as_ref()
        .expect("collapsed case should infer a result preview");
    assert_eq!(
        collapse_preview.proof_outcome,
        ProofOutcomeClassification::CollapsedToSequential
    );
    assert_eq!(
        collapse_preview.preview_text.clone(),
        Some("completed with a sequential execution path".to_string())
    );
    assert!(collapse_preview
        .provenance_lines
        .iter()
        .any(|line| line.contains("planner emitted one sequential step")));

    let failure_bus = EventBus::default();
    let failure_workflow_id = TaskId::new();
    let failure_graph_id = ExecutionGraphId::new();
    let failure_graph = sample_graph_summary(
        failure_workflow_id,
        failure_graph_id,
        GraphState::Failed,
        0,
        0,
        None,
    );
    let failure_topology = sample_topology_summary(
        failure_graph_id,
        TopologyKind::Sequential,
        1,
        TaskShapeKind::StrictChain,
    );

    publish_projection(
        &failure_bus,
        failure_graph,
        failure_topology,
        vec![],
        vec![],
    )
    .await;

    let failure_view = failure_bus
        .autonomy_status(&failure_workflow_id)
        .await
        .expect("failed-before-graph projection should assemble");
    assert_eq!(
        infer_proof_outcome_from_projection(
            &failure_view.graph,
            &failure_view.topology,
            &failure_view.branches,
            &failure_view.routing_history,
        ),
        Some(ProofOutcomeClassification::FailedBeforeGraph)
    );
    let failure_preview = failure_view
        .result_preview
        .as_ref()
        .expect("failed-before-graph case should infer a result preview");
    assert_eq!(
        failure_preview.proof_outcome,
        ProofOutcomeClassification::FailedBeforeGraph
    );
    assert_eq!(
        failure_preview.preview_text.clone(),
        Some("workflow failed before graph formation".to_string())
    );
    assert!(failure_preview
        .provenance_lines
        .iter()
        .any(|line| line.contains("workflow failed before usable graph formation")));
}

#[tokio::test]
async fn event_bus_keeps_failed_visible_graph_runs_in_the_frozen_failure_class() {
    let event_bus = EventBus::default();
    let workflow_id = TaskId::new();
    let graph_id = ExecutionGraphId::new();
    let branch = sample_branch_summary(graph_id);
    let graph = sample_graph_summary(
        workflow_id,
        graph_id,
        GraphState::Failed,
        2,
        3,
        Some(TopologyKind::Hybrid),
    );
    let topology =
        sample_topology_summary(graph_id, TopologyKind::Hybrid, 2, TaskShapeKind::FanoutJoin);
    let routing = sample_routing_summary(workflow_id, graph_id, branch.branch_id);

    publish_projection(&event_bus, graph, topology, vec![branch], vec![routing]).await;

    let view = event_bus
        .autonomy_status(&workflow_id)
        .await
        .expect("projection should still assemble for failed visible-graph runs");

    assert_eq!(
        infer_proof_outcome_from_projection(
            &view.graph,
            &view.topology,
            &view.branches,
            &view.routing_history,
        ),
        Some(ProofOutcomeClassification::FailedBeforeGraph)
    );
    let preview = view
        .result_preview
        .expect("failed visible-graph runs should stay in the frozen failure class");
    assert_eq!(
        preview.proof_outcome,
        ProofOutcomeClassification::FailedBeforeGraph
    );
    assert_eq!(
        preview.preview_text.as_deref(),
        Some("workflow failed before graph formation")
    );
}

#[tokio::test]
async fn event_bus_merges_explicit_preview_with_projection_provenance() {
    let event_bus = EventBus::default();
    let workflow_id = TaskId::new();
    let graph_id = ExecutionGraphId::new();
    let branch = sample_branch_summary(graph_id);
    let routing = sample_routing_summary(workflow_id, graph_id, branch.branch_id);
    let view = AutonomyStatusView {
        session_id: None,
        turn_index: None,
        coordinator_agent_id: None,
        resume_provenance: None,
        graph: sample_graph_summary(
            workflow_id,
            graph_id,
            GraphState::Completed,
            2,
            4,
            Some(TopologyKind::Hybrid),
        ),
        topology: sample_topology_summary(
            graph_id,
            TopologyKind::Hybrid,
            2,
            TaskShapeKind::FanoutJoin,
        ),
        team_sizing: None,
        branches: vec![branch],
        checkpoint_lineage: vec![],
        memory_pressure: vec![],
        routing_history: vec![routing],
        step_routing_history: vec![],
        result_preview: Some(sample_result_preview(
            workflow_id,
            ProofOutcomeClassification::GraphFormedAndCompleted,
        )),
        interventions: vec![],
        delegation_capabilities: vec![],
        delegation_alerts: vec![],
        external_capability_decisions: vec![],
        profiles: vec![],
        guard_decisions: vec![],
        supervision_evidence: None,
        conservative_reasons: vec![],
    };

    event_bus
        .publish(
            AutonomyEvent::StatusUpdated(Box::new(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: None,
                payload: view,
                operator_visible: true,
            }))
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();

    let preview = event_bus
        .autonomy_status(&workflow_id)
        .await
        .expect("status snapshot should assemble")
        .result_preview
        .expect("result preview should remain present");

    assert_eq!(
        preview.preview_text.as_deref(),
        Some("bounded answer preview")
    );
    assert!(preview
        .provenance_lines
        .iter()
        .any(|line| line.contains("canonical result stored in metadata.final_result")));
    assert!(preview.provenance_lines.iter().any(|line| {
        line.contains("projection observed graph state Completed with topology Hybrid")
    }));
    assert!(preview
        .provenance_lines
        .iter()
        .any(|line| line.contains("routing history retained 1 decision(s)")));
}

#[tokio::test]
async fn delegation_decision_projection_preserves_branch_and_retry_history() {
    let event_bus = EventBus::default();
    let workflow_id = TaskId::new();
    let graph_id = ExecutionGraphId::new();
    let branch_a = ExecutionBranchId::new();
    let branch_b = ExecutionBranchId::new();
    let capability_id = CapabilityId::new();

    let status_view = AutonomyStatusView {
        session_id: None,
        turn_index: None,
        coordinator_agent_id: None,
        resume_provenance: None,
        graph: ExecutionGraphSummary {
            graph_id,
            workflow_id,
            state: GraphState::Running,
            branch_count: 2,
            node_count: 2,
            active_topology: Some(TopologyKind::Hybrid),
        },
        topology: TopologyPlanSummary {
            graph_id,
            topology_kind: TopologyKind::Hybrid,
            parallelism_width: 2,
            task_shape: sample_task_shape(TaskShapeKind::FanoutJoin),
            coordination_policy: CoordinationPolicy::Mixed,
            rationale: TopologyRationale {
                dependency_shape: "parallel branches".to_string(),
                operational_signals: vec![],
                selected_for: "preserve review visibility".to_string(),
                fallback_reason: None,
            },
            fallback_topology: None,
        },
        team_sizing: None,
        branches: vec![
            BranchSummary {
                branch_id: branch_a,
                graph_id,
                state: BranchState::Running,
                assigned_agents: vec![],
                checkpoint_id: None,
                recovery_strategy: BranchRecoveryStrategy::Resume,
            },
            BranchSummary {
                branch_id: branch_b,
                graph_id,
                state: BranchState::Running,
                assigned_agents: vec![],
                checkpoint_id: None,
                recovery_strategy: BranchRecoveryStrategy::Resume,
            },
        ],
        checkpoint_lineage: vec![],
        memory_pressure: vec![],
        routing_history: vec![],
        step_routing_history: vec![],
        result_preview: Some(sample_result_preview(
            workflow_id,
            ProofOutcomeClassification::GraphFormedAndCompleted,
        )),
        interventions: vec![],
        delegation_capabilities: vec![],
        delegation_alerts: vec![],
        external_capability_decisions: vec![],
        profiles: vec![],
        guard_decisions: vec![],
        supervision_evidence: None,
        conservative_reasons: vec![],
    };

    event_bus
        .publish(
            AutonomyEvent::StatusUpdated(Box::new(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: None,
                payload: status_view,
                operator_visible: true,
            }))
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();

    let mut branch_a_allowed =
        sample_external_capability_decision(capability_id, DelegationScope::InvokeTool);
    branch_a_allowed.branch_id = Some(branch_a);

    let mut branch_b_allowed =
        sample_external_capability_decision(capability_id, DelegationScope::InvokeTool);
    branch_b_allowed.branch_id = Some(branch_b);

    let mut branch_a_rejected =
        sample_external_capability_decision(capability_id, DelegationScope::InvokeTool);
    branch_a_rejected.branch_id = Some(branch_a);
    branch_a_rejected.outcome = ExternalCapabilityDecisionOutcome::Rejected;
    branch_a_rejected.rationale = vec!["descriptor mismatch on retry".to_string()];

    for (branch_id, payload) in [
        (branch_a, branch_a_allowed),
        (branch_b, branch_b_allowed),
        (branch_a, branch_a_rejected),
    ] {
        event_bus
            .publish(
                AutonomyEvent::DelegationDecisionRecorded(AutonomyEventEnvelope {
                    workflow_id,
                    graph_id: Some(graph_id),
                    branch_id: Some(branch_id),
                    payload,
                    operator_visible: true,
                })
                .into_event("autonomy-test"),
            )
            .await
            .unwrap();
    }

    let view = event_bus.autonomy_status(&workflow_id).await.unwrap();
    assert_eq!(
        view.result_preview,
        Some(sample_result_preview(
            workflow_id,
            ProofOutcomeClassification::GraphFormedAndCompleted,
        ))
    );

    let view = event_bus
        .autonomy_status(&workflow_id)
        .await
        .expect("autonomy projection should preserve distinct boundary decisions");

    assert_eq!(view.external_capability_decisions.len(), 3);
    assert_eq!(
        view.external_capability_decisions
            .iter()
            .filter(|decision| decision.branch_id == Some(branch_a))
            .count(),
        2
    );
    assert_eq!(
        view.external_capability_decisions
            .iter()
            .filter(|decision| decision.branch_id == Some(branch_b))
            .count(),
        1
    );
    assert!(view
        .external_capability_decisions
        .iter()
        .any(|decision| decision.branch_id == Some(branch_a)
            && decision.outcome == ExternalCapabilityDecisionOutcome::Rejected
            && decision.boundary_surface == Some(ExternalCapabilityDecisionSurface::ToolBus)));
}

#[test]
fn operator_result_preview_roundtrips_with_shared_contract_fields() {
    let workflow_id = TaskId::new();
    let preview = sample_result_preview(
        workflow_id,
        ProofOutcomeClassification::GraphFormedAndCompleted,
    );
    let provenance = ResultProvenanceSummary {
        runtime_execution_mode: serde_json::json!({"execution_boundary": "tool_bus"}),
        graph_state: Some("completed".to_string()),
        graph_id: Some(ExecutionGraphId::new().to_string()),
        source_fields: vec![
            "metadata.final_result".to_string(),
            "metadata.aggregated_result".to_string(),
        ],
    };

    let preview_json = serde_json::to_string(&preview).unwrap();
    let preview_roundtrip: OperatorResultPreview = serde_json::from_str(&preview_json).unwrap();

    let provenance_json = serde_json::to_string(&provenance).unwrap();
    let provenance_roundtrip: ResultProvenanceSummary =
        serde_json::from_str(&provenance_json).unwrap();

    assert_eq!(preview_roundtrip, preview);
    assert_eq!(provenance_roundtrip, provenance);
}

#[tokio::test]
async fn delegation_alerts_clear_after_status_snapshot_and_reactivation() {
    let event_bus = EventBus::default();
    let workflow_id = TaskId::new();
    let graph_id = ExecutionGraphId::new();
    let branch_id = ExecutionBranchId::new();
    let capability_id = CapabilityId::new();

    let status_view = AutonomyStatusView {
        session_id: None,
        turn_index: None,
        coordinator_agent_id: None,
        resume_provenance: None,
        graph: ExecutionGraphSummary {
            graph_id,
            workflow_id,
            state: GraphState::Running,
            branch_count: 1,
            node_count: 1,
            active_topology: Some(TopologyKind::Sequential),
        },
        topology: TopologyPlanSummary {
            graph_id,
            topology_kind: TopologyKind::Sequential,
            parallelism_width: 1,
            task_shape: sample_task_shape(TaskShapeKind::StrictChain),
            coordination_policy: CoordinationPolicy::StrictSequence,
            rationale: TopologyRationale {
                dependency_shape: "single branch".to_string(),
                operational_signals: vec!["delegation revoked".to_string()],
                selected_for: "keep operator in the loop".to_string(),
                fallback_reason: Some("delegation scope suspended".to_string()),
            },
            fallback_topology: Some(TopologyKind::Sequential),
        },
        team_sizing: Some(sample_team_sizing(
            workflow_id,
            graph_id,
            TaskShapeKind::StrictChain,
        )),
        branches: vec![BranchSummary {
            branch_id,
            graph_id,
            state: BranchState::Pending,
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
        delegation_alerts: vec![DelegationAlert {
            capability_id: Some(capability_id),
            scope: Some(DelegationScope::ManageBranch),
            revocation_state: Some(RevocationState::Revoked),
            parent_capability: None,
            expires_at: None,
            chain_depth: 1,
            rejection_reason: Some("Delegation capability revoked".to_string()),
            message: "delegation suspended pending operator review".to_string(),
        }],
        external_capability_decisions: vec![],
        profiles: vec![],
        guard_decisions: vec![],
        supervision_evidence: None,
        conservative_reasons: vec!["delegation scope suspended".to_string()],
    };

    event_bus
        .publish(
            AutonomyEvent::StatusUpdated(Box::new(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(branch_id),
                payload: status_view,
                operator_visible: true,
            }))
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::DelegationUpdated(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(branch_id),
                payload: {
                    let issuer = AuthorityPrincipal::Policy("operator".to_string());
                    let recipient = AgentId::new();
                    let expires_at = chrono::Utc::now();
                    CapabilitySummary {
                        capability_id,
                        descriptor_id: None,
                        issuer: issuer.clone(),
                        recipient,
                        scope: DelegationScope::ManageBranch,
                        parent_capability: None,
                        expires_at,
                        provenance: sample_provenance(
                            issuer,
                            recipient,
                            capability_id,
                            DelegationScope::ManageBranch,
                            expires_at,
                        ),
                        revocation_state: RevocationState::Active,
                        rejection_reason: None,
                    }
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();

    let view = event_bus
        .autonomy_status(&workflow_id)
        .await
        .expect("autonomy projection should remain visible after reactivation");

    assert!(view.delegation_alerts.is_empty());
}

#[tokio::test]
async fn event_bus_derives_parallel_team_sizing_without_status_snapshot() {
    let event_bus = EventBus::default();
    let workflow_id = TaskId::new();
    let graph_id = ExecutionGraphId::new();
    let alpha_branch = ExecutionBranchId::new();
    let beta_branch = ExecutionBranchId::new();
    let alpha_agent = AgentId::new();
    let beta_agent = AgentId::new();

    event_bus
        .publish(
            AutonomyEvent::GraphUpdated(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: None,
                payload: ExecutionGraphSummary {
                    graph_id,
                    workflow_id,
                    state: GraphState::Running,
                    branch_count: 2,
                    node_count: 3,
                    active_topology: Some(TopologyKind::Parallel),
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    event_bus
        .publish(
            AutonomyEvent::TopologySelected(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: None,
                payload: TopologyPlanSummary {
                    graph_id,
                    topology_kind: TopologyKind::Parallel,
                    parallelism_width: 2,
                    task_shape: sample_task_shape(TaskShapeKind::ParallelFanout),
                    coordination_policy: CoordinationPolicy::Barrier,
                    rationale: TopologyRationale {
                        dependency_shape: "independent branches".to_string(),
                        operational_signals: vec!["healthy profile".to_string()],
                        selected_for: "maximize safe concurrency".to_string(),
                        fallback_reason: Some(
                            "degrade to sequential when budgets tighten".to_string(),
                        ),
                    },
                    fallback_topology: Some(TopologyKind::Sequential),
                },
                operator_visible: true,
            })
            .into_event("autonomy-test"),
        )
        .await
        .unwrap();
    for (branch_id, agent_id) in [(alpha_branch, alpha_agent), (beta_branch, beta_agent)] {
        event_bus
            .publish(
                AutonomyEvent::BranchUpdated(AutonomyEventEnvelope {
                    workflow_id,
                    graph_id: Some(graph_id),
                    branch_id: Some(branch_id),
                    payload: BranchSummary {
                        branch_id,
                        graph_id,
                        state: BranchState::Running,
                        assigned_agents: vec![agent_id],
                        checkpoint_id: None,
                        recovery_strategy: BranchRecoveryStrategy::Resume,
                    },
                    operator_visible: true,
                })
                .into_event("autonomy-test"),
            )
            .await
            .unwrap();
        event_bus
            .publish(
                AutonomyEvent::RoutingDecisionRecorded(AutonomyEventEnvelope {
                    workflow_id,
                    graph_id: Some(graph_id),
                    branch_id: Some(branch_id),
                    payload: RoutingDecisionSummary {
                        graph_id,
                        branch_id,
                        selected_agent: agent_id,
                        task_ids: vec![TaskId::new()],
                        recovery_strategy: BranchRecoveryStrategy::Resume,
                        checkpoint_id: None,
                        dependency_depth: 1,
                        budget_pressure: 10,
                        health_state: HealthState::Healthy,
                        profile_id: None,
                        rationale: vec![
                            "parallel routing preserved independent branches".to_string()
                        ],
                    },
                    operator_visible: true,
                })
                .into_event("autonomy-test"),
            )
            .await
            .unwrap();
    }

    let view = event_bus
        .autonomy_status(&workflow_id)
        .await
        .expect("parallel projection should remain operator visible");
    let team_sizing = view
        .team_sizing
        .expect("parallel typed event projection should infer team sizing");

    assert_eq!(team_sizing.decision_phase, "event_projection");
    assert_eq!(team_sizing.desired_workers, 2);
    assert_eq!(team_sizing.selected_workers, 2);
    assert_eq!(team_sizing.available_workers, 2);
    assert_eq!(team_sizing.branch_frontier_width, 2);
    assert_eq!(team_sizing.dependency_depth, 1);
    assert_eq!(team_sizing.budget_pressure, Some(10));
    assert!(!team_sizing.conservative_mode);
    assert!(team_sizing.cap_reason.is_none());
    assert!(team_sizing
        .rationale_lines
        .iter()
        .any(|line| line.contains("maximize safe concurrency")));
}
