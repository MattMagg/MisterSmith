# Message Framework

## Technical Specifications for Message Validation, Serialization, and Routing

**🔍 AGENT OPTIMIZATION STATUS**

- **Agent**: Agent 7, Team Beta  
- **Optimization Date**: 2025-07-07
- **Target**: Remove business content, improve technical accuracy
- **Status**: OPTIMIZING
- **Focus**: Message routing, validation, and delivery patterns

**Purpose**: This document defines the technical implementation patterns for message validation, serialization, routing, and correlation within the Mister Smith AI Agent Framework.

## Overview

This file contains the technical framework supporting the message schemas:

- **Validation Framework** - Multi-level validation with performance optimization
- **Serialization Specifications** - JSON, Protocol Buffers, and MessagePack support
- **Event Correlation Logic** - Correlation ID tracking, aggregation, and causality chains
- **Message Transformation Patterns** - Protocol adaptation and enrichment
- **Schema Version Management** - Evolution and compatibility strategies
- **Implementation Guidelines** - Code generation and testing approaches
- **Security Considerations** - Input validation and encryption support
- **Performance Optimization** - High-throughput and low-latency strategies

This framework provides the foundation for all message schemas:

- [Core message schemas](./core-message-schemas.md) for agent communication
- [Workflow message schemas](./workflow-message-schemas.md) for task management
- [System message schemas](./system-message-schemas.md) for CLI integration and monitoring

For practical implementation, see [Agent Communication](./agent-communication.md) and [Storage Patterns](./storage-patterns.md).

## 7. Validation Framework

### 7.1 Validation Rules and Levels

The framework supports multiple validation levels to balance performance and safety. These levels apply to all message types:

- [Agent communication messages](./core-message-schemas.md#agent-communication-messages)
- [Task management messages](./workflow-message-schemas.md#task-management-messages)
- [System operation messages](./system-message-schemas.md#system-operation-messages)
- [Claude CLI integration messages](./system-message-schemas.md#claude-cli-integration-messages)

```yaml
validation_levels:
  strict:
    description: "Full schema validation with all constraints"
    performance: "Slower but comprehensive"
    use_cases: ["development", "testing", "critical_operations"]
    
  standard:
    description: "Essential validation with performance optimization"
    performance: "Balanced performance and safety"
    use_cases: ["production", "normal_operations"]
    
  permissive:
    description: "Minimal validation for high-throughput scenarios"
    performance: "Fastest validation"
    use_cases: ["high_frequency_messages", "monitoring_data"]

validation_configuration:
  schema_cache_size: 1000
  schema_cache_ttl_seconds: 3600
  max_validation_errors: 100
  fail_fast: false
  error_aggregation: true
  format_validation: true
  content_encoding_validation: true
```

### 7.2 Error Code Classification

Standardized error codes for message validation failures. These codes are used across:

- [Agent status reporting](./core-message-schemas.md#agent-status-update-message) for health monitoring
- [Task result messages](./workflow-message-schemas.md#task-result-message) for execution feedback
- [System alert messages](./system-message-schemas.md#system-alert-message) for error escalation
- [Hook response messages](./system-message-schemas.md#hook-response-message) for CLI error handling

```json
{
  "validation_errors": {
    "V1001": {
      "description": "Schema validation failed",
      "severity": "error",
      "retryable": false
    },
    "V1002": {
      "description": "Required field missing",
      "severity": "error",
      "retryable": false
    },
    "V1003": {
      "description": "Invalid field format",
      "severity": "error", 
      "retryable": false
    },
    "V1004": {
      "description": "Field value out of range",
      "severity": "error",
      "retryable": false
    },
    "V1005": {
      "description": "Invalid enum value",
      "severity": "error",
      "retryable": false
    },
    "V2001": {
      "description": "Message too large",
      "severity": "warning",
      "retryable": false
    },
    "V2002": {
      "description": "Deprecated field used",
      "severity": "warning", 
      "retryable": true
    },
    "V3001": {
      "description": "Schema version incompatible",
      "severity": "error",
      "retryable": false
    },
    "V3002": {
      "description": "Unknown message type",
      "severity": "error",
      "retryable": false
    }
  }
}
```

### 7.3 Performance Optimization

Validation performance optimizations and caching strategies. Optimizations apply to:

- High-frequency [agent status updates](./core-message-schemas.md#agent-status-update-message)
- Batch [task assignment operations](./workflow-message-schemas.md#task-assignment-message)
- Real-time [system health monitoring](./system-message-schemas.md#system-health-check-message)
- CLI [hook event processing](./system-message-schemas.md#hook-event-message)

```rust
// Example validation configuration in Rust
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub level: ValidationLevel,
    pub schema_cache_size: usize,
    pub enable_fast_path: bool,
    pub max_validation_errors: usize,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone)]
pub enum ValidationLevel {
    Strict,    // Full validation
    Standard,  // Balanced validation
    Permissive // Minimal validation
}

// Fast-path validation for high-frequency message types
pub struct FastPathValidator {
    compiled_schemas: HashMap<String, CompiledSchema>,
    performance_cache: LruCache<String, ValidationResult>,
}
```

## 8. Serialization Specifications

### 8.1 JSON Serialization Standards

JSON serialization standards apply to all framework messages, ensuring consistency across:

- [Base message envelopes](./core-message-schemas.md#base-message-envelope) and common types
- [Workflow coordination](./workflow-message-schemas.md#workflow-coordination-message) and state sync
- [System monitoring](./system-message-schemas.md#system-health-check-message) and alert data
- [CLI integration](./system-message-schemas.md#claude-cli-integration-messages) payloads

```json
{
  "json_serialization": {
    "encoding": "UTF-8",
    "date_format": "ISO 8601 (RFC 3339)",
    "number_precision": "IEEE 754 double precision",
    "string_max_length": 1048576,
    "object_max_depth": 32,
    "array_max_length": 10000,
    "null_handling": "explicit",
    "pretty_print": false,
    "escape_unicode": false
  },
  "compression": {
    "algorithm": "gzip",
    "level": 6,
    "threshold_bytes": 1024,
    "content_types": ["application/json", "text/plain"]
  },
  "security": {
    "max_string_length": 1048576,
    "max_array_length": 10000,
    "max_object_properties": 1000,
    "max_nesting_depth": 32,
    "sanitization": {
      "html_entities": true,
      "sql_injection": true,
      "xss_prevention": true
    }
  }
}
```

### 8.2 Binary Serialization Alternatives

Support for Protocol Buffers and MessagePack for performance-critical scenarios, especially:

- High-throughput [agent communication](./core-message-schemas.md#agent-communication-messages)
- Large-scale [workflow orchestration](./workflow-message-schemas.md#workflow-orchestration-messages)
- Real-time [system monitoring](./system-message-schemas.md#system-operation-messages)
- Performance-sensitive [transport layers](../transport/transport-core.md)

```protobuf
// Protocol Buffers message envelope
syntax = "proto3";
package mister_smith.messages;

message BaseMessage {
  string message_id = 1;
  int64 timestamp_unix_nano = 2;
  string schema_version = 3;
  string message_type = 4;
  string correlation_id = 5;
  string trace_id = 6;
  string source_agent_id = 7;
  string target_agent_id = 8;
  int32 priority = 9;
  string reply_to = 10;
  int64 timeout_ms = 11;
  map<string, string> metadata = 12;
  bytes payload = 13;  // Serialized message-specific payload
}
```

MessagePack configuration for space-efficient serialization:

```yaml
messagepack_config:
  use_compact_integers: true
  use_string_keys: true
  preserve_order: false
  binary_threshold_bytes: 512
  compression_after_pack: true
```

## 9. Message Routing

### 9.1 Routing Table Implementation

Dynamic routing table management for efficient message delivery across:

- [Agent-to-agent communication](./core-message-schemas.md#agent-communication-messages) pathways
- [Workflow task distribution](./workflow-message-schemas.md#task-assignment-message) to capable agents
- [System operation routing](./system-message-schemas.md#system-operation-messages) for monitoring
- [CLI command routing](./system-message-schemas.md#claude-cli-integration-messages) to handlers

```yaml
routing_table_schema:
  static_routes:
    agent_direct:
      pattern: "agent.{agent_id}.command"
      destination: "direct:agent:{agent_id}"
      priority: 100
      ttl_seconds: 0  # No expiration
      
    workflow_broadcast:
      pattern: "workflow.*.assignment"
      destination: "fanout:workflow:coordinators"
      priority: 90
      filter: "agent.capabilities contains 'workflow'"
      
    system_monitoring:
      pattern: "system.health.*"
      destination: "topic:monitoring:health"
      priority: 95
      aggregation: true
      
  dynamic_routes:
    capability_based:
      discovery: "agent_registry"
      refresh_interval_seconds: 30
      routing_key: "required_capabilities"
      fallback: "queue:unrouted_messages"
      
    load_balanced:
      algorithm: "weighted_round_robin"
      health_check_interval: 5000
      remove_unhealthy: true
      rebalance_threshold: 0.2

routing_table_operations:
  add_route:
    validation: "pattern_syntax_check"
    conflict_resolution: "priority_based"
    activation: "immediate"
    
  remove_route:
    grace_period_seconds: 60
    drain_messages: true
    notify_endpoints: true
    
  update_route:
    versioning: "atomic_swap"
    rollback_on_error: true
    performance_test: true
```

### 9.2 Dynamic Route Discovery

Automatic route discovery and registration mechanisms for:

- [Agent registration](./core-message-schemas.md#agent-registration-message) auto-routing
- [Workflow participant discovery](./workflow-message-schemas.md#workflow-state-synchronization-message)
- [System service discovery](./system-message-schemas.md#system-health-check-message)
- Dynamic capability matching for task assignment

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("Discovery timeout: {0}")]
    Timeout(String),
    #[error("Registry query failed: {0}")]
    RegistryError(String),
    #[error("Route creation failed: {0}")]
    RouteCreationError(String),
    #[error("Too many routes: limit {limit} exceeded")]
    TooManyRoutes { limit: usize },
}

#[derive(Debug, Clone)]
pub struct RouteDiscovery {
    registry: Arc<RwLock<RouteRegistry>>,
    discovery_interval: std::time::Duration,
    capability_index: Arc<RwLock<CapabilityIndex>>,
}

#[derive(Debug, Clone)]
pub struct DiscoveredRoute {
    pub pattern: String,
    pub endpoints: Vec<Endpoint>,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
    pub health_status: HealthStatus,
}

impl RouteDiscovery {
    pub async fn discover_routes(&self) -> Result<Vec<DiscoveredRoute>, DiscoveryError> {
        // Query agent registry for active agents with timeout
        let agents = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            self.query_agent_registry()
        ).await
        .map_err(|_| DiscoveryError::Timeout("Agent registry query timed out".to_string()))?
        .map_err(DiscoveryError::from)?;
        
        // Build routes based on agent capabilities with bounds checking
        let mut routes = Vec::new();
        const MAX_ROUTES_PER_DISCOVERY: usize = 10_000;
        
        for agent in agents {
            // Create capability-based routes with limits
            for capability in &agent.capabilities {
                if routes.len() >= MAX_ROUTES_PER_DISCOVERY {
                    return Err(DiscoveryError::TooManyRoutes { 
                        limit: MAX_ROUTES_PER_DISCOVERY 
                    });
                }
                
                routes.push(DiscoveredRoute {
                    pattern: format!("capability.{}.request", capability),
                    endpoints: vec![agent.endpoint.clone()],
                    capabilities: agent.capabilities.clone(),
                    metadata: agent.metadata.clone(),
                    discovered_at: chrono::Utc::now(),
                    health_status: agent.health_status.clone(),
                });
            }
            
            // Create direct agent routes with bounds check
            if routes.len() >= MAX_ROUTES_PER_DISCOVERY {
                return Err(DiscoveryError::TooManyRoutes { 
                    limit: MAX_ROUTES_PER_DISCOVERY 
                });
            }
            
            routes.push(DiscoveredRoute {
                pattern: format!("agent.{}.direct", agent.id),
                endpoints: vec![agent.endpoint.clone()],
                capabilities: agent.capabilities.clone(),
                metadata: agent.metadata.clone(),
                discovered_at: chrono::Utc::now(),
                health_status: agent.health_status.clone(),
            });
        }
        
        // Update capability index for fast lookup
        self.update_capability_index(&routes).await?;
        
        Ok(routes)
    }
    
    pub async fn find_capable_agents(&self, required_capabilities: &[String]) -> Vec<String> {
        let index = self.capability_index.read().await;
        index.find_agents_with_capabilities(required_capabilities)
    }
}

// Service mesh integration for route discovery
pub struct ServiceMeshDiscovery {
    consul_client: Option<ConsulClient>,
    kubernetes_client: Option<K8sClient>,
    custom_discovery: Option<Box<dyn Discovery>>,
}
```

### 9.3 Load Balancing Strategies

Intelligent load distribution across message endpoints supporting:

- [High-frequency agent status updates](./core-message-schemas.md#agent-status-update-message)
- [Parallel task execution](./workflow-message-schemas.md#task-assignment-message) distribution
- [System monitoring aggregation](./system-message-schemas.md#system-health-check-message)
- Adaptive routing based on endpoint performance

```yaml
load_balancing_strategies:
  round_robin:
    description: "Simple sequential distribution"
    state: "endpoint_index"
    skip_unhealthy: true
    sticky_sessions: false
    
  weighted_round_robin:
    description: "Distribution based on endpoint capacity"
    weights:
      calculation: "cpu_cores * memory_gb * health_score"
      update_interval: 30
      smoothing_factor: 0.7
    rebalance_triggers:
      - "weight_change > 20%"
      - "endpoint_added"
      - "endpoint_removed"
      
  least_connections:
    description: "Route to endpoint with fewest active connections"
    connection_tracking: true
    include_pending: true
    connection_cost:
      message_size_factor: 1.0
      processing_time_factor: 2.0
      
  latency_based:
    description: "Route based on response time"
    measurement_window: 60
    percentile: 95  # Use p95 latency
    fallback_strategy: "round_robin"
    
  capability_affinity:
    description: "Prefer endpoints with matching capabilities"
    scoring:
      exact_match: 10
      partial_match: 5
      has_capability: 1
    tie_breaker: "least_connections"
    
  adaptive:
    description: "ML-based routing optimization"
    model: "gradient_boosting"
    features:
      - "endpoint_latency"
      - "endpoint_cpu"
      - "endpoint_queue_depth"
      - "message_size"
      - "message_priority"
    training_interval: 3600
    exploration_rate: 0.1

load_balancer_config:
  health_checking:
    interval_ms: 5000
    timeout_ms: 2000
    unhealthy_threshold: 3
    healthy_threshold: 2
    
  circuit_breaker:
    enabled: true
    error_threshold: 0.5
    volume_threshold: 20
    sleep_window_ms: 30000
    
  connection_pooling:
    min_connections: 2
    max_connections: 100
    idle_timeout_ms: 60000
    connection_timeout_ms: 5000
```

### 9.4 Topic-Based Routing

Hierarchical topic routing for publish-subscribe patterns supporting:

- [Agent event notifications](./core-message-schemas.md#agent-event-notification-message)
- [Workflow state changes](./workflow-message-schemas.md#workflow-state-synchronization-message)
- [System alerts broadcasting](./system-message-schemas.md#system-alert-message)
- Multi-level topic hierarchies with wildcards

```json
{
  "topic_routing": {
    "topic_hierarchy": {
      "separator": ".",
      "max_levels": 8,
      "wildcard_single": "*",
      "wildcard_multi": ">",
      "examples": [
        "agent.*.status",
        "workflow.task.>",
        "system.health.cpu.>",
        "cli.hook.*.response"
      ]
    },
    "subscription_management": {
      "durable_subscriptions": true,
      "subscription_groups": true,
      "max_subscribers_per_topic": 1000,
      "subscription_timeout_ms": 300000,
      "automatic_unsubscribe": true
    },
    "routing_rules": {
      "agent_events": {
        "pattern": "agent.{agent_id}.{event_type}",
        "retention": "24h",
        "fanout": true,
        "ordering": "per_agent"
      },
      "workflow_coordination": {
        "pattern": "workflow.{workflow_id}.{stage}",
        "retention": "7d",
        "partitioning": "workflow_id",
        "ordering": "strict"
      },
      "system_monitoring": {
        "pattern": "system.{component}.{metric}",
        "retention": "1h",
        "aggregation": true,
        "sampling": "adaptive"
      }
    },
    "topic_policies": {
      "access_control": "role_based",
      "message_ttl": "topic_specific",
      "max_message_size": "10MB",
      "compression": "auto"
    }
  }
}
```

### 9.5 Content-Based Routing

Advanced routing based on message content inspection for:

- [Task requirement matching](./workflow-message-schemas.md#task-assignment-message)
- [Agent capability filtering](./core-message-schemas.md#agent-command-message)
- [Alert severity routing](./system-message-schemas.md#system-alert-message)
- Complex conditional routing logic

```rust
use serde_json::Value;
use regex::Regex;
use lru::LruCache;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, thiserror::Error)]
pub enum RoutingError {
    #[error("Invalid routing condition: {0}")]
    InvalidCondition(String),
    #[error("Field extraction failed: {0}")]
    FieldExtraction(String),
    #[error("Transformation failed: {0}")]
    TransformationError(String),
}

#[derive(Debug, Clone)]
pub struct ContentRouter {
    rules: Vec<ContentRoutingRule>,
    default_route: String,
    performance_cache: LruCache<u64, String>,
}

#[derive(Debug, Clone)]
pub struct ContentRoutingRule {
    pub name: String,
    pub priority: u32,
    pub conditions: Vec<Condition>,
    pub destination: String,
    pub transformations: Vec<Transformation>,
}

#[derive(Debug, Clone)]
pub enum Condition {
    FieldEquals { path: String, value: Value },
    FieldMatches { path: String, pattern: Regex },
    FieldExists { path: String },
    FieldInRange { path: String, min: f64, max: f64 },
    CustomPredicate { function: String },
    And(Vec<Condition>),
    Or(Vec<Condition>),
    Not(Box<Condition>),
}

#[derive(Debug, Clone)]
pub enum Transformation {
    AddField { path: String, value: Value },
    RemoveField { path: String },
    ModifyField { path: String, operation: String },
    SetPriority { priority: u8 },
}

impl ContentRouter {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            default_route: "default".to_string(),
            performance_cache: LruCache::new(1000),
        }
    }
    
    /// Generate a hash for message content to enable caching
    fn hash_message_content(&self, message: &Value) -> u64 {
        let mut hasher = DefaultHasher::new();
        
        // Hash key fields for cache key generation
        if let Some(msg_type) = message.get("message_type") {
            msg_type.hash(&mut hasher);
        }
        if let Some(source) = message.get("source_agent_id") {
            source.hash(&mut hasher);
        }
        if let Some(target) = message.get("target_agent_id") {
            target.hash(&mut hasher);
        }
        
        // Hash a portion of the payload for differentiation
        if let Some(payload) = message.get("payload") {
            let payload_str = serde_json::to_string(payload).unwrap_or_default();
            let hash_slice = if payload_str.len() > 1000 {
                &payload_str[..1000] // Only hash first 1KB for performance
            } else {
                &payload_str
            };
            hash_slice.hash(&mut hasher);
        }
        
        hasher.finish()
    }
    
    pub fn route_message(&mut self, message: &Value) -> Result<String, RoutingError> {
        // Check performance cache first
        let cache_key = self.hash_message_content(message);
        if let Some(cached_route) = self.performance_cache.get(&cache_key) {
            return Ok(cached_route.clone());
        }
        
        // Evaluate rules in priority order
        let mut rules = self.rules.clone();
        rules.sort_by_key(|r| std::cmp::Reverse(r.priority));
        
        for rule in rules {
            if self.evaluate_conditions(&rule.conditions, message)? {
                // Apply transformations if specified
                let _transformed = self.apply_transformations(
                    message,
                    &rule.transformations
                )?;
                
                // Cache the routing decision
                self.performance_cache.put(cache_key, rule.destination.clone());
                
                return Ok(rule.destination);
            }
        }
        
        Ok(self.default_route.clone())
    }
    
    /// Extract field value from message using JSON path notation
    fn extract_field(&self, message: &Value, path: &str) -> Result<&Value, RoutingError> {
        let path_parts: Vec<&str> = path.split('.').collect();
        let mut current = message;
        
        for part in path_parts {
            current = current.get(part)
                .ok_or_else(|| RoutingError::FieldExtraction(
                    format!("Field '{}' not found in path '{}'", part, path)
                ))?;
        }
        
        Ok(current)
    }
    
    fn evaluate_conditions(&self, conditions: &[Condition], message: &Value) -> Result<bool, RoutingError> {
        for condition in conditions {
            match condition {
                Condition::FieldEquals { path, value } => {
                    let field_value = self.extract_field(message, path)?;
                    if field_value != value {
                        return Ok(false);
                    }
                }
                Condition::FieldMatches { path, pattern } => {
                    let field_value = self.extract_field(message, path)?;
                    if let Some(str_value) = field_value.as_str() {
                        if !pattern.is_match(str_value) {
                            return Ok(false);
                        }
                    } else {
                        return Ok(false);
                    }
                }
                Condition::And(subconditions) => {
                    if !self.evaluate_conditions(subconditions, message)? {
                        return Ok(false);
                    }
                }
                Condition::Or(subconditions) => {
                    let mut any_match = false;
                    for subcondition in subconditions {
                        if self.evaluate_conditions(&[subcondition.clone()], message)? {
                            any_match = true;
                            break;
                        }
                    }
                    if !any_match {
                        return Ok(false);
                    }
                }
                // ... other condition types
            }
        }
        Ok(true)
    }
    
    /// Apply message transformations for content-based routing
    fn apply_transformations(&self, message: &Value, transformations: &[Transformation]) -> Result<Value, RoutingError> {
        let mut result = message.clone();
        
        for transformation in transformations {
            match transformation {
                Transformation::AddField { path, value } => {
                    // Simple implementation for demonstration
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert(path.clone(), value.clone());
                    }
                }
                Transformation::RemoveField { path } => {
                    if let Some(obj) = result.as_object_mut() {
                        obj.remove(path);
                    }
                }
                Transformation::ModifyField { path, operation: _ } => {
                    // Placeholder for field modification logic
                    // In practice, this would implement specific operations
                    log::debug!("Field modification requested for path: {}", path);
                }
                Transformation::SetPriority { priority } => {
                    if let Some(obj) = result.as_object_mut() {
                        obj.insert("priority".to_string(), serde_json::json!(*priority));
                    }
                }
            }
        }
        
        Ok(result)
    }
}

// Example content routing rules configuration
pub fn example_content_routing_rules() -> Vec<ContentRoutingRule> {
    use serde_json::json;
    
    vec![
        ContentRoutingRule {
            name: "high_priority_tasks".to_string(),
            priority: 100,
            conditions: vec![
                Condition::FieldEquals {
                    path: "message_type".to_string(),
                    value: json!("task_assignment"),
                },
                Condition::FieldInRange {
                    path: "priority".to_string(),
                    min: 8.0,
                    max: 10.0,
                },
            ],
            destination: "queue:high_priority_tasks".to_string(),
            transformations: vec![],
        },
        ContentRoutingRule {
            name: "critical_alerts".to_string(),
            priority: 95,
            conditions: vec![
                Condition::FieldEquals {
                    path: "message_type".to_string(),
                    value: json!("system_alert"),
                },
                Condition::FieldEquals {
                    path: "severity".to_string(),
                    value: json!("critical"),
                },
            ],
            destination: "topic:alerts:critical".to_string(),
            transformations: vec![
                Transformation::AddField {
                    path: "routed_at".to_string(),
                    value: json!(chrono::Utc::now().to_rfc3339()),
                },
            ],
        },
    ]
}
```

### 9.6 Circuit Breaker Patterns

Fault tolerance and resilience for message routing supporting:

- [Agent communication failures](./core-message-schemas.md#agent-error-message)
- [Workflow error handling](./workflow-message-schemas.md#task-result-message)
- [System degradation management](./system-message-schemas.md#system-alert-message)
- Automatic recovery and fallback routing

```yaml
circuit_breaker_config:
  states:
    closed:
      description: "Normal operation, requests pass through"
      error_threshold: 5
      error_window_seconds: 60
      success_threshold: 2
      
    open:
      description: "Failures exceeded threshold, requests blocked"
      timeout_seconds: 30
      fallback_action: "route_to_secondary"
      notification: "alert:circuit_opened"
      
    half_open:
      description: "Testing if service recovered"
      test_requests: 3
      success_required: 2
      failure_action: "return_to_open"
      
  failure_detection:
    error_types:
      - "connection_timeout"
      - "connection_refused"
      - "5xx_status_codes"
      - "invalid_response"
    latency_threshold_ms: 5000
    consecutive_failures: 5
    
  recovery_strategies:
    exponential_backoff:
      initial_delay_ms: 1000
      max_delay_ms: 60000
      multiplier: 2
      jitter: 0.1
      
    gradual_recovery:
      initial_traffic_percent: 10
      increment_percent: 10
      increment_interval_seconds: 30
      success_rate_threshold: 0.95
      
  metrics:
    track:
      - "total_requests"
      - "failed_requests"
      - "success_rate"
      - "response_time_p99"
      - "circuit_state_changes"
    export_interval_seconds: 10
    retention_days: 30

per_route_circuit_breakers:
  agent_communication:
    error_threshold: 10
    timeout_seconds: 20
    fallback: "queue:agent_command_retry"
    
  workflow_coordination:
    error_threshold: 5
    timeout_seconds: 30
    fallback: "dead_letter:workflow_failed"
    
  system_monitoring:
    error_threshold: 20
    timeout_seconds: 10
    fallback: "local_cache:last_known_state"
```

### 9.7 Dead Letter Queue Management

Handling of undeliverable messages across the framework for:

- [Failed agent commands](./core-message-schemas.md#agent-command-message)
- [Unassigned tasks](./workflow-message-schemas.md#task-assignment-message)
- [Unprocessed system events](./system-message-schemas.md#system-operation-messages)
- Message retry and escalation strategies

```json
{
  "dead_letter_queue": {
    "configuration": {
      "max_retries": 3,
      "retry_delays": [1000, 5000, 15000],
      "max_age_days": 7,
      "storage_backend": "persistent",
      "compression": true,
      "encryption": true
    },
    "routing_rules": {
      "agent_messages": {
        "pattern": "agent.*",
        "dlq": "dlq:agent_messages",
        "retry_strategy": "exponential_backoff",
        "alert_after_retries": 2,
        "escalation": "supervisor_agent"
      },
      "workflow_messages": {
        "pattern": "workflow.*",
        "dlq": "dlq:workflow_messages",
        "retry_strategy": "linear_backoff",
        "preserve_order": true,
        "manual_intervention_after": 3
      },
      "system_messages": {
        "pattern": "system.*",
        "dlq": "dlq:system_messages",
        "retry_strategy": "immediate",
        "max_retries": 5,
        "discard_after": "1h"
      }
    },
    "message_metadata": {
      "failure_reason": "string",
      "original_destination": "string",
      "retry_count": "integer",
      "first_failure_time": "timestamp",
      "last_retry_time": "timestamp",
      "error_details": "object",
      "correlation_context": "object"
    },
    "recovery_operations": {
      "replay_messages": {
        "filters": ["time_range", "message_type", "destination"],
        "rate_limit": 100,
        "preserve_order": false,
        "update_timestamps": true
      },
      "bulk_retry": {
        "batch_size": 100,
        "parallel_workers": 5,
        "stop_on_error": false,
        "progress_tracking": true
      },
      "message_inspection": {
        "decode_payload": true,
        "validate_schema": true,
        "check_dependencies": true,
        "suggest_fixes": true
      }
    }
  }
}
```

### 9.8 Routing Monitoring and Metrics

Comprehensive monitoring of routing performance supporting:

- Real-time routing decisions tracking
- Performance bottleneck identification
- Route effectiveness analysis
- Adaptive routing optimization

```yaml
routing_metrics:
  performance_metrics:
    route_latency:
      description: "Time to make routing decision"
      unit: "microseconds"
      aggregations: ["p50", "p95", "p99"]
      alert_threshold_p99: 1000
      
    message_throughput:
      description: "Messages routed per second"
      unit: "messages/sec"
      aggregations: ["sum", "rate"]
      by_route: true
      
    routing_errors:
      description: "Failed routing attempts"
      unit: "errors"
      aggregations: ["count", "rate"]
      by_error_type: true
      
  effectiveness_metrics:
    route_hit_rate:
      description: "Percentage of messages matching routes"
      calculation: "matched_routes / total_messages"
      target: 0.95
      
    load_distribution:
      description: "Message distribution across endpoints"
      calculation: "standard_deviation(endpoint_message_counts)"
      target: "minimize"
      
    circuit_breaker_trips:
      description: "Circuit breaker activations"
      by_route: true
      severity_scoring: true
      
  queue_metrics:
    queue_depth:
      description: "Messages waiting in queues"
      by_queue: true
      alert_threshold: 1000
      
    queue_latency:
      description: "Time spent in queue"
      unit: "milliseconds"
      aggregations: ["p50", "p95", "p99", "max"]
      
    dlq_messages:
      description: "Messages in dead letter queues"
      by_reason: true
      alert_threshold: 100

monitoring_dashboards:
  routing_overview:
    widgets:
      - "message_flow_sankey"
      - "route_hit_rates"
      - "endpoint_health_map"
      - "error_rate_timeline"
      
  performance_analysis:
    widgets:
      - "latency_heatmap"
      - "throughput_by_route"
      - "queue_depth_gauge"
      - "circuit_breaker_status"
      
  troubleshooting:
    widgets:
      - "failed_routes_table"
      - "dlq_analysis"
      - "slow_routes_list"
      - "endpoint_errors_breakdown"
```

## 9. Event Correlation Logic

### 9.1 Correlation ID Generation and Tracking

The framework provides comprehensive correlation ID management for tracing message flows across distributed agent systems. Correlation enables:

- Request-response pattern tracking across [agent communication](./core-message-schemas.md#agent-communication-messages)
- Workflow execution tracing in [task management](./workflow-message-schemas.md#task-management-messages)
- System operation correlation for [monitoring and alerts](./system-message-schemas.md#system-operation-messages)
- CLI session tracking for [hook events](./system-message-schemas.md#hook-event-message)

```rust
use uuid::Uuid;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use rand::Rng;

/// Generate a 16-character hexadecimal span ID for distributed tracing
/// Compatible with OpenTelemetry and Jaeger tracing standards
fn generate_span_id() -> String {
    let mut rng = rand::thread_rng();
    let span_id: u64 = rng.gen();
    format!("{:016x}", span_id)
}

#[derive(Debug, Clone)]
pub struct CorrelationContext {
    pub correlation_id: Uuid,
    pub parent_correlation_id: Option<Uuid>,
    pub trace_id: Uuid,
    pub span_id: String,
    pub baggage: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub ttl_seconds: u64,
}

impl CorrelationContext {
    pub fn new() -> Self {
        Self {
            correlation_id: Uuid::new_v4(),
            parent_correlation_id: None,
            trace_id: Uuid::new_v4(),
            span_id: generate_span_id(),
            baggage: HashMap::new(),
            created_at: Utc::now(),
            ttl_seconds: 3600, // 1 hour default
        }
    }
    
    pub fn child(&self) -> Self {
        Self {
            correlation_id: Uuid::new_v4(),
            parent_correlation_id: Some(self.correlation_id),
            trace_id: self.trace_id, // Preserve trace across correlation chain
            span_id: generate_span_id(),
            baggage: self.baggage.clone(),
            created_at: Utc::now(),
            ttl_seconds: self.ttl_seconds,
        }
    }
    
    /// Add size limits and cleanup for baggage data
    pub fn add_baggage(&mut self, key: String, value: String) -> Result<(), &'static str> {
        const MAX_BAGGAGE_SIZE: usize = 8192; // 8KB limit
        const MAX_BAGGAGE_ITEMS: usize = 64;
        
        if self.baggage.len() >= MAX_BAGGAGE_ITEMS {
            return Err("Maximum baggage items exceeded");
        }
        
        let total_size: usize = self.baggage.iter()
            .map(|(k, v)| k.len() + v.len())
            .sum::<usize>() + key.len() + value.len();
            
        if total_size > MAX_BAGGAGE_SIZE {
            return Err("Maximum baggage size exceeded");
        }
        
        self.baggage.insert(key, value);
        Ok(())
    }
}

// High-performance correlation tracking
pub struct CorrelationTracker {
    active_correlations: HashMap<Uuid, CorrelationContext>,
    correlation_chains: HashMap<Uuid, Vec<Uuid>>, // parent -> children
    expiry_queue: BTreeMap<DateTime<Utc>, Vec<Uuid>>,
}

impl CorrelationTracker {
    pub fn new() -> Self {
        Self {
            active_correlations: HashMap::new(),
            correlation_chains: HashMap::new(),
            expiry_queue: BTreeMap::new(),
        }
    }
    
    /// Add a correlation context with automatic cleanup scheduling
    pub fn add_correlation(&mut self, context: CorrelationContext) {
        let correlation_id = context.correlation_id;
        let expiry_time = context.created_at + chrono::Duration::seconds(context.ttl_seconds as i64);
        
        // Add to expiry queue for cleanup
        self.expiry_queue
            .entry(expiry_time)
            .or_insert_with(Vec::new)
            .push(correlation_id);
            
        // Track parent-child relationships
        if let Some(parent_id) = context.parent_correlation_id {
            self.correlation_chains
                .entry(parent_id)
                .or_insert_with(Vec::new)
                .push(correlation_id);
        }
        
        self.active_correlations.insert(correlation_id, context);
    }
    
    /// Clean up expired correlations to prevent memory leaks
    pub fn cleanup_expired(&mut self) {
        let now = chrono::Utc::now();
        let expired_times: Vec<DateTime<Utc>> = self.expiry_queue
            .range(..=now)
            .map(|(time, _)| *time)
            .collect();
            
        for expired_time in expired_times {
            if let Some(expired_ids) = self.expiry_queue.remove(&expired_time) {
                for correlation_id in expired_ids {
                    self.active_correlations.remove(&correlation_id);
                    
                    // Clean up correlation chains
                    if let Some(children) = self.correlation_chains.remove(&correlation_id) {
                        for child_id in children {
                            self.active_correlations.remove(&child_id);
                        }
                    }
                }
            }
        }
    }
    
    /// Get correlation context with automatic expiry check
    pub fn get_correlation(&mut self, correlation_id: &Uuid) -> Option<&CorrelationContext> {
        // Check if expired first
        if let Some(context) = self.active_correlations.get(correlation_id) {
            let expiry_time = context.created_at + chrono::Duration::seconds(context.ttl_seconds as i64);
            if chrono::Utc::now() > expiry_time {
                // Remove expired context
                self.active_correlations.remove(correlation_id);
                return None;
            }
        }
        
        self.active_correlations.get(correlation_id)
    }
}
```

### 9.2 Event Aggregation Patterns

Event aggregation enables efficient processing of high-volume message streams. The framework supports multiple aggregation strategies:

```yaml
aggregation_patterns:
  windowed_aggregation:
    description: "Time-based event windowing"
    window_types:
      - fixed: "Fixed-size time windows"
      - sliding: "Overlapping time windows"
      - session: "Activity-based windows"
    configuration:
      window_size_ms: 5000
      slide_interval_ms: 1000
      max_events_per_window: 10000
      late_arrival_tolerance_ms: 1000
      
  keyed_aggregation:
    description: "Aggregate events by key attributes"
    key_extractors:
      - agent_id: "Group by source agent"
      - message_type: "Group by message type"
      - workflow_id: "Group by workflow instance"
      - composite: "Multi-attribute grouping"
    
  statistical_aggregation:
    description: "Compute statistics over event streams"
    computations:
      - count: "Event count per window"
      - sum: "Numeric field summation"
      - average: "Running averages"
      - percentiles: "P50, P95, P99"
      - min_max: "Range tracking"
      
  pattern_detection:
    description: "Complex event pattern matching"
    patterns:
      - sequence: "Ordered event sequences"
      - parallel: "Concurrent event patterns"
      - absence: "Missing event detection"
      - threshold: "Count-based triggers"
```

Implementation example for high-performance aggregation:

```rust
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

pub struct EventAggregator<T> {
    window_size: Duration,
    windows: HashMap<String, TimeWindow<T>>,
    aggregation_fn: Box<dyn Fn(&[T]) -> AggregationResult>,
}

pub struct TimeWindow<T> {
    start_time: Instant,
    events: VecDeque<(Instant, T)>,
    statistics: WindowStatistics,
}

pub struct WindowStatistics {
    event_count: u64,
    first_event_time: Option<Instant>,
    last_event_time: Option<Instant>,
    processing_latency_ns: u64,
}

impl<T> EventAggregator<T> {
    pub fn process_event(&mut self, key: String, event: T) -> Option<AggregationResult> {
        let now = Instant::now();
        let window = self.windows.entry(key).or_insert_with(|| {
            TimeWindow::new(now, self.window_size)
        });
        
        // Add event to window
        window.add_event(now, event);
        
        // Check if window is complete
        if window.is_complete(now) {
            let result = (self.aggregation_fn)(&window.get_events());
            self.windows.remove(&key);
            Some(result)
        } else {
            None
        }
    }
}
```

### 9.3 Temporal Correlation Windows

Temporal correlation enables relating events that occur within specific time boundaries:

```json
{
  "temporal_correlation": {
    "window_definitions": {
      "immediate": {
        "duration_ms": 100,
        "description": "Near-synchronous events"
      },
      "short_term": {
        "duration_ms": 5000,
        "description": "Related operational events"
      },
      "medium_term": {
        "duration_ms": 60000,
        "description": "Workflow-level correlation"
      },
      "long_term": {
        "duration_ms": 3600000,
        "description": "Session-level correlation"
      }
    },
    "correlation_rules": {
      "agent_coordination": {
        "window": "short_term",
        "required_events": ["agent_command", "agent_response"],
        "timeout_action": "alert"
      },
      "workflow_execution": {
        "window": "medium_term",
        "required_events": ["task_assigned", "task_started", "task_completed"],
        "partial_match_allowed": true
      },
      "system_health": {
        "window": "immediate",
        "correlation_attributes": ["node_id", "service_type"],
        "aggregation": "latest_value"
      }
    }
  }
}
```

### 9.4 Causality Chain Tracking

The framework maintains causality relationships between events to enable root cause analysis and impact assessment:

```rust
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

pub struct CausalityTracker {
    // Directed graph of causal relationships
    causality_graph: DiGraph<EventNode, CausalEdge>,
    // Fast lookup from event ID to graph node
    event_index: HashMap<Uuid, NodeIndex>,
    // Pruning configuration
    max_chain_length: usize,
    max_graph_size: usize,
}

#[derive(Debug, Clone)]
pub struct EventNode {
    pub event_id: Uuid,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub agent_id: String,
    pub correlation_id: Uuid,
    pub impact_score: f64,
}

#[derive(Debug, Clone)]
pub struct CausalEdge {
    pub relationship_type: CausalRelationship,
    pub confidence: f64,
    pub latency_ms: u64,
}

#[derive(Debug, Clone)]
pub enum CausalRelationship {
    Triggers,      // A triggers B
    Requires,      // A requires B to complete first
    Inhibits,      // A prevents B
    Correlates,    // A and B are correlated but causality unknown
}

impl CausalityTracker {
    pub fn add_causal_link(&mut self, 
        cause_event: Uuid, 
        effect_event: Uuid,
        relationship: CausalRelationship,
        confidence: f64
    ) -> Result<(), CausalityError> {
        // Validate no cycles
        if self.would_create_cycle(cause_event, effect_event) {
            return Err(CausalityError::CyclicDependency);
        }
        
        // Add nodes if not present
        let cause_idx = self.get_or_create_node(cause_event);
        let effect_idx = self.get_or_create_node(effect_event);
        
        // Calculate latency
        let cause_time = self.get_event_time(cause_idx);
        let effect_time = self.get_event_time(effect_idx);
        let latency_ms = (effect_time - cause_time).num_milliseconds() as u64;
        
        // Add edge
        self.causality_graph.add_edge(
            cause_idx,
            effect_idx,
            CausalEdge {
                relationship_type: relationship,
                confidence,
                latency_ms,
            }
        );
        
        // Prune if needed
        self.prune_old_chains();
        
        Ok(())
    }
    
    pub fn get_root_causes(&self, event_id: Uuid) -> Vec<EventNode> {
        // Traverse graph backwards to find root causes
        let mut roots = Vec::new();
        let mut visited = HashSet::new();
        
        if let Some(node_idx) = self.event_index.get(&event_id) {
            self.find_roots_recursive(*node_idx, &mut roots, &mut visited);
        }
        
        roots
    }
    
    pub fn get_impact_chain(&self, event_id: Uuid) -> Vec<Vec<EventNode>> {
        // Find all paths from this event forward
        let mut chains = Vec::new();
        
        if let Some(node_idx) = self.event_index.get(&event_id) {
            let mut current_path = Vec::new();
            self.find_impacts_recursive(*node_idx, &mut current_path, &mut chains);
        }
        
        chains
    }
}
```

### 9.5 Event Deduplication

Deduplication prevents duplicate processing of events in distributed systems:

```yaml
deduplication_strategies:
  content_hash:
    description: "Hash-based exact duplicate detection"
    implementation:
      hash_algorithm: "xxHash3"
      hash_fields: ["message_type", "source_agent_id", "payload"]
      cache_size: 100000
      ttl_seconds: 300
      
  semantic_deduplication:
    description: "Detect semantically equivalent events"
    rules:
      - name: "agent_status_updates"
        similarity_threshold: 0.95
        comparison_fields: ["agent_id", "status", "capabilities"]
        time_window_ms: 5000
        
      - name: "error_events"
        similarity_threshold: 0.90
        comparison_fields: ["error_code", "component", "stack_trace_hash"]
        time_window_ms: 60000
        
  sequence_deduplication:
    description: "Detect duplicate event sequences"
    configuration:
      sequence_length: 5
      similarity_metric: "levenshtein"
      threshold: 0.85
      sliding_window: true
```

High-performance deduplication implementation:

```rust
use xxhash_rust::xxh3::Xxh3;
use lru::LruCache;
use std::sync::RwLock;

pub struct EventDeduplicator {
    // Multiple deduplication strategies
    exact_cache: RwLock<LruCache<u64, DedupeEntry>>,
    semantic_cache: RwLock<HashMap<String, SemanticSignature>>,
    sequence_detector: SequenceDeduplicator,
    metrics: DeduplicationMetrics,
}

#[derive(Clone)]
struct DedupeEntry {
    event_id: Uuid,
    timestamp: DateTime<Utc>,
    occurrence_count: u32,
}

impl EventDeduplicator {
    pub fn is_duplicate(&self, event: &Message) -> DuplicateCheckResult {
        // Fast path: exact duplicate check
        let content_hash = self.compute_content_hash(event);
        
        if let Some(entry) = self.exact_cache.read().unwrap().peek(&content_hash) {
            if entry.timestamp.signed_duration_since(Utc::now()).num_seconds().abs() < 300 {
                self.metrics.exact_duplicates.fetch_add(1, Ordering::Relaxed);
                return DuplicateCheckResult::ExactDuplicate(entry.event_id);
            }
        }
        
        // Semantic duplicate check for specific message types
        if self.requires_semantic_check(&event.message_type) {
            if let Some(similar_event) = self.check_semantic_duplicate(event) {
                self.metrics.semantic_duplicates.fetch_add(1, Ordering::Relaxed);
                return DuplicateCheckResult::SemanticDuplicate(similar_event);
            }
        }
        
        // Sequence duplicate check
        if let Some(sequence_match) = self.sequence_detector.check_sequence(event) {
            self.metrics.sequence_duplicates.fetch_add(1, Ordering::Relaxed);
            return DuplicateCheckResult::SequenceDuplicate(sequence_match);
        }
        
        // Not a duplicate - add to caches
        self.add_to_caches(event, content_hash);
        DuplicateCheckResult::Unique
    }
    
    fn compute_content_hash(&self, event: &Message) -> u64 {
        let mut hasher = Xxh3::new();
        hasher.update(event.message_type.as_bytes());
        hasher.update(&event.source_agent_id.as_bytes());
        
        // Hash payload excluding volatile fields
        if let Ok(payload_bytes) = self.serialize_stable_payload(&event.payload) {
            hasher.update(&payload_bytes);
        }
        
        hasher.digest()
    }
}
```

### 9.6 Performance Optimizations for High-Volume Events

The event correlation system is optimized for processing millions of events per second:

```yaml
performance_optimizations:
  memory_management:
    - zero_copy_processing: "Avoid unnecessary data copies"
    - arena_allocation: "Pre-allocated memory pools"
    - cache_line_alignment: "Optimize for CPU cache"
    - numa_awareness: "NUMA-aware memory allocation"
    
  concurrency:
    - lock_free_structures: "Use atomic operations where possible"
    - sharded_processing: "Partition by correlation ID"
    - work_stealing: "Balance load across threads"
    - batch_processing: "Process events in batches"
    
  storage:
    - write_ahead_log: "Durability with performance"
    - column_store: "Efficient analytical queries"
    - compression: "Reduce storage footprint"
    - tiered_storage: "Hot/warm/cold data tiers"
    
  algorithms:
    - bloom_filters: "Probabilistic duplicate detection"
    - hyperloglog: "Cardinality estimation"
    - count_min_sketch: "Frequency estimation"
    - reservoir_sampling: "Statistical sampling"
```

Performance-critical correlation engine:

```rust
use crossbeam::channel::{bounded, Receiver, Sender};
use rayon::prelude::*;

pub struct HighPerformanceCorrelator {
    // Sharded processing channels
    shards: Vec<CorrelationShard>,
    // Ring buffer for recent events
    event_buffer: RingBuffer<Event>,
    // Lock-free statistics
    stats: Arc<AtomicStatistics>,
}

struct CorrelationShard {
    id: usize,
    receiver: Receiver<Event>,
    processor: ShardProcessor,
}

impl HighPerformanceCorrelator {
    pub fn process_event_batch(&self, events: Vec<Event>) {
        // Parallel preprocessing
        let preprocessed: Vec<_> = events
            .par_iter()
            .map(|e| (self.compute_shard(e), e.clone()))
            .collect();
            
        // Distribute to shards
        for (shard_id, event) in preprocessed {
            self.shards[shard_id].send(event);
        }
    }
    
    fn compute_shard(&self, event: &Event) -> usize {
        // Consistent hashing for shard assignment
        let hash = xxhash_rust::xxh3::xxh3_64(
            event.correlation_id.as_bytes()
        );
        (hash as usize) % self.shards.len()
    }
}

// Zero-allocation event processing
#[repr(align(64))] // Cache line aligned
struct EventBatch {
    events: [MaybeUninit<Event>; 1024],
    count: usize,
}
```

### 9.7 Integration with Message Framework

Event correlation integrates seamlessly with all message types:

| Message Type | Correlation Strategy | Key Attributes |
|-------------|---------------------|----------------|
| **[Agent Commands](./core-message-schemas.md#agent-command-message)** | Request-response correlation | correlation_id, reply_to |
| **[Task Assignment](./workflow-message-schemas.md#task-assignment-message)** | Workflow correlation | workflow_id, task_id |
| **[System Alerts](./system-message-schemas.md#system-alert-message)** | Causality tracking | incident_id, root_cause |
| **[Hook Events](./system-message-schemas.md#hook-event-message)** | Session correlation | session_id, hook_type |

Configuration for framework integration:

```json
{
  "correlation_integration": {
    "agent_communication": {
      "auto_correlation": true,
      "inherit_trace_context": true,
      "timeout_correlation_cleanup": 300000
    },
    "workflow_orchestration": {
      "workflow_id_extraction": "payload.workflow.id",
      "task_correlation_path": "payload.task.correlation_context",
      "preserve_causality_chain": true
    },
    "system_monitoring": {
      "alert_correlation_window": 60000,
      "incident_grouping": true,
      "root_cause_analysis": true
    },
    "cli_integration": {
      "session_correlation": true,
      "command_chaining": true,
      "interactive_correlation": true
    }
  }
}
```

## 10. Message Transformation Patterns

### 10.1 Protocol Adaptation

Transformation rules for adapting messages between different transport protocols. Transformations support:

- [NATS subject routing](./system-message-schemas.md#nats-subject-pattern-schemas) for message distribution
- [gRPC method mapping](../transport/grpc-transport.md) for synchronous operations
- [HTTP API conversion](../transport/http-transport.md) for web integration
- Cross-protocol [agent communication](./core-message-schemas.md#agent-communication-messages)

```yaml
transformation_rules:
  json_to_protobuf:
    - map_timestamps: "ISO string to Unix nanoseconds"
    - compress_payload: "Gzip compression for large payloads"
    - validate_required: "Ensure all required protobuf fields present"
    
  protobuf_to_json:
    - expand_timestamps: "Unix nanoseconds to ISO string"
    - decompress_payload: "Decompress gzipped payloads"
    - add_schema_version: "Include JSON schema version"
    
  nats_to_grpc:
    - extract_metadata: "Move NATS headers to gRPC metadata"
    - preserve_correlation: "Maintain correlation_id across protocols"
    - map_subjects: "Convert NATS subjects to gRPC method names"
    
  grpc_to_http:
    - convert_streaming: "Buffer streaming responses for HTTP"
    - map_status_codes: "gRPC status to HTTP status codes"
    - flatten_nested: "Flatten nested objects for query parameters"
```

### 10.2 Message Enrichment

Automatic message enhancement and metadata injection. Enrichment applies to:

- [Base message envelopes](./core-message-schemas.md#base-message-envelope) for metadata completion
- [Task assignments](./workflow-message-schemas.md#task-assignment-message) for capability matching
- [System alerts](./system-message-schemas.md#system-alert-message) for context injection
- [CLI hook events](./system-message-schemas.md#hook-event-message) for session context

```json
{
  "enrichment_rules": {
    "timestamp_injection": {
      "condition": "timestamp field missing",
      "action": "Add current UTC timestamp",
      "format": "ISO 8601 RFC 3339"
    },
    "correlation_generation": {
      "condition": "correlation_id field missing and reply_to present",
      "action": "Generate UUID v4 correlation_id",
      "propagation": "Include in all related messages"
    },
    "trace_propagation": {
      "condition": "Always for cross-service calls",
      "action": "Propagate or generate trace_id",
      "sampling": "Configurable rate based on service"
    },
    "agent_metadata": {
      "condition": "Message from known agent",
      "action": "Inject agent version and capabilities",
      "source": "Agent registry lookup"
    },
    "security_context": {
      "condition": "Authenticated session",
      "action": "Add security context metadata",
      "fields": ["user_id", "permissions", "security_level"]
    }
  }
}
```

### 10.3 Content Transformation

Field mapping and data type conversion rules. Transformations enable:

- Cross-version compatibility for [schema evolution](./message-framework.md#schema-version-management)
- [Agent registration](./core-message-schemas.md#agent-registration-message) format normalization
- [Workflow state](./workflow-message-schemas.md#workflow-state-synchronization-message) format conversion
- [System health data](./system-message-schemas.md#system-health-check-message) aggregation

```yaml
field_mappings:
  version_compatibility:
    v1_0_0_to_v1_1_0:
      - add_field: "schema_version" 
        default_value: "1.1.0"
      - rename_field: 
          from: "agent_type"
          to: "agent_classification"
      - split_field:
          from: "full_name"
          to: ["first_name", "last_name"]
          
  data_type_conversions:
    timestamp_normalization:
      - from: "Unix timestamp (seconds)"
        to: "ISO 8601 string"
        conversion: "multiply by 1000, convert to datetime, format ISO"
      - from: "Unix timestamp (milliseconds)"
        to: "ISO 8601 string" 
        conversion: "convert to datetime, format ISO"
        
  nested_object_handling:
    flattening_rules:
      - condition: "target protocol is HTTP query params"
        action: "Flatten nested objects with dot notation"
        example: "metadata.source -> metadata.source"
    expansion_rules:
      - condition: "source is flattened format"
        action: "Rebuild nested object structure"
        delimiter: "."
```

## 11. Schema Version Management

### 11.1 Versioning Strategy

Versioning strategy applies to all schema families:

- [Core message schemas](./core-message-schemas.md) for foundational compatibility
- [Workflow schemas](./workflow-message-schemas.md) for task management evolution
- [System schemas](./system-message-schemas.md) for CLI and monitoring updates

```json
{
  "versioning_strategy": {
    "semantic_versioning": {
      "format": "MAJOR.MINOR.PATCH",
      "major_change": "Breaking changes requiring code updates",
      "minor_change": "Backward-compatible additions",
      "patch_change": "Bug fixes and documentation"
    },
    "compatibility_policy": {
      "backward_compatibility": "Maintain for 2 major versions",
      "forward_compatibility": "Best effort with graceful degradation",
      "deprecation_period": "6 months minimum for major changes"
    },
    "schema_evolution": {
      "additive_changes": "Always allowed in minor versions",
      "field_removal": "Deprecated in minor, removed in major",
      "type_changes": "Only allowed in major versions",
      "constraint_tightening": "Only allowed in major versions"
    }
  }
}
```

### 11.2 Compatibility Matrix

Version compatibility and migration support across all message types:

- [Agent communication](./core-message-schemas.md#agent-communication-messages) backward compatibility
- [Task management](./workflow-message-schemas.md#task-management-messages) forward compatibility
- [System operation](./system-message-schemas.md#system-operation-messages) graceful degradation
- Cross-schema compatibility validation

```yaml
compatibility_matrix:
  v1.0.0:
    compatible_with: ["1.0.x"]
    migration_from: []
    deprecation_date: "2025-12-31"
    
  v1.1.0:
    compatible_with: ["1.0.x", "1.1.x"] 
    migration_from: ["1.0.0"]
    new_features:
      - "Enhanced error details"
      - "Agent health monitoring"
      - "Workflow coordination"
      
  v2.0.0:
    compatible_with: ["2.0.x"]
    migration_from: ["1.0.0", "1.1.0"]
    breaking_changes:
      - "Renamed agent_type to agent_classification"
      - "Required correlation_id for all requests"
      - "Changed timestamp format to nanosecond precision"

migration_tools:
  schema_converter:
    supports: ["1.0.0 -> 1.1.0", "1.1.0 -> 2.0.0"]
    validation: "Pre and post conversion validation"
    rollback: "Automatic rollback on conversion failure"
    
  version_negotiation:
    strategy: "Highest common version"
    fallback: "Graceful degradation to v1.0.0"
    discovery: "Schema registry lookup"
```

### 11.3 Schema Registry

Central schema registry for version management and discovery. Registry manages:

- [Core schema definitions](./core-message-schemas.md) and common types
- [Workflow schema versions](./workflow-message-schemas.md) and compatibility
- [System schema updates](./system-message-schemas.md) and CLI integration
- Cross-reference validation and dependency tracking

```json
{
  "schema_registry": {
    "base_url": "https://schemas.mister-smith.dev",
    "endpoints": {
      "list_schemas": "/schemas",
      "get_schema": "/schemas/{schema_id}",
      "get_version": "/schemas/{schema_id}/versions/{version}",
      "validate": "/validate/{schema_id}/{version}"
    },
    "caching": {
      "cache_duration": 3600,
      "cache_size": 1000,
      "cache_key_format": "{schema_id}:{version}"
    },
    "versioning": {
      "latest_alias": "Support for 'latest' version alias",
      "version_negotiation": "Automatic version compatibility checking",
      "schema_evolution": "Track schema evolution and migrations"
    }
  }
}
```

## 12. Implementation Guidelines

### 12.1 Code Generation

Framework for generating type-safe code from schemas. Code generation supports:

- [Agent communication](./core-message-schemas.md#agent-communication-messages) type safety
- [Workflow orchestration](./workflow-message-schemas.md#workflow-orchestration-messages) validation
- [System integration](./system-message-schemas.md#claude-cli-integration-messages) type checking
- Cross-language compatibility for multi-agent systems

```rust
// Example Rust code generation
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseMessage {
    pub message_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub schema_version: String,
    pub message_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_agent_id: Option<String>,
    #[serde(default = "default_priority")]
    pub priority: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

fn default_priority() -> u8 { 5 }

// Message type discriminated union
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "message_type")]
pub enum Message {
    #[serde(rename = "agent_command")]
    AgentCommand(AgentCommandMessage),
    #[serde(rename = "agent_status")]
    AgentStatus(AgentStatusMessage),
    #[serde(rename = "task_assignment")]
    TaskAssignment(TaskAssignmentMessage),
    // ... other message types
}
```

### 12.2 Validation Integration

Runtime validation with performance optimization. Integration patterns for:

- [Agent message validation](./core-message-schemas.md#agent-communication-messages) in communication pipelines
- [Task message validation](./workflow-message-schemas.md#task-management-messages) in workflow engines
- [System message validation](./system-message-schemas.md#system-operation-messages) in monitoring systems
- Real-time validation for [CLI integration](./system-message-schemas.md#claude-cli-integration-messages)

```rust
use jsonschema::{JSONSchema, ValidationError};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub struct MessageValidator {
    schemas: HashMap<String, Arc<JSONSchema>>,
    validation_level: ValidationLevel,
}

impl MessageValidator {
    pub fn validate_message(&self, message: &Value) -> Result<(), Vec<ValidationError>> {
        let message_type = message.get("message_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| vec![ValidationError::custom("missing message_type")])?;
            
        let schema = self.schemas.get(message_type)
            .ok_or_else(|| vec![ValidationError::custom("unknown message_type")])?;
            
        match self.validation_level {
            ValidationLevel::Strict => schema.validate(message).collect(),
            ValidationLevel::Standard => self.validate_essential_fields(message),
            ValidationLevel::Permissive => self.validate_minimal(message),
        }
    }
    
    fn validate_essential_fields(&self, message: &Value) -> Result<(), Vec<ValidationError>> {
        // Optimized validation for production use - validates critical fields only
        let mut errors = Vec::new();
        
        // Validate required base message fields
        if message.get("message_id").is_none() {
            errors.push(ValidationError::custom("Missing required field: message_id"));
        }
        
        if message.get("timestamp").is_none() {
            errors.push(ValidationError::custom("Missing required field: timestamp"));
        }
        
        if message.get("message_type").is_none() {
            errors.push(ValidationError::custom("Missing required field: message_type"));
        }
        
        // Validate message_id format (UUID)
        if let Some(msg_id) = message.get("message_id").and_then(|v| v.as_str()) {
            if uuid::Uuid::parse_str(msg_id).is_err() {
                errors.push(ValidationError::custom("Invalid message_id format: must be UUID"));
            }
        }
        
        // Validate timestamp format (ISO 8601)
        if let Some(timestamp) = message.get("timestamp").and_then(|v| v.as_str()) {
            if chrono::DateTime::parse_from_rfc3339(timestamp).is_err() {
                errors.push(ValidationError::custom("Invalid timestamp format: must be ISO 8601"));
            }
        }
        
        // Validate message size constraints
        let serialized_size = serde_json::to_string(message)
            .map(|s| s.len())
            .unwrap_or(0);
            
        if serialized_size > 1_048_576 { // 1MB limit
            errors.push(ValidationError::custom("Message exceeds maximum size limit"));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

### 12.3 Testing Strategies

Comprehensive testing approach for message schemas. Testing covers:

- [Core message schema](./core-message-schemas.md) validation and serialization
- [Workflow message](./workflow-message-schemas.md) end-to-end flow testing
- [System message](./system-message-schemas.md) integration and monitoring validation
- Cross-schema compatibility and transformation testing

```yaml
testing_strategy:
  unit_tests:
    - schema_validation_tests: "Test all valid and invalid message examples"
    - serialization_tests: "Round-trip testing for all formats"
    - compatibility_tests: "Cross-version compatibility validation"
    
  integration_tests:
    - end_to_end_flow: "Complete message flow through all components"
    - protocol_conversion: "Message transformation between protocols"
    - error_handling: "Proper error propagation and handling"
    
  performance_tests:
    - validation_benchmarks: "Measure validation performance"
    - serialization_benchmarks: "Compare serialization formats"
    - memory_usage: "Monitor memory consumption patterns"
    
  property_based_tests:
    - message_generation: "Generate random valid messages"
    - invariant_testing: "Verify schema invariants hold"
    - fuzzing: "Test with malformed message data"

test_data_generation:
  valid_examples:
    - minimal_messages: "Messages with only required fields"
    - complete_messages: "Messages with all optional fields"
    - edge_cases: "Boundary value testing"
    
  invalid_examples:
    - missing_required: "Test required field validation"
    - invalid_formats: "Test format validation"
    - constraint_violations: "Test constraint enforcement"
```

## 13. Security Considerations

### 13.1 Input Validation and Sanitization

Security-focused validation rules applied to all message types:

- [Agent communication](./core-message-schemas.md#agent-communication-messages) input sanitization
- [Task data](./workflow-message-schemas.md#task-assignment-message) payload validation
- [System monitoring](./system-message-schemas.md#system-operation-messages) data integrity
- [CLI integration](./system-message-schemas.md#claude-cli-integration-messages) injection prevention

```json
{
  "security_validation": {
    "input_sanitization": {
      "max_string_length": 1048576,
      "max_array_length": 10000,
      "max_object_properties": 1000,
      "max_nesting_depth": 32,
      "prohibited_patterns": [
        "javascript:",
        "data:",
        "vbscript:",
        "<script",
        "</script>",
        "{{.*}}",
        "${.*}"
      ]
    },
    "injection_prevention": {
      "sql_injection": "Escape SQL metacharacters",
      "xss_prevention": "HTML entity encoding",
      "command_injection": "Validate against allowed characters",
      "path_traversal": "Normalize and validate file paths"
    },
    "content_validation": {
      "validate_urls": "Check URL format and allowed schemes",
      "validate_emails": "RFC 5322 email format validation",
      "validate_file_types": "Whitelist allowed file extensions",
      "validate_encoding": "Ensure UTF-8 encoding compliance"
    }
  }
}
```

### 13.2 Message Encryption and Signing

Support for end-to-end encryption and message integrity across:

- [Agent-to-agent communication](./core-message-schemas.md#agent-communication-messages) security
- [Workflow coordination](./workflow-message-schemas.md#workflow-coordination-message) protection
- [System alert](./system-message-schemas.md#system-alert-message) integrity
- [CLI session](./system-message-schemas.md#claude-cli-integration-messages) encryption

```yaml
encryption_support:
  encryption_algorithms:
    - "AES-256-GCM" # For payload encryption
    - "ChaCha20-Poly1305" # Alternative encryption
    - "RSA-OAEP-256" # For key exchange
    
  signing_algorithms:
    - "Ed25519" # Digital signatures
    - "ECDSA-P256" # Alternative signing
    - "HMAC-SHA256" # Message authentication
    
  key_management:
    rotation_policy: "Monthly automatic rotation"
    key_derivation: "PBKDF2 with 100,000 iterations"
    secure_storage: "HSM or secure enclave"
    
  message_envelope:
    encrypted_payload: "Base64-encoded encrypted content"
    signature: "Digital signature of entire message"
    key_id: "Reference to encryption key"
    algorithm: "Encryption algorithm identifier"
```

## 14. Performance Optimization

### 14.1 Message Size Optimization

Strategies for reducing message overhead across all schema types:

- [Agent status updates](./core-message-schemas.md#agent-status-update-message) compression
- [Task assignment](./workflow-message-schemas.md#task-assignment-message) payload optimization
- [System health data](./system-message-schemas.md#system-health-check-message) aggregation
- [CLI hook events](./system-message-schemas.md#hook-event-message) efficient encoding

```yaml
size_optimization:
  field_optimization:
    - use_short_field_names: "For high-frequency messages"
    - optional_field_omission: "Skip null/empty fields"
    - enum_value_compression: "Use integers instead of strings"
    
  payload_compression:
    - compression_threshold: 1024 # bytes
    - algorithms: ["gzip", "lz4", "snappy"]
    - adaptive_compression: "Choose algorithm based on payload type"
    
  schema_optimization:
    - minimize_nesting: "Flatten nested structures where possible"
    - reuse_definitions: "Extensive use of $ref for common types"
    - efficient_validation: "Optimize schema for fast validation"

binary_formats:
  protobuf_optimization:
    - field_numbering: "Optimal field number assignment"
    - packed_encoding: "Use packed encoding for arrays"
    - oneof_fields: "Use oneof for discriminated unions"
    
  messagepack_optimization:
    - compact_integers: "Use smallest integer representation"
    - string_interning: "Reuse common string values"
    - binary_data: "Use binary type for large data"
```

### 14.2 Validation Performance

High-performance validation strategies optimized for:

- High-frequency [agent communication](./core-message-schemas.md#agent-communication-messages)
- Batch [workflow operations](./workflow-message-schemas.md#workflow-orchestration-messages)
- Real-time [system monitoring](./system-message-schemas.md#system-operation-messages)
- Interactive [CLI operations](./system-message-schemas.md#claude-cli-integration-messages)

```rust
// Fast-path validation for critical message types
pub struct FastPathValidator {
    // Pre-compiled validation rules for common cases
    essential_validators: HashMap<String, Box<dyn Fn(&Value) -> bool>>,
    
    // LRU cache for validation results
    validation_cache: LruCache<u64, ValidationResult>,
    
    // Statistics for optimization
    validation_stats: ValidationStats,
}

impl FastPathValidator {
    pub fn validate_fast(&mut self, message: &Value) -> ValidationResult {
        // Fast hash-based cache lookup
        let cache_key = self.hash_message(message);
        if let Some(cached_result) = self.validation_cache.get(&cache_key) {
            self.validation_stats.cache_hits += 1;
            return cached_result.clone();
        }
        
        // Fast-path validation for known message types
        if let Some(validator) = self.essential_validators.get(message_type) {
            let result = if validator(message) {
                ValidationResult::Valid
            } else {
                ValidationResult::Invalid(vec!["fast validation failed".into()])
            };
            
            self.validation_cache.put(cache_key, result.clone());
            return result;
        }
        
        // Fall back to full schema validation
        self.validate_full_schema(message)
    }
}
```

---

## Framework Integration

### Schema Support Matrix

| Framework Component | Core Messages | Workflow Messages | System Messages |
|-------------------|---------------|-------------------|------------------|
| **Validation** | ✓ Base envelope, Agent comm | ✓ Task mgmt, Coordination | ✓ CLI hooks, Health checks |
| **Serialization** | ✓ JSON, Protobuf, MessagePack | ✓ All formats + optimization | ✓ All formats + compression |
| **Event Correlation** | ✓ Request-response tracking | ✓ Workflow causality chains | ✓ Session correlation, Alert grouping |
| **Transformation** | ✓ Protocol adaptation | ✓ Workflow state mapping | ✓ CLI response formatting |
| **Versioning** | ✓ Foundation compatibility | ✓ Task evolution support | ✓ CLI integration updates |
| **Security** | ✓ Agent auth, encryption | ✓ Task data protection | ✓ System monitoring security |
| **Performance** | ✓ Agent comm optimization | ✓ Workflow batch processing | ✓ Real-time monitoring |

### Implementation Patterns

- **Agent Systems**: [Agent Communication](./agent-communication.md), [Agent Operations](./agent-operations.md)
- **Storage Systems**: [Persistence Operations](./persistence-operations.md), [Storage Patterns](./storage-patterns.md)
- **Transport Systems**: [NATS Transport](../transport/nats-transport.md), [gRPC Transport](../transport/grpc-transport.md)
- **Security Systems**: [Security Patterns](../security/security-patterns.md)

## Framework Integration with System Messages

### Integration with System Message Schemas

This framework directly supports all message types defined in [System Message Schemas](./system-message-schemas.md):

| Message Type | Framework Components Used | Routing Pattern |
|-------------|---------------------------|-----------------|
| **[Hook Event Message](./system-message-schemas.md#hook-event-message)** | Validation (essential fields), Correlation (session tracking), Content routing (hook type) | `cli.hooks.{hook_type}.{agent_id}` |
| **[Hook Response Message](./system-message-schemas.md#hook-response-message)** | Transformation (CLI formatting), Circuit breaker (timeout handling) | `cli.responses.{agent_id}` |
| **[System Alert Message](./system-message-schemas.md#system-alert-message)** | Event correlation (causality tracking), Content routing (severity-based) | `system.alerts.{severity}` |
| **[System Health Check](./system-message-schemas.md#system-health-check-message)** | Performance optimization (high-frequency), Aggregation (metrics collection) | `system.health.{component}` |

### NATS Subject Pattern Integration

Framework routing rules implement the [NATS subject patterns](./system-message-schemas.md#nats-subject-pattern-schemas):

```rust
// Hook event routing example
pub fn create_hook_routing_rules() -> Vec<ContentRoutingRule> {
    use serde_json::json;
    
    vec![
        ContentRoutingRule {
            name: "pre_task_hooks".to_string(),
            priority: 100,
            conditions: vec![
                Condition::FieldEquals {
                    path: "message_type".to_string(),
                    value: json!("hook_event"),
                },
                Condition::FieldEquals {
                    path: "payload.hook_type".to_string(),
                    value: json!("pre_task"),
                },
            ],
            destination: "cli.hooks.pre_task".to_string(),
            transformations: vec![
                Transformation::AddField {
                    path: "routing_metadata.correlation_required".to_string(),
                    value: json!(true),
                },
            ],
        },
        ContentRoutingRule {
            name: "critical_system_alerts".to_string(),
            priority: 95,
            conditions: vec![
                Condition::FieldEquals {
                    path: "message_type".to_string(),
                    value: json!("system_alert"),
                },
                Condition::FieldEquals {
                    path: "payload.severity".to_string(),
                    value: json!("critical"),
                },
            ],
            destination: "system.alerts.critical".to_string(),
            transformations: vec![
                Transformation::SetPriority { priority: 10 },
                Transformation::AddField {
                    path: "escalation.immediate".to_string(),
                    value: json!(true),
                },
            ],
        },
    ]
}
```

### Message Correlation Strategies

Framework correlation patterns support [system message correlation](./system-message-schemas.md#message-correlation-strategies):

- **Hook Event-Response Correlation**: Track CLI hook lifecycle using correlation_id
- **System Health Monitoring**: Aggregate health check messages by component
- **Alert Escalation Chains**: Maintain causality links for alert root cause analysis

## Navigation

This file is part of the Message Schema Documentation suite:

1. [Core Message Schemas](./core-message-schemas.md) - Foundation schemas and agent communication
2. [Workflow Message Schemas](./workflow-message-schemas.md) - Task management and workflow orchestration
3. [System Message Schemas](./system-message-schemas.md) - Claude CLI integration and system operations
4. **[Message Framework](./message-framework.md)** - Validation, serialization, and framework specifications *(current file)*

### Technical Implementation

- **Validation**: [Rules](#71-validation-rules-and-levels), [Error Codes](#72-error-code-classification), [Performance](#73-performance-optimization)
- **Serialization**: [JSON Standards](#81-json-serialization-standards), [Binary Formats](#82-binary-serialization-alternatives)
- **Event Correlation**: [Correlation ID Tracking](#91-correlation-id-generation-and-tracking), [Event Aggregation](#92-event-aggregation-patterns),
  [Temporal Windows](#9-event-correlation-logic), [Causality Chains](#9-event-correlation-logic),
  [Deduplication](#9-event-correlation-logic), [Performance](#9-event-correlation-logic)
- **Transformation**: [Protocol Adaptation](#8-serialization-specifications), [Enrichment](#8-serialization-specifications), [Content Mapping](#95-content-based-routing)
- **Management**: [Versioning](#8-serialization-specifications), [Compatibility](#8-serialization-specifications), [Registry](#94-topic-based-routing)

For the complete framework documentation, see the [Data Management Index](./CLAUDE.md).

*Message Schema Definitions v1.0.0 - Mister Smith AI Agent Framework*
