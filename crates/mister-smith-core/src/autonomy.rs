//! Shared Phase 10 autonomy value objects.
//!
//! These types are intentionally behavioral-free foundations for downstream
//! topology, memory, Guard, and delegation work. They define the stable
//! cross-crate contracts that later Phase 10 crates will exchange through
//! typed events and operator-facing summaries.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::enums::{
    BudgetPolicy, BudgetScope, CoordinationPolicy, DelegationScope, FailureClass, HealthState,
    InterventionType, ProfileTarget, RevocationState, SemanticSignalKind, TopologyKind,
};
use crate::ids::{
    AgentId, CapabilityId, CheckpointId, ContextBudgetId, ExecutionBranchId, ExecutionGraphId,
    ExecutionNodeId, GuardDecisionId, InterventionRecordId, ProfileSnapshotId,
};

/// Coarse task-structure class derived from dependency analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskShapeKind {
    /// A strict dependency chain with at most one ready step at a time.
    StrictChain,
    /// Independent work can fan out safely without a downstream join.
    ParallelFanout,
    /// Parallel work fans out and later reconverges through an explicit join.
    FanoutJoin,
    /// A deeper fanout tree benefits from hierarchical coordination.
    HierarchicalFanout,
    /// The graph does not fit one of the bounded representative shapes cleanly.
    MixedGraph,
}

impl TaskShapeKind {
    /// Return the stable status-surface label for this task shape.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StrictChain => "strict-chain",
            Self::ParallelFanout => "parallel-fanout",
            Self::FanoutJoin => "fanout-join",
            Self::HierarchicalFanout => "hierarchical-fanout",
            Self::MixedGraph => "mixed-graph",
        }
    }
}

/// Structured task-shape classification derived before topology selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskShapeClassification {
    /// Bounded representative task-shape label.
    pub kind: TaskShapeKind,
    /// Number of root nodes with no upstream dependencies.
    pub root_count: usize,
    /// Widest dependency level available for parallel execution.
    pub max_parallel_width: usize,
    /// Deepest dependency distance from any root.
    pub max_depth: usize,
    /// Whether the graph contains a downstream reconvergence point.
    pub has_join: bool,
    /// Whether any node fans out to more than one dependent.
    pub has_fanout: bool,
    /// Stable heuristic signals used to justify the classification.
    pub structural_signals: Vec<String>,
}

/// Structured explanation for why a topology was selected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopologyRationale {
    /// Dependency shape observed by the topology compiler.
    pub dependency_shape: String,
    /// Operational signals that influenced the topology choice.
    pub operational_signals: Vec<String>,
    /// Human-readable explanation for the selected topology.
    pub selected_for: String,
    /// Optional explanation for the conservative fallback topology.
    pub fallback_reason: Option<String>,
}

/// Chosen execution shape for a validated execution graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopologyPlan {
    /// Selected topology kind for the graph.
    pub topology_kind: TopologyKind,
    /// Maximum concurrent execution width for this plan.
    pub parallelism_width: usize,
    /// Dependency-derived task shape that informed topology selection.
    pub task_shape: TaskShapeClassification,
    /// Why this topology was selected.
    pub rationale: TopologyRationale,
    /// Coordination behavior used by the orchestrator.
    pub coordination_policy: CoordinationPolicy,
    /// Conservative fallback topology when signals degrade.
    pub fallback_topology: Option<TopologyKind>,
}

/// Bounded context allowance for a workflow scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextBudget {
    /// Stable identifier for this budget record.
    pub budget_id: ContextBudgetId,
    /// Scope the budget applies to.
    pub scope: BudgetScope,
    /// Maximum allowed context units.
    pub max_units: u64,
    /// Units already reserved or in use.
    pub reserved_units: u64,
    /// Policy to apply when the budget would be exceeded.
    pub policy: BudgetPolicy,
}

/// Aggregated measurements over a recent time window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricWindow {
    /// Number of samples included in the window.
    pub sample_count: u64,
    /// Smallest observed value in the window.
    pub minimum: u64,
    /// Largest observed value in the window.
    pub maximum: u64,
    /// Average observed value in the window.
    pub average: u64,
}

/// Step-level or stream-level degradation signal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticSignal {
    /// Kind of degradation or semantic warning observed.
    pub signal_kind: SemanticSignalKind,
    /// Coarse severity level from 0 to 100.
    pub severity: u8,
    /// Human-readable detail for operator visibility and audits.
    pub detail: String,
}

/// Telemetry and performance state used by routing and supervision decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileSnapshot {
    /// Stable identifier for this profile sample.
    pub profile_id: ProfileSnapshotId,
    /// Runtime entity the profile describes.
    pub target: ProfileTarget,
    /// Current health state for the target.
    pub health_state: HealthState,
    /// Recent latency measurements when available.
    pub latency_window: Option<MetricWindow>,
    /// Recent error measurements when available.
    pub error_window: Option<MetricWindow>,
    /// Step-level semantic or stream degradation signals.
    pub semantic_signals: Vec<SemanticSignal>,
    /// Timestamp when the profile snapshot was captured.
    pub updated_at: DateTime<Utc>,
}

/// Evidence used by the Guard layer when making a supervisory decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardEvidence {
    /// Optional profile snapshot that informed the decision.
    pub profile_id: Option<ProfileSnapshotId>,
    /// Human-readable descriptions of the triggering signals.
    pub signal_descriptions: Vec<String>,
    /// Related checkpoints that informed a recovery choice.
    pub checkpoint_ids: Vec<CheckpointId>,
    /// Additional operator-visible notes captured during evaluation.
    pub notes: Vec<String>,
}

/// Concrete scope targeted by a Guard decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardTarget {
    /// A specific execution node is the target.
    Node(ExecutionNodeId),
    /// A specific execution branch is the target.
    Branch(ExecutionBranchId),
    /// A whole execution graph is the target.
    Graph(ExecutionGraphId),
    /// A provider-level dependency is the target.
    Provider(String),
}

/// Supervisory decision produced by the Guard layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardDecision {
    /// Stable identifier for this decision.
    pub decision_id: GuardDecisionId,
    /// Failure class assigned by the Guard layer.
    pub failure_class: FailureClass,
    /// Intervention selected for the target.
    pub intervention: InterventionType,
    /// Evidence used to justify the decision.
    pub evidence: GuardEvidence,
    /// Concrete target scope of the intervention.
    pub target_scope: GuardTarget,
    /// Whether the decision must be surfaced to operators immediately.
    pub operator_visibility: bool,
}

/// Operator-visible audit record of an applied supervisory action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterventionRecord {
    /// Stable identifier for this record.
    pub record_id: InterventionRecordId,
    /// Guard decision that produced the intervention.
    pub decision_id: GuardDecisionId,
    /// Relevant state captured before the intervention.
    pub before_state: Value,
    /// Relevant state captured after the intervention, when known.
    pub after_state: Option<Value>,
    /// Human-readable rationale for the intervention.
    pub rationale: String,
    /// Timestamp when the record was emitted.
    pub emitted_at: DateTime<Utc>,
}

/// Authority principal that can issue delegation capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorityPrincipal {
    /// Authority issued by a concrete runtime agent.
    Agent(AgentId),
    /// Authority issued by a named policy principal.
    Policy(String),
}

/// Bounded unit of delegated authority for privileged autonomy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationCapability {
    /// Stable identifier for the capability.
    pub capability_id: CapabilityId,
    /// Principal that issued the capability.
    pub issuer: AuthorityPrincipal,
    /// Agent that receives the capability.
    pub recipient: AgentId,
    /// Scope granted by the capability.
    pub scope: DelegationScope,
    /// Timestamp when the capability expires.
    pub expires_at: DateTime<Utc>,
    /// Parent capability in the provenance chain, when delegated downstream.
    pub parent_capability: Option<CapabilityId>,
    /// Current revocation state for the capability.
    pub revocation_state: RevocationState,
}

/// One authority transfer in a delegation provenance chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceLink {
    /// Principal that issued the delegated capability.
    pub issuer: AuthorityPrincipal,
    /// Agent that received the delegated capability.
    pub recipient: AgentId,
    /// Capability issued for this link.
    pub capability_id: CapabilityId,
    /// Scope granted by this authority transfer.
    pub scope: DelegationScope,
    /// Timestamp when this link expires.
    pub expires_at: DateTime<Utc>,
}

/// Ordered chain of authority transfers attached to privileged execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceChain {
    /// Root authority for the chain.
    pub root_issuer: AuthorityPrincipal,
    /// Ordered delegation links from root to terminal capability.
    pub links: Vec<ProvenanceLink>,
    /// Capability used at execution time.
    pub terminal_capability: CapabilityId,
}
