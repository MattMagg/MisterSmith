#![deny(missing_docs, unsafe_code)]

//! Actor model primitives for the Mister Smith multi-agent orchestration framework.
//!
//! This crate provides the core actor system:
//!
//! - **mailbox** — Bounded/unbounded FIFO message queues via Tokio channels.
//! - **actor_ref** — Typed actor handles for tell (fire-and-forget) and ask (request-response).
//! - **actor_cell** — Internal runtime wrapper managing actor lifecycle and message processing.
//! - **system** — `ActorSystem` for spawning, registering, and shutting down actors.
//! - **context** — `ActorContext` providing actor identity and system access.
//! - **errors** — Re-exports of `ActorError` from `mister-smith-core`.

pub mod actor_cell;
pub mod actor_ref;
pub mod context;
pub mod errors;
pub mod mailbox;
pub mod system;

// Public re-exports for convenience
pub use actor_ref::ActorRef;
pub use context::ActorContext;
pub use errors::{ActorError, SupervisionError};
pub use mailbox::{Envelope, MailboxConfig, MailboxSender, SpawnConfig};
pub use system::{ActorSystem, ActorSystemConfig};
