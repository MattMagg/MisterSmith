//! Typed autonomy event envelopes and operator-facing summaries.
//!
//! These event contracts sit above the core autonomy value objects and give the
//! event layer stable, typed payloads for topology, memory pressure,
//! supervision, and delegation visibility.

use serde::{Deserialize, Serialize};

use mister_smith_core::{
    AgentId, AuthorityPrincipal, BranchRecoveryStrategy, BranchState, BudgetPolicy, BudgetScope,
    CapabilityId, CheckpointId, ContextBudgetId, CoordinationPolicy, DelegationScope,
    ExecutionBranchId, ExecutionGraphId, GraphState, GuardDecision, InterventionRecord,
    ProfileSnapshot, RevocationState, TaskId, TopologyKind, TopologyRationale,
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

/// Summary of context-pressure state for a budget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextPressureSummary {
    /// Budget record being reported.
    pub budget_id: ContextBudgetId,
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
    /// Current revocation state.
    pub revocation_state: RevocationState,
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
    /// Context-pressure summaries for active budgets.
    pub memory_pressure: Vec<ContextPressureSummary>,
    /// Applied intervention records visible to operators.
    pub interventions: Vec<InterventionRecord>,
    /// Delegation or provenance warnings.
    pub delegation_alerts: Vec<DelegationAlert>,
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
    /// Delegation capability changed.
    DelegationUpdated(AutonomyEventEnvelope<CapabilitySummary>),
    /// Aggregate status view changed.
    StatusUpdated(AutonomyEventEnvelope<AutonomyStatusView>),
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
}
