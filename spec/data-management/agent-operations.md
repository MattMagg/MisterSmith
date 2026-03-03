---
title: agent-operations
type: note
permalink: revision-swarm/data-management/agent-operations
tags:
- '#agent-orchestration #operations #discovery #workflow #error-handling #metrics'
---

## Agent Operations Architecture

## Discovery, Workflow, Error Handling & Metrics

> **📋 TECHNICAL SPECIFICATION: AGENT OPERATIONS**
>
> This document provides technical specifications for agent discovery, workflow orchestration,
> error handling, and metrics collection in distributed agent systems using Rust and Tokio.
> **Cross-References**:
>
> - See `agent-lifecycle.md` for basic agent architecture and supervision patterns (sections 1-3)
> - See `agent-communication.md` for message passing and coordination patterns (sections 4-5)
> - See `agent-integration.md` for resource management and integration patterns (sections 10-15)
> - See `../../internal-operations/framework-dev-docs/tech-framework.md` for authoritative technology stack specifications
>
> **Navigation**:
>
> - **Previous**: [Agent Communication](./agent-communication.md) - Message passing and task distribution
> - **Next**: [Agent Integration](./agent-integration.md) - Resource management and extensions
> - **Related**: [Agent Lifecycle](./agent-lifecycle.md) - Foundation agent patterns

## Technical Overview

This document defines operational patterns for distributed agent systems using async Rust and Tokio.
It covers agent discovery mechanisms, workflow orchestration patterns, error handling strategies,
and metrics collection. These patterns integrate with the supervision trees and message passing
from [Agent Orchestration](./agent-orchestration.md) to provide a complete operational framework.

## 6. Agent Discovery

> **Prerequisites**: This section assumes familiarity with basic agent types from [Agent Lifecycle](./agent-lifecycle.md) and message passing from [Agent Communication](./agent-communication.md).
>
> **Next Steps**: Discovery patterns here feed into the resource management and tool-bus integration described in [Agent Integration](./agent-integration.md) sections 10-11.

### 6.1 Registry Pattern

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: AgentId,
    pub agent_type: AgentType,
    pub capabilities: Vec<String>,
    pub address: String,
    pub health_status: HealthStatus,
}

#[derive(Clone)]
pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<AgentId, AgentInfo>>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn register(&self, agent: &Agent) -> Result<(), RegistryError> {
        let info = AgentInfo {
            id: agent.id.clone(),
            agent_type: agent.agent_type.clone(),
            capabilities: agent.get_capabilities().await?,
            address: agent.get_address(),
            health_status: HealthStatus::Healthy,
        };
        
        let mut agents = self.agents.write().await;
        agents.insert(agent.id.clone(), info);
        Ok(())
    }
    
    pub async fn discover(&self, criteria: &SearchCriteria) -> Result<Vec<AgentInfo>, RegistryError> {
        let agents = self.agents.read().await;
        Ok(agents.values()
            .filter(|info| criteria.matches(info))
            .cloned()
            .collect())
    }
}
```

### 6.2 Health Monitoring

```rust
use tokio::time::{interval, timeout, Duration};
use tokio::sync::mpsc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

pub struct HealthMonitor {
    agents: Arc<RwLock<HashMap<AgentId, HealthStatus>>>,
    check_interval: Duration,
    supervisor_tx: mpsc::UnboundedSender<SupervisionEvent>,
}

impl HealthMonitor {
    pub fn new(check_interval: Duration, supervisor_tx: mpsc::UnboundedSender<SupervisionEvent>) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            check_interval,
            supervisor_tx,
        }
    }
    
    pub async fn start_monitoring(&self) {
        let mut interval = interval(self.check_interval);
        
        loop {
            interval.tick().await;
            self.check_all_agents().await;
        }
    }
    
    async fn check_all_agents(&self) {
        let agent_ids: Vec<AgentId> = {
            let agents = self.agents.read().await;
            agents.keys().cloned().collect()
        };
        
        for agent_id in agent_ids {
            let status = self.check_agent_health(&agent_id).await;
            
            {
                let mut agents = self.agents.write().await;
                agents.insert(agent_id.clone(), status.clone());
            }
            
            if matches!(status, HealthStatus::Unhealthy) {
                let _ = self.supervisor_tx.send(SupervisionEvent::AgentUnhealthy(agent_id));
            }
        }
    }
    
    async fn check_agent_health(&self, agent_id: &AgentId) -> HealthStatus {
        match timeout(Duration::from_secs(5), self.send_health_check(agent_id)).await {
            Ok(Ok(_)) => HealthStatus::Healthy,
            Ok(Err(_)) | Err(_) => HealthStatus::Unhealthy,
        }
    }
    
    async fn send_health_check(&self, agent_id: &AgentId) -> Result<(), HealthCheckError> {
        // Implementation depends on transport layer
        // See agent-orchestration.md for message passing patterns
        todo!("Integrate with message transport from agent-orchestration.md")
    }
}
```

## 7. Simple Workflow Orchestration

### 7.1 Sequential Workflow

```rust
use tokio::sync::mpsc;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct WorkflowContext {
    pub data: serde_json::Value,
    pub agent_id: AgentId,
    pub workflow_id: String,
}

#[async_trait::async_trait]
pub trait WorkflowStep: Send + Sync {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<StepOutput, WorkflowError>;
}

pub struct SequentialWorkflow {
    steps: Vec<Arc<dyn WorkflowStep>>,
}

impl SequentialWorkflow {
    pub fn new(steps: Vec<Arc<dyn WorkflowStep>>) -> Self {
        Self { steps }
    }
    
    pub async fn execute(&self, mut context: WorkflowContext) -> Result<WorkflowContext, WorkflowError> {
        for (step_index, step) in self.steps.iter().enumerate() {
            match step.execute(&mut context).await {
                Ok(output) => {
                    // Update context with step output
                    context.data = output.merge_into(context.data)?;
                },
                Err(error) => {
                    return Err(WorkflowError::StepFailed {
                        step_index,
                        source: Box::new(error),
                    });
                }
            }
        }
        
        Ok(context)
    }
}
```

### 7.2 Parallel Workflow

```rust
use futures::future::join_all;
use tokio::task::JoinHandle;

pub struct ParallelWorkflow {
    tasks: Vec<Task>,
    agent_registry: Arc<AgentRegistry>,
}

impl ParallelWorkflow {
    pub fn new(tasks: Vec<Task>, agent_registry: Arc<AgentRegistry>) -> Self {
        Self { tasks, agent_registry }
    }
    
    pub async fn execute(&self) -> Result<Vec<TaskResult>, WorkflowError> {
        let mut handles: Vec<JoinHandle<Result<TaskResult, TaskError>>> = Vec::new();
        
        for task in &self.tasks {
            let task = task.clone();
            let registry = Arc::clone(&self.agent_registry);
            
            let handle = tokio::spawn(async move {
                // Select agent based on task requirements
                let agent = Self::select_agent(&registry, &task.requirements).await?;
                
                // Execute task on selected agent
                agent.execute(task).await
            });
            
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        let results = join_all(handles).await;
        
        // Collect results and handle any join errors
        let mut task_results = Vec::new();
        for result in results {
            match result {
                Ok(Ok(task_result)) => task_results.push(task_result),
                Ok(Err(task_error)) => return Err(WorkflowError::TaskFailed(task_error)),
                Err(join_error) => return Err(WorkflowError::JoinError(join_error)),
            }
        }
        
        Ok(task_results)
    }
    
    async fn select_agent(
        registry: &AgentRegistry,
        requirements: &TaskRequirements,
    ) -> Result<Arc<Agent>, AgentSelectionError> {
        let criteria = SearchCriteria::from_requirements(requirements);
        let available_agents = registry.discover(&criteria).await?;
        
        // Select least loaded agent with required capabilities
        available_agents
            .into_iter()
            .min_by_key(|agent| agent.current_load)
            .map(|info| Arc::new(Agent::from_info(info)))
            .ok_or(AgentSelectionError::NoAvailableAgents)
    }
}
```

## 8. Error Handling

### 8.1 Basic Error Recovery

```rust
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc;
use tracing::{error, warn, info};

#[derive(Debug, Clone)]
pub enum AgentErrorType {
    Timeout,
    ResourceExhausted,
    Fatal,
    Recoverable,
}

#[derive(Debug)]
pub struct AgentError {
    pub error_type: AgentErrorType,
    pub agent_id: AgentId,
    pub message: String,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

pub struct ErrorHandler {
    supervisor_tx: mpsc::UnboundedSender<SupervisionEvent>,
    restart_strategy: RestartStrategy,
}

impl ErrorHandler {
    pub fn new(
        supervisor_tx: mpsc::UnboundedSender<SupervisionEvent>,
        restart_strategy: RestartStrategy,
    ) -> Self {
        Self {
            supervisor_tx,
            restart_strategy,
        }
    }
    
    pub async fn handle_agent_error(&self, agent: &Agent, error: AgentError) -> Result<(), HandlerError> {
        match error.error_type {
            AgentErrorType::Timeout => {
                info!(agent_id = %error.agent_id, "Restarting agent due to timeout");
                self.restart_agent(agent).await?
            },
            AgentErrorType::ResourceExhausted => {
                warn!(agent_id = %error.agent_id, "Pausing agent due to resource exhaustion");
                self.pause_agent(agent).await?;
                self.schedule_retry(agent, Duration::from_secs(30)).await?;
            },
            AgentErrorType::Fatal => {
                error!(agent_id = %error.agent_id, error = %error.message, "Fatal agent error");
                self.terminate_agent(agent).await?;
                self.notify_supervisor(agent, error).await?;
            },
            AgentErrorType::Recoverable => {
                warn!(agent_id = %error.agent_id, error = %error.message, "Recoverable agent error");
            }
        }
        Ok(())
    }
    
    async fn restart_agent(&self, agent: &Agent) -> Result<(), HandlerError> {
        let restart_event = SupervisionEvent::RestartAgent {
            agent_id: agent.id.clone(),
            strategy: self.restart_strategy.clone(),
        };
        
        self.supervisor_tx.send(restart_event)
            .map_err(|_| HandlerError::SupervisorNotAvailable)
    }
    
    async fn pause_agent(&self, agent: &Agent) -> Result<(), HandlerError> {
        let pause_event = SupervisionEvent::PauseAgent(agent.id.clone());
        self.supervisor_tx.send(pause_event)
            .map_err(|_| HandlerError::SupervisorNotAvailable)
    }
    
    async fn schedule_retry(&self, agent: &Agent, delay: Duration) -> Result<(), HandlerError> {
        let agent_id = agent.id.clone();
        let supervisor_tx = self.supervisor_tx.clone();
        
        tokio::spawn(async move {
            sleep(delay).await;
            let _ = supervisor_tx.send(SupervisionEvent::ResumeAgent(agent_id));
        });
        
        Ok(())
    }
    
    async fn terminate_agent(&self, agent: &Agent) -> Result<(), HandlerError> {
        let terminate_event = SupervisionEvent::TerminateAgent(agent.id.clone());
        self.supervisor_tx.send(terminate_event)
            .map_err(|_| HandlerError::SupervisorNotAvailable)
    }
    
    async fn notify_supervisor(&self, agent: &Agent, error: AgentError) -> Result<(), HandlerError> {
        let notification = SupervisionEvent::FatalError {
            agent_id: agent.id.clone(),
            error: error.message,
        };
        
        self.supervisor_tx.send(notification)
            .map_err(|_| HandlerError::SupervisorNotAvailable)
    }
}
```

### 8.2 Circuit Breaker Pattern

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone, PartialEq)]
pub enum BreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug)]
pub enum CircuitBreakerError {
    Open,
    OperationFailed(Box<dyn std::error::Error + Send + Sync>),
}

pub struct CircuitBreaker {
    state: Arc<RwLock<BreakerState>>,
    failure_count: Arc<RwLock<u32>>,
    threshold: u32,
    timeout: Duration,
    last_failure: Arc<RwLock<Option<Instant>>>,
}

impl CircuitBreaker {
    pub fn new(threshold: u32, timeout: Duration) -> Self {
        Self {
            state: Arc::new(RwLock::new(BreakerState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            threshold,
            timeout,
            last_failure: Arc::new(RwLock::new(None)),
        }
    }
    
    pub async fn call<F, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError>
    where
        F: Future<Output = Result<T, E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        // Check if we should transition from OPEN to HALF_OPEN
        self.check_timeout().await;
        
        let current_state = {
            let state = self.state.read().await;
            state.clone()
        };
        
        match current_state {
            BreakerState::Open => {
                Err(CircuitBreakerError::Open)
            },
            BreakerState::Closed | BreakerState::HalfOpen => {
                match operation.await {
                    Ok(result) => {
                        // Reset on success, especially important for HALF_OPEN
                        if current_state == BreakerState::HalfOpen {
                            self.reset().await;
                        }
                        Ok(result)
                    },
                    Err(error) => {
                        self.record_failure().await;
                        Err(CircuitBreakerError::OperationFailed(Box::new(error)))
                    }
                }
            }
        }
    }
    
    async fn check_timeout(&self) {
        let should_transition = {
            let state = self.state.read().await;
            if *state != BreakerState::Open {
                return;
            }
            
            let last_failure = self.last_failure.read().await;
            if let Some(failure_time) = *last_failure {
                failure_time.elapsed() >= self.timeout
            } else {
                false
            }
        };
        
        if should_transition {
            let mut state = self.state.write().await;
            *state = BreakerState::HalfOpen;
        }
    }
    
    async fn record_failure(&self) {
        let mut failure_count = self.failure_count.write().await;
        *failure_count += 1;
        
        if *failure_count >= self.threshold {
            let mut state = self.state.write().await;
            *state = BreakerState::Open;
            
            let mut last_failure = self.last_failure.write().await;
            *last_failure = Some(Instant::now());
        }
    }
    
    async fn reset(&self) {
        let mut state = self.state.write().await;
        *state = BreakerState::Closed;
        
        let mut failure_count = self.failure_count.write().await;
        *failure_count = 0;
        
        let mut last_failure = self.last_failure.write().await;
        *last_failure = None;
    }
}
```

## 9. Basic Metrics

### 9.1 Agent Metrics

```rust
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use prometheus::{Counter, Histogram, Gauge, Registry, Opts, HistogramOpts};

#[derive(Clone)]
pub struct AgentMetrics {
    message_count: Counter,
    task_completion_time: Histogram,
    error_rate: Gauge,
    active_agents: Gauge,
    registry: Arc<Registry>,
}

impl AgentMetrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Arc::new(Registry::new());
        
        let message_count = Counter::with_opts(Opts::new(
            "agent_messages_total",
            "Total number of messages processed by agents"
        ))?;
        
        let task_completion_time = Histogram::with_opts(HistogramOpts::new(
            "agent_task_duration_seconds",
            "Time taken to complete agent tasks"
        ))?;
        
        let error_rate = Gauge::with_opts(Opts::new(
            "agent_error_rate",
            "Current error rate for agent operations"
        ))?;
        
        let active_agents = Gauge::with_opts(Opts::new(
            "active_agents_total",
            "Number of currently active agents"
        ))?;
        
        // Register metrics
        registry.register(Box::new(message_count.clone()))?;
        registry.register(Box::new(task_completion_time.clone()))?;
        registry.register(Box::new(error_rate.clone()))?;
        registry.register(Box::new(active_agents.clone()))?;
        
        Ok(Self {
            message_count,
            task_completion_time,
            error_rate,
            active_agents,
            registry,
        })
    }
    
    pub fn record_message(&self) {
        self.message_count.inc();
    }
    
    pub fn record_task_completion(&self, duration: Duration) {
        self.task_completion_time.observe(duration.as_secs_f64());
    }
    
    pub fn update_error_rate(&self, rate: f64) {
        self.error_rate.set(rate);
    }
    
    pub fn set_active_agents(&self, count: i64) {
        self.active_agents.set(count as f64);
    }
    
    pub fn registry(&self) -> Arc<Registry> {
        Arc::clone(&self.registry)
    }
    
    /// Get current metrics as a formatted string
    pub fn gather_metrics(&self) -> String {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode_to_string(&metric_families).unwrap_or_else(|_| "Error encoding metrics".to_string())
    }
}
```

## 10. Advanced Operational Patterns

### 10.1 Load Balancing and Task Distribution

```rust
CLASS LoadBalancer {
    PRIVATE agents: List<Agent>
    PRIVATE strategy: LoadBalancingStrategy
    
    FUNCTION selectAgent(task: Task) -> Agent {
        SWITCH strategy {
            CASE ROUND_ROBIN:
                RETURN agents[currentIndex++ % agents.length]
            CASE LEAST_LOADED:
                RETURN findAgentWithLowestLoad()
            CASE CAPABILITY_BASED:
                candidates = agents.filter(a => a.hasCapability(task.requiredCapability))
                RETURN selectLeastLoaded(candidates)
        }
    }
    
    FUNCTION distributeTask(task: Task) -> Result {
        FOR attempt IN 1..MAX_ATTEMPTS {
            agent = selectAgent(task)
            
            IF agent.isHealthy() AND agent.hasCapacity() THEN
                RETURN agent.execute(task)
            ELSE
                removeFromRotation(agent)
            END IF
        }
        
        RETURN Failure("No available agents")
    }
}
```

### 10.2 Agent Pool Management

```rust
CLASS AgentPool {
    PRIVATE minSize: Integer = 5
    PRIVATE maxSize: Integer = 25
    PRIVATE currentSize: Integer = 0
    PRIVATE activeAgents: Set<Agent>
    PRIVATE idleAgents: Queue<Agent>
    
    FUNCTION acquireAgent() -> Agent {
        IF idleAgents.isEmpty() AND currentSize < maxSize THEN
            spawnNewAgent()
        END IF
        
        IF NOT idleAgents.isEmpty() THEN
            agent = idleAgents.dequeue()
            activeAgents.add(agent)
            RETURN agent
        END IF
        
        THROW "Pool exhausted"
    }
    
    FUNCTION releaseAgent(agent: Agent) {
        activeAgents.remove(agent)
        
        IF agent.isHealthy() THEN
            idleAgents.enqueue(agent)
        ELSE
            terminateAgent(agent)
            currentSize -= 1
        END IF
        
        // Ensure minimum pool size
        WHILE currentSize < minSize {
            spawnNewAgent()
        }
    }
}
```

### 10.3 Deadline and Timeout Management

```rust
CLASS DeadlineManager {
    PRIVATE scheduledTasks: PriorityQueue<ScheduledTask>
    
    FUNCTION scheduleTask(task: Task, deadline: Timestamp) {
        scheduledTask = ScheduledTask{
            task: task,
            deadline: deadline,
            timeoutAction: ESCALATE
        }
        
        scheduledTasks.add(scheduledTask)
        scheduleTimeout(scheduledTask)
    }
    
    FUNCTION handleTimeout(task: ScheduledTask) {
        SWITCH task.timeoutAction {
            CASE RETRY:
                IF task.retryCount < MAX_RETRIES THEN
                    rescheduleWithBackoff(task)
                ELSE
                    markAsFailed(task)
                END IF
            CASE ESCALATE:
                escalateToSupervisor(task)
            CASE FAIL_FAST:
                markAsFailed(task)
        }
    }
}
```

## 11. Observability and Monitoring

### 11.1 Comprehensive Metrics Collection

```rust
CLASS OperationalMetrics {
    PRIVATE counters: Map<String, Counter>
    PRIVATE histograms: Map<String, Histogram>
    PRIVATE gauges: Map<String, Gauge>
    
    FUNCTION recordTaskExecution(duration: Duration, success: Boolean) {
        counters["tasks_total"].increment()
        histograms["task_duration"].observe(duration)
        
        IF success THEN
            counters["tasks_success"].increment()
        ELSE
            counters["tasks_failed"].increment()
        END IF
    }
    
    FUNCTION recordAgentUtilization(agentId: String, utilization: Float) {
        gauges["agent_utilization"].setWithLabels(
            value: utilization,
            labels: {"agent_id": agentId}
        )
    }
    
    FUNCTION generateHealthReport() -> HealthReport {
        RETURN HealthReport{
            totalTasks: counters["tasks_total"].value(),
            successRate: calculateSuccessRate(),
            averageResponseTime: histograms["task_duration"].average(),
            activeAgents: gauges["active_agents"].value(),
            systemLoad: calculateSystemLoad()
        }
    }
}
```

### 11.2 Distributed Tracing

```rust
CLASS DistributedTracing {
    FUNCTION startTrace(operation: String) -> TraceContext {
        traceId = generateTraceId()
        spanId = generateSpanId()
        
        context = TraceContext{
            traceId: traceId,
            spanId: spanId,
            operation: operation,
            startTime: now()
        }
        
        setCurrentContext(context)
        RETURN context
    }
    
    FUNCTION recordSpan(context: TraceContext, event: String, metadata: Map) {
        span = Span{
            traceId: context.traceId,
            parentSpanId: context.spanId,
            operation: event,
            duration: now() - context.startTime,
            metadata: metadata
        }
        
        sendToTraceCollector(span)
    }
}
```

## Implementation Summary

This document provides async Rust implementations for agent operational patterns:

1. **Agent Discovery** - Thread-safe registry using `Arc<RwLock<HashMap>>` with async agent registration and capability-based discovery
2. **Workflow Orchestration** - Sequential and parallel execution using `tokio::spawn` and `futures::join_all` for concurrent task processing
3. **Error Handling** - Supervision integration using `tokio::sync::mpsc` channels with restart strategies and circuit breaker patterns
4. **Metrics Collection** - Prometheus-based metrics with async collection and thread-safe counters
5. **Fault Tolerance** - Circuit breaker pattern with async state management and timeout handling using `tokio::time`

### Async Implementation Patterns

1. **Thread Safety** - All shared state uses `Arc<RwLock<T>>` or `Arc<Mutex<T>>` for concurrent access
2. **Channel Communication** - Uses `tokio::sync::mpsc` for supervision events and agent coordination
3. **Task Spawning** - Leverages `tokio::spawn` for concurrent agent task execution
4. **Timeout Management** - Integrates `tokio::time::timeout` and `tokio::time::interval` for deadline handling
5. **Error Propagation** - Uses `Result<T, E>` types with custom error enums for proper async error handling

### Cross-Reference Integration

- **Agent Orchestration**: Operations patterns integrate with supervision trees from [agent-orchestration.md](./agent-orchestration.md) using shared `SupervisionEvent` types and channel communication
- **Message Transport**: Health checks and agent communication rely on message passing patterns defined in agent-orchestration.md sections on actor communication
- **Resource Management**: Pool management and load balancing connect to agent spawning and lifecycle management from orchestration patterns

### Supervision Tree Integration

Operational patterns coordinate with supervision hierarchies through:

```rust
// Supervision events sent from operations to orchestration layer
pub enum SupervisionEvent {
    AgentUnhealthy(AgentId),
    RestartAgent { agent_id: AgentId, strategy: RestartStrategy },
    PauseAgent(AgentId),
    ResumeAgent(AgentId),
    TerminateAgent(AgentId),
    FatalError { agent_id: AgentId, error: String },
}
```

This enables seamless integration between operational monitoring and supervision decisions.

---

## Technical References

- **[Agent Orchestration](./agent-orchestration.md)**: Supervision trees, actor patterns, and message passing that integrate with these operational patterns
- **[Agent Integration](./agent-integration.md)**: Resource management and tool-bus integration for agent capabilities
- **[Agent Communication](./agent-communication.md)**: Message transport layers used by health monitoring and discovery
- **[Agent Lifecycle](./agent-lifecycle.md)**: Basic agent types and spawning patterns extended by operational workflows
- **[Message Framework](./message-framework.md)**: Schema specifications for health check and discovery messages
- **[Storage Patterns](./storage-patterns.md)**: Persistence patterns for metrics and agent state

### Implementation Dependencies

```toml
# Required Cargo.toml dependencies for operational patterns
[dependencies]
tokio = { version = "1.38", features = ["full"] }
futures = "0.3"
prometheus = "0.13"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
async-trait = "0.1"
```

---

*Agent Operations Architecture - Complete sections 6-9 of agent orchestration framework*
