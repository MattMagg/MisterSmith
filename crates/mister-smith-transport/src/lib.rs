//! Protocol-agnostic transport abstraction for the Mister Smith framework.
//!
//! Provides the `Transport` trait, `MessageEnvelope`, typed message structs,
//! subject taxonomy, and serialization helpers shared by all transport implementations.

pub mod availability;
pub mod envelope;
pub mod errors;
pub mod inmemory;
pub mod messages;
pub mod priority;
pub mod serialization;
pub mod subject;
pub mod transport;

// Re-export key types at crate root.
pub use availability::{is_valid_transition, AgentAvailability};
pub use envelope::{MessageEnvelope, MessageEnvelopeBuilder, DEFAULT_MAX_PAYLOAD_SIZE, SCHEMA_VERSION};
pub use errors::TransportError;
pub use inmemory::InMemoryTransport;
pub use messages::{
    AgentHeartbeat, AgentSpawn, AgentTerminate, ConfigUpdate, Severity, StepComplete,
    SystemEvent, TaskAssignment, TaskResult, TaskStatus, WorkflowResult, WorkflowStart,
    WorkflowStatus,
};
pub use priority::MessagePriority;
pub use serialization::{from_json, from_msgpack, to_json, to_msgpack};
pub use subject::SubjectTaxonomy;
pub use transport::{ReceivedMessage, Subscription, Transport};

/// Generated protobuf types from `common.proto`, `agent_service.proto`, and `system_service.proto`.
pub mod proto {
    /// Types from the `mister_smith.v1` package.
    pub mod mister_smith {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/mister_smith.v1.rs"));
        }
    }
}
