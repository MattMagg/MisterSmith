use mister_smith_core::{
    AgentId, AuthorityPrincipal, BranchRecoveryStrategy, BranchState, BudgetPolicy, BudgetScope,
    CapabilityId, CheckpointId, ContextBudgetId, CoordinationPolicy, DelegationScope,
    ExecutionBranchId, ExecutionGraphId, FailureClass, GraphState, GuardDecision, GuardDecisionId,
    GuardEvidence, HealthState, InterventionRecordId, InterventionType, ProfileSnapshot,
    ProfileSnapshotId, ProfileTarget, ProvenanceChain, ProvenanceLink, RevocationState, TaskId,
    TaskShapeClassification, TaskShapeKind, TeamSizingDecision, TopologyKind, TopologyRationale,
};
use mister_smith_events::{
    AutonomyEvent, AutonomyEventEnvelope, AutonomyEventType, AutonomyStatusView, BranchSummary,
    CapabilitySummary, CheckpointRecordSummary, ContextPressureSummary, DelegationAlert, EventBus,
    EventType, ExecutionGraphSummary, ResumeProvenanceSummary, RoutingDecisionSummary,
    StepRoutingDecisionSummary, TopologyPlanSummary,
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

#[test]
fn autonomy_event_surfaces_compile_with_shared_trait_bounds() {
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
fn autonomy_status_view_serializes_with_typed_summaries() {
    let workflow_id = TaskId::new();
    let graph_id = ExecutionGraphId::new();
    let branch_id = ExecutionBranchId::new();
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
        profiles: vec![ProfileSnapshot {
            profile_id: ProfileSnapshotId::new(),
            target: ProfileTarget::Branch,
            health_state: HealthState::Degraded,
            latency_window: None,
            error_window: None,
            semantic_signals: vec![],
            updated_at: chrono::Utc::now(),
        }],
        guard_decisions: vec![GuardDecision {
            decision_id: GuardDecisionId::new(),
            failure_class: FailureClass::Semantic,
            intervention: InterventionType::ContextRefresh,
            evidence: GuardEvidence {
                profile_id: None,
                signal_descriptions: vec!["loop detected".to_string()],
                checkpoint_ids: vec![],
                notes: vec!["operator review available".to_string()],
            },
            target_scope: mister_smith_core::GuardTarget::Branch(branch_id),
            operator_visibility: true,
        }],
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
        interventions: vec![],
        delegation_capabilities: vec![],
        delegation_alerts: vec![],
        profiles: vec![],
        guard_decisions: vec![],
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
    assert_eq!(view.routing_history.len(), 1);
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
        profiles: vec![],
        guard_decisions: vec![],
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
