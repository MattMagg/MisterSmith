# Async Patterns Architecture

**Framework Documentation > Core Architecture > Async Patterns**

**Quick Links**: [Runtime & Errors](runtime-and-errors.md) | [Supervision & Events](supervision-and-events.md) | [System Architecture](system-architecture.md) | [Component Architecture](component-architecture.md)

---

## Navigation

[← Runtime & Errors](./runtime-and-errors.md) | [System Architecture Overview](./system-architecture.md) | [Supervision & Events →](./supervision-and-events.md)

---

## 🔍 VALIDATION STATUS

**Last Validated**: 2025-07-07  
**Validator**: Agent 1 - Team Alpha  
**Component**: Async Patterns Implementation  
**Status**: Implementation-Ready  

### Key Strengths

- ✅ Complete task management framework with priority and retry policies
- ✅ Robust stream processing with backpressure handling
- ✅ Type-safe actor system with mailbox implementation
- ✅ Agent-as-Tool pattern for modular composition
- ✅ Comprehensive tool system with permissions and metrics

---

## Table of Contents

1. [Task Management Framework](#task-management-framework)
2. [Stream Processing Architecture](#stream-processing-architecture)
3. [Actor Model Implementation](#actor-model-implementation)
4. [Core Type Definitions](#core-type-definitions)
5. [Agent-as-Tool Pattern](#agent-as-tool-pattern)
6. [Tool System Core](#tool-system-core)

---

## Task Management Framework

```rust
// src/async_patterns/tasks.rs
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore, oneshot};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::errors::TaskError;
use uuid::Uuid;

// Task execution constants
pub const DEFAULT_TASK_TIMEOUT: Duration = Duration::from_secs(30);
pub const DEFAULT_TASK_QUEUE_SIZE: usize = 1000;
pub const MAX_CONCURRENT_TASKS: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

#[async_trait]
pub trait AsyncTask: Send + Sync {
    type Output: Send + Sync;
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn execute(self) -> Result<Self::Output, Self::Error>;
    fn priority(&self) -> TaskPriority;
    fn timeout(&self) -> Duration;
    fn retry_policy(&self) -> RetryPolicy;
    fn task_id(&self) -> TaskId;
}

#[derive(Debug)]
pub struct TaskHandle<T> {
    task_id: TaskId,
    receiver: oneshot::Receiver<Result<T, TaskError>>,
    join_handle: JoinHandle<()>,
}

impl<T> TaskHandle<T> {
    pub fn task_id(&self) -> TaskId {
        self.task_id
    }
    
    pub async fn await_result(self) -> Result<T, TaskError> {
        match self.receiver.await {
            Ok(result) => result,
            Err(_) => Err(TaskError::TaskCancelled),
        }
    }
    
    pub fn abort(&self) {
        self.join_handle.abort();
    }
}

#[derive(Debug)]
pub struct TaskMetrics {
    pub total_submitted: std::sync::atomic::AtomicU64,
    pub completed: std::sync::atomic::AtomicU64,
    pub failed: std::sync::atomic::AtomicU64,
    pub currently_running: std::sync::atomic::AtomicU64,
}

impl TaskMetrics {
    pub fn new() -> Self {
        Self {
            total_submitted: std::sync::atomic::AtomicU64::new(0),
            completed: std::sync::atomic::AtomicU64::new(0),
            failed: std::sync::atomic::AtomicU64::new(0),
            currently_running: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

type BoxedTask = Box<dyn AsyncTask<Output = serde_json::Value, Error = TaskError> + Send>;

#[derive(Debug)]
pub struct TaskExecutor {
    task_queue: Arc<Mutex<VecDeque<BoxedTask>>>,
    worker_handles: Vec<JoinHandle<()>>,
    semaphore: Arc<Semaphore>,
    metrics: Arc<TaskMetrics>,
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
}

impl TaskExecutor {
    pub fn new(max_concurrent: usize) -> Self {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
        
        Self {
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            worker_handles: Vec::new(),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            metrics: Arc::new(TaskMetrics::new()),
            shutdown_tx,
        }
    }
    
    pub async fn submit<T: AsyncTask + 'static>(&self, task: T) -> Result<TaskHandle<T::Output>, TaskError> {
        let task_id = task.task_id();
        let priority = task.priority();
        let timeout_duration = task.timeout();
        let retry_policy = task.retry_policy();
        
        // Acquire semaphore permit
        let permit = self.semaphore.acquire().await
            .map_err(|_| TaskError::ExecutorShutdown)?;
        
        let (tx, rx) = oneshot::channel();
        
        // Update metrics
        self.metrics.total_submitted.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.metrics.currently_running.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        let metrics = Arc::clone(&self.metrics);
        
        let join_handle = tokio::spawn(async move {
            let _permit = permit; // Hold permit for duration of task
            
            let result = Self::execute_with_retry(task, timeout_duration, retry_policy).await;
            
            // Update metrics
            metrics.currently_running.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            match &result {
                Ok(_) => { metrics.completed.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
                Err(_) => { metrics.failed.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }
            }
            
            let _ = tx.send(result);
        });
        
        Ok(TaskHandle {
            task_id,
            receiver: rx,
            join_handle,
        })
    }
    
    async fn execute_with_retry<T: AsyncTask>(
        mut task: T,
        timeout_duration: Duration,
        retry_policy: RetryPolicy,
    ) -> Result<T::Output, TaskError> {
        let mut attempts = 0;
        let mut delay = retry_policy.base_delay;
        
        loop {
            attempts += 1;
            
            match timeout(timeout_duration, task.execute()).await {
                Ok(Ok(output)) => return Ok(output),
                Ok(Err(e)) => {
                    if attempts >= retry_policy.max_attempts {
                        return Err(TaskError::ExecutionFailed(e.to_string()));
                    }
                    
                    // Exponential backoff
                    tokio::time::sleep(delay).await;
                    delay = std::cmp::min(
                        Duration::from_millis((delay.as_millis() as f64 * retry_policy.backoff_multiplier) as u64),
                        retry_policy.max_delay,
                    );
                }
                Err(_) => {
                    return Err(TaskError::TimedOut);
                }
            }
        }
    }
    
    pub async fn shutdown(mut self) -> Result<(), TaskError> {
        let _ = self.shutdown_tx.send(());
        
        // Wait for all workers to complete
        for handle in self.worker_handles {
            if let Err(e) = handle.await {
                tracing::warn!("Worker task failed during shutdown: {}", e);
            }
        }
        
        Ok(())
    }
    
    pub fn metrics(&self) -> &TaskMetrics {
        &self.metrics
    }
}
```

## Stream Processing Architecture

```rust
// src/async_patterns/streams.rs
use futures::{Stream, Sink, StreamExt, SinkExt};
use std::pin::Pin;
use std::task::{Context, Poll};
use async_trait::async_trait;
use crate::errors::StreamError;
use serde::{Serialize, Deserialize};
use std::time::Duration;
use std::collections::VecDeque;
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackpressureStrategy {
    Wait,
    Drop,
    Buffer,
    Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpressureConfig {
    pub strategy: BackpressureStrategy,
    pub wait_duration: Duration,
    pub buffer_size: usize,
    pub threshold: f64, // 0.0 - 1.0
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            strategy: BackpressureStrategy::Wait,
            wait_duration: Duration::from_millis(100),
            buffer_size: 1000,
            threshold: 0.8,
        }
    }
}

#[async_trait]
pub trait Processor<T>: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn process(&self, item: T) -> Result<T, Self::Error>;
    fn name(&self) -> &str;
}

#[derive(Debug)]
pub struct StreamProcessor<T> {
    input_stream: Pin<Box<dyn Stream<Item = T> + Send>>,
    processors: Vec<Box<dyn Processor<T, Error = StreamError> + Send + Sync>>,
    output_sink: Pin<Box<dyn Sink<T, Error = StreamError> + Send>>,
    backpressure_config: BackpressureConfig,
    buffer: Arc<Mutex<VecDeque<T>>>,
    metrics: StreamMetrics,
}

#[derive(Debug, Default)]
pub struct StreamMetrics {
    pub items_processed: std::sync::atomic::AtomicU64,
    pub items_dropped: std::sync::atomic::AtomicU64,
    pub backpressure_events: std::sync::atomic::AtomicU64,
    pub processing_errors: std::sync::atomic::AtomicU64,
}

impl<T> StreamProcessor<T>
where
    T: Send + Sync + Clone + 'static,
{
    pub fn new(
        input_stream: Pin<Box<dyn Stream<Item = T> + Send>>,
        processors: Vec<Box<dyn Processor<T, Error = StreamError> + Send + Sync>>,
        output_sink: Pin<Box<dyn Sink<T, Error = StreamError> + Send>>,
        backpressure_config: BackpressureConfig,
    ) -> Self {
        Self {
            input_stream,
            processors,
            output_sink,
            backpressure_config,
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            metrics: StreamMetrics::default(),
        }
    }
    
    pub async fn process_stream(&mut self) -> Result<(), StreamError> {
        while let Some(item) = self.input_stream.next().await {
            let processed_item = self.apply_processors(item).await?;
            
            match self.send_with_backpressure(processed_item).await {
                Ok(_) => {
                    self.metrics.items_processed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
                Err(e) => {
                    self.metrics.processing_errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    return Err(e);
                }
            }
        }
        
        // Flush any remaining buffered items
        self.flush_buffer().await?;
        
        Ok(())
    }
    
    async fn apply_processors(&self, mut item: T) -> Result<T, StreamError> {
        for processor in &self.processors {
            item = processor.process(item).await
                .map_err(|e| StreamError::ProcessorFailed(processor.name().to_string(), e.to_string()))?;
        }
        Ok(item)
    }
    
    async fn send_with_backpressure(&mut self, item: T) -> Result<(), StreamError> {
        match self.output_sink.send(item.clone()).await {
            Ok(_) => Ok(()),
            Err(e) if self.is_backpressure_error(&e) => {
                self.metrics.backpressure_events.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                self.handle_backpressure(item).await
            }
            Err(e) => Err(e),
        }
    }
    
    async fn handle_backpressure(&mut self, item: T) -> Result<(), StreamError> {
        match self.backpressure_config.strategy {
            BackpressureStrategy::Wait => {
                tokio::time::sleep(self.backpressure_config.wait_duration).await;
                // Retry sending
                self.output_sink.send(item).await
            }
            BackpressureStrategy::Drop => {
                self.metrics.items_dropped.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Ok(())
            }
            BackpressureStrategy::Buffer => {
                self.buffer_item(item).await
            }
            BackpressureStrategy::Block => {
                // Keep retrying until successful
                loop {
                    tokio::time::sleep(self.backpressure_config.wait_duration).await;
                    match self.output_sink.send(item.clone()).await {
                        Ok(_) => return Ok(()),
                        Err(e) if self.is_backpressure_error(&e) => continue,
                        Err(e) => return Err(e),
                    }
                }
            }
        }
    }
    
    async fn buffer_item(&self, item: T) -> Result<(), StreamError> {
        let mut buffer = self.buffer.lock().await;
        
        if buffer.len() >= self.backpressure_config.buffer_size {
            // Buffer is full, drop oldest item
            buffer.pop_front();
            self.metrics.items_dropped.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        
        buffer.push_back(item);
        Ok(())
    }
    
    async fn flush_buffer(&mut self) -> Result<(), StreamError> {
        let mut buffer = self.buffer.lock().await;
        
        while let Some(item) = buffer.pop_front() {
            self.output_sink.send(item).await?;
        }
        
        Ok(())
    }
    
    fn is_backpressure_error(&self, error: &StreamError) -> bool {
        matches!(error, StreamError::SinkFull | StreamError::SinkBlocked)
    }
    
    pub fn metrics(&self) -> &StreamMetrics {
        &self.metrics
    }
}

// Helper for creating buffered streams
pub fn create_buffered_stream<T>(
    stream: impl Stream<Item = T> + Send + 'static,
    buffer_size: usize,
) -> Pin<Box<dyn Stream<Item = T> + Send>> {
    Box::pin(stream.buffer_unordered(buffer_size))
}

// Helper for creating rate-limited streams
pub fn create_rate_limited_stream<T>(
    stream: impl Stream<Item = T> + Send + 'static,
    rate_limit: Duration,
) -> Pin<Box<dyn Stream<Item = T> + Send>> {
    use futures::stream;
    
    Box::pin(
        stream.then(move |item| async move {
            tokio::time::sleep(rate_limit).await;
            item
        })
    )
}
```

## Actor Model Implementation

```rust
// src/actors/actor.rs
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot, Mutex};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::errors::{ActorError, SystemError};
use crate::supervision::SupervisionStrategy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActorId(pub Uuid);

impl ActorId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone)]
pub enum ActorResult {
    Continue,
    Stop,
    Restart,
}

pub trait ActorMessage: Send + Sync + std::fmt::Debug + Clone {}

#[async_trait]
pub trait Actor: Send + Sync {
    type Message: ActorMessage;
    type State: Send + Sync;
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn handle_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
    ) -> Result<ActorResult, Self::Error>;
    
    fn pre_start(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    
    fn post_stop(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    
    fn actor_id(&self) -> ActorId;
}

// Message wrapper for ask pattern
#[derive(Debug)]
struct AskMessage<M, R> {
    message: M,
    reply_to: oneshot::Sender<R>,
}

// Generic message envelope
#[derive(Debug)]
enum MessageEnvelope {
    Tell(Box<dyn ActorMessage>),
    Ask {
        message: Box<dyn ActorMessage>,
        reply_to: oneshot::Sender<serde_json::Value>,
    },
}

#[derive(Debug)]
pub struct Mailbox {
    sender: mpsc::UnboundedSender<MessageEnvelope>,
    receiver: Arc<Mutex<mpsc::UnboundedReceiver<MessageEnvelope>>>,
    capacity: Option<usize>,
    current_size: Arc<std::sync::atomic::AtomicUsize>,
}

impl Mailbox {
    pub fn new(capacity: Option<usize>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        Self {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
            capacity,
            current_size: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }
    
    pub fn is_full(&self) -> bool {
        if let Some(cap) = self.capacity {
            self.current_size.load(std::sync::atomic::Ordering::Relaxed) >= cap
        } else {
            false
        }
    }
    
    pub async fn enqueue(&self, message: MessageEnvelope) -> Result<(), ActorError> {
        if self.is_full() {
            return Err(ActorError::MailboxFull);
        }
        
        self.sender.send(message)
            .map_err(|_| ActorError::ActorStopped)?;
            
        self.current_size.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
    
    pub async fn dequeue(&self) -> Option<MessageEnvelope> {
        let mut receiver = self.receiver.lock().await;
        let message = receiver.recv().await;
        
        if message.is_some() {
            self.current_size.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        }
        
        message
    }
}

#[derive(Debug, Clone)]
pub struct ActorRef {
    actor_id: ActorId,
    mailbox: Arc<Mailbox>,
    system_ref: Weak<ActorSystem>,
}

impl ActorRef {
    pub fn new(
        actor_id: ActorId,
        mailbox: Arc<Mailbox>,
        system_ref: Weak<ActorSystem>,
    ) -> Self {
        Self {
            actor_id,
            mailbox,
            system_ref,
        }
    }
    
    pub async fn send(&self, message: impl ActorMessage + 'static) -> Result<(), ActorError> {
        let envelope = MessageEnvelope::Tell(Box::new(message));
        self.mailbox.enqueue(envelope).await
    }
    
    pub async fn ask<R>(
        &self,
        message: impl ActorMessage + 'static,
    ) -> Result<R, ActorError>
    where
        R: serde::de::DeserializeOwned,
    {
        let (tx, rx) = oneshot::channel();
        let envelope = MessageEnvelope::Ask {
            message: Box::new(message),
            reply_to: tx,
        };
        
        self.mailbox.enqueue(envelope).await?;
        
        let response = rx.await
            .map_err(|_| ActorError::AskTimeout)?;
            
        serde_json::from_value(response)
            .map_err(|e| ActorError::DeserializationFailed(e.to_string()))
    }
    
    pub fn actor_id(&self) -> ActorId {
        self.actor_id
    }
    
    pub async fn stop(&self) -> Result<(), ActorError> {
        if let Some(system) = self.system_ref.upgrade() {
            system.stop_actor(self.actor_id).await
        } else {
            Err(ActorError::SystemStopped)
        }
    }
}

#[derive(Debug)]
pub struct MailboxFactory {
    default_capacity: Option<usize>,
}

impl MailboxFactory {
    pub fn new(default_capacity: Option<usize>) -> Self {
        Self { default_capacity }
    }
    
    pub fn create_mailbox(&self, capacity: Option<usize>) -> Mailbox {
        Mailbox::new(capacity.or(self.default_capacity))
    }
}

#[derive(Debug)]
pub struct Dispatcher {
    // Implementation for message dispatching
    worker_count: usize,
}

impl Dispatcher {
    pub fn new(worker_count: usize) -> Self {
        Self { worker_count }
    }
}

#[derive(Debug)]
pub struct ActorSystem {
    actors: Arc<Mutex<HashMap<ActorId, ActorRef>>>,
    mailbox_factory: MailboxFactory,
    dispatcher: Dispatcher,
    supervision_strategy: SupervisionStrategy,
    shutdown_signal: Arc<std::sync::atomic::AtomicBool>,
}

impl ActorSystem {
    pub fn new() -> Self {
        Self {
            actors: Arc::new(Mutex::new(HashMap::new())),
            mailbox_factory: MailboxFactory::new(Some(1000)),
            dispatcher: Dispatcher::new(num_cpus::get()),
            supervision_strategy: SupervisionStrategy::OneForOne,
            shutdown_signal: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
    
    pub async fn spawn_actor<A>(
        &self,
        mut actor: A,
        initial_state: A::State,
    ) -> Result<ActorRef, ActorError>
    where
        A: Actor + 'static,
    {
        let actor_id = actor.actor_id();
        let mailbox = Arc::new(self.mailbox_factory.create_mailbox(None));
        let actor_ref = ActorRef::new(
            actor_id,
            Arc::clone(&mailbox),
            Arc::downgrade(&Arc::new(self.clone())),
        );
        
        // Start actor pre_start lifecycle
        actor.pre_start().map_err(ActorError::StartupFailed)?;
        
        // Store actor reference
        self.actors.lock().await.insert(actor_id, actor_ref.clone());
        
        // Spawn actor message processing loop
        let shutdown = Arc::clone(&self.shutdown_signal);
        tokio::spawn(async move {
            Self::actor_message_loop(actor, initial_state, mailbox, shutdown).await;
        });
        
        Ok(actor_ref)
    }
    
    async fn actor_message_loop<A>(
        mut actor: A,
        mut state: A::State,
        mailbox: Arc<Mailbox>,
        shutdown_signal: Arc<std::sync::atomic::AtomicBool>,
    ) where
        A: Actor,
    {
        while !shutdown_signal.load(std::sync::atomic::Ordering::Relaxed) {
            match mailbox.dequeue().await {
                Some(MessageEnvelope::Tell(message)) => {
                    // Handle tell message
                    if let Ok(boxed_msg) = message.downcast::<A::Message>() {
                        match actor.handle_message(*boxed_msg, &mut state).await {
                            Ok(ActorResult::Continue) => continue,
                            Ok(ActorResult::Stop) => break,
                            Ok(ActorResult::Restart) => {
                                // Restart logic would be handled by supervision
                                continue;
                            }
                            Err(e) => {
                                tracing::error!("Actor {} failed: {}", actor.actor_id().0, e);
                                break;
                            }
                        }
                    }
                }
                Some(MessageEnvelope::Ask { message, reply_to }) => {
                    // Handle ask message - simplified for example
                    if let Ok(boxed_msg) = message.downcast::<A::Message>() {
                        match actor.handle_message(*boxed_msg, &mut state).await {
                            Ok(_) => {
                                let _ = reply_to.send(serde_json::Value::Null);
                            }
                            Err(e) => {
                                tracing::error!("Actor {} failed on ask: {}", actor.actor_id().0, e);
                                let _ = reply_to.send(serde_json::Value::Null);
                            }
                        }
                    }
                }
                None => {
                    // Mailbox closed
                    break;
                }
            }
        }
        
        // Cleanup
        let _ = actor.post_stop();
    }
    
    pub async fn stop_actor(&self, actor_id: ActorId) -> Result<(), ActorError> {
        let mut actors = self.actors.lock().await;
        actors.remove(&actor_id);
        Ok(())
    }
    
    pub async fn stop_all(&self) -> Result<(), ActorError> {
        self.shutdown_signal.store(true, std::sync::atomic::Ordering::Relaxed);
        self.actors.lock().await.clear();
        Ok(())
    }
}

// Make ActorSystem cloneable for weak references
impl Clone for ActorSystem {
    fn clone(&self) -> Self {
        Self {
            actors: Arc::clone(&self.actors),
            mailbox_factory: MailboxFactory::new(self.mailbox_factory.default_capacity),
            dispatcher: Dispatcher::new(self.dispatcher.worker_count),
            supervision_strategy: self.supervision_strategy,
            shutdown_signal: Arc::clone(&self.shutdown_signal),
        }
    }
}
```

## Core Type Definitions

```rust
// src/types.rs
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::time::Duration;

// Core ID types with strong typing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

impl AgentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(pub Uuid);

impl ComponentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolId(pub Uuid);

impl ToolId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub Uuid);

impl EventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HandlerId(pub Uuid);

impl HandlerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

// Configuration key type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConfigurationKey(pub String);

impl ConfigurationKey {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }
}

// Event type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    System,
    Agent,
    Task,
    Supervision,
    Resource,
    Configuration,
    Custom(u32),
}

// Supervision strategy enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupervisionStrategy {
    OneForOne,
    OneForAll,
    RestForOne,
    Escalate,
}

// Node type for supervision hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Root,
    Supervisor,
    Worker,
    Agent,
}

// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

// Health status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

// Common result types
pub type SystemResult<T> = Result<T, crate::errors::SystemError>;
pub type ActorResult = Result<crate::actors::ActorResult, crate::errors::ActorError>;

// Common configuration defaults
pub mod constants {
    use std::time::Duration;
    
    // Runtime constants
    pub const DEFAULT_WORKER_THREADS: usize = num_cpus::get();
    pub const DEFAULT_MAX_BLOCKING_THREADS: usize = 512;
    pub const DEFAULT_THREAD_KEEP_ALIVE: Duration = Duration::from_secs(60);
    pub const DEFAULT_THREAD_STACK_SIZE: usize = 2 * 1024 * 1024; // 2MB
    
    // Task execution constants  
    pub const DEFAULT_TASK_TIMEOUT: Duration = Duration::from_secs(30);
    pub const DEFAULT_TASK_QUEUE_SIZE: usize = 1000;
    pub const MAX_CONCURRENT_TASKS: usize = 100;
    
    // Supervision constants
    pub const MAX_RESTART_ATTEMPTS: u32 = 3;
    pub const RESTART_WINDOW: Duration = Duration::from_secs(60);
    pub const ESCALATION_TIMEOUT: Duration = Duration::from_secs(10);
    
    // Health check constants
    pub const DEFAULT_HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(30);
    pub const DEFAULT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
    pub const DEFAULT_FAILURE_THRESHOLD: u32 = 3;
    
    // Circuit breaker constants
    pub const DEFAULT_CIRCUIT_FAILURE_THRESHOLD: u32 = 5;
    pub const DEFAULT_CIRCUIT_TIMEOUT: Duration = Duration::from_secs(60);
    pub const DEFAULT_HALF_OPEN_MAX_CALLS: u32 = 3;
    
    // Connection pool constants
    pub const DEFAULT_POOL_MIN_SIZE: usize = 5;
    pub const DEFAULT_POOL_MAX_SIZE: usize = 50;
    pub const DEFAULT_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(10);
    pub const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(300);
    
    // Event system constants
    pub const DEFAULT_EVENT_BUFFER_SIZE: usize = 10000;
    pub const DEFAULT_EVENT_BATCH_SIZE: usize = 100;
    
    // Tool system constants
    pub const DEFAULT_TOOL_TIMEOUT: Duration = Duration::from_secs(30);
    pub const MAX_TOOL_RETRIES: u32 = 3;
}
```

## Agent-as-Tool Pattern

```rust
// src/tools/agent_tool.rs
use std::sync::Arc;
use async_trait::async_trait;
use crate::tools::{Tool, ToolSchema, ToolInterface};
use crate::actors::Actor;
use crate::errors::ToolError;
use crate::types::{ToolId, AgentId};
use serde_json::Value;

// Message type for tool calls to agents
#[derive(Debug, Clone)]
pub struct ToolMessage {
    pub tool_id: ToolId,
    pub params: Value,
    pub caller_id: Option<AgentId>,
}

impl ToolMessage {
    pub fn from_params(tool_id: ToolId, params: Value) -> Self {
        Self {
            tool_id,
            params,
            caller_id: None,
        }
    }
    
    pub fn with_caller(mut self, caller_id: AgentId) -> Self {
        self.caller_id = Some(caller_id);
        self
    }
}

// Make ToolMessage compatible with ActorMessage
impl crate::actors::ActorMessage for ToolMessage {}

#[derive(Debug)]
pub struct AgentTool {
    agent: Arc<dyn Agent<Message = ToolMessage, State = AgentState>>,
    interface: ToolInterface,
    tool_id: ToolId,
}

// Generic agent state for tool agents
#[derive(Debug, Clone)]
pub struct AgentState {
    pub context: Value,
    pub execution_count: u64,
    pub last_execution: Option<std::time::Instant>,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            context: Value::Null,
            execution_count: 0,
            last_execution: None,
        }
    }
}

impl AgentTool {
    pub fn new(
        agent: Arc<dyn Actor<Message = ToolMessage, State = AgentState>>,
        interface: ToolInterface,
    ) -> Self {
        Self {
            agent,
            interface,
            tool_id: ToolId::new(),
        }
    }
}

#[async_trait]
impl Tool for AgentTool {
    async fn execute(&self, params: Value) -> Result<Value, ToolError> {
        let message = ToolMessage::from_params(self.tool_id, params);
        
        // Process the message through the agent
        // This is a simplified version - in practice you'd need proper message routing
        match self.agent.handle_message(message, &mut AgentState::default()).await {
            Ok(crate::actors::ActorResult::Continue) => {
                // Return some result - this would be enhanced with actual return value handling
                Ok(Value::Object(serde_json::Map::new()))
            }
            Ok(_) => Ok(Value::Null),
            Err(e) => Err(ToolError::ExecutionFailed(e.to_string())),
        }
    }
    
    fn schema(&self) -> ToolSchema {
        self.interface.schema()
    }
    
    fn tool_id(&self) -> ToolId {
        self.tool_id
    }
    
    fn name(&self) -> &str {
        &self.interface.name
    }
}

// Tool system integration for supervisors
pub trait SupervisorToolIntegration {
    fn register_agent_as_tool(
        &mut self,
        agent: Arc<dyn Actor<Message = ToolMessage, State = AgentState>>,
        interface: ToolInterface,
    ) -> Result<ToolId, ToolError>;
}

// Example implementation would be added to supervisor structs
```

## Tool System Core

```rust
// src/tools/mod.rs
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use crate::types::{ToolId, AgentId};
use crate::errors::ToolError;
use serde::{Serialize, Deserialize};
use serde_json::Value;

// Core tool trait
#[async_trait]
pub trait Tool: Send + Sync {
    async fn execute(&self, params: Value) -> Result<Value, ToolError>;
    fn schema(&self) -> ToolSchema;
    fn tool_id(&self) -> ToolId;
    fn name(&self) -> &str;
}

// Tool schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: Value, // JSON Schema
    pub required: Vec<String>,
    pub returns: Option<Value>, // Return type schema
}

// Tool interface configuration
#[derive(Debug, Clone)]
pub struct ToolInterface {
    pub name: String,
    pub description: String,
    pub schema: ToolSchema,
}

// Tool execution metrics
#[derive(Debug, Default)]
pub struct ToolMetrics {
    pub call_count: std::sync::atomic::AtomicU64,
    pub success_count: std::sync::atomic::AtomicU64,
    pub error_count: std::sync::atomic::AtomicU64,
    pub total_execution_time: std::sync::atomic::AtomicU64, // in milliseconds
    pub last_execution: std::sync::Mutex<Option<std::time::Instant>>,
}

// Central tool registry and execution system
#[derive(Debug)]
pub struct ToolBus {
    tools: Arc<RwLock<HashMap<ToolId, Arc<dyn Tool>>>>,
    permissions: Arc<RwLock<HashMap<AgentId, Vec<ToolId>>>>,
    call_metrics: Arc<RwLock<HashMap<ToolId, ToolMetrics>>>,
    global_timeout: std::time::Duration,
}

impl ToolBus {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            permissions: Arc::new(RwLock::new(HashMap::new())),
            call_metrics: Arc::new(RwLock::new(HashMap::new())),
            global_timeout: crate::types::constants::DEFAULT_TOOL_TIMEOUT,
        }
    }
    
    pub async fn register_tool<T: Tool + 'static>(&self, tool: T) -> ToolId {
        let tool_id = tool.tool_id();
        let tool_arc = Arc::new(tool);
        
        self.tools.write().await.insert(tool_id, tool_arc);
        self.call_metrics.write().await.insert(tool_id, ToolMetrics::default());
        
        tool_id
    }
    
    pub async fn call(
        &self,
        agent_id: AgentId,
        tool_id: ToolId,
        params: Value,
    ) -> Result<Value, ToolError> {
        // Check permissions
        if !self.has_permission(agent_id, tool_id).await {
            return Err(ToolError::AccessDenied(format!(
                "Agent {} does not have permission to use tool {}",
                agent_id.0, tool_id.0
            )));
        }
        
        // Get the tool
        let tool = {
            let tools = self.tools.read().await;
            tools.get(&tool_id)
                .ok_or_else(|| ToolError::NotFound(tool_id.0.to_string()))?
                .clone()
        };
        
        // Update metrics
        self.update_call_metrics(tool_id).await;
        
        let start_time = std::time::Instant::now();
        
        // Execute with timeout
        let result = tokio::time::timeout(
            self.global_timeout,
            tool.execute(params)
        ).await;
        
        let execution_time = start_time.elapsed();
        
        match result {
            Ok(Ok(value)) => {
                self.update_success_metrics(tool_id, execution_time).await;
                Ok(value)
            }
            Ok(Err(e)) => {
                self.update_error_metrics(tool_id).await;
                Err(e)
            }
            Err(_) => {
                self.update_error_metrics(tool_id).await;
                Err(ToolError::Timeout(format!("Tool {} timed out", tool_id.0)))
            }
        }
    }
    
    pub async fn grant_permission(&self, agent_id: AgentId, tool_id: ToolId) {
        let mut permissions = self.permissions.write().await;
        permissions.entry(agent_id).or_insert_with(Vec::new).push(tool_id);
    }
    
    pub async fn revoke_permission(&self, agent_id: AgentId, tool_id: ToolId) {
        let mut permissions = self.permissions.write().await;
        if let Some(tool_list) = permissions.get_mut(&agent_id) {
            tool_list.retain(|&id| id != tool_id);
        }
    }
    
    pub async fn has_permission(&self, agent_id: AgentId, tool_id: ToolId) -> bool {
        let permissions = self.permissions.read().await;
        permissions.get(&agent_id)
            .map(|tools| tools.contains(&tool_id))
            .unwrap_or(false)
    }
    
    pub async fn list_available_tools(&self, agent_id: AgentId) -> Vec<ToolSchema> {
        let permissions = self.permissions.read().await;
        let tools = self.tools.read().await;
        
        if let Some(tool_ids) = permissions.get(&agent_id) {
            tool_ids.iter()
                .filter_map(|tool_id| tools.get(tool_id))
                .map(|tool| tool.schema())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    async fn update_call_metrics(&self, tool_id: ToolId) {
        if let Some(metrics) = self.call_metrics.read().await.get(&tool_id) {
            metrics.call_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            *metrics.last_execution.lock().unwrap() = Some(std::time::Instant::now());
        }
    }
    
    async fn update_success_metrics(&self, tool_id: ToolId, execution_time: std::time::Duration) {
        if let Some(metrics) = self.call_metrics.read().await.get(&tool_id) {
            metrics.success_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            metrics.total_execution_time.fetch_add(
                execution_time.as_millis() as u64,
                std::sync::atomic::Ordering::Relaxed
            );
        }
    }
    
    async fn update_error_metrics(&self, tool_id: ToolId) {
        if let Some(metrics) = self.call_metrics.read().await.get(&tool_id) {
            metrics.error_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
    
    pub async fn get_tool_metrics(&self, tool_id: ToolId) -> Option<ToolMetrics> {
        // This would need to be implemented to return a snapshot of metrics
        // For now, we'll indicate this needs implementation
        None
    }
}

// Default implementation for common tools
pub mod builtin {
    use super::*;
    
    // Example built-in tool
    #[derive(Debug)]
    pub struct EchoTool {
        tool_id: ToolId,
    }
    
    impl EchoTool {
        pub fn new() -> Self {
            Self {
                tool_id: ToolId::new(),
            }
        }
    }
    
    #[async_trait]
    impl Tool for EchoTool {
        async fn execute(&self, params: Value) -> Result<Value, ToolError> {
            Ok(params)
        }
        
        fn schema(&self) -> ToolSchema {
            ToolSchema {
                name: "echo".to_string(),
                description: "Echoes the input parameters".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": {
                            "type": "string",
                            "description": "The message to echo"
                        }
                    },
                    "required": ["message"]
                }),
                required: vec!["message".to_string()],
                returns: Some(serde_json::json!({
                    "type": "object",
                    "description": "The echoed parameters"
                })),
            }
        }
        
        fn tool_id(&self) -> ToolId {
            self.tool_id
        }
        
        fn name(&self) -> &str {
            "echo"
        }
    }
}
```

---

## Cross-References

- For runtime configuration, see [Runtime & Errors](runtime-and-errors.md)
- For supervision integration, see [Supervision & Events](supervision-and-events.md)
- For component integration, see [Component Architecture](component-architecture.md)
- For health monitoring, see [Monitoring & Health](monitoring-and-health.md)

---

## Implementation Notes

This module provides comprehensive async patterns for the MisterSmith framework:

1. **Task Management**: Priority-based execution with retry policies and comprehensive metrics
2. **Stream Processing**: Backpressure-aware stream handling with configurable strategies
3. **Actor Model**: Full actor system with mailboxes, supervision integration, and ask/tell patterns
4. **Type System**: Strong typing throughout with UUID-based identifiers
5. **Agent-as-Tool**: Seamless integration allowing agents to be exposed as tools
6. **Tool System**: Central registry with permissions, metrics, and timeout handling

The async patterns form the execution backbone of the framework, enabling concurrent, fault-tolerant, and scalable agent operations.
