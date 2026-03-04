# Quickstart: Phase 3 — Actor System & Supervision Trees

**Date**: 2026-03-04
**Feature**: [spec.md](spec.md)

## Scenario 1: Basic Actor Spawn and Communication

```rust
use mister_smith_actor::{ActorSystem, ActorSystemConfig, ActorRef, SpawnConfig};
use mister_smith_core::{Actor, AgentId, AgentState};
use async_trait::async_trait;

// Define a simple counter actor
struct CounterActor {
    id: AgentId,
}

#[derive(Debug)]
enum CounterMessage {
    Increment,
    GetCount { reply: oneshot::Sender<u64> },
}

#[async_trait]
impl Actor for CounterActor {
    type Message = CounterMessage;
    type State = u64; // the count
    type Error = Box<dyn std::error::Error + Send>;

    async fn handle_message(
        &mut self,
        message: CounterMessage,
        state: &mut u64,
    ) -> Result<(), Self::Error> {
        match message {
            CounterMessage::Increment => {
                *state += 1;
                Ok(())
            }
            CounterMessage::GetCount { reply } => {
                let _ = reply.send(*state);
                Ok(())
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

#[tokio::main]
async fn main() {
    let system = ActorSystem::new(ActorSystemConfig::default());

    // Spawn actor with default config (bounded mailbox, capacity 1000)
    let counter_ref: ActorRef<CounterMessage> = system
        .spawn(CounterActor { id: AgentId::new() }, 0u64, SpawnConfig::default())
        .await
        .expect("spawn failed");

    // Tell (fire-and-forget)
    counter_ref.tell(CounterMessage::Increment).unwrap();
    counter_ref.tell(CounterMessage::Increment).unwrap();

    // Ask (request-response)
    let (tx, rx) = oneshot::channel();
    counter_ref.tell(CounterMessage::GetCount { reply: tx }).unwrap();
    let count = rx.await.unwrap();
    assert_eq!(count, 2);

    system.shutdown().await.unwrap();
}
```

## Scenario 2: Supervised Actor with Restart

```rust
use mister_smith_actor::{ActorSystem, ActorSystemConfig, SpawnConfig};
use mister_smith_supervision::SupervisionTree;
use mister_smith_core::{
    RestartPolicy, RestartScope, SupervisionStrategy, EscalationPolicy, BackoffStrategy,
};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let system = ActorSystem::new(ActorSystemConfig::default());

    // Define supervision strategy
    let strategy = SupervisionStrategy {
        restart_policy: RestartPolicy::OneForOne,
        max_failures: 3,
        failure_window: Duration::from_secs(60),
        escalation_policy: EscalationPolicy::Escalate,
        backoff_strategy: BackoffStrategy::Exponential {
            initial: Duration::from_millis(100),
            max: Duration::from_secs(30),
            multiplier: 2.0,
        },
    };

    // Create a supervisor
    let supervisor_id = system
        .create_supervisor(strategy)
        .await
        .expect("supervisor creation failed");

    // Spawn supervised children
    let worker_a = system
        .spawn_supervised(
            WorkerActor::new(),
            WorkerState::default(),
            SpawnConfig { restart_scope: RestartScope::Permanent, ..Default::default() },
            supervisor_id,
        )
        .await
        .expect("spawn failed");

    let worker_b = system
        .spawn_supervised(
            WorkerActor::new(),
            WorkerState::default(),
            SpawnConfig { restart_scope: RestartScope::Transient, ..Default::default() },
            supervisor_id,
        )
        .await
        .expect("spawn failed");

    // If worker_a fails, only worker_a restarts (OneForOne)
    // If worker_b terminates normally, it is NOT restarted (Transient)
    // If worker_b fails with error, it IS restarted (Transient)

    system.shutdown().await.unwrap();
}
```

## Scenario 3: Hierarchical Supervision Tree

```rust
use mister_smith_supervision::SupervisionTree;
use mister_smith_core::{RestartPolicy, SupervisionStrategy};

#[tokio::main]
async fn main() {
    let system = ActorSystem::new(ActorSystemConfig::default());

    // Root supervisor — OneForAll policy
    let root = system
        .create_supervisor(SupervisionStrategy {
            restart_policy: RestartPolicy::OneForAll,
            max_failures: 5,
            ..Default::default()
        })
        .await?;

    // Mid-level supervisor under root — OneForOne policy
    let mid_supervisor = system
        .create_supervisor_under(
            SupervisionStrategy {
                restart_policy: RestartPolicy::OneForOne,
                max_failures: 3,
                ..Default::default()
            },
            root,
        )
        .await?;

    // Workers under mid-level supervisor
    for _ in 0..5 {
        system
            .spawn_supervised(WorkerActor::new(), WorkerState::default(), SpawnConfig::default(), mid_supervisor)
            .await?;
    }

    // Query tree status
    let status = system.tree_status().await;
    println!("Total nodes: {}", status.total_nodes);      // 7 (root + mid + 5 workers)
    println!("Tree depth: {}", status.tree_depth);          // 3
    println!("By state: {:?}", status.nodes_by_state);      // {Running: 7}

    // Graceful shutdown (reverse-start order)
    system.shutdown().await?;
    // Workers stop first, then mid-supervisor, then root
}
```

## Scenario 4: Event Integration

```rust
use mister_smith_events::{EventBus, EventHandler, EventFilter, EventType, AgentEventType, Event};
use mister_smith_actor::{ActorSystem, ActorSystemConfig};

struct LifecycleLogger;

#[async_trait]
impl EventHandler for LifecycleLogger {
    async fn handle_event(&self, event: Event) -> Result<(), EventBusError> {
        println!("[{}] {} from {}", event.timestamp, event.event_type, event.source);
        Ok(())
    }

    fn event_filter(&self) -> Option<EventFilter> {
        Some(EventFilter {
            event_types: Some(vec![
                EventType::Agent(AgentEventType::Created),
                EventType::Agent(AgentEventType::Started),
                EventType::Agent(AgentEventType::Failed),
                EventType::Agent(AgentEventType::Stopped),
            ]),
            sources: None,
            correlation_ids: None,
        })
    }
}

#[tokio::main]
async fn main() {
    let event_bus = Arc::new(EventBus::default());
    event_bus.subscribe(Arc::new(LifecycleLogger)).await;

    let system = ActorSystem::new(ActorSystemConfig::default())
        .with_event_publisher(event_bus.clone());

    // Spawn actor — emits Created + Started events
    let actor_ref = system.spawn(MyActor::new(), MyState::default(), SpawnConfig::default()).await?;

    // Shutdown — emits Stopped event for each actor
    system.shutdown().await?;
}
```

## Scenario 5: Health Monitoring

```rust
use mister_smith_monitoring::{HealthMonitor, HealthCheck};
use mister_smith_supervision::ActorSystemHealthCheck;

#[tokio::main]
async fn main() {
    let system = ActorSystem::new(ActorSystemConfig::default());
    // ... spawn actors ...

    let health_monitor = HealthMonitor::new(Duration::from_secs(10));
    let health_check = ActorSystemHealthCheck::new(system.tree_ref(), system.weak_ref());
    health_monitor.register_check(Arc::new(health_check)).await;

    // Health check reports:
    // - Status: Healthy/Degraded/Unhealthy
    // - Metadata: total_actors, actors_by_state, tree_depth, restart_count
}
```
