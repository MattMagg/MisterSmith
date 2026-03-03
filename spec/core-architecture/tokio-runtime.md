---
title: tokio-runtime-architecture
type: implementation
permalink: core-architecture/tokio-runtime
tags:
- '#rust #tokio #runtime #async #implementation'
---

## Tokio Runtime Architecture

**Navigation**: [Home](../../README.md) > [Core Architecture](./CLAUDE.md) > Tokio Runtime Architecture

**Quick Links**: [Async Patterns](async-patterns.md) | [Supervision Trees](supervision-trees.md) | [Component Architecture](component-architecture.md)

## Table of Contents

1. [Core Runtime Configuration](#1-tokio-runtime-architecture)
   - [1.1 Core Runtime Configuration](#11-core-runtime-configuration)
   - [1.2 Runtime Lifecycle Management](#12-runtime-lifecycle-management)
2. [Performance Tuning and Optimization](#2-performance-tuning-and-optimization)
   - [2.1 Runtime Performance Metrics](#21-runtime-performance-metrics)
   - [2.2 Performance Tuning Examples](#22-performance-tuning-examples)
3. [Multi-threaded Runtime Patterns](#3-multi-threaded-runtime-patterns)
   - [3.1 Work Stealing and Task Distribution](#31-work-stealing-and-task-distribution)
   - [3.2 Task Scheduling Strategies](#32-task-scheduling-strategies)
4. [Best Practices and Common Pitfalls](#4-best-practices-and-common-pitfalls)
   - [4.1 Runtime Configuration Best Practices](#41-runtime-configuration-best-practices)
   - [4.2 Common Pitfalls and Solutions](#42-common-pitfalls-and-solutions)

## Overview

Implementation-ready Rust specifications for the Mister Smith AI Agent Framework's Tokio runtime system.
This module provides concrete implementations for runtime configuration and lifecycle management using Tokio's async runtime.

### Key Features

- **Performance-Optimized Configurations**: Pre-configured runtime settings for different workload types
- **Advanced Threading Patterns**: Work-stealing, task distribution, and scheduling strategies
- **Comprehensive Monitoring**: Built-in metrics collection and performance monitoring
- **Best Practices Guide**: Common pitfalls and their solutions for production deployments

## Dependencies

```toml
[package]
name = "mister-smith-core"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.38", features = ["full"] }
futures = "0.3"
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
dashmap = "6.0"
num_cpus = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
metrics = "0.23"
```

## 🔍 VALIDATION STATUS

**Last Validated**: 2025-07-05  
**Validator**: Framework Documentation Team  
**Validation Score**: Pending full validation  
**Status**: Active Development  

### Implementation Status

- Runtime manager implementation complete
- Configuration structures defined
- Lifecycle management established
- Worker pool management implemented

## Core Error Types

```rust
// src/errors.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("Runtime error: {0}")]
    Runtime(#[from] RuntimeError),
    #[error("Supervision error: {0}")]
    Supervision(#[from] SupervisionError),
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigError),
    #[error("Resource error: {0}")]
    Resource(#[from] ResourceError),
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    #[error("Persistence error: {0}")]
    Persistence(#[from] PersistenceError),
    #[error("Actor system error: {0}")]
    Actor(#[from] ActorError),
    #[error("Task execution error: {0}")]
    Task(#[from] TaskError),
    #[error("Stream processing error: {0}")]
    Stream(#[from] StreamError),
    #[error("Event system error: {0}")]
    Event(#[from] EventError),
    #[error("Tool system error: {0}")]
    Tool(#[from] ToolError),
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Failed to build runtime: {0}")]
    BuildFailed(#[from] std::io::Error),
    #[error("Runtime startup failed: {0}")]
    StartupFailed(String),
    #[error("Runtime shutdown failed: {0}")]
    ShutdownFailed(String),
    #[error("Runtime configuration invalid: {0}")]
    ConfigurationInvalid(String),
}

// Error severity for system-wide error handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    Retry { max_attempts: u32, delay: std::time::Duration },
    Restart,
    Escalate,
    Reload,
    CircuitBreaker,
    Failover,
    Ignore,
}

impl SystemError {
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            SystemError::Runtime(_) => ErrorSeverity::Critical,
            SystemError::Supervision(_) => ErrorSeverity::High,
            SystemError::Configuration(_) => ErrorSeverity::Medium,
            SystemError::Resource(_) => ErrorSeverity::Medium,
            SystemError::Network(_) => ErrorSeverity::Low,
            SystemError::Persistence(_) => ErrorSeverity::High,
            SystemError::Actor(_) => ErrorSeverity::Medium,
            SystemError::Task(_) => ErrorSeverity::Low,
            SystemError::Stream(_) => ErrorSeverity::Medium,
            SystemError::Event(_) => ErrorSeverity::Low,
            SystemError::Tool(_) => ErrorSeverity::Low,
        }
    }
    
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            SystemError::Runtime(_) => RecoveryStrategy::Restart,
            SystemError::Supervision(_) => RecoveryStrategy::Escalate,
            SystemError::Configuration(_) => RecoveryStrategy::Reload,
            SystemError::Resource(_) => RecoveryStrategy::Retry { 
                max_attempts: 3, 
                delay: std::time::Duration::from_millis(1000) 
            },
            SystemError::Network(_) => RecoveryStrategy::CircuitBreaker,
            SystemError::Persistence(_) => RecoveryStrategy::Failover,
            _ => RecoveryStrategy::Retry { 
                max_attempts: 1, 
                delay: std::time::Duration::from_millis(100) 
            },
        }
    }
}
```

## 1. Tokio Runtime Architecture

### 1.1 Core Runtime Configuration

```rust
// src/core/runtime.rs
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Duration;
use tokio::runtime::Runtime;
use serde::{Serialize, Deserialize};

// Runtime constants with performance considerations
pub const DEFAULT_WORKER_THREADS: usize = num_cpus::get();
pub const DEFAULT_MAX_BLOCKING_THREADS: usize = 512;
pub const DEFAULT_THREAD_KEEP_ALIVE: Duration = Duration::from_secs(60);
pub const DEFAULT_THREAD_STACK_SIZE: usize = 2 * 1024 * 1024; // 2MB

// Performance-tuned configurations for different workloads
pub const HIGH_THROUGHPUT_WORKERS: usize = num_cpus::get() * 2;
pub const CPU_BOUND_WORKERS: usize = num_cpus::get();
pub const IO_BOUND_WORKERS: usize = num_cpus::get() * 4;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub worker_threads: Option<usize>,
    pub max_blocking_threads: usize,
    pub thread_keep_alive: Duration,
    pub thread_stack_size: Option<usize>,
    pub enable_all: bool,
    pub enable_time: bool,
    pub enable_io: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: None, // Uses Tokio default
            max_blocking_threads: DEFAULT_MAX_BLOCKING_THREADS,
            thread_keep_alive: DEFAULT_THREAD_KEEP_ALIVE,
            thread_stack_size: Some(DEFAULT_THREAD_STACK_SIZE),
            enable_all: true,
            enable_time: true,
            enable_io: true,
        }
    }
}

impl RuntimeConfig {
    /// Optimized configuration for CPU-bound workloads
    pub fn cpu_bound() -> Self {
        Self {
            worker_threads: Some(CPU_BOUND_WORKERS),
            max_blocking_threads: 128,
            thread_keep_alive: Duration::from_secs(10),
            thread_stack_size: Some(4 * 1024 * 1024), // 4MB for compute-heavy tasks
            ..Default::default()
        }
    }
    
    /// Optimized configuration for I/O-bound workloads
    pub fn io_bound() -> Self {
        Self {
            worker_threads: Some(IO_BOUND_WORKERS),
            max_blocking_threads: 1024,
            thread_keep_alive: Duration::from_secs(120),
            thread_stack_size: Some(1 * 1024 * 1024), // 1MB for lightweight tasks
            ..Default::default()
        }
    }
    
    /// High-throughput configuration for mixed workloads
    pub fn high_throughput() -> Self {
        Self {
            worker_threads: Some(HIGH_THROUGHPUT_WORKERS),
            max_blocking_threads: 768,
            thread_keep_alive: Duration::from_secs(60),
            thread_stack_size: Some(DEFAULT_THREAD_STACK_SIZE),
            ..Default::default()
        }
    }
    
    pub fn build_runtime(&self) -> Result<Runtime, RuntimeError> {
        let mut builder = tokio::runtime::Builder::new_multi_thread();
        
        if let Some(worker_threads) = self.worker_threads {
            builder.worker_threads(worker_threads);
        }
        
        builder
            .max_blocking_threads(self.max_blocking_threads)
            .thread_keep_alive(self.thread_keep_alive);
            
        if let Some(stack_size) = self.thread_stack_size {
            builder.thread_stack_size(stack_size);
        }
        
        if self.enable_all {
            builder.enable_all();
        } else {
            if self.enable_time {
                builder.enable_time();
            }
            if self.enable_io {
                builder.enable_io();
            }
        }
        
        builder.build().map_err(RuntimeError::BuildFailed)
    }
}
```

### 1.2 Runtime Lifecycle Management

```rust
// src/core/runtime.rs (continued)
use crate::supervision::SupervisionTree;
use crate::events::EventBus;
use crate::resources::health::{HealthMonitor, MetricsCollector};
use crate::errors::{RuntimeError, SystemError};
use tokio::signal;
use tokio::task::JoinHandle;
use tracing::{info, warn, error};

pub const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub struct RuntimeManager {
    runtime: Arc<Runtime>,
    shutdown_signal: Arc<AtomicBool>,
    health_monitor: Arc<HealthMonitor>,
    metrics_collector: Arc<MetricsCollector>,
    supervision_tree: Arc<SupervisionTree>,
    event_bus: Arc<EventBus>,
    tasks: Vec<JoinHandle<()>>,
}

impl RuntimeManager {
    pub fn initialize(config: RuntimeConfig) -> Result<Self, RuntimeError> {
        let runtime = Arc::new(config.build_runtime()?);
        let shutdown_signal = Arc::new(AtomicBool::new(false));
        let health_monitor = Arc::new(HealthMonitor::new());
        let metrics_collector = Arc::new(MetricsCollector::new());
        let supervision_tree = Arc::new(SupervisionTree::new());
        let event_bus = Arc::new(EventBus::new());
        
        Ok(Self {
            runtime,
            shutdown_signal,
            health_monitor,
            metrics_collector,
            supervision_tree,
            event_bus,
            tasks: Vec::new(),
        })
    }
    
    pub async fn start_system(&mut self) -> Result<(), RuntimeError> {
        info!("Starting runtime system components");
        
        // Start health monitoring
        let health_task = {
            let monitor = Arc::clone(&self.health_monitor);
            let shutdown = Arc::clone(&self.shutdown_signal);
            self.runtime.spawn(async move {
                monitor.run(shutdown).await;
            })
        };
        
        // Start metrics collection
        let metrics_task = {
            let collector = Arc::clone(&self.metrics_collector);
            let shutdown = Arc::clone(&self.shutdown_signal);
            self.runtime.spawn(async move {
                collector.run(shutdown).await;
            })
        };
        
        // Start supervision tree
        let supervision_task = {
            let tree = Arc::clone(&self.supervision_tree);
            let shutdown = Arc::clone(&self.shutdown_signal);
            self.runtime.spawn(async move {
                if let Err(e) = tree.start(shutdown).await {
                    error!("Supervision tree failed: {}", e);
                }
            })
        };
        
        // Start signal handler
        let signal_task = {
            let shutdown = Arc::clone(&self.shutdown_signal);
            self.runtime.spawn(async move {
                Self::signal_handler(shutdown).await;
            })
        };
        
        self.tasks.extend([health_task, metrics_task, supervision_task, signal_task]);
        
        info!("Runtime system started successfully");
        Ok(())
    }
    
    pub async fn graceful_shutdown(self) -> Result<(), RuntimeError> {
        info!("Initiating graceful shutdown");
        
        // Signal shutdown to all components
        self.shutdown_signal.store(true, Ordering::SeqCst);
        
        // Shutdown supervision tree first
        if let Err(e) = self.supervision_tree.shutdown().await {
            warn!("Error during supervision tree shutdown: {}", e);
        }
        
        // Flush metrics
        if let Err(e) = self.metrics_collector.flush().await {
            warn!("Error flushing metrics: {}", e);
        }
        
        // Wait for all tasks to complete
        for task in self.tasks {
            if let Err(e) = task.await {
                warn!("Task failed during shutdown: {}", e);
            }
        }
        
        // Shutdown runtime with timeout
        self.runtime.shutdown_timeout(DEFAULT_SHUTDOWN_TIMEOUT);
        
        info!("Graceful shutdown completed");
        Ok(())
    }
    
    async fn signal_handler(shutdown_signal: Arc<AtomicBool>) {
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to register SIGTERM handler");
        let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())
            .expect("Failed to register SIGINT handler");
        
        tokio::select! {
            _ = sigterm.recv() => {
                info!("Received SIGTERM, initiating shutdown");
            }
            _ = sigint.recv() => {
                info!("Received SIGINT, initiating shutdown");
            }
        }
        
        shutdown_signal.store(true, Ordering::SeqCst);
    }
    
    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }
}
```

## 2. Performance Tuning and Optimization

### 2.1 Runtime Performance Metrics

```rust
// src/core/runtime/metrics.rs
use std::sync::Arc;
use metrics::{counter, gauge, histogram};
use tokio::runtime::RuntimeMetrics;

#[derive(Debug, Clone)]
pub struct RuntimePerformanceMonitor {
    runtime_handle: tokio::runtime::Handle,
}

impl RuntimePerformanceMonitor {
    pub fn new(handle: tokio::runtime::Handle) -> Self {
        Self { runtime_handle: handle }
    }
    
    pub fn collect_metrics(&self) {
        let metrics = self.runtime_handle.metrics();
        
        // Worker thread metrics
        gauge!("runtime.workers.count", metrics.num_workers() as f64);
        gauge!("runtime.workers.blocking_count", metrics.num_blocking_threads() as f64);
        gauge!("runtime.workers.idle_count", metrics.num_idle_blocking_threads() as f64);
        
        // Task metrics
        counter!("runtime.tasks.spawned_total", metrics.spawned_tasks_count());
        gauge!("runtime.tasks.active", metrics.active_tasks_count() as f64);
        
        // Queue metrics
        gauge!("runtime.queue.local_size", metrics.local_queue_capacity() as f64);
        gauge!("runtime.queue.global_size", metrics.global_queue_depth() as f64);
        gauge!("runtime.queue.injection_size", metrics.injection_queue_depth() as f64);
        
        // Blocking metrics
        histogram!("runtime.blocking.queue_depth", metrics.blocking_queue_depth() as f64);
        
        // Park/unpark metrics
        counter!("runtime.park.count", metrics.park_count());
        counter!("runtime.park.no_work_count", metrics.noop_count());
        
        // Steal operations
        counter!("runtime.steal.operations", metrics.steal_operations());
        
        // Budget metrics
        counter!("runtime.budget.forced_yield", metrics.budget_forced_yield_count());
    }
}
```

### 2.2 Performance Tuning Examples

```rust
// src/core/runtime/tuning.rs
use crate::core::runtime::{RuntimeConfig, RuntimeManager};
use std::sync::Arc;
use std::time::Duration;

/// Performance-tuned runtime configurations for specific use cases
pub struct RuntimeTuning;

impl RuntimeTuning {
    /// Optimized for handling thousands of concurrent WebSocket connections
    pub fn websocket_server_config() -> RuntimeConfig {
        RuntimeConfig {
            worker_threads: Some(num_cpus::get() * 2),
            max_blocking_threads: 2048,
            thread_keep_alive: Duration::from_secs(180),
            thread_stack_size: Some(512 * 1024), // 512KB - small stack for many connections
            enable_all: true,
            enable_time: true,
            enable_io: true,
        }
    }
    
    /// Optimized for data processing pipelines
    pub fn data_pipeline_config() -> RuntimeConfig {
        RuntimeConfig {
            worker_threads: Some(num_cpus::get()),
            max_blocking_threads: 256,
            thread_keep_alive: Duration::from_secs(30),
            thread_stack_size: Some(8 * 1024 * 1024), // 8MB - large stack for complex processing
            enable_all: true,
            enable_time: true,
            enable_io: true,
        }
    }
    
    /// Optimized for mixed agent workloads
    pub fn agent_system_config() -> RuntimeConfig {
        RuntimeConfig {
            worker_threads: Some((num_cpus::get() as f64 * 1.5) as usize),
            max_blocking_threads: 512,
            thread_keep_alive: Duration::from_secs(60),
            thread_stack_size: Some(2 * 1024 * 1024), // 2MB - balanced for mixed workloads
            enable_all: true,
            enable_time: true,
            enable_io: true,
        }
    }
    
    /// Adaptive runtime that adjusts based on load
    pub async fn adaptive_runtime_example() -> Result<(), Box<dyn std::error::Error>> {
        let initial_config = RuntimeConfig::default();
        let mut manager = RuntimeManager::initialize(initial_config)?;
        
        // Monitor and adjust runtime parameters based on metrics
        let runtime_handle = manager.runtime().handle().clone();
        let monitor = Arc::new(RuntimePerformanceMonitor::new(runtime_handle.clone()));
        
        // Spawn monitoring task
        runtime_handle.spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                monitor.collect_metrics();
                
                // Adaptive logic based on metrics
                let metrics = runtime_handle.metrics();
                let queue_depth = metrics.injection_queue_depth();
                
                if queue_depth > 1000 {
                    tracing::warn!("High queue depth detected: {}", queue_depth);
                    // In a real system, you might trigger scaling actions here
                }
            }
        });
        
        manager.start_system().await?;
        Ok(())
    }
}
```

## 3. Multi-threaded Runtime Patterns

### 3.1 Work Stealing and Task Distribution

```rust
// src/core/runtime/patterns.rs
use tokio::task::{JoinHandle, yield_now};
use std::sync::Arc;
use std::time::Duration;

/// Demonstrates work-stealing behavior in multi-threaded runtime
pub struct WorkStealingPatterns;

impl WorkStealingPatterns {
    /// Spawn tasks across all worker threads efficiently
    pub fn distributed_spawn_pattern<F, T>(
        task_count: usize,
        task_factory: F,
    ) -> Vec<JoinHandle<T>>
    where
        F: Fn(usize) -> T + Send + Sync + 'static,
        T: Send + 'static,
    {
        let factory = Arc::new(task_factory);
        
        (0..task_count)
            .map(|idx| {
                let factory = Arc::clone(&factory);
                tokio::spawn(async move {
                    // Yield immediately to encourage distribution across threads
                    yield_now().await;
                    factory(idx)
                })
            })
            .collect()
    }
    
    /// CPU-bound work distribution pattern
    pub async fn cpu_bound_distribution_pattern() {
        let num_tasks = num_cpus::get() * 2;
        let tasks: Vec<_> = (0..num_tasks)
            .map(|task_id| {
                tokio::task::spawn_blocking(move || {
                    // Simulate CPU-bound work
                    let mut sum = 0u64;
                    for i in 0..10_000_000 {
                        sum = sum.wrapping_add(i);
                    }
                    (task_id, sum)
                })
            })
            .collect();
        
        // Wait for all tasks to complete
        for task in tasks {
            let (task_id, result) = task.await.unwrap();
            tracing::debug!("Task {} completed with result: {}", task_id, result);
        }
    }
    
    /// Mixed I/O and CPU workload pattern
    pub async fn mixed_workload_pattern() {
        let io_tasks = 100;
        let cpu_tasks = num_cpus::get();
        
        // Spawn I/O-bound tasks
        let io_handles: Vec<_> = (0..io_tasks)
            .map(|id| {
                tokio::spawn(async move {
                    // Simulate I/O operation
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    format!("IO Task {} completed", id)
                })
            })
            .collect();
        
        // Spawn CPU-bound tasks on blocking thread pool
        let cpu_handles: Vec<_> = (0..cpu_tasks)
            .map(|id| {
                tokio::task::spawn_blocking(move || {
                    // Simulate CPU-intensive work
                    std::thread::sleep(Duration::from_millis(50));
                    format!("CPU Task {} completed", id)
                })
            })
            .collect();
        
        // Await all tasks
        for handle in io_handles {
            tracing::debug!("{}", handle.await.unwrap());
        }
        
        for handle in cpu_handles {
            tracing::debug!("{}", handle.await.unwrap());
        }
    }
}
```

### 3.2 Task Scheduling Strategies

```rust
// src/core/runtime/scheduling.rs
use tokio::sync::{Semaphore, mpsc};
use std::sync::Arc;

/// Advanced task scheduling patterns for optimal performance
pub struct TaskScheduler {
    /// Limit concurrent executions
    concurrency_limiter: Arc<Semaphore>,
    /// Priority queue for tasks
    priority_queue: mpsc::Sender<PrioritizedTask>,
}

#[derive(Debug)]
struct PrioritizedTask {
    priority: u8,
    task: Box<dyn FnOnce() + Send>,
}

impl TaskScheduler {
    pub fn new(max_concurrent: usize) -> (Self, JoinHandle<()>) {
        let concurrency_limiter = Arc::new(Semaphore::new(max_concurrent));
        let (tx, mut rx) = mpsc::channel::<PrioritizedTask>(1000);
        
        let limiter = Arc::clone(&concurrency_limiter);
        let executor = tokio::spawn(async move {
            while let Some(task) = rx.recv().await {
                let permit = limiter.acquire().await.unwrap();
                tokio::spawn(async move {
                    (task.task)();
                    drop(permit); // Release semaphore
                });
            }
        });
        
        (Self {
            concurrency_limiter,
            priority_queue: tx,
        }, executor)
    }
    
    /// Batch processing pattern for improved throughput
    pub async fn batch_processing_pattern<T, F>(
        items: Vec<T>,
        batch_size: usize,
        processor: F,
    ) where
        T: Send + 'static,
        F: Fn(Vec<T>) -> Vec<T> + Send + Sync + 'static,
    {
        let processor = Arc::new(processor);
        let mut handles = Vec::new();
        
        for batch in items.chunks(batch_size) {
            let batch = batch.to_vec();
            let processor = Arc::clone(&processor);
            
            let handle = tokio::spawn(async move {
                // Process batch in blocking thread to avoid blocking runtime
                tokio::task::spawn_blocking(move || processor(batch))
                    .await
                    .unwrap()
            });
            
            handles.push(handle);
        }
        
        // Collect results
        for handle in handles {
            let _results = handle.await.unwrap();
        }
    }
    
    /// Fan-out/fan-in pattern for parallel processing
    pub async fn fanout_fanin_pattern<T, R, F>(
        input: Vec<T>,
        worker_count: usize,
        processor: F,
    ) -> Vec<R>
    where
        T: Send + 'static,
        R: Send + 'static,
        F: Fn(T) -> R + Send + Sync + 'static,
    {
        let (tx, mut rx) = mpsc::channel(100);
        let processor = Arc::new(processor);
        
        // Create a shared queue for work distribution
        let work_queue = Arc::new(tokio::sync::Mutex::new(input.into_iter()));
        
        // Create worker tasks
        let workers: Vec<_> = (0..worker_count)
            .map(|_| {
                let queue = Arc::clone(&work_queue);
                let processor = Arc::clone(&processor);
                
                tokio::spawn(async move {
                    let mut results = Vec::new();
                    loop {
                        let item = {
                            let mut queue = queue.lock().await;
                            queue.next()
                        };
                        
                        match item {
                            Some(item) => results.push(processor(item)),
                            None => break,
                        }
                    }
                    results
                })
            })
            .collect();
        
        // Collect results from all workers
        let mut all_results = Vec::new();
        for worker in workers {
            all_results.extend(worker.await.unwrap());
        }
        
        all_results
    }
}
```

## 4. Best Practices and Common Pitfalls

### 4.1 Runtime Configuration Best Practices

```rust
// src/core/runtime/best_practices.rs

/// Best practices for runtime configuration
pub struct RuntimeBestPractices;

impl RuntimeBestPractices {
    /// Choose appropriate worker thread count based on workload
    pub fn optimal_worker_threads(workload_type: WorkloadType) -> usize {
        match workload_type {
            WorkloadType::CpuBound => num_cpus::get(),
            WorkloadType::IoBound => num_cpus::get() * 4,
            WorkloadType::Mixed => (num_cpus::get() as f64 * 1.5) as usize,
            WorkloadType::LatencySensitive => num_cpus::get() * 2,
        }
    }
    
    /// Avoid common blocking pitfalls
    pub async fn avoid_blocking_runtime() {
        // BAD: Blocking the runtime thread
        // std::thread::sleep(Duration::from_secs(1));
        
        // GOOD: Use async sleep
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // BAD: CPU-intensive work on runtime thread
        // let result = expensive_computation();
        
        // GOOD: Move to blocking thread pool
        let result = tokio::task::spawn_blocking(|| {
            expensive_computation()
        }).await.unwrap();
    }
    
    /// Proper resource cleanup patterns
    pub async fn resource_cleanup_pattern() {
        // Use RAII and Drop implementations
        struct RuntimeResource {
            handle: tokio::runtime::Handle,
        }
        
        impl Drop for RuntimeResource {
            fn drop(&mut self) {
                // Cleanup logic here
                tracing::debug!("Cleaning up runtime resource");
            }
        }
        
        // Resources are automatically cleaned up when going out of scope
        {
            let _resource = RuntimeResource {
                handle: tokio::runtime::Handle::current(),
            };
            // Use resource
        } // Automatic cleanup here
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WorkloadType {
    CpuBound,
    IoBound,
    Mixed,
    LatencySensitive,
}

fn expensive_computation() -> u64 {
    // Placeholder for expensive computation
    42
}
```

### 4.2 Common Pitfalls and Solutions

```rust
// src/core/runtime/pitfalls.rs

/// Common runtime pitfalls and their solutions
pub struct RuntimePitfalls;

impl RuntimePitfalls {
    /// Pitfall: Creating multiple runtimes unnecessarily
    pub fn single_runtime_pattern() {
        // BAD: Creating runtime for each operation
        // for _ in 0..10 {
        //     let rt = tokio::runtime::Runtime::new().unwrap();
        //     rt.block_on(async { /* work */ });
        // }
        
        // GOOD: Reuse a single runtime
        let rt = tokio::runtime::Runtime::new().unwrap();
        for _ in 0..10 {
            rt.block_on(async { /* work */ });
        }
    }
    
    /// Pitfall: Blocking runtime threads with synchronous I/O
    pub async fn async_io_pattern() {
        // BAD: Using std::fs in async context
        // let contents = std::fs::read_to_string("file.txt").unwrap();
        
        // GOOD: Use tokio::fs for async I/O
        let contents = tokio::fs::read_to_string("file.txt").await.unwrap();
        
        // GOOD: Or move blocking I/O to blocking thread pool
        let contents = tokio::task::spawn_blocking(|| {
            std::fs::read_to_string("file.txt")
        }).await.unwrap().unwrap();
    }
    
    /// Pitfall: Unbounded task spawning
    pub async fn bounded_concurrency_pattern() {
        // BAD: Spawning unlimited tasks
        // for i in 0..1_000_000 {
        //     tokio::spawn(async move { process(i).await });
        // }
        
        // GOOD: Use semaphore to limit concurrency
        let semaphore = Arc::new(Semaphore::new(100));
        let mut handles = vec![];
        
        for i in 0..1_000_000 {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let handle = tokio::spawn(async move {
                let _permit = permit; // Hold permit until task completes
                process(i).await
            });
            handles.push(handle);
        }
        
        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }
    }
    
    /// Pitfall: Not handling panics in spawned tasks
    pub async fn panic_handling_pattern() {
        // BAD: Unhandled panics in spawned tasks
        // tokio::spawn(async {
        //     panic!("Task panicked!");
        // });
        
        // GOOD: Handle panics gracefully
        let handle = tokio::spawn(async {
            // Task that might panic
            risky_operation().await
        });
        
        match handle.await {
            Ok(result) => tracing::info!("Task completed: {:?}", result),
            Err(e) if e.is_panic() => {
                tracing::error!("Task panicked: {:?}", e);
                // Handle panic recovery
            }
            Err(e) => tracing::error!("Task failed: {:?}", e),
        }
    }
}

async fn process(_i: usize) {
    // Placeholder for processing logic
    tokio::time::sleep(Duration::from_millis(10)).await;
}

async fn risky_operation() -> Result<(), Box<dyn std::error::Error>> {
    // Placeholder for operation that might fail
    Ok(())
}

use tokio::sync::Semaphore;
use std::sync::Arc;
use std::time::Duration;
```

## External Dependencies

This module references the following components from other modules in the framework:

- `crate::supervision::SupervisionTree` - From [supervision-trees.md](supervision-trees.md)
- `crate::events::EventBus` - From event-system.md *(requires extraction)*
- `crate::resources::health::{HealthMonitor, MetricsCollector}` - From resource-management.md *(requires extraction)*
- Additional error types (`SupervisionError`, `ConfigError`, etc.) - From their respective modules

## Cross-References

### Internal Framework References

- **Async Patterns**: [async-patterns.md](async-patterns.md) - Async programming patterns used with this runtime
- **Supervision Trees**: [supervision-trees.md](supervision-trees.md) - Supervision tree integration
- **Component Architecture**: [component-architecture.md](component-architecture.md) - How runtime fits into overall component design
- **System Integration**: [system-integration.md](system-integration.md) - Runtime integration with other system components

### Related Architecture Documents

- **System Architecture**: [system-architecture.md](system-architecture.md) - Complete system design overview
- **Type Definitions**: [type-definitions.md](type-definitions.md) - Core type system used by runtime
- **Integration Patterns**: [integration-patterns.md](./integration-patterns.md) - Patterns for integrating with runtime

## Navigation

- **Up**: [Core Architecture](./CLAUDE.md)
- **Previous**: [Component Architecture](component-architecture.md)
- **Next**: [Async Patterns](async-patterns.md)
- **Related**: [Supervision Trees](supervision-trees.md)

---

*Extracted from system-architecture.md - Mister Smith AI Agent Framework*
