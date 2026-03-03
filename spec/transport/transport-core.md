---
title: transport-core
type: note
permalink: ms-framework/transport/transport-core
---

## Transport Core - Abstraction, Connection, Error & Security Patterns

## Agent Implementation Framework

> **Modularization Note**: This document contains core transport abstractions extracted from the complete transport layer specifications.
> For protocol-specific implementations, see companion files for NATS, gRPC, and HTTP specifics.
> **Technology Stack Reference**: See `/Users/mac-main/Mister-Smith/MisterSmith/ms-framework-docs/core-architecture/dependency-specifications.md` for authoritative dependency specifications

## Overview

This document defines foundational transport patterns for agent communication in the MisterSmith multi-agent framework.
Focus is on core abstractions, connection management, error handling, and security patterns that provide the foundation for protocol-specific implementations.

**Core Technologies**:

- **async-nats 0.34** - High-performance messaging and pub/sub
- **Tonic 0.11** - gRPC services and streaming
- **Axum 0.8** - HTTP/WebSocket API endpoints  
- **Tokio 1.38** - Async runtime foundation

**Document Scope**: Core transport abstractions and patterns. For protocol-specific implementations, see [transport-layer-specifications.md](./transport-layer-specifications.md).

## 1. Basic Message Transport Patterns

### 1.1 Request-Response Pattern

```rust
PATTERN RequestResponse:
    AGENT sends REQUEST to TARGET_AGENT
    TARGET_AGENT processes REQUEST
    TARGET_AGENT returns RESPONSE to AGENT
    
    Properties:
        - Synchronous communication
        - Direct addressing
        - Guaranteed response or timeout
```

### 1.2 Publish-Subscribe Pattern

```rust
PATTERN PublishSubscribe:
    PUBLISHER_AGENT publishes MESSAGE to TOPIC
    ALL SUBSCRIBER_AGENTS on TOPIC receive MESSAGE
    
    Properties:
        - Asynchronous broadcast
        - Topic-based routing
        - Multiple consumers
```

### 1.3 Queue Group Pattern

```rust
PATTERN QueueGroup:
    PRODUCER_AGENT sends TASK to QUEUE
    ONE WORKER_AGENT from GROUP receives TASK
    
    Properties:
        - Load balancing
        - Work distribution
        - Single consumer per message
```

### 1.4 Blackboard Pattern

```rust
PATTERN Blackboard:
    AGENTS write/read from SHARED_KNOWLEDGE_SPACE
    ALL AGENTS can observe changes to BLACKBOARD
    
    Properties:
        - Shared memory coordination
        - Decoupled collaboration
        - Knowledge persistence
        
    Implementation:
        - Shared store with versioning
        - Watch notifications on changes
        - Concurrent access control
```

## 5. Transport Abstraction

### 5.1 Generic Transport Interface

```rust
INTERFACE Transport:
    connect(config)
    disconnect()
    send(message) -> response
    subscribe(topic, handler)
    
IMPLEMENTATIONS:
    - NatsTransport
    - GrpcTransport
    - HttpTransport
```

### 5.2 Message Routing

```rust
ROUTING_LOGIC:
    RECEIVE message
    EXTRACT destination
    LOOKUP transport for destination
    FORWARD using appropriate protocol
    
FALLBACK:
    If primary transport fails
    Try secondary transport
    Log routing decision
```

## 6. Agent Communication Patterns

### 6.1 Core Agent Message Interfaces

```rust
AGENT_INTERFACES:
    Planner:
        create_plan(goal) -> TaskList
        refine_plan(feedback) -> TaskList
        
    Executor:
        execute_task(task) -> TaskOutput
        can_execute(task_type) -> bool
        
    Critic:
        evaluate(output, criteria) -> Feedback
        validate_plan(plan) -> ValidationResult
        
    Router:
        route_task(task) -> AgentId
        balance_load(tasks) -> TaskAssignments
        
    Memory:
        store(key, value, metadata) -> Result
        retrieve(key) -> Option<Value>
        query(pattern) -> List<Results>
```

### 6.2 Coordination Topologies

```rust
CENTRALIZED_PATTERN:
    ORCHESTRATOR maintains global_state
    ORCHESTRATOR assigns tasks to AGENTS
    AGENTS report results to ORCHESTRATOR
    
HIERARCHICAL_PATTERN:
    PARENT delegates to CHILDREN
    CHILDREN report to PARENT
    Multiple levels of delegation
    
PEER_TO_PEER_PATTERN:
    AGENTS communicate directly
    No central coordinator
    Self-organizing behavior
```

## 7. Advanced Connection Management

### 7.1 Connection Pool Architecture

```rust
INTERFACE ConnectionPoolManager {
    create_pool(config: PoolConfig) -> Pool
    monitor_health() -> HealthMetrics
    handle_backpressure() -> BackpressureAction
    scale_connections(metrics: LoadMetrics) -> ScalingDecision
}

CLASS AdvancedConnectionPool {
    PRIVATE config: PoolConfiguration
    PRIVATE connections: ConnectionSet
    PRIVATE health_monitor: HealthMonitor
    PRIVATE metrics_collector: MetricsCollector
    
    STRUCT PoolConfiguration {
        max_connections: Integer = 10
        min_connections: Integer = 2
        acquire_timeout: Duration = 30_seconds
        idle_timeout: Duration = 600_seconds
        max_lifetime: Duration = 3600_seconds
        health_check_interval: Duration = 30_seconds
        connection_recycling: RecyclingStrategy = FAST
    }
}
```

### 7.2 Protocol-Specific Pool Configurations

```rust
// NATS Connection Pool with Deadpool Pattern
CLASS NatsConnectionPool IMPLEMENTS ConnectionPoolManager {
    FUNCTION create_nats_pool(config: NatsConfig) -> NatsPool {
        pool_config = {
            max_connections: config.max_connections,
            reconnect_strategy: ExponentialBackoff {
                initial_delay: Duration.millis(100),
                max_delay: Duration.seconds(5),
                max_attempts: 10
            },
            connection_timeout: Duration.seconds(10),
            subscription_capacity: 1000,
            jetstream_domain: config.jetstream_domain
        }
        
        RETURN create_managed_pool(pool_config)
    }
    
    FUNCTION configure_nats_connection(conn: NatsConnection) -> Result {
        // Post-connection configuration
        conn.set_subscription_capacity(1000)
        conn.configure_jetstream(js_config)
        conn.enable_heartbeat(Duration.seconds(30))
        RETURN Success()
    }
}

// PostgreSQL Connection Pool (coordinated with data layer)
CLASS PostgresConnectionPool IMPLEMENTS ConnectionPoolManager {
    FUNCTION create_postgres_pool(config: PgConfig) -> PgPool {
        pool_config = {
            max_connections: config.max_connections,
            min_connections: config.min_connections,
            acquire_timeout: Duration.seconds(30),
            idle_timeout: Duration.minutes(10),
            max_lifetime: Duration.hours(2),
            after_connect: configure_session
        }
        
        RETURN deadpool_postgres.create_pool(pool_config)
    }
    
    FUNCTION configure_session(conn: PgConnection) -> Result {
        // Session-level configuration
        conn.execute("SET application_name = 'agent_transport'")
        conn.execute("SET statement_timeout = '30s'")
        conn.execute("SET idle_in_transaction_session_timeout = '60s'")
        RETURN Success()
    }
}

// gRPC Connection Pool
CLASS GrpcConnectionPool IMPLEMENTS ConnectionPoolManager {
    FUNCTION create_grpc_pool(config: GrpcConfig) -> GrpcPool {
        pool_config = {
            max_connections: config.max_connections,
            keep_alive_interval: Duration.seconds(60),
            keep_alive_timeout: Duration.seconds(5),
            connect_timeout: Duration.seconds(10),
            http2_flow_control: true,
            max_message_size: 4_MB
        }
        
        RETURN create_grpc_pool_with_config(pool_config)
    }
}
```

### 7.3 Connection String Templates and Configuration

```rust
CLASS ConnectionStringManager {
    ENUM ProtocolType {
        NATS,
        POSTGRESQL,
        GRPC,
        HTTP
    }
    
    FUNCTION build_connection_string(
        protocol: ProtocolType, 
        config: Map<String, String>
    ) -> String {
        SWITCH protocol {
            CASE NATS:
                RETURN build_nats_url(config)
            CASE POSTGRESQL:
                RETURN build_postgres_url(config)
            CASE GRPC:
                RETURN build_grpc_url(config)
            CASE HTTP:
                RETURN build_http_url(config)
        }
    }
    
    FUNCTION build_nats_url(config: Map<String, String>) -> String {
        // Support multiple formats:
        // nats://user:pass@host:4222
        // nats://host1:4222,host2:4222,host3:4222 (cluster)
        // tls://user:pass@host:4222 (secure)
        
        IF config.contains("cluster_hosts") THEN
            hosts = config.get("cluster_hosts").split(",")
            RETURN "nats://" + hosts.join(",")
        ELSE
            user_pass = build_auth_string(config)
            host_port = config.get("host", "localhost") + ":" + config.get("port", "4222")
            protocol = config.get("tls", "false") == "true" ? "tls" : "nats"
            RETURN protocol + "://" + user_pass + host_port
        END IF
    }
    
    FUNCTION build_postgres_url(config: Map<String, String>) -> String {
        // Support multiple formats:
        // postgres://user:pass@host:port/database
        // postgres://%2Fvar%2Frun%2Fpostgresql/database (Unix socket)
        // postgres://host/db?application_name=agent&sslmode=require
        
        IF config.contains("socket_path") THEN
            encoded_path = url_encode(config.get("socket_path"))
            database = config.get("database", "postgres")
            RETURN "postgres://" + encoded_path + "/" + database
        ELSE
            user_pass = build_auth_string(config)
            host_port = config.get("host", "localhost") + ":" + config.get("port", "5432")
            database = config.get("database", "postgres")
            query_params = build_query_params(config)
            RETURN "postgres://" + user_pass + host_port + "/" + database + query_params
        END IF
    }
}

// Environment-based configuration loading
CLASS EnvironmentConfigLoader {
    FUNCTION load_nats_config() -> Map<String, String> {
        config = Map()
        config.put("host", env.get("NATS_HOST", "localhost"))
        config.put("port", env.get("NATS_PORT", "4222"))
        config.put("user", env.get("NATS_USER", ""))
        config.put("password", env.get("NATS_PASSWORD", ""))
        config.put("cluster_hosts", env.get("NATS_CLUSTER", ""))
        config.put("tls", env.get("NATS_TLS", "false"))
        RETURN config
    }
    
    FUNCTION load_postgres_config() -> Map<String, String> {
        config = Map()
        config.put("host", env.get("PG_HOST", "localhost"))
        config.put("port", env.get("PG_PORT", "5432"))
        config.put("user", env.get("PG_USER", "postgres"))
        config.put("password", env.get("PG_PASSWORD", ""))
        config.put("database", env.get("PG_DATABASE", "postgres"))
        config.put("socket_path", env.get("PG_SOCKET_PATH", ""))
        config.put("application_name", env.get("PG_APP_NAME", "agent_transport"))
        config.put("sslmode", env.get("PG_SSL_MODE", "prefer"))
        RETURN config
    }
}
```

### 7.4 Advanced Health Checking and Recovery

```rust
CLASS AdvancedHealthMonitor {
    PRIVATE health_checkers: Map<ProtocolType, HealthChecker>
    PRIVATE circuit_breakers: Map<String, CircuitBreaker>
    PRIVATE recovery_strategies: Map<ProtocolType, RecoveryStrategy>
    
    INTERFACE HealthChecker {
        check_health() -> HealthStatus
        recover_connection() -> RecoveryResult
        get_metrics() -> HealthMetrics
    }
    
    CLASS NatsHealthChecker IMPLEMENTS HealthChecker {
        FUNCTION check_health() -> HealthStatus {
            TRY {
                start_time = now()
                response = connection.ping()
                latency = now() - start_time
                
                IF latency > Duration.seconds(1) THEN
                    RETURN HealthStatus.DEGRADED
                ELSE
                    RETURN HealthStatus.HEALTHY
                END IF
            } CATCH (error) {
                RETURN HealthStatus.UNHEALTHY
            }
        }
        
        FUNCTION recover_connection() -> RecoveryResult {
            // Implement exponential backoff reconnection
            attempts = 0
            max_attempts = 5
            backoff = Duration.millis(100)
            
            WHILE attempts < max_attempts {
                TRY {
                    connection.reconnect()
                    IF check_health() == HEALTHY THEN
                        RETURN RecoveryResult.SUCCESS
                    END IF
                } CATCH (error) {
                    log_error("Reconnection attempt failed", error)
                }
                
                sleep(backoff)
                backoff = min(backoff * 2, Duration.seconds(5))
                attempts += 1
            }
            
            RETURN RecoveryResult.FAILED
        }
    }
    
    CLASS PostgresHealthChecker IMPLEMENTS HealthChecker {
        FUNCTION check_health() -> HealthStatus {
            TRY {
                start_time = now()
                result = connection.execute("SELECT 1")
                latency = now() - start_time
                
                // Check for slow queries
                IF latency > Duration.millis(100) THEN
                    RETURN HealthStatus.DEGRADED
                ELSE
                    RETURN HealthStatus.HEALTHY
                END IF
            } CATCH (error) {
                RETURN HealthStatus.UNHEALTHY
            }
        }
    }
}

// Circuit breaker pattern for connection management
CLASS ConnectionCircuitBreaker {
    ENUM CircuitState {
        CLOSED,     // Normal operation
        OPEN,       // Failing fast, not attempting connections
        HALF_OPEN   // Testing if service has recovered
    }
    
    PRIVATE state: CircuitState = CLOSED
    PRIVATE failure_count: Integer = 0
    PRIVATE last_failure_time: Timestamp
    PRIVATE failure_threshold: Integer = 5
    PRIVATE recovery_timeout: Duration = Duration.seconds(60)
    
    FUNCTION attempt_connection() -> ConnectionResult {
        SWITCH state {
            CASE CLOSED:
                RETURN try_connection()
            CASE OPEN:
                IF now() - last_failure_time > recovery_timeout THEN
                    state = HALF_OPEN
                    RETURN try_connection()
                ELSE
                    RETURN ConnectionResult.CIRCUIT_OPEN
                END IF
            CASE HALF_OPEN:
                RETURN try_connection()
        }
    }
    
    FUNCTION record_success() {
        failure_count = 0
        state = CLOSED
    }
    
    FUNCTION record_failure() {
        failure_count += 1
        last_failure_time = now()
        
        IF failure_count >= failure_threshold THEN
            state = OPEN
        END IF
    }
}
```

### 7.5 Resource Limits and Backpressure Management

```rust
CLASS ResourceLimitManager {
    STRUCT ResourceLimits {
        max_memory_usage: Bytes = 512_MB
        max_connection_queue_size: Integer = 1000
        max_pending_requests: Integer = 10000
        connection_acquire_timeout: Duration = Duration.seconds(30)
        request_timeout: Duration = Duration.seconds(30)
        max_concurrent_operations: Integer = 100
    }
    
    CLASS BackpressureController {
        PRIVATE current_load: LoadMetrics
        PRIVATE resource_limits: ResourceLimits
        PRIVATE adaptive_throttling: AdaptiveThrottler
        
        FUNCTION handle_backpressure(request: ConnectionRequest) -> BackpressureAction {
            current_metrics = collect_current_metrics()
            
            // Check memory usage
            IF current_metrics.memory_usage > resource_limits.max_memory_usage * 0.9 THEN
                RETURN BackpressureAction.REJECT_REQUEST
            END IF
            
            // Check connection queue size
            IF current_metrics.queue_size > resource_limits.max_connection_queue_size THEN
                RETURN BackpressureAction.QUEUE_FULL
            END IF
            
            // Check pending request count
            IF current_metrics.pending_requests > resource_limits.max_pending_requests THEN
                RETURN BackpressureAction.THROTTLE_REQUEST
            END IF
            
            // Apply adaptive throttling based on success rate
            IF current_metrics.success_rate < 0.95 THEN
                throttle_delay = adaptive_throttling.calculate_delay(current_metrics)
                RETURN BackpressureAction.DELAY_REQUEST(throttle_delay)
            END IF
            
            RETURN BackpressureAction.ALLOW_REQUEST
        }
        
        FUNCTION collect_current_metrics() -> LoadMetrics {
            RETURN LoadMetrics {
                memory_usage: get_memory_usage(),
                queue_size: get_connection_queue_size(),
                pending_requests: get_pending_request_count(),
                success_rate: get_recent_success_rate(),
                average_latency: get_average_latency()
            }
        }
    }
    
    // Protocol-specific backpressure handling
    CLASS NatsBackpressureHandler {
        FUNCTION handle_slow_consumer() {
            // NATS built-in slow consumer protection
            subscription.set_pending_limits(1000, 50_MB)
            
            // Custom overflow handling
            IF subscription.pending_messages() > 800 THEN
                log_warning("Approaching NATS subscription limit")
                trigger_load_shedding()
            END IF
        }
        
        FUNCTION trigger_load_shedding() {
            // Drop non-critical messages
            // Increase processing parallelism
            // Signal upstream producers to slow down
        }
    }
}
```

### 7.6 Performance Monitoring and Metrics

```rust
CLASS ConnectionPerformanceMonitor {
    PRIVATE metrics_collector: MetricsCollector
    PRIVATE alert_manager: AlertManager
    
    STRUCT ConnectionMetrics {
        pool_size: Integer
        active_connections: Integer
        idle_connections: Integer
        pending_acquisitions: Integer
        total_acquisitions: Counter
        failed_acquisitions: Counter
        acquisition_time_histogram: Histogram
        connection_lifetime_histogram: Histogram
        health_check_success_rate: Gauge
    }
    
    FUNCTION collect_metrics() -> ConnectionMetrics {
        RETURN ConnectionMetrics {
            pool_size: connection_pool.size(),
            active_connections: connection_pool.active_count(),
            idle_connections: connection_pool.idle_count(),
            pending_acquisitions: connection_pool.pending_count(),
            total_acquisitions: acquisition_counter.value(),
            failed_acquisitions: failure_counter.value(),
            acquisition_time_histogram: acquisition_timer.snapshot(),
            connection_lifetime_histogram: lifetime_timer.snapshot(),
            health_check_success_rate: health_success_rate.value()
        }
    }
    
    FUNCTION monitor_thresholds() {
        metrics = collect_metrics()
        
        // Connection pool utilization alerts
        utilization = metrics.active_connections / metrics.pool_size
        IF utilization > 0.9 THEN
            alert_manager.trigger_alert(AlertType.HIGH_POOL_UTILIZATION, utilization)
        END IF
        
        // Acquisition time alerts
        p95_acquisition_time = metrics.acquisition_time_histogram.percentile(95)
        IF p95_acquisition_time > Duration.seconds(5) THEN
            alert_manager.trigger_alert(AlertType.SLOW_ACQUISITION, p95_acquisition_time)
        END IF
        
        // Failed acquisition rate alerts
        failure_rate = metrics.failed_acquisitions / metrics.total_acquisitions
        IF failure_rate > 0.05 THEN
            alert_manager.trigger_alert(AlertType.HIGH_FAILURE_RATE, failure_rate)
        END IF
        
        // Health check alerts
        IF metrics.health_check_success_rate < 0.95 THEN
            alert_manager.trigger_alert(AlertType.HEALTH_CHECK_FAILURES, 
                                      metrics.health_check_success_rate)
        END IF
    }
    
    FUNCTION export_metrics_for_prometheus() -> PrometheusMetrics {
        // Export metrics in Prometheus format for monitoring
        metrics = collect_metrics()
        
        prometheus_metrics = PrometheusMetrics()
        prometheus_metrics.add_gauge("connection_pool_size", metrics.pool_size)
        prometheus_metrics.add_gauge("connection_pool_active", metrics.active_connections)
        prometheus_metrics.add_gauge("connection_pool_idle", metrics.idle_connections)
        prometheus_metrics.add_counter("connection_acquisitions_total", metrics.total_acquisitions)
        prometheus_metrics.add_counter("connection_acquisition_failures_total", metrics.failed_acquisitions)
        prometheus_metrics.add_histogram("connection_acquisition_duration_seconds", 
                                        metrics.acquisition_time_histogram)
        
        RETURN prometheus_metrics
    }
}
```

## 8. Error Handling Patterns

### 8.1 Basic Retry Logic

```rust
RETRY_PATTERN:
    attempts = 0
    WHILE attempts < max_retries:
        TRY:
            SEND message
            RETURN success
        CATCH error:
            attempts += 1
            WAIT backoff_time
    RETURN failure
```

### 8.2 Timeout Handling

```rust
TIMEOUT_PATTERN:
    START timer(timeout_duration)
    SEND request
    WAIT for response OR timeout
    IF timeout:
        CANCEL request
        RETURN timeout_error
    ELSE:
        RETURN response
```

## 9. Basic Security

### 9.1 TLS Configuration

```rust
TLS_SETUP:
    LOAD certificates
    CONFIGURE tls_config:
        - server_cert
        - server_key
        - ca_cert (required for mTLS)
    APPLY to transport layer
    
mTLS_PATTERN:
    SERVER_CONFIG:
        cert_file: "./certs/nats-server.crt"
        key_file:  "./certs/nats-server.key"
        ca_file:   "./certs/ca.crt"
        verify: true  # Enforce client certificates
        
    CLIENT_CONFIG:
        require_tls: true
        root_certificates: ca.crt
        client_certificate: client.crt
        client_key: client.key
```

### 9.2 Authentication Pattern

```rust
AUTH_FLOW:
    CLIENT sends credentials
    SERVER validates credentials
    SERVER returns token
    CLIENT includes token in requests
    SERVER validates token on each request
    
SUBJECT_AUTHORIZATION:
    PER_USER_ACL:
        admin_permissions:
            publish: [ ">" ]      # Full access
            subscribe: [ ">" ]
            
        tenant_permissions:
            publish: { allow: ["tenantA.>"] }
            subscribe: { allow: ["tenantA.>"] }
            
    ACCOUNT_ISOLATION:
        - Each tenant gets NATS account
        - Complete namespace separation
        - Built-in multi-tenancy
        
    SECURITY_PRINCIPLES:
        - Never share accounts between tenants
        - Always enforce mTLS
        - Apply least privilege subjects
        - Set resource quotas per account
        - Monitor wildcard usage
```

### 9.3 Zero-Downtime Key Rotation

```rust
KEY_ROTATION_PATTERN:
    STATE_MACHINE:
        KeyAActive -> StagingNewKey
        StagingNewKey -> ReloadingConfig
        ReloadingConfig -> KeyBActive
        
    IMPLEMENTATION:
        SIGNAL_HANDLER(SIGHUP):
            LOAD new_certificates
            ATOMIC_SWAP certificates
            UPDATE connections
            NO_DOWNTIME
```

## 10. Implementation Guidelines

### 10.1 Agent Integration

Agents implementing transport should:

1. Choose appropriate pattern for use case
2. Handle connection failures gracefully
3. Implement basic retry logic
4. Log communication events

### 10.2 Testing Patterns

```rust
TEST_SCENARIOS:
    - Connection establishment
    - Message delivery
    - Timeout handling
    - Reconnection logic
    - Basic error cases
```

## 11. Configuration Templates

### 11.1 NATS Configuration

```rust
NATS_CONFIG:
    servers: ["nats://localhost:4222"]
    max_reconnects: 5
    reconnect_wait: 2_seconds
    timeout: 10_seconds
```

### 11.2 gRPC Configuration

```rust
GRPC_CONFIG:
    address: "localhost:50051"
    timeout: 30_seconds
    keepalive: 60_seconds
    max_message_size: 4_mb
```

### 11.3 HTTP Configuration

```rust
HTTP_CONFIG:
    bind_address: "0.0.0.0:8080"
    request_timeout: 30_seconds
    max_connections: 100
```

## 15. Error Response Standards

### 15.1 Standardized Error Codes

```json
ERROR_CODES: {
  "VALIDATION": {
    "INVALID_REQUEST": {
      "code": "E1001",
      "http_status": 400,
      "message": "Request validation failed",
      "retryable": false
    },
    "MISSING_REQUIRED_FIELD": {
      "code": "E1002",
      "http_status": 400,
      "message": "Required field is missing",
      "retryable": false
    },
    "INVALID_FIELD_FORMAT": {
      "code": "E1003",
      "http_status": 400,
      "message": "Field format is invalid",
      "retryable": false
    }
  },
  "AUTHENTICATION": {
    "INVALID_TOKEN": {
      "code": "E2001",
      "http_status": 401,
      "message": "Authentication token is invalid",
      "retryable": false
    },
    "TOKEN_EXPIRED": {
      "code": "E2002",
      "http_status": 401,
      "message": "Authentication token has expired",
      "retryable": true
    },
    "INSUFFICIENT_PERMISSIONS": {
      "code": "E2003",
      "http_status": 403,
      "message": "Insufficient permissions for this operation",
      "retryable": false
    }
  },
  "RESOURCE": {
    "AGENT_NOT_FOUND": {
      "code": "E3001",
      "http_status": 404,
      "message": "Agent not found",
      "retryable": false
    },
    "TASK_NOT_FOUND": {
      "code": "E3002",
      "http_status": 404,
      "message": "Task not found",
      "retryable": false
    },
    "AGENT_ALREADY_EXISTS": {
      "code": "E3003",
      "http_status": 409,
      "message": "Agent already exists",
      "retryable": false
    }
  },
  "SYSTEM": {
    "INTERNAL_ERROR": {
      "code": "E5001",
      "http_status": 500,
      "message": "Internal server error",
      "retryable": true
    },
    "SERVICE_UNAVAILABLE": {
      "code": "E5002",
      "http_status": 503,
      "message": "Service temporarily unavailable",
      "retryable": true
    },
    "TIMEOUT": {
      "code": "E5003",
      "http_status": 504,
      "message": "Request timeout",
      "retryable": true
    }
  },
  "AGENT": {
    "AGENT_OFFLINE": {
      "code": "E4001",
      "http_status": 503,
      "message": "Agent is offline",
      "retryable": true
    },
    "AGENT_BUSY": {
      "code": "E4002",
      "http_status": 429,
      "message": "Agent is at capacity",
      "retryable": true
    },
    "CAPABILITY_NOT_SUPPORTED": {
      "code": "E4003",
      "http_status": 400,
      "message": "Agent does not support required capability",
      "retryable": false
    }
  }
}
```

### 15.2 Error Response Format

```json
ERROR_RESPONSE_SCHEMA: {
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["error_code", "message", "timestamp"],
  "properties": {
    "error_code": {
      "type": "string",
      "pattern": "^E\\d{4}$",
      "description": "Standardized error code"
    },
    "message": {
      "type": "string",
      "description": "Human-readable error message"
    },
    "timestamp": {
      "type": "string",
      "format": "date-time",
      "description": "When the error occurred"
    },
    "trace_id": {
      "type": "string",
      "format": "uuid",
      "description": "Unique identifier for tracing this error"
    },
    "details": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "field": {
            "type": "string",
            "description": "Field that caused the error"
          },
          "issue": {
            "type": "string",
            "description": "Specific issue with the field"
          },
          "code": {
            "type": "string",
            "description": "Field-specific error code"
          }
        }
      }
    },
    "retry_after": {
      "type": "integer",
      "description": "Seconds to wait before retrying (for retryable errors)"
    },
    "documentation_url": {
      "type": "string",
      "format": "uri",
      "description": "Link to relevant documentation"
    }
  }
}
```

### 15.3 Retry Policies

```yaml
RETRY_POLICIES:
  exponential_backoff:
    initial_delay: 1000      # milliseconds
    max_delay: 30000         # milliseconds
    backoff_factor: 2.0
    jitter: true
    max_attempts: 5

  linear_backoff:
    initial_delay: 1000      # milliseconds
    delay_increment: 1000    # milliseconds
    max_delay: 10000         # milliseconds
    max_attempts: 3

  immediate_retry:
    delay: 0
    max_attempts: 2

RETRY_CONDITIONS:
  retryable_errors:
    - "E2002"  # TOKEN_EXPIRED
    - "E5001"  # INTERNAL_ERROR
    - "E5002"  # SERVICE_UNAVAILABLE
    - "E5003"  # TIMEOUT
    - "E4001"  # AGENT_OFFLINE
    - "E4002"  # AGENT_BUSY

  non_retryable_errors:
    - "E1001"  # INVALID_REQUEST
    - "E1002"  # MISSING_REQUIRED_FIELD
    - "E2001"  # INVALID_TOKEN
    - "E2003"  # INSUFFICIENT_PERMISSIONS
    - "E3001"  # AGENT_NOT_FOUND
    - "E4003"  # CAPABILITY_NOT_SUPPORTED

CIRCUIT_BREAKER:
  failure_threshold: 5
  recovery_timeout: 30000    # milliseconds
  test_request_volume: 3
  minimum_throughput: 10
```

## 16. Connection Pool Configurations

### 16.1 NATS Connection Pool

```yaml
NATS_CONNECTION_POOL:
  servers:
    - "nats://nats-01.internal:4222"
    - "nats://nats-02.internal:4222"
    - "nats://nats-03.internal:4222"
  
  connection_settings:
    max_connections: 100
    max_idle_connections: 20
    connection_timeout: 10000    # milliseconds
    ping_interval: 60000         # milliseconds
    max_reconnects: 10
    reconnect_wait: 2000         # milliseconds
    reconnect_jitter: 1000       # milliseconds
    disconnect_error_handler: true
    async_error_handler: true

  subscription_settings:
    max_subscriptions: 1000
    subscription_timeout: 5000   # milliseconds
    pending_message_limit: 1000
    pending_byte_limit: 10485760 # 10MB
    subscription_capacity: 1000

  tls_config:
    enabled: true
    cert_file: "certs/client.crt"
    key_file: "certs/client.key"
    ca_file: "certs/ca.crt"
    verify_certificates: true
    cipher_suites:
      - "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384"
      - "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256"

  jetstream_config:
    max_ack_pending: 1000
    ack_wait: 30000             # milliseconds
    max_deliver: 3
    delivery_policy: "new"
    replay_policy: "instant"

  monitoring:
    stats_interval: 60000       # milliseconds
    slow_consumer_threshold: 1000
    enable_metrics: true
    enable_tracing: true
```

### 16.2 gRPC Connection Pool

```yaml
GRPC_CONNECTION_POOL:
  target_addresses:
    - "grpc-01.internal:50051"
    - "grpc-02.internal:50051"
    - "grpc-03.internal:50051"

  connection_settings:
    max_connections_per_target: 10
    max_idle_connections: 5
    connection_timeout: 15000    # milliseconds
    keepalive_time: 60000        # milliseconds
    keepalive_timeout: 5000      # milliseconds
    keepalive_without_stream: true
    max_connection_idle: 300000  # milliseconds
    max_connection_age: 600000   # milliseconds

  call_settings:
    max_message_size: 16777216   # 16MB
    max_header_list_size: 8192   # 8KB
    call_timeout: 30000          # milliseconds
    retry_enabled: true
    max_retry_attempts: 3
    initial_backoff: 1000        # milliseconds
    max_backoff: 10000           # milliseconds
    backoff_multiplier: 2.0

  tls_config:
    enabled: true
    server_name_override: ""
    cert_file: "certs/client.crt"
    key_file: "certs/client.key"
    ca_file: "certs/ca.crt"
    insecure_skip_verify: false

  load_balancing:
    policy: "round_robin"       # round_robin, least_request, consistent_hash
    health_check_enabled: true
    health_check_interval: 30000 # milliseconds

  monitoring:
    enable_stats: true
    stats_tags:
      - "service"
      - "method"
      - "status"
    enable_tracing: true
    trace_sampling_rate: 0.1
```

### 16.3 HTTP Connection Pool

```yaml
HTTP_CONNECTION_POOL:
  max_connections: 200
  max_idle_connections: 50
  max_connections_per_host: 20
  idle_connection_timeout: 90000  # milliseconds
  connection_timeout: 10000       # milliseconds
  request_timeout: 30000          # milliseconds
  response_header_timeout: 10000  # milliseconds
  tls_handshake_timeout: 10000    # milliseconds
  expect_continue_timeout: 1000   # milliseconds

  retry_config:
    max_retries: 3
    initial_backoff: 1000         # milliseconds
    max_backoff: 10000           # milliseconds
    backoff_multiplier: 2.0
    retryable_status_codes:
      - 429  # Too Many Requests
      - 502  # Bad Gateway
      - 503  # Service Unavailable
      - 504  # Gateway Timeout

  rate_limiting:
    requests_per_second: 100
    burst_size: 200
    per_host_limit: 50

  compression:
    enabled: true
    algorithms:
      - "gzip"
      - "deflate"
    min_size: 1024              # bytes

  tls_config:
    min_version: "TLS1.2"
    max_version: "TLS1.3"
    cipher_suites:
      - "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384"
      - "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256"
    verify_certificates: true
    ca_file: "certs/ca.crt"
    cert_file: "certs/client.crt"
    key_file: "certs/client.key"

  monitoring:
    enable_metrics: true
    metrics_interval: 60000       # milliseconds
    enable_access_logs: true
    log_request_body: false
    log_response_body: false
```

## 17. Serialization Specifications

### 17.1 JSON Schema Definitions

```json
SERIALIZATION_STANDARDS: {
  "encoding": "UTF-8",
  "format": "JSON",
  "validation": "JSON Schema Draft 07",
  "date_format": "ISO 8601 (RFC 3339)",
  "number_precision": "IEEE 754 double precision",
  "string_max_length": 1048576,
  "object_max_depth": 32,
  "array_max_length": 10000
}

BASE_MESSAGE_SCHEMA: {
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://schemas.mister-smith.dev/base-message.json",
  "type": "object",
  "required": ["message_id", "timestamp", "version"],
  "properties": {
    "message_id": {
      "type": "string",
      "format": "uuid",
      "description": "Unique message identifier"
    },
    "timestamp": {
      "type": "string",
      "format": "date-time",
      "description": "Message creation timestamp in ISO 8601 format"
    },
    "version": {
      "type": "string",
      "pattern": "^\\d+\\.\\d+\\.\\d+$",
      "description": "Schema version (semantic versioning)"
    },
    "correlation_id": {
      "type": "string",
      "format": "uuid",
      "description": "Optional correlation identifier for message tracking"
    },
    "trace_id": {
      "type": "string",
      "format": "uuid",
      "description": "Distributed tracing identifier"
    }
  }
}

VALIDATION_RULES: {
  "strict_mode": true,
  "additional_properties": false,
  "validate_formats": true,
  "validate_required": true,
  "fail_on_unknown_keywords": true,
  "error_aggregation": true,
  "max_validation_errors": 100
}

COMPRESSION_SETTINGS: {
  "algorithm": "gzip",
  "compression_level": 6,
  "min_size_threshold": 1024,
  "content_types": [
    "application/json",
    "text/plain",
    "application/xml"
  ]
}

SECURITY_CONSTRAINTS: {
  "max_string_length": 1048576,
  "max_array_length": 10000,
  "max_object_properties": 1000,
  "max_nesting_depth": 32,
  "prohibited_patterns": [
    "javascript:",
    "data:",
    "vbscript:",
    "<script",
    "</script>"
  ],
  "sanitization": {
    "html_entities": true,
    "sql_injection": true,
    "xss_prevention": true
  }
}
```

### 17.2 Binary Serialization Patterns

```rust
BINARY_SERIALIZATION_APPROACH:
    Protocol Buffers (protobuf):
        - Compact binary format
        - Schema evolution support
        - Language-neutral definitions
        - High performance serialization
        
    MessagePack:
        - JSON-like structure, binary encoding
        - Smaller than JSON
        - Faster than JSON parsing
        
    Bincode:
        - Rust-native binary serialization
        - Maximum performance for Rust-to-Rust
        - Minimal overhead
        
USAGE_PATTERNS:
    High-frequency messages: Use protobuf CompactMessage
    Batch operations: Use protobuf MessageBatch
    Inter-service: Use protobuf BaseMessage
    Internal agent: Use bincode for performance
```

**Complete Protocol Buffer Definitions**: See [transport-layer-specifications.md](./transport-layer-specifications.md) Section 17.2

### 17.3 Serialization Performance Configuration

```yaml
SERIALIZATION_PERFORMANCE:
  json:
    parser: "serde_json"
    writer_buffer_size: 8192
    reader_buffer_size: 8192
    pretty_print: false
    escape_unicode: false
    floating_point_precision: 15
    use_compact_representation: true

  protobuf:
    codec: "prost"
    max_message_size: 67108864  # 64MB
    use_varint_encoding: true
    preserve_unknown_fields: false
    deterministic_serialization: true
    lazy_parsing: true

  messagepack:
    codec: "rmp-serde"
    use_compact_integers: true
    use_string_keys: true
    preserve_order: false

  compression:
    threshold_bytes: 1024
    algorithms:
      gzip:
        level: 6
        window_bits: 15
        memory_level: 8
      lz4:
        acceleration: 1
        compression_level: 0
      snappy:
        enable_checksum: true

VALIDATION_PERFORMANCE:
  schema_cache_size: 1000
  schema_cache_ttl: 3600      # seconds
  parallel_validation: true
  max_validation_threads: 4
  validation_timeout: 5000    # milliseconds
  enable_fast_path: true
  skip_optional_validation: false
```

## Summary

This document provides core transport foundations for agent communication, focusing on:

- Basic messaging patterns and abstractions
- Advanced connection management and pooling
- Comprehensive error handling and response standards
- Security foundations and authentication patterns
- Implementation guidelines and configuration templates
- Serialization specifications and performance optimization

Agents should implement these patterns incrementally, starting with basic abstractions and expanding to advanced connection management and error handling as needed.

## Navigation

### Document Relationship

**This Document** (`transport-core.md`):

- Core transport abstractions and patterns
- Connection management foundations
- Error handling patterns
- Security fundamentals
- Implementation guidelines

**Companion Document** ([`transport-layer-specifications.md`](./transport-layer-specifications.md)):

- Complete protocol implementations (NATS, gRPC, HTTP)
- Detailed service definitions and schemas
- Configuration specifications
- Performance benchmarks
- Production deployment patterns

### Protocol-Specific Transport Files

- **[NATS Transport](./nats-transport.md)** - NATS-specific implementation details
- **[gRPC Transport](./grpc-transport.md)** - gRPC service and streaming implementations  
- **[HTTP Transport](./http-transport.md)** - HTTP/WebSocket API implementations
- **[Transport Specifications](./transport-layer-specifications.md)** - Complete protocol specifications

### Framework Integration Points

**Core Architecture**:

- **[Async Patterns](../core-architecture/async-patterns.md)** - Tokio runtime integration
- **[Component Architecture](../core-architecture/component-architecture.md)** - Transport layer positioning
- **[Integration Patterns](../core-architecture/integration-patterns.md)** - Cross-system communication
- **[System Architecture](../core-architecture/system-architecture.md)** - Overall system design

**Data Management**:

- **[Message Schemas](../data-management/message-schemas.md)** - Transport message formats
- **[Agent Communication](../data-management/agent-communication.md)** - Agent messaging patterns
- **[Connection Management](../data-management/connection-management.md)** - Data layer coordination
- **[JetStream KV](../data-management/jetstream-kv.md)** - NATS persistence patterns

**Security Integration**:

- **[Authentication Specifications](../security/authentication-specifications.md)** - Transport authentication
- **[Security Patterns](../security/security-patterns.md)** - Transport security implementation
- **[Security Framework](../security/security-framework.md)** - Overall security architecture

**Operations**:

- **[Configuration Management](../operations/configuration-management.md)** - Transport configuration
- **[Observability Framework](../operations/observability-monitoring-framework.md)** - Transport monitoring
- **[Deployment Architecture](../operations/deployment-architecture-specifications.md)** - Production deployment

### Related Documentation

- **[Dependency Specifications](../core-architecture/dependency-specifications.md)** - Technology stack details
- **[Testing Framework](../testing/testing-framework.md)** - Transport testing patterns
- **[Agent Domains Analysis](../agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md)** - Agent communication requirements

### Implementation Guidance

**Start Here** (transport-core.md):

1. Understand basic messaging patterns (Sections 1-4)
2. Choose appropriate transport abstraction (Section 5)
3. Implement error handling (Section 8)
4. Add security layer (Section 9)

**Then Reference** (transport-layer-specifications.md):

1. Protocol-specific configurations
2. Service definitions and schemas
3. Performance optimization
4. Production deployment

---

*Transport Core - Foundation patterns and abstractions for MisterSmith agent communication*
