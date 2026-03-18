#![deny(missing_docs, unsafe_code)]

//! In-process pub/sub event system with typed events, filtering, dead letter
//! handling, and event store support.
//!
//! # Architecture
//!
//! The [`EventBus`] is the central distribution point. Events flow through:
//!
//! 1. **Event store** (optional) — persists the event for replay and audit.
//! 2. **Broadcast channel** — delivers to all broadcast subscribers.
//! 3. **Handler dispatch** — delivers to registered [`EventHandler`]
//!    implementations, applying [`EventFilter`] per handler.
//! 4. **Dead letter queue** — captures events where all matching handlers failed.
//!
//! The bus also implements `EventPublisher` from
//! the core crate, enabling components to publish events through the core trait
//! without depending on this crate directly.
//!
//! # Event Types
//!
//! Events use a rich type hierarchy ([`EventType`]) covering
//! system, agent, tool, and autonomy domains, with a custom escape hatch. The
//! [`Event`] struct carries full metadata including correlation
//! and causation IDs for distributed tracing.

pub mod autonomy;
pub mod builder;
pub mod bus;
pub mod dead_letter;
pub mod error;
pub mod handler;
pub mod store;
pub mod types;

// Re-exports for convenience.
pub use autonomy::{
    AutonomyEvent, AutonomyEventEnvelope, AutonomyEventType, AutonomyStatusView, BranchSummary,
    CapabilitySummary, CheckpointRecordSummary, ContextPressureSummary, DelegationAlert,
    ExecutionGraphSummary, ResumeProvenanceSummary, RoutingDecisionSummary, TopologyPlanSummary,
};
pub use builder::EventBuilder;
pub use bus::EventBus;
pub use dead_letter::DeadLetterQueue;
pub use error::EventBusError;
pub use handler::{EventFilter, EventHandler};
pub use store::{EventStore, InMemoryEventStore};
pub use types::{AgentEventType, Event, EventType, SystemEventType, ToolEventType};
