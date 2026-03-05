//! JetStream stream and consumer management.
//!
//! Wraps `async_nats::jetstream::Context` for durable messaging with
//! stream creation, consumer configuration, and explicit acknowledgment.

use async_nats::jetstream;
use bytes::Bytes;
use tracing::info;

use crate::config::JetStreamConfig;
use crate::errors::NatsError;
use mister_smith_transport::MessageEnvelope;

/// Manager for JetStream streams and consumers.
pub struct JetStreamManager {
    context: jetstream::Context,
    _config: JetStreamConfig,
}

impl JetStreamManager {
    /// Create a JetStream manager from an existing NATS client.
    pub fn new(client: async_nats::Client, config: JetStreamConfig) -> Self {
        let context = jetstream::new(client);
        Self {
            context,
            _config: config,
        }
    }

    /// Get a reference to the underlying JetStream context.
    pub fn context(&self) -> &jetstream::Context {
        &self.context
    }

    /// Create or update a stream.
    pub async fn create_stream(
        &self,
        name: &str,
        subjects: Vec<String>,
        retention: jetstream::stream::RetentionPolicy,
    ) -> Result<jetstream::stream::Stream, NatsError> {
        let config = jetstream::stream::Config {
            name: name.to_string(),
            subjects,
            retention,
            ..Default::default()
        };

        let stream = self
            .context
            .get_or_create_stream(config)
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        info!(stream = %name, "JetStream stream created/updated");
        Ok(stream)
    }

    /// Delete a stream.
    pub async fn delete_stream(&self, name: &str) -> Result<(), NatsError> {
        self.context
            .delete_stream(name)
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        info!(stream = %name, "JetStream stream deleted");
        Ok(())
    }

    /// Publish a message to JetStream (durable).
    ///
    /// Returns after the server acknowledges persistence.
    /// Uses the double-await pattern: first await sends, second await gets ack.
    pub async fn publish(
        &self,
        subject: &str,
        envelope: MessageEnvelope,
    ) -> Result<jetstream::context::PublishAckFuture, NatsError> {
        let payload = envelope
            .to_bytes()
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        let ack_future = self
            .context
            .publish(subject.to_string(), payload)
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        Ok(ack_future)
    }

    /// Publish and wait for acknowledgment in one call.
    pub async fn publish_and_ack(
        &self,
        subject: &str,
        envelope: MessageEnvelope,
    ) -> Result<(), NatsError> {
        let ack_future = self.publish(subject, envelope).await?;
        ack_future
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;
        Ok(())
    }

    /// Create a pull consumer on a stream.
    pub async fn create_pull_consumer(
        &self,
        stream_name: &str,
        consumer_config: jetstream::consumer::pull::Config,
    ) -> Result<jetstream::consumer::Consumer<jetstream::consumer::pull::Config>, NatsError> {
        let stream = self
            .context
            .get_stream(stream_name)
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        let consumer_name = consumer_config
            .durable_name
            .clone()
            .unwrap_or_else(|| "default".to_string());
        let consumer = stream
            .get_or_create_consumer(&consumer_name, consumer_config)
            .await
            .map_err(|e| NatsError::JetStreamError(e.to_string()))?;

        Ok(consumer)
    }

    /// Deserialize a JetStream message payload into a MessageEnvelope.
    pub fn decode_message(payload: &Bytes) -> Result<MessageEnvelope, NatsError> {
        MessageEnvelope::from_bytes(payload)
            .map_err(|e| NatsError::JetStreamError(format!("decode failed: {e}")))
    }
}
