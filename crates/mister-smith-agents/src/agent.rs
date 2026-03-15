use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use mister_smith_actor::mailbox::{MailboxConfig, SpawnConfig};
use mister_smith_actor::system::ActorSystem;
use mister_smith_actor::ActorRef;
use mister_smith_core::{Actor, ActorError, AgentId, AgentState, AgentType, RestartScope};
use mister_smith_core::{DelegationCapability, ProvenanceChain};
use mister_smith_security::jwt::AgentClaims;
use mister_smith_supervision::SupervisedSystem;
use tokio::sync::RwLock;
use tracing::instrument;

use crate::config::{AgentConfig, HealthLevel};
use crate::errors::AgentSystemError;
use crate::registry::{AgentEntry, AgentRegistry};

/// Shared context accessible by the AgentRuntime and external consumers.
#[derive(Debug)]
pub struct AgentContext {
    pub agent_id: AgentId,
    pub agent_type: AgentType,
    pub config: AgentConfig,
    pub delegation_chain: Vec<String>,
    pub delegation_capability: Option<DelegationCapability>,
    pub provenance_chain: Option<ProvenanceChain>,
    pub started_at: RwLock<Option<DateTime<Utc>>>,
    pub restart_count: AtomicU32,
    pub health: RwLock<HealthLevel>,
}

impl AgentContext {
    pub fn new(agent_id: AgentId, config: AgentConfig) -> Self {
        Self::new_with_delegation_chain(agent_id, config, Vec::new())
    }

    pub fn new_with_delegation_chain(
        agent_id: AgentId,
        config: AgentConfig,
        delegation_chain: Vec<String>,
    ) -> Self {
        Self::new_with_delegation(agent_id, config, delegation_chain, None, None)
    }

    pub fn new_with_delegation(
        agent_id: AgentId,
        config: AgentConfig,
        delegation_chain: Vec<String>,
        delegation_capability: Option<DelegationCapability>,
        provenance_chain: Option<ProvenanceChain>,
    ) -> Self {
        Self {
            agent_type: config.agent_type,
            agent_id,
            config,
            delegation_chain,
            delegation_capability,
            provenance_chain,
            started_at: RwLock::new(None),
            restart_count: AtomicU32::new(0),
            health: RwLock::new(HealthLevel::Healthy),
        }
    }
}

#[derive(Debug, Default)]
struct DelegationRuntimeContext {
    delegation_chain: Vec<String>,
    delegation_capability: Option<DelegationCapability>,
    provenance_chain: Option<ProvenanceChain>,
}

/// Core agent runtime — bridges Actor (Phase 3) with Agent orchestration.
///
/// Holds an ActorRef for message passing and shared AgentContext for
/// lifecycle state, health, and configuration. Uses the ActorSystem for
/// state queries and stop signals.
pub struct AgentRuntime<M: Send + 'static, R: Send + 'static> {
    pub context: Arc<AgentContext>,
    pub actor_ref: ActorRef<M, R>,
    system: Arc<ActorSystem>,
}

impl<M: Send + Clone + 'static, R: Send + Clone + 'static> AgentRuntime<M, R> {
    /// Get the agent's unique identifier.
    pub fn agent_id(&self) -> AgentId {
        self.context.agent_id
    }

    /// Get the agent's type.
    pub fn agent_type(&self) -> AgentType {
        self.context.agent_type
    }

    /// Get the agent's current lifecycle state from the actor system.
    #[instrument(skip(self), fields(agent.id = %self.context.agent_id, agent.type = ?self.context.agent_type))]
    pub async fn state(&self) -> AgentState {
        self.system
            .get_actor_state(&self.context.agent_id)
            .await
            .unwrap_or(AgentState::Terminated)
    }

    /// Get a reference to the underlying ActorRef.
    pub fn actor_ref(&self) -> &ActorRef<M, R> {
        &self.actor_ref
    }

    /// Get a clone of the underlying ActorRef.
    pub fn actor_ref_clone(&self) -> ActorRef<M, R> {
        self.actor_ref.clone()
    }

    /// Send a message without waiting for a response.
    pub fn tell(&self, message: M) -> Result<(), ActorError> {
        self.actor_ref.tell(message)
    }

    /// Send a message and await a typed response.
    #[instrument(skip(self, message), fields(agent.id = %self.context.agent_id, agent.type = ?self.context.agent_type))]
    pub async fn ask(&self, message: M, timeout: Duration) -> Result<R, ActorError> {
        self.actor_ref.ask(message, timeout).await
    }

    /// Request graceful stop of the agent.
    #[instrument(skip(self), fields(agent.id = %self.context.agent_id, agent.type = ?self.context.agent_type))]
    pub async fn stop(&self) -> Result<(), AgentSystemError> {
        let stopped = self.system.stop_actor(&self.context.agent_id).await;
        if stopped {
            Ok(())
        } else {
            Err(AgentSystemError::AgentNotFound(
                self.context.agent_id.to_string(),
            ))
        }
    }

    /// Check if the underlying actor is still alive.
    pub fn is_alive(&self) -> bool {
        self.actor_ref.is_alive()
    }

    /// Get the current restart count.
    pub fn restart_count(&self) -> u32 {
        self.context.restart_count.load(Ordering::SeqCst)
    }

    /// Get the current health level.
    pub async fn health(&self) -> HealthLevel {
        *self.context.health.read().await
    }

    /// Get a reference to the shared context.
    pub fn context(&self) -> &Arc<AgentContext> {
        &self.context
    }

    /// Get the ordered chain of delegating parent agents for this runtime.
    pub fn delegation_chain(&self) -> &[String] {
        &self.context.delegation_chain
    }

    pub fn delegation_capability(&self) -> Option<&DelegationCapability> {
        self.context.delegation_capability.as_ref()
    }

    pub fn provenance_chain(&self) -> Option<&ProvenanceChain> {
        self.context.provenance_chain.as_ref()
    }

    /// Get a reference to the actor system.
    pub fn system(&self) -> &Arc<ActorSystem> {
        &self.system
    }
}

/// Spawn an actor within an ActorSystem and wrap it as an AgentRuntime.
pub async fn spawn_agent<A>(
    system: Arc<ActorSystem>,
    actor: A,
    initial_state: A::State,
    config: AgentConfig,
) -> Result<AgentRuntime<A::Message, A::Response>, AgentSystemError>
where
    A: Actor + 'static,
    A::Message: Send + 'static,
    A::State: Send + 'static,
{
    spawn_agent_with_chain(
        system,
        actor,
        initial_state,
        config,
        DelegationRuntimeContext::default(),
    )
    .await
}

/// Spawn a child agent and propagate the parent's delegation chain into the runtime context.
pub async fn spawn_agent_delegated<A>(
    system: Arc<ActorSystem>,
    parent_claims: &AgentClaims,
    actor: A,
    initial_state: A::State,
    config: AgentConfig,
) -> Result<AgentRuntime<A::Message, A::Response>, AgentSystemError>
where
    A: Actor + 'static,
    A::Message: Send + 'static,
    A::State: Send + 'static,
{
    let child_claims = parent_claims.delegated_to(
        actor.actor_id().to_string(),
        format!("{:?}", config.agent_type).to_lowercase(),
    );
    spawn_agent_with_chain(
        system,
        actor,
        initial_state,
        config,
        DelegationRuntimeContext {
            delegation_chain: child_claims.delegation_chain,
            delegation_capability: child_claims.delegation_capability,
            provenance_chain: child_claims.provenance_chain,
        },
    )
    .await
}

async fn spawn_agent_with_chain<A>(
    system: Arc<ActorSystem>,
    actor: A,
    initial_state: A::State,
    config: AgentConfig,
    delegation_context: DelegationRuntimeContext,
) -> Result<AgentRuntime<A::Message, A::Response>, AgentSystemError>
where
    A: Actor + 'static,
    A::Message: Send + 'static,
    A::State: Send + 'static,
{
    let agent_id = actor.actor_id();
    let spawn_config = SpawnConfig {
        mailbox: MailboxConfig {
            capacity: Some(config.mailbox_capacity),
        },
        restart_scope: RestartScope::Transient,
        shutdown_timeout: Duration::from_secs(5),
    };

    let context = Arc::new(AgentContext::new_with_delegation(
        agent_id,
        config,
        delegation_context.delegation_chain,
        delegation_context.delegation_capability,
        delegation_context.provenance_chain,
    ));

    let actor_ref = system
        .spawn(actor, initial_state, spawn_config)
        .await
        .map_err(|e| AgentSystemError::SpawnFailed(e.to_string()))?;

    *context.started_at.write().await = Some(Utc::now());

    Ok(AgentRuntime {
        context,
        actor_ref,
        system,
    })
}

/// Spawn an actor under a supervisor within a SupervisedSystem and wrap it as an AgentRuntime.
pub async fn spawn_supervised<A, F>(
    supervised: &SupervisedSystem,
    supervisor_id: AgentId,
    factory: F,
    config: AgentConfig,
) -> Result<AgentRuntime<A::Message, A::Response>, AgentSystemError>
where
    A: Actor + 'static,
    A::Message: Send + Clone + 'static,
    A::State: Send + 'static,
    A::Response: Send + Clone + 'static,
    F: Fn() -> (A, A::State) + Send + Sync + 'static,
{
    let (sample_actor, _) = factory();
    let agent_id = sample_actor.actor_id();
    drop(sample_actor);

    spawn_supervised_with_chain(
        supervised,
        supervisor_id,
        agent_id,
        factory,
        config,
        DelegationRuntimeContext::default(),
    )
    .await
}

/// Spawn a supervised child agent with a propagated delegation chain in its runtime context.
pub async fn spawn_supervised_delegated<A, F>(
    supervised: &SupervisedSystem,
    supervisor_id: AgentId,
    parent_claims: &AgentClaims,
    factory: F,
    config: AgentConfig,
) -> Result<AgentRuntime<A::Message, A::Response>, AgentSystemError>
where
    A: Actor + 'static,
    A::Message: Send + Clone + 'static,
    A::State: Send + 'static,
    A::Response: Send + Clone + 'static,
    F: Fn() -> (A, A::State) + Send + Sync + 'static,
{
    let (sample_actor, _) = factory();
    let agent_id = sample_actor.actor_id();
    let child_claims = parent_claims.delegated_to(
        agent_id.to_string(),
        format!("{:?}", config.agent_type).to_lowercase(),
    );
    drop(sample_actor);

    spawn_supervised_with_chain(
        supervised,
        supervisor_id,
        agent_id,
        factory,
        config,
        DelegationRuntimeContext {
            delegation_chain: child_claims.delegation_chain,
            delegation_capability: child_claims.delegation_capability,
            provenance_chain: child_claims.provenance_chain,
        },
    )
    .await
}

async fn spawn_supervised_with_chain<A, F>(
    supervised: &SupervisedSystem,
    supervisor_id: AgentId,
    agent_id: AgentId,
    factory: F,
    config: AgentConfig,
    delegation_context: DelegationRuntimeContext,
) -> Result<AgentRuntime<A::Message, A::Response>, AgentSystemError>
where
    A: Actor + 'static,
    A::Message: Send + Clone + 'static,
    A::State: Send + 'static,
    A::Response: Send + Clone + 'static,
    F: Fn() -> (A, A::State) + Send + Sync + 'static,
{
    let spawn_config = SpawnConfig {
        mailbox: MailboxConfig {
            capacity: Some(config.mailbox_capacity),
        },
        restart_scope: RestartScope::Transient,
        shutdown_timeout: Duration::from_secs(5),
    };

    let context = Arc::new(AgentContext::new_with_delegation(
        agent_id,
        config,
        delegation_context.delegation_chain,
        delegation_context.delegation_capability,
        delegation_context.provenance_chain,
    ));

    let actor_ref = supervised
        .spawn_supervised(supervisor_id, factory, spawn_config)
        .await
        .map_err(|e| AgentSystemError::SpawnFailed(e.to_string()))?;

    *context.started_at.write().await = Some(Utc::now());

    Ok(AgentRuntime {
        context,
        actor_ref,
        system: supervised.system_arc(),
    })
}

/// Register an AgentRuntime with an AgentRegistry, creating an AgentEntry.
pub async fn register_agent<M: Send + Clone + 'static, R: Send + Clone + 'static>(
    runtime: &AgentRuntime<M, R>,
    registry: &AgentRegistry,
    capabilities: Vec<String>,
) {
    let entry = AgentEntry {
        agent_id: runtime.agent_id(),
        agent_type: runtime.agent_type(),
        state: runtime.state().await,
        health: runtime.health().await,
        capabilities,
        command_subject: format!(
            "agents.{}.commands.{}",
            runtime.agent_id(),
            format!("{:?}", runtime.agent_type()).to_lowercase()
        ),
        heartbeat_at: Utc::now(),
        started_at: *runtime.context.started_at.read().await,
        restart_count: runtime.restart_count(),
        metadata: if runtime.delegation_chain().is_empty() {
            if runtime.delegation_capability().is_none() {
                serde_json::Value::Null
            } else {
                serde_json::json!({
                    "delegation_capability": runtime.delegation_capability(),
                    "provenance_chain": runtime.provenance_chain(),
                })
            }
        } else {
            serde_json::json!({
                "delegation_chain": runtime.delegation_chain(),
                "delegation_capability": runtime.delegation_capability(),
                "provenance_chain": runtime.provenance_chain(),
            })
        },
        supervisor_id: None,
    };
    registry.register(entry);
}

/// Deregister an agent from the registry.
pub fn deregister_agent<M: Send + Clone + 'static, R: Send + Clone + 'static>(
    runtime: &AgentRuntime<M, R>,
    registry: &AgentRegistry,
) {
    registry.deregister(&runtime.agent_id());
}

#[cfg(test)]
mod tests {
    use super::*;
    use mister_smith_actor::system::ActorSystemConfig;
    use mister_smith_core::Actor;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestMessage(String);

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestState {
        counter: u32,
    }

    #[derive(Debug, thiserror::Error)]
    #[error("test error: {0}")]
    struct TestError(String);

    struct TestActor {
        id: AgentId,
    }

    #[async_trait::async_trait]
    impl Actor for TestActor {
        type Message = TestMessage;
        type State = TestState;
        type Error = TestError;
        type Response = String;

        async fn handle_message(
            &mut self,
            msg: Self::Message,
            state: &mut Self::State,
        ) -> Result<Self::Response, Self::Error> {
            state.counter += 1;
            let handled = &msg.0;
            let count = state.counter;
            Ok(format!("handled: {handled} (count: {count})"))
        }

        fn pre_start(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn post_stop(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn actor_id(&self) -> AgentId {
            self.id
        }
    }

    #[tokio::test]
    async fn test_spawn_agent() {
        let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
        let agent_id = AgentId::new();
        let actor = TestActor { id: agent_id };
        let state = TestState { counter: 0 };
        let config = AgentConfig::default();

        let runtime = spawn_agent(system, actor, state, config).await.unwrap();

        assert_eq!(runtime.agent_id(), agent_id);
        assert!(runtime.is_alive());
        assert_eq!(runtime.restart_count(), 0);
        assert!(runtime.context.started_at.read().await.is_some());
    }

    #[tokio::test]
    async fn test_agent_ask() {
        let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
        let agent_id = AgentId::new();
        let actor = TestActor { id: agent_id };
        let state = TestState { counter: 0 };
        let config = AgentConfig::default();

        let runtime = spawn_agent(system, actor, state, config).await.unwrap();

        let response = runtime
            .ask(TestMessage("hello".into()), Duration::from_secs(5))
            .await
            .unwrap();
        assert_eq!(response, "handled: hello (count: 1)");
    }

    #[tokio::test]
    async fn test_agent_tell_and_ask() {
        let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
        let agent_id = AgentId::new();
        let actor = TestActor { id: agent_id };
        let state = TestState { counter: 0 };
        let config = AgentConfig::default();

        let runtime = spawn_agent(system, actor, state, config).await.unwrap();

        // Tell doesn't return a response but increments state
        runtime.tell(TestMessage("fire".into())).unwrap();
        // Small delay for message processing
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Ask should see counter at 2
        let response = runtime
            .ask(TestMessage("verify".into()), Duration::from_secs(5))
            .await
            .unwrap();
        assert_eq!(response, "handled: verify (count: 2)");
    }

    #[tokio::test]
    async fn test_agent_stop() {
        let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
        let agent_id = AgentId::new();
        let actor = TestActor { id: agent_id };
        let state = TestState { counter: 0 };
        let config = AgentConfig::default();

        let runtime = spawn_agent(system.clone(), actor, state, config)
            .await
            .unwrap();

        assert!(runtime.is_alive());
        runtime.stop().await.unwrap();

        // Give it a moment to shut down
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(!runtime.is_alive());
    }

    #[tokio::test]
    async fn test_agent_state_query() {
        let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
        let agent_id = AgentId::new();
        let actor = TestActor { id: agent_id };
        let state = TestState { counter: 0 };
        let config = AgentConfig::default();

        let runtime = spawn_agent(system.clone(), actor, state, config)
            .await
            .unwrap();

        // After spawn, state should be Running (the actor cell transitions to Running on successful pre_start)
        let current_state = runtime.state().await;
        assert!(
            current_state == AgentState::Running || current_state == AgentState::Initializing,
            "unexpected state: {:?}",
            current_state
        );
    }
}
