//! NATS-specific error types.

use mister_smith_transport::TransportError;
use thiserror::Error;

/// Errors specific to the NATS transport implementation.
#[derive(Debug, Error)]
pub enum NatsError {
    /// Failed to connect to NATS server.
    #[error("NATS connection failed: {0}")]
    ConnectionFailed(String),

    /// Subscription operation failed.
    #[error("NATS subscription error: {0}")]
    SubscriptionFailed(String),

    /// Publish operation failed.
    #[error("NATS publish error: {0}")]
    PublishFailed(String),

    /// JetStream operation failed.
    #[error("JetStream error: {0}")]
    JetStreamError(String),

    /// Subject routing error.
    #[error("subject routing error: {0}")]
    SubjectRoutingError(String),

    /// Client not connected.
    #[error("NATS client not connected")]
    NotConnected,
}

impl From<async_nats::ConnectError> for NatsError {
    fn from(err: async_nats::ConnectError) -> Self {
        NatsError::ConnectionFailed(err.to_string())
    }
}

impl From<async_nats::PublishError> for NatsError {
    fn from(err: async_nats::PublishError) -> Self {
        NatsError::PublishFailed(err.to_string())
    }
}

impl From<async_nats::SubscribeError> for NatsError {
    fn from(err: async_nats::SubscribeError) -> Self {
        NatsError::SubscriptionFailed(err.to_string())
    }
}

impl From<async_nats::RequestError> for NatsError {
    fn from(err: async_nats::RequestError) -> Self {
        match &err.kind() {
            async_nats::RequestErrorKind::TimedOut => {
                NatsError::PublishFailed(format!("request timed out: {err}"))
            }
            _ => NatsError::PublishFailed(err.to_string()),
        }
    }
}

impl From<NatsError> for TransportError {
    fn from(err: NatsError) -> Self {
        match err {
            NatsError::ConnectionFailed(msg) => TransportError::ConnectionFailed(msg),
            NatsError::SubscriptionFailed(msg) => TransportError::SubscriptionError(msg),
            NatsError::PublishFailed(msg) => TransportError::PublishError(msg),
            NatsError::JetStreamError(msg) => TransportError::ProtocolError(msg),
            NatsError::SubjectRoutingError(msg) => TransportError::SubjectInvalid(msg),
            NatsError::NotConnected => {
                TransportError::ConnectionFailed("NATS client not connected".into())
            }
        }
    }
}

impl From<TransportError> for NatsError {
    fn from(err: TransportError) -> Self {
        match err {
            TransportError::ConnectionFailed(msg) => NatsError::ConnectionFailed(msg),
            TransportError::SubscriptionError(msg) => NatsError::SubscriptionFailed(msg),
            TransportError::PublishError(msg) => NatsError::PublishFailed(msg),
            TransportError::SubjectInvalid(msg) => NatsError::SubjectRoutingError(msg),
            other => NatsError::PublishFailed(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nats_error_to_transport_error() {
        let nats_err = NatsError::ConnectionFailed("host unreachable".into());
        let transport_err: TransportError = nats_err.into();
        assert!(matches!(transport_err, TransportError::ConnectionFailed(_)));
    }

    #[test]
    fn transport_error_to_nats_error() {
        let transport_err = TransportError::SubscriptionError("topic gone".into());
        let nats_err: NatsError = transport_err.into();
        assert!(matches!(nats_err, NatsError::SubscriptionFailed(_)));
    }
}
