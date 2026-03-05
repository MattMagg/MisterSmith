use std::sync::Arc;
use std::time::Duration;

use mister_smith_actor::system::{ActorSystem, ActorSystemConfig};
use mister_smith_agents::config::{AgentConfig, HealthLevel};
use mister_smith_agents::agent::{deregister_agent, register_agent, spawn_agent};
use mister_smith_agents::registry::AgentRegistry;
use mister_smith_core::{Actor, AgentId, AgentState, AgentType};
use serde::{Deserialize, Serialize};

// Test actor for lifecycle tests
#[derive(Debug, Clone, Serialize, Deserialize)]
enum LifecycleMessage {
    Ping,
    GetCounter,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct LifecycleState {
    counter: u32,
}

#[derive(Debug, thiserror::Error)]
#[error("lifecycle error: {0}")]
struct LifecycleError(String);

struct LifecycleActor {
    id: AgentId,
}

#[async_trait::async_trait]
impl Actor for LifecycleActor {
    type Message = LifecycleMessage;
    type State = LifecycleState;
    type Error = LifecycleError;
    type Response = serde_json::Value;

    async fn handle_message(
        &mut self,
        msg: Self::Message,
        state: &mut Self::State,
    ) -> Result<Self::Response, Self::Error> {
        match msg {
            LifecycleMessage::Ping => {
                state.counter += 1;
                Ok(serde_json::json!({"pong": state.counter}))
            }
            LifecycleMessage::GetCounter => {
                Ok(serde_json::json!({"counter": state.counter}))
            }
        }
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
async fn test_spawn_running() {
    let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
    let id = AgentId::new();
    let runtime = spawn_agent(
        system,
        LifecycleActor { id },
        LifecycleState::default(),
        AgentConfig::for_type(AgentType::Worker),
    )
    .await
    .unwrap();

    assert_eq!(runtime.agent_id(), id);
    assert!(runtime.is_alive());

    let state = runtime.state().await;
    assert!(
        state == AgentState::Running || state == AgentState::Initializing,
        "unexpected: {:?}",
        state
    );
}

#[tokio::test]
async fn test_stop_terminates() {
    let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
    let id = AgentId::new();
    let runtime = spawn_agent(
        system.clone(),
        LifecycleActor { id },
        LifecycleState::default(),
        AgentConfig::default(),
    )
    .await
    .unwrap();

    assert!(runtime.is_alive());
    runtime.stop().await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(!runtime.is_alive());
}

#[tokio::test]
async fn test_message_processing() {
    let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
    let id = AgentId::new();
    let runtime = spawn_agent(
        system,
        LifecycleActor { id },
        LifecycleState::default(),
        AgentConfig::default(),
    )
    .await
    .unwrap();

    // Ping twice
    let r1 = runtime
        .ask(LifecycleMessage::Ping, Duration::from_secs(5))
        .await
        .unwrap();
    assert_eq!(r1["pong"], 1);

    let r2 = runtime
        .ask(LifecycleMessage::Ping, Duration::from_secs(5))
        .await
        .unwrap();
    assert_eq!(r2["pong"], 2);

    // Verify counter
    let counter = runtime
        .ask(LifecycleMessage::GetCounter, Duration::from_secs(5))
        .await
        .unwrap();
    assert_eq!(counter["counter"], 2);
}

#[tokio::test]
async fn test_auto_registration() {
    let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
    let registry = AgentRegistry::new();
    let id = AgentId::new();
    let runtime = spawn_agent(
        system,
        LifecycleActor { id },
        LifecycleState::default(),
        AgentConfig::for_type(AgentType::Worker),
    )
    .await
    .unwrap();

    // Register
    register_agent(&runtime, &registry, vec!["analysis".to_string()]).await;
    assert_eq!(registry.count(), 1);

    let entry = registry.find_by_id(&id).unwrap();
    assert_eq!(entry.agent_type, AgentType::Worker);
    assert!(entry.capabilities.contains(&"analysis".to_string()));

    // Find by type
    let workers = registry.find_by_type(AgentType::Worker);
    assert_eq!(workers.len(), 1);

    // Find available
    let available = registry.find_available(AgentType::Worker, &["analysis".to_string()]);
    assert_eq!(available.len(), 1);

    // Deregister
    deregister_agent(&runtime, &registry);
    assert_eq!(registry.count(), 0);
}

#[tokio::test]
async fn test_health_tracking() {
    let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
    let registry = AgentRegistry::new();
    let id = AgentId::new();
    let runtime = spawn_agent(
        system,
        LifecycleActor { id },
        LifecycleState::default(),
        AgentConfig::default(),
    )
    .await
    .unwrap();

    register_agent(&runtime, &registry, vec![]).await;

    // Initially healthy
    let entry = registry.find_by_id(&id).unwrap();
    assert_eq!(entry.health, HealthLevel::Healthy);

    // Degrade health
    registry.update_health(&id, HealthLevel::Degraded);
    let entry = registry.find_by_id(&id).unwrap();
    assert_eq!(entry.health, HealthLevel::Degraded);

    // Unhealthy agents not returned by find_available
    registry.update_health(&id, HealthLevel::Unhealthy);
    let available = registry.find_available(AgentType::Worker, &[]);
    assert!(available.is_empty());
}

#[tokio::test]
async fn test_restart_count_tracking() {
    let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
    let id = AgentId::new();
    let runtime = spawn_agent(
        system,
        LifecycleActor { id },
        LifecycleState::default(),
        AgentConfig::default(),
    )
    .await
    .unwrap();

    assert_eq!(runtime.restart_count(), 0);

    // Simulate incrementing restart count
    runtime
        .context
        .restart_count
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    assert_eq!(runtime.restart_count(), 1);
}

#[tokio::test]
async fn test_multiple_agents() {
    let system = Arc::new(ActorSystem::new(ActorSystemConfig::default()));
    let registry = AgentRegistry::new();

    // Spawn 3 workers
    for _ in 0..3 {
        let id = AgentId::new();
        let runtime = spawn_agent(
            system.clone(),
            LifecycleActor { id },
            LifecycleState::default(),
            AgentConfig::for_type(AgentType::Worker),
        )
        .await
        .unwrap();
        register_agent(&runtime, &registry, vec!["compute".to_string()]).await;
    }

    // Spawn 1 coordinator
    let coord_id = AgentId::new();
    let coord = spawn_agent(
        system.clone(),
        LifecycleActor { id: coord_id },
        LifecycleState::default(),
        AgentConfig::for_type(AgentType::Coordinator),
    )
    .await
    .unwrap();
    register_agent(&coord, &registry, vec![]).await;

    assert_eq!(registry.count(), 4);
    assert_eq!(registry.find_by_type(AgentType::Worker).len(), 3);
    assert_eq!(registry.find_by_type(AgentType::Coordinator).len(), 1);
}
