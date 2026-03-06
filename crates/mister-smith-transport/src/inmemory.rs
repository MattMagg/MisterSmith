//! In-memory transport implementation for testing.
//!
//! Backed by `tokio::sync::broadcast` channels. Supports publish/subscribe,
//! queue group delivery (round-robin), and request-reply with timeout.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex, RwLock};
use uuid::Uuid;

use crate::envelope::MessageEnvelope;
use crate::errors::TransportError;
use crate::transport::{ReceivedMessage, Subscription, Transport};

/// Default broadcast channel capacity.
const DEFAULT_CHANNEL_CAPACITY: usize = 1024;

/// Internal message sent through broadcast channels.
#[derive(Debug, Clone)]
struct InternalMessage {
    envelope: MessageEnvelope,
    reply_subject: Option<String>,
    /// For queue groups: intended receiver index by queue-group key.
    queue_indices: HashMap<String, usize>,
}

/// Tracks queue group membership for round-robin delivery.
struct QueueGroup {
    counter: AtomicUsize,
    member_count: AtomicUsize,
}

/// In-memory transport for testing and development.
///
/// Implements the full `Transport` trait without external dependencies.
/// Uses `tokio::sync::broadcast` channels for pub/sub delivery.
#[derive(Clone)]
pub struct InMemoryTransport {
    /// Per-subject broadcast senders.
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<InternalMessage>>>>,
    /// Queue group state for round-robin delivery.
    queue_groups: Arc<RwLock<HashMap<String, Arc<QueueGroup>>>>,
    /// Pending request-reply responses keyed by correlation ID.
    pending_replies: Arc<Mutex<HashMap<Uuid, tokio::sync::oneshot::Sender<MessageEnvelope>>>>,
    /// Channel capacity.
    capacity: usize,
}

impl InMemoryTransport {
    /// Create a new in-memory transport with default capacity.
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            queue_groups: Arc::new(RwLock::new(HashMap::new())),
            pending_replies: Arc::new(Mutex::new(HashMap::new())),
            capacity: DEFAULT_CHANNEL_CAPACITY,
        }
    }

    /// Create a new in-memory transport with custom channel capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            queue_groups: Arc::new(RwLock::new(HashMap::new())),
            pending_replies: Arc::new(Mutex::new(HashMap::new())),
            capacity,
        }
    }

    /// Get or create a broadcast channel for a subject.
    async fn get_or_create_sender(&self, subject: &str) -> broadcast::Sender<InternalMessage> {
        {
            let channels = self.channels.read().await;
            if let Some(sender) = channels.get(subject) {
                return sender.clone();
            }
        }
        let mut channels = self.channels.write().await;
        channels
            .entry(subject.to_string())
            .or_insert_with(|| broadcast::channel(self.capacity).0)
            .clone()
    }

    /// Subscribe to a broadcast channel for a subject.
    async fn subscribe_to_channel(&self, subject: &str) -> broadcast::Receiver<InternalMessage> {
        let sender = self.get_or_create_sender(subject).await;
        sender.subscribe()
    }
}

impl Default for InMemoryTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for InMemoryTransport {
    async fn publish(
        &self,
        subject: &str,
        envelope: MessageEnvelope,
    ) -> Result<(), TransportError> {
        // Check if this is a reply to a pending request.
        if let Some(corr_id) = envelope.correlation_id {
            let mut pending = self.pending_replies.lock().await;
            if let Some(sender) = pending.remove(&corr_id) {
                let _ = sender.send(envelope);
                return Ok(());
            }
        }

        let sender = self.get_or_create_sender(subject).await;

        // Determine queue group round-robin indices for all queue groups on this subject.
        let queue_indices = {
            let groups = self.queue_groups.read().await;
            let mut indices = HashMap::new();
            let subject_prefix = format!("{subject}:");
            for (key, group) in groups.iter() {
                if key.starts_with(&subject_prefix) {
                    let count = group.member_count.load(Ordering::Relaxed);
                    if count > 0 {
                        indices.insert(
                            key.clone(),
                            group.counter.fetch_add(1, Ordering::Relaxed) % count,
                        );
                    }
                }
            }
            indices
        };

        let msg = InternalMessage {
            envelope,
            reply_subject: None,
            queue_indices,
        };

        // Broadcast to all subscribers. Ignore errors (no receivers).
        let _ = sender.send(msg);
        Ok(())
    }

    async fn subscribe(&self, subject: &str) -> Result<Subscription, TransportError> {
        let mut rx = self.subscribe_to_channel(subject).await;

        let stream = async_stream::stream! {
            loop {
                match rx.recv().await {
                    Ok(msg) => {
                        // Regular subscribers receive all messages.
                        yield ReceivedMessage {
                            envelope: msg.envelope,
                            reply_subject: msg.reply_subject,
                        };
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                }
            }
        };

        Ok(Subscription::new(stream))
    }

    async fn queue_subscribe(
        &self,
        subject: &str,
        queue: &str,
    ) -> Result<Subscription, TransportError> {
        let group_key = format!("{subject}:{queue}");

        // Register this subscriber in the queue group.
        let member_index = {
            let mut groups = self.queue_groups.write().await;
            let group = groups.entry(group_key.clone()).or_insert_with(|| {
                Arc::new(QueueGroup {
                    counter: AtomicUsize::new(0),
                    member_count: AtomicUsize::new(0),
                })
            });
            group.member_count.fetch_add(1, Ordering::Relaxed)
        };

        let mut rx = self.subscribe_to_channel(subject).await;

        let stream = async_stream::stream! {
            loop {
                match rx.recv().await {
                    Ok(msg) => {
                        // Queue subscribers only receive messages assigned to their index.
                        if let Some(idx) = msg.queue_indices.get(&group_key) {
                            if *idx == member_index {
                                yield ReceivedMessage {
                                    envelope: msg.envelope,
                                    reply_subject: msg.reply_subject,
                                };
                            }
                        } else {
                            // No queue routing — deliver to all (fallback).
                            yield ReceivedMessage {
                                envelope: msg.envelope,
                                reply_subject: msg.reply_subject,
                            };
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                }
            }
        };

        Ok(Subscription::new(stream))
    }

    async fn request(
        &self,
        subject: &str,
        mut envelope: MessageEnvelope,
        timeout: Duration,
    ) -> Result<MessageEnvelope, TransportError> {
        // Generate a correlation ID for matching the response.
        let correlation_id = Uuid::new_v4();
        envelope.correlation_id = Some(correlation_id);

        // Create a oneshot channel for the reply.
        let (tx, rx) = tokio::sync::oneshot::channel();
        {
            let mut pending = self.pending_replies.lock().await;
            pending.insert(correlation_id, tx);
        }

        // Generate a reply subject and publish with it.
        let reply_subject = format!("_INBOX.{}", Uuid::new_v4());

        // Publish the request with the reply subject.
        let sender = self.get_or_create_sender(subject).await;
        let msg = InternalMessage {
            envelope,
            reply_subject: Some(reply_subject),
            queue_indices: HashMap::new(),
        };
        let _ = sender.send(msg);

        // Wait for the reply with timeout.
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => {
                // Oneshot sender dropped without sending.
                Err(TransportError::Timeout("reply sender dropped".into()))
            }
            Err(_) => {
                // Remove the pending reply entry on timeout.
                let mut pending = self.pending_replies.lock().await;
                pending.remove(&correlation_id);
                Err(TransportError::Timeout(format!(
                    "request to '{subject}' timed out after {timeout:?}"
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::MessageEnvelope;
    use crate::priority::MessagePriority;

    fn test_envelope(msg_type: &str) -> MessageEnvelope {
        MessageEnvelope::builder(msg_type)
            .payload_raw(b"test-payload".to_vec())
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn publish_subscribe_delivery() {
        let transport = InMemoryTransport::new();
        let mut sub = transport.subscribe("test.topic").await.unwrap();

        let envelope = test_envelope("test.msg");
        let msg_id = envelope.message_id;
        transport.publish("test.topic", envelope).await.unwrap();

        let received = tokio::time::timeout(Duration::from_secs(1), sub.next())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(received.envelope.message_id, msg_id);
        assert_eq!(received.envelope.message_type, "test.msg");
        assert!(received.reply_subject.is_none());
    }

    #[tokio::test]
    async fn queue_group_single_delivery() {
        let transport = InMemoryTransport::new();

        // Create 3 queue subscribers.
        let mut sub1 = transport
            .queue_subscribe("tasks.work", "workers")
            .await
            .unwrap();
        let mut sub2 = transport
            .queue_subscribe("tasks.work", "workers")
            .await
            .unwrap();
        let mut sub3 = transport
            .queue_subscribe("tasks.work", "workers")
            .await
            .unwrap();

        // Publish 3 messages — each should go to a different subscriber.
        for i in 0..3 {
            let envelope = test_envelope(&format!("task.{i}"));
            transport.publish("tasks.work", envelope).await.unwrap();
        }

        // Each subscriber should get exactly 1 message.
        let r1 = tokio::time::timeout(Duration::from_millis(100), sub1.next()).await;
        let r2 = tokio::time::timeout(Duration::from_millis(100), sub2.next()).await;
        let r3 = tokio::time::timeout(Duration::from_millis(100), sub3.next()).await;

        let mut received_count = 0;
        if r1.is_ok() && r1.unwrap().is_some() {
            received_count += 1;
        }
        if r2.is_ok() && r2.unwrap().is_some() {
            received_count += 1;
        }
        if r3.is_ok() && r3.unwrap().is_some() {
            received_count += 1;
        }
        assert_eq!(
            received_count, 3,
            "each of 3 messages should go to one subscriber"
        );
    }

    #[tokio::test]
    async fn queue_groups_on_same_subject_route_independently() {
        let transport = InMemoryTransport::new();

        // workers queue group
        let mut workers_1 = transport
            .queue_subscribe("tasks.shared", "workers")
            .await
            .unwrap();
        let mut workers_2 = transport
            .queue_subscribe("tasks.shared", "workers")
            .await
            .unwrap();

        // processors queue group
        let mut processors_1 = transport
            .queue_subscribe("tasks.shared", "processors")
            .await
            .unwrap();
        let mut processors_2 = transport
            .queue_subscribe("tasks.shared", "processors")
            .await
            .unwrap();

        for i in 0..4 {
            transport
                .publish("tasks.shared", test_envelope(&format!("task.shared.{i}")))
                .await
                .unwrap();
        }

        // Each queue group should receive all 4 messages split 2/2 across its subscribers.
        let mut workers_count = [0usize; 2];
        let mut processors_count = [0usize; 2];

        for _ in 0..2 {
            if tokio::time::timeout(Duration::from_millis(100), workers_1.next())
                .await
                .unwrap()
                .is_some()
            {
                workers_count[0] += 1;
            }
            if tokio::time::timeout(Duration::from_millis(100), workers_2.next())
                .await
                .unwrap()
                .is_some()
            {
                workers_count[1] += 1;
            }
            if tokio::time::timeout(Duration::from_millis(100), processors_1.next())
                .await
                .unwrap()
                .is_some()
            {
                processors_count[0] += 1;
            }
            if tokio::time::timeout(Duration::from_millis(100), processors_2.next())
                .await
                .unwrap()
                .is_some()
            {
                processors_count[1] += 1;
            }
        }

        assert_eq!(workers_count, [2, 2]);
        assert_eq!(processors_count, [2, 2]);
    }

    #[tokio::test]
    async fn request_reply_with_correlation_id() {
        let transport = InMemoryTransport::new();

        // Set up a responder.
        let transport_clone = transport.clone();
        let mut sub = transport.subscribe("service.echo").await.unwrap();

        tokio::spawn(async move {
            if let Some(msg) = sub.next().await {
                // Reply with the same payload but different message type.
                let response = MessageEnvelope::builder("echo.response")
                    .correlation_id(msg.envelope.correlation_id.unwrap())
                    .payload_raw(msg.envelope.payload.clone())
                    .build()
                    .unwrap();
                // Publish to the reply subject.
                if let Some(reply_subject) = msg.reply_subject {
                    transport_clone
                        .publish(&reply_subject, response)
                        .await
                        .unwrap();
                }
            }
        });

        // Send a request.
        let request = test_envelope("echo.request");
        let response = transport
            .request("service.echo", request, Duration::from_secs(5))
            .await
            .unwrap();

        assert_eq!(response.message_type, "echo.response");
    }

    #[tokio::test]
    async fn request_timeout() {
        let transport = InMemoryTransport::new();

        // No responder — should time out.
        let _sub = transport.subscribe("service.slow").await.unwrap();

        let request = test_envelope("slow.request");
        let result = transport
            .request("service.slow", request, Duration::from_millis(50))
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransportError::Timeout(_)));
    }

    #[tokio::test]
    async fn multiple_subscribers_all_receive() {
        let transport = InMemoryTransport::new();
        let mut sub1 = transport.subscribe("broadcast.topic").await.unwrap();
        let mut sub2 = transport.subscribe("broadcast.topic").await.unwrap();

        let envelope = test_envelope("broadcast.msg");
        let msg_id = envelope.message_id;
        transport
            .publish("broadcast.topic", envelope)
            .await
            .unwrap();

        let r1 = tokio::time::timeout(Duration::from_millis(100), sub1.next())
            .await
            .unwrap()
            .unwrap();
        let r2 = tokio::time::timeout(Duration::from_millis(100), sub2.next())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(r1.envelope.message_id, msg_id);
        assert_eq!(r2.envelope.message_id, msg_id);
    }

    #[tokio::test]
    async fn publish_to_subject_with_no_subscribers_succeeds() {
        let transport = InMemoryTransport::new();
        let result = transport
            .publish("no.listeners", test_envelope("orphan"))
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn envelope_priority_preserved() {
        let transport = InMemoryTransport::new();
        let mut sub = transport.subscribe("priority.test").await.unwrap();

        let envelope = MessageEnvelope::builder("priority.msg")
            .priority(MessagePriority::Critical)
            .build()
            .unwrap();
        transport.publish("priority.test", envelope).await.unwrap();

        let received = tokio::time::timeout(Duration::from_millis(100), sub.next())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(received.envelope.priority, MessagePriority::Critical);
    }
}
