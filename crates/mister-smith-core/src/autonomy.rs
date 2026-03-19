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
    ExecutionNodeId, GuardDecisionId, InterventionRecordId, ProfileSnapshotId, TaskId,
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

/// Operator-visible sizing decision for one workflow or frontier transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamSizingDecision {
    /// Workflow that owns the decision.
    pub workflow_id: TaskId,
    /// Graph the decision applies to.
    pub graph_id: ExecutionGraphId,
    /// Decision phase for this slice, e.g. `initial` or `frontier_rebalance`.
    pub decision_phase: String,
    /// Width implied by structure before caps.
    pub desired_workers: usize,
    /// Final worker count after caps.
    pub selected_workers: usize,
    /// Workers available to the runtime at decision time.
    pub available_workers: usize,
    /// Frontier width that shaped the decision.
    pub branch_frontier_width: usize,
    /// Depth signal that shaped the decision.
    pub dependency_depth: usize,
    /// Whether the runtime narrowed posture conservatively.
    pub conservative_mode: bool,
    /// Budget-pressure signal used for capping when present.
    pub budget_pressure: Option<u8>,
    /// Main explanation when `selected_workers < desired_workers`.
    pub cap_reason: Option<String>,
    /// Operator-visible explanation of the decision.
    pub rationale_lines: Vec<String>,
    /// Decision timestamp.
    pub decided_at: DateTime<Utc>,
}

/// Stable proof classification shared across task, session, and operator result surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofOutcomeClassification {
    /// A real graph formed and completed successfully.
    GraphFormedAndCompleted,
    /// The workflow completed, but the planner collapsed it to a trivial sequential path.
    CollapsedToSequential,
    /// The workflow failed before a usable graph outcome existed.
    FailedBeforeGraph,
}

impl ProofOutcomeClassification {
    /// Return the stable contract label for this proof classification.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GraphFormedAndCompleted => "graph_formed_and_completed",
            Self::CollapsedToSequential => "collapsed_to_sequential",
            Self::FailedBeforeGraph => "failed_before_graph",
        }
    }
}

/// Canonical runtime result contract rooted at metadata `final_result`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnifiedResultEnvelope {
    /// Workflow that produced the result.
    pub workflow_id: TaskId,
    /// Provider path used for the run.
    pub provider_kind: String,
    /// Model used for the run.
    pub model_id: String,
    /// Workflow request summary or description.
    pub description: String,
    /// Existing runtime execution markers such as supervised actor and tool bus.
    pub runtime_execution_mode: Value,
    /// Existing planner output captured by the runtime.
    pub planner_output: Value,
    /// Existing normalized execution plan captured by the runtime.
    pub execution_plan: Value,
    /// Existing per-step results captured by the runtime.
    pub step_results: Vec<Value>,
    /// Execution-produced payload nested inside the canonical result object.
    pub aggregated_result: Value,
    /// Proof outcome classification for the run.
    pub proof_outcome: ProofOutcomeClassification,
}

/// Task-facing result envelope exposed through `task.result`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskResultView {
    /// Workflow identifier shown on the task surface.
    pub workflow_id: TaskId,
    /// Terminal workflow status.
    pub status: String,
    /// Canonical result envelope exposed by the task surface.
    pub result: UnifiedResultEnvelope,
    /// Task-facing outcome classification.
    pub proof_outcome: ProofOutcomeClassification,
}

/// Shared provenance summary reused by session and operator result projections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultProvenanceSummary {
    /// Execution-mode summary from the canonical result object.
    pub runtime_execution_mode: Value,
    /// Graph state if a graph was formed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_state: Option<String>,
    /// Graph identifier when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_id: Option<String>,
    /// Canonical fields used to derive the projection.
    pub source_fields: Vec<String>,
}

/// Retained session-facing projection stored as `assistant_result`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRetainedResultView {
    /// Workflow linked to the retained turn.
    pub workflow_id: TaskId,
    /// Session turn that owns the projection.
    pub turn_index: u32,
    /// Turn or workflow status.
    pub status: String,
    /// Session-facing projection derived from the canonical result object.
    pub assistant_result: Value,
    /// Compact preview extracted from the canonical result object when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
    /// Bounded provenance summary for retained context.
    pub provenance: ResultProvenanceSummary,
}

/// Compact operator-facing result preview rendered alongside autonomy status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorResultPreview {
    /// Workflow being inspected.
    pub workflow_id: TaskId,
    /// Outcome classification visible to operators.
    pub proof_outcome: ProofOutcomeClassification,
    /// Bounded result preview, omitted when not safe or available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview_text: Option<String>,
    /// Where the full result comes from, for example `task.result`.
    pub payload_location: String,
    /// Compact explanation of how the result was produced and classified.
    pub provenance_lines: Vec<String>,
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

/// Typed privileged action that a capability descriptor can expose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityActionKind {
    /// Discover that a capability surface exists.
    Discover,
    /// Execute a privileged action through the capability surface.
    Execute,
}

impl CapabilityActionKind {
    /// Return the canonical policy action string for this action kind.
    #[must_use]
    pub fn policy_action(self) -> &'static str {
        match self {
            Self::Discover => "discover",
            Self::Execute => "execute",
        }
    }
}

/// Policy binding associated with a typed delegated action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegatedActionPolicy {
    /// Policy action string evaluated by the RBAC engine.
    pub action: String,
    /// Policy resource string evaluated by the RBAC engine.
    pub resource: String,
    /// Policy scope string evaluated by the RBAC engine.
    pub scope: String,
    /// Optional concrete resource identifier bound to the action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
}

/// Typed delegated action bound to a capability descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegatedAction {
    /// Stable descriptor identifier this action belongs to.
    pub descriptor_id: String,
    /// Stable identifier for the action within the descriptor.
    pub action_id: String,
    /// Human-readable title for the action.
    pub title: String,
    /// Human-readable description for the action.
    pub description: String,
    /// Typed action kind.
    pub kind: CapabilityActionKind,
    /// Policy binding evaluated before execution.
    pub policy: DelegatedActionPolicy,
    /// Delegation scope required for privileged execution, when any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_scope: Option<DelegationScope>,
    /// Stable revocation key for action-level revocation.
    pub revocation_key: String,
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
    /// Capability descriptor this authority is bound to, when narrowed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descriptor_id: Option<String>,
    /// Parent capability in the provenance chain, when delegated downstream.
    pub parent_capability: Option<CapabilityId>,
    /// Current revocation state for the capability.
    pub revocation_state: RevocationState,
}

/// Transport-safe envelope for delegated authority crossing external boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalDelegationEnvelope {
    /// Capability used at the destination boundary.
    pub capability: DelegationCapability,
    /// Ordered authority lineage for the capability.
    pub provenance: ProvenanceChain,
    /// Specific delegated action carried across the boundary, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<DelegatedAction>,
}

impl ExternalDelegationEnvelope {
    /// Create an external delegation envelope from a capability and provenance chain.
    #[must_use]
    pub fn new(capability: DelegationCapability, provenance: ProvenanceChain) -> Self {
        Self {
            capability,
            provenance,
            action: None,
        }
    }

    /// Attach a delegated action to the envelope.
    #[must_use]
    pub fn with_action(mut self, action: DelegatedAction) -> Self {
        self.action = Some(action);
        self
    }

    /// Return the descriptor ID carried by the envelope, if any.
    #[must_use]
    pub fn descriptor_id(&self) -> Option<&str> {
        self.action
            .as_ref()
            .map(|action| action.descriptor_id.as_str())
            .or(self.capability.descriptor_id.as_deref())
    }
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
    /// Capability descriptor this authority transfer is bound to, when narrowed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descriptor_id: Option<String>,
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
