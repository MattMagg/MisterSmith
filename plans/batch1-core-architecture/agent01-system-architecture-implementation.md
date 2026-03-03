# System Architecture Implementation Plan
**Agent-01 | MS Framework Implementation Planning | Batch 1 - Core Architecture**  
**Date**: 2025-07-05  
**Status**: IMPLEMENTATION PLAN READY

## Executive Summary

This implementation plan transforms the MS Framework system architecture from its current 82% readiness to production-ready status. It addresses critical gaps identified in the validation bridge, particularly the supervision tree pseudocode conversion and integration requirements.

**Key Focus Areas**:
- Converting supervision tree pseudocode to production Rust code
- Implementing agent orchestration communication (47% → 95%)
- Completing system integration patterns
- Establishing proper async runtime coordination

**Implementation Timeline**: 10-12 weeks  
**Resource Requirements**: 3-4 senior Rust developers  
**Priority**: CRITICAL

## Prerequisites and Dependencies

### Rust Toolchain Requirements

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.75"
components = ["rustfmt", "clippy", "rust-analyzer"]
profile = "default"
```

### Required Crates and Versions

```toml
[workspace]
members = [
    "mister-smith-core",
    "mister-smith-supervision",
    "mister-smith-actors",
    "mister-smith-events",
    "mister-smith-resources"
]

[workspace.dependencies]
# Core async runtime
tokio = { version = "1.45.1", features = ["full"] }
tokio-util = { version = "0.7", features = ["codec", "io"] }
futures = "0.3"
async-trait = "0.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Concurrency primitives
dashmap = "6.0"
crossbeam = "0.8"
parking_lot = "0.12"

# Utilities
uuid = { version = "1.0", features = ["v4", "serde"] }
num_cpus = "1.16"
bytes = "1.8"

# Logging and monitoring
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
metrics = "0.23"
metrics-exporter-prometheus = "0.15"

# Testing
tokio-test = "0.4"
proptest = "1.0"
criterion = { version = "0.5", features = ["async_tokio"] }
```

### Development Environment Setup

```bash
#!/bin/bash
# setup-dev-env.sh

# Install Rust if not present
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
fi

# Update to required version
rustup update stable
rustup default 1.75

# Install required components
rustup component add rustfmt clippy rust-analyzer

# Install development tools
cargo install cargo-watch cargo-nextest cargo-audit cargo-outdated

# Setup pre-commit hooks
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
EOF
chmod +x .git/hooks/pre-commit
```

## System Architecture Implementation Steps

### Phase 1: Core System Initialization (Weeks 1-2)

#### Step 1.1: Project Structure Setup

```bash
# Create workspace structure
mkdir -p mister-smith-framework/{
    mister-smith-core/src/{errors,core,config},
    mister-smith-supervision/src/{tree,detector,recovery},
    mister-smith-actors/src/{core,mailbox,routing},
    mister-smith-events/src/{bus,handlers,store},
    mister-smith-resources/src/{pool,health,metrics}
}
```

#### Step 1.2: Core Error Types Implementation

```rust
// mister-smith-core/src/errors.rs
use thiserror::Error;
use std::sync::Arc;

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("Runtime error: {0}")]
    Runtime(#[from] RuntimeError),
    #[error("Supervision error: {0}")]
    Supervision(#[from] SupervisionError),
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigError),
    #[error("Actor system error: {0}")]
    Actor(#[from] ActorError),
    #[error("Resource error: {0}")]
    Resource(#[from] ResourceError),
}

// Implement error context enrichment
pub trait ErrorContext {
    fn with_context<C>(self, context: C) -> Self
    where
        C: std::fmt::Display + Send + Sync + 'static;
}

impl<T> ErrorContext for Result<T, SystemError> {
    fn with_context<C>(self, context: C) -> Self
    where
        C: std::fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            tracing::error!(error = ?e, context = %context, "Operation failed");
            e
        })
    }
}
```

#### Step 1.3: Runtime Manager Implementation

```rust
// mister-smith-core/src/core/runtime.rs
use std::sync::Arc;
use tokio::runtime::{Builder, Runtime};
use crate::errors::{RuntimeError, SystemError};

pub struct RuntimeManager {
    runtime: Arc<Runtime>,
    config: RuntimeConfig,
    shutdown_token: tokio_util::sync::CancellationToken,
    components: Arc<dashmap::DashMap<String, Box<dyn SystemComponent>>>,
}

#[async_trait::async_trait]
pub trait SystemComponent: Send + Sync {
    async fn start(&self) -> Result<(), SystemError>;
    async fn stop(&self) -> Result<(), SystemError>;
    fn health_check(&self) -> ComponentHealth;
    fn name(&self) -> &str;
}

impl RuntimeManager {
    pub fn new(config: RuntimeConfig) -> Result<Self, RuntimeError> {
        let mut builder = Builder::new_multi_thread();
        
        if let Some(threads) = config.worker_threads {
            builder.worker_threads(threads);
        }
        
        builder
            .thread_name("mister-smith-worker")
            .thread_stack_size(config.thread_stack_size.unwrap_or(2 * 1024 * 1024))
            .enable_all();
            
        let runtime = Arc::new(builder.build()?);
        
        Ok(Self {
            runtime,
            config,
            shutdown_token: tokio_util::sync::CancellationToken::new(),
            components: Arc::new(dashmap::DashMap::new()),
        })
    }
    
    pub async fn register_component<C: SystemComponent + 'static>(
        &self,
        component: C,
    ) -> Result<(), SystemError> {
        let name = component.name().to_string();
        self.components.insert(name.clone(), Box::new(component));
        tracing::info!(component = %name, "Component registered");
        Ok(())
    }
    
    pub async fn start_all(&self) -> Result<(), SystemError> {
        let futures: Vec<_> = self.components
            .iter()
            .map(|entry| {
                let component = entry.value();
                let name = entry.key().clone();
                async move {
                    component.start().await
                        .with_context(format!("Starting component: {}", name))
                }
            })
            .collect();
            
        futures::future::try_join_all(futures).await?;
        Ok(())
    }
}
```

### Phase 2: Supervision Tree Implementation (Weeks 3-5)

#### Step 2.1: Convert Supervision Tree Pseudocode to Rust

```rust
// mister-smith-supervision/src/tree/mod.rs
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use dashmap::DashMap;
use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupervisionStrategy {
    OneForOne,
    OneForAll,
    RestForOne,
    Escalate,
}

#[derive(Debug, Clone)]
pub struct RestartPolicy {
    pub max_restarts: u32,
    pub within_duration: std::time::Duration,
    pub restart_delay: std::time::Duration,
    pub backoff_multiplier: f64,
}

pub struct SupervisionTree {
    root_supervisor: Arc<RwLock<Box<dyn Supervisor>>>,
    node_registry: Arc<DashMap<NodeId, SupervisorNode>>,
    failure_detector: Arc<FailureDetector>,
    restart_policies: Arc<DashMap<String, RestartPolicy>>,
}

#[async_trait]
pub trait Supervisor: Send + Sync {
    type Child: Send + Sync + 'static;
    
    async fn supervise(&self, children: Vec<Self::Child>) -> Result<SupervisionResult, SupervisionError>;
    fn supervision_strategy(&self) -> SupervisionStrategy;
    fn restart_policy(&self) -> RestartPolicy;
    fn escalation_policy(&self) -> EscalationPolicy;
    
    // Hub-and-spoke routing implementation
    async fn route_task(&self, task: Task) -> Result<AgentId, RoutingError> {
        use crate::routing::TaskRouter;
        
        let router = TaskRouter::new();
        match task.task_type {
            TaskType::Research => router.find_agent("researcher").await,
            TaskType::Code => router.find_agent("coder").await,
            TaskType::Analysis => router.find_agent("analyst").await,
            _ => router.default_agent().await,
        }
    }
}

pub struct SupervisorNode {
    node_id: NodeId,
    node_type: String,
    children: Arc<RwLock<Vec<ChildRef>>>,
    supervisor_ref: Option<Arc<dyn Supervisor<Child = Box<dyn Any + Send + Sync>>>>,
    state: Arc<Mutex<SupervisorState>>,
    metrics: Arc<SupervisionMetrics>,
    restart_history: Arc<Mutex<RestartHistory>>,
}

#[derive(Debug)]
struct RestartHistory {
    attempts: Vec<RestartAttempt>,
    window_start: std::time::Instant,
}

impl SupervisorNode {
    pub async fn handle_child_failure(
        &self,
        child_id: NodeId,
        error: Box<dyn std::error::Error + Send + Sync>,
    ) -> Result<SupervisionDecision, SupervisionError> {
        let strategy = self.supervisor_ref
            .as_ref()
            .ok_or(SupervisionError::NoSupervisor)?
            .supervision_strategy();
            
        tracing::warn!(
            ?child_id,
            ?error,
            ?strategy,
            "Handling child failure"
        );
        
        match strategy {
            SupervisionStrategy::OneForOne => {
                self.restart_child(child_id).await?;
                Ok(SupervisionDecision::Handled)
            }
            SupervisionStrategy::OneForAll => {
                self.restart_all_children().await?;
                Ok(SupervisionDecision::Handled)
            }
            SupervisionStrategy::RestForOne => {
                self.restart_child_and_siblings(child_id).await?;
                Ok(SupervisionDecision::Handled)
            }
            SupervisionStrategy::Escalate => {
                Ok(SupervisionDecision::Escalate(error))
            }
        }
    }
    
    async fn restart_child(&self, child_id: NodeId) -> Result<(), SupervisionError> {
        let mut children = self.children.write().await;
        
        let child_index = children.iter()
            .position(|c| c.id() == child_id)
            .ok_or(SupervisionError::ChildNotFound)?;
            
        let child_ref = &mut children[child_index];
        
        // Check restart policy
        if !self.should_restart(&child_ref).await? {
            return Err(SupervisionError::RestartLimitExceeded);
        }
        
        // Stop the old child
        child_ref.stop().await?;
        
        // Apply restart delay with exponential backoff
        let delay = self.calculate_restart_delay().await;
        tokio::time::sleep(delay).await;
        
        // Start new instance
        let new_child = child_ref.create_new().await?;
        children[child_index] = new_child;
        
        // Update metrics
        self.metrics.record_restart(child_id).await;
        
        Ok(())
    }
    
    async fn should_restart(&self, child: &ChildRef) -> Result<bool, SupervisionError> {
        let mut history = self.restart_history.lock().await;
        let policy = self.restart_policy();
        
        // Clean old attempts outside the window
        let window_end = history.window_start + policy.within_duration;
        if std::time::Instant::now() > window_end {
            history.attempts.clear();
            history.window_start = std::time::Instant::now();
        }
        
        // Check if we've exceeded max restarts
        Ok(history.attempts.len() < policy.max_restarts as usize)
    }
}
```

#### Step 2.2: Implement Failure Detection

```rust
// mister-smith-supervision/src/detector/mod.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

pub struct FailureDetector {
    heartbeat_interval: Duration,
    failure_threshold: u32,
    monitored_nodes: Arc<RwLock<HashMap<NodeId, NodeHealth>>>,
    phi_detector: Arc<PhiAccrualFailureDetector>,
    event_bus: Arc<EventBus>,
}

#[derive(Debug, Clone)]
struct NodeHealth {
    last_heartbeat: Instant,
    consecutive_failures: u32,
    phi_value: f64,
    status: HealthStatus,
}

impl FailureDetector {
    pub fn new(
        heartbeat_interval: Duration,
        failure_threshold: u32,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            heartbeat_interval,
            failure_threshold,
            monitored_nodes: Arc::new(RwLock::new(HashMap::new())),
            phi_detector: Arc::new(PhiAccrualFailureDetector::new()),
            event_bus,
        }
    }
    
    pub async fn monitor_node(&self, node_id: NodeId) -> Result<(), FailureError> {
        let mut interval = tokio::time::interval(self.heartbeat_interval);
        let shutdown_token = self.shutdown_token.clone();
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    match self.send_heartbeat(node_id).await {
                        Ok(response) => {
                            self.update_node_health(node_id, response).await?;
                        }
                        Err(e) => {
                            self.handle_heartbeat_failure(node_id, e).await?;
                        }
                    }
                }
                _ = shutdown_token.cancelled() => {
                    tracing::info!(?node_id, "Stopping node monitoring");
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn update_node_health(
        &self,
        node_id: NodeId,
        response: HeartbeatResponse,
    ) -> Result<(), FailureError> {
        let mut nodes = self.monitored_nodes.write().await;
        let phi = self.phi_detector.phi(node_id, response.timestamp);
        
        let health = nodes.entry(node_id).or_insert(NodeHealth {
            last_heartbeat: Instant::now(),
            consecutive_failures: 0,
            phi_value: 0.0,
            status: HealthStatus::Healthy,
        });
        
        health.last_heartbeat = Instant::now();
        health.consecutive_failures = 0;
        health.phi_value = phi;
        
        if phi > CONFIGURABLE_PHI_THRESHOLD {
            health.status = HealthStatus::Suspected;
            self.report_suspicion(node_id, phi).await?;
        } else {
            health.status = HealthStatus::Healthy;
        }
        
        Ok(())
    }
}
```

#### Step 2.3: Implement Circuit Breaker Pattern

```rust
// mister-smith-supervision/src/recovery/circuit_breaker.rs
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};
use std::future::Future;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitBreakerState>>,
    config: CircuitBreakerConfig,
    metrics: Arc<CircuitBreakerMetrics>,
}

struct CircuitBreakerState {
    current: CircuitState,
    failure_count: u32,
    last_failure_time: Option<Instant>,
    half_open_calls: u32,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub timeout: Duration,
    pub half_open_max_calls: u32,
    pub success_threshold: u32,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitBreakerState {
                current: CircuitState::Closed,
                failure_count: 0,
                last_failure_time: None,
                half_open_calls: 0,
            })),
            config,
            metrics: Arc::new(CircuitBreakerMetrics::new()),
        }
    }
    
    pub async fn call<F, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        let mut state = self.state.lock().await;
        
        // Check if we should transition from Open to HalfOpen
        if state.current == CircuitState::Open {
            if let Some(last_failure) = state.last_failure_time {
                if last_failure.elapsed() >= self.config.timeout {
                    state.current = CircuitState::HalfOpen;
                    state.half_open_calls = 0;
                }
            }
        }
        
        match state.current {
            CircuitState::Closed => {
                drop(state); // Release lock during operation
                
                match operation.await {
                    Ok(result) => {
                        self.record_success().await;
                        Ok(result)
                    }
                    Err(e) => {
                        self.record_failure().await?;
                        Err(CircuitBreakerError::OperationFailed(e))
                    }
                }
            }
            CircuitState::Open => {
                self.metrics.record_rejection();
                Err(CircuitBreakerError::Open)
            }
            CircuitState::HalfOpen => {
                if state.half_open_calls >= self.config.half_open_max_calls {
                    Err(CircuitBreakerError::HalfOpenLimitExceeded)
                } else {
                    state.half_open_calls += 1;
                    drop(state); // Release lock during operation
                    
                    match operation.await {
                        Ok(result) => {
                            self.record_half_open_success().await;
                            Ok(result)
                        }
                        Err(e) => {
                            self.record_half_open_failure().await;
                            Err(CircuitBreakerError::OperationFailed(e))
                        }
                    }
                }
            }
        }
    }
    
    async fn record_failure(&self) -> Result<(), CircuitBreakerError<std::io::Error>> {
        let mut state = self.state.lock().await;
        state.failure_count += 1;
        state.last_failure_time = Some(Instant::now());
        
        if state.failure_count >= self.config.failure_threshold {
            state.current = CircuitState::Open;
            self.metrics.record_state_change(CircuitState::Open);
            tracing::warn!("Circuit breaker opened after {} failures", state.failure_count);
        }
        
        Ok(())
    }
}
```

### Phase 3: Component Integration (Weeks 6-8)

#### Step 3.1: Actor System Integration

```rust
// mister-smith-actors/src/core/actor_system.rs
use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::mpsc;

pub struct ActorSystem {
    actors: Arc<DashMap<ActorId, ActorContainer>>,
    supervision_tree: Arc<SupervisionTree>,
    event_bus: Arc<EventBus>,
    router: Arc<ActorRouter>,
    config: ActorSystemConfig,
}

struct ActorContainer {
    actor: Box<dyn Actor>,
    mailbox: ActorMailbox,
    supervisor: Option<NodeId>,
    state: ActorState,
}

impl ActorSystem {
    pub async fn spawn_actor<A: Actor + 'static>(
        &self,
        actor: A,
        config: ActorConfig,
    ) -> Result<ActorRef<A::Message>, ActorError> {
        let actor_id = ActorId::new();
        let (sender, receiver) = mpsc::channel(config.mailbox_size.unwrap_or(1000));
        
        let mailbox = ActorMailbox::new(receiver, config.mailbox_type);
        let container = ActorContainer {
            actor: Box::new(actor),
            mailbox,
            supervisor: config.supervisor,
            state: ActorState::Starting,
        };
        
        // Register with supervision tree if supervised
        if let Some(supervisor_id) = config.supervisor {
            self.supervision_tree
                .register_child(supervisor_id, actor_id)
                .await?;
        }
        
        self.actors.insert(actor_id, container);
        
        // Start actor processing loop
        self.start_actor_loop(actor_id).await?;
        
        Ok(ActorRef::new(actor_id, sender))
    }
    
    async fn start_actor_loop(&self, actor_id: ActorId) -> Result<(), ActorError> {
        let system = self.clone();
        
        tokio::spawn(async move {
            loop {
                let container = match system.actors.get(&actor_id) {
                    Some(c) => c,
                    None => break,
                };
                
                match container.mailbox.receive().await {
                    Some(message) => {
                        let result = container.actor.handle_message(message).await;
                        
                        match result {
                            Ok(ActorResult::Continue) => continue,
                            Ok(ActorResult::Stop) => {
                                system.stop_actor(actor_id).await;
                                break;
                            }
                            Ok(ActorResult::Restart) => {
                                system.restart_actor(actor_id).await;
                            }
                            Err(e) => {
                                system.handle_actor_error(actor_id, e).await;
                            }
                        }
                    }
                    None => {
                        // Mailbox closed, stop actor
                        system.stop_actor(actor_id).await;
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }
}
```

#### Step 3.2: Event Bus Implementation

```rust
// mister-smith-events/src/bus/mod.rs
use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::broadcast;
use async_trait::async_trait;

pub struct EventBus {
    topics: Arc<DashMap<String, TopicChannel>>,
    subscribers: Arc<DashMap<SubscriberId, SubscriberInfo>>,
    config: EventBusConfig,
    metrics: Arc<EventBusMetrics>,
}

struct TopicChannel {
    sender: broadcast::Sender<Event>,
    subscriber_count: AtomicUsize,
}

#[async_trait]
pub trait EventHandler: Send + Sync {
    type Event: Send + Sync + Clone;
    
    async fn handle(&self, event: Self::Event) -> Result<(), EventError>;
    fn event_type(&self) -> &str;
}

impl EventBus {
    pub fn new(config: EventBusConfig) -> Self {
        Self {
            topics: Arc::new(DashMap::new()),
            subscribers: Arc::new(DashMap::new()),
            config,
            metrics: Arc::new(EventBusMetrics::new()),
        }
    }
    
    pub async fn publish<E: Into<Event>>(&self, topic: &str, event: E) -> Result<(), EventError> {
        let event = event.into();
        
        self.metrics.record_publish(topic);
        
        match self.topics.get(topic) {
            Some(channel) => {
                match channel.sender.send(event.clone()) {
                    Ok(receiver_count) => {
                        tracing::trace!(
                            topic = %topic,
                            receivers = receiver_count,
                            "Event published"
                        );
                        Ok(())
                    }
                    Err(_) => {
                        // No active receivers
                        if self.config.store_undelivered {
                            self.store_event(topic, event).await?;
                        }
                        Ok(())
                    }
                }
            }
            None => {
                if self.config.create_topic_on_publish {
                    self.create_topic(topic)?;
                    self.publish(topic, event).await
                } else {
                    Err(EventError::TopicNotFound(topic.to_string()))
                }
            }
        }
    }
    
    pub async fn subscribe<H: EventHandler + 'static>(
        &self,
        topic: &str,
        handler: H,
    ) -> Result<SubscriptionHandle, EventError> {
        let topic_channel = self.topics.entry(topic.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(self.config.channel_capacity);
                TopicChannel {
                    sender: tx,
                    subscriber_count: AtomicUsize::new(0),
                }
            });
            
        let mut receiver = topic_channel.sender.subscribe();
        let subscriber_id = SubscriberId::new();
        
        // Spawn handler task
        let handler = Arc::new(handler);
        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                if let Err(e) = handler.handle(event).await {
                    tracing::error!(
                        error = ?e,
                        handler = %handler.event_type(),
                        "Event handler failed"
                    );
                }
            }
        });
        
        Ok(SubscriptionHandle::new(subscriber_id, self.clone()))
    }
}
```

#### Step 3.3: Resource Pool Implementation

```rust
// mister-smith-resources/src/pool/mod.rs
use std::sync::Arc;
use tokio::sync::{Semaphore, Mutex};
use std::collections::VecDeque;

pub struct ResourcePool<T: Resource> {
    resources: Arc<Mutex<VecDeque<T>>>,
    semaphore: Arc<Semaphore>,
    factory: Arc<dyn ResourceFactory<Item = T>>,
    health_checker: Arc<dyn HealthChecker<T>>,
    config: PoolConfig,
    metrics: Arc<PoolMetrics>,
}

#[async_trait]
pub trait Resource: Send + Sync {
    async fn is_valid(&self) -> bool;
    async fn reset(&mut self) -> Result<(), ResourceError>;
}

#[async_trait]
pub trait ResourceFactory: Send + Sync {
    type Item: Resource;
    
    async fn create(&self) -> Result<Self::Item, ResourceError>;
    async fn validate(&self, resource: &Self::Item) -> bool;
}

impl<T: Resource + 'static> ResourcePool<T> {
    pub async fn acquire(&self) -> Result<PooledResource<T>, ResourceError> {
        let permit = self.semaphore
            .acquire()
            .await
            .map_err(|_| ResourceError::PoolClosed)?;
            
        self.metrics.record_acquire_attempt();
        
        // Try to get existing resource
        let resource = {
            let mut resources = self.resources.lock().await;
            resources.pop_front()
        };
        
        let resource = match resource {
            Some(mut res) => {
                // Validate existing resource
                if self.health_checker.check(&res).await? {
                    res
                } else {
                    // Resource invalid, create new one
                    self.metrics.record_invalid_resource();
                    self.factory.create().await?
                }
            }
            None => {
                // No resources available, create new one
                self.metrics.record_resource_creation();
                self.factory.create().await?
            }
        };
        
        Ok(PooledResource::new(resource, self.clone(), permit))
    }
    
    pub async fn return_resource(&self, mut resource: T) -> Result<(), ResourceError> {
        // Reset resource state
        resource.reset().await?;
        
        // Validate before returning to pool
        if self.health_checker.check(&resource).await? {
            let mut resources = self.resources.lock().await;
            if resources.len() < self.config.max_size {
                resources.push_back(resource);
                self.metrics.record_resource_return();
            } else {
                // Pool full, discard resource
                drop(resource);
                self.metrics.record_resource_discard();
            }
        } else {
            // Invalid resource, discard
            drop(resource);
            self.metrics.record_invalid_resource();
        }
        
        Ok(())
    }
}
```

### Phase 4: Configuration and Deployment (Weeks 9-10)

#### Step 4.1: Configuration Management

```rust
// mister-smith-core/src/config/mod.rs
use serde::{Deserialize, Serialize};
use std::path::Path;
use config::{Config, Environment, File};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub runtime: RuntimeConfig,
    pub supervision: SupervisionConfig,
    pub actors: ActorSystemConfig,
    pub events: EventBusConfig,
    pub resources: ResourceConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionConfig {
    pub strategies: HashMap<String, SupervisionStrategy>,
    pub restart_policies: HashMap<String, RestartPolicy>,
    pub failure_detection: FailureDetectionConfig,
    pub circuit_breaker: CircuitBreakerConfig,
}

impl SystemConfig {
    pub fn load<P: AsRef<Path>>(config_path: P) -> Result<Self, ConfigError> {
        let mut builder = Config::builder()
            .add_source(File::from(config_path.as_ref()))
            .add_source(Environment::with_prefix("MISTER_SMITH"))
            .set_default("runtime.worker_threads", num_cpus::get())?
            .set_default("supervision.default_strategy", "one_for_one")?
            .set_default("actors.default_mailbox_size", 1000)?;
            
        let config = builder.build()?;
        Ok(config.try_deserialize()?)
    }
    
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate runtime config
        if let Some(threads) = self.runtime.worker_threads {
            if threads == 0 {
                return Err(ConfigError::ValidationFailed(
                    "Worker threads must be > 0".into()
                ));
            }
        }
        
        // Validate supervision config
        for (name, policy) in &self.supervision.restart_policies {
            if policy.max_restarts == 0 {
                return Err(ConfigError::ValidationFailed(
                    format!("Restart policy '{}' has max_restarts = 0", name)
                ));
            }
        }
        
        Ok(())
    }
}
```

#### Step 4.2: System Bootstrap

```rust
// mister-smith-core/src/bootstrap.rs
use crate::{RuntimeManager, SystemConfig, SupervisionTree, ActorSystem, EventBus};

pub struct SystemBootstrap {
    config: SystemConfig,
    runtime_manager: Option<RuntimeManager>,
    supervision_tree: Option<Arc<SupervisionTree>>,
    actor_system: Option<Arc<ActorSystem>>,
    event_bus: Option<Arc<EventBus>>,
}

impl SystemBootstrap {
    pub fn new(config: SystemConfig) -> Self {
        config.validate().expect("Invalid configuration");
        
        Self {
            config,
            runtime_manager: None,
            supervision_tree: None,
            actor_system: None,
            event_bus: None,
        }
    }
    
    pub async fn initialize(&mut self) -> Result<(), SystemError> {
        tracing::info!("Initializing Mister Smith system");
        
        // Initialize runtime
        self.runtime_manager = Some(RuntimeManager::new(self.config.runtime.clone())?);
        
        // Initialize event bus
        self.event_bus = Some(Arc::new(EventBus::new(self.config.events.clone())));
        
        // Initialize supervision tree
        self.supervision_tree = Some(Arc::new(
            SupervisionTree::new(
                self.config.supervision.clone(),
                self.event_bus.as_ref().unwrap().clone(),
            )
        ));
        
        // Initialize actor system
        self.actor_system = Some(Arc::new(
            ActorSystem::new(
                self.config.actors.clone(),
                self.supervision_tree.as_ref().unwrap().clone(),
                self.event_bus.as_ref().unwrap().clone(),
            )
        ));
        
        // Register core components
        if let Some(runtime) = &self.runtime_manager {
            runtime.register_component(self.event_bus.as_ref().unwrap().clone()).await?;
            runtime.register_component(self.supervision_tree.as_ref().unwrap().clone()).await?;
            runtime.register_component(self.actor_system.as_ref().unwrap().clone()).await?;
        }
        
        Ok(())
    }
    
    pub async fn start(&self) -> Result<(), SystemError> {
        tracing::info!("Starting Mister Smith system");
        
        if let Some(runtime) = &self.runtime_manager {
            runtime.start_all().await?;
        }
        
        tracing::info!("System started successfully");
        Ok(())
    }
    
    pub async fn shutdown(self) -> Result<(), SystemError> {
        tracing::info!("Shutting down Mister Smith system");
        
        if let Some(runtime) = self.runtime_manager {
            runtime.graceful_shutdown().await?;
        }
        
        tracing::info!("System shutdown complete");
        Ok(())
    }
}
```

## Testing and Validation

### Unit Testing Framework

```rust
// tests/unit/supervision_tree_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    
    #[test]
    async fn test_one_for_one_supervision() {
        let supervisor = TestSupervisor::new(SupervisionStrategy::OneForOne);
        let child1 = TestChild::new("child1");
        let child2 = TestChild::new("child2");
        
        supervisor.add_child(child1).await;
        supervisor.add_child(child2).await;
        
        // Simulate child1 failure
        supervisor.fail_child("child1").await;
        
        // Verify only child1 was restarted
        assert!(supervisor.was_restarted("child1").await);
        assert!(!supervisor.was_restarted("child2").await);
    }
    
    #[test]
    async fn test_circuit_breaker_transitions() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            timeout: Duration::from_millis(100),
            half_open_max_calls: 1,
            success_threshold: 1,
        });
        
        // Should start closed
        assert_eq!(breaker.state().await, CircuitState::Closed);
        
        // Fail 3 times to open circuit
        for _ in 0..3 {
            let _ = breaker.call(failing_operation()).await;
        }
        
        assert_eq!(breaker.state().await, CircuitState::Open);
        
        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Should transition to half-open
        let _ = breaker.call(successful_operation()).await;
        assert_eq!(breaker.state().await, CircuitState::HalfOpen);
    }
}
```

### Integration Testing

```rust
// tests/integration/system_integration_tests.rs
#[test]
async fn test_full_system_startup_shutdown() {
    let config = SystemConfig::load("tests/fixtures/test_config.toml").unwrap();
    let mut bootstrap = SystemBootstrap::new(config);
    
    // Initialize and start system
    bootstrap.initialize().await.unwrap();
    bootstrap.start().await.unwrap();
    
    // Verify all components are running
    let health = bootstrap.health_check().await.unwrap();
    assert!(health.all_healthy());
    
    // Graceful shutdown
    bootstrap.shutdown().await.unwrap();
}

#[test]
async fn test_actor_supervision_integration() {
    let system = create_test_system().await;
    
    // Spawn supervised actor
    let actor = FailingActor::new(3); // Fails after 3 messages
    let actor_ref = system.actor_system
        .spawn_actor(actor, ActorConfig::supervised())
        .await
        .unwrap();
        
    // Send messages
    for i in 0..5 {
        actor_ref.send(TestMessage(i)).await.unwrap();
    }
    
    // Verify actor was restarted after failure
    let metrics = system.supervision_tree.metrics().await;
    assert_eq!(metrics.restarts_for_actor(actor_ref.id()), 1);
}
```

### Performance Testing

```rust
// benches/supervision_benchmarks.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn supervision_tree_benchmarks(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("supervision");
    
    for num_children in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("restart_child", num_children),
            num_children,
            |b, &num| {
                b.to_async(&runtime).iter(|| async {
                    let tree = create_tree_with_children(num).await;
                    tree.restart_child(NodeId::new()).await.unwrap();
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, supervision_tree_benchmarks);
criterion_main!(benches);
```

## Deployment Configuration

### Docker Configuration

```dockerfile
# Dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/mister-smith /usr/local/bin/

EXPOSE 8080 9090
CMD ["mister-smith"]
```

### Kubernetes Deployment

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mister-smith
  namespace: mister-smith
spec:
  replicas: 3
  selector:
    matchLabels:
      app: mister-smith
  template:
    metadata:
      labels:
        app: mister-smith
    spec:
      containers:
      - name: mister-smith
        image: mister-smith:latest
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: metrics
        env:
        - name: MISTER_SMITH_RUNTIME__WORKER_THREADS
          value: "8"
        - name: MISTER_SMITH_SUPERVISION__DEFAULT_STRATEGY
          value: "one_for_one"
        resources:
          requests:
            cpu: 2
            memory: 4Gi
          limits:
            cpu: 4
            memory: 8Gi
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

## Resource Requirements and Timeline

### Development Team Structure

```
Team Lead (1)
├── Core Systems Team (2 developers)
│   ├── Runtime & Configuration
│   └── Error Handling & Logging
├── Supervision Team (2 developers)
│   ├── Supervision Tree Implementation
│   └── Failure Detection & Recovery
└── Integration Team (2 developers)
    ├── Actor System & Event Bus
    └── Resource Management & Testing
```

### Implementation Timeline

| Week | Phase | Deliverables |
|------|-------|--------------|
| 1-2 | Core System | Runtime manager, error types, configuration |
| 3-5 | Supervision Tree | Convert pseudocode to Rust, implement all supervision patterns |
| 6-8 | Integration | Actor system, event bus, resource pools |
| 9-10 | Configuration | System bootstrap, deployment configs |
| 11-12 | Testing & Documentation | Unit/integration tests, performance benchmarks |

### Critical Milestones

1. **Week 2**: Core runtime operational with basic configuration
2. **Week 5**: Supervision tree fully implemented and tested
3. **Week 8**: All components integrated and communicating
4. **Week 10**: System deployable to Kubernetes
5. **Week 12**: Production-ready with full test coverage

## Risk Mitigation

### Technical Risks

1. **Async Complexity**: Use structured concurrency patterns, extensive testing
2. **Performance Bottlenecks**: Profile early, optimize critical paths
3. **Integration Issues**: Continuous integration testing, clear interfaces

### Process Risks

1. **Timeline Slippage**: Buffer time included, parallel work streams
2. **Resource Availability**: Cross-training, documentation
3. **Scope Creep**: Strict change control, phased delivery

## Success Criteria

1. ✅ All pseudocode converted to production Rust
2. ✅ Supervision tree handles all failure scenarios
3. ✅ 90%+ test coverage with integration tests
4. ✅ Performance benchmarks meet requirements
5. ✅ Successful deployment to Kubernetes cluster
6. ✅ Documentation complete for all components

---

**Implementation Plan Status**: READY FOR EXECUTION  
*MS Framework System Architecture Implementation | Agent-01 | 2025-07-05*