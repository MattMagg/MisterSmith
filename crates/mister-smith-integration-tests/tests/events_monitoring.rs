//! Integration test: EventBus publish/subscribe + MetricsCollector recording.
//!
//! Covers T166: EventBus publishes `SystemEventType::Started`, a broadcast
//! subscriber receives it, and `MetricsCollector` records the
//! `events_published_total` metric.

use std::time::Duration;

use mister_smith_events::{Event, EventBus, EventType, SystemEventType};
use mister_smith_monitoring::MetricsCollector;

#[tokio::test]
async fn event_bus_publish_subscribe_with_metrics() {
    // 1. Create an EventBus with capacity 16.
    let bus = EventBus::new(16);

    // 2. Subscribe to the broadcast channel before publishing.
    let mut rx = bus.subscribe_broadcast();

    // 3. Create a Started system event.
    let event = Event::new("test", EventType::System(SystemEventType::Started));

    // 4. Publish the event.
    bus.publish(event).await.expect("publish should succeed");

    // 5. Receive the event from the broadcast subscriber and verify it.
    let received = rx.recv().await.expect("should receive broadcast event");
    assert_eq!(
        received.event_type,
        EventType::System(SystemEventType::Started),
        "received event should be SystemEventType::Started"
    );
    assert_eq!(received.source, "test");

    // 6. Create a MetricsCollector.
    let collector = MetricsCollector::new(Duration::from_secs(60));

    // 7. Record an event-published metric.
    collector.record_event_published().await;

    // 8. Assert the buffered count is 1.
    assert_eq!(
        collector.buffered_count().await,
        1,
        "MetricsCollector should have exactly one buffered metric after record_event_published"
    );
}
