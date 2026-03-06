use mister_smith_core::{ActorError, ToolError};
use thiserror::Error;

/// Errors produced by the agent system.
#[derive(Debug, Error)]
pub enum AgentSystemError {
    #[error("Agent spawn failed: {0}")]
    SpawnFailed(String),

    #[error("Message delivery failed: {0}")]
    MessageDeliveryFailed(String),

    #[error("Registry error: {0}")]
    RegistryError(String),

    #[error("Scheduling error: {0}")]
    SchedulingError(String),

    #[error("Orchestration error: {0}")]
    OrchestrationError(String),

    #[error("Tool bus error: {0}")]
    ToolBusError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Operation timed out: {0}")]
    Timeout(String),

    #[error("Tool unavailable: {0}")]
    ToolUnavailable(String),

    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Team error: {0}")]
    TeamError(String),

    #[error("Actor error: {0}")]
    Actor(#[from] ActorError),

    #[error("Tool error: {0}")]
    Tool(#[from] ToolError),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
