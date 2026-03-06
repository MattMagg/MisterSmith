//! EventStore trait and InMemoryEventStore implementation.
//!
//! The [`EventStore`] trait provides a persistence abstraction for events,
//! enabling replay, audit, and correlation queries. [`InMemoryEventStore`]
//! is a simple in-memory implementation suitable for testing and development.

use async_trait::async_trait;
use std::time::SystemTime;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::EventBusError;
use crate::types::Event;

/// Trait for persistent event storage with query capabilities.
///
/// Implementations should be safe for concurrent access from multiple
/// handlers and the event bus.
#[async_trait]
pub trait EventStore: Send + Sync + 'static {
    /// Append an event to the store.
    async fn append(&self, event: Event) -> Result<(), EventBusError>;

    /// Query events within a time range.
    async fn query(&self, from: SystemTime, to: SystemTime) -> Result<Vec<Event>, EventBusError>;

    /// Retrieve a specific event by its ID.
    async fn get_by_id(&self, id: Uuid) -> Result<Option<Event>, EventBusError>;

    /// Retrieve all events sharing a correlation ID.
    async fn get_by_correlation(&self, correlation_id: Uuid) -> Result<Vec<Event>, EventBusError>;
}

/// In-memory event store backed by a [`Vec`] under a [`RwLock`].
///
/// Suitable for testing and development. Not suitable for production use
/// where durability and bounded memory are required.
#[derive(Debug)]
pub struct InMemoryEventStore {
    events: RwLock<Vec<Event>>,
}

impl InMemoryEventStore {
    /// Create a new empty in-memory event store.
    pub fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
        }
    }

    /// Returns the number of events currently stored.
    pub async fn len(&self) -> usize {
        self.events.read().await.len()
    }

    /// Returns `true` if the store contains no events.
    pub async fn is_empty(&self) -> bool {
        self.events.read().await.is_empty()
    }
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn append(&self, event: Event) -> Result<(), EventBusError> {
        self.events.write().await.push(event);
        Ok(())
    }

    async fn query(&self, from: SystemTime, to: SystemTime) -> Result<Vec<Event>, EventBusError> {
        let events = self.events.read().await;
        let results = events
            .iter()
            .filter(|e| e.timestamp >= from && e.timestamp <= to)
            .cloned()
            .collect();
        Ok(results)
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Event>, EventBusError> {
        let events = self.events.read().await;
        Ok(events.iter().find(|e| e.id == id).cloned())
    }

    async fn get_by_correlation(&self, correlation_id: Uuid) -> Result<Vec<Event>, EventBusError> {
        let events = self.events.read().await;
        let results = events
            .iter()
            .filter(|e| e.correlation_id == Some(correlation_id))
            .cloned()
            .collect();
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EventType, SystemEventType};
    use std::time::Duration;

    fn make_event(source: &str) -> Event {
        Event::new(source, EventType::System(SystemEventType::Started))
    }

    #[tokio::test]
    async fn append_and_get_by_id() {
        let store = InMemoryEventStore::new();
        let event = make_event("test");
        let id = event.id;

        store.append(event).await.unwrap();
        assert_eq!(store.len().await, 1);

        let found = store.get_by_id(id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, id);
    }

    #[tokio::test]
    async fn get_by_id_returns_none_for_missing() {
        let store = InMemoryEventStore::new();
        let result = store.get_by_id(Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn query_time_range() {
        let store = InMemoryEventStore::new();

        let before = SystemTime::now();
        tokio::time::sleep(Duration::from_millis(10)).await;

        store.append(make_event("in-range")).await.unwrap();

        tokio::time::sleep(Duration::from_millis(10)).await;
        let after = SystemTime::now();

        // Event should be in range.
        let results = store.query(before, after).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source, "in-range");

        // Event should not be in a future range.
        let future = after + Duration::from_secs(100);
        let results = store.query(after, future).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn get_by_correlation() {
        let store = InMemoryEventStore::new();
        let cid = Uuid::new_v4();

        let mut e1 = make_event("a");
        e1.correlation_id = Some(cid);

        let mut e2 = make_event("b");
        e2.correlation_id = Some(cid);

        let e3 = make_event("c"); // no correlation

        store.append(e1).await.unwrap();
        store.append(e2).await.unwrap();
        store.append(e3).await.unwrap();

        let results = store.get_by_correlation(cid).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn is_empty_on_new_store() {
        let store = InMemoryEventStore::new();
        assert!(store.is_empty().await);
    }

    #[tokio::test]
    async fn default_creates_empty_store() {
        let store = InMemoryEventStore::default();
        assert!(store.is_empty().await);
    }
}
