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
    MessageId, ProfileSnapshotId, ResourceId, SessionId, TaskId, ToolId,
};

// Shared autonomy value objects
pub use autonomy::{
    AuthorityPrincipal, CapabilityActionKind, ContextBudget, DelegatedAction,
    DelegatedActionPolicy, DelegationCapability, ExternalDelegationEnvelope, GuardDecision,
    GuardEvidence, GuardTarget, InterventionRecord, MetricWindow, OperatorResultPreview,
    ProfileSnapshot, ProofOutcomeClassification, ProvenanceChain, ProvenanceLink,
    ResultProvenanceSummary, SemanticSignal, SessionRetainedResultView, StepEvaluationRecord,
    TaskResultView, TaskShapeClassification, TaskShapeKind, TeamSizingDecision, TopologyPlan,
    TopologyRationale, UnifiedResultEnvelope,
};

// Core enums
pub use enums::{
    AgentAvailability, AgentState, AgentType, BranchRecoveryStrategy, BranchState, BudgetPolicy,
    BudgetScope, CheckpointPolicy, CoordinationPolicy, DelegationScope, DependencyType,
    FailureClass, GraphState, HealthState, InterventionType, MessagePriority, NodeState,
    ProcessLifecycle, ProfileTarget, RepairDirectiveAction, RevocationState, SemanticSignalKind,
    SessionStatus, ShutdownReason, TopologyKind, VerifierVerdict,
};

// Supervision types
pub use supervision::{
    BackoffStrategy, EscalationPolicy, RepairDirective, RestartPolicy, RestartScope,
    SupervisionStrategy,
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
