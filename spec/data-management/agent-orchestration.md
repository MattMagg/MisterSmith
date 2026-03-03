---
title: revised-agent-orchestration
type: note
permalink: revision-swarm/data-management/revised-agent-orchestration
tags:
- '#revised-document #agent-orchestration #foundation-focus #validation-warnings'
---

## 🔧 TECHNICAL SPECIFICATIONS: AGENT ORCHESTRATION

**CRITICAL SCHEMA STANDARDIZATION REQUIREMENTS**

1. **Message Priority Scale**: Standardized to 0-4 range (5 levels) for consistent processing
2. **AgentId Format**: Unified pattern using UUID v4 format for global uniqueness
3. **Security Integration**: mTLS and authentication required for all agent communication
4. **Distributed Architecture**: Multi-node supervision tree coordination patterns

**Implementation Notes**: This document provides async Rust patterns for actor-based agent
orchestration using Tokio runtime and supervision trees.

---

## Agent Orchestration & Supervision Architecture

## Foundation Patterns Guide

> **Canonical Reference**: See `tech-framework.md` for authoritative technology stack specifications

## Executive Summary

**Technical Foundation**: Actor-model orchestration using Tokio supervision trees

This document defines async Rust patterns for agent orchestration using the actor model with Tokio runtime.
It provides supervision tree implementations, async agent lifecycle management, and coordination patterns
for distributed agent systems. These patterns integrate with the operational patterns defined in
[Agent Operations](./agent-operations.md) for complete system orchestration.

### Integration with Agent Operations

Orchestration patterns coordinate with operational patterns through:

- **Discovery Integration**: Uses `AgentRegistry` from agent-operations.md for capability-based agent discovery
- **Health Monitoring**: Integrates with `HealthMonitor` for supervision decisions
- **Error Handling**: Shares `ErrorHandler` and `CircuitBreaker` patterns for fault tolerance
- **Metrics Collection**: Coordinates with `AgentMetrics` for observability across supervision trees

### Schema Standardization Requirements

**Message Priority Scale**: Unified 0-4 priority levels

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Critical = 0,  // System-critical messages
    High = 1,      // Important operational messages  
    Normal = 2,    // Standard agent communication
    Low = 3,       // Background tasks
    Bulk = 4,      // Batch operations
}
```

**AgentId Format**: UUID v4 with agent prefix

```rust
pub type AgentId = String;  // Format: "agent-{uuid-v4}"

// Validation pattern
const AGENT_ID_PATTERN: &str = r"^agent-[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$";
```

**Security Integration**: mTLS with message authentication

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct SecureMessage {
    pub content: MessageContent,
    pub sender_id: AgentId,
    pub signature: String,      // HMAC-SHA256 signature
    pub timestamp: i64,         // Unix timestamp
    pub nonce: String,          // Prevents replay attacks
}
```

## 1. Basic Agent Architecture

**Implementation Requirements**:

- Resource limits configured per agent type using Tokio runtime constraints
- Memory/CPU monitoring integrated with supervision tree decisions
- Thread-safe state management using `Arc<RwLock<T>>` patterns

### 1.1 Agent Types

```rust
use tokio::sync::mpsc;
use std::sync::Arc;
use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    Supervisor,    // Manages supervision trees
    Worker,        // Performs computational tasks
    Coordinator,   // Orchestrates workflows
    Monitor,       // Observes system state
    Planner,       // Decomposes goals into tasks
    Executor,      // Carries out atomic actions
    Critic,        // Validates outcomes
    Router,        // Load balances and routes tasks
    Memory,        // Persistent knowledge storage
}

#[async_trait]
pub trait Agent: Send + Sync {
    async fn start(&self) -> Result<(), AgentError>;
    async fn stop(&self) -> Result<(), AgentError>;
    async fn handle_message(&self, message: Message) -> Result<(), AgentError>;
    async fn get_status(&self) -> AgentStatus;
    fn agent_id(&self) -> &AgentId;
    fn agent_type(&self) -> AgentType;
}
```

### 1.2 Core Agent Role Taxonomies

#### Planner Agent

- **Purpose**: Decomposes high-level goals into concrete subtasks
- **Interface Pattern**:

```rust
#[async_trait]
pub trait Planner: Agent {
    async fn create_plan(&self, goal: Goal) -> Result<TaskList, PlannerError>;
    async fn refine_plan(&self, feedback: CriticFeedback) -> Result<TaskList, PlannerError>;
    async fn validate_dependencies(&self, tasks: &[Task]) -> Result<(), DependencyError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskList {
    pub tasks: Vec<Task>,
    pub dependencies: HashMap<TaskId, Vec<TaskId>>,
    pub priority_order: Vec<TaskId>,
    pub estimated_duration: Duration,
    pub resource_requirements: ResourceRequirements,
}
```

#### Executor Agent

- **Purpose**: Carries out atomic actions and subtasks
- **Interface Pattern**:

```rust
#[async_trait]
pub trait Executor: Agent {
    async fn execute_task(&self, task: Task) -> Result<TaskOutput, ExecutorError>;
    async fn can_execute(&self, task_type: &TaskType) -> bool;
    async fn get_capacity(&self) -> ExecutorCapacity;
    async fn cancel_task(&self, task_id: TaskId) -> Result<(), ExecutorError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskOutput {
    Success { 
        result: serde_json::Value,
        metrics: ExecutionMetrics,
    },
    PartialResult { 
        partial_result: serde_json::Value,
        remaining_tasks: Vec<SubTask>,
        completion_percentage: f32,
    },
    Failed { 
        error: ExecutorError,
        retry_possible: bool,
        partial_work: Option<serde_json::Value>,
    },
}
```

#### Critic Agent

- **Purpose**: Validates outcomes against goals and quality criteria
- **Interface Pattern**:

```rust
#[async_trait]
pub trait Critic: Agent {
    async fn evaluate(&self, output: TaskOutput, criteria: QualityCriteria) -> CriticFeedback;
    async fn validate_plan(&self, plan: &TaskList) -> ValidationResult;
    async fn continuous_monitoring(&self, agent_id: AgentId) -> Result<(), CriticError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticFeedback {
    pub score: f32,              // 0.0 to 1.0 quality score
    pub confidence: f32,         // Confidence in the evaluation
    pub issues: Vec<Issue>,
    pub suggestions: Vec<Improvement>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub evaluation_duration: Duration,
}
```

#### Router Agent

- **Purpose**: Assigns tasks to appropriate specialized agents
- **Interface Pattern**:

```rust
#[async_trait]
pub trait Router: Agent {
    async fn route_task(&self, task: Task) -> Result<AgentId, RoutingError>;
    async fn get_agent_capabilities(&self, agent_id: &AgentId) -> Result<Vec<Capability>, RoutingError>;
    async fn balance_load(&self, tasks: Vec<Task>) -> Result<HashMap<AgentId, Vec<Task>>, RoutingError>;
    async fn update_agent_load(&self, agent_id: &AgentId, load: LoadMetrics) -> Result<(), RoutingError>;
    
    /// Integrates with AgentRegistry from agent-operations.md
    async fn discover_agents(&self, requirements: &TaskRequirements) -> Result<Vec<AgentId>, RoutingError>;
}
```

#### Memory Agent

- **Purpose**: Stores and retrieves shared knowledge
- **Interface Pattern**:

```rust
#[async_trait]
pub trait Memory: Agent {
    async fn store(&self, key: String, value: serde_json::Value, metadata: Metadata) -> Result<(), MemoryError>;
    async fn retrieve(&self, key: &str) -> Result<Option<(serde_json::Value, Metadata)>, MemoryError>;
    async fn query(&self, pattern: QueryPattern) -> Result<Vec<(String, serde_json::Value)>, MemoryError>;
    async fn delete(&self, key: &str) -> Result<bool, MemoryError>;
    async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>, MemoryError>;
    
    /// Transaction support for atomic operations
    async fn transaction<F, R>(&self, operation: F) -> Result<R, MemoryError>
    where
        F: FnOnce(&mut MemoryTransaction) -> Result<R, MemoryError> + Send,
        R: Send;
}
```

### 1.3 Agent Lifecycle State Machine

**Tokio Implementation**: Async state machine using `Arc<RwLock<AgentState>>` for thread-safe transitions with supervision tree integration.

#### 1.3.1 Agent State Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://mister-smith.ai/schemas/agent-state",
  "title": "Agent State Definition",
  "type": "object",
  "required": ["current_state", "previous_state", "transition_timestamp", "state_data"],
  "properties": {
    "current_state": {
      "enum": ["INITIALIZING", "RUNNING", "PAUSED", "STOPPING", "TERMINATED", "ERROR", "RESTARTING"],
      "description": "Current agent state"
    },
    "previous_state": {
      "enum": ["INITIALIZING", "RUNNING", "PAUSED", "STOPPING", "TERMINATED", "ERROR", "RESTARTING", null],
      "description": "Previous agent state (null for initial state)"
    },
    "transition_timestamp": {
      "type": "string",
      "format": "date-time",
      "description": "When the state transition occurred"
    },
    "state_data": {
      "type": "object",
      "description": "State-specific metadata",
      "properties": {
        "initialization_progress": {
          "type": "number",
          "minimum": 0,
          "maximum": 100,
          "description": "Initialization completion percentage"
        },
        "pause_reason": {
          "enum": ["MANUAL", "RESOURCE_CONSTRAINT", "DEPENDENCY_WAIT", "ERROR_RECOVERY"]
        },
        "termination_reason": {
          "enum": ["MANUAL", "COMPLETED", "ERROR", "TIMEOUT", "RESOURCE_EXHAUSTED"]
        },
        "error_details": {
          "type": "object",
          "properties": {
            "error_code": { "type": "string" },
            "error_message": { "type": "string" },
            "recovery_attempted": { "type": "boolean" },
            "retry_count": { "type": "integer", "minimum": 0 }
          }
        },
        "restart_count": {
          "type": "integer",
          "minimum": 0,
          "description": "Number of restarts for this agent instance"
        }
      }
    },
    "transition_history": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "from_state": { "type": "string" },
          "to_state": { "type": "string" },
          "timestamp": { "type": "string", "format": "date-time" },
          "trigger": { "type": "string" },
          "duration_ms": { "type": "integer", "minimum": 0 }
        },
        "required": ["from_state", "to_state", "timestamp"]
      },
      "maxItems": 100,
      "description": "Recent state transition history"
    }
  }
}
```

#### 1.3.2 State Transition Rules Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://mister-smith.ai/schemas/state-transitions",
  "title": "Agent State Transition Rules",
  "type": "object",
  "description": "Valid state transitions and their constraints",
  "properties": {
    "INITIALIZING": {
      "type": "object",
      "properties": {
        "allowed_transitions": {
          "type": "array",
          "items": { "enum": ["RUNNING", "ERROR", "TERMINATED"] },
          "uniqueItems": true
        },
        "timeout_seconds": { "type": "integer", "default": 30 },
        "required_conditions": {
          "type": "array",
          "items": { "enum": ["CONFIGURATION_LOADED", "RESOURCES_ALLOCATED", "DEPENDENCIES_READY"] }
        }
      }
    },
    "RUNNING": {
      "type": "object",
      "properties": {
        "allowed_transitions": {
          "type": "array",
          "items": { "enum": ["PAUSED", "STOPPING", "ERROR", "RESTARTING"] },
          "uniqueItems": true
        },
        "health_check_interval_seconds": { "type": "integer", "default": 30 },
        "max_idle_seconds": { "type": "integer", "default": 300 }
      }
    },
    "PAUSED": {
      "type": "object",
      "properties": {
        "allowed_transitions": {
          "type": "array",
          "items": { "enum": ["RUNNING", "STOPPING", "ERROR"] },
          "uniqueItems": true
        },
        "max_pause_duration_seconds": { "type": "integer", "default": 3600 },
        "auto_resume_conditions": {
          "type": "array",
          "items": { "enum": ["RESOURCE_AVAILABLE", "DEPENDENCY_RESOLVED", "MANUAL_TRIGGER"] }
        }
      }
    },
    "STOPPING": {
      "type": "object",
      "properties": {
        "allowed_transitions": {
          "type": "array",
          "items": { "enum": ["TERMINATED", "ERROR"] },
          "uniqueItems": true
        },
        "graceful_shutdown_timeout_seconds": { "type": "integer", "default": 60 },
        "force_kill_after_timeout": { "type": "boolean", "default": true }
      }
    },
    "ERROR": {
      "type": "object",
      "properties": {
        "allowed_transitions": {
          "type": "array",
          "items": { "enum": ["RESTARTING", "TERMINATED"] },
          "uniqueItems": true
        },
        "auto_restart_conditions": {
          "type": "array",
          "items": { "enum": ["RECOVERABLE_ERROR", "RETRY_LIMIT_NOT_EXCEEDED", "SUPERVISOR_POLICY"] }
        },
        "max_restart_attempts": { "type": "integer", "default": 3 },
        "restart_backoff_seconds": { "type": "integer", "default": 10 }
      }
    },
    "RESTARTING": {
      "type": "object",
      "properties": {
        "allowed_transitions": {
          "type": "array",
          "items": { "enum": ["INITIALIZING", "ERROR", "TERMINATED"] },
          "uniqueItems": true
        },
        "restart_timeout_seconds": { "type": "integer", "default": 45 }
      }
    },
    "TERMINATED": {
      "type": "object",
      "properties": {
        "allowed_transitions": {
          "type": "array",
          "items": [],
          "description": "Terminal state - no transitions allowed"
        },
        "cleanup_timeout_seconds": { "type": "integer", "default": 30 }
      }
    }
  }
}
```

#### 1.3.3 Agent Lifecycle Management

```rust
CLASS AgentLifecycle {
    PRIVATE state: AgentState
    PRIVATE transitionRules: StateTransitionRules
    
    FUNCTION transition(newState: AgentState, trigger: String) -> Result {
        currentRule = transitionRules[state.current_state]
        
        IF NOT currentRule.allowed_transitions.contains(newState) THEN
            RETURN Failure("Invalid transition from " + state.current_state + " to " + newState)
        END IF
        
        IF NOT checkTransitionConditions(state.current_state, newState) THEN
            RETURN Failure("Transition conditions not met")
        END IF
        
        previousState = state.current_state
        state.previous_state = previousState
        state.current_state = newState
        state.transition_timestamp = NOW()
        
        recordTransitionHistory(previousState, newState, trigger)
        notifyObservers(state)
        
        RETURN Success()
    }
    
    FUNCTION checkTransitionConditions(fromState: String, toState: String) -> Boolean {
        // Implementation-specific condition checking
        RETURN validateRequiredConditions(fromState, toState)
    }
}
```

## 2. Supervision Patterns

**Implementation Guide**: Async supervision patterns using Tokio runtime with channel-based communication and hierarchical fault tolerance.

### 2.1 Hub-and-Spoke Supervisor Pattern

Central routing logic with domain-specific delegation:

```rust
use tokio::sync::{mpsc, RwLock};
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait Supervisor: Agent {
    async fn route_task(&self, task: Task) -> Result<AgentId, SupervisionError>;
    async fn spawn_child(&self, agent_type: AgentType, config: AgentConfig) -> Result<AgentId, SupervisionError>;
    async fn terminate_child(&self, agent_id: &AgentId) -> Result<(), SupervisionError>;
    async fn handle_child_failure(&self, agent_id: &AgentId, error: AgentError) -> Result<(), SupervisionError>;
}

#[derive(Clone)]
pub struct HubSupervisor {
    children: Arc<RwLock<HashMap<AgentId, ChildAgent>>>,
    task_tx: mpsc::UnboundedSender<SupervisionCommand>,
    strategy: SupervisionStrategy,
    agent_registry: Arc<AgentRegistry>,  // From agent-operations.md
}

#[derive(Debug)]
struct ChildAgent {
    agent: Arc<dyn Agent>,
    handle: tokio::task::JoinHandle<Result<(), AgentError>>,
    restart_count: u32,
    last_restart: Option<std::time::Instant>,
}

impl HubSupervisor {
    pub fn new(strategy: SupervisionStrategy, agent_registry: Arc<AgentRegistry>) -> Self {
        let (task_tx, task_rx) = mpsc::unbounded_channel();
        
        let supervisor = Self {
            children: Arc::new(RwLock::new(HashMap::new())),
            task_tx,
            strategy,
            agent_registry,
        };
        
        // Spawn supervision event loop
        let supervisor_clone = supervisor.clone();
        tokio::spawn(async move {
            supervisor_clone.supervision_loop(task_rx).await
        });
        
        supervisor
    }
    
    async fn supervision_loop(&self, mut task_rx: mpsc::UnboundedReceiver<SupervisionCommand>) {
        while let Some(command) = task_rx.recv().await {
            if let Err(e) = self.handle_supervision_command(command).await {
                tracing::error!("Supervision command failed: {:?}", e);
            }
        }
    }
    
    async fn handle_supervision_command(&self, command: SupervisionCommand) -> Result<(), SupervisionError> {
        match command {
            SupervisionCommand::SpawnAgent { agent_type, config, response_tx } => {
                let result = self.spawn_child_internal(agent_type, config).await;
                let _ = response_tx.send(result);
            },
            SupervisionCommand::TerminateAgent { agent_id, response_tx } => {
                let result = self.terminate_child_internal(&agent_id).await;
                let _ = response_tx.send(result);
            },
            SupervisionCommand::AgentFailed { agent_id, error } => {
                self.handle_child_failure_internal(&agent_id, error).await?;
            }
        }
        Ok(())
    }
}
```

### 2.2 Event-Driven Message Bus Pattern

```rust
use tokio::sync::{broadcast, mpsc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub enum RoutingMode {
    Direct(AgentId),           // Send to specific agent
    Broadcast,                 // Send to all agents
    RoundRobin,               // Distribute evenly
    LoadBalanced,             // Send to least loaded agent
    CapabilityBased(String),  // Send to agent with capability
}

#[derive(Clone)]
pub struct MessageBus {
    agent_channels: Arc<RwLock<HashMap<AgentId, mpsc::UnboundedSender<Message>>>>,
    broadcast_tx: broadcast::Sender<Message>,
    routing_strategy: Arc<dyn RoutingStrategy + Send + Sync>,
    metrics: Arc<AgentMetrics>,  // From agent-operations.md
}

impl MessageBus {
    pub fn new(routing_strategy: Arc<dyn RoutingStrategy + Send + Sync>, metrics: Arc<AgentMetrics>) -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);
        
        Self {
            agent_channels: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
            routing_strategy,
            metrics,
        }
    }
    
    pub async fn register_agent(&self, agent_id: AgentId, tx: mpsc::UnboundedSender<Message>) -> Result<(), MessageBusError> {
        let mut channels = self.agent_channels.write().await;
        channels.insert(agent_id, tx);
        Ok(())
    }
    
    pub async fn unregister_agent(&self, agent_id: &AgentId) -> Result<(), MessageBusError> {
        let mut channels = self.agent_channels.write().await;
        channels.remove(agent_id);
        Ok(())
    }
    
    pub async fn publish(&self, message: Message) -> Result<(), MessageBusError> {
        self.metrics.record_message();
        
        match message.routing_mode {
            RoutingMode::Direct(agent_id) => {
                self.send_to_agent(&agent_id, message).await
            },
            RoutingMode::Broadcast => {
                self.broadcast_tx.send(message)
                    .map_err(|_| MessageBusError::BroadcastFailed)?;
                Ok(())
            },
            RoutingMode::RoundRobin | RoutingMode::LoadBalanced | RoutingMode::CapabilityBased(_) => {
                let channels = self.agent_channels.read().await;
                let agent_ids: Vec<AgentId> = channels.keys().cloned().collect();
                
                let target_agent = self.routing_strategy.select_recipient(&message, &agent_ids).await
                    .ok_or(MessageBusError::NoAvailableAgents)?;
                    
                self.send_to_agent(&target_agent, message).await
            }
        }
    }
    
    async fn send_to_agent(&self, agent_id: &AgentId, message: Message) -> Result<(), MessageBusError> {
        let channels = self.agent_channels.read().await;
        
        if let Some(tx) = channels.get(agent_id) {
            tx.send(message)
                .map_err(|_| MessageBusError::AgentChannelClosed(agent_id.clone()))
        } else {
            Err(MessageBusError::AgentNotFound(agent_id.clone()))
        }
    }
}

// Enhanced routing strategies with async support
#[async_trait]
pub trait RoutingStrategy: Send + Sync {
    async fn select_recipient(&self, message: &Message, available_agents: &[AgentId]) -> Option<AgentId>;
}
```

### 2.3 Basic Supervision Tree

```rust
use tokio::task::JoinHandle;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub enum RestartStrategy {
    OneForOne,        // Restart only failed agent
    AllForOne,        // Restart all agents when any fails
    RestForOne,       // Restart failed agent and all subsequent ones
    OneForAll,        // One failure terminates all
}

#[derive(Debug, Clone)]
pub struct SupervisionStrategy {
    pub restart_strategy: RestartStrategy,
    pub max_restarts: u32,
    pub restart_window: Duration,
    pub escalation_strategy: EscalationStrategy,
}

#[derive(Debug, Clone)]
pub enum EscalationStrategy {
    Terminate,        // Terminate the supervisor
    Restart,          // Restart the supervisor
    EscalateUp,       // Report to parent supervisor
}

#[derive(Clone)]
pub struct SupervisionTree {
    supervisor_id: AgentId,
    children: Arc<RwLock<HashMap<AgentId, SupervisedChild>>>,
    strategy: SupervisionStrategy,
    parent_tx: Option<mpsc::UnboundedSender<SupervisionEvent>>,
    event_tx: mpsc::UnboundedSender<SupervisionEvent>,
}

#[derive(Debug)]
struct SupervisedChild {
    agent: Arc<dyn Agent>,
    handle: JoinHandle<Result<(), AgentError>>,
    restart_count: u32,
    last_restart: Option<Instant>,
    health_monitor: Option<JoinHandle<()>>,
}

impl SupervisionTree {
    pub fn new(
        supervisor_id: AgentId,
        strategy: SupervisionStrategy,
        parent_tx: Option<mpsc::UnboundedSender<SupervisionEvent>>
    ) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        
        let tree = Self {
            supervisor_id,
            children: Arc::new(RwLock::new(HashMap::new())),
            strategy,
            parent_tx,
            event_tx,
        };
        
        // Spawn supervision event handler
        let tree_clone = tree.clone();
        tokio::spawn(async move {
            tree_clone.handle_supervision_events(event_rx).await
        });
        
        tree
    }
    
    pub async fn supervise_child(&self, agent: Arc<dyn Agent>) -> Result<(), SupervisionError> {
        let agent_id = agent.agent_id().clone();
        
        // Start the agent
        let agent_clone = Arc::clone(&agent);
        let event_tx = self.event_tx.clone();
        let agent_id_clone = agent_id.clone();
        
        let handle = tokio::spawn(async move {
            // Run the agent with error reporting
            let result = agent_clone.start().await;
            
            if let Err(error) = &result {
                let _ = event_tx.send(SupervisionEvent::ChildFailed {
                    agent_id: agent_id_clone,
                    error: error.clone(),
                });
            }
            
            result
        });
        
        // Start health monitoring
        let health_monitor = self.start_health_monitoring(&agent_id).await;
        
        let supervised_child = SupervisedChild {
            agent,
            handle,
            restart_count: 0,
            last_restart: None,
            health_monitor: Some(health_monitor),
        };
        
        let mut children = self.children.write().await;
        children.insert(agent_id, supervised_child);
        
        Ok(())
    }
    
    async fn handle_supervision_events(&self, mut event_rx: mpsc::UnboundedReceiver<SupervisionEvent>) {
        while let Some(event) = event_rx.recv().await {
            if let Err(e) = self.handle_event(event).await {
                tracing::error!(supervisor_id = %self.supervisor_id, "Supervision event handling failed: {:?}", e);
            }
        }
    }
    
    async fn handle_event(&self, event: SupervisionEvent) -> Result<(), SupervisionError> {
        match event {
            SupervisionEvent::ChildFailed { agent_id, error } => {
                self.handle_child_failure(&agent_id, error).await
            },
            SupervisionEvent::HealthCheckFailed(agent_id) => {
                self.handle_health_failure(&agent_id).await
            },
            SupervisionEvent::RestartChild { agent_id } => {
                self.restart_child(&agent_id).await
            },
            _ => Ok(()),
        }
    }
    
    async fn handle_child_failure(&self, agent_id: &AgentId, error: AgentError) -> Result<(), SupervisionError> {
        match self.strategy.restart_strategy {
            RestartStrategy::OneForOne => {
                self.restart_child(agent_id).await
            },
            RestartStrategy::AllForOne => {
                self.restart_all_children().await
            },
            RestartStrategy::RestForOne => {
                self.restart_from_child(agent_id).await
            },
            RestartStrategy::OneForAll => {
                self.terminate_all_children().await
            }
        }
    }
    
    async fn restart_child(&self, agent_id: &AgentId) -> Result<(), SupervisionError> {
        let should_restart = {
            let children = self.children.read().await;
            if let Some(child) = children.get(agent_id) {
                let restart_allowed = child.restart_count < self.strategy.max_restarts;
                let window_ok = child.last_restart
                    .map(|last| last.elapsed() > self.strategy.restart_window)
                    .unwrap_or(true);
                restart_allowed && window_ok
            } else {
                false
            }
        };
        
        if should_restart {
            // Terminate existing agent
            self.terminate_child(agent_id).await?;
            
            // Restart with new instance
            // Implementation depends on agent factory pattern
            // This integrates with AgentRegistry from agent-operations.md
            
            let mut children = self.children.write().await;
            if let Some(child) = children.get_mut(agent_id) {
                child.restart_count += 1;
                child.last_restart = Some(Instant::now());
            }
            
            Ok(())
        } else {
            // Escalate to parent or apply escalation strategy
            self.escalate_failure(agent_id).await
        }
    }
    
    async fn start_health_monitoring(&self, agent_id: &AgentId) -> JoinHandle<()> {
        let agent_id = agent_id.clone();
        let event_tx = self.event_tx.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                
                // Health check logic - integrates with HealthMonitor from agent-operations.md
                // If health check fails, send event
                let _ = event_tx.send(SupervisionEvent::HealthCheckFailed(agent_id.clone()));
            }
        })
    }
}
```

### 2.4 Simple Restart Logic

```rust
CLASS RestartPolicy {
    PRIVATE maxRestarts: Integer
    PRIVATE timeWindow: Duration
    PRIVATE restartCounts: Map<String, List<Timestamp>>
    
    FUNCTION shouldRestart(agentId: String) -> Boolean {
        recentRestarts = countRecentRestarts(agentId, timeWindow)
        RETURN recentRestarts < maxRestarts
    }
    
    FUNCTION recordRestart(agentId: String) {
        restartCounts[agentId].add(NOW())
    }
}
```

## 3. Message Passing and Communication Patterns

### 3.1 Direct RPC Pattern

Point-to-point communication for immediate responses:

```rust
struct DirectChannel {
    target_endpoint: String,
    timeout: Duration,
}

impl DirectChannel {
    async fn call(&self, request: Request) -> Result<Response, Error> {
        // Direct HTTP/gRPC call
        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()?;
        
        let response = client
            .post(&self.target_endpoint)
            .json(&request)
            .send()
            .await?;
            
        Ok(response.json().await?)
    }
}
```

### 3.2 Publish/Subscribe Pattern

Topic-based message distribution:

```rust
struct PubSubBus {
    broker_url: String,
    subscriptions: HashMap<Topic, Vec<CallbackFn>>,
}

impl PubSubBus {
    async fn publish(&self, topic: Topic, message: Message) -> Result<(), Error> {
        // Publish to broker (e.g., NATS)
        let nc = nats::connect(&self.broker_url)?;
        nc.publish(&topic.as_str(), &message.serialize()?)?;
        Ok(())
    }
    
    async fn subscribe<F>(&mut self, topic: Topic, callback: F) 
    where F: Fn(Message) + Send + 'static {
        let nc = nats::connect(&self.broker_url)?;
        let sub = nc.subscribe(&topic.as_str())?;
        
        tokio::spawn(async move {
            for msg in sub.messages() {
                callback(Message::deserialize(&msg.data).unwrap());
            }
        });
    }
}
```

### 3.3 Blackboard Pattern

Shared memory coordination:

```rust
struct Blackboard {
    store: Arc<RwLock<HashMap<String, BlackboardEntry>>>,
    watchers: Arc<RwLock<HashMap<Pattern, Vec<WatcherFn>>>>,
}

struct BlackboardEntry {
    value: Value,
    timestamp: Instant,
    author: AgentId,
    version: u64,
}

impl Blackboard {
    async fn write(&self, key: String, value: Value, agent_id: AgentId) -> Result<(), Error> {
        let mut store = self.store.write().await;
        let version = store.get(&key).map(|e| e.version + 1).unwrap_or(1);
        
        store.insert(key.clone(), BlackboardEntry {
            value: value.clone(),
            timestamp: Instant::now(),
            author: agent_id,
            version,
        });
        
        // Notify watchers
        self.notify_watchers(&key, &value).await;
        Ok(())
    }
    
    async fn read(&self, key: &str) -> Option<BlackboardEntry> {
        self.store.read().await.get(key).cloned()
    }
}
```

### 3.4 Complete Message Schema Definitions

**⚠️ CRITICAL WARNING [Team Alpha]**: These schemas contain the priority scale inconsistencies
that MUST be fixed before implementation. Each schema below requires validation updates after
standardization decision.

#### 3.4.1 Base Message Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://mister-smith.ai/schemas/base-message",
  "title": "Base Agent Message",
  "description": "Core message structure for all agent communications",
  "type": "object",
  "required": ["id", "type", "sender", "timestamp", "routing"],
  "properties": {
    "id": {
      "type": "string",
      "format": "uuid",
      "description": "Unique message identifier"
    },
    "type": {
      "type": "string",
      "description": "Message type discriminator"
    },
    "payload": {
      "type": "object",
      "description": "Message-specific data",
      "additionalProperties": true
    },
    "sender": {
      "$ref": "#/$defs/AgentId",
      "description": "Agent ID of message sender"
    },
    "timestamp": {
      "type": "string",
      "format": "date-time",
      "description": "ISO 8601 timestamp of message creation"
    },
    "routing": {
      "$ref": "#/$defs/RoutingInfo",
      "description": "Message routing configuration"
    },
    "correlation_id": {
      "type": "string",
      "format": "uuid",
      "description": "Optional correlation ID for request/response tracking"
    },
    "ttl": {
      "type": "integer",
      "minimum": 0,
      "description": "Time-to-live in seconds (0 = no expiration)",
      "default": 300
    },
    "priority": {
      "type": "integer",
      "minimum": 0,
      "maximum": 9,
      "description": "Message priority (0=lowest, 9=highest)",
      "default": 5,
      "$comment": "⚠️ CRITICAL INCONSISTENCY [Team Alpha]: Implementation uses 5 priority levels (0-4), 
                  not 10 levels (0-9). This WILL cause runtime array index out of bounds errors. 
                  FIX REQUIRED before production."
    }
  },
  "$defs": {
    "AgentId": {
      "type": "string",
      "pattern": "^agent-[a-zA-Z0-9]{8}-[a-zA-Z0-9]{4}-[a-zA-Z0-9]{4}-[a-zA-Z0-9]{4}-[a-zA-Z0-9]{12}$",
      "description": "Unique agent identifier",
      "$comment": "⚠️ CRITICAL INCONSISTENCY [Team Alpha]: Some components may expect simpler pattern like 
                  '^[a-zA-Z0-9_-]+$'. This WILL cause validation failures between system components. 
                  STANDARDIZATION REQUIRED before implementation."
    },
    "RoutingInfo": {
      "oneOf": [
        {
          "type": "object",
          "properties": {
            "type": { "const": "broadcast" },
            "exclude": {
              "type": "array",
              "items": { "$ref": "#/$defs/AgentId" },
              "description": "Agents to exclude from broadcast"
            }
          },
          "required": ["type"]
        },
        {
          "type": "object",
          "properties": {
            "type": { "const": "target" },
            "target": { "$ref": "#/$defs/AgentId" }
          },
          "required": ["type", "target"]
        },
        {
          "type": "object",
          "properties": {
            "type": { "const": "round_robin" },
            "pool": {
              "type": "array",
              "items": { "$ref": "#/$defs/AgentId" },
              "minItems": 1
            }
          },
          "required": ["type", "pool"]
        }
      ]
    }
  }
}
```

#### 3.4.2 System Messages Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://mister-smith.ai/schemas/system-message",
  "title": "System Control Message",
  "allOf": [
    { "$ref": "https://mister-smith.ai/schemas/base-message" }
  ],
  "properties": {
    "type": {
      "enum": ["START", "STOP", "PAUSE", "RESUME", "HEALTH_CHECK", "RESTART", "SHUTDOWN"]
    },
    "payload": {
      "type": "object",
      "properties": {
        "reason": {
          "type": "string",
          "description": "Human-readable reason for the system command"
        },
        "force": {
          "type": "boolean",
          "description": "Whether to force the action (bypass graceful shutdown)",
          "default": false
        },
        "timeout": {
          "type": "integer",
          "minimum": 0,
          "description": "Timeout in seconds for the operation"
        }
      }
    }
  }
}
```

#### 3.4.3 Task Messages Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://mister-smith.ai/schemas/task-message",
  "title": "Task Assignment and Execution Message",
  "allOf": [
    { "$ref": "https://mister-smith.ai/schemas/base-message" }
  ],
  "properties": {
    "type": {
      "enum": ["TASK_ASSIGN", "TASK_PROGRESS", "TASK_COMPLETE", "TASK_FAILED", "TASK_CANCEL"]
    },
    "payload": {
      "type": "object",
      "properties": {
        "task_id": {
          "type": "string",
          "format": "uuid",
          "description": "Unique task identifier"
        },
        "task_type": {
          "enum": ["RESEARCH", "CODE", "ANALYZE", "REVIEW", "DEPLOY", "MONITOR", "CUSTOM"]
        },
        "description": {
          "type": "string",
          "maxLength": 1000,
          "description": "Human-readable task description"
        },
        "requirements": {
          "type": "object",
          "properties": {
            "capabilities": {
              "type": "array",
              "items": { "type": "string" },
              "description": "Required agent capabilities"
            },
            "resources": {
              "type": "object",
              "properties": {
                "memory_mb": { "type": "integer", "minimum": 0 },
                "cpu_cores": { "type": "number", "minimum": 0 },
                "storage_mb": { "type": "integer", "minimum": 0 }
              }
            },
            "deadline": {
              "type": "string",
              "format": "date-time",
              "description": "Task completion deadline"
            }
          }
        },
        "dependencies": {
          "type": "array",
          "items": {
            "type": "string",
            "format": "uuid"
          },
          "description": "Task IDs that must complete before this task"
        },
        "progress": {
          "type": "number",
          "minimum": 0,
          "maximum": 100,
          "description": "Task completion percentage"
        },
        "result": {
          "type": "object",
          "description": "Task execution result (for completion messages)",
          "additionalProperties": true
        },
        "error": {
          "type": "object",
          "properties": {
            "code": { "type": "string" },
            "message": { "type": "string" },
            "details": { "type": "object" },
            "recoverable": { "type": "boolean" }
          },
          "required": ["code", "message"]
        }
      },
      "required": ["task_id", "task_type"]
    }
  }
}
```

#### 3.4.4 Agent Communication Messages Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://mister-smith.ai/schemas/agent-communication",
  "title": "Inter-Agent Communication Message",
  "allOf": [
    { "$ref": "https://mister-smith.ai/schemas/base-message" }
  ],
  "properties": {
    "type": {
      "enum": ["REQUEST", "RESPONSE", "NOTIFICATION", "COLLABORATION"]
    },
    "payload": {
      "type": "object",
      "properties": {
        "method": {
          "type": "string",
          "description": "Method or action being requested"
        },
        "parameters": {
          "type": "object",
          "description": "Method parameters",
          "additionalProperties": true
        },
        "response_data": {
          "type": "object",
          "description": "Response payload (for RESPONSE type)",
          "additionalProperties": true
        },
        "status_code": {
          "type": "integer",
          "description": "HTTP-style status code for responses"
        },
        "notification_type": {
          "enum": ["INFO", "WARNING", "ERROR", "SUCCESS"],
          "description": "Type of notification"
        },
        "collaboration_mode": {
          "enum": ["PEER_REVIEW", "PAIR_WORK", "KNOWLEDGE_SHARE", "DELEGATION"]
        }
      }
    }
  }
}
```

#### 3.4.5 Supervision Messages Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://mister-smith.ai/schemas/supervision-message",
  "title": "Agent Supervision and Lifecycle Message",
  "allOf": [
    { "$ref": "https://mister-smith.ai/schemas/base-message" }
  ],
  "properties": {
    "type": {
      "enum": ["SPAWN", "TERMINATE", "RESTART", "HEALTH_CHECK", "RESOURCE_UPDATE", "CAPABILITY_CHANGE"]
    },
    "payload": {
      "type": "object",
      "properties": {
        "agent_spec": {
          "type": "object",
          "properties": {
            "agent_type": {
              "enum": ["SUPERVISOR", "WORKER", "COORDINATOR", "MONITOR", "PLANNER", "EXECUTOR", "CRITIC", "ROUTER", "MEMORY"]
            },
            "capabilities": {
              "type": "array",
              "items": { "type": "string" }
            },
            "configuration": {
              "type": "object",
              "additionalProperties": true
            }
          },
          "required": ["agent_type"]
        },
        "health_status": {
          "enum": ["HEALTHY", "DEGRADED", "UNHEALTHY", "UNKNOWN"]
        },
        "metrics": {
          "type": "object",
          "properties": {
            "cpu_usage": { "type": "number", "minimum": 0, "maximum": 100 },
            "memory_usage": { "type": "number", "minimum": 0, "maximum": 100 },
            "active_tasks": { "type": "integer", "minimum": 0 },
            "messages_processed": { "type": "integer", "minimum": 0 },
            "error_rate": { "type": "number", "minimum": 0, "maximum": 1 }
          }
        },
        "restart_reason": {
          "enum": ["TIMEOUT", "RESOURCE_EXHAUSTED", "FATAL_ERROR", "CONFIGURATION_CHANGE", "MANUAL"]
        }
      }
    }
  }
}
```

#### 3.4.6 Hook Integration Messages Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://mister-smith.ai/schemas/hook-message",
  "title": "Claude-CLI Hook Integration Message",
  "allOf": [
    { "$ref": "https://mister-smith.ai/schemas/base-message" }
  ],
  "properties": {
    "type": {
      "enum": ["PRE_TASK", "POST_TASK", "ON_ERROR", "ON_FILE_CHANGE", "STARTUP"]
    },
    "payload": {
      "type": "object",
      "properties": {
        "hook_context": {
          "type": "object",
          "properties": {
            "cwd": { "type": "string" },
            "project_config": { "type": "object" },
            "environment": {
              "type": "object",
              "additionalProperties": { "type": "string" }
            }
          },
          "required": ["cwd"]
        },
        "task_info": {
          "type": "object",
          "properties": {
            "task_id": { "type": "string", "format": "uuid" },
            "description": { "type": "string" },
            "agent_id": { "$ref": "https://mister-smith.ai/schemas/base-message#/$defs/AgentId" }
          }
        },
        "file_changes": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "path": { "type": "string" },
              "change_type": { "enum": ["CREATED", "MODIFIED", "DELETED"] },
              "timestamp": { "type": "string", "format": "date-time" }
            },
            "required": ["path", "change_type"]
          }
        },
        "error_info": {
          "type": "object",
          "properties": {
            "error_type": { "type": "string" },
            "message": { "type": "string" },
            "stack_trace": { "type": "string" },
            "context": { "type": "object" }
          },
          "required": ["error_type", "message"]
        }
      }
    }
  }
}
```

### 3.5 Enhanced Message Validation & Processing

#### 3.5.1 Runtime Message Validation

```rust
STRUCT MessageValidator {
    schemas: HashMap<String, MessageSchema>,
    validation_cache: Arc<RwLock<LruCache<String, ValidationResult>>>,
    custom_rules: Vec<Box<dyn ValidationRule>>,
    metrics: ValidationMetrics
}

TRAIT ValidationRule {
    FUNCTION rule_name(&self) -> &str
    ASYNC FUNCTION validate(&self, message: &Message) -> ValidationResult
    FUNCTION severity(&self) -> ValidationSeverity
}

IMPL MessageValidator {
    #[tracing::instrument(skip(self, message))]
    ASYNC FUNCTION validate_message(&self, message: &Message) -> ValidationResult {
        // Check cache first
        cache_key = format!("{}-{}", message.message_type, message.content_hash())
        IF LET Some(cached_result) = self.validation_cache.read().await.get(&cache_key) {
            RETURN cached_result.clone()
        }
        
        // Get schema for message type
        schema = self.schemas.get(&message.message_type)
            .ok_or(ValidationError::UnknownMessageType(message.message_type.clone()))?
        
        validation_errors = Vec::new()
        
        // Validate required fields
        FOR field IN &schema.required_fields {
            IF !message.has_field(&field.name) {
                validation_errors.push(ValidationError::MissingField(field.name.clone()))
            } ELSE {
                field_value = message.get_field(&field.name)
                IF !field.validates(field_value) {
                    validation_errors.push(ValidationError::InvalidField {
                        field: field.name.clone(),
                        expected: field.field_type.clone(),
                        actual: field_value.type_name()
                    })
                }
            }
        }
        
        // Apply custom validation rules
        FOR rule IN &self.custom_rules {
            rule_result = rule.validate(message).await
            IF rule_result.is_invalid() {
                validation_errors.extend(rule_result.errors)
            }
        }
        
        result = IF validation_errors.is_empty() {
            ValidationResult::Valid
        } ELSE {
            ValidationResult::Invalid {
                errors: validation_errors,
                message_id: message.id.clone()
            }
        }
        
        // Cache result
        self.validation_cache.write().await.insert(cache_key, result.clone())
        self.metrics.record_validation(&message.message_type, &result)
        
        result
    }
}

// Built-in validation rules
STRUCT AgentCapabilityRule {
    capability_registry: Arc<CapabilityRegistry>
}

IMPL ValidationRule FOR AgentCapabilityRule {
    ASYNC FUNCTION validate(&self, message: &Message) -> ValidationResult {
        // Validate sender has required capabilities for message type
        required_capabilities = self.get_required_capabilities(&message.message_type)
        agent_capabilities = self.capability_registry.get_agent_capabilities(&message.sender).await?
        
        missing_capabilities = required_capabilities.iter()
            .filter(|cap| !agent_capabilities.contains(cap))
            .cloned()
            .collect::<Vec<_>>()
        
        IF !missing_capabilities.is_empty() {
            ValidationResult::Invalid {
                errors: vec![ValidationError::InsufficientCapabilities {
                    agent_id: message.sender.clone(),
                    required: required_capabilities,
                    missing: missing_capabilities
                }],
                message_id: message.id.clone()
            }
        } ELSE {
            ValidationResult::Valid
        }
    }
}
```

#### 3.5.2 Priority-Based Mailbox with Backpressure

```rust
STRUCT AgentMailbox {
    priority_queues: [VecDeque<Message>; 5], // ⚠️ CRITICAL INCONSISTENCY [Team Alpha]: Only 5 levels but schemas define 0-9 scale
    capacity_per_priority: [usize; 5],       // ⚠️ CRITICAL: Will cause runtime array index out of bounds for priorities 5-9
    // FIX REQUIRED: Either update to [VecDeque<Message>; 10] or standardize schemas to 0-4 scale
    total_capacity: usize,
    current_size: AtomicUsize,
    backpressure_strategy: BackpressureStrategy,
    message_validator: MessageValidator,
    metrics: MailboxMetrics,
    notification_channel: mpsc::Sender<MailboxEvent>
}

ENUM BackpressureStrategy {
    Block,                    // Block sender until space available
    Drop(MessagePriority),    // Drop messages below specified priority
    Overflow(usize),          // Allow temporary overflow up to limit
    Reject,                   // Reject new messages immediately
    SpillToSecondary(String)  // Spill to secondary storage
}

ENUM MailboxEvent {
    MessageEnqueued { message_id: String, priority: MessagePriority },
    MessageDequeued { message_id: String, wait_time: Duration },
    BackpressureActivated { strategy: String, queue_size: usize },
    MessageDropped { message_id: String, reason: String },
    QueueFull { priority: MessagePriority, size: usize }
}

IMPL AgentMailbox {
    #[tracing::instrument(skip(self, message))]
    ASYNC FUNCTION send(&self, message: Message) -> Result<SendResult, MailboxError> {
        // Validate message first
        validation_result = self.message_validator.validate_message(&message).await
        IF !validation_result.is_valid() {
            self.metrics.record_validation_failure(&message.message_type)
            RETURN Err(MailboxError::ValidationFailed(validation_result))
        }
        
        priority_index = message.priority as usize
        current_size = self.current_size.load(Ordering::Acquire)
        
        // Check if message has expired
        IF message.is_expired() {
            self.metrics.record_expired_message(&message.message_type)
            RETURN Err(MailboxError::MessageExpired)
        }
        
        // Handle capacity constraints
        IF current_size >= self.total_capacity {
            RETURN self.handle_capacity_exceeded(message).await
        }
        
        // Check priority-specific capacity
        priority_queue_size = self.priority_queues[priority_index].len()
        IF priority_queue_size >= self.capacity_per_priority[priority_index] {
            RETURN self.handle_priority_queue_full(message, priority_index).await
        }
        
        // Enqueue message
        self.priority_queues[priority_index].push_back(message.clone())
        new_size = self.current_size.fetch_add(1, Ordering::Release) + 1
        
        // Send notification
        mailbox_event = MailboxEvent::MessageEnqueued {
            message_id: message.id.clone(),
            priority: message.priority
        }
        self.notification_channel.try_send(mailbox_event).ok() // Non-blocking
        
        // Record metrics
        self.metrics.record_message_enqueued(&message.message_type, message.priority)
        
        tracing::debug!(
            message_id = %message.id,
            message_type = %message.message_type,
            priority = ?message.priority,
            queue_size = new_size,
            "Message enqueued successfully"
        )
        
        Ok(SendResult::Enqueued { queue_position: self.estimate_queue_position(&message) })
    }
    
    #[tracing::instrument(skip(self))]
    ASYNC FUNCTION receive(&self, timeout: Option<Duration>) -> Result<Message, MailboxError> {
        deadline = timeout.map(|t| Instant::now() + t)
        start_time = Instant::now()
        
        LOOP {
            // Check priority queues in order (highest priority first)
            FOR priority_index IN 0..5 {
                IF LET Some(message) = self.priority_queues[priority_index].pop_front() {
                    self.current_size.fetch_sub(1, Ordering::Release)
                    wait_time = start_time.elapsed()
                    
                    // Send notification
                    mailbox_event = MailboxEvent::MessageDequeued {
                        message_id: message.id.clone(),
                        wait_time
                    }
                    self.notification_channel.try_send(mailbox_event).ok()
                    
                    // Record metrics
                    self.metrics.record_message_dequeued(&message.message_type, message.priority, wait_time)
                    
                    tracing::debug!(
                        message_id = %message.id,
                        message_type = %message.message_type,
                        priority = ?message.priority,
                        wait_time = ?wait_time,
                        "Message dequeued"
                    )
                    
                    RETURN Ok(message)
                }
            }
            
            // No messages available, check timeout
            IF LET Some(deadline) = deadline {
                IF Instant::now() >= deadline {
                    RETURN Err(MailboxError::ReceiveTimeout)
                }
            }
            
            // Wait for new messages with exponential backoff
            wait_duration = Duration::from_millis(1 << min(5, start_time.elapsed().as_millis() / 100))
            tokio::time::sleep(wait_duration).await
        }
    }
    
    ASYNC FUNCTION handle_capacity_exceeded(&self, message: Message) -> Result<SendResult, MailboxError> {
        MATCH &self.backpressure_strategy {
            BackpressureStrategy::Block => {
                // Wait for space with exponential backoff
                backoff = ExponentialBackoff::new(Duration::from_millis(1), Duration::from_secs(1))
                
                WHILE self.current_size.load(Ordering::Acquire) >= self.total_capacity {
                    tokio::time::sleep(backoff.next_delay()).await
                }
                
                self.send(message).await
            },
            BackpressureStrategy::Drop(min_priority) => {
                IF message.priority >= *min_priority {
                    // Try to make space by dropping lower priority messages
                    dropped_count = self.drop_lower_priority_messages(message.priority)
                    IF dropped_count > 0 {
                        self.send(message).await
                    } ELSE {
                        self.metrics.record_message_dropped(&message.message_type, "no_space_after_drop")
                        Err(MailboxError::Dropped("Unable to make space".to_string()))
                    }
                } ELSE {
                    self.metrics.record_message_dropped(&message.message_type, "low_priority")
                    Err(MailboxError::Dropped("Message priority too low".to_string()))
                }
            },
            BackpressureStrategy::Overflow(max_overflow) => {
                current_overflow = self.current_size.load(Ordering::Acquire) - self.total_capacity
                IF current_overflow < *max_overflow {
                    // Allow temporary overflow
                    self.priority_queues[message.priority as usize].push_back(message.clone())
                    self.current_size.fetch_add(1, Ordering::Release)
                    
                    // Schedule cleanup task
                    self.schedule_overflow_cleanup().await
                    
                    Ok(SendResult::Overflowed { overflow_count: current_overflow + 1 })
                } ELSE {
                    Err(MailboxError::OverflowLimitExceeded)
                }
            },
            BackpressureStrategy::Reject => {
                self.metrics.record_message_rejected(&message.message_type)
                Err(MailboxError::Rejected("Mailbox at capacity".to_string()))
            },
            BackpressureStrategy::SpillToSecondary(storage_path) => {
                // Spill to secondary storage
                spill_result = self.spill_to_storage(&message, storage_path).await
                MATCH spill_result {
                    Ok(spill_id) => Ok(SendResult::Spilled { spill_id }),
                    Err(e) => Err(MailboxError::SpillFailed(e))
                }
            }
        }
    }
}

### 3.6 Agent State Machine Management

#### 3.6.1 State Persistence with Event Sourcing

```rust
// Event sourcing for agent state management
STRUCT AgentStateManager {
    event_store: EventStore,
    current_states: Arc<RwLock<HashMap<AgentId, AgentState>>>,
    state_cache: Arc<RwLock<LruCache<AgentId, CachedState>>>,
    snapshot_store: SnapshotStore,
    state_validators: Vec<Box<dyn StateValidator>>
}

TRAIT StateEvent {
    FUNCTION event_type(&self) -> &str
    FUNCTION agent_id(&self) -> &AgentId
    FUNCTION apply_to_state(&self, state: &mut AgentState) -> Result<()>
    FUNCTION timestamp(&self) -> DateTime<Utc>
    FUNCTION version(&self) -> u64
}

ENUM AgentStateEvent {
    AgentSpawned {
        agent_id: AgentId,
        agent_type: String,
        configuration: AgentConfig,
        supervisor_id: Option<AgentId>,
        timestamp: DateTime<Utc>
    },
    StateTransition {
        agent_id: AgentId,
        from_state: AgentLifecycleState,
        to_state: AgentLifecycleState,
        reason: String,
        timestamp: DateTime<Utc>
    },
    TaskAssigned {
        agent_id: AgentId,
        task_id: TaskId,
        task_spec: TaskSpecification,
        timestamp: DateTime<Utc>
    },
    TaskCompleted {
        agent_id: AgentId,
        task_id: TaskId,
        result: TaskResult,
        execution_time: Duration,
        timestamp: DateTime<Utc>
    },
    CapabilityUpdated {
        agent_id: AgentId,
        capability: String,
        operation: CapabilityOperation,
        timestamp: DateTime<Utc>
    },
    ConfigurationChanged {
        agent_id: AgentId,
        config_changes: HashMap<String, Value>,
        timestamp: DateTime<Utc>
    }
}

IMPL AgentStateManager {
    #[tracing::instrument(skip(self, event))]
    ASYNC FUNCTION persist_state_event(&self, event: Box<dyn StateEvent>) -> Result<()> {
        // Validate event
        FOR validator IN &self.state_validators {
            validator.validate_event(&*event)?
        }
        
        // Store event in event store
        event_id = self.event_store.append_event(event.clone()).await?
        
        // Apply event to current state
        current_states = self.current_states.write().await
        state = current_states.entry(event.agent_id().clone())
            .or_insert_with(|| AgentState::new(event.agent_id().clone()))
        
        event.apply_to_state(state)?
        state.last_event_id = Some(event_id)
        state.version += 1
        
        // Update cache
        self.state_cache.write().await.insert(
            event.agent_id().clone(),
            CachedState {
                state: state.clone(),
                cached_at: Instant::now(),
                ttl: Duration::from_secs(300)
            }
        )
        
        // Check if snapshot needed
        IF state.version % SNAPSHOT_INTERVAL == 0 {
            self.create_state_snapshot(event.agent_id().clone()).await?
        }
        
        tracing::info!(
            agent_id = %event.agent_id(),
            event_type = %event.event_type(),
            version = state.version,
            "State event persisted"
        )
        
        Ok(())
    }
    
    #[tracing::instrument(skip(self))]
    ASYNC FUNCTION restore_agent_state(&self, agent_id: &AgentId) -> Result<AgentState> {
        // Check cache first
        IF LET Some(cached) = self.state_cache.read().await.get(agent_id) {
            IF !cached.is_expired() {
                RETURN Ok(cached.state.clone())
            }
        }
        
        // Try to load latest snapshot
        state = IF LET Some(snapshot) = self.snapshot_store.load_latest(agent_id).await? {
            snapshot.state
        } ELSE {
            AgentState::new(agent_id.clone())
        }
        
        // Apply events since snapshot
        last_event_id = state.last_event_id.clone()
        events = self.event_store.load_events_since(agent_id, last_event_id).await?
        
        FOR event IN events {
            event.apply_to_state(&mut state)?
            state.version += 1
        }
        
        // Cache restored state
        self.state_cache.write().await.insert(
            agent_id.clone(),
            CachedState {
                state: state.clone(),
                cached_at: Instant::now(),
                ttl: Duration::from_secs(300)
            }
        )
        
        // Update current states
        self.current_states.write().await.insert(agent_id.clone(), state.clone())
        
        Ok(state)
    }
    
    ASYNC FUNCTION create_state_snapshot(&self, agent_id: AgentId) -> Result<()> {
        current_states = self.current_states.read().await
        IF LET Some(state) = current_states.get(&agent_id) {
            snapshot = StateSnapshot {
                agent_id: agent_id.clone(),
                state: state.clone(),
                created_at: Utc::now(),
                version: state.version
            }
            
            self.snapshot_store.save_snapshot(snapshot).await?
            tracing::info!(agent_id = %agent_id, version = state.version, "State snapshot created")
        }
        
        Ok(())
    }
}
```

#### 3.6.2 State Machine with Supervision Integration

```rust
STRUCT AgentStateMachine {
    current_state: Arc<RwLock<AgentLifecycleState>>,
    allowed_transitions: HashMap<AgentLifecycleState, Vec<AgentLifecycleState>>,
    state_handlers: HashMap<AgentLifecycleState, Box<dyn StateHandler>>,
    transition_guards: HashMap<(AgentLifecycleState, AgentLifecycleState), Box<dyn TransitionGuard>>,
    state_manager: AgentStateManager,
    metrics: StateMachineMetrics
}

ENUM AgentLifecycleState {
    Initializing,
    Idle,
    Busy(TaskId),
    Paused,
    Error(ErrorInfo),
    Restarting,
    Terminating,
    Terminated
}

TRAIT StateHandler {
    ASYNC FUNCTION on_enter(&self, agent_id: &AgentId, previous_state: Option<AgentLifecycleState>) -> Result<()>
    ASYNC FUNCTION on_exit(&self, agent_id: &AgentId, next_state: AgentLifecycleState) -> Result<()>
    ASYNC FUNCTION handle_message(&self, agent_id: &AgentId, message: Message) -> Result<StateHandlerResult>
}

TRAIT TransitionGuard {
    ASYNC FUNCTION can_transition(&self, agent_id: &AgentId, from: &AgentLifecycleState, to: &AgentLifecycleState) -> bool
    FUNCTION guard_name(&self) -> &str
}

ENUM StateHandlerResult {
    Handled,
    TransitionTo(AgentLifecycleState),
    Forward(Message),
    Error(String)
}

IMPL AgentStateMachine {
    #[tracing::instrument(skip(self))]
    ASYNC FUNCTION transition_to(
        &self, 
        agent_id: &AgentId, 
        new_state: AgentLifecycleState,
        reason: String
    ) -> Result<(), StateMachineError> {
        current_state_guard = self.current_state.read().await
        current_state = current_state_guard.clone()
        drop(current_state_guard)
        
        // Check if transition is allowed
        allowed_states = self.allowed_transitions.get(&current_state)
            .ok_or(StateMachineError::InvalidCurrentState(current_state.clone()))?
        
        IF !allowed_states.contains(&new_state) {
            RETURN Err(StateMachineError::TransitionNotAllowed {
                from: current_state,
                to: new_state,
                allowed: allowed_states.clone()
            })
        }
        
        // Check transition guards
        guard_key = (current_state.clone(), new_state.clone())
        IF LET Some(guard) = self.transition_guards.get(&guard_key) {
            can_transition = guard.can_transition(agent_id, &current_state, &new_state).await
            IF !can_transition {
                RETURN Err(StateMachineError::TransitionBlocked {
                    guard: guard.guard_name().to_string(),
                    reason: format!("Transition from {:?} to {:?} blocked", current_state, new_state)
                })
            }
        }
        
        // Execute state exit handler
        IF LET Some(current_handler) = self.state_handlers.get(&current_state) {
            current_handler.on_exit(agent_id, new_state.clone()).await?
        }
        
        // Perform the transition
        {
            current_state_guard = self.current_state.write().await
            *current_state_guard = new_state.clone()
        }
        
        // Persist state change event
        state_event = AgentStateEvent::StateTransition {
            agent_id: agent_id.clone(),
            from_state: current_state.clone(),
            to_state: new_state.clone(),
            reason,
            timestamp: Utc::now()
        }
        
        self.state_manager.persist_state_event(Box::new(state_event)).await?
        
        // Execute state enter handler
        IF LET Some(new_handler) = self.state_handlers.get(&new_state) {
            new_handler.on_enter(agent_id, Some(current_state.clone())).await?
        }
        
        // Record metrics
        self.metrics.record_state_transition(&current_state, &new_state)
        
        tracing::info!(
            agent_id = %agent_id,
            from_state = ?current_state,
            to_state = ?new_state,
            "State transition completed"
        )
        
        Ok(())
    }
    
    #[tracing::instrument(skip(self, message))]
    ASYNC FUNCTION handle_message(&self, agent_id: &AgentId, message: Message) -> Result<()> {
        current_state = self.current_state.read().await.clone()
        
        IF LET Some(handler) = self.state_handlers.get(&current_state) {
            result = handler.handle_message(agent_id, message.clone()).await?
            
            MATCH result {
                StateHandlerResult::Handled => {
                    tracing::debug!(agent_id = %agent_id, "Message handled in current state")
                },
                StateHandlerResult::TransitionTo(new_state) => {
                    self.transition_to(agent_id, new_state, "Message-triggered transition".to_string()).await?
                },
                StateHandlerResult::Forward(forwarded_message) => {
                    // Forward message to supervisor or other agents
                    self.forward_message(agent_id, forwarded_message).await?
                },
                StateHandlerResult::Error(error_msg) => {
                    error_state = AgentLifecycleState::Error(ErrorInfo {
                        error_type: "MessageHandlingError".to_string(),
                        message: error_msg,
                        timestamp: Utc::now(),
                        recoverable: true
                    })
                    self.transition_to(agent_id, error_state, "Message handling error".to_string()).await?
                }
            }
        } ELSE {
            tracing::warn!(
                agent_id = %agent_id,
                state = ?current_state,
                "No handler for current state"
            )
        }
        
        Ok(())
    }
}

// Built-in state handlers
STRUCT IdleStateHandler;

IMPL StateHandler FOR IdleStateHandler {
    ASYNC FUNCTION on_enter(&self, agent_id: &AgentId, _previous_state: Option<AgentLifecycleState>) -> Result<()> {
        tracing::info!(agent_id = %agent_id, "Agent entered idle state")
        Ok(())
    }
    
    ASYNC FUNCTION handle_message(&self, _agent_id: &AgentId, message: Message) -> Result<StateHandlerResult> {
        MATCH message.message_type.as_str() {
            "TASK_ASSIGN" => {
                task_id = message.payload.get("task_id").unwrap().as_str().unwrap()
                Ok(StateHandlerResult::TransitionTo(AgentLifecycleState::Busy(task_id.to_string())))
            },
            "PAUSE" => {
                Ok(StateHandlerResult::TransitionTo(AgentLifecycleState::Paused))
            },
            "TERMINATE" => {
                Ok(StateHandlerResult::TransitionTo(AgentLifecycleState::Terminating))
            },
            _ => Ok(StateHandlerResult::Handled)
        }
    }
}
```

## 4. Task Distribution

### 4.1 Work Queue Pattern

```rust
CLASS TaskDistributor {
    PRIVATE workers: List<WorkerAgent>
    PRIVATE taskQueue: Queue<Task>
    
    FUNCTION distribute(task: Task) {
        // Find available worker
        worker = findAvailableWorker()
        
        IF worker != NULL THEN
            worker.assign(task)
        ELSE
            taskQueue.enqueue(task)
        END IF
    }
    
    FUNCTION findAvailableWorker() -> WorkerAgent? {
        FOR worker IN workers {
            IF worker.isAvailable() THEN
                RETURN worker
            END IF
        }
        RETURN NULL
    }
}
```

### 4.2 Load Balancing

**⚠️ VALIDATION NOTE [Team Alpha]**: Current load balancing is single-node only. For production readiness, requires:

- Multi-node aware load distribution
- Health-based routing decisions
- Failover mechanisms

```rust
CLASS LoadBalancer {
    PRIVATE agents: List<Agent>
    PRIVATE currentIndex: Integer = 0
    
    FUNCTION selectAgent() -> Agent {
        // Round-robin selection
        agent = agents[currentIndex]
        currentIndex = (currentIndex + 1) % agents.size()
        RETURN agent
    }
    
    FUNCTION selectLeastLoaded() -> Agent {
        // Select agent with fewest active tasks
        RETURN agents.minBy(agent => agent.getActiveTaskCount())
    }
}
```

## 5. Coordination Patterns

**⚠️ VALIDATION WARNING [Team Alpha - HIGH PRIORITY]**:

- **Missing Distributed Coordination**: No consensus algorithms for distributed decision making
- **Missing**: Leader election mechanisms and distributed locking patterns
- **Performance Concerns**: Centralized request-response handler scalability issues
- **Impact**: System will not scale beyond single node without these mechanisms

### 5.1 Request-Response Pattern

```rust
CLASS RequestHandler {
    PRIVATE pendingRequests: Map<UUID, ResponseCallback>
    
    FUNCTION sendRequest(target: Agent, request: Request) -> Future<Response> {
        requestId = generateUUID()
        message = RequestMessage(requestId, request)
        
        future = Future<Response>()
        pendingRequests[requestId] = future.callback
        
        target.send(message)
        RETURN future
    }
    
    FUNCTION handleResponse(response: Response) {
        callback = pendingRequests.remove(response.requestId)
        IF callback != NULL THEN
            callback(response)
        END IF
    }
}
```

### 5.2 Publish-Subscribe Pattern

```rust
CLASS EventBus {
    PRIVATE subscribers: Map<String, List<Agent>>
    
    FUNCTION subscribe(topic: String, agent: Agent) {
        IF NOT subscribers.contains(topic) THEN
            subscribers[topic] = []
        END IF
        subscribers[topic].add(agent)
    }
    
    FUNCTION publish(topic: String, event: Event) {
        agents = subscribers.get(topic, [])
        FOR agent IN agents {
            agent.send(EventMessage(topic, event))
        }
    }
}
```

## 6. Agent Discovery

### 6.1 Registry Pattern

```rust
CLASS AgentRegistry {
    PRIVATE agents: Map<String, AgentInfo>
    
    FUNCTION register(agent: Agent) {
        info = AgentInfo{
            id: agent.id,
            type: agent.type,
            capabilities: agent.getCapabilities(),
            address: agent.getAddress()
        }
        agents[agent.id] = info
    }
    
    FUNCTION discover(criteria: SearchCriteria) -> List<AgentInfo> {
        RETURN agents.values()
            .filter(info => criteria.matches(info))
    }
}
```

### 6.2 Health Monitoring

```rust
CLASS HealthMonitor {
    PRIVATE agents: Map<String, HealthStatus>
    PRIVATE checkInterval: Duration
    
    FUNCTION monitorHealth() {
        EVERY checkInterval {
            FOR agent IN agents.keys() {
                status = checkAgentHealth(agent)
                agents[agent] = status
                
                IF status == UNHEALTHY THEN
                    notifySupervisor(agent)
                END IF
            }
        }
    }
    
    FUNCTION checkAgentHealth(agentId: String) -> HealthStatus {
        TRY {
            response = sendHealthCheck(agentId)
            RETURN response.status
        } CATCH (timeout) {
            RETURN UNHEALTHY
        }
    }
}
```

## 7. Simple Workflow Orchestration

### 7.1 Sequential Workflow

```rust
CLASS SequentialWorkflow {
    PRIVATE steps: List<WorkflowStep>
    
    FUNCTION execute(context: WorkflowContext) -> Result {
        FOR step IN steps {
            result = step.execute(context)
            
            IF result.isFailure() THEN
                RETURN result
            END IF
            
            context.updateWith(result.output)
        }
        
        RETURN Success(context)
    }
}
```

### 7.2 Parallel Workflow

```rust
CLASS ParallelWorkflow {
    PRIVATE tasks: List<Task>
    
    FUNCTION execute() -> Result<List<TaskResult>> {
        futures = []
        
        FOR task IN tasks {
            future = async {
                agent = selectAgent(task.requirements)
                RETURN agent.execute(task)
            }
            futures.add(future)
        }
        
        // Wait for all tasks to complete
        results = awaitAll(futures)
        RETURN Success(results)
    }
}
```

## 8. Error Handling

### 8.1 Basic Error Recovery

```rust
CLASS ErrorHandler {
    FUNCTION handleAgentError(agent: Agent, error: Error) {
        SWITCH error.type {
            CASE TIMEOUT:
                restartAgent(agent)
            CASE RESOURCE_EXHAUSTED:
                pauseAgent(agent)
                scheduleRetry(agent, delay: 30.seconds)
            CASE FATAL:
                terminateAgent(agent)
                notifySupervisor(agent, error)
            DEFAULT:
                logError(agent, error)
        }
    }
}
```

### 8.2 Circuit Breaker Pattern

**✅ VALIDATION STRENGTH**: Good circuit breaker implementation.

**⚠️ ENHANCEMENT NEEDED [Team Alpha]**: Consider adding:

- Distributed circuit breaker state synchronization
- Half-open state testing strategies
- Metric-based threshold adjustments

```rust
CLASS CircuitBreaker {
    PRIVATE state: BreakerState = CLOSED
    PRIVATE failureCount: Integer = 0
    PRIVATE threshold: Integer = 5
    PRIVATE timeout: Duration = 60.seconds
    
    FUNCTION call(operation: Function) -> Result {
        IF state == OPEN THEN
            IF timeoutExpired() THEN
                state = HALF_OPEN
            ELSE
                RETURN Failure("Circuit breaker open")
            END IF
        END IF
        
        TRY {
            result = operation()
            IF state == HALF_OPEN THEN
                state = CLOSED
                failureCount = 0
            END IF
            RETURN result
        } CATCH (error) {
            failureCount += 1
            IF failureCount >= threshold THEN
                state = OPEN
                scheduleTimeout()
            END IF
            THROW error
        }
    }
}
```

## 9. Basic Metrics

**⚠️ VALIDATION WARNING [Team Alpha - MEDIUM PRIORITY]**:

- **Missing**: Performance testing specifications and load testing scenarios
- **Missing**: Coordination performance metrics and scalability limits
- **Recommendation**: Add comprehensive benchmarking suite before production deployment

### 9.1 Agent Metrics

```rust
CLASS AgentMetrics {
    PRIVATE messageCount: Counter
    PRIVATE taskCompletionTime: Histogram
    PRIVATE errorRate: Gauge
    
    FUNCTION recordMessage() {
        messageCount.increment()
    }
    
    FUNCTION recordTaskCompletion(duration: Duration) {
        taskCompletionTime.observe(duration)
    }
    
    FUNCTION updateErrorRate(rate: Float) {
        errorRate.set(rate)
    }
}
```

## 10. Spawn and Resource Management Patterns

**⚠️ VALIDATION WARNING [Team Alpha - HIGH PRIORITY]**:

- **Missing**: Default resource limits per agent type
- **Missing**: Agent density specifications (agents per supervisor node)
- **Missing**: Cross-node agent migration strategies
- **Missing**: Auto-scaling policies based on resource utilization
- **Critical**: Without these limits, system is vulnerable to resource exhaustion

### 10.1 Role-Based Spawning

Dynamic team composition based on project requirements:

```rust
enum AgentRole {
    ProductManager { sop: StandardProcedure },
    Architect { design_patterns: Vec<Pattern> },
    Engineer { toolchain: ToolSet },
}

struct RoleSpawner {
    role_registry: HashMap<String, AgentRole>,
    
    async fn spawn_team(&self, project: ProjectSpec) -> Team {
        let mut agents = vec![];
        
        // Spawn based on project needs
        for role in project.required_roles() {
            agents.push(self.spawn_role(role).await);
        }
        
        Team::new(agents, project.coordination_mode())
    }
}

// Anti-pattern: Static role assignment
// ❌ Fixed teams for all projects
// ✅ Dynamic team composition based on task analysis
```

### 10.2 Resource-Bounded Spawning

Prevent uncontrolled agent proliferation:

```rust
// ❌ BAD: Unlimited spawning
async fn handle_task(task: Task) {
    for subtask in task.decompose() {
        spawn_agent(subtask); // No limits!
    }
}

// ✅ GOOD: Resource-bounded spawning
struct SpawnController {
    max_agents: usize,         // ⚠️ Team Alpha: Define per agent type limits
    active: Arc<AtomicUsize>,  // ⚠️ Team Alpha: Track by type, not just total
    
    async fn spawn_bounded(&self, role: AgentRole) -> Result<Agent> {
        if self.active.load(Ordering::SeqCst) >= self.max_agents {
            return Err("Agent limit reached");
        }
        // Spawn with cleanup on drop
        Ok(BoundedAgent::new(role, self.active.clone()))
    }
}
```

### 10.3 Context Management

Prevent memory overflow with windowed context:

```rust
// ❌ BAD: Accumulating unlimited context
struct NaiveAgent {
    context: Vec<Message>, // Grows forever
}

// ✅ GOOD: Windowed context with summarization
struct SmartAgent {
    recent_context: VecDeque<Message>,
    context_summary: Summary,
    max_context_size: usize,

    fn add_context(&mut self, msg: Message) {
        self.recent_context.push_back(msg);
        if self.recent_context.len() > self.max_context_size {
            self.summarize_old_context();
        }
    }
}
```

### 10.4 Claude-CLI Parallel Execution Integration

Support for Claude-CLI's built-in parallel execution capabilities through Task tool coordination:

```rust
// Enhanced Claude-CLI parallel execution pattern
struct ClaudeTaskOutputParser {
    task_regex: Regex,
    nats_client: async_nats::Client,
    metrics: ParallelExecutionMetrics,
}

impl ClaudeTaskOutputParser {
    fn new(nats_client: async_nats::Client) -> Self {
        let task_regex = Regex::new(r"● Task\((?:Patch Agent )?(\d+|[^)]+)\)").unwrap();

        Self {
            task_regex,
            nats_client,
            metrics: ParallelExecutionMetrics::new(),
        }
    }

    // Parse multiple Claude-CLI task output formats
    fn parse_task_output(&self, line: &str) -> Option<TaskInfo> {
        if let Some(caps) = self.task_regex.captures(line) {
            let task_identifier = caps.get(1).unwrap().as_str();

            // Handle both numeric agent IDs and task descriptions
            if let Ok(agent_id) = task_identifier.parse::<u32>() {
                Some(TaskInfo::AgentId(agent_id))
            } else {
                Some(TaskInfo::Description(task_identifier.to_string()))
            }
        } else {
            None
        }
    }

    // Route task output to appropriate NATS subjects
    async fn route_task_output(&self, task_info: TaskInfo, line: &str) -> Result<(), RoutingError> {
        let subject = match task_info {
            TaskInfo::AgentId(id) => format!("agents.{}.output", id),
            TaskInfo::Description(desc) => format!("tasks.{}.output", desc.replace(" ", "_")),
        };

        let event = TaskOutputEvent {
            task_info,
            output_line: line.to_string(),
            timestamp: Utc::now(),
        };

        self.nats_client.publish(subject, serde_json::to_vec(&event)?).await?;
        self.metrics.increment_routed_messages();

        Ok(())
    }
}

enum TaskInfo {
    AgentId(u32),
    Description(String),
}

struct TaskOutputEvent {
    task_info: TaskInfo,
    output_line: String,
    timestamp: DateTime<Utc>,
}
```

**Parallel Coordination Patterns:**

```rust
// Multi-agent coordination using Claude-CLI Task tool
impl AgentOrchestrator {
    async fn coordinate_parallel_claude_tasks(&mut self, tasks: Vec<TaskRequest>) -> Result<Vec<TaskResult>, CoordinationError> {
        // Build parallel task prompt for Claude-CLI
        let parallel_prompt = self.build_parallel_task_prompt(&tasks)?;

        // Spawn Claude-CLI agent with task coordination
        let claude_agent_id = self.spawn_claude_cli_agent(SpawnRequest {
            prompt: parallel_prompt,
            max_concurrent_tasks: tasks.len(),
            coordination_mode: CoordinationMode::Parallel,
        }).await?;

        // Monitor parallel task execution
        let task_results = self.monitor_parallel_execution(claude_agent_id, tasks.len()).await?;

        Ok(task_results)
    }

    fn build_parallel_task_prompt(&self, tasks: &[TaskRequest]) -> Result<String, PromptError> {
        let task_descriptions: Vec<String> = tasks.iter()
            .enumerate()
            .map(|(i, task)| format!("Task {}: {}", i + 1, task.description))
            .collect();

        Ok(format!(
            "Execute these {} tasks in parallel using the Task tool:\n{}",
            tasks.len(),
            task_descriptions.join("\n")
        ))
    }

    async fn monitor_parallel_execution(&self, claude_agent_id: AgentId, expected_tasks: usize) -> Result<Vec<TaskResult>, MonitoringError> {
        let mut task_results = Vec::new();
        let mut completed_tasks = 0;

        // Subscribe to task output subjects
        let mut task_subscriber = self.nats_client.subscribe("tasks.*.output").await?;

        // Monitor until all tasks complete
        while completed_tasks < expected_tasks {
            if let Some(message) = task_subscriber.next().await {
                let event: TaskOutputEvent = serde_json::from_slice(&message.payload)?;

                // Check for task completion indicators
                if self.is_task_complete(&event.output_line) {
                    task_results.push(TaskResult::from_output_event(event));
                    completed_tasks += 1;
                }
            }
        }

        Ok(task_results)
    }
}
```

**Key Integration Points:**

- **Task Tool Integration**: Leverages Claude-CLI's built-in Task tool for parallel execution
- **Output Pattern Recognition**: Handles both `Task(Patch Agent <n>)` and `Task(Description)` formats
- **NATS Subject Routing**: Routes to `agents.{id}.output` and `tasks.{name}.output` subjects
- **Supervision Compatibility**: Integrates with existing Tokio supervision trees
- **Resource Management**: Coordinates with agent pool limits (25-30 concurrent agents)
- **Memory Persistence**: Compatible with existing Postgres/JetStream KV storage

## 11. Tool-Bus Integration Patterns

**⚠️ VALIDATION WARNING [Team Alpha - MEDIUM PRIORITY]**:

- **Integration Coherence**: Tool-bus architecture may conflict with existing system boundaries
- **Missing**: Clear integration specifications with supervision trees
- **Recommendation**: Define explicit tool permissions and resource limits per agent type

### 11.1 Shared Tool Registry

```rust
struct ToolBus {
    tools: Arc<RwLock<HashMap<ToolId, Box<dyn Tool>>>>,
    permissions: HashMap<AgentId, Vec<ToolId>>,
}

trait Tool: Send + Sync {
    async fn execute(&self, params: Value) -> Result<Value>;
    fn schema(&self) -> ToolSchema;
}

// Extension mechanism
impl ToolBus {
    fn register_tool<T: Tool + 'static>(&mut self, id: ToolId, tool: T) {
        self.tools.write().unwrap().insert(id, Box::new(tool));
    }
    
    async fn call(&self, agent_id: AgentId, tool_id: ToolId, params: Value) -> Result<Value> {
        // Permission check
        if !self.has_permission(agent_id, tool_id) {
            return Err("Unauthorized tool access");
        }
        
        let tools = self.tools.read().unwrap();
        tools.get(&tool_id)?.execute(params).await
    }
}
```

### 11.2 Agent-as-Tool Pattern

```rust
struct AgentTool {
    agent: Arc<dyn Agent>,
    interface: ToolInterface,
}

impl Tool for AgentTool {
    async fn execute(&self, params: Value) -> Result<Value> {
        // Convert tool call to agent message
        let msg = Message::from_tool_params(params);
        self.agent.process(msg).await
    }
}

// Allows supervisors to treat sub-agents as tools
impl Supervisor {
    fn register_agent_as_tool(&mut self, agent: Arc<dyn Agent>) {
        let tool = AgentTool::new(agent);
        self.tool_bus.register(tool);
    }
}
```

## 12. Extension and Middleware Patterns

### 12.1 Middleware Pattern

```rust
trait AgentMiddleware: Send + Sync {
    async fn before_process(&self, msg: &Message) -> Result<()>;
    async fn after_process(&self, msg: &Message, result: &Value) -> Result<()>;
}

struct Agent {
    middleware: Vec<Box<dyn AgentMiddleware>>,
    
    async fn process(&self, msg: Message) -> Result<Value> {
        // Before hooks
        for mw in &self.middleware {
            mw.before_process(&msg).await?;
        }
        
        let result = self.core_process(msg).await?;
        
        // After hooks
        for mw in &self.middleware {
            mw.after_process(&msg, &result).await?;
        }
        
        Ok(result)
    }
}
```

### 12.2 Event Emitter Pattern

```rust
enum SystemEvent {
    AgentSpawned(AgentId),
    TaskCompleted(TaskId, Value),
    ToolCalled(AgentId, ToolId),
    Error(AgentId, String),
}

struct EventBus {
    subscribers: HashMap<TypeId, Vec<Box<dyn EventHandler>>>,
    
    fn emit(&self, event: SystemEvent) {
        if let Some(handlers) = self.subscribers.get(&event.type_id()) {
            for handler in handlers {
                handler.handle(event.clone());
            }
        }
    }
}
```

## 13. Claude-CLI Hook System Integration

**✅ VALIDATION STRENGTH**: Well-designed hook system and task output parsing patterns.

**⚠️ VALIDATION NOTE [Team Alpha]**: While Claude-CLI integration is well-documented, ensure SystemCore wiring specifications are completed before implementation.

### 13.1 Hook Shim Pattern

```rust
// Hook system integration with NATS bus
struct HookShim {
    nats_client: async_nats::Client,
    hook_dir: PathBuf,
    agent_id: String,
}

impl HookShim {
    async fn handle_hook(&self, hook_type: HookType, payload: HookPayload) -> Result<HookResponse> {
        // Execute hook script and capture output
        let hook_path = self.hook_dir.join(hook_type.as_str());
        let mut cmd = Command::new(&hook_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Send JSON payload to hook via stdin
        let stdin = cmd.stdin.as_mut().unwrap();
        stdin.write_all(&serde_json::to_vec(&payload)?).await?;

        // Capture hook response
        let output = cmd.wait_with_output().await?;
        let response: HookResponse = serde_json::from_slice(&output.stdout)?;

        // Publish to NATS for orchestrator processing
        let subject = format!("agent.{}.hook_response", self.agent_id);
        self.nats_client.publish(subject, serde_json::to_vec(&response)?).await?;

        Ok(response)
    }

    async fn subscribe_to_hooks(&self) -> Result<()> {
        let subject = format!("agent.{}.pre", self.agent_id);
        let mut subscriber = self.nats_client.subscribe(subject).await?;

        while let Some(msg) = subscriber.next().await {
            let payload: HookPayload = serde_json::from_slice(&msg.payload)?;

            match payload.hook.as_str() {
                "pre_task" => {
                    self.handle_hook(HookType::PreTask, payload).await?;
                }
                "post_task" => {
                    self.handle_hook(HookType::PostTask, payload).await?;
                }
                "on_error" => {
                    self.handle_hook(HookType::OnError, payload).await?;
                }
                _ => {
                    eprintln!("Unknown hook type: {}", payload.hook);
                }
            }
        }

        Ok(())
    }
}

enum HookType {
    Startup,
    PreTask,
    PostTask,
    OnError,
    OnFileChange,
}

impl HookType {
    fn as_str(&self) -> &'static str {
        match self {
            HookType::Startup => "startup",
            HookType::PreTask => "pre_task",
            HookType::PostTask => "post_task",
            HookType::OnError => "on_error",
            HookType::OnFileChange => "on_file_change",
        }
    }
}

#[derive(Serialize, Deserialize)]
struct HookPayload {
    hook: String,
    cwd: String,
    project_config: Value,
    task: Option<TaskInfo>,
}

#[derive(Serialize, Deserialize)]
struct HookResponse {
    task: Option<TaskInfo>,
    env: Option<HashMap<String, String>>,
}
```

### 13.2 Async Hook Listener Pattern

```rust
// Integration with existing agent supervision
impl Supervisor {
    async fn setup_hook_integration(&self) -> Result<()> {
        // Subscribe to hook events from Claude-CLI
        let startup_sub = self.nats_client.subscribe("control.startup").await?;
        let file_change_sub = self.nats_client.subscribe("ctx.*.file_change").await?;

        // Spawn hook processing tasks
        tokio::spawn(self.process_startup_hooks(startup_sub));
        tokio::spawn(self.process_file_change_hooks(file_change_sub));

        Ok(())
    }

    async fn process_startup_hooks(&self, mut subscriber: Subscriber) {
        while let Some(msg) = subscriber.next().await {
            // Record CLI version and capabilities
            let startup_info: StartupInfo = serde_json::from_slice(&msg.payload).unwrap();
            self.record_cli_capabilities(startup_info).await;
        }
    }

    async fn process_file_change_hooks(&self, mut subscriber: Subscriber) {
        while let Some(msg) = subscriber.next().await {
            // Trigger code quality agents
            let file_change: FileChangeEvent = serde_json::from_slice(&msg.payload).unwrap();
            self.trigger_code_quality_agents(file_change).await;
        }
    }
}
```

## 14. Database Schemas

**⚠️ VALIDATION WARNING [Team Alpha - MEDIUM PRIORITY]**:

- **Missing**: Database migration strategies and schema evolution procedures
- **Missing**: Clear versioning strategy for schema changes
- **Note**: Priority inconsistencies exist in tasks (line 2303) and messages (line 2349) tables
- **Recommendation**: Implement schema version tracking and migration tooling before production

### 14.1 Core Tables

#### 14.1.1 Agents Table

```sql
CREATE TABLE agents (
    agent_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_type VARCHAR(20) NOT NULL CHECK (agent_type IN (
        'SUPERVISOR', 'WORKER', 'COORDINATOR', 'MONITOR', 
        'PLANNER', 'EXECUTOR', 'CRITIC', 'ROUTER', 'MEMORY'
    )),
    current_state VARCHAR(20) NOT NULL CHECK (current_state IN (
        'INITIALIZING', 'RUNNING', 'PAUSED', 'STOPPING', 
        'TERMINATED', 'ERROR', 'RESTARTING'
    )),
    previous_state VARCHAR(20),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_heartbeat TIMESTAMPTZ,
    supervisor_id UUID REFERENCES agents(agent_id),
    capabilities JSONB NOT NULL DEFAULT '[]',
    configuration JSONB NOT NULL DEFAULT '{}',
    state_data JSONB NOT NULL DEFAULT '{}',
    metrics JSONB NOT NULL DEFAULT '{}',
    restart_count INTEGER NOT NULL DEFAULT 0,
    error_count INTEGER NOT NULL DEFAULT 0,
    version VARCHAR(50) NOT NULL DEFAULT '1.0.0',
    tags JSONB NOT NULL DEFAULT '[]'
);

-- Indexes for performance
CREATE INDEX idx_agents_type ON agents(agent_type);
CREATE INDEX idx_agents_state ON agents(current_state);
CREATE INDEX idx_agents_supervisor ON agents(supervisor_id);
CREATE INDEX idx_agents_heartbeat ON agents(last_heartbeat);
CREATE INDEX idx_agents_capabilities ON agents USING GIN(capabilities);
CREATE INDEX idx_agents_tags ON agents USING GIN(tags);

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_agents_updated_at BEFORE UPDATE ON agents
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
```

#### 14.1.2 Tasks Table

```sql
CREATE TABLE tasks (
    task_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID REFERENCES agents(agent_id) ON DELETE SET NULL,
    parent_task_id UUID REFERENCES tasks(task_id),
    task_type VARCHAR(20) NOT NULL CHECK (task_type IN (
        'RESEARCH', 'CODE', 'ANALYZE', 'REVIEW', 'DEPLOY', 'MONITOR', 'CUSTOM'
    )),
    status VARCHAR(20) NOT NULL CHECK (status IN (
        'PENDING', 'ASSIGNED', 'IN_PROGRESS', 'COMPLETED', 'FAILED', 'CANCELLED'
    )),
    priority INTEGER NOT NULL DEFAULT 5 CHECK (priority BETWEEN 0 AND 9), -- ⚠️ CRITICAL INCONSISTENCY [Team Alpha]: DB allows 0-9 but implementation only handles 0-4
    -- FIX REQUIRED: Standardize to either 0-4 or 0-9 across all components to prevent runtime errors
    title VARCHAR(200) NOT NULL,
    description TEXT,
    requirements JSONB NOT NULL DEFAULT '{}',
    input_data JSONB NOT NULL DEFAULT '{}',
    output_data JSONB,
    progress_percentage INTEGER DEFAULT 0 CHECK (progress_percentage BETWEEN 0 AND 100),
    error_info JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assigned_at TIMESTAMPTZ,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    deadline TIMESTAMPTZ,
    estimated_duration_seconds INTEGER,
    actual_duration_seconds INTEGER,
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    tags JSONB NOT NULL DEFAULT '[]'
);

-- Indexes
CREATE INDEX idx_tasks_agent ON tasks(agent_id);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_type ON tasks(task_type);
CREATE INDEX idx_tasks_priority ON tasks(priority DESC);
CREATE INDEX idx_tasks_created ON tasks(created_at);
CREATE INDEX idx_tasks_deadline ON tasks(deadline) WHERE deadline IS NOT NULL;
CREATE INDEX idx_tasks_parent ON tasks(parent_task_id);
CREATE INDEX idx_tasks_tags ON tasks USING GIN(tags);

CREATE TRIGGER update_tasks_updated_at BEFORE UPDATE ON tasks
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
```

#### 14.1.3 Messages Table

```sql
CREATE TABLE messages (
    message_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sender_id UUID REFERENCES agents(agent_id),
    receiver_id UUID REFERENCES agents(agent_id),
    message_type VARCHAR(50) NOT NULL,
    correlation_id UUID,
    routing_type VARCHAR(20) NOT NULL CHECK (routing_type IN (
        'BROADCAST', 'TARGET', 'ROUND_ROBIN'
    )),
    priority INTEGER NOT NULL DEFAULT 5 CHECK (priority BETWEEN 0 AND 9), -- ⚠️ CRITICAL INCONSISTENCY [Team Alpha]: DB allows 0-9 but implementation only handles 0-4
    -- FIX REQUIRED: Standardize to either 0-4 or 0-9 across all components to prevent runtime errors
    ttl_seconds INTEGER NOT NULL DEFAULT 300,
    payload JSONB NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'PENDING' CHECK (status IN (
        'PENDING', 'DELIVERED', 'PROCESSED', 'FAILED', 'EXPIRED'
    )),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    delivered_at TIMESTAMPTZ,
    processed_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '300 seconds'),
    retry_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT
);

-- Indexes
CREATE INDEX idx_messages_sender ON messages(sender_id);
CREATE INDEX idx_messages_receiver ON messages(receiver_id);
CREATE INDEX idx_messages_type ON messages(message_type);
CREATE INDEX idx_messages_correlation ON messages(correlation_id) WHERE correlation_id IS NOT NULL;
CREATE INDEX idx_messages_status ON messages(status);
CREATE INDEX idx_messages_created ON messages(created_at);
CREATE INDEX idx_messages_expires ON messages(expires_at);
CREATE INDEX idx_messages_priority ON messages(priority DESC, created_at);

-- Partitioning by creation date for large scale
CREATE TABLE messages_y2024m01 PARTITION OF messages
    FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');
-- Add more partitions as needed
```

#### 14.1.4 Supervision Events Table

```sql
CREATE TABLE supervision_events (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    supervisor_id UUID NOT NULL REFERENCES agents(agent_id),
    subject_agent_id UUID REFERENCES agents(agent_id),
    event_type VARCHAR(30) NOT NULL CHECK (event_type IN (
        'AGENT_SPAWNED', 'AGENT_TERMINATED', 'AGENT_RESTARTED', 
        'HEALTH_CHECK_FAILED', 'RESOURCE_LIMIT_EXCEEDED', 'POLICY_VIOLATION'
    )),
    event_data JSONB NOT NULL DEFAULT '{}',
    severity VARCHAR(10) NOT NULL CHECK (severity IN ('LOW', 'MEDIUM', 'HIGH', 'CRITICAL')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    resolution_action VARCHAR(100),
    correlation_id UUID
);

-- Indexes
CREATE INDEX idx_supervision_supervisor ON supervision_events(supervisor_id);
CREATE INDEX idx_supervision_subject ON supervision_events(subject_agent_id);
CREATE INDEX idx_supervision_type ON supervision_events(event_type);
CREATE INDEX idx_supervision_severity ON supervision_events(severity);
CREATE INDEX idx_supervision_created ON supervision_events(created_at);
CREATE INDEX idx_supervision_unresolved ON supervision_events(resolved_at) WHERE resolved_at IS NULL;
```

#### 14.1.5 Agent Metrics Table

```sql
CREATE TABLE agent_metrics (
    metric_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(agent_id) ON DELETE CASCADE,
    metric_type VARCHAR(50) NOT NULL,
    metric_value NUMERIC NOT NULL,
    metric_unit VARCHAR(20),
    tags JSONB NOT NULL DEFAULT '{}',
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_metrics_agent ON agent_metrics(agent_id);
CREATE INDEX idx_metrics_type ON agent_metrics(metric_type);
CREATE INDEX idx_metrics_recorded ON agent_metrics(recorded_at);
CREATE INDEX idx_metrics_agent_type_time ON agent_metrics(agent_id, metric_type, recorded_at);

-- Hypertable for TimescaleDB (if using)
-- SELECT create_hypertable('agent_metrics', 'recorded_at');
```

### 14.2 Audit and Logging Tables

#### 14.2.1 State Transitions Table

```sql
CREATE TABLE state_transitions (
    transition_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES agents(agent_id) ON DELETE CASCADE,
    from_state VARCHAR(20) NOT NULL,
    to_state VARCHAR(20) NOT NULL,
    trigger_event VARCHAR(100),
    transition_data JSONB NOT NULL DEFAULT '{}',
    duration_ms INTEGER,
    success BOOLEAN NOT NULL DEFAULT true,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_transitions_agent ON state_transitions(agent_id);
CREATE INDEX idx_transitions_states ON state_transitions(from_state, to_state);
CREATE INDEX idx_transitions_created ON state_transitions(created_at);
```

### 14.3 Views for Common Queries

```sql
-- Active agents view
CREATE VIEW active_agents AS
SELECT 
    agent_id,
    agent_type,
    current_state,
    capabilities,
    last_heartbeat,
    NOW() - last_heartbeat as last_seen_duration
FROM agents 
WHERE current_state IN ('RUNNING', 'PAUSED')
    AND last_heartbeat > NOW() - INTERVAL '5 minutes';

-- Task queue view
CREATE VIEW task_queue AS
SELECT 
    task_id,
    task_type,
    priority,
    title,
    requirements,
    created_at,
    deadline
FROM tasks 
WHERE status = 'PENDING'
ORDER BY priority DESC, created_at ASC;

-- Agent health summary
CREATE VIEW agent_health_summary AS
SELECT 
    a.agent_id,
    a.agent_type,
    a.current_state,
    a.restart_count,
    a.error_count,
    COUNT(t.task_id) as active_tasks,
    AVG(CASE WHEN am.metric_type = 'cpu_usage' THEN am.metric_value END) as avg_cpu_usage,
    AVG(CASE WHEN am.metric_type = 'memory_usage' THEN am.metric_value END) as avg_memory_usage
FROM agents a
LEFT JOIN tasks t ON a.agent_id = t.agent_id AND t.status = 'IN_PROGRESS'
LEFT JOIN agent_metrics am ON a.agent_id = am.agent_id 
    AND am.recorded_at > NOW() - INTERVAL '5 minutes'
GROUP BY a.agent_id, a.agent_type, a.current_state, a.restart_count, a.error_count;
```

## 15. Message Serialization and Communication

**⚠️ VALIDATION WARNING [Team Alpha - CRITICAL for Security]**:

- **CRITICAL GAP**: No authentication mechanisms in serialization layer
- **MISSING**: Encryption specifications for sensitive message payloads
- **MISSING**: Message signing and verification protocols
- **Impact**: System vulnerable to message tampering and unauthorized access
- **Required**: Implement end-to-end encryption and message authentication codes (MAC)

### 15.1 Serialization Formats

#### 15.1.1 Primary Format: JSON

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://mister-smith.ai/schemas/message-envelope",
  "title": "Message Serialization Envelope",
  "type": "object",
  "required": ["version", "encoding", "compression", "message"],
  "properties": {
    "version": {
      "type": "string",
      "pattern": "^\\d+\\.\\d+\\.\\d+$",
      "description": "Serialization format version (semantic versioning)",
      "default": "1.0.0"
    },
    "encoding": {
      "enum": ["json", "msgpack", "protobuf"],
      "description": "Message encoding format",
      "default": "json"
    },
    "compression": {
      "enum": ["none", "gzip", "lz4", "zstd"],
      "description": "Compression algorithm applied",
      "default": "none"
    },
    "checksum": {
      "type": "string",
      "pattern": "^[a-f0-9]{64}$",
      "description": "SHA-256 checksum of message content"
    },
    "message": {
      "description": "The actual message content (varies by encoding)"
    },
    "metadata": {
      "type": "object",
      "properties": {
        "size_bytes": { "type": "integer", "minimum": 0 },
        "compression_ratio": { "type": "number", "minimum": 0 },
        "serialization_time_ms": { "type": "number", "minimum": 0 }
      }
    }
  }
}
```

#### 15.1.2 Performance Format: MessagePack Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://mister-smith.ai/schemas/msgpack-config",
  "title": "MessagePack Serialization Configuration",
  "type": "object",
  "properties": {
    "use_bin_type": {
      "type": "boolean",
      "default": true,
      "description": "Use bin format for binary data"
    },
    "raw": {
      "type": "boolean",
      "default": false,
      "description": "Use raw format (deprecated but faster)"
    },
    "strict_map_key": {
      "type": "boolean",
      "default": true,
      "description": "Enforce string keys in maps"
    },
    "use_single_float": {
      "type": "boolean",
      "default": false,
      "description": "Use 32-bit floats when possible"
    },
    "autoreset": {
      "type": "boolean",
      "default": true,
      "description": "Reset buffer automatically"
    },
    "max_buffer_size": {
      "type": "integer",
      "minimum": 1024,
      "maximum": 104857600,
      "default": 1048576,
      "description": "Maximum buffer size in bytes (1MB default)"
    }
  }
}
```

### 15.2 Error Handling Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://mister-smith.ai/schemas/serialization-error",
  "title": "Serialization Error Response",
  "type": "object",
  "required": ["error_type", "error_code", "message", "timestamp"],
  "properties": {
    "error_type": {
      "enum": [
        "SERIALIZATION_FAILED",
        "DESERIALIZATION_FAILED", 
        "SCHEMA_VALIDATION_FAILED",
        "COMPRESSION_FAILED",
        "CHECKSUM_MISMATCH",
        "VERSION_INCOMPATIBLE",
        "SIZE_LIMIT_EXCEEDED"
      ]
    },
    "error_code": {
      "type": "string",
      "pattern": "^[A-Z0-9_]+$",
      "description": "Machine-readable error code"
    },
    "message": {
      "type": "string",
      "maxLength": 500,
      "description": "Human-readable error message"
    },
    "timestamp": {
      "type": "string",
      "format": "date-time"
    },
    "context": {
      "type": "object",
      "properties": {
        "input_size_bytes": { "type": "integer" },
        "expected_schema": { "type": "string" },
        "actual_format": { "type": "string" },
        "validation_errors": {
          "type": "array",
          "items": { "type": "string" }
        }
      }
    },
    "recovery_suggestions": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "action": { "type": "string" },
          "description": { "type": "string" }
        }
      }
    }
  }
}
```

### 15.3 Version Compatibility Rules

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://mister-smith.ai/schemas/version-compatibility",
  "title": "Message Format Version Compatibility Matrix",
  "type": "object",
  "properties": {
    "current_version": {
      "type": "string",
      "pattern": "^\\d+\\.\\d+\\.\\d+$"
    },
    "compatibility_matrix": {
      "type": "object",
      "patternProperties": {
        "^\\d+\\.\\d+\\.\\d+$": {
          "type": "object",
          "properties": {
            "read_compatible": { "type": "boolean" },
            "write_compatible": { "type": "boolean" },
            "requires_migration": { "type": "boolean" },
            "migration_strategy": {
              "enum": ["AUTOMATIC", "MANUAL", "UNSUPPORTED"]
            },
            "breaking_changes": {
              "type": "array",
              "items": { "type": "string" }
            }
          }
        }
      }
    },
    "migration_rules": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "from_version": { "type": "string" },
          "to_version": { "type": "string" },
          "transformation": { "type": "string" },
          "data_loss_risk": { "enum": ["NONE", "LOW", "MEDIUM", "HIGH"] }
        }
      }
    }
  }
}
```

## Implementation Requirements

### Team Alpha Critical Issues Resolution Checklist

**⚠️ BLOCKER ISSUES - MUST FIX BEFORE ANY IMPLEMENTATION**:

#### CRITICAL Severity (Immediate Action Required)

1. **Schema Validation Inconsistencies** ❌
   - [ ] Standardize AgentId patterns across all components
   - [ ] Fix message priority scale mismatch (choose 0-4 or 0-9)
   - [ ] Implement runtime schema validation with proper error handling
   - **Estimated Fix Time**: 1 week

2. **Missing Security Integration** ❌  
   - [ ] Add authentication mechanisms to message schemas
   - [ ] Define encryption specifications for sensitive payloads
   - [ ] Integrate mTLS with message validation
   - **Estimated Fix Time**: 2 weeks

3. **Horizontal Scaling Architecture Gap** ❌
   - [ ] Design multi-node coordination mechanisms
   - [ ] Implement distributed supervision tree coordination
   - [ ] Add cluster-aware agent distribution
   - **Estimated Fix Time**: 3-4 weeks

4. **Component Integration Specifications** ❌
   - [ ] Complete SystemCore wiring specifications
   - [ ] Define EventBus integration patterns
   - [ ] Document ResourceManager coordination protocols
   - **Estimated Fix Time**: 1-2 weeks

#### HIGH Severity (Required Before Production)

1. **Pattern Selection Guidance** ⚠️
   - [ ] Create decision matrix for orchestration pattern selection
   - [ ] Document performance trade-offs between patterns
   - [ ] Add selection criteria flowchart

2. **Resource Limit Specifications** ⚠️
   - [ ] Define agent type resource bounds
   - [ ] Set memory/CPU limits per agent class
   - [ ] Implement resource scaling policies

3. **Distributed Coordination Mechanisms** ⚠️
   - [ ] Implement consensus algorithms
   - [ ] Add leader election patterns
   - [ ] Create distributed locking mechanisms

4. **Error Handling Completeness** ⚠️
   - [ ] Define retry policies for each message type
   - [ ] Implement dead letter queue specifications
   - [ ] Document error propagation across supervision boundaries

5. **Performance Bottleneck Mitigation** ⚠️
   - [ ] Address central bus scaling limitations
   - [ ] Implement message batching strategies
   - [ ] Add horizontal scaling patterns

6. **Security Protocol Integration** ⚠️
   - [ ] Complete mTLS specifications
   - [ ] Add authentication mechanism details
   - [ ] Define authorization patterns

**ORIGINAL CRITICAL FIXES NEEDED BEFORE PRODUCTION**:

1. **Standardize Priority Scales**:
   - Option A: Update all schemas to use 0-4 scale (5 levels)
   - Option B: Update implementation to handle full 0-9 scale (10 levels)
   - Current state will cause array index out of bounds errors

2. **Standardize AgentId Format**:
   - Define single canonical AgentId pattern
   - Update all schemas and validation to use same pattern
   - Consider backward compatibility requirements

3. **Schema Validation**:
   - Implement runtime schema validation
   - Add migration path for existing data
   - Create comprehensive test suite for message validation

4. **Complete Implementation**:
   - Convert remaining pseudocode to Rust
   - Implement missing error handling
   - Add integration tests for cross-component communication

**Estimated Time to Fix**: 2-3 weeks for schema standardization and validation implementation

## Team Alpha Implementation Readiness Assessment

### Component Readiness Scores

| Component | Score | Status | Action Required |
|-----------|-------|--------|----------------|
| Agent Creation & Initialization | 72% | ⚠️ PARTIAL | Add resource specifications |
| Orchestration Patterns | 48% | ❌ BLOCKED | Need selection framework |
| Communication Protocols | 60% | ⚠️ PARTIAL | Security integration required |
| Coordination Mechanisms | 40% | ❌ BLOCKED | Distributed patterns missing |
| Scalability & Performance | 32% | ❌ CRITICAL | Horizontal scaling required |
| **Overall Architecture** | **47%** | **❌ NOT READY** | Major revision required |

### Ready for Implementation ✅

- Agent type definitions and role specifications
- Database schema design and indexing (with fixes)
- Basic message validation frameworks
- Claude-CLI integration patterns
- State machine definitions

### Blocked for Implementation ❌

- Schema consistency issues
- Security protocol gaps
- Horizontal scaling architecture
- Component integration specifications
- Distributed coordination mechanisms

### Recommendation

**DO NOT PROCEED** with implementation until critical issues are resolved. The technical depth is impressive, but architectural coherence and consistency issues will cause production failures.

## Summary

This document provides comprehensive agent orchestration patterns including:

1. **Agent architecture** - Core agent types (Planner, Executor, Critic, Router, Memory) with Rust trait definitions
2. **Supervision patterns** - Hub-and-spoke, event-driven message bus, and hierarchical supervision trees
3. **Communication patterns** - Direct RPC, publish-subscribe, blackboard, and mailbox patterns
4. **Task distribution** - Work queues and load balancing
5. **Coordination patterns** - Request-response and publish-subscribe
6. **Agent discovery** - Registry and health monitoring
7. **Workflow orchestration** - Sequential and parallel execution
8. **Error handling** - Recovery strategies and circuit breakers
9. **Basic metrics** - Simple performance monitoring
10. **Spawn patterns** - Role-based spawning with resource bounds
11. **Tool-bus integration** - Shared tool registry and agent-as-tool patterns
12. **Extension mechanisms** - Middleware and event emitter patterns

### Key Design Principles

1. **Hierarchical supervision** - Use hierarchical supervisors with bounded spawning
2. **Event-driven architecture** - Async channels with routing strategies  
3. **Shared tool registry** - Tools with permission management
4. **Context management** - Windowed memory with periodic summarization
5. **Extension points** - Middleware, event emitters, and trait-based plugins
6. **Resource control** - Always bound agent spawning and set timeouts
7. **Error boundaries** - Isolate agent failures from system crashes

These patterns provide a solid foundation for building distributed agent systems while avoiding common anti-patterns and maintaining type safety and performance.
