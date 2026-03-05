//! Core trait definitions: Actor, Tool, Agent, Resource, Supervisor, Transport, EventPublisher.
//!
//! These are interface contracts only — no implementations are provided in Phase 1.
//! Each trait defines the extension point for downstream crates to implement.

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::Duration;

use crate::enums::AgentType;
use crate::error::{EventError, NetworkError, ToolError};
use crate::ids::{AgentId, ResourceId, ToolId};
use crate::supervision::{EscalationPolicy, RestartPolicy, SupervisionStrategy};

// ---------------------------------------------------------------------------
// Placeholder types (expanded in later phases)
// ---------------------------------------------------------------------------

/// Tool schema descriptor (placeholder — expanded in Phase 7).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolSchema;

/// Tool capability descriptor (placeholder — expanded in Phase 7).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolCapabilities;

/// Health status of a resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Resource is fully operational.
    Healthy,
    /// Resource is operational but degraded.
    Degraded,
    /// Resource is not operational.
    Unhealthy,
    /// Resource health is unknown.
    Unknown,
}

/// Transport connection status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    /// Connected and operational.
    Connected,
    /// Disconnected.
    Disconnected,
    /// Attempting to reconnect.
    Reconnecting,
}

/// Minimal transport configuration (placeholder — full definition in Phase 4).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransportConfig {
    /// Connection URL.
    pub url: Option<String>,
}

/// System event for the event publisher trait.
///
/// Uses `serde_json::Value` as payload to avoid pulling full event type
/// definitions into the core crate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    /// Event type identifier.
    pub event_type: String,
    /// Event payload as JSON.
    pub payload: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Core Traits
// ---------------------------------------------------------------------------

/// Core actor behavior with message type safety.
///
/// Actors handle messages, maintain state, and participate in supervision trees.
/// Lifecycle hooks (`pre_start`, `post_stop`) allow setup and cleanup.
#[async_trait]
pub trait Actor: Send + 'static {
    /// The message type this actor can handle.
    type Message: Send + 'static;
    /// The state type maintained by this actor.
    type State: Send + 'static;
    /// The error type returned by this actor's operations.
    type Error: Send + std::error::Error + 'static;
    /// The typed response returned by `ask` requests.
    type Response: Send + 'static;

    /// Handle an incoming message, potentially mutating state.
    async fn handle_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
    ) -> Result<Self::Response, Self::Error>;

    /// Called before the actor starts processing messages.
    fn pre_start(&mut self) -> Result<(), Self::Error>;

    /// Called after the actor stops processing messages.
    fn post_stop(&mut self) -> Result<(), Self::Error>;

    /// Returns this actor's unique identifier.
    fn actor_id(&self) -> AgentId;
}

/// Canonical tool interface.
///
/// Tools are stateless, callable units of functionality that agents can invoke.
/// Each tool has a schema describing its parameters and capabilities.
#[async_trait]
pub trait Tool: Send + Sync + 'static {
    /// Execute the tool with the given parameters.
    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, ToolError>;

    /// Returns the JSON schema describing this tool's parameters.
    fn schema(&self) -> ToolSchema;

    /// Returns the capabilities of this tool.
    fn capabilities(&self) -> ToolCapabilities;

    /// Returns this tool's unique identifier.
    fn tool_id(&self) -> ToolId;

    /// Returns this tool's version.
    fn version(&self) -> semver::Version;
}

/// Agent interface extending Tool with context and role.
///
/// Agents are the primary unit of work in the framework. They extend Tool
/// with message processing, role assignment, and initialization.
#[async_trait]
pub trait Agent: Tool + Send + Sync + 'static {
    /// The context type providing dependencies and state to this agent.
    type Context: Send + Sync;
    /// The error type returned by agent-specific operations.
    type Error: Send + std::error::Error + 'static;

    /// Process an incoming message.
    async fn process(
        &self,
        message: serde_json::Value,
    ) -> Result<serde_json::Value, Self::Error>;

    /// Returns this agent's role in the system.
    fn role(&self) -> AgentType;

    /// Returns a reference to this agent's context.
    fn context(&self) -> &Self::Context;

    /// Initialize the agent with the given context.
    async fn initialize(&mut self, context: Self::Context) -> Result<(), Self::Error>;

    /// Returns the type IDs of services this agent depends on.
    fn dependencies() -> Vec<std::any::TypeId>
    where
        Self: Sized;
}

/// Generic resource abstraction with lifecycle management.
///
/// Resources represent external systems (databases, connections, etc.) that
/// can be acquired, released, and health-checked.
#[async_trait]
pub trait Resource: Send + Sync + 'static {
    /// Configuration type for acquiring this resource.
    type Config: Send + Sync + Clone + 'static;
    /// Error type for resource operations.
    type Error: Send + std::error::Error + 'static;

    /// Acquire a new instance of this resource.
    async fn acquire(config: Self::Config) -> Result<Self, Self::Error>
    where
        Self: Sized;

    /// Release this resource, performing cleanup.
    async fn release(self) -> Result<(), Self::Error>;

    /// Quick synchronous health check.
    fn is_healthy(&self) -> bool;

    /// Detailed asynchronous health check.
    async fn health_check(&self) -> Result<HealthStatus, Self::Error>;

    /// Returns this resource's unique identifier.
    fn resource_id(&self) -> ResourceId;
}

/// Supervision hierarchy management.
///
/// Supervisors manage child lifecycles using configurable restart strategies,
/// escalation policies, and backoff behavior.
#[async_trait]
pub trait Supervisor: Send + Sync + 'static {
    /// The type of children managed by this supervisor.
    type Child: Send + 'static;
    /// Error type for supervision operations.
    type Error: Send + std::error::Error + 'static;

    /// Supervise the given set of children.
    async fn supervise(&self, children: Vec<Self::Child>) -> Result<(), Self::Error>;

    /// Returns the supervision strategy configuration.
    fn supervision_strategy(&self) -> &SupervisionStrategy;

    /// Returns the restart policy for child failures.
    fn restart_policy(&self) -> RestartPolicy;

    /// Returns the escalation policy for exceeded restart limits.
    fn escalation_policy(&self) -> EscalationPolicy;

    /// Returns this supervisor's unique identifier.
    fn supervisor_id(&self) -> AgentId;
}

/// Protocol-agnostic transport interface.
///
/// Provides messaging operations (send, broadcast, subscribe, request-response)
/// and connection management. Uses `NetworkError` for all transport failures.
#[async_trait]
pub trait Transport: Send + Sync + 'static {
    /// Message type transported.
    type Message: Send + Sync + Serialize + DeserializeOwned + 'static;
    /// Subscription handle type.
    type Subscription: Send + 'static;
    /// Connection information type.
    type ConnectionInfo: Send + Sync + 'static;

    /// Send a message to a specific destination.
    async fn send(&self, destination: &str, message: Self::Message) -> Result<(), NetworkError>;

    /// Broadcast a message to a topic.
    async fn broadcast(&self, topic: &str, message: Self::Message) -> Result<(), NetworkError>;

    /// Subscribe to messages matching a pattern.
    async fn subscribe(&self, pattern: &str) -> Result<Self::Subscription, NetworkError>;

    /// Send a request and wait for a response with timeout.
    async fn request_response(
        &self,
        destination: &str,
        message: Self::Message,
        timeout: Duration,
    ) -> Result<Self::Message, NetworkError>;

    /// Connect to the transport backend.
    async fn connect(
        &mut self,
        config: &TransportConfig,
    ) -> Result<Self::ConnectionInfo, NetworkError>;

    /// Disconnect from the transport backend.
    async fn disconnect(&mut self) -> Result<(), NetworkError>;

    /// Returns the current connection status.
    fn connection_status(&self) -> ConnectionStatus;
}

/// Event publisher trait for breaking circular dependencies.
///
/// Defined in the core crate so that both monitoring and events crates
/// can use it without depending on each other. `EventBus` (events crate)
/// implements this trait; `HealthMonitor` (monitoring crate) accepts an
/// `Option<Arc<dyn EventPublisher>>`.
#[async_trait]
pub trait EventPublisher: Send + Sync + 'static {
    /// Publish a system event.
    async fn publish(&self, event: SystemEvent) -> Result<(), EventError>;
}
