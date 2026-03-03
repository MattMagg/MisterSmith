---
title: connection-management
type: data-management
permalink: ms-framework-docs/data-management/connection-management
tags:
- '#data-management #connection-pooling #transaction-management #distributed-coordination'
---

# Connection Pool & Transaction Management

## Technical Specification

Connection pooling strategies and transaction management patterns for the Mister Smith AI Agent Framework. Covers multi-pool architectures, distributed transaction coordination using SAGA patterns, connection health monitoring, and failover strategies with focus on high availability, optimal resource utilization, and consistency across PostgreSQL and JetStream KV stores.

## Cross-References

- **Core Data Trilogy**: [[database-schemas]] ⟷ **connection-management** ⟷ [[persistence-operations]]
- **Related**: [[stream-processing]] | [[data-management/CLAUDE]]
- **Integration**: [[../core-architecture/integration-implementation]]

## 5. Advanced Connection Pool & Transaction Management

### 5.1 Enterprise Connection Pool Architecture

```rust
INTERFACE ConnectionPoolCoordinator {
    create_postgres_pool(config: PostgresPoolConfig) -> PostgresPool
    create_jetstream_pool(config: JetStreamPoolConfig) -> JetStreamPool
    coordinate_transactions(operations: List<CrossSystemOperation>) -> Result
    monitor_pool_health() -> HealthStatus
    scale_pools(metrics: LoadMetrics) -> ScalingResult
}

CLASS EnterpriseConnectionManager {
    PRIVATE postgres_pools: Map<String, PostgresPool>  -- Multiple pools for different purposes
    PRIVATE jetstream_kv_pools: Map<String, JetStreamKVPool>
    PRIVATE transaction_coordinator: DistributedTransactionCoordinator
    PRIVATE connection_monitor: ConnectionHealthMonitor
    PRIVATE pool_metrics: PoolMetricsCollector
    
    STRUCT PostgresPoolConfig {
        -- Connection pool sizing (based on SQLx and Deadpool patterns)
        max_connections: Integer = 10
        min_connections: Integer = 2
        acquire_timeout: Duration = Duration.seconds(30)
        idle_timeout: Duration = Duration.minutes(10)
        max_lifetime: Duration = Duration.hours(2)
        
        -- SQLx-specific configurations (verified against SQLx 0.8)
        statement_cache_capacity: Integer = 100
        test_before_acquire: Boolean = true
        
        -- Session-level configurations  
        application_name: String = "agent_system"
        statement_timeout: Duration = Duration.seconds(30)
        idle_in_transaction_timeout: Duration = Duration.seconds(60)
        
        -- Error handling configuration
        max_connection_retries: Integer = 3
        retry_backoff_base: Duration = Duration.millis(100)
        connection_error_threshold: Float = 0.05
        
        -- Performance optimizations
        after_connect_hooks: List<SessionConfigHook>
        connection_recycling_method: RecyclingMethod = FAST
        
        -- Monitoring and alerting
        slow_query_threshold: Duration = Duration.millis(100)
        connection_leak_detection: Boolean = true
    }
    
    STRUCT JetStreamKVPoolConfig {
        max_connections: Integer = 5
        connection_timeout: Duration = Duration.seconds(10)
        kv_bucket_ttl: Duration = Duration.minutes(30)
        replicas: Integer = 3
        storage_type: StorageType = FILE_STORAGE
        max_bucket_size: Bytes = 1_GB
        history_depth: Integer = 1  -- Keep only latest values
    }
}
```

### 5.2 Connection Pool Sizing Strategies

```rust
CLASS ConnectionPoolSizer {
    FUNCTION calculate_optimal_pool_size(
        agent_count: Integer,
        avg_operations_per_second: Float,
        avg_operation_duration: Duration,
        target_utilization: Float = 0.8
    ) -> PoolSizeRecommendation {
        
        -- Base calculation using Little's Law (mathematically verified)
        -- Pool Size = (Operations/sec) * (Average Duration) / Utilization
        base_size = (avg_operations_per_second * avg_operation_duration.seconds()) / target_utilization
        
        -- Adjust for agent concurrency patterns
        agent_factor = calculate_agent_concurrency_factor(agent_count)
        adjusted_size = base_size * agent_factor
        
        -- Apply improved bounds for production workloads
        min_safe_size = max(5, agent_count / 2)  -- Higher minimum for stability
        max_reasonable_size = min(100, agent_count * 3)  -- Higher ceiling for large deployments
        
        recommended_size = clamp(adjusted_size, min_safe_size, max_reasonable_size)
        
        RETURN PoolSizeRecommendation {
            recommended_size: Math.ceil(recommended_size),
            min_connections: Math.ceil(recommended_size * 0.3),  -- Higher baseline
            max_connections: Math.ceil(recommended_size),
            reasoning: "Based on " + agent_count + " agents, " + avg_operations_per_second + " ops/sec"
        }
    }
    
    FUNCTION calculate_agent_concurrency_factor(agent_count: Integer) -> Float {
        -- Account for different agent types and their connection patterns
        IF agent_count <= 5 THEN
            RETURN 1.0  -- Small deployments: 1:1 ratio
        ELSE IF agent_count <= 20 THEN
            RETURN 0.8  -- Medium deployments: some connection sharing
        ELSE
            RETURN 0.6  -- Large deployments: significant connection sharing
        END IF
    }
    
    -- Environment-specific sizing templates
    FUNCTION get_environment_template(env: EnvironmentType) -> PoolSizeTemplate {
        SWITCH env {
            CASE DEVELOPMENT:
                RETURN PoolSizeTemplate {
                    postgres_max: 5,
                    postgres_min: 1,
                    jetstream_max: 2,
                    acquire_timeout: Duration.seconds(10)
                }
            CASE STAGING:
                RETURN PoolSizeTemplate {
                    postgres_max: 10,
                    postgres_min: 2,
                    jetstream_max: 5,
                    acquire_timeout: Duration.seconds(20)
                }
            CASE PRODUCTION:
                RETURN PoolSizeTemplate {
                    postgres_max: 20,
                    postgres_min: 5,
                    jetstream_max: 10,
                    acquire_timeout: Duration.seconds(30)
                }
        }
    }
}
```

### 5.3 Advanced Transaction Isolation and Boundaries

```rust
CLASS AdvancedTransactionManager {
    ENUM TransactionIsolationLevel {
        READ_UNCOMMITTED,   -- Lowest isolation, fastest performance
        READ_COMMITTED,     -- Default for most operations
        REPEATABLE_READ,    -- For state flush operations and consistency requirements
        SERIALIZABLE        -- For critical updates requiring full isolation
    }
    
    ENUM TransactionBoundary {
        SINGLE_OPERATION,       -- Individual DB operation
        AGENT_TASK,            -- Complete agent task execution
        CROSS_SYSTEM,          -- Spans PostgreSQL + JetStream KV
        DISTRIBUTED_SAGA       -- Multi-agent coordination
    }
    
    CLASS TransactionContext {
        boundary: TransactionBoundary
        isolation_level: TransactionIsolationLevel
        timeout: Duration
        retry_policy: RetryPolicy
        compensation_actions: List<CompensationAction>
        correlation_id: String
        agent_id: String
    }
    
    FUNCTION execute_with_isolation(
        context: TransactionContext,
        operations: List<DatabaseOperation>
    ) -> TransactionResult {
        
        -- Select appropriate isolation level based on operation type
        isolation = determine_isolation_level(context, operations)
        
        connection = acquire_connection_for_transaction(context)
        transaction = connection.begin_transaction(isolation)
        
        -- Configure transaction settings
        configure_transaction_settings(transaction, context)
        
        TRY {
            -- Execute operations within transaction boundary
            FOR operation IN operations {
                result = operation.execute(transaction)
                
                -- Check for conflicts and adjust strategy
                IF result.has_conflict() THEN
                    conflict_resolution = handle_transaction_conflict(
                        result.conflict_type, 
                        context
                    )
                    IF conflict_resolution == ABORT_AND_RETRY THEN
                        transaction.rollback()
                        RETURN retry_with_backoff(context, operations)
                    END IF
                END IF
            }
            
            -- Pre-commit validation
            validation_result = validate_transaction_constraints(transaction, context)
            IF NOT validation_result.is_valid THEN
                transaction.rollback()
                RETURN TransactionResult.VALIDATION_FAILED(validation_result.errors)
            END IF
            
            transaction.commit()
            RETURN TransactionResult.SUCCESS
            
        } CATCH (error) {
            transaction.rollback()
            
            -- Classify error and determine retry strategy
            error_classification = classify_transaction_error(error)
            
            SWITCH error_classification {
                CASE SERIALIZATION_FAILURE:
                    RETURN retry_with_exponential_backoff(context, operations)
                CASE DEADLOCK_DETECTED:
                    RETURN retry_with_jitter(context, operations)
                CASE CONSTRAINT_VIOLATION:
                    RETURN TransactionResult.PERMANENT_FAILURE(error)
                CASE CONNECTION_FAILURE:
                    RETURN retry_with_new_connection(context, operations)
                DEFAULT:
                    RETURN TransactionResult.UNKNOWN_FAILURE(error)
            }
        }
    }
    
    FUNCTION determine_isolation_level(
        context: TransactionContext, 
        operations: List<DatabaseOperation>
    ) -> TransactionIsolationLevel {
        
        -- Override isolation level if explicitly specified
        IF context.isolation_level != NULL THEN
            RETURN context.isolation_level
        END IF
        
        -- Determine based on operation characteristics
        has_writes = operations.any(op -> op.is_write())
        has_reads = operations.any(op -> op.is_read())
        affects_shared_state = operations.any(op -> op.affects_shared_state())
        requires_consistency = context.boundary == CROSS_SYSTEM
        
        IF requires_consistency AND affects_shared_state THEN
            RETURN SERIALIZABLE  -- Strongest consistency for cross-system operations
        ELSE IF has_writes AND affects_shared_state THEN
            RETURN REPEATABLE_READ  -- Prevent phantom reads during state updates
        ELSE IF has_writes THEN
            RETURN READ_COMMITTED  -- Standard isolation for most write operations
        ELSE
            RETURN READ_COMMITTED  -- Default for read operations
        END IF
    }
    
    FUNCTION configure_transaction_settings(
        transaction: Transaction, 
        context: TransactionContext
    ) {
        -- SECURITY FIX: Use parameterized timeout setting to prevent SQL injection
        timeout_seconds = validate_timeout_range(context.timeout.seconds(), 1, 3600)
        transaction.execute("SET LOCAL statement_timeout = $1", timeout_seconds + "s")
        
        -- Configure based on boundary type with error handling
        TRY {
            SWITCH context.boundary {
                CASE AGENT_TASK:
                    transaction.execute("SET LOCAL idle_in_transaction_session_timeout = '60s'")
                    -- Validate agent_id to prevent injection
                    validated_agent_id = validate_agent_id(context.agent_id)
                    transaction.execute("SET LOCAL application_name = $1", "agent_" + validated_agent_id)
                    
                CASE CROSS_SYSTEM:
                    transaction.execute("SET LOCAL idle_in_transaction_session_timeout = '30s'")
                    transaction.execute("SET LOCAL synchronous_commit = on")  -- Ensure durability
                    
                CASE DISTRIBUTED_SAGA:
                    transaction.execute("SET LOCAL idle_in_transaction_session_timeout = '120s'")
                    transaction.execute("SET LOCAL synchronous_commit = on")
                    transaction.execute("SET LOCAL log_statement = 'all'")
            }
        } CATCH (config_error) {
            log_error("Transaction configuration failed", config_error)
            THROW TransactionConfigurationError(config_error)
        }
    }
    
    FUNCTION validate_timeout_range(value: Integer, min: Integer, max: Integer) -> Integer {
        RETURN clamp(value, min, max)
    }
    
    FUNCTION validate_agent_id(agent_id: String) -> String {
        -- Only allow alphanumeric and hyphens for security
        IF NOT agent_id.matches("^[a-zA-Z0-9-]+$") THEN
            THROW InvalidAgentIdError("Agent ID contains invalid characters")
        END IF
        RETURN agent_id
    }
}
```

### 5.4 Distributed Transaction Coordination

```rust
CLASS DistributedTransactionCoordinator {
    PRIVATE postgres_pool: PostgresPool
    PRIVATE jetstream_kv: JetStreamKV
    PRIVATE saga_manager: SagaManager
    PRIVATE compensation_executor: CompensationExecutor
    
    FUNCTION execute_cross_system_transaction(
        postgres_operations: List<PostgresOperation>,
        jetstream_operations: List<JetStreamOperation>,
        coordination_strategy: CoordinationStrategy = SAGA_PATTERN
    ) -> DistributedTransactionResult {
        
        correlation_id = generate_correlation_id()
        
        SWITCH coordination_strategy {
            CASE TWO_PHASE_COMMIT:
                RETURN execute_two_phase_commit(postgres_operations, jetstream_operations, correlation_id)
            CASE SAGA_PATTERN:
                RETURN execute_saga_pattern(postgres_operations, jetstream_operations, correlation_id)
            CASE EVENTUAL_CONSISTENCY:
                RETURN execute_eventual_consistency(postgres_operations, jetstream_operations, correlation_id)
        }
    }
    
    FUNCTION execute_saga_pattern(
        postgres_ops: List<PostgresOperation>,
        jetstream_ops: List<JetStreamOperation>,
        correlation_id: String
    ) -> DistributedTransactionResult {
        
        saga_definition = SagaDefinition {
            correlation_id: correlation_id,
            steps: build_saga_steps(postgres_ops, jetstream_ops),
            compensation_timeout: Duration.minutes(5),
            max_retry_attempts: 3
        }
        
        saga_execution = saga_manager.start_saga(saga_definition)
        
        TRY {
            -- Step 1: Execute JetStream KV operations (fast, reversible)
            FOR jetstream_op IN jetstream_ops {
                result = execute_jetstream_operation(jetstream_op, correlation_id)
                IF result.failed() THEN
                    -- JetStream failures are typically retryable
                    retry_result = retry_jetstream_operation(jetstream_op, correlation_id)
                    IF retry_result.failed() THEN
                        RETURN initiate_saga_rollback(saga_execution, "JetStream operation failed")
                    END IF
                END IF
                
                saga_execution.mark_step_completed("jetstream_" + jetstream_op.id)
            }
            
            -- Step 2: Execute PostgreSQL operations (durable, requires careful handling)
            postgres_transaction = postgres_pool.begin_transaction(REPEATABLE_READ)
            
            TRY {
                FOR postgres_op IN postgres_ops {
                    result = postgres_op.execute(postgres_transaction)
                    saga_execution.mark_step_completed("postgres_" + postgres_op.id)
                }
                
                -- Final consistency check before commit
                consistency_check = verify_cross_system_consistency(correlation_id)
                IF NOT consistency_check.is_consistent THEN
                    postgres_transaction.rollback()
                    RETURN initiate_saga_rollback(saga_execution, "Consistency check failed")
                END IF
                
                postgres_transaction.commit()
                saga_execution.mark_completed()
                
                RETURN DistributedTransactionResult.SUCCESS(correlation_id)
                
            } CATCH (postgres_error) {
                postgres_transaction.rollback()
                RETURN initiate_saga_rollback(saga_execution, "PostgreSQL error: " + postgres_error.message)
            }
            
        } CATCH (saga_error) {
            RETURN DistributedTransactionResult.SAGA_FAILED(saga_error)
        }
    }
    
    FUNCTION initiate_saga_rollback(
        saga_execution: SagaExecution, 
        failure_reason: String
    ) -> DistributedTransactionResult {
        
        compensation_plan = build_compensation_plan(saga_execution)
        
        compensation_result = compensation_executor.execute_compensation(compensation_plan)
        
        IF compensation_result.successful() THEN
            RETURN DistributedTransactionResult.ROLLED_BACK(failure_reason)
        ELSE
            -- Compensation failed - requires manual intervention
            RETURN DistributedTransactionResult.COMPENSATION_FAILED(
                failure_reason, 
                compensation_result.errors
            )
        END IF
    }
    
    FUNCTION build_compensation_plan(saga_execution: SagaExecution) -> CompensationPlan {
        completed_steps = saga_execution.get_completed_steps()
        compensation_actions = List<CompensationAction>()
        
        -- Build compensation in reverse order
        FOR step IN completed_steps.reverse() {
            SWITCH step.type {
                CASE "jetstream_write":
                    -- JetStream KV compensation: delete or revert value
                    compensation_actions.add(JetStreamDeleteAction(step.key))
                    
                CASE "postgres_insert":
                    -- PostgreSQL compensation: delete inserted record
                    compensation_actions.add(PostgresDeleteAction(step.table, step.record_id))
                    
                CASE "postgres_update":
                    -- PostgreSQL compensation: restore previous value
                    compensation_actions.add(PostgresRestoreAction(step.table, step.record_id, step.previous_value))
            }
        }
        
        RETURN CompensationPlan {
            correlation_id: saga_execution.correlation_id,
            actions: compensation_actions,
            timeout: Duration.minutes(2),
            retry_attempts: 3
        }
    }
}
```

### 5.5 Connection Pool Health Monitoring

```rust
CLASS ConnectionPoolHealthMonitor {
    PRIVATE postgres_pools: Map<String, PostgresPool>
    PRIVATE jetstream_pools: Map<String, JetStreamKVPool>
    PRIVATE health_metrics: HealthMetricsCollector
    PRIVATE alert_manager: AlertManager
    
    STRUCT PoolHealthMetrics {
        pool_name: String
        pool_type: PoolType
        total_connections: Integer
        active_connections: Integer
        idle_connections: Integer
        pending_acquisitions: Integer
        acquisition_time_p95: Duration
        connection_errors: Counter
        health_check_success_rate: Float
        last_health_check: Timestamp
    }
    
    FUNCTION monitor_all_pools() {
        postgres_metrics = collect_postgres_metrics()
        jetstream_metrics = collect_jetstream_metrics()
        
        -- Analyze metrics and trigger alerts
        analyze_pool_health(postgres_metrics)
        analyze_pool_health(jetstream_metrics)
        
        -- Update health status
        update_overall_health_status(postgres_metrics, jetstream_metrics)
    }
    
    FUNCTION collect_postgres_metrics() -> List<PoolHealthMetrics> {
        metrics = List<PoolHealthMetrics>()
        
        FOR pool_name, pool IN postgres_pools {
            pool_metrics = PoolHealthMetrics {
                pool_name: pool_name,
                pool_type: POSTGRES,
                total_connections: pool.size(),
                active_connections: pool.active_count(),
                idle_connections: pool.idle_count(),
                pending_acquisitions: pool.pending_count(),
                acquisition_time_p95: pool.acquisition_time_percentile(95),
                connection_errors: pool.error_count(),
                health_check_success_rate: calculate_health_success_rate(pool),
                last_health_check: now()
            }
            
            metrics.add(pool_metrics)
        }
        
        RETURN metrics
    }
    
    FUNCTION analyze_pool_health(metrics: List<PoolHealthMetrics>) {
        FOR metric IN metrics {
            -- Check pool utilization
            utilization = metric.active_connections / metric.total_connections
            IF utilization > 0.9 THEN
                alert_manager.trigger_alert(AlertType.HIGH_POOL_UTILIZATION, {
                    pool_name: metric.pool_name,
                    utilization: utilization,
                    severity: HIGH
                })
            END IF
            
            -- Check acquisition time
            IF metric.acquisition_time_p95 > Duration.seconds(5) THEN
                alert_manager.trigger_alert(AlertType.SLOW_CONNECTION_ACQUISITION, {
                    pool_name: metric.pool_name,
                    p95_time: metric.acquisition_time_p95,
                    severity: MEDIUM
                })
            END IF
            
            -- Check connection errors
            error_rate = metric.connection_errors / (metric.active_connections + 1)
            IF error_rate > 0.05 THEN
                alert_manager.trigger_alert(AlertType.HIGH_CONNECTION_ERROR_RATE, {
                    pool_name: metric.pool_name,
                    error_rate: error_rate,
                    severity: HIGH
                })
            END IF
            
            -- Check health success rate
            IF metric.health_check_success_rate < 0.95 THEN
                alert_manager.trigger_alert(AlertType.HEALTH_CHECK_FAILURES, {
                    pool_name: metric.pool_name,
                    success_rate: metric.health_check_success_rate,
                    severity: CRITICAL
                })
            END IF
        }
    }
    
    FUNCTION perform_health_checks() {
        -- PostgreSQL health checks
        FOR pool_name, pool IN postgres_pools {
            health_result = check_postgres_pool_health(pool)
            health_metrics.record_health_check(pool_name, POSTGRES, health_result)
        }
        
        -- JetStream KV health checks
        FOR pool_name, pool IN jetstream_pools {
            health_result = check_jetstream_pool_health(pool)
            health_metrics.record_health_check(pool_name, JETSTREAM_KV, health_result)
        }
    }
    
    FUNCTION check_postgres_pool_health(pool: PostgresPool) -> HealthCheckResult {
        TRY {
            connection = pool.acquire_timeout(Duration.seconds(5))
            
            start_time = now()
            result = connection.execute("SELECT 1 as health_check")
            latency = now() - start_time
            
            pool.release(connection)
            
            IF latency > Duration.millis(100) THEN
                RETURN HealthCheckResult.DEGRADED(latency)
            ELSE
                RETURN HealthCheckResult.HEALTHY(latency)
            END IF
            
        } CATCH (timeout_error) {
            RETURN HealthCheckResult.UNHEALTHY("Connection acquisition timeout")
        } CATCH (query_error) {
            RETURN HealthCheckResult.UNHEALTHY("Query execution failed: " + query_error.message)
        }
    }
    
    FUNCTION check_jetstream_pool_health(pool: JetStreamKVPool) -> HealthCheckResult {
        TRY {
            kv_connection = pool.acquire()
            
            start_time = now()
            -- Perform a lightweight operation
            kv_info = kv_connection.get_bucket_info()
            latency = now() - start_time
            
            pool.release(kv_connection)
            
            IF latency > Duration.millis(50) THEN
                RETURN HealthCheckResult.DEGRADED(latency)
            ELSE
                RETURN HealthCheckResult.HEALTHY(latency)
            END IF
            
        } CATCH (error) {
            RETURN HealthCheckResult.UNHEALTHY("JetStream KV error: " + error.message)
        }
    }
}
```

### 5.6 Connection String Templates and Configuration Management

```rust
CLASS DataLayerConfigurationManager {
    FUNCTION build_postgres_connection_string(env: Environment) -> String {
        config = load_postgres_config(env)
        
        -- Support various connection formats based on environment
        SWITCH env.deployment_type {
            CASE LOCAL_DEVELOPMENT:
                RETURN build_local_postgres_url(config)
            CASE DOCKER_COMPOSE:
                RETURN build_docker_postgres_url(config)
            CASE KUBERNETES:
                RETURN build_k8s_postgres_url(config)
            CASE CLOUD_MANAGED:
                RETURN build_cloud_postgres_url(config)
        }
    }
    
    FUNCTION build_local_postgres_url(config: PostgresConfig) -> String {
        -- Local development with Unix sockets or localhost
        IF config.use_unix_socket THEN
            socket_path = url_encode(config.socket_path)
            RETURN "postgres://" + socket_path + "/" + config.database + 
                   "?application_name=" + config.application_name
        ELSE
            RETURN "postgres://" + config.username + ":" + config.password + 
                   "@localhost:" + config.port + "/" + config.database +
                   "?application_name=" + config.application_name + 
                   "&sslmode=disable"
        END IF
    }
    
    FUNCTION build_cloud_postgres_url(config: PostgresConfig) -> String {
        -- Cloud deployment with SSL and connection pooling
        RETURN "postgres://" + config.username + ":" + config.password + 
               "@" + config.host + ":" + config.port + "/" + config.database +
               "?application_name=" + config.application_name +
               "&sslmode=require" +
               "&sslrootcert=" + config.ssl_root_cert +
               "&connect_timeout=" + config.connect_timeout.seconds() +
               "&statement_timeout=" + config.statement_timeout.seconds()
    }
    
    FUNCTION load_postgres_config(env: Environment) -> PostgresConfig {
        RETURN PostgresConfig {
            host: env.get("PG_HOST", "localhost"),
            port: env.get_int("PG_PORT", 5432),
            database: env.get("PG_DATABASE", "agent_system"),
            username: env.get("PG_USER", "postgres"),
            password: env.get("PG_PASSWORD", ""),
            application_name: env.get("PG_APP_NAME", "agent_data_layer"),
            socket_path: env.get("PG_SOCKET_PATH", "/var/run/postgresql"),
            use_unix_socket: env.get_bool("PG_USE_SOCKET", false),
            ssl_root_cert: env.get("PG_SSL_ROOT_CERT", ""),
            connect_timeout: Duration.seconds(env.get_int("PG_CONNECT_TIMEOUT", 10)),
            statement_timeout: Duration.seconds(env.get_int("PG_STATEMENT_TIMEOUT", 30)),
            max_connections: env.get_int("PG_MAX_CONNECTIONS", 10),
            min_connections: env.get_int("PG_MIN_CONNECTIONS", 2)
        }
    }
    
    FUNCTION build_jetstream_kv_config(env: Environment) -> JetStreamKVConfig {
        RETURN JetStreamKVConfig {
            servers: env.get_list("NATS_SERVERS", ["nats://localhost:4222"]),
            credentials_file: env.get("NATS_CREDS_FILE", ""),
            tls_cert: env.get("NATS_TLS_CERT", ""),
            tls_key: env.get("NATS_TLS_KEY", ""),
            ca_cert: env.get("NATS_CA_CERT", ""),
            bucket_prefix: env.get("NATS_KV_PREFIX", "agent_"),
            default_ttl: Duration.minutes(env.get_int("NATS_KV_TTL_MINUTES", 30)),
            replicas: env.get_int("NATS_KV_REPLICAS", 3),
            max_bucket_size: parse_bytes(env.get("NATS_KV_MAX_SIZE", "1GB")),
            compression: env.get_bool("NATS_KV_COMPRESSION", true)
        }
    }
}
```

## 12. Connection Pool Configuration & Management

### 12.1 Multiple Pool Configurations

```rust
// Example Rust configuration using SQLx
use sqlx::postgres::{PgPoolOptions, PgConnectOptions};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub primary_pool: PgPoolConfig,
    pub replica_pool: PgPoolConfig,
    pub background_pool: PgPoolConfig,
    pub analytics_pool: PgPoolConfig,
}

#[derive(Debug, Clone)]
pub struct PgPoolConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Option<Duration>,
    pub max_lifetime: Option<Duration>,
    pub test_before_acquire: bool,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
}

impl DatabaseConfig {
    pub fn production() -> Self {
        Self {
            // Primary pool for writes and consistent reads
            primary_pool: PgPoolConfig {
                max_connections: 20,
                min_connections: 5,
                acquire_timeout: Duration::from_secs(30),
                idle_timeout: Some(Duration::from_secs(600)), // 10 minutes
                max_lifetime: Some(Duration::from_secs(3600)), // 1 hour
                test_before_acquire: true,
                host: "primary-db.internal".to_string(),
                port: 5432,
                database: "mister_smith".to_string(),
                username: "app_primary".to_string(),
                password: env::var("PRIMARY_DB_PASSWORD").unwrap(),
            },
            
            // Replica pool for read-only queries
            replica_pool: PgPoolConfig {
                max_connections: 15,
                min_connections: 3,
                acquire_timeout: Duration::from_secs(20),
                idle_timeout: Some(Duration::from_secs(300)), // 5 minutes
                max_lifetime: Some(Duration::from_secs(1800)), // 30 minutes
                test_before_acquire: true,
                host: "replica-db.internal".to_string(),
                port: 5432,
                database: "mister_smith".to_string(),
                username: "app_replica".to_string(),
                password: env::var("REPLICA_DB_PASSWORD").unwrap(),
            },
            
            // Background pool for maintenance operations
            background_pool: PgPoolConfig {
                max_connections: 5,
                min_connections: 1,
                acquire_timeout: Duration::from_secs(60),
                idle_timeout: Some(Duration::from_secs(1800)), // 30 minutes
                max_lifetime: Some(Duration::from_secs(7200)), // 2 hours
                test_before_acquire: false,
                host: "primary-db.internal".to_string(),
                port: 5432,
                database: "mister_smith".to_string(),
                username: "app_background".to_string(),
                password: env::var("BACKGROUND_DB_PASSWORD").unwrap(),
            },
            
            // Analytics pool for reporting queries
            analytics_pool: PgPoolConfig {
                max_connections: 10,
                min_connections: 2,
                acquire_timeout: Duration::from_secs(45),
                idle_timeout: Some(Duration::from_secs(900)), // 15 minutes
                max_lifetime: Some(Duration::from_secs(3600)), // 1 hour
                test_before_acquire: true,
                host: "analytics-db.internal".to_string(),
                port: 5432,
                database: "mister_smith_analytics".to_string(),
                username: "app_analytics".to_string(),
                password: env::var("ANALYTICS_DB_PASSWORD")
                    .map_err(|_| DatabaseConfigError::MissingEnvironmentVariable("ANALYTICS_DB_PASSWORD"))?,
                max_connection_retries: 3,
                retry_backoff_ms: 150,
            },
        })
    }
    
    /// Create connection pool with comprehensive error handling
    pub async fn create_pool(&self, config: &PgPoolConfig) -> Result<sqlx::PgPool, DatabaseError> {
        let connect_options = PgConnectOptions::new()
            .host(&config.host)
            .port(config.port)
            .database(&config.database)
            .username(&config.username)
            .password(&config.password)
            .application_name("mister_smith_agent")
            .statement_timeout(Duration::from_secs(30))
            .log_statements(log::LevelFilter::Debug);
            
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(config.acquire_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .test_before_acquire(config.test_before_acquire)
            .connect_with(connect_options)
            .await
            .map_err(DatabaseError::ConnectionFailed)?;
            
        // Verify pool health
        self.verify_pool_health(&pool).await?;
        
        Ok(pool)
    }
    
    async fn verify_pool_health(&self, pool: &sqlx::PgPool) -> Result<(), DatabaseError> {
        let health_check = timeout(
            Duration::from_secs(5),
            sqlx::query("SELECT 1")
                .execute(pool)
        ).await
        .map_err(|_| DatabaseError::HealthCheckTimeout)?
        .map_err(DatabaseError::HealthCheckFailed)?;
        
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnvironmentVariable(&'static str),
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(#[from] sqlx::Error),
    #[error("Health check timeout")]
    HealthCheckTimeout,
    #[error("Health check failed: {0}")]
    HealthCheckFailed(sqlx::Error),
    #[error("Pool exhausted")]
    PoolExhausted,
    #[error("Transaction timeout")]
    TransactionTimeout,
}
```

### 12.2 Connection Pool Health Monitoring

```sql
-- ============================================================================
-- CONNECTION POOL MONITORING AND HEALTH CHECKS
-- See [[persistence-operations]] for comprehensive monitoring framework
-- ============================================================================

-- Core monitoring view for connection statistics
CREATE OR REPLACE VIEW monitoring.connection_stats AS
SELECT 
  datname, usename, application_name, state,
  COUNT(*) as connection_count,
  AVG(EXTRACT(epoch FROM (NOW() - backend_start))) as avg_connection_age_seconds
FROM pg_stat_activity 
WHERE datname = current_database()
GROUP BY datname, usename, application_name, state
ORDER BY connection_count DESC;

-- Health check function (detailed implementation in [[persistence-operations]])
CREATE OR REPLACE FUNCTION monitoring.check_connection_pool_health()
RETURNS TABLE(pool_name TEXT, health_status TEXT, recommendations TEXT[]) AS $$
-- Implementation details moved to persistence-operations.md for centralized monitoring
$$ LANGUAGE plpgsql;
```

### 12.3 Failover and Load Balancing

```rust
// Thread-safe connection management with comprehensive failover support
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::time::{timeout, Duration};

#[derive(Clone)]
pub struct DatabaseManager {
    primary_pool: Arc<Pool<Postgres>>,
    replica_pools: Vec<Arc<Pool<Postgres>>>,
    background_pool: Arc<Pool<Postgres>>,
    current_replica_index: Arc<AtomicUsize>,
    health_status: Arc<std::sync::RwLock<HealthStatus>>,
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub primary_healthy: bool,
    pub replica_health: Vec<bool>,
    pub last_check: std::time::Instant,
}

impl DatabaseManager {
    /// Get primary pool for write operations with health checking
    pub async fn get_write_pool(&self) -> Result<&Pool<Postgres>, DatabaseError> {
        if !self.is_primary_healthy().await {
            return Err(DatabaseError::PrimaryUnhealthy);
        }
        Ok(&self.primary_pool)
    }
    
    /// Get read pool with intelligent failover
    pub async fn get_read_pool(&self) -> Result<&Pool<Postgres>, DatabaseError> {
        // Try replicas first with health checking
        for attempt in 0..self.replica_pools.len() {
            let index = self.current_replica_index
                .fetch_add(1, Ordering::Relaxed) % self.replica_pools.len();
            
            if self.is_replica_healthy(index).await {
                return Ok(&self.replica_pools[index]);
            }
        }
        
        // Fallback to primary if all replicas unhealthy
        if self.is_primary_healthy().await {
            Ok(&self.primary_pool)
        } else {
            Err(DatabaseError::AllPoolsUnhealthy)
        }
    }
    
    /// Comprehensive health monitoring
    pub async fn health_check(&self) -> HealthCheckResult {
        let mut result = HealthCheckResult::new();
        
        // Check primary pool
        result.primary_healthy = self.check_pool_health(&self.primary_pool).await;
        
        // Check replica pools
        for (i, pool) in self.replica_pools.iter().enumerate() {
            let healthy = self.check_pool_health(pool).await;
            result.replica_health.push(healthy);
        }
        
        // Update cached health status
        if let Ok(mut status) = self.health_status.write() {
            status.primary_healthy = result.primary_healthy;
            status.replica_health = result.replica_health.clone();
            status.last_check = std::time::Instant::now();
        }
        
        result
    }
    
    async fn check_pool_health(&self, pool: &Pool<Postgres>) -> bool {
        timeout(
            Duration::from_secs(5),
            sqlx::query("SELECT 1")
                .execute(pool)
        ).await.is_ok_and(|result| result.is_ok())
    }
    
    async fn is_primary_healthy(&self) -> bool {
        // Use cached health status if recent enough
        if let Ok(status) = self.health_status.read() {
            if status.last_check.elapsed() < Duration::from_secs(30) {
                return status.primary_healthy;
            }
        }
        
        // Perform fresh health check
        self.check_pool_health(&self.primary_pool).await
    }
    
    async fn is_replica_healthy(&self, index: usize) -> bool {
        if index >= self.replica_pools.len() {
            return false;
        }
        
        // Use cached health status if recent enough
        if let Ok(status) = self.health_status.read() {
            if status.last_check.elapsed() < Duration::from_secs(30) 
                && index < status.replica_health.len() {
                return status.replica_health[index];
            }
        }
        
        // Perform fresh health check
        self.check_pool_health(&self.replica_pools[index]).await
    }
}

#[derive(Debug)]
pub struct HealthCheckResult {
    pub primary_healthy: bool,
    pub replica_health: Vec<bool>,
    pub overall_healthy: bool,
}

impl HealthCheckResult {
    fn new() -> Self {
        Self {
            primary_healthy: false,
            replica_health: Vec::new(),
            overall_healthy: false,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Primary database unhealthy")]
    PrimaryUnhealthy,
    #[error("All database pools unhealthy")]
    AllPoolsUnhealthy,
    #[error("Connection pool exhausted")]
    PoolExhausted,
    #[error("Operation timeout")]
    OperationTimeout,
}
```

## Implementation Patterns Summary

### Core Capabilities

1. **Multi-Pool Architecture**: Specialized connection pools for primary, replica, background, and analytics workloads
2. **Dynamic Pool Sizing**: Mathematical sizing using Little's Law with agent concurrency factors
3. **Transaction Management**: Isolation level selection based on operation characteristics
4. **Distributed Coordination**: SAGA pattern for PostgreSQL + JetStream KV consistency
5. **Health Monitoring**: Real-time pool health with automatic failover
6. **Error Handling**: Comprehensive error recovery with retry mechanisms
7. **Thread Safety**: All connection management operations are thread-safe

### Technical Validation

- **SQL Standards**: All isolation levels conform to PostgreSQL standards
- **SQLx Compatibility**: Configuration patterns verified against SQLx 0.8+
- **Security**: Input validation prevents SQL injection vulnerabilities
- **Performance**: Pool sizing optimized for agent workload patterns

### Implementation Sequence

```
[database-schemas] -> [connection-management] -> [persistence-operations]
     (schema DDL)      (pools + transactions)    (monitoring + ops)
```

### Error Handling Patterns

- **Connection Failures**: Automatic retry with exponential backoff
- **Pool Exhaustion**: Graceful degradation and load shedding
- **Transaction Timeouts**: Configurable timeouts with cleanup
- **Health Check Failures**: Automatic failover to healthy replicas

## Related Documentation

- **[[database-schemas]]** - PostgreSQL schema specifications and DDL
- **[[persistence-operations]]** - Monitoring, migrations, and operational procedures
- **[[stream-processing]]** - JetStream KV integration patterns
- **[[../core-architecture/integration-implementation]]** - Testing and validation
