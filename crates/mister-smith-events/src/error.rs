//! Event error types.
//!
//! Defines [`EventBusError`] for the events crate's internal error handling.
//! The core crate provides [`mister_smith_core::EventError`] which is used
//! by the `EventPublisher` trait. This module provides a richer error type
//! with additional variants needed by the event bus internals, along with
//! conversions to/from the core error type.

use thiserror::Error;

/// Rich error type for the event bus subsystem.
///
/// Extends beyond the core [`mister_smith_core::EventError`] with additional
/// variants for timeout, validation, and correlation failures.
#[derive(Debug, Error)]
pub enum EventBusError {
    /// Event publication failed.
    #[error("Event publication failed: {0}")]
    PublishFailed(String),

    /// Operation timed out.
    #[error("Event operation timed out: {0}")]
    Timeout(String),

    /// Event subscription failed.
    #[error("Event subscription failed: {0}")]
    SubscriptionFailed(String),

    /// Event validation failed.
    #[error("Event validation failed: {0}")]
    ValidationFailed(String),

    /// Correlation chain lookup failed.
    #[error("Event correlation failed: {0}")]
    CorrelationFailed(String),

    /// Event store operation failed.
    #[error("Event store operation failed: {0}")]
    StoreFailed(String),

    /// Event serialization/deserialization failed.
    #[error("Event serialization failed: {0}")]
    SerializationFailed(String),

    /// Event handler failed.
    #[error("Event handler failed: {0}")]
    HandlerFailed(String),
}

impl From<EventBusError> for mister_smith_core::EventError {
    fn from(err: EventBusError) -> Self {
        match err {
            EventBusError::PublishFailed(msg) => {
                mister_smith_core::EventError::PublicationFailed(msg)
            }
            EventBusError::SubscriptionFailed(msg) => {
                mister_smith_core::EventError::SubscriptionFailed(msg)
            }
            EventBusError::StoreFailed(msg) => mister_smith_core::EventError::StoreFailed(msg),
            EventBusError::SerializationFailed(msg) => {
                mister_smith_core::EventError::SerializationFailed(msg)
            }
            EventBusError::HandlerFailed(msg) => {
                mister_smith_core::EventError::HandlerFailed(msg)
            }
            // Core EventError lacks these variants; map to closest equivalent.
            EventBusError::Timeout(msg) => {
                mister_smith_core::EventError::PublicationFailed(format!("timeout: {msg}"))
            }
            EventBusError::ValidationFailed(msg) => {
                mister_smith_core::EventError::HandlerFailed(format!("validation: {msg}"))
            }
            EventBusError::CorrelationFailed(msg) => {
                mister_smith_core::EventError::StoreFailed(format!("correlation: {msg}"))
            }
        }
    }
}

impl From<mister_smith_core::EventError> for EventBusError {
    fn from(err: mister_smith_core::EventError) -> Self {
        match err {
            mister_smith_core::EventError::HandlerFailed(msg) => EventBusError::HandlerFailed(msg),
            mister_smith_core::EventError::SerializationFailed(msg) => {
                EventBusError::SerializationFailed(msg)
            }
            mister_smith_core::EventError::PublicationFailed(msg) => {
                EventBusError::PublishFailed(msg)
            }
            mister_smith_core::EventError::SubscriptionFailed(msg) => {
                EventBusError::SubscriptionFailed(msg)
            }
            mister_smith_core::EventError::StoreFailed(msg) => EventBusError::StoreFailed(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_output() {
        let err = EventBusError::PublishFailed("channel closed".into());
        assert_eq!(err.to_string(), "Event publication failed: channel closed");
    }

    #[test]
    fn display_all_variants() {
        let cases: Vec<(EventBusError, &str)> = vec![
            (
                EventBusError::PublishFailed("x".into()),
                "Event publication failed: x",
            ),
            (
                EventBusError::Timeout("x".into()),
                "Event operation timed out: x",
            ),
            (
                EventBusError::SubscriptionFailed("x".into()),
                "Event subscription failed: x",
            ),
            (
                EventBusError::ValidationFailed("x".into()),
                "Event validation failed: x",
            ),
            (
                EventBusError::CorrelationFailed("x".into()),
                "Event correlation failed: x",
            ),
            (
                EventBusError::StoreFailed("x".into()),
                "Event store operation failed: x",
            ),
            (
                EventBusError::SerializationFailed("x".into()),
                "Event serialization failed: x",
            ),
            (
                EventBusError::HandlerFailed("x".into()),
                "Event handler failed: x",
            ),
        ];
        for (err, expected) in cases {
            assert_eq!(err.to_string(), expected);
        }
    }

    #[test]
    fn convert_to_core_event_error() {
        let bus_err = EventBusError::PublishFailed("test".into());
        let core_err: mister_smith_core::EventError = bus_err.into();
        assert!(core_err.to_string().contains("test"));
    }

    #[test]
    fn convert_from_core_event_error() {
        let core_err = mister_smith_core::EventError::HandlerFailed("core fail".into());
        let bus_err: EventBusError = core_err.into();
        match bus_err {
            EventBusError::HandlerFailed(msg) => assert_eq!(msg, "core fail"),
            _ => panic!("expected HandlerFailed"),
        }
    }

    #[test]
    fn timeout_maps_to_publication_failed_in_core() {
        let bus_err = EventBusError::Timeout("5s".into());
        let core_err: mister_smith_core::EventError = bus_err.into();
        match core_err {
            mister_smith_core::EventError::PublicationFailed(msg) => {
                assert!(msg.contains("timeout"));
            }
            _ => panic!("expected PublicationFailed"),
        }
    }
}
