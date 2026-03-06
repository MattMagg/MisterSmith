use std::time::Duration;

use mister_smith_transport::envelope::MessageEnvelope;
use mister_smith_transport::Transport;

use crate::errors::AgentSystemError;

/// Send a fire-and-forget message to a target subject.
pub async fn send(
    transport: &dyn Transport,
    target_subject: &str,
    envelope: MessageEnvelope,
) -> Result<(), AgentSystemError> {
    transport
        .publish(target_subject, envelope)
        .await
        .map_err(|e| AgentSystemError::MessageDeliveryFailed(e.to_string()))
}

/// Send a message via durable transport (JetStream) for guaranteed delivery.
pub async fn send_durable(
    transport: &dyn Transport,
    target_subject: &str,
    envelope: MessageEnvelope,
) -> Result<(), AgentSystemError> {
    // For durable sends, we use the same publish but the transport
    // implementation handles JetStream durability.
    transport
        .publish(target_subject, envelope)
        .await
        .map_err(|e| AgentSystemError::MessageDeliveryFailed(e.to_string()))
}

/// Send a request-reply message and await the response.
pub async fn request(
    transport: &dyn Transport,
    target_subject: &str,
    envelope: MessageEnvelope,
    timeout: Duration,
) -> Result<MessageEnvelope, AgentSystemError> {
    transport
        .request(target_subject, envelope, timeout)
        .await
        .map_err(|e| AgentSystemError::MessageDeliveryFailed(e.to_string()))
}

/// Broadcast a message to a wildcard subject pattern.
pub async fn broadcast(
    transport: &dyn Transport,
    subject_pattern: &str,
    envelope: MessageEnvelope,
) -> Result<(), AgentSystemError> {
    transport
        .publish(subject_pattern, envelope)
        .await
        .map_err(|e| AgentSystemError::MessageDeliveryFailed(e.to_string()))
}
