//! Transport error types shared by all transport implementations.

use thiserror::Error;

/// Errors that can occur during transport operations.
#[derive(Debug, Error)]
pub enum TransportError {
    /// Failed to establish or maintain a transport connection.
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    /// Serialization of a message payload failed.
    #[error("serialization error: {0}")]
    SerializationError(String),

    /// Deserialization of a message payload failed.
    #[error("deserialization error: {0}")]
    DeserializationError(String),

    /// A transport operation timed out.
    #[error("operation timed out: {0}")]
    Timeout(String),

    /// The subject string is invalid for the transport protocol.
    #[error("invalid subject: {0}")]
    SubjectInvalid(String),

    /// The message payload exceeds the maximum allowed size.
    #[error("payload too large: {size} bytes exceeds limit of {limit} bytes")]
    PayloadTooLarge {
        /// Actual payload size in bytes.
        size: usize,
        /// Maximum allowed size in bytes.
        limit: usize,
    },

    /// Failed to create or manage a subscription.
    #[error("subscription error: {0}")]
    SubscriptionError(String),

    /// Failed to publish a message.
    #[error("publish error: {0}")]
    PublishError(String),

    /// A protocol-level error occurred.
    #[error("protocol error: {0}")]
    ProtocolError(String),
}

impl From<TransportError> for mister_smith_core::NetworkError {
    fn from(err: TransportError) -> Self {
        match err {
            TransportError::ConnectionFailed(msg) => {
                mister_smith_core::NetworkError::ConnectionFailed(msg)
            }
            TransportError::Timeout(msg) => mister_smith_core::NetworkError::Timeout(msg),
            other => mister_smith_core::NetworkError::ProtocolError(other.to_string()),
        }
    }
}

impl From<mister_smith_core::NetworkError> for TransportError {
    fn from(err: mister_smith_core::NetworkError) -> Self {
        match err {
            mister_smith_core::NetworkError::ConnectionFailed(msg) => {
                TransportError::ConnectionFailed(msg)
            }
            mister_smith_core::NetworkError::Timeout(msg) => TransportError::Timeout(msg),
            mister_smith_core::NetworkError::ProtocolError(msg) => {
                TransportError::ProtocolError(msg)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_error_display() {
        let err = TransportError::ConnectionFailed("host unreachable".into());
        assert_eq!(err.to_string(), "connection failed: host unreachable");

        let err = TransportError::PayloadTooLarge {
            size: 2_000_000,
            limit: 1_000_000,
        };
        assert!(err.to_string().contains("2000000"));
        assert!(err.to_string().contains("1000000"));
    }

    #[test]
    fn transport_to_network_error() {
        let transport_err = TransportError::ConnectionFailed("test".into());
        let network_err: mister_smith_core::NetworkError = transport_err.into();
        assert!(matches!(
            network_err,
            mister_smith_core::NetworkError::ConnectionFailed(_)
        ));
    }

    #[test]
    fn network_to_transport_error() {
        let network_err = mister_smith_core::NetworkError::Timeout("test".into());
        let transport_err: TransportError = network_err.into();
        assert!(matches!(transport_err, TransportError::Timeout(_)));
    }
}
