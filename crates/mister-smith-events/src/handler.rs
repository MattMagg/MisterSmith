//! EventHandler trait and EventFilter for selective event processing.
//!
//! Handlers implement [`EventHandler`] to receive events. Each handler may
//! optionally provide an [`EventFilter`] to restrict which events it receives.

use async_trait::async_trait;
use uuid::Uuid;

use crate::error::EventBusError;
use crate::types::{Event, EventType};

/// Filter for selecting which events a handler receives.
///
/// All non-`None` fields must match for an event to pass the filter (AND logic).
/// A `None` field means "match any" for that criterion.
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    /// If set, only events whose type is in this list will match.
    pub event_types: Option<Vec<EventType>>,
    /// If set, only events from these sources will match.
    pub sources: Option<Vec<String>>,
    /// If set, only events with a correlation ID in this list will match.
    pub correlation_ids: Option<Vec<Uuid>>,
}

impl EventFilter {
    /// Check whether the given event matches this filter.
    ///
    /// Returns `true` if all non-`None` criteria match the event.
    pub fn matches(&self, event: &Event) -> bool {
        if let Some(ref types) = self.event_types {
            if !types.contains(&event.event_type) {
                return false;
            }
        }

        if let Some(ref sources) = self.sources {
            if !sources.contains(&event.source) {
                return false;
            }
        }

        if let Some(ref correlation_ids) = self.correlation_ids {
            match event.correlation_id {
                Some(cid) => {
                    if !correlation_ids.contains(&cid) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        true
    }
}

/// Trait for event handlers that process events asynchronously.
///
/// Implementations receive events through [`handle_event`](EventHandler::handle_event)
/// and can optionally filter events via [`event_filter`](EventHandler::event_filter).
#[async_trait]
pub trait EventHandler: Send + Sync + 'static {
    /// Handle an incoming event.
    async fn handle_event(&self, event: Event) -> Result<(), EventBusError>;

    /// Returns an optional filter restricting which events this handler receives.
    ///
    /// Defaults to `None`, meaning the handler receives all events.
    fn event_filter(&self) -> Option<EventFilter> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AgentEventType, EventType, SystemEventType};

    #[test]
    fn empty_filter_matches_everything() {
        let filter = EventFilter::default();
        let event = Event::new("src", EventType::System(SystemEventType::Started));
        assert!(filter.matches(&event));
    }

    #[test]
    fn filter_by_event_type() {
        let filter = EventFilter {
            event_types: Some(vec![EventType::System(SystemEventType::Started)]),
            ..Default::default()
        };

        let matching = Event::new("src", EventType::System(SystemEventType::Started));
        let non_matching = Event::new("src", EventType::System(SystemEventType::Stopped));

        assert!(filter.matches(&matching));
        assert!(!filter.matches(&non_matching));
    }

    #[test]
    fn filter_by_source() {
        let filter = EventFilter {
            sources: Some(vec!["agent-1".into()]),
            ..Default::default()
        };

        let matching = Event::new("agent-1", EventType::Agent(AgentEventType::Started));
        let non_matching = Event::new("agent-2", EventType::Agent(AgentEventType::Started));

        assert!(filter.matches(&matching));
        assert!(!filter.matches(&non_matching));
    }

    #[test]
    fn filter_by_correlation_id() {
        let cid = Uuid::new_v4();
        let filter = EventFilter {
            correlation_ids: Some(vec![cid]),
            ..Default::default()
        };

        let mut matching = Event::new("src", EventType::Custom("test".into()));
        matching.correlation_id = Some(cid);

        let no_cid = Event::new("src", EventType::Custom("test".into()));

        let mut wrong_cid = Event::new("src", EventType::Custom("test".into()));
        wrong_cid.correlation_id = Some(Uuid::new_v4());

        assert!(filter.matches(&matching));
        assert!(!filter.matches(&no_cid));
        assert!(!filter.matches(&wrong_cid));
    }

    #[test]
    fn filter_combines_criteria_with_and() {
        let filter = EventFilter {
            event_types: Some(vec![EventType::System(SystemEventType::Started)]),
            sources: Some(vec!["agent-1".into()]),
            correlation_ids: None,
        };

        // Matches type but not source.
        let wrong_source = Event::new("agent-2", EventType::System(SystemEventType::Started));
        assert!(!filter.matches(&wrong_source));

        // Matches source but not type.
        let wrong_type = Event::new("agent-1", EventType::System(SystemEventType::Stopped));
        assert!(!filter.matches(&wrong_type));

        // Matches both.
        let matching = Event::new("agent-1", EventType::System(SystemEventType::Started));
        assert!(filter.matches(&matching));
    }

    #[test]
    fn filter_multiple_event_types() {
        let filter = EventFilter {
            event_types: Some(vec![
                EventType::System(SystemEventType::Started),
                EventType::System(SystemEventType::Stopped),
            ]),
            ..Default::default()
        };

        let started = Event::new("src", EventType::System(SystemEventType::Started));
        let stopped = Event::new("src", EventType::System(SystemEventType::Stopped));
        let stopping = Event::new("src", EventType::System(SystemEventType::Stopping));

        assert!(filter.matches(&started));
        assert!(filter.matches(&stopped));
        assert!(!filter.matches(&stopping));
    }
}
