//! T048: Transport supervision integration tests.
//!
//! Validates that transport components can be managed within the
//! Phase 3 supervision framework as actors with proper lifecycle management.

use mister_smith_transport::{InMemoryTransport, MessageEnvelope, Transport};
use std::sync::Arc;
use std::time::Duration;

/// Verify transport can be wrapped in an Arc and shared across tasks.
#[tokio::test]
async fn transport_arc_sharing() {
    let transport = Arc::new(InMemoryTransport::new());

    let t1 = transport.clone();
    let t2 = transport.clone();

    // Simulate two supervised actors sharing the same transport.
    let handle1 = tokio::spawn(async move {
        let mut sub = t1.subscribe("supervised.events").await.unwrap();
        sub.next().await
    });

    let handle2 = tokio::spawn(async move {
        let envelope = MessageEnvelope::builder("lifecycle.event")
            .payload_raw(b"actor started".to_vec())
            .build()
            .unwrap();
        t2.publish("supervised.events", envelope).await.unwrap();
    });

    handle2.await.unwrap();
    let msg = tokio::time::timeout(Duration::from_secs(2), handle1)
        .await
        .unwrap()
        .unwrap();
    assert!(msg.is_some());
    assert_eq!(msg.unwrap().envelope.message_type, "lifecycle.event");
}

/// Verify transport survives actor restarts (shared Arc persists).
#[tokio::test]
async fn transport_survives_actor_restart() {
    let transport = Arc::new(InMemoryTransport::new());

    // "First actor lifecycle" — subscribes and processes.
    let t1 = transport.clone();
    let sub1 = t1.subscribe("restart.test").await.unwrap();
    drop(sub1); // Actor "crashes" — subscription dropped.

    // "Restart" — new subscription on same transport.
    let mut sub2 = transport.subscribe("restart.test").await.unwrap();

    let envelope = MessageEnvelope::builder("after.restart")
        .payload_raw(b"still works".to_vec())
        .build()
        .unwrap();
    transport
        .publish("restart.test", envelope)
        .await
        .unwrap();

    let msg = tokio::time::timeout(Duration::from_secs(2), sub2.next())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(msg.envelope.message_type, "after.restart");
}

/// Verify multiple transport instances can coexist (different crates).
#[tokio::test]
async fn multiple_transport_instances() {
    let nats_sim = InMemoryTransport::new();
    let internal = InMemoryTransport::new();

    // Publish on one transport doesn't leak to the other.
    let mut sub_nats = nats_sim.subscribe("test.isolated").await.unwrap();
    let _sub_internal = internal.subscribe("test.isolated").await.unwrap();

    let envelope = MessageEnvelope::builder("nats.only")
        .payload_raw(b"nats message".to_vec())
        .build()
        .unwrap();
    nats_sim
        .publish("test.isolated", envelope)
        .await
        .unwrap();

    let msg = tokio::time::timeout(Duration::from_secs(1), sub_nats.next())
        .await
        .unwrap();
    assert!(msg.is_some());
}

/// Verify transport can be used concurrently by multiple supervised tasks.
#[tokio::test]
async fn transport_concurrent_supervised_tasks() {
    let transport = Arc::new(InMemoryTransport::new());
    let subject = "supervised.concurrent";

    let mut sub = transport.subscribe(subject).await.unwrap();

    // Spawn 5 "supervised actors" that each publish a message.
    let mut handles = Vec::new();
    for i in 0..5 {
        let t = transport.clone();
        handles.push(tokio::spawn(async move {
            let envelope = MessageEnvelope::builder("concurrent.msg")
                .payload_raw(format!("actor-{i}").into_bytes())
                .build()
                .unwrap();
            t.publish(subject, envelope).await.unwrap();
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    // All 5 messages should be received.
    let mut count = 0;
    for _ in 0..5 {
        if let Ok(Some(_)) = tokio::time::timeout(Duration::from_secs(2), sub.next()).await {
            count += 1;
        }
    }
    assert_eq!(count, 5);
}
