//! Bidirectional mapping between framework transport errors and gRPC status codes.
//!
//! Every [`TransportError`] variant has an unambiguous mapping to a [`tonic::Code`],
//! and every relevant `Code` maps back to a `TransportError`. The mapping is
//! exhaustively tested in both directions.

use thiserror::Error;
use tonic::{Code, Status};

/// Transport layer errors.
///
/// This enum will eventually live in `mister-smith-transport`. It is defined
/// here temporarily so that the gRPC crate can be built and tested independently.
#[derive(Debug, Error)]
pub enum TransportError {
    /// Operation timed out.
    #[error("transport timeout: {0}")]
    Timeout(String),

    /// Failed to establish or maintain a connection.
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    /// Subject or topic name is invalid.
    #[error("invalid subject: {0}")]
    SubjectInvalid(String),

    /// Message exceeds the maximum allowed size.
    #[error("payload too large: {0}")]
    PayloadTooLarge(String),

    /// Failed to serialize a message for transport.
    #[error("serialization error: {0}")]
    SerializationError(String),

    /// Failed to deserialize a received message.
    #[error("deserialization error: {0}")]
    DeserializationError(String),

    /// Subscription operation failed.
    #[error("subscription error: {0}")]
    SubscriptionError(String),

    /// Publish operation failed.
    #[error("publish error: {0}")]
    PublishError(String),

    /// Protocol-level error.
    #[error("protocol error: {0}")]
    ProtocolError(String),
}

// ---------------------------------------------------------------------------
// TransportError -> tonic::Status
// ---------------------------------------------------------------------------

impl From<TransportError> for Status {
    fn from(err: TransportError) -> Self {
        match &err {
            TransportError::Timeout(_) => Status::new(Code::DeadlineExceeded, err.to_string()),
            TransportError::ConnectionFailed(_) => {
                Status::new(Code::Unavailable, err.to_string())
            }
            TransportError::SubjectInvalid(_) => {
                Status::new(Code::InvalidArgument, err.to_string())
            }
            TransportError::PayloadTooLarge(_) => {
                Status::new(Code::ResourceExhausted, err.to_string())
            }
            TransportError::SerializationError(_)
            | TransportError::DeserializationError(_)
            | TransportError::SubscriptionError(_)
            | TransportError::PublishError(_)
            | TransportError::ProtocolError(_) => Status::new(Code::Internal, err.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// tonic::Status -> TransportError
// ---------------------------------------------------------------------------

impl From<Status> for TransportError {
    fn from(status: Status) -> Self {
        let msg = status.message().to_string();
        match status.code() {
            Code::DeadlineExceeded => TransportError::Timeout(msg),
            Code::Unavailable => TransportError::ConnectionFailed(msg),
            Code::InvalidArgument => TransportError::SubjectInvalid(msg),
            Code::ResourceExhausted => TransportError::PayloadTooLarge(msg),
            // All other codes map to ProtocolError as a catch-all.
            _ => TransportError::ProtocolError(msg),
        }
    }
}

// ---------------------------------------------------------------------------
// Convenience conversion from core NetworkError
// ---------------------------------------------------------------------------

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

    // -----------------------------------------------------------------------
    // TransportError -> tonic::Status
    // -----------------------------------------------------------------------

    #[test]
    fn timeout_to_deadline_exceeded() {
        let err = TransportError::Timeout("request timed out".into());
        let status: Status = err.into();
        assert_eq!(status.code(), Code::DeadlineExceeded);
        assert!(status.message().contains("timed out"));
    }

    #[test]
    fn connection_failed_to_unavailable() {
        let err = TransportError::ConnectionFailed("host unreachable".into());
        let status: Status = err.into();
        assert_eq!(status.code(), Code::Unavailable);
        assert!(status.message().contains("unreachable"));
    }

    #[test]
    fn subject_invalid_to_invalid_argument() {
        let err = TransportError::SubjectInvalid("bad subject".into());
        let status: Status = err.into();
        assert_eq!(status.code(), Code::InvalidArgument);
        assert!(status.message().contains("bad subject"));
    }

    #[test]
    fn payload_too_large_to_resource_exhausted() {
        let err = TransportError::PayloadTooLarge("exceeds 4MB".into());
        let status: Status = err.into();
        assert_eq!(status.code(), Code::ResourceExhausted);
        assert!(status.message().contains("4MB"));
    }

    #[test]
    fn serialization_error_to_internal() {
        let err = TransportError::SerializationError("failed to encode".into());
        let status: Status = err.into();
        assert_eq!(status.code(), Code::Internal);
        assert!(status.message().contains("encode"));
    }

    #[test]
    fn deserialization_error_to_internal() {
        let err = TransportError::DeserializationError("invalid payload".into());
        let status: Status = err.into();
        assert_eq!(status.code(), Code::Internal);
        assert!(status.message().contains("invalid payload"));
    }

    #[test]
    fn subscription_error_to_internal() {
        let err = TransportError::SubscriptionError("subscribe failed".into());
        let status: Status = err.into();
        assert_eq!(status.code(), Code::Internal);
        assert!(status.message().contains("subscribe"));
    }

    #[test]
    fn publish_error_to_internal() {
        let err = TransportError::PublishError("publish failed".into());
        let status: Status = err.into();
        assert_eq!(status.code(), Code::Internal);
        assert!(status.message().contains("publish"));
    }

    #[test]
    fn protocol_error_to_internal() {
        let err = TransportError::ProtocolError("bad frame".into());
        let status: Status = err.into();
        assert_eq!(status.code(), Code::Internal);
        assert!(status.message().contains("bad frame"));
    }

    // -----------------------------------------------------------------------
    // tonic::Status -> TransportError
    // -----------------------------------------------------------------------

    #[test]
    fn deadline_exceeded_to_timeout() {
        let status = Status::deadline_exceeded("request timed out");
        let err: TransportError = status.into();
        assert!(matches!(err, TransportError::Timeout(_)));
        assert!(err.to_string().contains("timed out"));
    }

    #[test]
    fn unavailable_to_connection_failed() {
        let status = Status::unavailable("host unreachable");
        let err: TransportError = status.into();
        assert!(matches!(err, TransportError::ConnectionFailed(_)));
        assert!(err.to_string().contains("unreachable"));
    }

    #[test]
    fn invalid_argument_to_subject_invalid() {
        let status = Status::invalid_argument("bad subject");
        let err: TransportError = status.into();
        assert!(matches!(err, TransportError::SubjectInvalid(_)));
        assert!(err.to_string().contains("bad subject"));
    }

    #[test]
    fn resource_exhausted_to_payload_too_large() {
        let status = Status::resource_exhausted("exceeds limit");
        let err: TransportError = status.into();
        assert!(matches!(err, TransportError::PayloadTooLarge(_)));
        assert!(err.to_string().contains("exceeds limit"));
    }

    #[test]
    fn internal_to_protocol_error() {
        let status = Status::internal("internal failure");
        let err: TransportError = status.into();
        assert!(matches!(err, TransportError::ProtocolError(_)));
        assert!(err.to_string().contains("internal failure"));
    }

    #[test]
    fn unknown_code_to_protocol_error() {
        let status = Status::unknown("mystery error");
        let err: TransportError = status.into();
        assert!(matches!(err, TransportError::ProtocolError(_)));
        assert!(err.to_string().contains("mystery error"));
    }

    #[test]
    fn ok_status_to_protocol_error() {
        let status = Status::ok("all good");
        let err: TransportError = status.into();
        assert!(matches!(err, TransportError::ProtocolError(_)));
    }

    // -----------------------------------------------------------------------
    // Round-trip: TransportError -> Status -> TransportError
    // -----------------------------------------------------------------------

    #[test]
    fn roundtrip_timeout() {
        let original = TransportError::Timeout("timeout".into());
        let status: Status = original.into();
        let recovered: TransportError = status.into();
        assert!(matches!(recovered, TransportError::Timeout(_)));
    }

    #[test]
    fn roundtrip_connection_failed() {
        let original = TransportError::ConnectionFailed("fail".into());
        let status: Status = original.into();
        let recovered: TransportError = status.into();
        assert!(matches!(recovered, TransportError::ConnectionFailed(_)));
    }

    #[test]
    fn roundtrip_subject_invalid() {
        let original = TransportError::SubjectInvalid("invalid".into());
        let status: Status = original.into();
        let recovered: TransportError = status.into();
        assert!(matches!(recovered, TransportError::SubjectInvalid(_)));
    }

    #[test]
    fn roundtrip_payload_too_large() {
        let original = TransportError::PayloadTooLarge("too big".into());
        let status: Status = original.into();
        let recovered: TransportError = status.into();
        assert!(matches!(recovered, TransportError::PayloadTooLarge(_)));
    }

    // Internal-mapped variants round-trip to ProtocolError (lossy by design
    // because tonic::Code::Internal does not distinguish sub-variants).
    #[test]
    fn roundtrip_internal_variants_are_lossy() {
        let variants: Vec<TransportError> = vec![
            TransportError::SerializationError("ser".into()),
            TransportError::DeserializationError("de".into()),
            TransportError::SubscriptionError("sub".into()),
            TransportError::PublishError("pub".into()),
            TransportError::ProtocolError("proto".into()),
        ];
        for variant in variants {
            let status: Status = variant.into();
            assert_eq!(status.code(), Code::Internal);
            let recovered: TransportError = status.into();
            // All Internal codes become ProtocolError on the way back.
            assert!(matches!(recovered, TransportError::ProtocolError(_)));
        }
    }

    // -----------------------------------------------------------------------
    // NetworkError -> TransportError
    // -----------------------------------------------------------------------

    #[test]
    fn network_error_connection_failed() {
        let net_err = mister_smith_core::NetworkError::ConnectionFailed("down".into());
        let transport_err: TransportError = net_err.into();
        assert!(matches!(transport_err, TransportError::ConnectionFailed(_)));
    }

    #[test]
    fn network_error_timeout() {
        let net_err = mister_smith_core::NetworkError::Timeout("slow".into());
        let transport_err: TransportError = net_err.into();
        assert!(matches!(transport_err, TransportError::Timeout(_)));
    }

    #[test]
    fn network_error_protocol() {
        let net_err = mister_smith_core::NetworkError::ProtocolError("bad".into());
        let transport_err: TransportError = net_err.into();
        assert!(matches!(transport_err, TransportError::ProtocolError(_)));
    }
}
