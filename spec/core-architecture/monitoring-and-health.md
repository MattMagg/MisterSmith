# Monitoring and Health Architecture

**Framework Documentation > Core Architecture > Monitoring and Health**

**Quick Links**: [Supervision & Events](supervision-and-events.md) | [System Architecture](system-architecture.md) | [Runtime & Errors](runtime-and-errors.md) | [Implementation Guidelines](implementation-guidelines.md)

---

## Navigation

[← Supervision & Events](./supervision-and-events.md) | [System Architecture Overview](./system-architecture.md) | [Implementation Guidelines →](./implementation-guidelines.md)

---

## 🔍 VALIDATION STATUS

**Last Validated**: 2025-07-07  
**Validator**: Agent 1 - Team Alpha  
**Component**: Health Monitoring and Metrics  
**Status**: Basic Implementation  

### Implementation Status

- ✅ Health check system with async trait pattern
- ✅ Basic metrics collection framework
- ✅ Event integration for health status
- ⚠️ Production metrics backends (Prometheus, OpenTelemetry) need integration
- ⚠️ Advanced health check patterns need implementation

---

## Table of Contents

1. [Health Check System](#health-check-system)
   - [Core Health Architecture](#core-health-architecture)
   - [Health Check Implementations](#health-check-implementations)
2. [Metrics Collection](#metrics-collection)
   - [Metrics Collector](#metrics-collector)
   - [Metric Types](#metric-types)
3. [Integration with System Components](#integration-with-system-components)

---

## Health Check System

❌ **VALIDATION ALERT**: Health monitoring was referenced but never implemented. This is critical for system observability.

### Core Health Architecture

```rust
// src/health/monitor.rs
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::sync::RwLock;
use std::collections::HashMap;
use async_trait::async_trait;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use crate::events::{EventBus, EventBuilder, EventType, SystemEventType};
use crate::runtime::RuntimeManager;
use crate::metrics::MetricsCollector;
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub component_id: ComponentId,
    pub status: Status,
    pub last_check: Instant,
    pub message: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Status {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> Result<HealthStatus, Box<dyn std::error::Error>>;
    fn component_id(&self) -> ComponentId;
    fn check_interval(&self) -> Duration;
}

pub struct HealthMonitor {
    check_interval: Duration,
    health_checks: Arc<RwLock<Vec<Box<dyn HealthCheck>>>>,
    status_cache: Arc<RwLock<HashMap<ComponentId, HealthStatus>>>,
    event_bus: Option<Arc<EventBus>>,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            health_checks: Arc::new(RwLock::new(Vec::new())),
            status_cache: Arc::new(RwLock::new(HashMap::new())),
            event_bus: None,
        }
    }
    
    pub fn with_event_bus(mut self, event_bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }
    
    pub async fn register_check(&self, check: Box<dyn HealthCheck>) {
        let mut checks = self.health_checks.write().await;
        checks.push(check);
    }
    
    pub async fn run(&self, shutdown_signal: Arc<AtomicBool>) {
        while !shutdown_signal.load(Ordering::Relaxed) {
            self.perform_health_checks().await;
            tokio::time::sleep(self.check_interval).await;
        }
    }
    
    async fn perform_health_checks(&self) {
        let checks = self.health_checks.read().await;
        
        for check in checks.iter() {
            let component_id = check.component_id();
            
            match check.check().await {
                Ok(status) => {
                    self.update_status(status.clone()).await;
                    
                    // Publish health event if status changed
                    if let Some(event_bus) = &self.event_bus {
                        let event_type = match status.status {
                            Status::Healthy => SystemEventType::HealthCheckPassed,
                            _ => SystemEventType::HealthCheckFailed,
                        };
                        
                        let event = EventBuilder::new(
                            component_id.clone(),
                            EventType::System(event_type)
                        )
                        .with_payload(&status)
                        .unwrap()
                        .build();
                        
                        let _ = event_bus.publish(event).await;
                    }
                }
                Err(e) => {
                    error!("Health check failed for {}: {}", component_id.0, e);
                    
                    let status = HealthStatus {
                        component_id: component_id.clone(),
                        status: Status::Unknown,
                        last_check: Instant::now(),
                        message: Some(format!("Check failed: {}", e)),
                        metadata: HashMap::new(),
                    };
                    
                    self.update_status(status).await;
                }
            }
        }
    }
    
    async fn update_status(&self, status: HealthStatus) {
        let mut cache = self.status_cache.write().await;
        cache.insert(status.component_id.clone(), status);
    }
    
    pub async fn get_status(&self, component_id: &ComponentId) -> Option<HealthStatus> {
        let cache = self.status_cache.read().await;
        cache.get(component_id).cloned()
    }
    
    pub async fn get_all_statuses(&self) -> HashMap<ComponentId, HealthStatus> {
        let cache = self.status_cache.read().await;
        cache.clone()
    }
    
    pub async fn is_system_healthy(&self) -> bool {
        let cache = self.status_cache.read().await;
        cache.values().all(|status| status.status == Status::Healthy)
    }
}
```

### Health Check Implementations

```rust
// Example health check implementation
pub struct RuntimeHealthCheck {
    component_id: ComponentId,
    runtime_manager: Arc<RuntimeManager>,
}

impl RuntimeHealthCheck {
    pub fn new(runtime_manager: Arc<RuntimeManager>) -> Self {
        Self {
            component_id: ComponentId("runtime".to_string()),
            runtime_manager,
        }
    }
}

#[async_trait]
impl HealthCheck for RuntimeHealthCheck {
    async fn check(&self) -> Result<HealthStatus, Box<dyn std::error::Error>> {
        // Check if runtime is still responsive
        let check_future = tokio::time::timeout(
            Duration::from_secs(5),
            async { 
                // Simple responsiveness check
                tokio::task::yield_now().await;
                Ok(())
            }
        );
        
        match check_future.await {
            Ok(_) => Ok(HealthStatus {
                component_id: self.component_id.clone(),
                status: Status::Healthy,
                last_check: Instant::now(),
                message: None,
                metadata: HashMap::new(),
            }),
            Err(_) => Ok(HealthStatus {
                component_id: self.component_id.clone(),
                status: Status::Unhealthy,
                last_check: Instant::now(),
                message: Some("Runtime unresponsive".to_string()),
                metadata: HashMap::new(),
            }),
        }
    }
    
    fn component_id(&self) -> ComponentId {
        self.component_id.clone()
    }
    
    fn check_interval(&self) -> Duration {
        Duration::from_secs(10)
    }
}

// Database connection health check
pub struct DatabaseHealthCheck {
    component_id: ComponentId,
    connection_pool: Arc<DatabasePool>,
}

impl DatabaseHealthCheck {
    pub fn new(connection_pool: Arc<DatabasePool>) -> Self {
        Self {
            component_id: ComponentId("database".to_string()),
            connection_pool,
        }
    }
}

#[async_trait]
impl HealthCheck for DatabaseHealthCheck {
    async fn check(&self) -> Result<HealthStatus, Box<dyn std::error::Error>> {
        let start = Instant::now();
        
        // Try to acquire a connection and run a simple query
        match self.connection_pool.acquire().await {
            Ok(conn) => {
                match conn.execute("SELECT 1").await {
                    Ok(_) => {
                        let latency = start.elapsed();
                        let mut metadata = HashMap::new();
                        metadata.insert(
                            "latency_ms".to_string(),
                            serde_json::json!(latency.as_millis())
                        );
                        metadata.insert(
                            "pool_size".to_string(),
                            serde_json::json!(self.connection_pool.size())
                        );
                        
                        Ok(HealthStatus {
                            component_id: self.component_id.clone(),
                            status: if latency.as_millis() > 100 {
                                Status::Degraded
                            } else {
                                Status::Healthy
                            },
                            last_check: Instant::now(),
                            message: None,
                            metadata,
                        })
                    }
                    Err(e) => Ok(HealthStatus {
                        component_id: self.component_id.clone(),
                        status: Status::Unhealthy,
                        last_check: Instant::now(),
                        message: Some(format!("Query failed: {}", e)),
                        metadata: HashMap::new(),
                    }),
                }
            }
            Err(e) => Ok(HealthStatus {
                component_id: self.component_id.clone(),
                status: Status::Unhealthy,
                last_check: Instant::now(),
                message: Some(format!("Connection failed: {}", e)),
                metadata: HashMap::new(),
            }),
        }
    }
    
    fn component_id(&self) -> ComponentId {
        self.component_id.clone()
    }
    
    fn check_interval(&self) -> Duration {
        Duration::from_secs(30)
    }
}

// Agent system health check
pub struct AgentSystemHealthCheck {
    component_id: ComponentId,
    actor_system: Arc<ActorSystem>,
}

impl AgentSystemHealthCheck {
    pub fn new(actor_system: Arc<ActorSystem>) -> Self {
        Self {
            component_id: ComponentId("agent_system".to_string()),
            actor_system,
        }
    }
}

#[async_trait]
impl HealthCheck for AgentSystemHealthCheck {
    async fn check(&self) -> Result<HealthStatus, Box<dyn std::error::Error>> {
        let mut metadata = HashMap::new();
        
        // Get actor count and other metrics
        let actor_count = self.actor_system.active_actor_count().await;
        metadata.insert("actor_count".to_string(), serde_json::json!(actor_count));
        
        // Check if we can spawn a test actor
        let test_spawn = tokio::time::timeout(
            Duration::from_secs(2),
            self.actor_system.spawn_test_actor()
        ).await;
        
        match test_spawn {
            Ok(Ok(_)) => Ok(HealthStatus {
                component_id: self.component_id.clone(),
                status: Status::Healthy,
                last_check: Instant::now(),
                message: None,
                metadata,
            }),
            Ok(Err(e)) => Ok(HealthStatus {
                component_id: self.component_id.clone(),
                status: Status::Degraded,
                last_check: Instant::now(),
                message: Some(format!("Spawn test failed: {}", e)),
                metadata,
            }),
            Err(_) => Ok(HealthStatus {
                component_id: self.component_id.clone(),
                status: Status::Unhealthy,
                last_check: Instant::now(),
                message: Some("Actor system unresponsive".to_string()),
                metadata,
            }),
        }
    }
    
    fn component_id(&self) -> ComponentId {
        self.component_id.clone()
    }
    
    fn check_interval(&self) -> Duration {
        Duration::from_secs(20)
    }
}
```

## Metrics Collection

⚠️ **VALIDATION NOTE**: Basic implementation provided. Production systems should integrate with Prometheus, OpenTelemetry, or similar.

### Metrics Collector

```rust
// src/metrics/collector.rs
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use crate::events::{EventType, EventError};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: MetricValue,
    pub timestamp: Instant,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Summary { sum: f64, count: u64 },
}

pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<String, Vec<Metric>>>>,
    flush_interval: Duration,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            flush_interval: Duration::from_secs(60),
        }
    }
    
    pub async fn record_event_published(&self, event_type: &EventType) {
        let metric = Metric {
            name: "event.published".to_string(),
            value: MetricValue::Counter(1),
            timestamp: Instant::now(),
            tags: HashMap::from([
                ("event_type".to_string(), format!("{:?}", event_type))
            ]),
        };
        
        let mut metrics = self.metrics.write().await;
        metrics.entry(metric.name.clone())
            .or_insert_with(Vec::new)
            .push(metric);
    }
    
    pub fn record_handler_error(&self, error: &EventError) {
        // Basic error recording - would be async in production
        // This is simplified for the example
    }
    
    pub async fn run(&self, shutdown_signal: Arc<AtomicBool>) {
        while !shutdown_signal.load(Ordering::Relaxed) {
            tokio::time::sleep(self.flush_interval).await;
            self.flush().await.ok();
        }
    }
    
    pub async fn flush(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut metrics = self.metrics.write().await;
        
        // In production, this would send to monitoring backend
        for (name, values) in metrics.iter() {
            info!("Metric {}: {} data points", name, values.len());
        }
        
        // Clear old metrics
        metrics.clear();
        Ok(())
    }
}
```

### Metric Types

```rust
// Extended metrics implementation
impl MetricsCollector {
    // Counter metric - always increases
    pub async fn increment_counter(&self, name: &str, tags: HashMap<String, String>) {
        let metric = Metric {
            name: name.to_string(),
            value: MetricValue::Counter(1),
            timestamp: Instant::now(),
            tags,
        };
        
        let mut metrics = self.metrics.write().await;
        metrics.entry(metric.name.clone())
            .or_insert_with(Vec::new)
            .push(metric);
    }
    
    // Gauge metric - can go up or down
    pub async fn set_gauge(&self, name: &str, value: f64, tags: HashMap<String, String>) {
        let metric = Metric {
            name: name.to_string(),
            value: MetricValue::Gauge(value),
            timestamp: Instant::now(),
            tags,
        };
        
        let mut metrics = self.metrics.write().await;
        metrics.entry(metric.name.clone())
            .or_insert_with(Vec::new)
            .push(metric);
    }
    
    // Histogram metric - tracks distribution
    pub async fn record_histogram(&self, name: &str, value: f64, tags: HashMap<String, String>) {
        let metric = Metric {
            name: name.to_string(),
            value: MetricValue::Histogram(vec![value]),
            timestamp: Instant::now(),
            tags,
        };
        
        let mut metrics = self.metrics.write().await;
        metrics.entry(metric.name.clone())
            .or_insert_with(Vec::new)
            .push(metric);
    }
    
    // Summary metric - tracks sum and count
    pub async fn record_summary(&self, name: &str, value: f64, tags: HashMap<String, String>) {
        let metric = Metric {
            name: name.to_string(),
            value: MetricValue::Summary { sum: value, count: 1 },
            timestamp: Instant::now(),
            tags,
        };
        
        let mut metrics = self.metrics.write().await;
        metrics.entry(metric.name.clone())
            .or_insert_with(Vec::new)
            .push(metric);
    }
}

// Production backend integration
pub trait MetricsBackend: Send + Sync {
    async fn send_metrics(&self, metrics: Vec<Metric>) -> Result<(), Box<dyn std::error::Error>>;
}

// Example Prometheus integration
pub struct PrometheusBackend {
    endpoint: String,
    client: reqwest::Client,
}

impl PrometheusBackend {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl MetricsBackend for PrometheusBackend {
    async fn send_metrics(&self, metrics: Vec<Metric>) -> Result<(), Box<dyn std::error::Error>> {
        // Convert metrics to Prometheus format
        let prometheus_metrics = self.convert_to_prometheus_format(metrics);
        
        // Send to Prometheus pushgateway
        self.client
            .post(&self.endpoint)
            .body(prometheus_metrics)
            .send()
            .await?;
            
        Ok(())
    }
}
```

## Integration with System Components

```rust
// Integration helper for wiring health and metrics into the system
pub struct MonitoringSystem {
    health_monitor: Arc<HealthMonitor>,
    metrics_collector: Arc<MetricsCollector>,
    event_bus: Arc<EventBus>,
}

impl MonitoringSystem {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        let health_monitor = Arc::new(
            HealthMonitor::new().with_event_bus(event_bus.clone())
        );
        let metrics_collector = Arc::new(MetricsCollector::new());
        
        Self {
            health_monitor,
            metrics_collector,
            event_bus,
        }
    }
    
    pub async fn register_component_health_checks(&self, components: SystemComponents) {
        // Register runtime health check
        if let Some(runtime) = components.runtime_manager {
            let check = Box::new(RuntimeHealthCheck::new(runtime));
            self.health_monitor.register_check(check).await;
        }
        
        // Register actor system health check
        if let Some(actor_system) = components.actor_system {
            let check = Box::new(AgentSystemHealthCheck::new(actor_system));
            self.health_monitor.register_check(check).await;
        }
        
        // Register database health check
        if let Some(db_pool) = components.database_pool {
            let check = Box::new(DatabaseHealthCheck::new(db_pool));
            self.health_monitor.register_check(check).await;
        }
    }
    
    pub async fn start(&self, shutdown_signal: Arc<AtomicBool>) {
        let health_task = {
            let monitor = self.health_monitor.clone();
            let signal = shutdown_signal.clone();
            tokio::spawn(async move {
                monitor.run(signal).await;
            })
        };
        
        let metrics_task = {
            let collector = self.metrics_collector.clone();
            let signal = shutdown_signal.clone();
            tokio::spawn(async move {
                collector.run(signal).await;
            })
        };
        
        // Wait for both tasks
        tokio::try_join!(health_task, metrics_task).ok();
    }
    
    pub fn health_monitor(&self) -> &Arc<HealthMonitor> {
        &self.health_monitor
    }
    
    pub fn metrics_collector(&self) -> &Arc<MetricsCollector> {
        &self.metrics_collector
    }
}

// System components struct for registration
pub struct SystemComponents {
    pub runtime_manager: Option<Arc<RuntimeManager>>,
    pub actor_system: Option<Arc<ActorSystem>>,
    pub database_pool: Option<Arc<DatabasePool>>,
    pub supervision_tree: Option<Arc<SupervisionTree>>,
}
```

---

## Cross-References

- For event integration, see [Supervision & Events](supervision-and-events.md)
- For runtime integration, see [Runtime & Errors](runtime-and-errors.md)
- For component architecture, see [Component Architecture](component-architecture.md)
- For implementation patterns, see [Implementation Guidelines](implementation-guidelines.md)

---

## Implementation Notes

This module provides comprehensive monitoring capabilities for the MisterSmith framework:

### Health Monitoring

- **Async health checks** with customizable intervals
- **Component-specific implementations** for runtime, database, and actor system
- **Event integration** for health status changes
- **Aggregated system health** assessment

### Metrics Collection

- **Multiple metric types**: Counter, Gauge, Histogram, Summary
- **Tag-based organization** for dimensional metrics
- **Async collection** with periodic flushing
- **Backend abstraction** for Prometheus, OpenTelemetry integration

### Production Considerations

1. **Integrate with real metrics backends** (Prometheus, Grafana, etc.)
2. **Add distributed tracing** support (OpenTelemetry)
3. **Implement alerting** based on health status
4. **Add performance profiling** metrics
5. **Configure retention policies** for metric data

The monitoring system is designed to provide deep observability into the distributed agent framework, enabling proactive issue detection and performance optimization.
