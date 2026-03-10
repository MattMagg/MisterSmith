use mister_smith_core::{
    AgentId, AuthorityPrincipal, BranchRecoveryStrategy, BranchState, BudgetPolicy, BudgetScope,
    CapabilityId, CheckpointId, ContextBudgetId, CoordinationPolicy, DelegationScope,
    ExecutionBranchId, ExecutionGraphId, FailureClass, GraphState, GuardDecision, GuardDecisionId,
    GuardEvidence, HealthState, InterventionRecordId, InterventionType, ProfileSnapshot,
    ProfileSnapshotId, ProfileTarget, RevocationState, TaskId, TopologyKind, TopologyRationale,
};
use mister_smith_events::{
    AutonomyEvent, AutonomyEventEnvelope, AutonomyEventType, AutonomyStatusView, BranchSummary,
    CapabilitySummary, ContextPressureSummary, DelegationAlert, EventType, ExecutionGraphSummary,
    TopologyPlanSummary,
};
use serde::de::DeserializeOwned;

fn assert_event_traits<T>()
where
    T: Clone + Send + Sync + std::fmt::Debug + serde::Serialize + DeserializeOwned + 'static,
{
}

#[test]
fn autonomy_event_surfaces_compile_with_shared_trait_bounds() {
    assert_event_traits::<ExecutionGraphSummary>();
    assert_event_traits::<TopologyPlanSummary>();
    assert_event_traits::<BranchSummary>();
    assert_event_traits::<ContextPressureSummary>();
    assert_event_traits::<CapabilitySummary>();
    assert_event_traits::<DelegationAlert>();
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
    let graph_id = ExecutionGraphId::new();
    let branch_id = ExecutionBranchId::new();
    let view = AutonomyStatusView {
        graph: ExecutionGraphSummary {
            graph_id,
            workflow_id: TaskId::new(),
            state: GraphState::Running,
            branch_count: 2,
            node_count: 5,
            active_topology: Some(TopologyKind::Hybrid),
        },
        topology: TopologyPlanSummary {
            graph_id,
            topology_kind: TopologyKind::Hybrid,
            parallelism_width: 2,
            coordination_policy: CoordinationPolicy::Mixed,
            rationale: TopologyRationale {
                dependency_shape: "mixed graph".to_string(),
                operational_signals: vec!["context pressure".to_string()],
                selected_for: "balance concurrency and recovery".to_string(),
                fallback_reason: None,
            },
            fallback_topology: Some(TopologyKind::Sequential),
        },
        branches: vec![BranchSummary {
            branch_id,
            graph_id,
            state: BranchState::Checkpointed,
            assigned_agents: vec![AgentId::new()],
            checkpoint_id: Some(CheckpointId::new()),
            recovery_strategy: BranchRecoveryStrategy::Resume,
        }],
        memory_pressure: vec![ContextPressureSummary {
            budget_id: ContextBudgetId::new(),
            scope: BudgetScope::Branch,
            max_units: 4096,
            reserved_units: 3072,
            policy: BudgetPolicy::Summarize,
        }],
        interventions: vec![mister_smith_core::InterventionRecord {
            record_id: InterventionRecordId::new(),
            decision_id: GuardDecisionId::new(),
            before_state: serde_json::json!({"state": "running"}),
            after_state: Some(serde_json::json!({"state": "isolated"})),
            rationale: "branch isolation".to_string(),
            emitted_at: chrono::Utc::now(),
        }],
        delegation_alerts: vec![
            DelegationAlert {
                capability_id: Some(CapabilityId::new()),
                scope: Some(DelegationScope::ManageBranch),
                revocation_state: Some(RevocationState::Revoked),
                message: "delegation revoked before branch resume".to_string(),
            },
            DelegationAlert {
                capability_id: None,
                scope: None,
                revocation_state: None,
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
}

#[test]
fn capability_summary_preserves_policy_issuers() {
    let summary = CapabilitySummary {
        capability_id: CapabilityId::new(),
        issuer: AuthorityPrincipal::Policy("bootstrap-policy".to_string()),
        recipient: AgentId::new(),
        scope: DelegationScope::ApplyIntervention,
        revocation_state: RevocationState::Active,
    };

    let json = serde_json::to_string(&summary).unwrap();
    let roundtrip: CapabilitySummary = serde_json::from_str(&json).unwrap();

    assert_eq!(roundtrip, summary);
}
