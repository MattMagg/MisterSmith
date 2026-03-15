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
    InterventionRecord, MemorySnapshotId, ProfileSnapshot, ProfileSnapshotId, ProvenanceChain,
    RevocationState, TaskId, TopologyKind, TopologyRationale,
};

use crate::builder::EventBuilder;
use crate::types::{Event, EventType};

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

/// Operator-facing autonomy status reconstructed from typed event state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutonomyStatusView {
    /// Graph summary for the running workflow.
    pub graph: ExecutionGraphSummary,
    /// Selected topology summary.
    pub topology: TopologyPlanSummary,
    /// Branch-level status summaries.
    pub branches: Vec<BranchSummary>,
    /// Checkpoint lineage visible to operators for targeted recovery.
    pub checkpoint_lineage: Vec<CheckpointRecordSummary>,
    /// Context-pressure summaries for active budgets.
    pub memory_pressure: Vec<ContextPressureSummary>,
    /// Routing history visible to operators.
    pub routing_history: Vec<RoutingDecisionSummary>,
    /// Applied intervention records visible to operators.
    pub interventions: Vec<InterventionRecord>,
    /// Active or recently rejected delegation capabilities.
    pub delegation_capabilities: Vec<CapabilitySummary>,
    /// Delegation or provenance warnings.
    pub delegation_alerts: Vec<DelegationAlert>,
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
    /// Aggregate status view changed.
    StatusUpdated(Box<AutonomyEventEnvelope<AutonomyStatusView>>),
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
            AutonomyEvent::StatusUpdated(envelope) => envelope.branch_id,
        }
    }
}
