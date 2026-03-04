//! Dead letter queue for events that failed handler delivery.
//!
//! Events that cannot be processed by any handler are routed to the
//! [`DeadLetterQueue`] for later inspection, retry, or alerting.

use std::collections::VecDeque;
use std::sync::Mutex;

use crate::types::Event;

/// Default maximum size for the dead letter queue.
const DEFAULT_MAX_SIZE: usize = 1_000;

/// Queue for events that failed handler delivery.
///
/// Uses a bounded [`VecDeque`] under a [`Mutex`] to store failed events.
/// When the queue is full, the oldest event is dropped to make room.
pub struct DeadLetterQueue {
    queue: Mutex<VecDeque<Event>>,
    max_size: usize,
}

impl DeadLetterQueue {
    /// Create a new dead letter queue with the given maximum size.
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: Mutex::new(VecDeque::with_capacity(max_size.min(1_024))),
            max_size,
        }
    }

    /// Enqueue a failed event.
    ///
    /// If the queue is at capacity, the oldest event is dropped.
    pub fn enqueue(&self, event: Event) {
        let mut queue = self.queue.lock().expect("dead letter queue lock poisoned");
        if queue.len() >= self.max_size {
            queue.pop_front();
        }
        queue.push_back(event);
    }

    /// Drain all events from the queue, returning them in insertion order.
    pub fn drain(&self) -> Vec<Event> {
        let mut queue = self.queue.lock().expect("dead letter queue lock poisoned");
        queue.drain(..).collect()
    }

    /// Returns the number of events currently in the queue.
    pub fn len(&self) -> usize {
        self.queue
            .lock()
            .expect("dead letter queue lock poisoned")
            .len()
    }

    /// Returns `true` if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for DeadLetterQueue {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_SIZE)
    }
}

impl std::fmt::Debug for DeadLetterQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.len();
        f.debug_struct("DeadLetterQueue")
            .field("len", &len)
            .field("max_size", &self.max_size)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EventType, SystemEventType};

    fn make_event(source: &str) -> Event {
        Event::new(source, EventType::System(SystemEventType::Started))
    }

    #[test]
    fn enqueue_and_drain() {
        let dlq = DeadLetterQueue::new(10);
        dlq.enqueue(make_event("a"));
        dlq.enqueue(make_event("b"));
        assert_eq!(dlq.len(), 2);

        let events = dlq.drain();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].source, "a");
        assert_eq!(events[1].source, "b");
        assert!(dlq.is_empty());
    }

    #[test]
    fn evicts_oldest_when_full() {
        let dlq = DeadLetterQueue::new(2);
        dlq.enqueue(make_event("first"));
        dlq.enqueue(make_event("second"));
        dlq.enqueue(make_event("third"));

        assert_eq!(dlq.len(), 2);
        let events = dlq.drain();
        assert_eq!(events[0].source, "second");
        assert_eq!(events[1].source, "third");
    }

    #[test]
    fn is_empty_when_new() {
        let dlq = DeadLetterQueue::default();
        assert!(dlq.is_empty());
        assert_eq!(dlq.len(), 0);
    }

    #[test]
    fn debug_format() {
        let dlq = DeadLetterQueue::new(5);
        dlq.enqueue(make_event("x"));
        let debug = format!("{dlq:?}");
        assert!(debug.contains("len: 1"));
        assert!(debug.contains("max_size: 5"));
    }
}
