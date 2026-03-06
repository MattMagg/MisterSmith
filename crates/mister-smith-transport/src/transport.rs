//! Protocol-agnostic transport trait and supporting types.
//!
//! The `Transport` trait defines the unified communication contract for all
//! transport implementations (NATS, HTTP, gRPC, MCP). Reply semantics are
//! protocol-specific: responders use `transport.publish()` to the
//! `reply_subject` from `ReceivedMessage`.

use async_trait::async_trait;
use std::pin::Pin;
use std::time::Duration;

use futures::Stream;

use crate::envelope::MessageEnvelope;
use crate::errors::TransportError;

/// A message received from a transport subscription.
///
/// Wraps the `MessageEnvelope` with an optional `reply_subject` that carries
/// the protocol-specific reply address (e.g., NATS reply subject). For protocols
/// where replies are implicit (HTTP response, gRPC return value), this is `None`.
#[derive(Debug)]
pub struct ReceivedMessage {
    /// The message envelope containing headers, metadata, and payload.
    pub envelope: MessageEnvelope,

    /// Protocol-specific reply address. Populated by NATS from the incoming
    /// message's reply field. `None` for HTTP/gRPC where replies are implicit.
    pub reply_subject: Option<String>,
}

/// A stream of received messages from a subscription.
pub struct Subscription {
    inner: Pin<Box<dyn Stream<Item = ReceivedMessage> + Send>>,
}

impl Subscription {
    /// Create a new subscription from an async stream.
    pub fn new(stream: impl Stream<Item = ReceivedMessage> + Send + 'static) -> Self {
        Self {
            inner: Box::pin(stream),
        }
    }

    /// Get the next message from the subscription.
    pub async fn next(&mut self) -> Option<ReceivedMessage> {
        use futures::StreamExt;
        self.inner.next().await
    }

    /// Convert into the underlying pinned stream.
    pub fn into_stream(self) -> Pin<Box<dyn Stream<Item = ReceivedMessage> + Send>> {
        self.inner
    }
}

impl std::fmt::Debug for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subscription").finish_non_exhaustive()
    }
}

/// Protocol-agnostic transport interface.
///
/// Provides four core messaging operations: publish, subscribe, queue subscribe,
/// and request-reply. Implementations exist for NATS (`NatsTransport`),
/// in-memory testing (`InMemoryTransport`), and others.
///
/// Reply semantics are protocol-specific. When a `ReceivedMessage` has a
/// `reply_subject`, the responder should call `transport.publish(reply_subject, response)`.
#[async_trait]
pub trait Transport: Send + Sync + 'static {
    /// Publish a message to a subject.
    async fn publish(&self, subject: &str, envelope: MessageEnvelope)
        -> Result<(), TransportError>;

    /// Subscribe to messages on a subject.
    async fn subscribe(&self, subject: &str) -> Result<Subscription, TransportError>;

    /// Subscribe to a subject with a queue group for load-balanced delivery.
    ///
    /// Only one subscriber in the queue group receives each message.
    async fn queue_subscribe(
        &self,
        subject: &str,
        queue: &str,
    ) -> Result<Subscription, TransportError>;

    /// Send a request and wait for a response with timeout.
    ///
    /// Creates a temporary subscription, publishes the request with a reply
    /// subject, and waits for the response envelope.
    async fn request(
        &self,
        subject: &str,
        envelope: MessageEnvelope,
        timeout: Duration,
    ) -> Result<MessageEnvelope, TransportError>;
}
