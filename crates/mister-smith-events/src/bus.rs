//! EventBus: in-process pub/sub with broadcast, filtering, and dead letter handling.
//!
//! The [`EventBus`] is the central event distribution mechanism. It implements
//! the core [`EventPublisher`] trait, allowing
//! any component that depends on `mister-smith-core` to publish events without
//! depending on this crate directly.

use std::sync::Arc;
use std::time::SystemTime;

use async_trait::async_trait;
use tokio::sync::{broadcast, RwLock};
use tracing;

use mister_smith_core::{EventPublisher, SystemEvent};

use crate::dead_letter::DeadLetterQueue;
use crate::error::EventBusError;
use crate::handler::{EventFilter, EventHandler};
use crate::store::EventStore;
use crate::types::Event;

/// Default broadcast channel capacity.
const DEFAULT_BROADCAST_CAPACITY: usize = 10_000;

/// In-process event bus with handler dispatch, broadcast, and dead letter handling.
///
/// The event bus distributes events to registered handlers (with optional filtering)
/// and to broadcast subscribers. Events that fail all handler delivery are routed
/// to a dead letter queue for later inspection.
pub struct EventBus {
    handlers: RwLock<Vec<Arc<dyn EventHandler>>>,
    broadcast_tx: broadcast::Sender<Event>,
    event_store: Option<Arc<dyn EventStore>>,
    dead_letter: Arc<DeadLetterQueue>,
}

impl EventBus {
    /// Create a new event bus with the given broadcast channel capacity.
    pub fn new(broadcast_capacity: usize) -> Self {
        let (broadcast_tx, _) = broadcast::channel(broadcast_capacity);
        Self {
            handlers: RwLock::new(Vec::new()),
            broadcast_tx,
            event_store: None,
            dead_letter: Arc::new(DeadLetterQueue::default()),
        }
    }

    /// Attach an event store for persistence and replay.
    ///
    /// Consumes and returns `self` for builder-style chaining.
    pub fn with_event_store(mut self, store: Arc<dyn EventStore>) -> Self {
        self.event_store = Some(store);
        self
    }

    /// Publish an event to all matching handlers, broadcast subscribers, and the event store.
    pub async fn publish(&self, event: Event) -> Result<(), EventBusError> {
        // Persist to event store if configured.
        if let Some(ref store) = self.event_store {
            store.append(event.clone()).await.map_err(|e| {
                tracing::error!(event_id = %event.id, "Failed to persist event to store: {e}");
                e
            })?;
        }

        // Broadcast to subscribers (ignore send errors — no receivers is not an error).
        let _ = self.broadcast_tx.send(event.clone());

        // Dispatch to handlers.
        self.process_event(event).await;

        Ok(())
    }

    /// Register a handler to receive events.
    pub async fn subscribe(&self, handler: Arc<dyn EventHandler>) {
        self.handlers.write().await.push(handler);
    }

    /// Subscribe to the broadcast channel for all events.
    ///
    /// The returned receiver will see every published event regardless of handler filters.
    pub fn subscribe_broadcast(&self) -> broadcast::Receiver<Event> {
        self.broadcast_tx.subscribe()
    }

    /// Replay events from the event store within a time range, optionally filtered.
    ///
    /// Returns an error if no event store is configured.
    pub async fn replay_events(
        &self,
        from: SystemTime,
        to: SystemTime,
        filter: Option<EventFilter>,
    ) -> Result<Vec<Event>, EventBusError> {
        let store = self.event_store.as_ref().ok_or_else(|| {
            EventBusError::StoreFailed("no event store configured for replay".into())
        })?;

        let events = store.query(from, to).await?;

        match filter {
            Some(f) => Ok(events.into_iter().filter(|e| f.matches(e)).collect()),
            None => Ok(events),
        }
    }

    /// Returns a reference to the dead letter queue.
    pub fn dead_letter_queue(&self) -> &DeadLetterQueue {
        &self.dead_letter
    }

    /// Dispatch an event to all registered handlers, applying filters and
    /// routing failures to the dead letter queue.
    async fn process_event(&self, event: Event) {
        let handlers = {
            let handlers = self.handlers.read().await;

            if handlers.is_empty() {
                return;
            }

            handlers.iter().cloned().collect::<Vec<_>>()
        };

        let mut any_handled = false;
        let mut all_failed = true;

        for handler in handlers {
            // Apply handler filter.
            if let Some(filter) = handler.event_filter() {
                if !filter.matches(&event) {
                    continue;
                }
            }

            any_handled = true;

            match handler.handle_event(event.clone()).await {
                Ok(()) => {
                    all_failed = false;
                }
                Err(e) => {
                    tracing::warn!(
                        event_id = %event.id,
                        event_type = %event.event_type,
                        "Event handler failed: {e}"
                    );
                }
            }
        }

        // If at least one handler matched but all of them failed, dead-letter the event.
        if any_handled && all_failed {
            tracing::error!(
                event_id = %event.id,
                "All matching handlers failed; routing to dead letter queue"
            );
            self.dead_letter.enqueue(event);
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(DEFAULT_BROADCAST_CAPACITY)
    }
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBus")
            .field("dead_letter", &self.dead_letter)
            .field("has_event_store", &self.event_store.is_some())
            .finish_non_exhaustive()
    }
}

/// Implement the core [`EventPublisher`] trait so the event bus can be used
/// through the core crate's trait object interface.
///
/// Converts the minimal [`SystemEvent`] from core into the richer [`Event`] type.
#[async_trait]
impl EventPublisher for EventBus {
    async fn publish(
        &self,
        system_event: SystemEvent,
    ) -> Result<(), mister_smith_core::EventError> {
        let event = Event::new(
            "system",
            crate::types::EventType::Custom(system_event.event_type),
        );
        // Construct a full Event with the system event's payload.
        let event = Event {
            payload: system_event.payload,
            ..event
        };

        // Delegate to EventBus::publish, converting the error.
        EventBus::publish(self, event)
            .await
            .map_err(|e| -> mister_smith_core::EventError { e.into() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::InMemoryEventStore;
    use crate::types::{AgentEventType, EventType, SystemEventType};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    /// Test handler that counts invocations.
    struct CountingHandler {
        count: AtomicUsize,
        filter: Option<EventFilter>,
    }

    impl CountingHandler {
        fn new() -> Self {
            Self {
                count: AtomicUsize::new(0),
                filter: None,
            }
        }

        fn with_filter(filter: EventFilter) -> Self {
            Self {
                count: AtomicUsize::new(0),
                filter: Some(filter),
            }
        }

        fn count(&self) -> usize {
            self.count.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl EventHandler for CountingHandler {
        async fn handle_event(&self, _event: Event) -> Result<(), EventBusError> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn event_filter(&self) -> Option<EventFilter> {
            self.filter.clone()
        }
    }

    struct ConcurrentSubscribeHandler {
        bus: Arc<EventBus>,
    }

    #[async_trait]
    impl EventHandler for ConcurrentSubscribeHandler {
        async fn handle_event(&self, _event: Event) -> Result<(), EventBusError> {
            self.bus.subscribe(Arc::new(CountingHandler::new())).await;
            Ok(())
        }
    }

    /// Handler that always fails.
    struct FailingHandler;

    #[async_trait]
    impl EventHandler for FailingHandler {
        async fn handle_event(&self, _event: Event) -> Result<(), EventBusError> {
            Err(EventBusError::HandlerFailed("intentional failure".into()))
        }
    }

    #[tokio::test]
    async fn publish_delivers_to_handler() {
        let bus = EventBus::default();
        let handler = Arc::new(CountingHandler::new());
        bus.subscribe(handler.clone()).await;

        let event = Event::new("test", EventType::System(SystemEventType::Started));
        bus.publish(event).await.unwrap();

        assert_eq!(handler.count(), 1);
    }

    #[tokio::test]
    async fn handler_filter_is_applied() {
        let bus = EventBus::default();

        let filter = EventFilter {
            event_types: Some(vec![EventType::System(SystemEventType::Started)]),
            ..Default::default()
        };
        let handler = Arc::new(CountingHandler::with_filter(filter));
        bus.subscribe(handler.clone()).await;

        // Matching event.
        let matching = Event::new("test", EventType::System(SystemEventType::Started));
        bus.publish(matching).await.unwrap();
        assert_eq!(handler.count(), 1);

        // Non-matching event.
        let non_matching = Event::new("test", EventType::System(SystemEventType::Stopped));
        bus.publish(non_matching).await.unwrap();
        assert_eq!(handler.count(), 1); // unchanged
    }

    #[tokio::test]
    async fn broadcast_delivers_all_events() {
        let bus = EventBus::default();
        let mut rx = bus.subscribe_broadcast();

        let event = Event::new("test", EventType::Agent(AgentEventType::Created));
        let event_id = event.id;
        bus.publish(event).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.id, event_id);
    }

    #[tokio::test]
    async fn handler_can_subscribe_during_dispatch_without_stall() {
        let bus = Arc::new(EventBus::default());
        bus.subscribe(Arc::new(ConcurrentSubscribeHandler {
            bus: Arc::clone(&bus),
        }))
        .await;

        let event = Event::new("test", EventType::Custom("concurrent-subscribe".into()));

        let publish_result =
            tokio::time::timeout(Duration::from_millis(250), bus.publish(event)).await;
        assert!(
            publish_result.is_ok(),
            "publish timed out due to lock contention"
        );
        assert!(publish_result.unwrap().is_ok());
    }

    #[tokio::test]
    async fn failing_handler_routes_to_dead_letter() {
        let bus = EventBus::default();
        bus.subscribe(Arc::new(FailingHandler)).await;

        let event = Event::new("test", EventType::Custom("fail".into()));
        bus.publish(event).await.unwrap();

        assert_eq!(bus.dead_letter_queue().len(), 1);
    }

    #[tokio::test]
    async fn partial_handler_failure_does_not_dead_letter() {
        let bus = EventBus::default();
        let success_handler = Arc::new(CountingHandler::new());
        bus.subscribe(success_handler.clone()).await;
        bus.subscribe(Arc::new(FailingHandler)).await;

        let event = Event::new("test", EventType::Custom("mixed".into()));
        bus.publish(event).await.unwrap();

        // One handler succeeded, so event should NOT be dead-lettered.
        assert_eq!(bus.dead_letter_queue().len(), 0);
        assert_eq!(success_handler.count(), 1);
    }

    #[tokio::test]
    async fn event_store_persists_events() {
        let store = Arc::new(InMemoryEventStore::new());
        let bus = EventBus::default().with_event_store(store.clone());

        let event = Event::new(
            "test",
            EventType::System(SystemEventType::ConfigurationChanged),
        );
        let event_id = event.id;
        bus.publish(event).await.unwrap();

        let found = store.get_by_id(event_id).await.unwrap();
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn replay_events_with_filter() {
        let store = Arc::new(InMemoryEventStore::new());
        let bus = EventBus::default().with_event_store(store);

        let before = SystemTime::now();
        tokio::time::sleep(Duration::from_millis(5)).await;

        bus.publish(Event::new(
            "agent-1",
            EventType::System(SystemEventType::Started),
        ))
        .await
        .unwrap();
        bus.publish(Event::new(
            "agent-2",
            EventType::Agent(AgentEventType::Created),
        ))
        .await
        .unwrap();

        tokio::time::sleep(Duration::from_millis(5)).await;
        let after = SystemTime::now();

        // No filter — get all.
        let all = bus.replay_events(before, after, None).await.unwrap();
        assert_eq!(all.len(), 2);

        // Filter by source.
        let filter = EventFilter {
            sources: Some(vec!["agent-1".into()]),
            ..Default::default()
        };
        let filtered = bus
            .replay_events(before, after, Some(filter))
            .await
            .unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].source, "agent-1");
    }

    #[tokio::test]
    async fn replay_without_store_returns_error() {
        let bus = EventBus::default();
        let result = bus
            .replay_events(SystemTime::now(), SystemTime::now(), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn publish_no_handlers_is_ok() {
        let bus = EventBus::default();
        let event = Event::new("test", EventType::Custom("orphan".into()));
        // Should not error even with no handlers.
        bus.publish(event).await.unwrap();
        assert_eq!(bus.dead_letter_queue().len(), 0);
    }

    #[tokio::test]
    async fn event_publisher_trait_impl() {
        let bus = EventBus::default();
        let handler = Arc::new(CountingHandler::new());
        bus.subscribe(handler.clone()).await;

        // Use the core trait interface.
        let publisher: &dyn EventPublisher = &bus;
        let system_event = SystemEvent {
            event_type: "test.event".into(),
            payload: serde_json::json!({"key": "value"}),
        };
        publisher.publish(system_event).await.unwrap();

        assert_eq!(handler.count(), 1);
    }
}
