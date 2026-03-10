//! # Mister Smith Agents
//!
//! Multi-agent orchestration layer for the Mister Smith framework (Phase 7).
//!
//! Composes Phase 1-6 foundations (actors, supervision, transport, security,
//! persistence) into a coordinated agent runtime with lifecycle management,
//! inter-agent communication, task scheduling, team orchestration, and
//! tool integration.
//!
//! ## Key Components
//!
//! - [`AgentRuntime`] — Actor-Agent bridge wrapping `ActorRef` with lifecycle state
//! - [`AgentRegistry`] — Concurrent agent discovery via `DashMap`
//! - [`TaskScheduler`] — Task state machine with deadline monitoring
//! - [`Orchestrator`] — Decompose → assign → aggregate workflow engine
//! - [`Team`] — Ephemeral supervision subtree for multi-agent collaboration
//! - [`ToolBus`] — Central tool registry with metrics tracking
//! - [`HeartbeatEmitter`] — Periodic liveness signals for failure detection
//!
//! ## Specialized Roles
//!
//! Nine agent role implementations in [`roles`]:
//! Supervisor, Worker, Coordinator, Monitor, Planner, Executor, Critic, Router, Memory.
//!
//! ## Gate 7 Validation
//!
//! End-to-end test: Coordinator decomposes task → Workers execute under Supervisor →
//! failure injection → reassignment → correct aggregated result with no duplicate work.

/// Agent runtime bridge between Actor trait and orchestration layer.
pub mod agent;
/// Configuration types: AgentConfig, HealthLevel, TeamPattern, TaskState.
pub mod config;
/// Error types for the agent system.
pub mod errors;
/// Execution graph contracts and validation.
pub mod execution_graph;
/// Periodic heartbeat emission for liveness detection.
pub mod heartbeat;
/// Inter-agent messaging helpers (send, request, broadcast, durable).
pub mod messaging;
/// Task decomposition and result aggregation orchestration.
pub mod orchestrator;
/// Cross-boundary quarantine inspection actor.
pub mod quarantine;
/// Concurrent agent registry with discovery queries.
pub mod registry;
/// Nine specialized agent role implementations.
pub mod roles;
/// Agent sandbox integration for lifecycle-scoped credentials.
pub mod sandbox;
/// Task scheduling with state machine and deadline monitoring.
pub mod scheduler;
/// Ephemeral team management for multi-agent collaboration.
pub mod team;
/// Central tool registry with native and MCP tool support.
pub mod tool_bus;
/// Deterministic topology compilation for execution graphs.
pub mod topology;

// Re-exports
pub use agent::AgentRuntime;
pub use config::AgentConfig;
pub use errors::AgentSystemError;
pub use execution_graph::{
    BranchCheckpoint, ExecutionBranch, ExecutionEdge, ExecutionGraph, ExecutionNode,
};
pub use heartbeat::HeartbeatEmitter;
pub use messaging::{broadcast, request, send, send_durable};
pub use orchestrator::Orchestrator;
pub use quarantine::{QuarantineActor, QuarantineTransfer, SharedStateAccess};
pub use registry::AgentRegistry;
pub use sandbox::{AgentSandbox, SandboxedAgentRuntime};
pub use scheduler::{DeadlineMonitor, ResultAggregator, TaskDecomposer, TaskScheduler};
pub use team::Team;
pub use tool_bus::ToolBus;
pub use topology::{TopologyCompiler, TopologySignals};
