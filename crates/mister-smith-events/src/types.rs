//! Event types: SystemEventType, AgentEventType, ToolEventType, AutonomyEventType, EventType,
//! and Event.
//!
//! The [`Event`] struct is the rich event representation used throughout the events crate.
//! It is distinct from [`mister_smith_core::SystemEvent`], which is a minimal struct
//! used by the `EventPublisher` trait in the core crate. The `EventBus` converts
//! between the two when implementing `EventPublisher`.

use crate::autonomy::AutonomyEventType;
use crate::autonomy::AutonomyEvent;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

/// System-level event types for framework lifecycle and health.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemEventType {
    /// Framework or component has started.
    Started,
    /// Framework or component is stopping.
    Stopping,
    /// Framework or component has stopped.
    Stopped,
    /// Health check passed successfully.
    HealthCheckPassed,
    /// Health check failed.
    HealthCheckFailed,
    /// Configuration has changed.
    ConfigurationChanged,
    /// Resource pool has been exhausted.
    ResourcePoolExhausted,
    /// Circuit breaker has opened (failing fast).
    CircuitBreakerOpen,
    /// Circuit breaker has closed (normal operation resumed).
    CircuitBreakerClosed,
}

/// Agent-level event types for agent lifecycle and messaging.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentEventType {
    /// Agent was created.
    Created,
    /// Agent has started.
    Started,
    /// Agent has stopped.
    Stopped,
    /// Agent has failed.
    Failed,
    /// Agent received a message.
    MessageReceived,
    /// Agent processed a message.
    MessageProcessed,
    /// Agent state changed.
    StateChanged,
}

/// Tool-level event types for tool lifecycle and execution.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolEventType {
    /// Tool was registered in the registry.
    Registered,
    /// Tool was unregistered from the registry.
    Unregistered,
    /// Tool execution started.
    ExecutionStarted,
    /// Tool execution completed successfully.
    ExecutionCompleted,
    /// Tool execution failed.
    ExecutionFailed,
    /// Tool access was denied.
    PermissionDenied,
}

/// Top-level event type discriminator.
///
/// Groups events into system, agent, and tool domains, with a [`Custom`](EventType::Custom)
/// escape hatch for user-defined event types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    /// System-level events.
    System(SystemEventType),
    /// Agent-level events.
    Agent(AgentEventType),
    /// Tool-level events.
    Tool(ToolEventType),
    /// Autonomy control-plane events.
    Autonomy(AutonomyEventType),
    /// Custom event type identified by a string key.
    Custom(String),
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::System(t) => write!(f, "system.{t:?}"),
            EventType::Agent(t) => write!(f, "agent.{t:?}"),
            EventType::Tool(t) => write!(f, "tool.{t:?}"),
            EventType::Autonomy(t) => write!(f, "autonomy.{t:?}"),
            EventType::Custom(s) => write!(f, "custom.{s}"),
        }
    }
}

/// Rich event structure used throughout the events crate.
///
/// Contains full metadata including correlation and causation IDs for
/// distributed tracing. This is distinct from [`mister_smith_core::SystemEvent`],
/// which is a minimal type used by the core `EventPublisher` trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique event identifier.
    pub id: Uuid,
    /// When the event was created.
    pub timestamp: SystemTime,
    /// Source component that emitted the event.
    pub source: String,
    /// The type of this event.
    pub event_type: EventType,
    /// Event payload as JSON.
    pub payload: serde_json::Value,
    /// Correlation ID linking related events across a workflow.
    pub correlation_id: Option<Uuid>,
    /// Causation ID referencing the event that directly caused this one.
    pub causation_id: Option<Uuid>,
}

impl Event {
    /// Create a new event with the given source and type.
    ///
    /// Sets a fresh UUID, current timestamp, and null payload.
    /// Use [`EventBuilder`](crate::builder::EventBuilder) for richer construction.
    pub fn new(source: impl Into<String>, event_type: EventType) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            source: source.into(),
            event_type,
            payload: serde_json::Value::Null,
            correlation_id: None,
            causation_id: None,
        }
    }

    /// Convert this rich event into a core [`SystemEvent`](mister_smith_core::SystemEvent).
    ///
    /// The `event_type` is serialized to its `Display` representation, and the
    /// full event (including metadata) becomes the payload.
    pub fn to_core_event(&self) -> mister_smith_core::SystemEvent {
        mister_smith_core::SystemEvent {
            event_type: self.event_type.to_string(),
            payload: serde_json::to_value(self).unwrap_or(serde_json::Value::Null),
        }
    }

    /// Decode the payload as a typed autonomy event when the discriminator matches.
    pub fn autonomy_event(&self) -> Result<Option<AutonomyEvent>, serde_json::Error> {
        match self.event_type {
            EventType::Autonomy(_) => serde_json::from_value(self.payload.clone()).map(Some),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_new_sets_defaults() {
        let event = Event::new("test-source", EventType::System(SystemEventType::Started));
        assert_eq!(event.source, "test-source");
        assert_eq!(
            event.event_type,
            EventType::System(SystemEventType::Started)
        );
        assert_eq!(event.payload, serde_json::Value::Null);
        assert!(event.correlation_id.is_none());
        assert!(event.causation_id.is_none());
    }

    #[test]
    fn event_type_display() {
        assert_eq!(
            EventType::System(SystemEventType::Started).to_string(),
            "system.Started"
        );
        assert_eq!(
            EventType::Agent(AgentEventType::Created).to_string(),
            "agent.Created"
        );
        assert_eq!(
            EventType::Tool(ToolEventType::Registered).to_string(),
            "tool.Registered"
        );
        assert_eq!(
            EventType::Custom("my.event".into()).to_string(),
            "custom.my.event"
        );
    }

    #[test]
    fn event_serde_roundtrip() {
        let event = Event::new(
            "test-component",
            EventType::Agent(AgentEventType::MessageReceived),
        );
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, event.id);
        assert_eq!(deserialized.source, "test-component");
        assert_eq!(
            deserialized.event_type,
            EventType::Agent(AgentEventType::MessageReceived)
        );
    }

    #[test]
    fn to_core_event_conversion() {
        let event = Event::new("my-source", EventType::System(SystemEventType::Stopped));
        let core_event = event.to_core_event();
        assert_eq!(core_event.event_type, "system.Stopped");
        // Payload should contain the serialized Event.
        assert!(core_event.payload.is_object());
    }

    #[test]
    fn system_event_type_equality() {
        assert_eq!(SystemEventType::Started, SystemEventType::Started);
        assert_ne!(SystemEventType::Started, SystemEventType::Stopped);
    }

    #[test]
    fn event_type_hash_works() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(EventType::System(SystemEventType::Started));
        set.insert(EventType::System(SystemEventType::Started)); // duplicate
        set.insert(EventType::Agent(AgentEventType::Created));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn unique_event_ids() {
        let a = Event::new("src", EventType::Custom("a".into()));
        let b = Event::new("src", EventType::Custom("a".into()));
        assert_ne!(a.id, b.id);
    }
}
