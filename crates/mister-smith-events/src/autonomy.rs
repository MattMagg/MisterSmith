//! Typed autonomy event envelopes and operator-facing summaries.
//!
//! These event contracts sit above the core autonomy value objects and give the
//! event layer stable, typed payloads for topology, memory pressure,
//! supervision, and delegation visibility.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use mister_smith_core::{
    AgentId, AuthorityPrincipal, BranchRecoveryStrategy, BranchState, BudgetPolicy, BudgetScope,
    CapabilityId, CheckpointId, ContextBudgetId, CoordinationPolicy, DelegationScope,
    ExecutionBranchId, ExecutionGraphId, ExecutionNodeId, GraphState, GuardDecision, HealthState,
    InterventionRecord, MemorySnapshotId, OperatorResultPreview, ProfileSnapshot,
    ProfileSnapshotId, ProofOutcomeClassification, ProvenanceChain, RevocationState, SessionId,
    TaskId, TaskShapeClassification, TaskShapeKind, TeamSizingDecision, TopologyKind,
    TopologyRationale,
};

use crate::builder::EventBuilder;
use crate::types::{Event, EventType};

fn is_false(value: &bool) -> bool {
    !*value
}

/// Summary of an execution graph for operator-visible status surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionGraphSummary {
    /// Stable execution graph identifier.
    pub graph_id: ExecutionGraphId,
    /// Workflow that owns the graph.
    pub workflow_id: TaskId,
    /// Current graph lifecycle state.
    pub state: GraphState,
    /// Number of branches currently known in the graph.
    pub branch_count: usize,
    /// Number of nodes currently known in the graph.
    pub node_count: usize,
    /// Active topology for the graph when selected.
    pub active_topology: Option<TopologyKind>,
}

/// Summary of the topology selected for a graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopologyPlanSummary {
    /// Graph this topology applies to.
    pub graph_id: ExecutionGraphId,
    /// Selected topology kind.
    pub topology_kind: TopologyKind,
    /// Maximum execution width for the selected topology.
    pub parallelism_width: usize,
    /// Dependency-derived task shape that informed topology selection.
    pub task_shape: TaskShapeClassification,
    /// Coordination behavior for the graph.
    pub coordination_policy: CoordinationPolicy,
    /// Structured rationale for the topology choice.
    pub rationale: TopologyRationale,
    /// Conservative fallback topology when signals degrade.
    pub fallback_topology: Option<TopologyKind>,
}

/// Summary of a checkpointable execution branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchSummary {
    /// Stable branch identifier.
    pub branch_id: ExecutionBranchId,
    /// Parent graph that owns the branch.
    pub graph_id: ExecutionGraphId,
    /// Current branch lifecycle state.
    pub state: BranchState,
    /// Agents currently assigned to the branch.
    pub assigned_agents: Vec<AgentId>,
    /// Latest durable checkpoint when available.
    pub checkpoint_id: Option<CheckpointId>,
    /// Recovery strategy the branch will use on failure.
    pub recovery_strategy: BranchRecoveryStrategy,
}

/// Summary of a durable branch checkpoint capture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckpointRecordSummary {
    /// Parent graph that owns the checkpoint lineage entry.
    pub graph_id: ExecutionGraphId,
    /// Branch that recorded the durable checkpoint.
    pub branch_id: ExecutionBranchId,
    /// Stable checkpoint identifier.
    pub checkpoint_id: CheckpointId,
    /// When the durable checkpoint was captured.
    pub captured_at: DateTime<Utc>,
    /// Managed-memory snapshot used for branch resume.
    pub memory_snapshot_id: MemorySnapshotId,
    /// Nodes already completed safely at this checkpoint.
    pub completed_nodes: Vec<ExecutionNodeId>,
    /// Nodes still pending from the checkpoint-safe recovery point.
    pub pending_nodes: Vec<ExecutionNodeId>,
    /// Recovery strategy active for the branch when the checkpoint was recorded.
    pub recovery_strategy: BranchRecoveryStrategy,
    /// Optional failure or intervention context captured at checkpoint time.
    pub failure_context: Option<Value>,
}

/// Summary of a routing decision emitted for a ready branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingDecisionSummary {
    /// Parent graph that owns the routed branch.
    pub graph_id: ExecutionGraphId,
    /// Branch selected for routing.
    pub branch_id: ExecutionBranchId,
    /// Agent selected to execute or resume the branch.
    pub selected_agent: AgentId,
    /// Task identifiers assigned by the routing decision.
    pub task_ids: Vec<TaskId>,
    /// Recovery strategy that shaped the routing path.
    pub recovery_strategy: BranchRecoveryStrategy,
    /// Latest checkpoint used for branch-local recovery when available.
    pub checkpoint_id: Option<CheckpointId>,
    /// Deepest dependency distance across the routed branch scope.
    pub dependency_depth: usize,
    /// Coarse pressure score from 0 to 100 derived from branch budgets.
    pub budget_pressure: u8,
    /// Health state observed for the branch at routing time.
    pub health_state: HealthState,
    /// Latest profile snapshot that informed the routing decision, when available.
    pub profile_id: Option<ProfileSnapshotId>,
    /// Operator-visible rationale lines explaining the decision.
    pub rationale: Vec<String>,
}

/// Summary of one step-level routing decision carried between workflow steps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepRoutingDecisionSummary {
    /// Stable step identifier from the routing metadata.
    pub step_id: String,
    /// Monotonic step index when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_index: Option<u32>,
    /// Semantic step kind such as planner or critic when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_kind: Option<String>,
    /// Model selected for the routed step.
    pub model_id: String,
    /// Tier label exposed by the router for the selected model.
    pub tier: String,
    /// Router-provided explanation for the decision.
    pub reason: String,
    /// Prior step identifier when this entry is compared against a previous step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_step_id: Option<String>,
    /// Prior step action when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_action: Option<String>,
    /// Prior selected tier when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_tier: Option<String>,
    /// Action that the next step should inherit from this decision.
    pub action: String,
    /// Whether the carryover action changed relative to the prior step.
    #[serde(default, skip_serializing_if = "is_false")]
    pub action_changed: bool,
    /// Preferred tier that the workflow should carry into the next step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_tier_after: Option<String>,
    /// Estimated token cost reported by the router when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_cost_tokens: Option<u64>,
    /// Confidence score surfaced for the step when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence_score: Option<f32>,
    /// Triggered verification checkpoints that shaped the step decision.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub triggered_checkpoints: Vec<String>,
    /// Operator-visible explanation of why the decision changed relative to the prior step.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub change_rationale: Vec<String>,
}

/// Summary of context-pressure state for a budget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextPressureSummary {
    /// Budget record being reported.
    pub budget_id: ContextBudgetId,
    /// Branch the budget pressure applies to, when available.
    pub branch_id: Option<ExecutionBranchId>,
    /// Scope the budget applies to.
    pub scope: BudgetScope,
    /// Maximum context units allowed.
    pub max_units: u64,
    /// Units currently reserved or in use.
    pub reserved_units: u64,
    /// Policy used when the budget is exceeded.
    pub policy: BudgetPolicy,
}

/// Summary of a delegation capability for autonomy status views.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilitySummary {
    /// Capability identifier.
    pub capability_id: CapabilityId,
    /// Capability descriptor identifier bound to the capability, when any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descriptor_id: Option<String>,
    /// Principal that issued or currently owns the capability.
    pub issuer: AuthorityPrincipal,
    /// Agent that received the capability.
    pub recipient: AgentId,
    /// Scope granted by the capability.
    pub scope: DelegationScope,
    /// Parent capability in the delegation chain, when any.
    pub parent_capability: Option<CapabilityId>,
    /// When the capability expires.
    pub expires_at: DateTime<Utc>,
    /// Ordered provenance chain for the capability.
    pub provenance: ProvenanceChain,
    /// Current revocation state.
    pub revocation_state: RevocationState,
    /// Operator-visible rejection reason when the capability was denied.
    pub rejection_reason: Option<String>,
}

/// Operator-visible delegation warning or provenance concern.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationAlert {
    /// Capability involved in the alert when available.
    pub capability_id: Option<CapabilityId>,
    /// Scope involved in the alert when available.
    pub scope: Option<DelegationScope>,
    /// Revocation state relevant to the alert when available.
    pub revocation_state: Option<RevocationState>,
    /// Parent capability in the chain, when available.
    pub parent_capability: Option<CapabilityId>,
    /// Effective expiry time for the capability, when available.
    pub expires_at: Option<DateTime<Utc>>,
    /// Depth of the provenance chain.
    pub chain_depth: usize,
    /// Human-readable rejection reason when the capability was denied.
    pub rejection_reason: Option<String>,
    /// Human-readable explanation of the alert.
    pub message: String,
}

/// Operator-facing outcome for an external capability boundary decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ExternalCapabilityDecisionOutcome {
    /// The external capability call remained authorized at the boundary.
    Allowed,
    /// The external capability call was rejected at the boundary.
    Rejected,
}

/// Operator-visible explanation of one external capability boundary decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalCapabilityDecisionSummary {
    /// Branch that exercised the external capability boundary, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_id: Option<ExecutionBranchId>,
    /// Capability used for the external boundary call, when one was present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_id: Option<CapabilityId>,
    /// Descriptor bound to the capability, when any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_descriptor_id: Option<String>,
    /// Descriptor requested by the external delegated action, when any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_descriptor_id: Option<String>,
    /// Stable action identifier at the boundary, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_id: Option<String>,
    /// Human-readable delegated action title, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_title: Option<String>,
    /// Scope carried by the capability at the boundary, when one was present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<DelegationScope>,
    /// Required scope requested by the external action, when any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_scope: Option<DelegationScope>,
    /// Policy action evaluated for the delegated boundary, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_action: Option<String>,
    /// Policy resource evaluated for the delegated boundary, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_resource: Option<String>,
    /// Policy scope evaluated for the delegated boundary, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_scope: Option<String>,
    /// Optional concrete policy resource identifier at the boundary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_resource_id: Option<String>,
    /// Effective revocation state observed for the capability, when one was present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revocation_state: Option<RevocationState>,
    /// Depth of the authority chain used for the boundary call.
    pub chain_depth: usize,
    /// Final operator-facing decision outcome.
    pub outcome: ExternalCapabilityDecisionOutcome,
    /// Time when the boundary decision was observed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_at: Option<DateTime<Utc>>,
    /// Operator-visible explanation of why the decision was allowed or rejected.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rationale: Vec<String>,
}

/// Operator-visible restart and resume provenance for a workflow turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumeProvenanceSummary {
    /// Workflow record was recovered after a runtime restart.
    #[serde(default, skip_serializing_if = "is_false")]
    pub recovered_after_restart: bool,
    /// Resumed turn continues after a restart-recovered prior workflow.
    #[serde(default, skip_serializing_if = "is_false")]
    pub resumed_after_restart: bool,
    /// Timestamp recorded when the runtime marked the workflow as recovered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovered_at: Option<DateTime<Utc>>,
    /// Human-readable recovery reason recorded in workflow metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_reason: Option<String>,
    /// Prior workflow in the resumed turn lineage, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resumed_from_workflow_id: Option<TaskId>,
    /// Prior turn index in the resumed turn lineage, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resumed_from_turn_index: Option<u32>,
}

/// Operator-facing autonomy status reconstructed from typed event state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutonomyStatusView {
    /// Owning conversation session when the workflow belongs to one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,
    /// Accepted turn order within the owning session, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_index: Option<u32>,
    /// Stable session coordinator identity when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordinator_agent_id: Option<AgentId>,
    /// Restart and resume provenance for the workflow when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resume_provenance: Option<ResumeProvenanceSummary>,
    /// Graph summary for the running workflow.
    pub graph: ExecutionGraphSummary,
    /// Selected topology summary.
    pub topology: TopologyPlanSummary,
    /// Frozen adaptive team-sizing decision when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_sizing: Option<TeamSizingDecision>,
    /// Branch-level status summaries.
    pub branches: Vec<BranchSummary>,
    /// Checkpoint lineage visible to operators for targeted recovery.
    pub checkpoint_lineage: Vec<CheckpointRecordSummary>,
    /// Context-pressure summaries for active budgets.
    pub memory_pressure: Vec<ContextPressureSummary>,
    /// Routing history visible to operators.
    pub routing_history: Vec<RoutingDecisionSummary>,
    /// Step-level routing history projected from workflow metadata when available.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub step_routing_history: Vec<StepRoutingDecisionSummary>,
    /// Compact operator-facing result preview and provenance when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_preview: Option<OperatorResultPreview>,
    /// Applied intervention records visible to operators.
    pub interventions: Vec<InterventionRecord>,
    /// Active or recently rejected delegation capabilities.
    pub delegation_capabilities: Vec<CapabilitySummary>,
    /// Delegation or provenance warnings.
    pub delegation_alerts: Vec<DelegationAlert>,
    /// External capability boundary decisions projected for operator inspection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_capability_decisions: Vec<ExternalCapabilityDecisionSummary>,
    /// Supervisory profile snapshots retained for operator inspection.
    pub profiles: Vec<ProfileSnapshot>,
    /// Guard decisions that informed the current supervision posture.
    pub guard_decisions: Vec<GuardDecision>,
    /// Reasons the system narrowed autonomy conservatively.
    pub conservative_reasons: Vec<String>,
}

/// Common envelope used by typed autonomy events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutonomyEventEnvelope<T> {
    /// Workflow the event belongs to.
    pub workflow_id: TaskId,
    /// Graph related to the event when available.
    pub graph_id: Option<ExecutionGraphId>,
    /// Branch related to the event when available.
    pub branch_id: Option<ExecutionBranchId>,
    /// Typed payload carried by the event.
    pub payload: T,
    /// Whether the event should be surfaced directly to operators.
    pub operator_visible: bool,
}

/// Event-type discriminator for Phase 10 autonomy payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AutonomyEventType {
    /// Execution-graph summary changed.
    GraphUpdated,
    /// A topology was selected or changed.
    TopologySelected,
    /// Branch state changed.
    BranchUpdated,
    /// Context pressure changed for a budget or scope.
    ContextPressureObserved,
    /// Profile snapshot recorded for routing or supervision.
    ProfileSnapshotRecorded,
    /// Guard decision evaluated for a target.
    GuardDecisionEvaluated,
    /// Intervention record emitted after an action.
    InterventionRecorded,
    /// Branch checkpoint captured for targeted recovery.
    CheckpointRecorded,
    /// Routing decision recorded for a ready branch.
    RoutingDecisionRecorded,
    /// Delegation capability or provenance state changed.
    DelegationUpdated,
    /// Delegation allow/reject decision changed.
    DelegationDecisionRecorded,
    /// Aggregate autonomy status view changed.
    StatusUpdated,
}

/// Strongly typed autonomy event payloads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AutonomyEvent {
    /// Execution-graph summary changed.
    GraphUpdated(AutonomyEventEnvelope<ExecutionGraphSummary>),
    /// Topology selection changed.
    TopologySelected(AutonomyEventEnvelope<TopologyPlanSummary>),
    /// Branch state changed.
    BranchUpdated(AutonomyEventEnvelope<BranchSummary>),
    /// Context pressure changed.
    ContextPressureObserved(AutonomyEventEnvelope<ContextPressureSummary>),
    /// Profile snapshot recorded.
    ProfileSnapshotRecorded(AutonomyEventEnvelope<ProfileSnapshot>),
    /// Guard decision evaluated.
    GuardDecisionEvaluated(AutonomyEventEnvelope<GuardDecision>),
    /// Intervention recorded.
    InterventionRecorded(AutonomyEventEnvelope<InterventionRecord>),
    /// Branch checkpoint recorded.
    CheckpointRecorded(AutonomyEventEnvelope<CheckpointRecordSummary>),
    /// Routing decision recorded.
    RoutingDecisionRecorded(AutonomyEventEnvelope<RoutingDecisionSummary>),
    /// Delegation capability changed.
    DelegationUpdated(AutonomyEventEnvelope<CapabilitySummary>),
    /// Delegation allow/reject decision changed.
    DelegationDecisionRecorded(AutonomyEventEnvelope<ExternalCapabilityDecisionSummary>),
    /// Aggregate status view changed.
    StatusUpdated(Box<AutonomyEventEnvelope<AutonomyStatusView>>),
}

/// Derive the shared proof-outcome class from a typed autonomy projection when possible.
#[must_use]
pub fn infer_proof_outcome_from_projection(
    graph: &ExecutionGraphSummary,
    topology: &TopologyPlanSummary,
    branches: &[BranchSummary],
    routing_history: &[RoutingDecisionSummary],
) -> Option<ProofOutcomeClassification> {
    match graph.state {
        GraphState::Completed => {
            let collapsed_to_sequential = topology.topology_kind == TopologyKind::Sequential
                && topology.parallelism_width <= 1
                && (topology.task_shape.max_parallel_width > 1
                    || !matches!(topology.task_shape.kind, TaskShapeKind::StrictChain));

            Some(if collapsed_to_sequential {
                ProofOutcomeClassification::CollapsedToSequential
            } else {
                ProofOutcomeClassification::GraphFormedAndCompleted
            })
        }
        GraphState::Failed | GraphState::Aborted => {
            let formed_visible_graph = graph.branch_count > 1
                || graph.node_count > 1
                || !branches.is_empty()
                || !routing_history.is_empty();
            (!formed_visible_graph).then_some(ProofOutcomeClassification::FailedBeforeGraph)
        }
        GraphState::Pending | GraphState::Running | GraphState::Checkpointed => None,
    }
}

/// Derive a bounded operator-facing result preview from a typed autonomy projection.
#[must_use]
pub fn infer_result_preview_from_projection(
    graph: &ExecutionGraphSummary,
    topology: &TopologyPlanSummary,
    branches: &[BranchSummary],
    routing_history: &[RoutingDecisionSummary],
) -> Option<OperatorResultPreview> {
    let proof_outcome =
        infer_proof_outcome_from_projection(graph, topology, branches, routing_history)?;

    let preview_text = match proof_outcome {
        ProofOutcomeClassification::GraphFormedAndCompleted => Some(format!(
            "workflow completed with {} branch(es) across {} node(s)",
            graph.branch_count, graph.node_count
        )),
        ProofOutcomeClassification::CollapsedToSequential => {
            Some("completed with a sequential execution path".to_string())
        }
        ProofOutcomeClassification::FailedBeforeGraph => {
            Some("workflow failed before graph formation".to_string())
        }
    };

    let mut provenance_lines = vec![
        "canonical result stored in metadata.final_result".to_string(),
        "aggregated payload nested under metadata.aggregated_result".to_string(),
        "full payload remains recoverable from task.result".to_string(),
    ];
    provenance_lines.push(format!(
        "projection observed graph state {:?} with topology {:?}",
        graph.state, topology.topology_kind
    ));
    if !routing_history.is_empty() {
        provenance_lines.push(format!(
            "routing history retained {} decision(s)",
            routing_history.len()
        ));
    }

    Some(OperatorResultPreview {
        workflow_id: graph.workflow_id,
        proof_outcome,
        preview_text,
        payload_location: "task.result".to_string(),
        provenance_lines,
    })
}

impl CapabilitySummary {
    /// Return the depth of the provenance chain for this capability.
    #[must_use]
    pub fn chain_depth(&self) -> usize {
        self.provenance.links.len()
    }

    /// Build an operator-visible alert when the capability is not currently usable.
    #[must_use]
    pub fn to_alert(&self) -> Option<DelegationAlert> {
        if self.revocation_state == RevocationState::Active && self.rejection_reason.is_none() {
            return None;
        }

        let message = self.rejection_reason.clone().unwrap_or_else(|| {
            format!(
                "delegation for agent {} in scope {:?} is {:?}",
                self.recipient, self.scope, self.revocation_state
            )
        });

        Some(DelegationAlert {
            capability_id: Some(self.capability_id),
            scope: Some(self.scope),
            revocation_state: Some(self.revocation_state),
            parent_capability: self.parent_capability,
            expires_at: Some(self.expires_at),
            chain_depth: self.chain_depth(),
            rejection_reason: self.rejection_reason.clone(),
            message,
        })
    }
}

impl AutonomyEvent {
    /// Returns the autonomy event type for this payload.
    pub fn kind(&self) -> AutonomyEventType {
        match self {
            AutonomyEvent::GraphUpdated(_) => AutonomyEventType::GraphUpdated,
            AutonomyEvent::TopologySelected(_) => AutonomyEventType::TopologySelected,
            AutonomyEvent::BranchUpdated(_) => AutonomyEventType::BranchUpdated,
            AutonomyEvent::ContextPressureObserved(_) => AutonomyEventType::ContextPressureObserved,
            AutonomyEvent::ProfileSnapshotRecorded(_) => AutonomyEventType::ProfileSnapshotRecorded,
            AutonomyEvent::GuardDecisionEvaluated(_) => AutonomyEventType::GuardDecisionEvaluated,
            AutonomyEvent::InterventionRecorded(_) => AutonomyEventType::InterventionRecorded,
            AutonomyEvent::CheckpointRecorded(_) => AutonomyEventType::CheckpointRecorded,
            AutonomyEvent::RoutingDecisionRecorded(_) => AutonomyEventType::RoutingDecisionRecorded,
            AutonomyEvent::DelegationUpdated(_) => AutonomyEventType::DelegationUpdated,
            AutonomyEvent::DelegationDecisionRecorded(_) => {
                AutonomyEventType::DelegationDecisionRecorded
            }
            AutonomyEvent::StatusUpdated(_) => AutonomyEventType::StatusUpdated,
        }
    }

    /// Converts the typed autonomy payload into the crate's generic [`Event`].
    pub fn into_event(self, source: impl Into<String>) -> Event {
        EventBuilder::new(source, EventType::Autonomy(self.kind()))
            .with_payload(&self)
            .build()
    }

    /// Return the workflow identifier carried by the autonomy event.
    pub fn workflow_id(&self) -> TaskId {
        match self {
            AutonomyEvent::GraphUpdated(envelope) => envelope.workflow_id,
            AutonomyEvent::TopologySelected(envelope) => envelope.workflow_id,
            AutonomyEvent::BranchUpdated(envelope) => envelope.workflow_id,
            AutonomyEvent::ContextPressureObserved(envelope) => envelope.workflow_id,
            AutonomyEvent::ProfileSnapshotRecorded(envelope) => envelope.workflow_id,
            AutonomyEvent::GuardDecisionEvaluated(envelope) => envelope.workflow_id,
            AutonomyEvent::InterventionRecorded(envelope) => envelope.workflow_id,
            AutonomyEvent::CheckpointRecorded(envelope) => envelope.workflow_id,
            AutonomyEvent::RoutingDecisionRecorded(envelope) => envelope.workflow_id,
            AutonomyEvent::DelegationUpdated(envelope) => envelope.workflow_id,
            AutonomyEvent::DelegationDecisionRecorded(envelope) => envelope.workflow_id,
            AutonomyEvent::StatusUpdated(envelope) => envelope.workflow_id,
        }
    }

    /// Return the branch identifier carried by the autonomy event, when available.
    pub fn branch_id(&self) -> Option<ExecutionBranchId> {
        match self {
            AutonomyEvent::GraphUpdated(envelope) => envelope.branch_id,
            AutonomyEvent::TopologySelected(envelope) => envelope.branch_id,
            AutonomyEvent::BranchUpdated(envelope) => envelope.branch_id,
            AutonomyEvent::ContextPressureObserved(envelope) => envelope.branch_id,
            AutonomyEvent::ProfileSnapshotRecorded(envelope) => envelope.branch_id,
            AutonomyEvent::GuardDecisionEvaluated(envelope) => envelope.branch_id,
            AutonomyEvent::InterventionRecorded(envelope) => envelope.branch_id,
            AutonomyEvent::CheckpointRecorded(envelope) => envelope.branch_id,
            AutonomyEvent::RoutingDecisionRecorded(envelope) => envelope.branch_id,
            AutonomyEvent::DelegationUpdated(envelope) => envelope.branch_id,
            AutonomyEvent::DelegationDecisionRecorded(envelope) => envelope.branch_id,
            AutonomyEvent::StatusUpdated(envelope) => envelope.branch_id,
        }
    }
}
