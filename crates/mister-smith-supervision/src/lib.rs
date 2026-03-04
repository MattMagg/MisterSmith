#![deny(missing_docs, unsafe_code)]

//! Supervision trees for the Mister Smith multi-agent orchestration framework.
//!
//! Implements Erlang/OTP-style hierarchical fault tolerance:
//!
//! - **tree** — `SupervisionTree` for managing supervisor node hierarchy.
//! - **supervisor** — `SupervisorNode` and `ChildEntry` for tracking supervised children.
//! - **strategy** — Restart policy execution, scope filtering, budget checking, backoff.
//! - **escalation** — Failure escalation through the supervision hierarchy.
//! - **health** — `ActorSystemHealthCheck` implementing the `HealthCheck` trait.
//! - **events** — Lifecycle event emission to the EventBus.

pub mod escalation;
pub mod events;
pub mod health;
pub mod strategy;
pub mod supervised;
pub mod supervisor;
pub mod tree;

// Public re-exports for convenience
pub use escalation::escalate;
pub use strategy::{
    apply_restart_policy, check_restart_budget, compute_backoff, should_restart,
    SupervisionDecision, TerminationType,
};
pub use health::{ActorSystemHealthCheck, ActorSystemMetrics};
pub use supervised::SupervisedSystem;
pub use supervisor::{ChildEntry, SupervisionEvent, SupervisionEventType, SupervisorNode};
pub use tree::{SupervisionTree, TreeStatus};
