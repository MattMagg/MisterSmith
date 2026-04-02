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
    InterventionType, ProfileTarget, RepairDirectiveAction, RevocationState, SemanticSignalKind,
    TopologyKind, VerifierVerdict,
};
use crate::ids::{
    AgentId, CapabilityId, CheckpointId, ContextBudgetId, ExecutionBranchId, ExecutionGraphId,
    ExecutionNodeId, GuardDecisionId, InterventionRecordId, ProfileFingerprintId,
    ProfileSnapshotId, TaskId,
};
use crate::supervision::{FailureContextCheckpoint, RepairDirective};

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
    /// Frozen proof-outcome matrix for packet 015 before runtime proof-path work expands.
    pub const ALL: [Self; 3] = [
        Self::GraphFormedAndCompleted,
        Self::CollapsedToSequential,
        Self::FailedBeforeGraph,
    ];

    /// Return the stable contract label for this proof classification.
    #[must_use]
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

/// Execution-owned evaluation of one workflow step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepEvaluationRecord {
    /// Workflow that owns the evaluated step.
    pub workflow_id: TaskId,
    /// Stable identifier for the evaluated step or handoff.
    pub step_id: String,
    /// Verifier verdict for the step.
    pub verdict: VerifierVerdict,
    /// Optional bounded confidence score from the verifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    /// Human-readable explanation for the verdict.
    pub reason: String,
    /// Optional structured failure or deficiency code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_code: Option<String>,
    /// Optional reference to the last stable checkpoint used for repair.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_ref: Option<String>,
    /// Optional bounded repair directive retained for repair-loop provenance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repair_directive: Option<RepairDirective>,
    /// Optional first-class clarification request for weak or incomplete handoffs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clarification_request: Option<HandoffClarificationRequest>,
    /// Optional preserved failure context and checkpoint for local repair.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_context_checkpoint: Option<FailureContextCheckpoint>,
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

/// Operator-facing projection of verifier and repair history for one workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrchestrationQualityView {
    /// Stable identifier for the latest evaluated workflow step or handoff.
    pub step_id: String,
    /// Final verifier verdict visible to operators.
    pub verdict: VerifierVerdict,
    /// Bounded repair action when a repair path was taken.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repair_action: Option<RepairDirectiveAction>,
    /// Number of clarification attempts retained for the rendered step.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub clarification_attempt_count: u32,
    /// Stable checkpoint reference anchoring the repair branch when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_ref: Option<String>,
    /// Last stable upstream step retained for local repair when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_stable_step_id: Option<String>,
    /// Stable failure-context reference retained across repair attempts when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_context_ref: Option<String>,
    /// Bounded operator-facing summary of the final repair-loop outcome.
    pub outcome_summary: String,
}

/// First-class clarification request emitted for a weak handoff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffClarificationRequest {
    /// Step that produced the incomplete handoff.
    pub source_step_id: String,
    /// Step that is blocked waiting for clarification.
    pub target_step_id: String,
    /// Explicit constraints or assumptions that must be clarified.
    pub missing_constraints: Vec<String>,
    /// Number of clarification attempts already consumed.
    pub attempt_count: u32,
    /// Optional guardrail for abandoning stale clarification loops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

/// Canonical packet-023 proof-boundary wording for placeholder step runs.
pub const PACKET_023_GRAPH_EXECUTION_SUCCESS: &str = "workflow graph executed successfully";
/// Canonical packet-023 semantic-completion wording for placeholder step runs.
pub const PACKET_023_SEMANTIC_COMPLETION_UNPROVEN: &str = "semantic completion not yet proven";
/// Canonical packet-023 grounded-tool wording for placeholder step runs.
pub const PACKET_023_GROUNDED_TOOL_EXECUTION_MINIMAL: &str =
    "grounded tool execution: none/minimal";
/// Canonical packet-023 task-proof wording for placeholder step runs.
pub const PACKET_023_ORCHESTRATION_ONLY: &str =
    "result is orchestration proof, not substantive task proof";

/// Strongest execution-evidence class the runtime can honestly claim for one run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionEvidenceClass {
    /// The runtime proved workflow-substrate completion only.
    #[serde(rename = "orchestration_substrate_completion")]
    SubstrateCompletion,
    /// The runtime completed a placeholder or simulated step boundary.
    PlaceholderOrSimulatedStepCompletion,
    /// The runtime reached a grounded tool boundary with real external evidence.
    GroundedToolExecution,
    /// The runtime proved grounded task completion.
    GroundedTaskProof,
}

impl ExecutionEvidenceClass {
    /// Return the stable contract label for this evidence class.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SubstrateCompletion => "orchestration_substrate_completion",
            Self::PlaceholderOrSimulatedStepCompletion => {
                "placeholder_or_simulated_step_completion"
            }
            Self::GroundedToolExecution => "grounded_tool_execution",
            Self::GroundedTaskProof => "grounded_task_proof",
        }
    }
}

/// Bounded relationship kinds surfaced by the packet-023 run-trace taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunTraceRelationshipKind {
    /// Graph-level workflow execution exists for the run.
    Graph,
    /// Branch-level execution exists for the run.
    Branch,
    /// Node-level execution exists for the run.
    Node,
    /// A tool boundary was crossed during execution.
    ToolBoundary,
    /// A handoff relationship occurred during execution.
    Handoff,
    /// A repair relationship occurred during execution.
    Repair,
    /// A retry relationship occurred during execution.
    Retry,
    /// The run fanned out into multiple execution paths.
    FanOut,
    /// The run rejoined after fan-out.
    Join,
    /// Supervision state attached to the run.
    Supervision,
}

/// Stable reference to real grounded evidence touched during a run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroundedEvidenceReference {
    /// Stable evidence class such as `file`, `endpoint`, `artifact`, or `checkpoint`.
    #[serde(rename = "kind", alias = "source")]
    pub kind: String,
    /// Stable identifier, path, URL, or artifact key.
    pub reference: String,
    /// Short human-readable explanation for the evidence.
    #[serde(
        rename = "label",
        alias = "detail",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub label: Option<String>,
}

/// Shared packet-023 run-trace summary anchored to one workflow run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunTraceSummaryView {
    /// Canonical trace root identifier for the run.
    pub trace_root_id: String,
    /// Workflow that owns the run trace.
    pub workflow_id: TaskId,
    /// Graph identifier when one exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_id: Option<ExecutionGraphId>,
    /// Branch identifier when one exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_id: Option<ExecutionBranchId>,
    /// Node identifier when one exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<ExecutionNodeId>,
    /// Observed bounded run-trace relationship kinds for the run.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<RunTraceRelationshipKind>,
}

/// Packet-023 proof-boundary projection rendered across operator surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofBoundaryView {
    /// Honest statement about workflow-graph execution.
    pub graph_execution: String,
    /// Honest statement about semantic task completion.
    pub semantic_completion: String,
    /// Honest statement about grounded tool execution.
    pub grounded_tool_execution: String,
    /// Honest statement about substantive task proof.
    pub task_proof: String,
}

/// Shared packet-023 runtime-truth projection carried across result and status surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeTruthView {
    /// Strongest evidence class the runtime can honestly claim for the run.
    pub evidence_class: ExecutionEvidenceClass,
    /// Shared proof-boundary wording for the run.
    pub proof_boundary: ProofBoundaryView,
    /// Bounded run-trace summary for the run.
    pub run_trace: RunTraceSummaryView,
    /// Stable grounded evidence refs when real evidence exists.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grounded_evidence: Vec<GroundedEvidenceReference>,
}

/// Return the canonical packet-023 placeholder-step proof-boundary wording.
#[must_use]
pub fn packet_023_placeholder_proof_boundary() -> ProofBoundaryView {
    ProofBoundaryView {
        graph_execution: PACKET_023_GRAPH_EXECUTION_SUCCESS.to_string(),
        semantic_completion: PACKET_023_SEMANTIC_COMPLETION_UNPROVEN.to_string(),
        grounded_tool_execution: PACKET_023_GROUNDED_TOOL_EXECUTION_MINIMAL.to_string(),
        task_proof: PACKET_023_ORCHESTRATION_ONLY.to_string(),
    }
}

/// Build the canonical packet-023 runtime-truth block for a placeholder-step run.
#[must_use]
pub fn packet_023_placeholder_runtime_truth(
    workflow_id: TaskId,
    graph_id: Option<ExecutionGraphId>,
    branch_id: Option<ExecutionBranchId>,
    node_id: Option<ExecutionNodeId>,
    relationships: Vec<RunTraceRelationshipKind>,
    grounded_evidence: Vec<GroundedEvidenceReference>,
) -> RuntimeTruthView {
    RuntimeTruthView {
        evidence_class: ExecutionEvidenceClass::PlaceholderOrSimulatedStepCompletion,
        proof_boundary: packet_023_placeholder_proof_boundary(),
        run_trace: RunTraceSummaryView {
            trace_root_id: workflow_id.to_string(),
            workflow_id,
            graph_id,
            branch_id,
            node_id,
            relationships,
        },
        grounded_evidence,
    }
}

/// Task-facing result envelope exposed through `task.result`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskResultView {
    /// Workflow identifier shown on the task surface.
    pub workflow_id: TaskId,
    /// Terminal workflow status.
    pub status: String,
    /// Task-facing outcome classification.
    pub proof_outcome: ProofOutcomeClassification,
    /// Bounded verifier and repair provenance when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orchestration_quality: Option<OrchestrationQualityView>,
    /// Packet-023 runtime-truth projection when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_truth: Option<RuntimeTruthView>,
    /// Bounded packet-021 predictive-supervision evidence when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supervision_evidence: Option<SupervisionEvidenceView>,
    /// Canonical result envelope exposed by the task surface.
    pub result: UnifiedResultEnvelope,
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
    /// Packet-023 runtime-truth projection when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_truth: Option<RuntimeTruthView>,
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
    /// Bounded verifier and repair provenance when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orchestration_quality: Option<OrchestrationQualityView>,
    /// Packet-023 runtime-truth projection when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_truth: Option<RuntimeTruthView>,
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

/// Stable runtime scope encoded on the frozen packet-021 supervision surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupervisionTargetKind {
    /// Pre-graph provider-local supervision.
    Provider,
    /// Graph-wide supervision posture.
    Graph,
    /// Branch-local supervision posture.
    Branch,
    /// Node-local supervision posture.
    Node,
}

/// Canonical packet-021 target scope shared across core, events, and orchestrator surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupervisionTargetScope {
    /// Coarse supervision target kind.
    pub kind: SupervisionTargetKind,
    /// Provider label when supervision is still pre-graph and provider-scoped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Graph identifier when graph context exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_id: Option<ExecutionGraphId>,
    /// Branch identifier when branch scope exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_id: Option<ExecutionBranchId>,
    /// Node identifier when node scope exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<ExecutionNodeId>,
}

/// Persisted advisory fingerprint keyed to a supported runtime role or target class.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileFingerprint {
    /// Stable identifier for the persisted fingerprint.
    pub fingerprint_id: ProfileFingerprintId,
    /// Supported runtime role or target class, e.g. `planner` or `provider`.
    pub target_kind: String,
    /// Stable selector for the profiled runtime target.
    pub target_selector: String,
    /// Replayable run, checkpoint, or fixture references used to build the fingerprint.
    pub source_refs: Vec<String>,
    /// Structured supervisory summary only; no duplicated raw transcript bodies.
    pub summary_payload: Value,
    /// Ordered recurring failure tendencies.
    pub dominant_failure_modes: Vec<String>,
    /// Ordered preferred interventions suggested by the fingerprint.
    pub preferred_interventions: Vec<InterventionType>,
    /// Bounded confidence score for advisory use.
    pub confidence: f32,
    /// Timestamp after which the fingerprint is no longer trusted automatically.
    pub expires_at: DateTime<Utc>,
    /// Last refresh timestamp.
    pub updated_at: DateTime<Utc>,
}

/// Lightweight reference to a current fingerprint used during supervision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileFingerprintRef {
    /// Stable identifier for the referenced fingerprint.
    pub fingerprint_id: ProfileFingerprintId,
    /// Operator-visible fingerprint key or selector label.
    pub fingerprint_key: String,
    /// Advisory confidence score.
    pub confidence: f32,
    /// Timestamp when the fingerprint reference expires.
    pub expires_at: DateTime<Utc>,
}

/// How the latest supervision decision was formed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupervisionDecisionBasis {
    /// Live signals alone were sufficient for the decision.
    LiveSignalsOnly,
    /// A current fingerprint reinforced the live evidence.
    FingerprintReinforced,
    /// Conservative fallback was used because higher-fidelity evidence was unavailable.
    ConservativeFallback,
}

impl SupervisionDecisionBasis {
    /// Return the stable packet-021 label for this basis.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LiveSignalsOnly => "live_signals_only",
            Self::FingerprintReinforced => "fingerprint_reinforced",
            Self::ConservativeFallback => "conservative_fallback",
        }
    }
}

/// Lightweight pointer back to packet-020 verifier and repair lineage when present.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepairLineageRef {
    /// Stable source packet label for the linked lineage.
    pub source: String,
    /// Packet-020 checkpoint reference when the runtime already knows it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_ref: Option<String>,
}

/// Telemetry and performance state used by routing and supervision decisions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Current fingerprint reference that reinforced this live snapshot, when any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint_ref: Option<ProfileFingerprintRef>,
    /// Timestamp when the profile snapshot was captured.
    pub updated_at: DateTime<Utc>,
}

/// Evidence used by the Guard layer when making a supervisory decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardEvidence {
    /// Optional profile snapshot that informed the decision.
    pub profile_id: Option<ProfileSnapshotId>,
    /// Frozen packet-021 explanation of whether live or fingerprint evidence drove the decision.
    pub decision_basis: SupervisionDecisionBasis,
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

/// Canonical packet-021 supervision projection shared across task, autonomy, and run-detail surfaces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupervisionEvidenceView {
    /// Frozen target scope for the latest supervision state.
    pub target_scope: SupervisionTargetScope,
    /// Current advisory fingerprint reference when one reinforced the decision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint_ref: Option<ProfileFingerprintRef>,
    /// Latest live profile snapshot when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_snapshot: Option<ProfileSnapshot>,
    /// Latest typed Guard decision when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guard_decision: Option<GuardDecision>,
    /// Latest applied intervention record when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intervention_record: Option<InterventionRecord>,
    /// Human-readable summary of how the decision was formed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_basis: Option<String>,
    /// Linked packet-020 repair lineage when the runtime already has it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repair_lineage_ref: Option<RepairLineageRef>,
    /// Explicit proof-boundary note when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_boundary: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RepairDirectiveAction, TaskId};
    use serde_json::json;

    #[test]
    fn step_evaluation_record_serde_roundtrip() {
        let record = StepEvaluationRecord {
            workflow_id: TaskId::new(),
            step_id: "draft-outline".to_string(),
            verdict: VerifierVerdict::Rejected,
            confidence: Some(0.42),
            reason: "missing cost constraint in handoff".to_string(),
            failure_code: Some("missing_constraint".to_string()),
            checkpoint_ref: Some("checkpoint-1".to_string()),
            repair_directive: Some(RepairDirective {
                action: RepairDirectiveAction::ClarifyHandoff,
                issued_by: "verifier.runtime".to_string(),
                failure_context_ref: "draft-outline/missing-cost".to_string(),
                retry_budget_remaining: 1,
            }),
            clarification_request: Some(HandoffClarificationRequest {
                source_step_id: "draft-outline".to_string(),
                target_step_id: "write-brief".to_string(),
                missing_constraints: vec!["budget ceiling".to_string()],
                attempt_count: 1,
                expires_at: Some(Utc::now()),
            }),
            failure_context_checkpoint: Some(FailureContextCheckpoint {
                failed_step_id: "draft-outline".to_string(),
                last_stable_step_id: Some("collect-evidence".to_string()),
                checkpoint_ref: Some("checkpoint-1".to_string()),
                failure_context_ref: "draft-outline/missing-cost".to_string(),
                failure_code: Some("missing_constraint".to_string()),
                reason: "missing cost constraint in handoff".to_string(),
                attempt_count: 1,
            }),
        };

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: StepEvaluationRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, record);
    }

    #[test]
    fn orchestration_quality_view_serde_roundtrip() {
        let summary = OrchestrationQualityView {
            step_id: "draft-outline".to_string(),
            verdict: VerifierVerdict::Accepted,
            repair_action: Some(RepairDirectiveAction::ClarifyHandoff),
            clarification_attempt_count: 1,
            checkpoint_ref: Some("checkpoint-clarify".to_string()),
            last_stable_step_id: Some("collect-evidence".to_string()),
            failure_context_ref: Some("draft-outline/clarify".to_string()),
            outcome_summary: "accepted_after_clarify_handoff".to_string(),
        };

        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: OrchestrationQualityView = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, summary);
    }

    #[test]
    fn task_result_view_serde_roundtrip_preserves_supervision_evidence() {
        let workflow_id = TaskId::new();
        let runtime_truth = packet_023_placeholder_runtime_truth(
            workflow_id,
            Some(ExecutionGraphId::new()),
            Some(ExecutionBranchId::new()),
            Some(ExecutionNodeId::new()),
            vec![
                RunTraceRelationshipKind::Graph,
                RunTraceRelationshipKind::Branch,
                RunTraceRelationshipKind::ToolBoundary,
                RunTraceRelationshipKind::Supervision,
            ],
            vec![],
        );
        let task_result = TaskResultView {
            workflow_id,
            status: "completed".to_string(),
            proof_outcome: ProofOutcomeClassification::GraphFormedAndCompleted,
            orchestration_quality: Some(OrchestrationQualityView {
                step_id: "finalize".to_string(),
                verdict: VerifierVerdict::Accepted,
                repair_action: Some(RepairDirectiveAction::ClarifyHandoff),
                clarification_attempt_count: 1,
                checkpoint_ref: Some("checkpoint-clarify".to_string()),
                last_stable_step_id: Some("collect-evidence".to_string()),
                failure_context_ref: Some("draft-outline/clarify".to_string()),
                outcome_summary: "accepted_after_clarify_handoff".to_string(),
            }),
            runtime_truth: Some(runtime_truth),
            supervision_evidence: Some(SupervisionEvidenceView {
                target_scope: SupervisionTargetScope {
                    kind: SupervisionTargetKind::Branch,
                    provider: None,
                    graph_id: Some(ExecutionGraphId::new()),
                    branch_id: Some(ExecutionBranchId::new()),
                    node_id: None,
                },
                fingerprint_ref: None,
                profile_snapshot: None,
                guard_decision: None,
                intervention_record: None,
                decision_basis: Some("live_signals_only".to_string()),
                repair_lineage_ref: Some(RepairLineageRef {
                    source: "packet-020".to_string(),
                    checkpoint_ref: Some("checkpoint-clarify".to_string()),
                }),
                proof_boundary: Some("deterministic-only".to_string()),
            }),
            result: UnifiedResultEnvelope {
                workflow_id,
                provider_kind: "openai_chatgpt".to_string(),
                model_id: "gpt-5.4".to_string(),
                description: "task result roundtrip".to_string(),
                runtime_execution_mode: json!({
                    "execution_boundary": "tool_bus",
                }),
                planner_output: json!({"goal": "roundtrip"}),
                execution_plan: json!({"steps": []}),
                step_results: vec![],
                aggregated_result: json!({"ok": true}),
                proof_outcome: ProofOutcomeClassification::GraphFormedAndCompleted,
            },
        };

        let json = serde_json::to_string(&task_result).unwrap();
        let deserialized: TaskResultView = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, task_result);
    }

    #[test]
    fn packet_023_placeholder_runtime_truth_uses_canonical_wording() {
        let workflow_id = TaskId::new();
        let graph_id = ExecutionGraphId::new();
        let runtime_truth = packet_023_placeholder_runtime_truth(
            workflow_id,
            Some(graph_id),
            Some(ExecutionBranchId::new()),
            Some(ExecutionNodeId::new()),
            vec![
                RunTraceRelationshipKind::Graph,
                RunTraceRelationshipKind::Branch,
                RunTraceRelationshipKind::Node,
                RunTraceRelationshipKind::ToolBoundary,
            ],
            vec![],
        );

        assert_eq!(
            runtime_truth.evidence_class,
            ExecutionEvidenceClass::PlaceholderOrSimulatedStepCompletion
        );
        assert_eq!(
            runtime_truth.proof_boundary,
            ProofBoundaryView {
                graph_execution: PACKET_023_GRAPH_EXECUTION_SUCCESS.to_string(),
                semantic_completion: PACKET_023_SEMANTIC_COMPLETION_UNPROVEN.to_string(),
                grounded_tool_execution: PACKET_023_GROUNDED_TOOL_EXECUTION_MINIMAL.to_string(),
                task_proof: PACKET_023_ORCHESTRATION_ONLY.to_string(),
            }
        );
        assert_eq!(
            runtime_truth.run_trace.trace_root_id,
            workflow_id.to_string()
        );
        assert_eq!(runtime_truth.run_trace.workflow_id, workflow_id);
        assert_eq!(runtime_truth.run_trace.graph_id, Some(graph_id));
        assert!(runtime_truth
            .run_trace
            .relationships
            .contains(&RunTraceRelationshipKind::ToolBoundary));
        assert!(runtime_truth.grounded_evidence.is_empty());
    }
}
