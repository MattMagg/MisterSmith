---
title: persistence-operations
type: data-management
permalink: ms-framework-docs/data-management/persistence-operations
tags:
- '#data-management #error-handling #monitoring #migrations #operations'
---

## Persistence Operations & Maintenance

## Error Handling, Monitoring & Migration Framework

> **Technical Specifications**: Persistence error handling and recovery patterns integrated with agent operations
>
> **Cross-References**: Integrates with [agent-integration.md](./agent-integration.md) for unified agent-to-persistence error handling
> **Navigation**: Part of the modularized data persistence framework
>
> - **Core Trilogy**: [[persistence-operations]] ⟷ [[storage-patterns]] ⟷ [[connection-management]]
> - Related: [[stream-processing]] | [[schema-definitions]] | [[data-management/CLAUDE]]
> - External: [[../core-architecture/integration-implementation]]

## Executive Summary

This document defines operational patterns for data persistence with comprehensive error handling and recovery strategies.
It covers unified error handling (integrated with [agent-integration.md](./agent-integration.md)), conflict resolution,
monitoring frameworks, and migration procedures. These patterns ensure robust persistence operations with
proper integration points for agent system failures and recovery scenarios.

## 6. Error Handling and Conflict Resolution

### 6.1 Enhanced Error Types

```rust
ENUM DataError {
    NOT_FOUND,
    DUPLICATE_KEY,
    CONNECTION_FAILED,
    SERIALIZATION_ERROR,
    VERSION_CONFLICT,    -- For optimistic locking
    TTL_EXPIRED,        -- KV entry expired
    CONSISTENCY_VIOLATION
}

CLASS DataResult<T> {
    value: T?
    error: DataError?
    metadata: ResultMetadata?  -- Version, timestamp, etc.
    
    FUNCTION isSuccess() -> Boolean
    FUNCTION getValue() -> T
    FUNCTION getError() -> DataError
    FUNCTION requiresRetry() -> Boolean {
        RETURN error IN [VERSION_CONFLICT, SERIALIZATION_ERROR]
    }
}
```

### 6.2 Conflict Resolution Strategies

```rust
CLASS ConflictResolver {
    ENUM Strategy {
        LAST_WRITE_WINS,
        VECTOR_CLOCK,
        CRDT_MERGE,
        CUSTOM_MERGE
    }
    
    FUNCTION resolve(
        current: StateEntry, 
        incoming: StateEntry, 
        strategy: Strategy
    ) -> StateEntry {
        SWITCH strategy {
            CASE LAST_WRITE_WINS:
                RETURN (incoming.timestamp > current.timestamp) ? 
                       incoming : current
                       
            CASE VECTOR_CLOCK:
                IF vectorClockDominates(incoming.clock, current.clock) THEN
                    RETURN incoming
                ELSE IF vectorClockDominates(current.clock, incoming.clock) THEN
                    RETURN current
                ELSE
                    -- Concurrent, need merge
                    RETURN mergeStates(current, incoming)
                END IF
                
            CASE CRDT_MERGE:
                RETURN StateEntry {
                    value: crdtMerge(current.value, incoming.value),
                    timestamp: max(current.timestamp, incoming.timestamp),
                    version: max(current.version, incoming.version) + 1
                }
        }
    }
}
```

### 6.3 Retry Logic with Backoff

```rust
CLASS RetryHandler {
    FUNCTION withRetry(
        operation: Function, 
        maxAttempts: Integer = 3,
        backoffBase: Duration = 100ms
    ) -> Result {
        attempts = 0
        
        WHILE attempts < maxAttempts {
            result = operation()
            
            IF result.isSuccess() THEN
                RETURN result
            END IF
            
            IF NOT result.requiresRetry() THEN
                RETURN result  -- Non-retryable error
            END IF
            
            attempts += 1
            IF attempts < maxAttempts THEN
                backoff = backoffBase * (2 ^ attempts)  -- Exponential
                sleep(min(backoff, Duration.seconds(5)))  -- Cap at 5s
            END IF
        }
        
        RETURN Failure("Max retry attempts reached")
    }
}
```

### 6.4 Advanced Error Recovery Patterns

```rust
CLASS ErrorRecoveryManager {
    PRIVATE circuit_breaker: CircuitBreaker
    PRIVATE fallback_strategy: FallbackStrategy
    PRIVATE recovery_metrics: RecoveryMetrics
    
    FUNCTION execute_with_recovery(
        operation: Operation,
        recovery_context: RecoveryContext
    ) -> OperationResult {
        
        -- Check circuit breaker state
        IF circuit_breaker.is_open() THEN
            RETURN fallback_strategy.execute(operation, recovery_context)
        END IF
        
        TRY {
            result = operation.execute()
            circuit_breaker.record_success()
            RETURN result
            
        } CATCH (error) {
            circuit_breaker.record_failure()
            recovery_metrics.record_error(error.type, error.severity)
            
            -- Classify error and determine recovery action
            recovery_action = classify_error_and_determine_recovery(error, recovery_context)
            
            SWITCH recovery_action {
                CASE RETRY_WITH_BACKOFF:
                    RETURN retry_with_exponential_backoff(operation, recovery_context)
                CASE FALLBACK_TO_CACHE:
                    RETURN fallback_strategy.use_cached_data(operation.cache_key)
                CASE GRACEFUL_DEGRADATION:
                    RETURN fallback_strategy.degrade_gracefully(operation, error)
                CASE ESCALATE_TO_OPERATOR:
                    alert_manager.escalate_critical_error(error, recovery_context)
                    RETURN OperationResult.CRITICAL_FAILURE(error)
                DEFAULT:
                    RETURN OperationResult.UNRECOVERABLE_FAILURE(error)
            }
        }
    }
    
    FUNCTION classify_error_and_determine_recovery(
        error: Error, 
        context: RecoveryContext
    ) -> RecoveryAction {
        
        -- Check retry limits first to prevent infinite loops
        IF context.retry_count >= context.max_retries THEN
            RETURN ESCALATE_TO_OPERATOR
        END IF
        
        -- Network and connectivity errors (limited retries)
        IF error.type IN [CONNECTION_TIMEOUT, NETWORK_UNREACHABLE] THEN
            IF context.retry_count < 3 THEN
                RETURN RETRY_WITH_BACKOFF
            ELSE
                RETURN FALLBACK_TO_CACHE
            END IF
        END IF
        
        -- DNS failures should not retry immediately
        IF error.type = DNS_FAILURE THEN
            IF context.time_since_last_success > Duration.minutes(5) THEN
                RETURN ESCALATE_TO_OPERATOR
            ELSE
                RETURN FALLBACK_TO_CACHE
            END IF
        END IF
        
        -- Data consistency errors (quick retry then escalate)
        IF error.type IN [VERSION_CONFLICT, SERIALIZATION_FAILURE] THEN
            IF context.retry_count < 2 THEN
                RETURN RETRY_WITH_BACKOFF
            ELSE
                RETURN ESCALATE_TO_OPERATOR
            END IF
        END IF
        
        -- Resource exhaustion (consider context)
        IF error.type IN [CONNECTION_POOL_EXHAUSTED, MEMORY_LIMIT_EXCEEDED] THEN
            IF context.system_load > 0.8 THEN
                RETURN GRACEFUL_DEGRADATION
            ELSE
                RETURN RETRY_WITH_BACKOFF
            END IF
        END IF
        
        -- Data corruption or integrity violations (never retry)
        IF error.type IN [CONSTRAINT_VIOLATION, DATA_CORRUPTION] THEN
            RETURN ESCALATE_TO_OPERATOR
        END IF
        
        -- Temporary service degradation (context-aware)
        IF error.type IN [SERVICE_UNAVAILABLE, RATE_LIMITED] THEN
            IF context.cache_available THEN
                RETURN FALLBACK_TO_CACHE
            ELSE
                RETURN GRACEFUL_DEGRADATION
            END IF
        END IF
        
        -- Integration with agent-integration.md errors
        IF error.type IN [AGENT_SPAWN_FAILED, COORDINATION_TIMEOUT] THEN
            RETURN GRACEFUL_DEGRADATION
        END IF
        
        -- Unknown or unclassified errors
        RETURN ESCALATE_TO_OPERATOR
    }
}

CLASS FallbackStrategy {
    PRIVATE cache_manager: CacheManager
    PRIVATE degradation_policies: Map<OperationType, DegradationPolicy>
    
    FUNCTION use_cached_data(cache_key: String) -> OperationResult {
        cached_result = cache_manager.get(cache_key)
        IF cached_result.exists() THEN
            RETURN OperationResult.SUCCESS_FROM_CACHE(cached_result.value)
        ELSE
            RETURN OperationResult.CACHE_MISS()
        END IF
    }
    
    FUNCTION degrade_gracefully(
        operation: Operation, 
        error: Error
    ) -> OperationResult {
        policy = degradation_policies.get(operation.type)
        
        SWITCH policy.degradation_level {
            CASE REDUCE_ACCURACY:
                -- Return approximate or estimated results
                RETURN operation.execute_with_reduced_accuracy()
            CASE SKIP_NON_ESSENTIAL:
                -- Execute only critical parts of the operation
                RETURN operation.execute_critical_path_only()
            CASE RETURN_DEFAULT:
                -- Return safe default values
                RETURN OperationResult.SUCCESS_WITH_DEFAULTS(policy.default_value)
            CASE QUEUE_FOR_LATER:
                -- Queue operation for retry when service recovers
                queue_manager.enqueue_for_retry(operation)
                RETURN OperationResult.QUEUED_FOR_RETRY()
        }
    }
}
```

## 7. Basic Monitoring

### 7.1 Consistency Metrics

```rust
CLASS ConsistencyMonitor {
    PRIVATE metrics: MetricsCollector
    
    FUNCTION trackConsistencyWindow(agent_id: String) {
        -- Measure time between KV write and SQL flush with null safety
        kv_time = getLastKVWrite(agent_id)
        sql_time = getLastSQLWrite(agent_id)
        
        -- Handle missing data gracefully
        IF kv_time IS NULL OR sql_time IS NULL THEN
            metrics.incrementCounter("consistency_data_missing", {"agent": agent_id})
            RETURN
        END IF
        
        -- Ensure operations are from the same logical sequence
        IF NOT areFromSameOperation(kv_time, sql_time) THEN
            metrics.incrementCounter("consistency_sequence_mismatch", {"agent": agent_id})
            RETURN
        END IF
        
        -- Calculate lag ensuring it's non-negative (KV should happen first)
        lag = sql_time - kv_time
        IF lag < 0 THEN
            -- Log reverse order as potential issue
            metrics.incrementCounter("consistency_reverse_order", {"agent": agent_id})
            lag = 0  -- Treat as synchronous for metrics
        END IF
        
        metrics.recordGauge("consistency_lag_ms", lag, {"agent": agent_id})
        
        -- Check for violations with agent-integration.md error types
        IF lag > Duration.millis(200) THEN
            metrics.incrementCounter("consistency_violations")
            triggerRepairJob(agent_id, lag)
            
            -- Integrate with agent spawn errors if persistence is blocking
            IF lag > Duration.seconds(5) THEN
                notifyAgentIntegrationLayer(agent_id, PERSISTENCE_LAG_CRITICAL)
            END IF
        END IF
    }
    
    FUNCTION getMemoryStats() -> MemoryStats {
        RETURN MemoryStats {
            kv_entries: countKVEntries(),
            sql_rows: countSQLRows(),
            dirty_entries: countDirtyEntries(),
            avg_flush_time: metrics.getAverage("flush_duration_ms"),
            consistency_window_p95: metrics.getPercentile("consistency_lag_ms", 95)
        }
    }
}
```

### 7.2 Health Checks

```rust
CLASS HealthChecker {
    FUNCTION checkDatabase() -> HealthStatus {
        TRY {
            start = now()
            db.execute("SELECT 1")
            latency = now() - start
            
            RETURN HealthStatus {
                status: (latency < 10ms) ? HEALTHY : DEGRADED,
                latency: latency,
                details: "Database responding"
            }
        } CATCH (error) {
            RETURN HealthStatus.UNHEALTHY
        }
    }
    
    FUNCTION checkJetStream() -> HealthStatus {
        TRY {
            jetstream.ping()
            -- Check KV bucket status
            bucket_info = jetstream.getBucketInfo("AGENT_STATE")
            
            RETURN HealthStatus {
                status: HEALTHY,
                details: {
                    entries: bucket_info.entry_count,
                    bytes: bucket_info.bytes,
                    ttl_config: bucket_info.ttl
                }
            }
        } CATCH (error) {
            RETURN HealthStatus.UNHEALTHY
        }
    }
    
    FUNCTION checkConsistency() -> HealthStatus {
        stats = ConsistencyMonitor.getMemoryStats()
        IF stats.consistency_window_p95 > 200 THEN
            RETURN HealthStatus.DEGRADED
        ELSE
            RETURN HealthStatus.HEALTHY
        END IF
    }
}
```

### 7.3 Comprehensive Monitoring Framework

```rust
CLASS PersistenceMonitoringFramework {
    PRIVATE metrics_collector: MetricsCollector
    PRIVATE alert_manager: AlertManager
    PRIVATE health_checker: HealthChecker
    PRIVATE consistency_monitor: ConsistencyMonitor
    
    STRUCT MonitoringConfig {
        check_interval: Duration = Duration.seconds(30)
        alert_thresholds: AlertThresholds
        retention_period: Duration = Duration.days(30)
        export_endpoints: List<String>
    }
    
    FUNCTION start_monitoring(config: MonitoringConfig) {
        -- Start periodic health checks
        schedule_recurring_task(config.check_interval) {
            perform_comprehensive_health_check()
        }
        
        -- Start consistency monitoring
        schedule_recurring_task(Duration.seconds(10)) {
            monitor_consistency_across_agents()
        }
        
        -- Start metrics collection
        schedule_recurring_task(Duration.seconds(5)) {
            collect_performance_metrics()
        }
    }
    
    FUNCTION perform_comprehensive_health_check() -> SystemHealthReport {
        health_report = SystemHealthReport()
        
        -- Database health
        db_health = health_checker.checkDatabase()
        health_report.add_component("postgresql", db_health)
        
        -- JetStream health
        js_health = health_checker.checkJetStream()
        health_report.add_component("jetstream_kv", js_health)
        
        -- Connection pool health
        pool_health = health_checker.checkConnectionPools()
        health_report.add_component("connection_pools", pool_health)
        
        -- Data consistency health
        consistency_health = health_checker.checkConsistency()
        health_report.add_component("data_consistency", consistency_health)
        
        -- Analyze overall system health
        overall_status = determine_overall_health(health_report)
        health_report.set_overall_status(overall_status)
        
        -- Trigger alerts if necessary
        IF overall_status IN [DEGRADED, UNHEALTHY] THEN
            alert_manager.trigger_system_health_alert(health_report)
        END IF
        
        RETURN health_report
    }
    
    FUNCTION monitor_consistency_across_agents() {
        active_agents = get_active_agent_list()
        
        FOR agent_id IN active_agents {
            consistency_monitor.trackConsistencyWindow(agent_id)
            
            -- Check for orphaned data
            orphaned_kv_entries = find_orphaned_kv_entries(agent_id)
            IF orphaned_kv_entries.size() > 0 THEN
                metrics_collector.recordGauge(
                    "orphaned_kv_entries", 
                    orphaned_kv_entries.size(), 
                    {"agent": agent_id}
                )
                schedule_cleanup_job(agent_id, orphaned_kv_entries)
            END IF
        }
    }
    
    FUNCTION collect_performance_metrics() {
        -- Connection pool metrics
        FOR pool_name, pool IN connection_pools {
            metrics_collector.recordGauge("pool_active_connections", pool.active_count(), {"pool": pool_name})
            metrics_collector.recordGauge("pool_idle_connections", pool.idle_count(), {"pool": pool_name})
            metrics_collector.recordGauge("pool_pending_acquisitions", pool.pending_count(), {"pool": pool_name})
        }
        
        -- Database performance metrics
        db_stats = get_database_performance_stats()
        metrics_collector.recordGauge("db_active_connections", db_stats.active_connections)
        metrics_collector.recordGauge("db_idle_connections", db_stats.idle_connections)
        metrics_collector.recordGauge("db_query_latency_p95", db_stats.query_latency_p95)
        
        -- JetStream KV metrics
        kv_stats = get_jetstream_kv_stats()
        metrics_collector.recordGauge("kv_bucket_size", kv_stats.total_bytes)
        metrics_collector.recordGauge("kv_entry_count", kv_stats.entry_count)
        metrics_collector.recordGauge("kv_operation_latency_p95", kv_stats.operation_latency_p95)
        
        -- Agent state metrics
        state_stats = get_agent_state_stats()
        metrics_collector.recordGauge("dirty_state_entries", state_stats.dirty_count)
        metrics_collector.recordGauge("state_flush_frequency", state_stats.flush_frequency)
    }
    
    FUNCTION generate_monitoring_dashboard() -> Dashboard {
        dashboard = Dashboard("Persistence Layer Monitoring")
        
        -- System health overview
        dashboard.add_panel(HealthOverviewPanel {
            data_sources: ["postgresql", "jetstream_kv", "connection_pools"],
            refresh_interval: Duration.seconds(30)
        })
        
        -- Performance metrics
        dashboard.add_panel(PerformanceMetricsPanel {
            metrics: ["query_latency", "operation_throughput", "error_rate"],
            time_range: Duration.hours(24)
        })
        
        -- Consistency monitoring
        dashboard.add_panel(ConsistencyTrackingPanel {
            metrics: ["consistency_lag", "orphaned_entries", "sync_failures"],
            alerting_enabled: true
        })
        
        -- Resource utilization
        dashboard.add_panel(ResourceUtilizationPanel {
            metrics: ["memory_usage", "connection_utilization", "disk_usage"],
            capacity_planning: true
        })
        
        RETURN dashboard
    }
}
```

## 10. Migration Framework & Versioning Strategy

### 10.1 Migration Infrastructure

```sql
-- ============================================================================
-- MIGRATION TRACKING AND VERSIONING SYSTEM
-- ============================================================================

CREATE SCHEMA IF NOT EXISTS migrations;

-- Migration tracking table
CREATE TABLE migrations.applied_migrations (
  migration_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  version VARCHAR(20) NOT NULL UNIQUE,
  name VARCHAR(255) NOT NULL,
  description TEXT,
  migration_type VARCHAR(50) NOT NULL DEFAULT 'schema',
  sql_hash VARCHAR(64) NOT NULL, -- SHA-256 of migration SQL
  
  -- Dependencies and rollback
  depends_on_version VARCHAR(20),
  rollback_sql TEXT,
  is_rollbackable BOOLEAN DEFAULT FALSE,
  
  -- Execution tracking
  applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  applied_by VARCHAR(100) DEFAULT CURRENT_USER,
  execution_time_ms BIGINT,
  
  -- Environment and context
  database_version VARCHAR(50),
  application_version VARCHAR(50),
  environment VARCHAR(50),
  
  CONSTRAINT valid_version CHECK (version ~ '^\d+\.\d+\.\d+(-\w+)?$'),
  CONSTRAINT valid_migration_type CHECK (migration_type IN ('schema', 'data', 'index', 'partition', 'function')),
  CONSTRAINT valid_execution_time CHECK (execution_time_ms >= 0)
);

-- Migration locks to prevent concurrent execution
CREATE TABLE migrations.locks (
  lock_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  lock_name VARCHAR(100) UNIQUE NOT NULL,
  locked_by VARCHAR(100) NOT NULL,
  locked_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
  
  CONSTRAINT future_expiry CHECK (expires_at > locked_at)
);

-- Migration validation and testing results
CREATE TABLE migrations.validation_results (
  validation_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  migration_version VARCHAR(20) NOT NULL REFERENCES migrations.applied_migrations(version),
  test_name VARCHAR(255) NOT NULL,
  test_type VARCHAR(50) NOT NULL,
  status VARCHAR(20) NOT NULL,
  error_message TEXT,
  execution_time_ms BIGINT,
  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
  
  CONSTRAINT valid_test_type CHECK (test_type IN ('unit', 'integration', 'performance', 'rollback')),
  CONSTRAINT valid_status CHECK (status IN ('passed', 'failed', 'skipped', 'error'))
);
```

### 10.2 Migration Procedures

```sql
-- ============================================================================
-- MIGRATION EXECUTION FUNCTIONS
-- ============================================================================

-- Function to safely execute migrations with locking
CREATE OR REPLACE FUNCTION migrations.execute_migration(
  p_version VARCHAR(20),
  p_name VARCHAR(255),
  p_description TEXT,
  p_migration_sql TEXT,
  p_rollback_sql TEXT DEFAULT NULL,
  p_migration_type VARCHAR(50) DEFAULT 'schema'
) RETURNS BOOLEAN AS $$
DECLARE
  v_lock_acquired BOOLEAN := FALSE;
  v_start_time TIMESTAMP WITH TIME ZONE;
  v_execution_time BIGINT;
  v_sql_hash VARCHAR(64);
BEGIN
  -- Calculate SQL hash for integrity verification
  v_sql_hash := encode(digest(p_migration_sql, 'sha256'), 'hex');
  
  -- Acquire migration lock
  BEGIN
    INSERT INTO migrations.locks (lock_name, locked_by, expires_at)
    VALUES ('migration_execution', CURRENT_USER, NOW() + INTERVAL '1 hour');
    v_lock_acquired := TRUE;
  EXCEPTION WHEN unique_violation THEN
    RAISE EXCEPTION 'Migration already in progress. Please wait for completion.';
  END;
  
  -- Check if migration already applied
  IF EXISTS (SELECT 1 FROM migrations.applied_migrations WHERE version = p_version) THEN
    DELETE FROM migrations.locks WHERE lock_name = 'migration_execution';
    RAISE EXCEPTION 'Migration version % already applied', p_version;
  END IF;
  
  -- Execute migration within transaction
  v_start_time := clock_timestamp();
  
  BEGIN
    EXECUTE p_migration_sql;
    
    -- Record successful migration
    v_execution_time := EXTRACT(epoch FROM (clock_timestamp() - v_start_time)) * 1000;
    
    INSERT INTO migrations.applied_migrations (
      version, name, description, migration_type, sql_hash,
      rollback_sql, is_rollbackable, execution_time_ms,
      database_version, application_version
    ) VALUES (
      p_version, p_name, p_description, p_migration_type, v_sql_hash,
      p_rollback_sql, (p_rollback_sql IS NOT NULL), v_execution_time,
      version(), current_setting('application.version', true)
    );
    
    -- Release lock
    DELETE FROM migrations.locks WHERE lock_name = 'migration_execution';
    
    RETURN TRUE;
    
  EXCEPTION WHEN OTHERS THEN
    -- Release lock on error
    IF v_lock_acquired THEN
      DELETE FROM migrations.locks WHERE lock_name = 'migration_execution';
    END IF;
    
    RAISE;
  END;
END;
$$ LANGUAGE plpgsql;

-- Function to rollback migrations
CREATE OR REPLACE FUNCTION migrations.rollback_migration(p_version VARCHAR(20))
RETURNS BOOLEAN AS $$
DECLARE
  v_migration RECORD;
  v_lock_acquired BOOLEAN := FALSE;
BEGIN
  -- Acquire rollback lock
  BEGIN
    INSERT INTO migrations.locks (lock_name, locked_by, expires_at)
    VALUES ('migration_rollback', CURRENT_USER, NOW() + INTERVAL '1 hour');
    v_lock_acquired := TRUE;
  EXCEPTION WHEN unique_violation THEN
    RAISE EXCEPTION 'Rollback already in progress. Please wait for completion.';
  END;
  
  -- Get migration details
  SELECT * INTO v_migration 
  FROM migrations.applied_migrations 
  WHERE version = p_version;
  
  IF NOT FOUND THEN
    DELETE FROM migrations.locks WHERE lock_name = 'migration_rollback';
    RAISE EXCEPTION 'Migration version % not found', p_version;
  END IF;
  
  IF NOT v_migration.is_rollbackable THEN
    DELETE FROM migrations.locks WHERE lock_name = 'migration_rollback';
    RAISE EXCEPTION 'Migration version % is not rollbackable', p_version;
  END IF;
  
  -- Execute rollback
  BEGIN
    EXECUTE v_migration.rollback_sql;
    
    -- Remove migration record
    DELETE FROM migrations.applied_migrations WHERE version = p_version;
    
    -- Release lock
    DELETE FROM migrations.locks WHERE lock_name = 'migration_rollback';
    
    RETURN TRUE;
    
  EXCEPTION WHEN OTHERS THEN
    -- Release lock on error
    IF v_lock_acquired THEN
      DELETE FROM migrations.locks WHERE lock_name = 'migration_rollback';
    END IF;
    
    RAISE;
  END;
END;
$$ LANGUAGE plpgsql;
```

### 10.3 Zero-Downtime Migration Patterns

```sql
-- ============================================================================
-- ZERO-DOWNTIME MIGRATION STRATEGIES
-- ============================================================================

-- Pattern 1: Online column addition with gradual deployment
CREATE OR REPLACE FUNCTION migrations.add_column_online(
  p_table_name TEXT,
  p_column_name TEXT,
  p_column_type TEXT,
  p_default_value TEXT DEFAULT NULL
) RETURNS VOID AS $$
BEGIN
  -- Step 1: Add column with default value (non-blocking)
  EXECUTE format(
    'ALTER TABLE %I ADD COLUMN %I %s %s',
    p_table_name, p_column_name, p_column_type,
    CASE WHEN p_default_value IS NOT NULL THEN 'DEFAULT ' || p_default_value ELSE '' END
  );
  
  -- Step 2: Gradually populate existing rows (if needed)
  IF p_default_value IS NOT NULL THEN
    EXECUTE format(
      'UPDATE %I SET %I = %s WHERE %I IS NULL',
      p_table_name, p_column_name, p_default_value, p_column_name
    );
  END IF;
END;
$$ LANGUAGE plpgsql;

-- Pattern 2: Table restructuring with shadow table
CREATE OR REPLACE FUNCTION migrations.restructure_table_shadow(
  p_old_table TEXT,
  p_new_table_ddl TEXT,
  p_data_migration_sql TEXT
) RETURNS VOID AS $$
DECLARE
  v_temp_table TEXT;
BEGIN
  v_temp_table := p_old_table || '_migration_' || extract(epoch from now())::bigint;
  
  -- Step 1: Create shadow table
  EXECUTE p_new_table_ddl;
  
  -- Step 2: Migrate existing data
  EXECUTE p_data_migration_sql;
  
  -- Step 3: Create triggers for ongoing synchronization
  EXECUTE format('
    CREATE OR REPLACE FUNCTION sync_%s_to_%s() RETURNS TRIGGER AS $sync$
    BEGIN
      -- Custom synchronization logic here
      RETURN COALESCE(NEW, OLD);
    END;
    $sync$ LANGUAGE plpgsql;
  ', p_old_table, v_temp_table);
  
  -- Additional steps would include trigger setup, validation, and cutover
END;
$$ LANGUAGE plpgsql;
```

### 10.4 Migration Orchestration Framework

```rust
CLASS MigrationOrchestrator {
    PRIVATE migration_repository: MigrationRepository
    PRIVATE validator: MigrationValidator
    PRIVATE executor: MigrationExecutor
    PRIVATE rollback_manager: RollbackManager
    
    FUNCTION plan_migration_sequence(
        target_version: String,
        current_version: String
    ) -> MigrationPlan {
        
        available_migrations = migration_repository.get_migrations_between(
            current_version, 
            target_version
        )
        
        -- Build dependency graph
        dependency_graph = build_migration_dependency_graph(available_migrations)
        
        -- Topological sort to determine execution order
        execution_order = topological_sort(dependency_graph)
        
        -- Validate migration sequence
        validation_result = validator.validate_migration_sequence(execution_order)
        IF NOT validation_result.is_valid THEN
            THROW MigrationValidationException(validation_result.errors)
        END IF
        
        RETURN MigrationPlan {
            migrations: execution_order,
            estimated_duration: calculate_estimated_duration(execution_order),
            rollback_strategy: determine_rollback_strategy(execution_order),
            risk_assessment: assess_migration_risks(execution_order)
        }
    }
    
    FUNCTION execute_migration_plan(plan: MigrationPlan) -> MigrationResult {
        execution_context = MigrationExecutionContext {
            start_time: now(),
            current_step: 0,
            total_steps: plan.migrations.size(),
            rollback_points: []
        }
        
        TRY {
            FOR migration IN plan.migrations {
                step_result = execute_single_migration(migration, execution_context)
                
                IF step_result.failed() THEN
                    -- Initiate rollback if migration fails
                    rollback_result = rollback_manager.rollback_to_last_stable_point(
                        execution_context
                    )
                    RETURN MigrationResult.FAILED_WITH_ROLLBACK(step_result, rollback_result)
                END IF
                
                execution_context.add_rollback_point(migration.version)
                execution_context.increment_step()
            }
            
            RETURN MigrationResult.SUCCESS(execution_context)
            
        } CATCH (critical_error) {
            -- Handle critical failures
            emergency_rollback = rollback_manager.emergency_rollback(execution_context)
            RETURN MigrationResult.CRITICAL_FAILURE(critical_error, emergency_rollback)
        }
    }
    
    FUNCTION execute_single_migration(
        migration: Migration, 
        context: MigrationExecutionContext
    ) -> MigrationStepResult {
        
        -- Pre-migration validation
        pre_validation = validator.validate_pre_migration_conditions(migration)
        IF NOT pre_validation.is_valid THEN
            RETURN MigrationStepResult.VALIDATION_FAILED(pre_validation.errors)
        END IF
        
        -- Create migration checkpoint
        checkpoint = create_migration_checkpoint(migration, context)
        
        -- Execute migration with monitoring
        execution_result = executor.execute_with_monitoring(migration, checkpoint)
        
        -- Post-migration validation
        post_validation = validator.validate_post_migration_conditions(migration)
        IF NOT post_validation.is_valid THEN
            -- Rollback this specific migration
            rollback_result = rollback_manager.rollback_single_migration(migration)
            RETURN MigrationStepResult.POST_VALIDATION_FAILED(
                post_validation.errors, 
                rollback_result
            )
        END IF
        
        RETURN MigrationStepResult.SUCCESS(execution_result)
    }
}
```

## Summary

This document provides comprehensive operational patterns for persistence layer management:

1. **Error Handling**: Enhanced error types, conflict resolution strategies, and retry logic with backoff
2. **Advanced Recovery**: Circuit breakers, fallback strategies, and graceful degradation patterns
3. **Monitoring Framework**: Consistency tracking, health checks, and comprehensive metrics collection
4. **Migration Infrastructure**: Version-controlled migrations with locking, validation, and rollback capabilities
5. **Zero-Downtime Patterns**: Online schema changes and shadow table migrations
6. **Migration Orchestration**: Dependency management, risk assessment, and automated rollback procedures

These patterns ensure robust operation, observability, and maintainability of the agent framework's data persistence layer while minimizing downtime and operational risk.

### Implementation Workflow

For complete operational readiness:

1. **Foundation [[storage-patterns]]**: Core storage architecture must be established
2. **Infrastructure [[connection-management]]**: Connection pools and transactions must be configured
3. **Operations (This Document)**: Implement comprehensive error handling, monitoring, and maintenance

> **Prerequisites**: This document builds upon [[storage-patterns]] foundation and [[connection-management]] infrastructure.
> Implement these operational patterns last to complete the data layer trilogy.

## Related Documentation

### Core Data Management Trilogy

- **[[storage-patterns]]** - Core storage architecture and repository patterns (foundation patterns for operations)
- **[[connection-management]]** - Connection pooling and transaction coordination (infrastructure for operations)

### Extended Framework

- [[stream-processing]] - JetStream KV patterns and stream management
- [[backup-recovery]] - Backup strategies and disaster recovery procedures
- [[data-management/CLAUDE]] - Complete data management navigation

### Integration Points

- [[../core-architecture/integration-implementation]] - Integration testing and validation patterns

---
*Agent 10 - Framework Modularization Operation - Phase 1, Group 1B*
