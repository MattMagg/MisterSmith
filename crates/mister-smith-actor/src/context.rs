//! Actor context providing identity and system access.
//!
//! `ActorContext` is available to actors during spawning and provides
//! the actor's own identity and a weak reference to the actor system.

use mister_smith_core::AgentId;

/// Context provided to an actor, containing its identity.
#[derive(Debug, Clone)]
pub struct ActorContext {
    /// This actor's unique identifier.
    pub actor_id: AgentId,
}

impl ActorContext {
    /// Create a new actor context.
    pub fn new(actor_id: AgentId) -> Self {
        Self { actor_id }
    }
}
