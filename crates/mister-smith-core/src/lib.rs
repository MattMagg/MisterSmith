#![deny(missing_docs, unsafe_code)]

//! Core types, traits, and error hierarchy for the Mister Smith multi-agent orchestration framework.

mod enums;
mod error;
mod ids;
mod supervision;
mod traits;

// ID newtypes
pub use ids::{AgentId, MessageId, ResourceId, TaskId, ToolId};

// Core enums
pub use enums::{
    AgentAvailability, AgentState, AgentType, MessagePriority, ProcessLifecycle, ShutdownReason,
};

// Supervision types
pub use supervision::{
    BackoffStrategy, EscalationPolicy, RestartPolicy, RestartScope, SupervisionStrategy,
};

// Error hierarchy
pub use error::{
    ActorError, ConfigError, ErrorSeverity, EventError, FrameworkResult, NetworkError,
    PersistenceError, RecoveryStrategy, ResourceError, RuntimeError, SecurityError, StreamError,
    SupervisionError, SystemError, TaskError, ToolError,
};

// Core traits
pub use traits::{
    Actor, Agent, ConnectionStatus, EventPublisher, HealthStatus, Resource, Supervisor,
    SystemEvent, Tool, ToolCapabilities, ToolSchema, Transport, TransportConfig,
};
