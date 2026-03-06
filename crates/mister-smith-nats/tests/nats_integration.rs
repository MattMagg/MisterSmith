//! Integration tests for NATS transport.
//!
//! Requires a running NATS server at localhost:4222 with JetStream enabled.
//! Start with: `docker run -d --name NATS -p 4222:4222 -p 8222:8222 nats:latest -js`

use std::time::Duration;

use mister_smith_nats::{JetStreamConfig, JetStreamManager, NatsTransport, NatsTransportConfig};
use mister_smith_transport::{MessageEnvelope, Transport};

fn test_config() -> NatsTransportConfig {
    NatsTransportConfig {
        server_urls: vec!["nats://localhost:4222".to_string()],
        name: "integration-test".to_string(),
        connection_timeout: Duration::from_secs(5),
        request_timeout: Duration::from_secs(5),
        ..Default::default()
    }
}

fn test_envelope(msg_type: &str) -> MessageEnvelope {
    MessageEnvelope::builder(msg_type)
        .payload_raw(b"integration-test-payload".to_vec())
        .build()
        .unwrap()
}

#[tokio::test]
async fn pubsub_message_delivery() {
    let transport = NatsTransport::new(test_config());
    transport.connect().await.unwrap();

    let subject = format!("test.pubsub.{}", uuid::Uuid::new_v4());
    let mut sub = transport.subscribe(&subject).await.unwrap();

    let envelope = test_envelope("pubsub.test");
    let msg_id = envelope.message_id;
    transport.publish(&subject, envelope).await.unwrap();

    let received = tokio::time::timeout(Duration::from_secs(5), sub.next())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(received.envelope.message_id, msg_id);
    assert_eq!(received.envelope.message_type, "pubsub.test");
}

#[tokio::test]
async fn queue_group_single_delivery() {
    let transport = NatsTransport::new(test_config());
    transport.connect().await.unwrap();

    let subject = format!("test.queue.{}", uuid::Uuid::new_v4());
    let queue = "test-workers";

    let mut sub1 = transport.queue_subscribe(&subject, queue).await.unwrap();
    let mut sub2 = transport.queue_subscribe(&subject, queue).await.unwrap();
    let mut sub3 = transport.queue_subscribe(&subject, queue).await.unwrap();

    // Send 100 messages — each should go to exactly one subscriber.
    let msg_count = 100;
    for i in 0..msg_count {
        let envelope = test_envelope(&format!("queue.msg.{i}"));
        transport.publish(&subject, envelope).await.unwrap();
    }

    // Collect all received messages from all subscribers with a timeout.
    let mut total_received = 0u32;

    let collect = async {
        loop {
            tokio::select! {
                Some(_) = sub1.next() => total_received += 1,
                Some(_) = sub2.next() => total_received += 1,
                Some(_) = sub3.next() => total_received += 1,
                else => break,
            }
            if total_received >= msg_count {
                break;
            }
        }
    };

    tokio::time::timeout(Duration::from_secs(5), collect)
        .await
        .unwrap();

    // SC-007: Each message delivered to exactly one subscriber.
    assert_eq!(
        total_received, msg_count,
        "expected {msg_count} total messages across all queue subscribers"
    );
}

#[tokio::test]
async fn request_reply_with_correlation_id() {
    let transport = NatsTransport::new(test_config());
    transport.connect().await.unwrap();

    let subject = format!("test.reqrep.{}", uuid::Uuid::new_v4());

    // Set up a responder.
    let transport_clone = transport.clone();
    let subject_clone = subject.clone();
    let mut sub = transport.subscribe(&subject).await.unwrap();

    tokio::spawn(async move {
        if let Some(msg) = sub.next().await {
            let response = MessageEnvelope::builder("echo.response")
                .payload_raw(msg.envelope.payload.clone())
                .build()
                .unwrap();
            if let Some(reply_subject) = msg.reply_subject {
                transport_clone
                    .publish(&reply_subject, response)
                    .await
                    .unwrap();
            }
        }
    });

    // Small delay to ensure subscriber is ready.
    tokio::time::sleep(Duration::from_millis(50)).await;

    let request = test_envelope("echo.request");
    let response = transport
        .request(&subject_clone, request, Duration::from_secs(5))
        .await
        .unwrap();

    assert_eq!(response.message_type, "echo.response");
    assert_eq!(response.payload, b"integration-test-payload");
}

#[tokio::test]
async fn request_timeout_error() {
    let transport = NatsTransport::new(test_config());
    transport.connect().await.unwrap();

    let subject = format!("test.timeout.{}", uuid::Uuid::new_v4());

    // No responder — should time out.
    let request = test_envelope("timeout.request");
    let result = transport
        .request(&subject, request, Duration::from_millis(200))
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn connection_state_tracking() {
    let transport = NatsTransport::new(test_config());

    // Before connecting, state should be Disconnected.
    let state = transport.connection_state().await;
    assert_eq!(state, async_nats::connection::State::Disconnected);

    // After connecting, state should be Connected.
    transport.connect().await.unwrap();
    let state = transport.connection_state().await;
    assert_eq!(state, async_nats::connection::State::Connected);
}

#[tokio::test]
async fn health_check_reports_healthy_when_connected() {
    use mister_smith_monitoring::{HealthCheck, Status};
    use mister_smith_nats::NatsHealthCheck;

    let transport = NatsTransport::new(test_config());
    transport.connect().await.unwrap();

    let health = NatsHealthCheck::new(transport);
    let status = health.check().await.unwrap();
    assert_eq!(status, Status::Healthy);
}

#[tokio::test]
async fn jetstream_publish_and_consume() {
    let transport = NatsTransport::new(test_config());
    transport.connect().await.unwrap();

    let client = transport.inner_client().await.unwrap();
    let js = JetStreamManager::new(client, JetStreamConfig::default());

    let stream_name = format!("TEST_{}", uuid::Uuid::new_v4().simple());

    // Create stream.
    js.create_stream(
        &stream_name,
        vec![format!("{stream_name}.>")],
        async_nats::jetstream::stream::RetentionPolicy::WorkQueue,
    )
    .await
    .unwrap();

    // Publish a durable message.
    let subject = format!("{stream_name}.tasks");
    let envelope = test_envelope("jetstream.task");
    js.publish_and_ack(&subject, envelope).await.unwrap();

    // Create a pull consumer and fetch the message.
    let consumer = js
        .create_pull_consumer(
            &stream_name,
            async_nats::jetstream::consumer::pull::Config {
                durable_name: Some("test-consumer".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    use futures::StreamExt;
    let mut messages = consumer.messages().await.unwrap();
    let msg = tokio::time::timeout(Duration::from_secs(5), messages.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();

    let decoded = JetStreamManager::decode_message(&msg.payload).unwrap();
    assert_eq!(decoded.message_type, "jetstream.task");

    // Acknowledge.
    msg.ack().await.unwrap();

    // Cleanup.
    js.delete_stream(&stream_name).await.unwrap();
}

#[tokio::test]
async fn durable_transport_ack_nak_term_in_progress() {
    use mister_smith_transport::DurableTransport;

    let transport = NatsTransport::new(test_config());
    transport.connect().await.unwrap();

    let client = transport.inner_client().await.unwrap();
    let js = JetStreamManager::new(client, JetStreamConfig::default());

    let stream_name = format!("ACKTEST_{}", uuid::Uuid::new_v4().simple());
    let filter = format!("{stream_name}.>");

    js.create_stream(
        &stream_name,
        vec![filter.clone()],
        async_nats::jetstream::stream::RetentionPolicy::WorkQueue,
    )
    .await
    .unwrap();

    let subject = format!("{stream_name}.tasks");

    // --- Test in_progress: resets ack deadline without acknowledging ---
    let env1 = test_envelope("ack-test.in-progress");
    transport.durable_publish(&subject, env1).await.unwrap();

    let mut sub = transport
        .durable_subscribe(&stream_name, "test-ack-consumer", &filter)
        .await
        .unwrap();

    let msg1 = tokio::time::timeout(Duration::from_secs(5), sub.next())
        .await
        .expect("timed out waiting for message")
        .expect("subscription ended");
    assert_eq!(msg1.envelope.message_type, "ack-test.in-progress");

    // Signal work-in-progress (should succeed without error).
    msg1.in_progress().await.unwrap();
    // Then acknowledge to clear the message.
    msg1.ack().await.unwrap();

    // --- Test nak: message is redelivered ---
    let env2 = test_envelope("ack-test.nak");
    transport.durable_publish(&subject, env2).await.unwrap();

    let msg2 = tokio::time::timeout(Duration::from_secs(5), sub.next())
        .await
        .expect("timed out waiting for message")
        .expect("subscription ended");
    assert_eq!(msg2.envelope.message_type, "ack-test.nak");

    // Nak without delay — requests immediate redelivery.
    msg2.nak(None).await.unwrap();

    // The same message should be redelivered.
    let msg2_redelivered = tokio::time::timeout(Duration::from_secs(5), sub.next())
        .await
        .expect("timed out waiting for redelivered message")
        .expect("subscription ended after nak");
    assert_eq!(msg2_redelivered.envelope.message_type, "ack-test.nak");
    msg2_redelivered.ack().await.unwrap();

    // --- Test term: message is NOT redelivered ---
    let env3 = test_envelope("ack-test.term");
    transport.durable_publish(&subject, env3).await.unwrap();

    let msg3 = tokio::time::timeout(Duration::from_secs(5), sub.next())
        .await
        .expect("timed out waiting for message")
        .expect("subscription ended");
    assert_eq!(msg3.envelope.message_type, "ack-test.term");

    // Term — marks as terminal failure, no redelivery.
    msg3.term().await.unwrap();

    // Publish another message to prove the subscription is alive but the
    // termed message is gone.
    let env4 = test_envelope("ack-test.after-term");
    transport.durable_publish(&subject, env4).await.unwrap();

    let msg4 = tokio::time::timeout(Duration::from_secs(5), sub.next())
        .await
        .expect("timed out waiting for message after term")
        .expect("subscription ended");
    // We should get the NEW message, not a redelivery of the termed one.
    assert_eq!(msg4.envelope.message_type, "ack-test.after-term");
    msg4.ack().await.unwrap();

    // Cleanup.
    js.delete_stream(&stream_name).await.unwrap();
}
