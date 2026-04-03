#![deny(missing_docs, unsafe_code)]

//! Core types, traits, and error hierarchy for the Mister Smith multi-agent orchestration framework.

mod autonomy;
mod enums;
mod error;
mod ids;
mod supervision;
mod traits;

// ID newtypes
pub use ids::{
    AgentId, CapabilityId, CheckpointId, ContextBudgetId, ExecutionBranchId, ExecutionGraphId,
    ExecutionNodeId, GuardDecisionId, InterventionRecordId, MemoryFragmentId, MemorySnapshotId,
    MessageId, ProfileFingerprintId, ProfileSnapshotId, ResourceId, SessionId, TaskId, ToolId,
};

// Shared autonomy value objects
pub use autonomy::{
    packet_023_placeholder_proof_boundary, packet_023_placeholder_runtime_truth,
    AuthorityPrincipal, CapabilityActionKind, ContextBudget, DelegatedAction,
    DelegatedActionPolicy, DelegationCapability, ExecutionEvidenceClass,
    ExternalDelegationEnvelope, GroundedEvidenceReference, GuardDecision, GuardEvidence,
    GuardTarget, HandoffClarificationRequest, InterventionRecord, MetricWindow,
    OperatorResultPreview, OrchestrationQualityView, ProfileFingerprint, ProfileFingerprintRef,
    ProfileSnapshot, ProofBoundaryView, ProofOutcomeClassification, ProvenanceChain,
    ProvenanceLink, RepairLineageRef, ResultProvenanceSummary, RunTraceRelationshipKind,
    RunTraceSummaryView, RuntimeTruthView, SemanticSignal, SessionRetainedResultView,
    StepBudgetPressureLevel, StepBudgetPressureSummary, StepDifficultyAssessment,
    StepDifficultyBucket, StepEvaluationRecord, StepPolicyAction, StepPolicyConfidenceLabel,
    StepPolicyDecision, StepPolicyInputRefs, StepPolicyProofBoundaryRef, StepPolicySummaryView,
    SupervisionDecisionBasis, SupervisionEvidenceView, SupervisionTargetKind,
    SupervisionTargetScope, TaskResultView, TaskShapeClassification, TaskShapeKind,
    TeamSizingDecision, TopologyPlan, TopologyRationale, UnifiedResultEnvelope,
    PACKET_023_GRAPH_EXECUTION_SUCCESS, PACKET_023_GROUNDED_TOOL_EXECUTION_MINIMAL,
    PACKET_023_ORCHESTRATION_ONLY, PACKET_023_SEMANTIC_COMPLETION_UNPROVEN,
};

// Core enums
pub use enums::{
    AgentAvailability, AgentState, AgentType, BranchRecoveryStrategy, BranchState, BudgetPolicy,
    BudgetScope, CheckpointPolicy, CoordinationPolicy, DelegationScope, DependencyType,
    DurableWorkflowEventKind, DurableWorkflowLifecycleState, DurableWorkflowLifecycleVerb,
    EffectBoundaryIntentState, EffectBoundaryOutcomeState, FailureClass, GraphState, HealthState,
    HistoryCompactionMode, InterventionType, LifecycleDecisionOutcome, MessagePriority, NodeState,
    ProcessLifecycle, ProfileTarget, RepairDirectiveAction, RevocationState, SemanticSignalKind,
    SessionStatus, ShutdownReason, TopologyKind, VerifierVerdict,
};

// Supervision types
pub use supervision::{
    BackoffStrategy, EscalationPolicy, FailureContextCheckpoint, RepairDirective, RestartPolicy,
    RestartScope, SupervisionStrategy,
};

// Error hierarchy
pub use error::{
    ActorError, AutonomyError, ConfigError, DelegationError, ErrorSeverity, EventError,
    FrameworkResult, GuardError, LlmError, MemoryError, NetworkError, PersistenceError,
    RecoveryStrategy, ResourceError, RuntimeError, SecurityError, StreamError, SupervisionError,
    SystemError, TaskError, ToolError, TopologyError,
};

// Core traits
pub use traits::{
    Actor, Agent, ConnectionStatus, EventPublisher, HealthStatus, Resource, ResourceConfig,
    Supervisor, SystemEvent, Tool, ToolCapabilities, ToolSchema, Transport, TransportConfig,
};
