//! Durable transport abstraction for acknowledged messaging.
//!
//! Extends the fire-and-forget [`Transport`] trait with JetStream-style
//! durable consumer semantics: explicit acknowledgment, negative
//! acknowledgment with redelivery delay, and message termination.
//!
//! The [`DurableTransport`] trait is implemented by transports that
//! support persistent, at-least-once delivery (e.g., NATS JetStream).

use async_trait::async_trait;
use std::pin::Pin;
use std::time::Duration;

use futures::Stream;

use crate::envelope::MessageEnvelope;
use crate::errors::TransportError;

/// Acknowledgment operations for a durable message.
///
/// Transport backends implement this trait to bridge the generic
/// interface to their protocol-specific ack mechanism (e.g., JetStream
/// `Message::ack()`).
#[async_trait]
pub trait MessageAcker: Send + Sync {
    /// Acknowledge successful processing. The message will not be redelivered.
    async fn ack(&self) -> Result<(), TransportError>;

    /// Negatively acknowledge. The message will be redelivered after `delay`
    /// (or immediately if `None`).
    async fn nak(&self, delay: Option<Duration>) -> Result<(), TransportError>;

    /// Terminate processing. The message will not be redelivered and is
    /// marked as terminally failed.
    async fn term(&self) -> Result<(), TransportError>;

    /// Signal that processing is still in progress, resetting the ack timeout.
    async fn in_progress(&self) -> Result<(), TransportError>;
}

/// A message received from a durable subscription with acknowledgment control.
///
/// Unlike [`ReceivedMessage`](crate::ReceivedMessage), a `DurableMessage`
/// **must** be explicitly acknowledged after processing. Dropping without
/// acking causes redelivery after the consumer's ack timeout.
pub struct DurableMessage {
    /// The message envelope containing headers, metadata, and payload.
    pub envelope: MessageEnvelope,

    /// Protocol-specific reply address.
    pub reply_subject: Option<String>,

    /// Acknowledgment handle.
    acker: Box<dyn MessageAcker>,
}

impl DurableMessage {
    /// Create a new durable message.
    pub fn new(
        envelope: MessageEnvelope,
        reply_subject: Option<String>,
        acker: impl MessageAcker + 'static,
    ) -> Self {
        Self {
            envelope,
            reply_subject,
            acker: Box::new(acker),
        }
    }

    /// Acknowledge successful processing.
    pub async fn ack(&self) -> Result<(), TransportError> {
        self.acker.ack().await
    }

    /// Negatively acknowledge with optional redelivery delay.
    pub async fn nak(&self, delay: Option<Duration>) -> Result<(), TransportError> {
        self.acker.nak(delay).await
    }

    /// Terminate processing (no redelivery).
    pub async fn term(&self) -> Result<(), TransportError> {
        self.acker.term().await
    }

    /// Signal in-progress (reset ack timeout).
    pub async fn in_progress(&self) -> Result<(), TransportError> {
        self.acker.in_progress().await
    }
}

impl std::fmt::Debug for DurableMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DurableMessage")
            .field("envelope", &self.envelope)
            .field("reply_subject", &self.reply_subject)
            .finish_non_exhaustive()
    }
}

/// A stream of durable messages with acknowledgment semantics.
pub struct DurableSubscription {
    inner: Pin<Box<dyn Stream<Item = DurableMessage> + Send>>,
}

impl DurableSubscription {
    /// Create a new durable subscription from an async stream.
    pub fn new(stream: impl Stream<Item = DurableMessage> + Send + 'static) -> Self {
        Self {
            inner: Box::pin(stream),
        }
    }

    /// Get the next durable message.
    pub async fn next(&mut self) -> Option<DurableMessage> {
        use futures::StreamExt;
        self.inner.next().await
    }

    /// Convert into the underlying pinned stream.
    pub fn into_stream(self) -> Pin<Box<dyn Stream<Item = DurableMessage> + Send>> {
        self.inner
    }
}

impl std::fmt::Debug for DurableSubscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DurableSubscription")
            .finish_non_exhaustive()
    }
}

/// Transport with durable messaging and acknowledgment semantics.
///
/// Extends [`Transport`](crate::Transport) with operations backed by
/// persistent streams (e.g., NATS JetStream). Messages delivered through
/// a `DurableSubscription` must be explicitly acknowledged.
///
/// # Delivery Guarantee
///
/// At-least-once delivery. Messages are redelivered if not acknowledged
/// within the consumer's ack timeout. Exactly-once semantics are achieved
/// at the application layer via idempotent processing keyed by `message_id`.
///
/// # Publisher Deduplication
///
/// Publishers should set `MessageEnvelope.message_id` to enable server-side
/// deduplication within the stream's dedup window.
#[async_trait]
pub trait DurableTransport: crate::Transport {
    /// Publish a message with durable persistence.
    ///
    /// Waits for server acknowledgment that the message has been persisted
    /// to the stream. Returns an error if persistence fails.
    async fn durable_publish(
        &self,
        subject: &str,
        envelope: MessageEnvelope,
    ) -> Result<(), TransportError>;

    /// Subscribe to a subject with durable consumer semantics.
    ///
    /// Creates or attaches to a named durable consumer. Messages are
    /// delivered via pull semantics and must be explicitly acknowledged.
    ///
    /// `stream_name` identifies the JetStream stream.
    /// `consumer_name` is the durable consumer name (persists across reconnects).
    async fn durable_subscribe(
        &self,
        stream_name: &str,
        consumer_name: &str,
        filter_subject: &str,
    ) -> Result<DurableSubscription, TransportError>;
}
