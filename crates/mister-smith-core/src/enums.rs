//! Core enumeration types for agent state, priority, and classification.

use serde::{Deserialize, Serialize};

/// Message priority levels with explicit discriminants.
///
/// Lower discriminant values represent higher priority.
/// `Critical` (0) is highest; `Bulk` (4) is lowest.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum MessagePriority {
    /// Highest priority — system-critical messages.
    Critical = 0,
    /// High priority — time-sensitive operations.
    High = 1,
    /// Normal priority — standard message processing.
    #[default]
    Normal = 2,
    /// Low priority — background operations.
    Low = 3,
    /// Lowest priority — bulk/batch operations.
    Bulk = 4,
}

/// Lifecycle state machine for Phase 7 agent lifecycle management.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    /// Agent is initializing resources and configuration.
    Initializing,
    /// Agent is actively processing messages.
    Running,
    /// Agent is temporarily paused.
    Paused,
    /// Agent is gracefully shutting down.
    Stopping,
    /// Agent has terminated.
    Terminated,
    /// Agent encountered an error.
    Error,
    /// Agent is restarting after a failure.
    Restarting,
}

/// Transport/runtime availability signal for status channels and heartbeats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentAvailability {
    /// Agent is idle and ready to accept work.
    Idle,
    /// Agent is currently busy processing.
    Busy,
    /// Agent is in an error state.
    Error,
    /// Agent is offline and unreachable.
    Offline,
    /// Agent is in the process of starting.
    Starting,
    /// Agent is in the process of stopping.
    Stopping,
}

/// Classification of agent roles in the framework.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    /// Manages child agent lifecycles.
    Supervisor,
    /// Performs assigned tasks.
    Worker,
    /// Coordinates multi-agent workflows.
    Coordinator,
    /// Observes and reports on system state.
    Monitor,
    /// Creates execution plans.
    Planner,
    /// Carries out planned actions.
    Executor,
    /// Reviews and validates outputs.
    Critic,
    /// Routes messages between agents.
    Router,
    /// Manages persistent memory and context.
    Memory,
}

/// Process lifecycle state machine for the application binary.
///
/// Tracks the overall framework process from startup through shutdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ProcessLifecycle {
    /// Process is starting — loading config, connecting services.
    Starting = 0,
    /// Process is fully initialized and accepting work.
    Ready = 1,
    /// Process is draining — graceful shutdown in progress.
    Draining = 2,
    /// Process has stopped cleanly.
    Stopped = 3,
    /// Process failed to start or encountered an unrecoverable error.
    Failed = 4,
}

/// Reason for process shutdown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShutdownReason {
    /// Shutdown triggered by OS signal (SIGTERM, SIGINT).
    Signal(String),
    /// Shutdown triggered by an unrecoverable error.
    Error(String),
    /// Startup timeout exceeded before reaching Ready state.
    StartupTimeout,
    /// Forced shutdown via second signal during graceful shutdown.
    Forced,
}

/// Lifecycle state for a validated execution graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GraphState {
    /// The graph exists but has not started running.
    Pending,
    /// The graph is actively executing.
    Running,
    /// The graph has a durable checkpoint boundary recorded.
    Checkpointed,
    /// The graph completed successfully.
    Completed,
    /// The graph failed and needs recovery or escalation.
    Failed,
    /// The graph was stopped intentionally.
    Aborted,
}

/// Lifecycle state for an execution node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeState {
    /// The node has not yet been scheduled.
    Pending,
    /// The node is ready to run once capacity is available.
    Ready,
    /// The node is actively executing.
    Running,
    /// The node has a checkpoint boundary recorded.
    Checkpointed,
    /// The node completed successfully.
    Completed,
    /// The node failed.
    Failed,
    /// The node is blocked by dependencies or policy.
    Blocked,
}

/// Lifecycle state for a checkpointable execution branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BranchState {
    /// The branch exists but has not started running.
    Pending,
    /// The branch is actively executing.
    Running,
    /// The branch has a usable checkpoint.
    Checkpointed,
    /// The branch was isolated for targeted recovery.
    Isolated,
    /// The branch completed successfully.
    Completed,
    /// The branch failed.
    Failed,
    /// The branch was reassigned to new execution capacity.
    Reassigned,
}

/// Dependency semantics between execution nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyType {
    /// Downstream execution waits for upstream completion.
    Completion,
    /// Downstream context assembly depends on upstream output.
    Data,
    /// Downstream recovery depends on an upstream checkpoint.
    Checkpoint,
    /// Downstream execution is gated by policy or authorization.
    Policy,
}

/// Execution topology chosen by the topology compiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TopologyKind {
    /// Strictly ordered single-lane execution.
    Sequential,
    /// Independent branches execute concurrently.
    Parallel,
    /// Ordered stages execute with bounded overlap.
    Pipeline,
    /// Subtrees execute with local aggregation.
    Hierarchical,
    /// Mixed topology across graph regions.
    Hybrid,
}

/// Coordination behavior used by the orchestrator for a topology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoordinationPolicy {
    /// Each step waits for the prior step to finish.
    StrictSequence,
    /// Parallel branches synchronize at explicit barriers.
    Barrier,
    /// Results are consumed as they stream.
    Streaming,
    /// Subtrees aggregate before handing results upward.
    HierarchicalReduce,
    /// Mixed coordination rules across the graph.
    Mixed,
}

/// Policy describing when branch checkpoints should be recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CheckpointPolicy {
    /// Checkpoints are created only when explicitly requested.
    Manual,
    /// Checkpoints are created after each node completion.
    OnNodeCompletion,
    /// Checkpoints are created when branch state changes.
    OnBranchChange,
    /// Checkpoints are created on a periodic schedule.
    Periodic,
}

/// Recovery strategy for a failed or degraded branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BranchRecoveryStrategy {
    /// Resume from the latest checkpoint.
    Resume,
    /// Reassign the branch to new capacity.
    Reassign,
    /// Isolate the branch from the rest of the graph.
    Isolate,
    /// Escalate to a supervisor or operator.
    Escalate,
}

/// Scope that a context budget applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BudgetScope {
    /// Budget applies to an agent role.
    Role,
    /// Budget applies to a specific node.
    Node,
    /// Budget applies to a specific branch.
    Branch,
    /// Budget applies to a whole workflow.
    Workflow,
}

/// Policy used when a context budget would be exceeded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BudgetPolicy {
    /// Evict low-priority context first.
    Evict,
    /// Summarize older context.
    Summarize,
    /// Consolidate context into managed memory.
    Consolidate,
    /// Reject the request rather than widening autonomy.
    Reject,
}

/// Entity target for a runtime performance profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProfileTarget {
    /// Profile describes a specific agent.
    Agent,
    /// Profile describes a branch.
    Branch,
    /// Profile describes a topology or graph-level execution mode.
    Topology,
    /// Profile describes a provider dependency.
    Provider,
}

/// Health signal used by routing and supervision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthState {
    /// Operating normally.
    Healthy,
    /// Operating but showing degraded signals.
    Degraded,
    /// Operating state is unhealthy.
    Unhealthy,
    /// Health is not currently known.
    Unknown,
}

/// Step-level or stream-level semantic signal kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SemanticSignalKind {
    /// Output or stream progress has stalled.
    Stalled,
    /// Output is repetitive or looping.
    Repetitive,
    /// Output quality or confidence is degraded.
    LowConfidence,
    /// Required tool or memory context is missing.
    MissingContext,
    /// Policy constraints conflict with the current execution path.
    PolicyConflict,
}

/// Failure taxonomy used by the Guard layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FailureClass {
    /// Transient failure where retry or failover may help.
    Transient,
    /// Structural failure such as invalid config or auth.
    Structural,
    /// Streaming or transport degradation.
    Streaming,
    /// Semantic degradation in reasoning quality.
    Semantic,
}

/// Intervention chosen by the Guard layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InterventionType {
    /// Retry the same work with the current configuration.
    Retry,
    /// Fail over to alternative capacity.
    Failover,
    /// Refresh or narrow the working context.
    ContextRefresh,
    /// Isolate a branch from the wider graph.
    BranchIsolation,
    /// Reassign work to a different execution target.
    Reassignment,
    /// Escalate to a supervisor or operator.
    Escalation,
    /// Abort the affected work entirely.
    Abort,
}

/// Scope granted by a delegation capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DelegationScope {
    /// Execute or resume workflow-level work.
    ExecuteWorkflow,
    /// Manage a specific branch lifecycle.
    ManageBranch,
    /// Refresh or narrow runtime context.
    RefreshContext,
    /// Apply a supervisory intervention.
    ApplyIntervention,
    /// Access managed memory or snapshots.
    AccessMemory,
    /// Invoke a privileged tool or external action.
    InvokeTool,
}

/// Revocation state for a delegation capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RevocationState {
    /// Capability is currently valid.
    Active,
    /// Capability was explicitly revoked.
    Revoked,
    /// Capability expired naturally.
    Expired,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_priority_discriminants() {
        assert_eq!(MessagePriority::Critical as u8, 0);
        assert_eq!(MessagePriority::High as u8, 1);
        assert_eq!(MessagePriority::Normal as u8, 2);
        assert_eq!(MessagePriority::Low as u8, 3);
        assert_eq!(MessagePriority::Bulk as u8, 4);
    }

    #[test]
    fn message_priority_default() {
        assert_eq!(MessagePriority::default(), MessagePriority::Normal);
    }

    #[test]
    fn message_priority_ordering() {
        assert!(MessagePriority::Critical < MessagePriority::High);
        assert!(MessagePriority::High < MessagePriority::Normal);
        assert!(MessagePriority::Normal < MessagePriority::Bulk);
    }

    #[test]
    fn agent_state_serde_roundtrip() {
        let state = AgentState::Running;
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: AgentState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }
}
