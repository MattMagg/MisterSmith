//! NatsTransport — Transport trait implementation over async-nats 0.46.

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use mister_smith_transport::{
    MessageEnvelope, ReceivedMessage, Subscription, Transport, TransportError,
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
        envelope: MessageEnvelope,
    ) -> Result<(), TransportError> {
        let client = self.get_client().await.map_err(TransportError::from)?;
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
        envelope: MessageEnvelope,
        timeout: Duration,
    ) -> Result<MessageEnvelope, TransportError> {
        let client = self.get_client().await.map_err(TransportError::from)?;
        let payload = envelope.to_bytes()?;

        let request = async_nats::Request::new().timeout(Some(timeout)).payload(payload);

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
