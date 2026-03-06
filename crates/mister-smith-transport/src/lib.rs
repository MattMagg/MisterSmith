//! Protocol-agnostic transport abstraction for the Mister Smith framework.
//!
//! Provides the `Transport` trait, `MessageEnvelope`, typed message structs,
//! subject taxonomy, and serialization helpers shared by all transport implementations.

pub mod availability;
pub mod durable;
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
pub use durable::{DurableMessage, DurableSubscription, DurableTransport, MessageAcker};
pub use envelope::{
    extract_trace_context, extract_tracestate, inject_trace_context, MessageEnvelope,
    MessageEnvelopeBuilder, DEFAULT_MAX_PAYLOAD_SIZE, SCHEMA_VERSION, TRACEPARENT_HEADER,
    TRACESTATE_HEADER,
};
pub use errors::TransportError;
pub use inmemory::InMemoryTransport;
pub use messages::{
    AgentHeartbeat, AgentSpawn, AgentTerminate, ConfigUpdate, Severity, StepComplete, SystemEvent,
    TaskAssignment, TaskResult, TaskStatus, WorkflowResult, WorkflowStart, WorkflowStatus,
};
pub use priority::MessagePriority;
pub use serialization::{from_json, from_msgpack, to_json, to_msgpack};
pub use subject::SubjectTaxonomy;
pub use transport::{ReceivedMessage, Subscription, Transport};

/// Generated protobuf types from `common.proto`, `agent_service.proto`, `system_service.proto`, and `health_service.proto`.
pub mod proto {
    /// Types from the `mister_smith.v1` package (`common`, `agent_service`, and `system_service`).
    pub mod mister_smith {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/mister_smith.v1.rs"));
        }
    }

    /// Types from the `grpc.health.v1` package (`health_service`).
    pub mod grpc {
        pub mod health {
            pub mod v1 {
                include!(concat!(env!("OUT_DIR"), "/grpc.health.v1.rs"));
            }
        }
    }
}

#[cfg(test)]
mod proto_tests {
    use super::proto;

    #[test]
    fn health_proto_types_are_accessible() {
        let request = proto::grpc::health::v1::HealthCheckRequest {
            service: "transport".to_string(),
        };

        let response = proto::grpc::health::v1::HealthCheckResponse {
            status: proto::grpc::health::v1::health_check_response::ServingStatus::Serving.into(),
        };

        assert_eq!(request.service, "transport");
        assert_eq!(
            response.status,
            proto::grpc::health::v1::health_check_response::ServingStatus::Serving as i32
        );
    }
}
