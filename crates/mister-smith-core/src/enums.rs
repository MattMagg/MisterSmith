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
