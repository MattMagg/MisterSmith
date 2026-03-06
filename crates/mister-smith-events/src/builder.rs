//! EventBuilder for constructing [`Event`] instances with a fluent API.
//!
//! Use the builder when you need to set optional fields like correlation IDs
//! or custom payloads. For simple events, [`Event::new`] is sufficient.

use serde::Serialize;
use uuid::Uuid;

use crate::types::{Event, EventType};

/// Fluent builder for constructing [`Event`] instances.
///
/// # Example
///
/// ```
/// use mister_smith_events::builder::EventBuilder;
/// use mister_smith_events::types::{EventType, SystemEventType};
///
/// let event = EventBuilder::new("my-component", EventType::System(SystemEventType::Started))
///     .with_payload(&serde_json::json!({"version": "1.0"}))
///     .build();
/// ```
pub struct EventBuilder {
    source: String,
    event_type: EventType,
    payload: Option<serde_json::Value>,
    correlation_id: Option<Uuid>,
    causation_id: Option<Uuid>,
}

impl EventBuilder {
    /// Create a new builder with the required source and event type.
    pub fn new(source: impl Into<String>, event_type: EventType) -> Self {
        Self {
            source: source.into(),
            event_type,
            payload: None,
            correlation_id: None,
            causation_id: None,
        }
    }

    /// Set the event payload from any serializable value.
    ///
    /// If serialization fails, the payload is set to a JSON string containing
    /// the error message rather than silently dropping the data.
    pub fn with_payload<T: Serialize>(mut self, value: &T) -> Self {
        self.payload =
            Some(serde_json::to_value(value).unwrap_or_else(|e| {
                serde_json::Value::String(format!("serialization error: {e}"))
            }));
        self
    }

    /// Set the correlation ID for linking related events.
    pub fn with_correlation_id(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Set the causation ID referencing the event that caused this one.
    pub fn with_causation_id(mut self, id: Uuid) -> Self {
        self.causation_id = Some(id);
        self
    }

    /// Build the event, consuming the builder.
    pub fn build(self) -> Event {
        Event {
            id: Uuid::new_v4(),
            timestamp: std::time::SystemTime::now(),
            source: self.source,
            event_type: self.event_type,
            payload: self.payload.unwrap_or(serde_json::Value::Null),
            correlation_id: self.correlation_id,
            causation_id: self.causation_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AgentEventType, SystemEventType};

    #[test]
    fn build_minimal_event() {
        let event =
            EventBuilder::new("builder-test", EventType::System(SystemEventType::Started)).build();
        assert_eq!(event.source, "builder-test");
        assert_eq!(
            event.event_type,
            EventType::System(SystemEventType::Started)
        );
        assert_eq!(event.payload, serde_json::Value::Null);
        assert!(event.correlation_id.is_none());
        assert!(event.causation_id.is_none());
    }

    #[test]
    fn build_with_payload() {
        let event = EventBuilder::new("builder-test", EventType::Agent(AgentEventType::Created))
            .with_payload(&serde_json::json!({"agent_name": "worker-1"}))
            .build();
        assert_eq!(event.payload["agent_name"], "worker-1");
    }

    #[test]
    fn build_with_correlation_and_causation() {
        let cid = Uuid::new_v4();
        let cause = Uuid::new_v4();

        let event = EventBuilder::new("test", EventType::Custom("x".into()))
            .with_correlation_id(cid)
            .with_causation_id(cause)
            .build();

        assert_eq!(event.correlation_id, Some(cid));
        assert_eq!(event.causation_id, Some(cause));
    }

    #[test]
    fn builder_accepts_string_source() {
        let source = String::from("dynamic-source");
        let event = EventBuilder::new(source, EventType::System(SystemEventType::Stopped)).build();
        assert_eq!(event.source, "dynamic-source");
    }

    #[test]
    fn build_with_struct_payload() {
        #[derive(Serialize)]
        struct Info {
            count: u32,
        }
        let event = EventBuilder::new("test", EventType::Custom("info".into()))
            .with_payload(&Info { count: 42 })
            .build();
        assert_eq!(event.payload["count"], 42);
    }
}
