# Coding Standards & Conventions

## Overview

Concrete coding standards for implementing agents in the Mister Smith Framework.
Focused on Rust best practices, agent-specific patterns, and security requirements.

---

## 1. Core Principles

### 1.1 Core Requirements

- **Type Safety**: Use Rust's type system to prevent runtime errors
- **Async by Default**: All I/O operations use Tokio async/await
- **Explicit Error Handling**: No `unwrap()` or `panic!()` in production
- **Documentation**: All public APIs require rustdoc
- **Testing**: Minimum 80% coverage with unit and integration tests
- **Security**: Input validation, resource limits, audit logging
- **Performance**: Minimize allocations, use `Arc` for shared state

### 1.2 Agent Design Patterns

- **Actor Model**: Agents as isolated actors with message passing
- **Supervision Trees**: Hierarchical fault tolerance
- **RAII Resources**: Automatic cleanup via Drop trait
- **Builder Pattern**: For agent and config construction
- **Tool Registry**: Dynamic tool registration and discovery

---

## 2. Naming Conventions

### 2.1 Module Names

```rust
// ✅ Good
mod agent_runtime;
mod tool_registry;
mod message_handler;

// ❌ Bad
mod AgentRuntime;
mod tool-registry;
```

### 2.2 Type Names

Use `PascalCase` for all type names:

```rust
// ✅ Good - Structs
pub struct RuntimeManager;
pub struct ActorSystem;
pub struct SupervisionTree;

// ✅ Good - Enums
pub enum SupervisionStrategy {
    OneForOne,
    OneForAll,
    RestForOne,
}

// ✅ Good - Traits
pub trait AsyncTask;
pub trait EventHandler;
pub trait Supervisor;

// ❌ Bad
pub struct runtime_manager;
pub enum supervision_strategy;
pub trait async_task;
```

### 2.3 Function and Method Names

Use `snake_case` for functions and methods:

```rust
// ✅ Good
impl RuntimeManager {
    pub fn new() -> Self { /* ... */ }
    pub async fn start_runtime(&self) -> SystemResult<()> { /* ... */ }
    pub fn health_check(&self) -> HealthStatus { /* ... */ }
    pub async fn shutdown_gracefully(&self) -> SystemResult<()> { /* ... */ }
}

// ❌ Bad
impl RuntimeManager {
    pub fn startRuntime(&self) { /* ... */ }
    pub fn healthCheck(&self) { /* ... */ }
    pub fn shutdownGracefully(&self) { /* ... */ }
}
```

### 2.4 Constants and Statics

Use `SCREAMING_SNAKE_CASE` for constants:

```rust
// ✅ Good
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RETRY_ATTEMPTS: usize = 3;
const SUPERVISION_INTERVAL: Duration = Duration::from_millis(100);

static GLOBAL_REGISTRY: Lazy<ServiceRegistry> = Lazy::new(ServiceRegistry::new);

// ❌ Bad
const defaultTimeout: Duration = Duration::from_secs(30);
const max_retry_attempts: usize = 3;
```

### 2.5 Error Types

Error types must end with `Error`:

```rust
// ✅ Good
#[derive(Debug, Error)]
pub enum SystemError { /* ... */ }

#[derive(Debug, Error)]
pub enum RuntimeError { /* ... */ }

#[derive(Debug, Error)]
pub enum ActorError { /* ... */ }

// ❌ Bad
pub enum SystemFailure { /* ... */ }
pub enum RuntimeException { /* ... */ }
```

### 2.6 Result Types

Define custom Result types for modules:

```rust
// ✅ Good
pub type SystemResult<T> = Result<T, SystemError>;
pub type RuntimeResult<T> = Result<T, RuntimeError>;
pub type ActorResult<T> = Result<T, ActorError>;

// Usage
pub async fn start_system() -> SystemResult<SystemCore> { /* ... */ }
```

---

## 3. Code Organization

### 3.1 File Structure

Each module should follow this structure:

```markdown
module_name/
├── mod.rs              # Module declaration and re-exports
├── types.rs            # Type definitions and enums
├── manager.rs          # Main implementation
├── config.rs           # Configuration types
├── error.rs            # Error definitions
└── tests.rs            # Module-specific tests
```

### 3.2 Module Declaration Template

```rust
//! Module Name
//! 
//! Brief description of the module's purpose and responsibilities.
//! 
//! # Examples
//! 
//! ```rust
//! use mister_smith::{ModuleName, ModuleConfig};
//! 
//! let config = ModuleConfig::default();
//! let instance = ModuleName::new(config)?;
//! ```

// Re-exports for public API
pub use self::{
    manager::ModuleManager,
    config::{ModuleConfig, ModuleSettings},
    error::{ModuleError, ModuleResult},
    types::{ModuleType, ModuleState},
};

// Internal modules
mod manager;
mod config;
mod error;
mod types;

#[cfg(test)]
mod tests;
```

### 3.3 Dependency Hierarchy

Modules must follow this dependency order (no circular dependencies):

```markdown
errors ← (foundation module)
config ← errors
events ← errors, config
runtime ← config, monitoring, errors
monitoring ← events, errors
async_patterns ← runtime, errors, monitoring
resources ← config, monitoring, errors
transport ← events, errors
security ← config, errors
actor ← async_patterns, events, supervision, errors
supervision ← actor, events, monitoring, errors
tools ← agents, security, errors
agents ← actor, tools, supervision, config, errors
```

---

## 4. Type System Guidelines

### 4.1 Generic Constraints

Always be explicit with generic constraints:

```rust
// ✅ Good - Explicit constraints
#[async_trait]
pub trait AsyncTask: Send + Sync + 'static {
    type Output: Send + 'static;
    type Error: Send + std::error::Error + 'static;
    
    async fn execute(&self) -> Result<Self::Output, Self::Error>;
}

// ❌ Bad - Missing constraints
pub trait AsyncTask {
    type Output;
    type Error;
    
    async fn execute(&self) -> Result<Self::Output, Self::Error>;
}
```

### 4.2 Trait Definitions

Use the following pattern for trait definitions:

```rust
#[async_trait]
pub trait ComponentName: Send + Sync + 'static {
    /// Associated types with proper bounds
    type Config: Send + Sync + Clone + 'static;
    type Error: Send + std::error::Error + 'static;
    type Output: Send + 'static;
    
    /// Core functionality methods
    async fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error>;
    async fn process(&self, input: Input) -> Result<Self::Output, Self::Error>;
    fn health_check(&self) -> HealthStatus;
    
    /// Optional methods with default implementations
    fn component_id(&self) -> ComponentId {
        ComponentId::new()
    }
    
    fn dependencies() -> Vec<TypeId>
    where Self: Sized {
        Vec::new()
    }
}
```

### 4.3 Struct Definitions

```rust
/// Component description with examples
/// 
/// # Examples
/// 
/// ```rust
/// let config = ComponentConfig::default();
/// let component = Component::new(config)?;
/// ```
pub struct Component<T, E> 
where 
    T: Send + Sync + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    /// Field documentation
    inner: Arc<Mutex<InnerState<T>>>,
    config: ComponentConfig,
    metrics: ComponentMetrics,
    _phantom: PhantomData<E>,
}

impl<T, E> Component<T, E>
where 
    T: Send + Sync + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    /// Constructor with validation
    pub fn new(config: ComponentConfig) -> Result<Self, ComponentError> {
        config.validate()?;
        
        Ok(Self {
            inner: Arc::new(Mutex::new(InnerState::new())),
            config,
            metrics: ComponentMetrics::new(),
            _phantom: PhantomData,
        })
    }
}
```

### 4.4 Enum Definitions

```rust
/// Enum with comprehensive documentation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentState {
    /// Initial state before initialization
    Uninitialized,
    /// Component is starting up
    Starting,
    /// Component is running normally
    Running {
        /// Timestamp when component started
        started_at: Instant,
        /// Current processing count
        active_tasks: usize,
    },
    /// Component is shutting down
    Stopping,
    /// Component has stopped
    Stopped {
        /// Reason for stopping
        reason: StopReason,
    },
    /// Component encountered an error
    Failed {
        /// Error that caused failure
        error: String,
        /// Whether recovery is possible
        recoverable: bool,
    },
}
```

---

## 5. Async/Await Patterns

### 5.1 Async Trait Usage

Always use `#[async_trait]` for traits with async methods:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait AsyncProcessor: Send + Sync + 'static {
    type Input: Send + 'static;
    type Output: Send + 'static;
    type Error: Send + std::error::Error + 'static;
    
    async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}
```

### 5.2 Timeout Handling

Always handle timeouts explicitly:

```rust
use tokio::time::{timeout, Duration};

// ✅ Good - Explicit timeout
pub async fn execute_with_timeout<T>(
    future: impl Future<Output = T>,
    timeout_duration: Duration,
) -> Result<T, TimeoutError> {
    timeout(timeout_duration, future)
        .await
        .map_err(|_| TimeoutError::Elapsed)
}

// ✅ Good - Using timeout in methods
impl TaskExecutor {
    pub async fn submit<T>(&self, task: T) -> TaskResult<T::Output>
    where T: AsyncTask {
        let timeout_duration = task.timeout();
        let result = timeout(timeout_duration, task.execute()).await;
        
        match result {
            Ok(Ok(output)) => Ok(output),
            Ok(Err(e)) => Err(TaskError::ExecutionFailed(e.to_string())),
            Err(_) => Err(TaskError::Timeout),
        }
    }
}
```

### 5.3 Concurrency Patterns

Use appropriate concurrency primitives:

```rust
// ✅ Good - Multiple concurrent operations
pub async fn process_all<T, F, Fut>(
    items: Vec<T>,
    processor: F,
) -> Vec<Result<F::Output, F::Error>>
where
    F: Fn(T) -> Fut + Send + Sync,
    Fut: Future<Output = Result<F::Output, F::Error>> + Send,
    F::Output: Send,
    F::Error: Send,
{
    let futures = items.into_iter().map(processor);
    futures::future::join_all(futures).await
}

// ✅ Good - Rate limiting with semaphore
pub struct RateLimitedExecutor {
    semaphore: Arc<Semaphore>,
}

impl RateLimitedExecutor {
    pub async fn execute<F, T>(&self, operation: F) -> T
    where F: Future<Output = T> {
        let _permit = self.semaphore.acquire().await.unwrap();
        operation.await
    }
}
```

### 5.4 Spawning Tasks

Follow these patterns for task spawning:

```rust
// ✅ Good - Named task with error handling
pub async fn spawn_background_task<F, T>(
    name: &str,
    future: F,
) -> JoinHandle<Result<T, TaskError>>
where
    F: Future<Output = Result<T, TaskError>> + Send + 'static,
    T: Send + 'static,
{
    tokio::spawn(async move {
        tracing::info!("Starting background task: {}", name);
        let result = future.await;
        if let Err(ref e) = result {
            tracing::error!("Background task {} failed: {}", name, e);
        }
        result
    })
}

// ✅ Good - Task with cancellation support
pub async fn spawn_cancellable_task<F, T>(
    future: F,
    cancellation_token: CancellationToken,
) -> Result<T, TaskError>
where
    F: Future<Output = T> + Send,
    T: Send,
{
    tokio::select! {
        result = future => Ok(result),
        _ = cancellation_token.cancelled() => Err(TaskError::Cancelled),
    }
}
```

---

## 6. Error Handling

### 6.1 Error Hierarchy

Define errors using `thiserror` with proper hierarchy:

```rust
use thiserror::Error;

/// Top-level system error encompassing all subsystem errors
#[derive(Debug, Error)]
pub enum SystemError {
    #[error("Runtime error: {0}")]
    Runtime(#[from] RuntimeError),
    
    #[error("Actor system error: {0}")]
    Actor(#[from] ActorError),
    
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigError),
    
    #[error("Resource error: {0}")]
    Resource(#[from] ResourceError),
    
    #[error("Tool execution error: {0}")]
    Tool(#[from] ToolError),
    
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
}

/// Specific subsystem error with context
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Failed to initialize runtime: {reason}")]
    InitializationFailed { reason: String },
    
    #[error("Runtime shutdown timeout after {timeout_secs} seconds")]
    ShutdownTimeout { timeout_secs: u64 },
    
    #[error("Invalid runtime configuration: {field} = {value}")]
    InvalidConfiguration { field: String, value: String },
    
    #[error("Runtime panic occurred: {message}")]
    PanicOccurred { message: String },
    
    #[error("I/O error during runtime operation")]
    IoError(#[from] std::io::Error),
}
```

### 6.2 Result Type Definitions

```rust
/// Module-specific result type
pub type RuntimeResult<T> = Result<T, RuntimeError>;
pub type SystemResult<T> = Result<T, SystemError>;

/// Convenient type aliases for common cases
pub type VoidResult = SystemResult<()>;
pub type ComponentResult<T> = Result<T, ComponentError>;
```

### 6.3 Error Construction and Context

```rust
impl RuntimeError {
    /// Constructor for configuration errors
    pub fn invalid_config(field: &str, value: &str) -> Self {
        Self::InvalidConfiguration {
            field: field.to_string(),
            value: value.to_string(),
        }
    }
    
    /// Constructor for initialization errors
    pub fn init_failed(reason: impl Into<String>) -> Self {
        Self::InitializationFailed {
            reason: reason.into(),
        }
    }
}

// ✅ Good - Using error constructors
impl RuntimeManager {
    pub fn validate_config(config: &RuntimeConfig) -> RuntimeResult<()> {
        if config.worker_threads == 0 {
            return Err(RuntimeError::invalid_config("worker_threads", "0"));
        }
        
        if config.max_memory_mb > 16384 {
            return Err(RuntimeError::invalid_config(
                "max_memory_mb", 
                &config.max_memory_mb.to_string()
            ));
        }
        
        Ok(())
    }
}
```

### 6.4 Error Propagation

```rust
// ✅ Good - Use ? operator for error propagation
impl SystemCore {
    pub async fn initialize(config: SystemConfig) -> SystemResult<Self> {
        let runtime_manager = RuntimeManager::new(config.runtime)?;
        let actor_system = ActorSystem::new(config.actor)?;
        let supervision_tree = SupervisionTree::new(config.supervision)?;
        
        let core = Self {
            runtime_manager,
            actor_system,
            supervision_tree,
        };
        
        core.start_all_components().await?;
        Ok(core)
    }
    
    async fn start_all_components(&self) -> SystemResult<()> {
        self.runtime_manager.start().await?;
        self.actor_system.start().await?;
        self.supervision_tree.start().await?;
        Ok(())
    }
}

// ❌ Bad - Don't use unwrap() in production code
impl SystemCore {
    pub async fn initialize(config: SystemConfig) -> Self {
        let runtime_manager = RuntimeManager::new(config.runtime).unwrap();
        let actor_system = ActorSystem::new(config.actor).unwrap();
        // ... more unwraps
    }
}
```

---

## 7. Documentation Standards

### 7.1 Crate-Level Documentation

```rust
//! Mister Smith AI Agent Framework
//!
//! A comprehensive framework for building AI agents with async execution,
//! supervision trees, and tool integration.
//!
//! # Features
//!
//! - **Async Runtime**: Built on Tokio for high-performance async execution
//! - **Supervision Trees**: Fault-tolerant actor supervision patterns
//! - **Tool Integration**: Extensible tool system for agent capabilities
//! - **Event System**: Reactive event-driven architecture
//! - **Configuration**: Hot-reloadable configuration management
//!
//! # Quick Start
//!
//! ```rust
//! use mister_smith::{SystemBuilder, SystemConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = SystemConfig::default();
//!     let system = SystemBuilder::new()
//!         .with_config(config)
//!         .build()
//!         .await?;
//!     
//!     system.start().await?;
//!     Ok(())
//! }
//! ```

#![doc = include_str!("../README.md")]
#![deny(missing_docs, unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
```

### 7.2 Module Documentation

```rust
//! Runtime Management
//!
//! This module provides comprehensive runtime management capabilities for the
//! Mister Smith framework, including Tokio runtime lifecycle management,
//! resource monitoring, and graceful shutdown procedures.
//!
//! # Architecture
//!
//! The runtime manager follows a layered architecture:
//!
//! ```text
//! ┌─────────────────────┐
//! │   RuntimeManager    │
//! ├─────────────────────┤
//! │   HealthMonitor     │
//! ├─────────────────────┤
//! │   MetricsCollector  │
//! ├─────────────────────┤
//! │   Tokio Runtime     │
//! └─────────────────────┘
//! ```
//!
//! # Examples
//!
//! Creating and managing a runtime:
//!
//! ```rust
//! use mister_smith::runtime::{RuntimeManager, RuntimeConfig};
//!
//! async fn setup_runtime() -> Result<RuntimeManager, RuntimeError> {
//!     let config = RuntimeConfig::builder()
//!         .worker_threads(4)
//!         .max_memory_mb(1024)
//!         .build();
//!     
//!     let manager = RuntimeManager::new(config)?;
//!     manager.start().await?;
//!     Ok(manager)
//! }
//! ```
```

### 7.3 Function Documentation

```rust
impl RuntimeManager {
    /// Creates a new runtime manager with the specified configuration.
    ///
    /// This method validates the configuration and initializes the underlying
    /// Tokio runtime with the specified parameters. The runtime is created
    /// but not started until [`start`] is called.
    ///
    /// # Arguments
    ///
    /// * `config` - Runtime configuration parameters
    ///
    /// # Returns
    ///
    /// A new `RuntimeManager` instance ready for startup.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::InvalidConfiguration`] if the configuration
    /// contains invalid values:
    /// - `worker_threads` cannot be 0
    /// - `max_memory_mb` cannot exceed system limits
    /// - `shutdown_timeout` must be positive
    ///
    /// Returns [`RuntimeError::InitializationFailed`] if the underlying
    /// Tokio runtime cannot be created due to system resource constraints.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mister_smith::runtime::{RuntimeManager, RuntimeConfig};
    ///
    /// let config = RuntimeConfig::builder()
    ///     .worker_threads(4)
    ///     .build();
    ///
    /// let manager = RuntimeManager::new(config)?;
    /// assert!(!manager.is_running());
    /// # Ok::<(), RuntimeError>(())
    /// ```
    ///
    /// # See Also
    ///
    /// - [`start`] - Start the runtime
    /// - [`shutdown`] - Gracefully shutdown the runtime
    pub fn new(config: RuntimeConfig) -> RuntimeResult<Self> {
        config.validate()?;
        
        let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(config.worker_threads)
            .max_blocking_threads(config.max_blocking_threads)
            .thread_name("mister-smith-worker")
            .build()
            .map_err(|e| RuntimeError::InitializationFailed {
                reason: e.to_string(),
            })?;
        
        Ok(Self {
            runtime: Arc::new(tokio_runtime),
            config,
            state: Arc::new(AtomicBool::new(false)),
            health_monitor: HealthMonitor::new(),
        })
    }
}
```

### 7.4 Error Documentation

```rust
/// Runtime management errors
///
/// This enum represents all possible errors that can occur during runtime
/// management operations. Each variant provides specific context about
/// the error condition and potential recovery strategies.
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// Configuration validation failed
    ///
    /// This error occurs when the provided [`RuntimeConfig`] contains
    /// invalid values that would prevent proper runtime operation.
    ///
    /// # Recovery
    ///
    /// - Check configuration values against documented limits
    /// - Use [`RuntimeConfig::validate`] to identify specific issues
    /// - Consider using [`RuntimeConfig::default`] as a starting point
    #[error("Invalid runtime configuration: {field} = {value}")]
    InvalidConfiguration {
        /// Name of the invalid configuration field
        field: String,
        /// The invalid value that was provided
        value: String,
    },
    
    /// Runtime initialization failed
    ///
    /// This error occurs when the underlying Tokio runtime cannot be
    /// created, typically due to system resource constraints or
    /// permission issues.
    ///
    /// # Recovery
    ///
    /// - Reduce the number of worker threads
    /// - Check available system memory
    /// - Verify process permissions
    #[error("Runtime initialization failed: {reason}")]
    InitializationFailed {
        /// Detailed reason for the initialization failure
        reason: String,
    },
}
```

---

## 8. Import Organization

### 8.1 Import Ordering

Organize imports in this order with blank lines between sections:

```rust
// 1. Internal crate imports
use crate::{
    errors::{SystemError, SystemResult},
    config::{Configuration, RuntimeConfig},
    monitoring::{HealthMonitor, MetricsCollector},
};

// 2. External async/tokio imports
use tokio::{
    sync::{Mutex, RwLock, Semaphore},
    task::JoinHandle,
    time::{timeout, Duration, Instant},
};

// 3. Other external crate imports
use async_trait::async_trait;
use futures::{Stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

// 4. Standard library imports
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, atomic::{AtomicBool, Ordering}},
    time::Duration as StdDuration,
};
```

### 8.2 Re-exports

Use strategic re-exports in `mod.rs` files:

```rust
// src/runtime/mod.rs

//! Runtime management module

// Public API re-exports
pub use self::{
    manager::{RuntimeManager, RuntimeState},
    config::{RuntimeConfig, RuntimeConfigBuilder},
    error::{RuntimeError, RuntimeResult},
    health::{HealthMonitor, HealthStatus},
    metrics::{RuntimeMetrics, MetricsCollector},
};

// Internal modules
mod manager;
mod config;
mod error;
mod health;
mod metrics;

// Conditional re-exports based on features
#[cfg(feature = "tracing")]
pub use self::tracing::RuntimeTracer;

#[cfg(feature = "tracing")]
mod tracing;
```

### 8.3 Grouped Imports

Group related imports together:

```rust
// Async primitives
use tokio::{
    sync::{Mutex, RwLock, Semaphore, mpsc, oneshot},
    task::{JoinHandle, yield_now},
    time::{sleep, timeout, interval, Duration, Instant},
};

// Futures utilities
use futures::{
    future::{join_all, try_join_all, select_all},
    stream::{Stream, StreamExt, TryStreamExt},
    sink::{Sink, SinkExt},
};

// Serialization
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json, from_str, to_string};

// Error handling
use thiserror::Error;
use anyhow::{Context, Result as AnyhowResult};
```

---

## 9. Testing Conventions

### 9.1 Test Module Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::{assert_ready, assert_pending};
    use std::time::Duration;
    
    // Test helper functions
    fn create_test_config() -> RuntimeConfig {
        RuntimeConfig::builder()
            .worker_threads(1)
            .max_memory_mb(64)
            .build()
    }
    
    async fn setup_test_runtime() -> RuntimeResult<RuntimeManager> {
        let config = create_test_config();
        RuntimeManager::new(config)
    }
    
    // Unit tests
    #[tokio::test]
    async fn test_runtime_manager_creation_succeeds() {
        let manager = setup_test_runtime().await;
        assert!(manager.is_ok());
    }
    
    #[tokio::test]
    async fn test_runtime_manager_starts_successfully() {
        let manager = setup_test_runtime().await.unwrap();
        let result = manager.start().await;
        assert!(result.is_ok());
        assert!(manager.is_running());
    }
    
    #[tokio::test]
    async fn test_runtime_manager_handles_invalid_config() {
        let config = RuntimeConfig::builder()
            .worker_threads(0) // Invalid
            .build();
        
        let result = RuntimeManager::new(config);
        assert!(matches!(result, Err(RuntimeError::InvalidConfiguration { .. })));
    }
    
    // Property-based tests
    #[cfg(feature = "proptest")]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;
        
        proptest! {
            #[test]
            fn test_runtime_config_validation(
                worker_threads in 1u32..16,
                max_memory_mb in 64u32..2048
            ) {
                let config = RuntimeConfig::builder()
                    .worker_threads(worker_threads)
                    .max_memory_mb(max_memory_mb)
                    .build();
                
                assert!(config.validate().is_ok());
            }
        }
    }
    
    // Integration tests
    mod integration {
        use super::*;
        
        #[tokio::test]
        async fn test_full_runtime_lifecycle() {
            let manager = setup_test_runtime().await.unwrap();
            
            // Start
            manager.start().await.unwrap();
            assert!(manager.is_running());
            
            // Use runtime
            let handle = manager.spawn(async { 42 }).await.unwrap();
            let result = handle.await.unwrap();
            assert_eq!(result, 42);
            
            // Stop
            manager.shutdown().await.unwrap();
            assert!(!manager.is_running());
        }
    }
}
```

### 9.2 Test Naming Conventions

```rust
// ✅ Good - Descriptive test names
#[tokio::test]
async fn test_actor_system_spawns_actor_successfully() { /* ... */ }

#[tokio::test]
async fn test_supervision_tree_restarts_failed_actor() { /* ... */ }

#[tokio::test]
async fn test_tool_execution_validates_parameters() { /* ... */ }

#[tokio::test]
async fn test_event_bus_delivers_events_to_subscribers() { /* ... */ }

// ❌ Bad - Vague test names
#[tokio::test]
async fn test_actor() { /* ... */ }

#[tokio::test]
async fn test_supervision() { /* ... */ }

#[tokio::test]
async fn test_tools() { /* ... */ }
```

### 9.3 Async Test Patterns

```rust
#[cfg(test)]
mod async_tests {
    use super::*;
    use tokio::time::{sleep, Duration};
    use tokio_test::{assert_ready, assert_pending, task};
    
    #[tokio::test]
    async fn test_async_operation_completes() {
        let operation = async_operation();
        let result = operation.await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_async_operation_with_timeout() {
        let operation = async_operation();
        let result = tokio::time::timeout(Duration::from_secs(1), operation).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_future_readiness() {
        let mut task = task::spawn(async_operation());
        
        // Future should be pending initially
        assert_pending!(task.poll());
        
        // After some time, should be ready
        sleep(Duration::from_millis(100)).await;
        assert_ready!(task.poll());
    }
    
    #[tokio::test]
    async fn test_stream_processing() {
        let stream = create_test_stream();
        let results: Vec<_> = stream.collect().await;
        assert_eq!(results.len(), 5);
    }
}
```

### 9.4 Mock and Test Doubles

```rust
#[cfg(test)]
mod mocks {
    use super::*;
    use mockall::{mock, predicate::*};
    
    mock! {
        pub TestActor {}
        
        #[async_trait]
        impl Actor for TestActor {
            type Message = TestMessage;
            type State = TestState;
            type Error = TestError;
            
            async fn handle_message(
                &mut self,
                message: Self::Message,
                state: &mut Self::State,
            ) -> Result<ActorResult, Self::Error>;
        }
    }
    
    #[tokio::test]
    async fn test_with_mock_actor() {
        let mut mock_actor = MockTestActor::new();
        mock_actor
            .expect_handle_message()
            .with(eq(TestMessage::Start))
            .times(1)
            .returning(|_| Ok(ActorResult::Continue));
        
        let result = mock_actor.handle_message(
            TestMessage::Start,
            &mut TestState::new(),
        ).await;
        
        assert!(result.is_ok());
    }
}
```

---

## 10. Code Review Guidelines

### 10.1 Type Safety Checklist

- [ ] All generic constraints are explicit and appropriate
- [ ] Lifetime parameters are correctly specified
- [ ] No unnecessary `unsafe` code
- [ ] Error handling is comprehensive (no `unwrap()` in production)
- [ ] All public APIs have proper bounds (`Send + Sync + 'static` where needed)
- [ ] Trait objects use appropriate constraints
- [ ] Resource cleanup is handled through RAII patterns

### 10.2 Async Patterns Checklist

- [ ] `#[async_trait]` is used for trait objects with async methods
- [ ] Timeout handling is explicit and appropriate
- [ ] No blocking operations in async context
- [ ] Proper use of `tokio::spawn` vs direct `.await`
- [ ] Cancellation is handled where appropriate
- [ ] Resource sharing uses `Arc<T>` appropriately
- [ ] Channel usage follows established patterns

### 10.3 Architecture Checklist

- [ ] Follows dependency injection patterns
- [ ] Proper separation of concerns
- [ ] Uses established error hierarchy
- [ ] Follows module organization standards
- [ ] Configuration is validated appropriately
- [ ] Metrics and logging are properly integrated
- [ ] Security considerations are addressed

### 10.4 Documentation Checklist

- [ ] All public items have rustdoc comments
- [ ] Examples are provided for complex APIs
- [ ] Error conditions are documented
- [ ] Safety requirements are documented for unsafe code
- [ ] Module-level documentation explains purpose and usage
- [ ] Links to related functionality are included

### 10.5 Testing Checklist

- [ ] Unit tests cover all public functionality
- [ ] Error conditions are tested
- [ ] Async functionality is properly tested
- [ ] Integration tests cover component interactions
- [ ] Property-based tests for complex logic
- [ ] Mock objects are used appropriately
- [ ] Performance-critical paths have benchmarks

---

## 11. Performance & Security

### 11.1 Memory Management

```rust
// ✅ Good - Use Arc for shared ownership
pub struct SharedResource {
    data: Arc<RwLock<HashMap<String, Value>>>,
    metrics: Arc<Metrics>,
}

// ✅ Good - Use appropriate collection types
use dashmap::DashMap; // For concurrent access
use indexmap::IndexMap; // When insertion order matters
use smallvec::SmallVec; // For small collections

// ✅ Good - Minimize allocations
pub fn process_items(items: &[Item]) -> Vec<ProcessedItem> {
    let mut results = Vec::with_capacity(items.len());
    for item in items {
        results.push(process_item(item));
    }
    results
}

// ❌ Bad - Unnecessary cloning
pub fn bad_processing(items: Vec<Item>) -> Vec<ProcessedItem> {
    items.into_iter()
        .map(|item| process_item(&item.clone())) // Unnecessary clone
        .collect()
}
```

### 11.2 Security Practices

**Core Security Implementation Requirements:**

```rust
// ✅ Good - Input validation at boundaries with comprehensive security checks
pub async fn execute_tool(
    tool_id: &ToolId,
    params: Value,
    auth_context: &AuthenticationContext,
) -> Result<Value, ToolError> {
    // Multi-layer security validation
    
    // 1. Authentication and authorization
    let user_id = auth_context.verify_authentication()?;
    self.rbac_engine.check_permission(&user_id, "tools", &tool_id.to_string()).await?;
    
    // 2. Input validation and sanitization
    if !tool_id.is_valid() {
        self.audit_logger.log_security_event(SecurityEvent {
            event_type: "InvalidToolId",
            user_id: user_id.clone(),
            details: "Attempted execution with invalid tool ID",
            severity: SeverityLevel::Medium,
        }).await;
        return Err(ToolError::InvalidToolId);
    }
    
    // 3. Schema validation with XSS/injection protection
    let tool = self.tools.get(tool_id)
        .ok_or(ToolError::ToolNotFound)?;
    
    let schema = tool.schema();
    let sanitized_params = security_sanitize_input(&params)?;
    schema.validate(&sanitized_params)?;
    
    // 4. Rate limiting and resource protection
    self.rate_limiter.check_rate(&user_id, tool_id).await?;
    
    // 5. Execution with security monitoring
    let start_time = SystemTime::now();
    let result = timeout(
        tool.execution_timeout(),
        tool.execute_secure(sanitized_params, auth_context),
    ).await??;
    
    // 6. Audit logging
    self.audit_logger.log_tool_execution(ToolExecutionEvent {
        user_id,
        tool_id: tool_id.clone(),
        execution_time: start_time.elapsed().unwrap(),
        success: true,
        result_hash: calculate_result_hash(&result),
    }).await;
    
    Ok(result)
}

// ✅ Good - Secure error handling with audit trail
impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityError::AuthenticationFailed => {
                // Log security event without exposing details
                audit_security_error("authentication_failed", None);
                write!(f, "Authentication failed")
            },
            SecurityError::InsufficientPermissions => {
                write!(f, "Insufficient permissions")
            },
            SecurityError::MFARequired => {
                write!(f, "Multi-factor authentication required")
            },
            SecurityError::CertificateExpired => {
                write!(f, "Certificate expired")
            },
            SecurityError::SuspiciousActivity => {
                write!(f, "Suspicious activity detected")
            },
            // Don't expose internal details
            SecurityError::InternalError(_) => {
                write!(f, "Internal security error")
            },
        }
    }
}

// ✅ Good - mTLS certificate management
pub struct SecureCertificateManager {
    vault_client: Arc<VaultClient>,
    cert_cache: Arc<RwLock<HashMap<String, CertificateBundle>>>,
    rotation_scheduler: Arc<CertificateRotationScheduler>,
}

impl SecureCertificateManager {
    pub async fn get_certificate(&self, domain: &str) -> Result<CertificateBundle, CertError> {
        // Implement certificate retrieval with cache-first strategy
        if let Some(cert) = self.cert_cache.read().await.get(domain) {
            if !cert.is_expired() && !cert.is_revoked().await? {
                return Ok(cert.clone());
            }
        }
        
        // Fetch from Vault with OCSP validation
        let cert = self.vault_client.get_certificate(domain).await?;
        self.validate_certificate_chain(&cert).await?;
        
        self.cert_cache.write().await.insert(domain.to_string(), cert.clone());
        Ok(cert)
    }
    
    pub async fn rotate_certificate(&self, domain: &str) -> Result<(), CertError> {
        // Zero-downtime certificate rotation with validation
        let new_cert = self.vault_client.request_new_certificate(domain).await?;
        self.validate_certificate_chain(&new_cert).await?;
        
        // Atomic update with rollback capability
        let old_cert = self.cert_cache.write().await
            .insert(domain.to_string(), new_cert.clone());
        
        // Verify new certificate is working
        if let Err(e) = self.test_certificate_connectivity(domain, &new_cert).await {
            // Rollback on failure
            if let Some(old) = old_cert {
                self.cert_cache.write().await.insert(domain.to_string(), old);
            }
            return Err(e);
        }
        
        Ok(())
    }
}

// ✅ Good - RBAC implementation with audit
pub struct RBACEngine {
    roles: Arc<RwLock<HashMap<String, Role>>>,
    user_roles: Arc<RwLock<HashMap<String, Vec<String>>>>,
    policy_evaluator: Arc<PolicyEvaluator>,
    audit_logger: Arc<TamperProofAuditLogger>,
}

impl RBACEngine {
    pub async fn check_permission(&self, user_id: &str, resource: &str, action: &str) -> bool {
        let user_roles = self.get_user_roles(user_id).await;
        let effective_permissions = self.calculate_effective_permissions(&user_roles).await;
        
        let permission_check = Permission {
            resource: resource.to_string(),
            action: action.to_string(),
            conditions: None,
        };
        
        let has_permission = effective_permissions.contains(&permission_check);
        
        // Comprehensive audit logging
        self.audit_logger.log_access_attempt(AccessAttempt {
            user_id: user_id.to_string(),
            resource: resource.to_string(),
            action: action.to_string(),
            granted: has_permission,
            timestamp: SystemTime::now(),
            source_ip: self.get_source_ip().await,
            user_agent: self.get_user_agent().await,
            session_id: self.get_session_id().await,
        }).await;
        
        has_permission
    }
}

// ✅ Good - Tamper-proof audit logging
pub struct TamperProofAuditLogger {
    log_chain: Arc<RwLock<Vec<AuditLogEntry>>>,
    merkle_tree: Arc<RwLock<MerkleTree<Sha256>>>,
    encryption_key: Arc<EncryptionKey>,
    storage_backend: Arc<dyn AuditStorage>,
}

impl TamperProofAuditLogger {
    pub async fn log_audit_event(&self, event: AuditEvent) -> Result<String, AuditError> {
        let mut chain = self.log_chain.write().await;
        
        let previous_hash = chain.last()
            .map(|entry| entry.hash.clone())
            .unwrap_or_else(|| "genesis".to_string());
        
        let entry = AuditLogEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            user_id: event.user_id,
            action: event.action,
            resource: event.resource,
            outcome: event.outcome,
            metadata: event.metadata,
            hash: String::new(), // To be calculated
            previous_hash,
        };
        
        // Calculate cryptographic hash including previous hash for chain integrity
        let entry_hash = self.calculate_entry_hash(&entry);
        let mut entry_with_hash = entry;
        entry_with_hash.hash = entry_hash;
        
        // Encrypt sensitive data before storage
        let encrypted_entry = self.encrypt_audit_entry(&entry_with_hash).await?;
        
        // Add to Merkle tree for additional integrity verification
        self.merkle_tree.write().await.push(entry_with_hash.hash.clone());
        
        // Store in persistent backend with redundancy
        self.storage_backend.store_audit_entry(&encrypted_entry).await?;
        
        chain.push(entry_with_hash.clone());
        Ok(entry_with_hash.id)
    }
}

// ✅ Good - Security event correlation
pub struct SecurityEventCorrelator {
    event_buffer: Arc<RwLock<VecDeque<SecurityEvent>>>,
    correlation_rules: Arc<RwLock<Vec<CorrelationRule>>>,
    alert_dispatcher: broadcast::Sender<SecurityAlert>,
    ml_analyzer: Arc<MLAnomalyDetector>,
}

impl SecurityEventCorrelator {
    pub async fn process_event(&self, event: SecurityEvent) {
        // Add to sliding window buffer
        let mut buffer = self.event_buffer.write().await;
        buffer.push_back(event.clone());
        
        // Maintain memory limits
        while buffer.len() > 10000 {
            buffer.pop_front();
        }
        drop(buffer);
        
        // Run correlation analysis
        let correlations = self.analyze_correlations(&event).await;
        
        // ML-based anomaly detection
        let anomaly_score = self.ml_analyzer.analyze_event(&event).await;
        
        // Generate alerts for significant findings
        if anomaly_score > 0.8 || !correlations.is_empty() {
            let alert = SecurityAlert {
                alert_id: Uuid::new_v4().to_string(),
                alert_type: if anomaly_score > 0.8 { 
                    AlertType::AnomalousActivity 
                } else { 
                    AlertType::CorrelatedThreat 
                },
                severity: self.calculate_severity(anomaly_score, &correlations),
                events: vec![event],
                recommended_actions: self.get_recommended_actions(&correlations).await,
                timestamp: SystemTime::now(),
            };
            
            let _ = self.alert_dispatcher.send(alert);
        }
    }
}
```

**Security Requirements:**

1. **Authentication & Authorization**
   - Token-based authentication with expiration
   - Role-based access control (RBAC) for agent permissions
   - Certificate validation for agent-to-agent communication

2. **Data Protection**
   - TLS 1.3 for all network communications
   - Encrypt sensitive configuration at rest
   - Secure credential storage in memory

3. **Audit & Monitoring**
   - Log all tool executions with user context
   - Tamper-resistant audit trail
   - Rate limiting on API endpoints

4. **Input Validation**
   - Validate all external inputs against schemas
   - Sanitize tool parameters
   - Enforce resource limits (memory, CPU, execution time)

### 11.3 Resource Management

```rust
// ✅ Good - RAII resource management
pub struct ResourceManager<R: Resource> {
    pool: Arc<Mutex<VecDeque<R>>>,
    config: ResourceConfig,
}

impl<R: Resource> ResourceManager<R> {
    pub async fn acquire(&self) -> Result<ResourceGuard<R>, ResourceError> {
        let resource = self.pool.lock().await.pop_front()
            .ok_or(ResourceError::PoolExhausted)?;
        
        Ok(ResourceGuard::new(resource, self.pool.clone()))
    }
}

pub struct ResourceGuard<R: Resource> {
    resource: Option<R>,
    pool: Arc<Mutex<VecDeque<R>>>,
}

impl<R: Resource> Drop for ResourceGuard<R> {
    fn drop(&mut self) {
        if let Some(resource) = self.resource.take() {
            if let Ok(mut pool) = self.pool.try_lock() {
                pool.push_back(resource);
            }
        }
    }
}

impl<R: Resource> Deref for ResourceGuard<R> {
    type Target = R;
    
    fn deref(&self) -> &Self::Target {
        self.resource.as_ref().unwrap()
    }
}
```

---

## 12. Agent Implementation Examples

### 12.1 Agent Implementation Template

```rust
use crate::{
    actors::{Actor, ActorRef, ActorResult},
    tools::{Tool, ToolRegistry},
    supervision::{SupervisionStrategy, Supervisor},
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Example agent implementation
pub struct ExampleAgent {
    id: AgentId,
    tool_registry: Arc<ToolRegistry>,
    state: Arc<RwLock<AgentState>>,
}

impl ExampleAgent {
    pub fn new(id: AgentId, tool_registry: Arc<ToolRegistry>) -> Self {
        Self {
            id,
            tool_registry,
            state: Arc::new(RwLock::new(AgentState::Initializing)),
        }
    }
    
    /// Execute a tool with proper error handling
    async fn execute_tool(&self, tool_id: &str, params: Value) -> AgentResult<Value> {
        // Validate tool exists
        let tool = self.tool_registry
            .get_tool(tool_id)
            .ok_or_else(|| AgentError::ToolNotFound(tool_id.to_string()))?;
        
        // Check permissions
        if !self.has_permission(tool_id).await? {
            return Err(AgentError::PermissionDenied);
        }
        
        // Execute with timeout
        let result = timeout(
            Duration::from_secs(30),
            tool.execute(params)
        ).await
        .map_err(|_| AgentError::ToolTimeout)?;
        
        result
    }
}

#[async_trait]
impl Actor for ExampleAgent {
    type Message = AgentMessage;
    type Error = AgentError;
    
    async fn handle_message(&mut self, msg: Self::Message) -> ActorResult<()> {
        match msg {
            AgentMessage::ExecuteTool { tool_id, params, respond_to } => {
                let result = self.execute_tool(&tool_id, params).await;
                let _ = respond_to.send(result);
            }
            AgentMessage::Shutdown => {
                self.shutdown().await?;
            }
        }
        Ok(())
    }
}
```

### 12.2 Tool Implementation Template

```rust
#[async_trait]
impl Tool for CustomTool {
    fn id(&self) -> &str {
        "custom_tool"
    }
    
    fn schema(&self) -> &ToolSchema {
        &self.schema
    }
    
    async fn execute(
        &self,
        params: Value,
        context: &ExecutionContext,
    ) -> Result<Value, ToolError> {
        // Validate parameters against schema
        self.schema.validate(&params)?;
        
        // Extract typed parameters
        let input: CustomInput = serde_json::from_value(params)
            .map_err(|e| ToolError::InvalidParams(e.to_string()))?;
        
        // Check rate limits
        self.rate_limiter.check(context.user_id()).await?;
        
        // Execute with monitoring
        let start = Instant::now();
        let result = self.execute_internal(input).await?;
        
        // Record metrics
        context.metrics().record_execution(
            self.id(),
            start.elapsed(),
            result.is_ok()
        );
        
        Ok(serde_json::to_value(result)?)
    }
}
```

### 12.3 Error Definition Template

```rust
use thiserror::Error;

/// Module-specific error types
#[derive(Debug, Error)]
pub enum ModuleError {
    /// Configuration validation failed
    #[error("Invalid configuration: {field} = {value}")]
    InvalidConfiguration {
        field: String,
        value: String,
    },
    
    /// Component initialization failed
    #[error("Initialization failed: {reason}")]
    InitializationFailed {
        reason: String,
    },
    
    /// Operation timeout
    #[error("Operation timed out after {timeout_ms}ms")]
    Timeout {
        timeout_ms: u64,
    },
    
    /// Conversion from external error
    #[error("I/O error occurred")]
    IoError(#[from] std::io::Error),
}

/// Convenient result type for this module
pub type ModuleResult<T> = Result<T, ModuleError>;
```

### 12.4 Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::{assert_ready, assert_pending};
    
    fn create_test_config() -> ModuleConfig {
        ModuleConfig::builder()
            .setting1("test_value")
            .setting2(42)
            .build()
    }
    
    #[tokio::test]
    async fn test_module_creation_succeeds() {
        let config = create_test_config();
        let module = ModuleManager::new(config);
        assert!(module.is_ok());
    }
    
    #[tokio::test]
    async fn test_module_initialization() {
        let config = create_test_config();
        let mut module = ModuleManager::new(config).unwrap();
        
        let result = module.initialize().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_module_handles_invalid_config() {
        let config = ModuleConfig::builder()
            .setting2(0) // Invalid value
            .build();
        
        let result = ModuleManager::new(config);
        assert!(matches!(
            result,
            Err(ModuleError::InvalidConfiguration { .. })
        ));
    }
}
```

---

## 13. Agent Implementation Checklist

### Agent Development Checklist

- [ ] **Architecture**
  - [ ] Implement Actor trait for message handling
  - [ ] Define message types with proper error variants
  - [ ] Set up supervision strategy
  - [ ] Configure health checks

- [ ] **Error Handling**
  - [ ] Define agent-specific error types
  - [ ] No `unwrap()` or `panic!()` in code
  - [ ] Proper error propagation with `?`
  - [ ] Timeout handling for all async operations

- [ ] **Resource Management**
  - [ ] Use Arc for shared state
  - [ ] Implement Drop for cleanup
  - [ ] Rate limiting on tool executions
  - [ ] Memory limits enforced

- [ ] **Security**
  - [ ] Input validation on all external data
  - [ ] Permission checks before tool execution
  - [ ] Audit logging for all actions
  - [ ] Secure credential handling

- [ ] **Testing**
  - [ ] Unit tests for all public methods
  - [ ] Integration tests for actor communication
  - [ ] Mock tool registry for testing
  - [ ] Error condition coverage

- [ ] **Documentation**
  - [ ] Rustdoc on all public APIs
  - [ ] Examples in documentation
  - [ ] Error conditions documented
  - [ ] Configuration options explained

### Code Review Checklist

- [ ] **Rust Best Practices**
  - [ ] Proper use of lifetimes and borrowing
  - [ ] No unnecessary clones
  - [ ] Efficient data structures chosen
  - [ ] Const/static used appropriately

- [ ] **Async Safety**
  - [ ] No blocking operations in async context
  - [ ] Proper cancellation handling
  - [ ] Resource cleanup on drop
  - [ ] Timeout on all network operations

- [ ] **Agent Patterns**
  - [ ] Message passing instead of shared mutable state
  - [ ] Supervision hierarchy properly configured
  - [ ] Tools registered with appropriate permissions
  - [ ] Health checks return accurate status
