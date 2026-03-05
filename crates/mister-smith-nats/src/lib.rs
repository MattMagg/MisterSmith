//! NATS transport implementation for inter-agent communication.
//!
//! Provides pub/sub, request-reply, queue groups, and JetStream durable messaging
//! over async-nats 0.46.

pub mod client;
pub mod config;
pub mod errors;
pub mod health;
pub mod jetstream;
pub mod subjects;

pub use client::NatsTransport;
pub use config::{JetStreamConfig, NatsTransportConfig};
pub use errors::NatsError;
pub use health::NatsHealthCheck;
pub use jetstream::JetStreamManager;
pub use subjects::{to_nats_subject, validate_nats_subject, wildcard_all, wildcard_single};
