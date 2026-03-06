//! NatsTransport — Transport trait implementation over async-nats 0.46.

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use mister_smith_transport::{
    DurableMessage, DurableSubscription, DurableTransport, MessageAcker, MessageEnvelope,
    ReceivedMessage, Subscription, Transport, TransportError,
};

use crate::config::NatsTransportConfig;
use crate::errors::NatsError;

/// NATS transport for inter-agent communication.
///
/// Wraps an `async_nats::Client` and implements the `Transport` trait.
/// The client is cheap to clone (Arc internals) — all operations go through
/// a bounded channel to a single connection handler task.
#[derive(Clone)]
pub struct NatsTransport {
    client: Arc<RwLock<Option<async_nats::Client>>>,
    config: NatsTransportConfig,
}

impl NatsTransport {
    /// Create a new NatsTransport (not yet connected).
    pub fn new(config: NatsTransportConfig) -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// Connect to the NATS server using the configured options.
    pub async fn connect(&self) -> Result<(), NatsError> {
        let urls = self.config.server_urls.join(",");

        let mut opts = async_nats::ConnectOptions::new()
            .name(&self.config.name)
            .connection_timeout(self.config.connection_timeout)
            .client_capacity(self.config.client_capacity)
            .subscription_capacity(self.config.subscription_capacity)
            .event_callback(|event| async move {
                match event {
                    async_nats::Event::Connected => {
                        info!("NATS connected");
                    }
                    async_nats::Event::Disconnected => {
                        warn!("NATS disconnected");
                    }
                    async_nats::Event::LameDuckMode => {
                        warn!("NATS server entering lame duck mode");
                    }
                    async_nats::Event::SlowConsumer(sid) => {
                        warn!(?sid, "NATS slow consumer detected");
                    }
                    _ => {
                        debug!("NATS event received");
                    }
                }
            });

        if let Some(max) = self.config.max_reconnects {
            opts = opts.max_reconnects(max);
        }

        let client = async_nats::connect_with_options(&urls, opts).await?;
        info!(urls = %urls, name = %self.config.name, "NATS transport connected");

        let mut guard = self.client.write().await;
        *guard = Some(client);
        Ok(())
    }

    /// Get a reference to the underlying NATS client.
    async fn get_client(&self) -> Result<async_nats::Client, NatsError> {
        let guard = self.client.read().await;
        guard.clone().ok_or(NatsError::NotConnected)
    }

    /// Get the current connection state.
    pub async fn connection_state(&self) -> async_nats::connection::State {
        match self.client.read().await.as_ref() {
            Some(client) => client.connection_state(),
            None => async_nats::connection::State::Disconnected,
        }
    }

    /// Get a reference to the underlying async-nats client (for JetStream).
    pub async fn inner_client(&self) -> Result<async_nats::Client, NatsError> {
        self.get_client().await
    }

    /// Disconnect from the NATS server.
    pub async fn disconnect(&self) -> Result<(), NatsError> {
        let mut guard = self.client.write().await;
        if let Some(_client) = guard.take() {
            info!("NATS transport disconnected");
        }
        Ok(())
    }
}

#[async_trait]
impl Transport for NatsTransport {
    async fn publish(
        &self,
        subject: &str,
        mut envelope: MessageEnvelope,
    ) -> Result<(), TransportError> {
        let client = self.get_client().await.map_err(TransportError::from)?;

        // Inject W3C trace context into the envelope before publishing
        mister_smith_transport::inject_trace_context(&mut envelope);

        let payload = envelope.to_bytes()?;

        client
            .publish(subject.to_string(), payload)
            .await
            .map_err(|e| TransportError::PublishError(e.to_string()))?;

        Ok(())
    }

    async fn subscribe(&self, subject: &str) -> Result<Subscription, TransportError> {
        let client = self.get_client().await.map_err(TransportError::from)?;

        let subscriber = client
            .subscribe(subject.to_string())
            .await
            .map_err(|e| TransportError::SubscriptionError(e.to_string()))?;

        let stream = async_stream::stream! {
            use futures::StreamExt;
            let mut subscriber = subscriber;
            while let Some(msg) = subscriber.next().await {
                match MessageEnvelope::from_bytes(&msg.payload) {
                    Ok(envelope) => {
                        // Extract W3C trace context for span correlation
                        if let Some(traceparent) = mister_smith_transport::extract_trace_context(&envelope) {
                            debug!(traceparent, "Extracted trace context from received message");
                        }
                        yield ReceivedMessage {
                            envelope,
                            reply_subject: msg.reply.map(|s| s.to_string()),
                        };
                    }
                    Err(e) => {
                        error!("Failed to deserialize NATS message: {e}");
                    }
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
        let client = self.get_client().await.map_err(TransportError::from)?;

        let subscriber = client
            .queue_subscribe(subject.to_string(), queue.to_string())
            .await
            .map_err(|e| TransportError::SubscriptionError(e.to_string()))?;

        let stream = async_stream::stream! {
            use futures::StreamExt;
            let mut subscriber = subscriber;
            while let Some(msg) = subscriber.next().await {
                match MessageEnvelope::from_bytes(&msg.payload) {
                    Ok(envelope) => {
                        if let Some(traceparent) = mister_smith_transport::extract_trace_context(&envelope) {
                            debug!(traceparent, "Extracted trace context from queue message");
                        }
                        yield ReceivedMessage {
                            envelope,
                            reply_subject: msg.reply.map(|s| s.to_string()),
                        };
                    }
                    Err(e) => {
                        error!("Failed to deserialize NATS message: {e}");
                    }
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
        let client = self.get_client().await.map_err(TransportError::from)?;

        // Inject W3C trace context before sending request
        mister_smith_transport::inject_trace_context(&mut envelope);

        let payload = envelope.to_bytes()?;

        let request = async_nats::Request::new()
            .timeout(Some(timeout))
            .payload(payload);

        let response = client
            .send_request(subject.to_string(), request)
            .await
            .map_err(|e| {
                if matches!(e.kind(), async_nats::RequestErrorKind::TimedOut) {
                    TransportError::Timeout(format!(
                        "request to '{subject}' timed out after {timeout:?}"
                    ))
                } else {
                    TransportError::PublishError(e.to_string())
                }
            })?;

        MessageEnvelope::from_bytes(&response.payload)
    }
}

// ---------------------------------------------------------------------------
// Durable transport (JetStream-backed)
// ---------------------------------------------------------------------------

/// Acknowledgment handle wrapping a JetStream message.
struct JetStreamAcker {
    message: async_nats::jetstream::Message,
}

#[async_trait]
impl MessageAcker for JetStreamAcker {
    async fn ack(&self) -> Result<(), TransportError> {
        self.message
            .ack()
            .await
            .map_err(|e| TransportError::ProtocolError(format!("ack failed: {e}")))
    }

    async fn nak(&self, delay: Option<Duration>) -> Result<(), TransportError> {
        let ack_kind = match delay {
            Some(d) => async_nats::jetstream::AckKind::Nak(Some(d)),
            None => async_nats::jetstream::AckKind::Nak(None),
        };
        self.message
            .ack_with(ack_kind)
            .await
            .map_err(|e| TransportError::ProtocolError(format!("nak failed: {e}")))
    }

    async fn term(&self) -> Result<(), TransportError> {
        self.message
            .ack_with(async_nats::jetstream::AckKind::Term)
            .await
            .map_err(|e| TransportError::ProtocolError(format!("term failed: {e}")))
    }

    async fn in_progress(&self) -> Result<(), TransportError> {
        self.message
            .ack_with(async_nats::jetstream::AckKind::Progress)
            .await
            .map_err(|e| TransportError::ProtocolError(format!("in_progress failed: {e}")))
    }
}

#[async_trait]
impl DurableTransport for NatsTransport {
    async fn durable_publish(
        &self,
        subject: &str,
        mut envelope: MessageEnvelope,
    ) -> Result<(), TransportError> {
        let client = self.get_client().await.map_err(TransportError::from)?;
        let js = async_nats::jetstream::new(client);

        // Inject W3C trace context before durable publish
        mister_smith_transport::inject_trace_context(&mut envelope);

        let payload = envelope.to_bytes()?;

        // Set MsgId header for server-side deduplication using the envelope's message_id.
        let mut headers = async_nats::HeaderMap::new();
        headers.insert("Nats-Msg-Id", envelope.message_id.to_string().as_str());

        let ack_future = js
            .publish_with_headers(subject.to_string(), headers, payload)
            .await
            .map_err(|e| TransportError::PublishError(format!("durable publish failed: {e}")))?;

        // Wait for server persistence acknowledgment.
        ack_future
            .await
            .map_err(|e| TransportError::PublishError(format!("publish ack failed: {e}")))?;

        debug!(subject, "durable message published and acknowledged");
        Ok(())
    }

    async fn durable_subscribe(
        &self,
        stream_name: &str,
        consumer_name: &str,
        filter_subject: &str,
    ) -> Result<DurableSubscription, TransportError> {
        let client = self.get_client().await.map_err(TransportError::from)?;
        let js = async_nats::jetstream::new(client);

        let stream = js.get_stream(stream_name).await.map_err(|e| {
            TransportError::SubscriptionError(format!("stream '{stream_name}' not found: {e}"))
        })?;

        let consumer_config = async_nats::jetstream::consumer::pull::Config {
            durable_name: Some(consumer_name.to_string()),
            filter_subject: filter_subject.to_string(),
            ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
            ack_wait: self.config.jetstream.ack_timeout,
            max_ack_pending: self.config.jetstream.max_ack_inflight as i64,
            ..Default::default()
        };

        let consumer = stream
            .get_or_create_consumer(consumer_name, consumer_config)
            .await
            .map_err(|e| {
                TransportError::SubscriptionError(format!(
                    "consumer '{consumer_name}' creation failed: {e}"
                ))
            })?;

        let messages = consumer.messages().await.map_err(|e| {
            TransportError::SubscriptionError(format!("message stream failed: {e}"))
        })?;

        let stream = async_stream::stream! {
            use futures::StreamExt;
            let mut messages = messages;
            while let Some(result) = messages.next().await {
                match result {
                    Ok(js_msg) => {
                        match MessageEnvelope::from_bytes(&js_msg.payload) {
                            Ok(envelope) => {
                                let reply_subject = js_msg.reply.as_ref().map(|s| s.to_string());
                                let acker = JetStreamAcker { message: js_msg };
                                yield DurableMessage::new(envelope, reply_subject, acker);
                            }
                            Err(e) => {
                                error!("Failed to deserialize JetStream message: {e}");
                                // Term malformed messages to prevent infinite redelivery.
                                if let Err(term_err) = js_msg.ack_with(
                                    async_nats::jetstream::AckKind::Term
                                ).await {
                                    error!("Failed to term malformed message: {term_err}");
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("JetStream message stream error: {e}");
                    }
                }
            }
        };

        info!(
            stream = stream_name,
            consumer = consumer_name,
            filter = filter_subject,
            "durable subscription created"
        );

        Ok(DurableSubscription::new(stream))
    }
}
