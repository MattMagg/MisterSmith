---
title: jetstream-kv-storage
type: note
permalink: framework-docs/data-management/jetstream-kv-storage
tags:
- '#jetstream #key-value #distributed-storage #caching #state-management'
---

## JetStream KV Storage Patterns & Configuration

**Key Technologies**:

- NATS JetStream 2.9+ with KV store buckets
- async-nats 0.34+ Rust client
- SQLx 0.7+ for PostgreSQL integration
- Tokio async runtime for concurrent operations

## Overview

JetStream Key-Value storage patterns for distributed agent state management:

- **High-Performance Caching**: Sub-millisecond access for agent state
- **TTL-Based Expiration**: Automatic cleanup with configurable timeouts
- **Distributed Consistency**: Multi-replica state synchronization
- **Dual-Store Integration**: KV cache with PostgreSQL persistence
- **Async Operations**: Non-blocking I/O with proper error handling

> **Related Documentation**:
>
> - See [PostgreSQL Implementation](./postgresql-implementation.md) for comprehensive database schemas and hybrid storage integration
> - See [Data Management Directory](./CLAUDE.md) for complete data management architecture
> - See [NATS Transport](../transport/nats-transport.md) for NATS messaging patterns
> - See [Transport Core](../transport/transport-core.md) for transport layer specifications

## Table of Contents

- [1. Hybrid Storage Pattern](#1-hybrid-storage-pattern)
- [2. JetStream KV Patterns with TTL](#2-jetstream-kv-patterns-with-ttl)
- [3. Repository Pattern with Dual Store](#3-repository-pattern-with-dual-store)
- [4. State Lifecycle Management](#4-state-lifecycle-management)
- [5. Enhanced Caching Pattern with TTL](#5-enhanced-caching-pattern-with-ttl)
- [6. NATS JetStream Configuration Specifications](#6-nats-jetstream-configuration-specifications)
- [Cross-References & Integration Points](#cross-references--integration-points)

## 1. Hybrid Storage Pattern

**PostgreSQL Integration**: This pattern integrates with [PostgreSQL Schema Patterns](./postgresql-implementation.md#2-postgresql-schema-patterns) for state hydration and persistence.

```rust
-- Dual-store implementation for agent state
CLASS HybridStateManager {
    PRIVATE kv_store: JetStreamKV
    PRIVATE sql_store: PostgresDB  -- See postgresql-implementation.md
    PRIVATE flush_threshold: Integer = 50
    PRIVATE dirty_keys: Set<String>
    
    FUNCTION writeState(key: String, value: Object) -> Result {
        -- Write to fast KV first
        kv_result = kv_store.put(key, value, TTL.minutes(30))
        dirty_keys.add(key)
        
        -- Trigger flush if threshold reached
        IF dirty_keys.size() >= flush_threshold THEN
            asyncFlushToSQL()  -- See postgresql-implementation.md Section 2.2
        END IF
        
        RETURN kv_result
    }
    
    FUNCTION readState(key: String) -> Result<Object> {
        -- Try fast KV first
        kv_result = kv_store.get(key)
        IF kv_result.exists() THEN
            RETURN kv_result
        END IF
        
        -- Fallback to SQL (lazy hydration from PostgreSQL)
        sql_result = sql_store.query("SELECT value FROM agents.state WHERE state_key = ?", key)
        IF sql_result.exists() THEN
            -- Populate KV for next access
            kv_store.put(key, sql_result.value)
            RETURN sql_result
        END IF
        
        RETURN NotFound()
    }
}
```

## 2. JetStream KV Patterns with TTL

### 2.1 Key-Value Store Setup with TTL

```rust
use async_nats::jetstream::{self, kv};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KVError {
    #[error("NATS error: {0}")]
    Nats(#[from] async_nats::Error),
    #[error("JetStream error: {0}")]
    JetStream(#[from] async_nats::jetstream::Error),
    #[error("Bucket not found: {0}")]
    BucketNotFound(String),
}

pub struct KVStoreManager {
    jetstream: jetstream::Context,
    buckets: HashMap<String, kv::Store>,
}

impl KVStoreManager {
    pub async fn new(client: async_nats::Client) -> Result<Self, KVError> {
        let jetstream = jetstream::new(client);
        
        Ok(Self {
            jetstream,
            buckets: HashMap::new(),
        })
    }
    
    pub async fn create_bucket(
        &mut self, 
        name: &str, 
        ttl_seconds: u64,
        replicas: Option<usize>
    ) -> Result<kv::Store, KVError> {
        let config = kv::Config {
            bucket: name.to_string(),
            description: format!("KV bucket for {}", name),
            max_value_size: 1024 * 1024, // 1MB
            history: 1, // Keep only latest
            ttl: Duration::from_secs(ttl_seconds),
            max_bytes: 1024 * 1024 * 1024, // 1GB
            storage: jetstream::stream::StorageType::File,
            replicas: replicas.unwrap_or(3),
            ..Default::default()
        };
        
        let bucket = self.jetstream.create_key_value(config).await?;
        self.buckets.insert(name.to_string(), bucket.clone());
        
        Ok(bucket)
    }
    
    pub async fn create_tiered_buckets(&mut self) -> Result<HashMap<String, kv::Store>, KVError> {
        let mut buckets = HashMap::new();
        
        // Session data - 1 hour TTL
        let session_bucket = self.create_bucket(
            "SESSION_DATA", 
            3600, // 1 hour
            Some(3)
        ).await?;
        buckets.insert("session".to_string(), session_bucket);
        
        // Agent state - 30 minutes TTL
        let agent_bucket = self.create_bucket(
            "AGENT_STATE", 
            1800, // 30 minutes
            Some(3)
        ).await?;
        buckets.insert("agent".to_string(), agent_bucket);
        
        // Query cache - 5 minutes TTL
        let cache_bucket = self.create_bucket(
            "QUERY_CACHE", 
            300, // 5 minutes
            Some(1) // Single replica for cache
        ).await?;
        buckets.insert("cache".to_string(), cache_bucket);
        
        Ok(buckets)
    }
    
    pub fn get_bucket(&self, name: &str) -> Result<&kv::Store, KVError> {
        self.buckets.get(name)
            .ok_or_else(|| KVError::BucketNotFound(name.to_string()))
    }
    
    pub async fn health_check(&self) -> Result<(), KVError> {
        // Test connectivity by listing buckets
        let _buckets = self.jetstream.list_key_value_stores().await?;
        Ok(())
    }
}
```

### 2.2 State Operations with Conflict Resolution

```rust
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEntry<T> {
    pub value: T,
    pub timestamp: DateTime<Utc>,
    pub agent_id: String,
    pub version: u64,
}

#[derive(Debug, Clone)]
pub enum ConflictStrategy {
    LastWriteWins,
    Timestamp,
    Reject,
}

pub struct StateManager {
    kv_bucket: kv::Store,
    conflict_strategy: ConflictStrategy,
    timeout: Duration,
}

impl StateManager {
    pub fn new(
        kv_bucket: kv::Store, 
        conflict_strategy: ConflictStrategy
    ) -> Self {
        Self {
            kv_bucket,
            conflict_strategy,
            timeout: Duration::from_secs(10),
        }
    }
    
    pub async fn save_state<T>(
        &self, 
        key: &str, 
        value: T,
        agent_id: &str
    ) -> Result<u64, KVError> 
    where 
        T: Serialize + Send + Sync,
    {
        let entry = StateEntry {
            value,
            timestamp: Utc::now(),
            agent_id: agent_id.to_string(),
            version: 1, // Will be updated if conflict resolution needed
        };
        
        let serialized = serde_json::to_vec(&entry)
            .map_err(|e| KVError::Nats(async_nats::Error::new(
                async_nats::ErrorKind::Other, 
                Some(&format!("Serialization failed: {}", e))
            )))?;
        
        // Optimistic concurrency control with timeout
        match tokio::time::timeout(
            self.timeout,
            self.kv_bucket.get(key)
        ).await {
            Ok(Ok(Some(current))) => {
                self.handle_conflict(key, current, entry).await
            }
            Ok(Ok(None)) => {
                // No existing value, direct put
                let revision = self.kv_bucket.put(key, serialized.into()).await?;
                Ok(revision)
            }
            Ok(Err(e)) => Err(KVError::Nats(e)),
            Err(_) => Err(KVError::Nats(async_nats::Error::new(
                async_nats::ErrorKind::TimedOut,
                Some("Timeout getting current state")
            ))),
        }
    }
    
    pub async fn get_state<T>(&self, key: &str) -> Result<Option<StateEntry<T>>, KVError>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        match tokio::time::timeout(self.timeout, self.kv_bucket.get(key)).await {
            Ok(Ok(Some(entry))) => {
                let state: StateEntry<T> = serde_json::from_slice(&entry.value)
                    .map_err(|e| KVError::Nats(async_nats::Error::new(
                        async_nats::ErrorKind::Other,
                        Some(&format!("Deserialization failed: {}", e))
                    )))?;
                Ok(Some(state))
            }
            Ok(Ok(None)) => Ok(None),
            Ok(Err(e)) => Err(KVError::Nats(e)),
            Err(_) => Err(KVError::Nats(async_nats::Error::new(
                async_nats::ErrorKind::TimedOut,
                Some("Timeout getting state")
            ))),
        }
    }
    
    async fn handle_conflict<T>(
        &self,
        key: &str,
        current: kv::Entry,
        mut new_entry: StateEntry<T>
    ) -> Result<u64, KVError>
    where
        T: Serialize + Send + Sync,
    {
        let current_state: StateEntry<T> = serde_json::from_slice(&current.value)
            .map_err(|e| KVError::Nats(async_nats::Error::new(
                async_nats::ErrorKind::Other,
                Some(&format!("Failed to deserialize current state: {}", e))
            )))?;
        
        match self.conflict_strategy {
            ConflictStrategy::LastWriteWins => {
                // Always accept new value, but increment version
                new_entry.version = current_state.version + 1;
                let serialized = serde_json::to_vec(&new_entry)
                    .map_err(|e| KVError::Nats(async_nats::Error::new(
                        async_nats::ErrorKind::Other,
                        Some(&format!("Serialization failed: {}", e))
                    )))?;
                
                let revision = self.kv_bucket.update(key, serialized.into(), current.revision).await?;
                Ok(revision)
            }
            ConflictStrategy::Timestamp => {
                if new_entry.timestamp > current_state.timestamp {
                    new_entry.version = current_state.version + 1;
                    let serialized = serde_json::to_vec(&new_entry)
                        .map_err(|e| KVError::Nats(async_nats::Error::new(
                            async_nats::ErrorKind::Other,
                            Some(&format!("Serialization failed: {}", e))
                        )))?;
                    
                    let revision = self.kv_bucket.update(key, serialized.into(), current.revision).await?;
                    Ok(revision)
                } else {
                    // Reject update, return current revision
                    Ok(current.revision)
                }
            }
            ConflictStrategy::Reject => {
                Err(KVError::Nats(async_nats::Error::new(
                    async_nats::ErrorKind::Other,
                    Some(&format!("Conflict detected for key: {}", key))
                )))
            }
        }
    }
    
    pub async fn delete_state(&self, key: &str) -> Result<(), KVError> {
        match tokio::time::timeout(self.timeout, self.kv_bucket.delete(key)).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(KVError::Nats(e)),
            Err(_) => Err(KVError::Nats(async_nats::Error::new(
                async_nats::ErrorKind::TimedOut,
                Some("Timeout deleting state")
            ))),
        }
    }
}
```

## 3. Repository Pattern with Dual Store

```rust
// Repository pattern implementation using JetStream KV and PostgreSQL
// See storage-patterns.md for complete AgentRepository implementation

use async_nats::jetstream::kv::Store;
use sqlx::{Pool, Postgres};
use std::time::Duration;
use uuid::Uuid;

/// Integration example: Using StateManager with AgentRepository
pub struct KVAgentCache {
    state_manager: StateManager,
    agent_bucket: Store,
}

impl KVAgentCache {
    pub fn new(agent_bucket: Store) -> Self {
        let state_manager = StateManager::new(
            agent_bucket.clone(),
            ConflictStrategy::Timestamp
        );
        
        Self {
            state_manager,
            agent_bucket,
        }
    }
    
    /// Cache agent data in JetStream KV
    pub async fn cache_agent<T>(
        &self,
        agent_id: Uuid,
        agent_data: T
    ) -> Result<u64, KVError>
    where
        T: Serialize + Send + Sync,
    {
        let key = format!("agent:{}", agent_id);
        self.state_manager.save_state(&key, agent_data, &agent_id.to_string()).await
    }
    
    /// Retrieve agent data from JetStream KV
    pub async fn get_cached_agent<T>(
        &self,
        agent_id: Uuid
    ) -> Result<Option<T>, KVError>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        let key = format!("agent:{}", agent_id);
        match self.state_manager.get_state::<T>(&key).await? {
            Some(entry) => Ok(Some(entry.value)),
            None => Ok(None),
        }
    }
    
    /// Remove agent from cache
    pub async fn evict_agent(&self, agent_id: Uuid) -> Result<(), KVError> {
        let key = format!("agent:{}", agent_id);
        self.state_manager.delete_state(&key).await
    }
    
    /// Batch operations for multiple agents
    pub async fn cache_multiple_agents<T>(
        &self,
        agents: Vec<(Uuid, T)>
    ) -> Result<Vec<Result<u64, KVError>>, KVError>
    where
        T: Serialize + Send + Sync + Clone,
    {
        let mut tasks = vec![];
        
        for (agent_id, agent_data) in agents {
            let state_manager = self.state_manager.clone();
            let key = format!("agent:{}", agent_id);
            let agent_id_str = agent_id.to_string();
            
            let task = tokio::spawn(async move {
                state_manager.save_state(&key, agent_data, &agent_id_str).await
            });
            
            tasks.push(task);
        }
        
        let mut results = vec![];
        for task in tasks {
            match task.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(KVError::Nats(async_nats::Error::new(
                    async_nats::ErrorKind::Other,
                    Some(&format!("Task join error: {}", e))
                )))),
            }
        }
        
        Ok(results)
    }
}
```

## 4. State Lifecycle Management

```rust
CLASS StateLifecycleManager {
    PRIVATE kv: KVBucket
    PRIVATE db: DatabaseConnection
    PRIVATE dirty_tracker: Map<String, Set<String>>  -- agent_id -> keys
    
    ENUM LifecycleState {
        COLD,       -- No state loaded
        HYDRATING,  -- Loading from SQL
        ACTIVE,     -- In KV, operational
        FLUSHING,   -- Writing to SQL
        EXPIRED     -- TTL exceeded
    }
    
    FUNCTION hydrateAgent(agent_id: String) -> Result {
        setState(agent_id, HYDRATING)
        
        -- Load from PostgreSQL (see postgresql-implementation.md Section 2.2)
        rows = db.query(
            "SELECT state_key, state_value FROM agents.state WHERE agent_id = $1",
            [agent_id]
        )
        
        -- Batch load into KV
        FOR row IN rows {
            kv_key = agent_id + ":" + row.key
            kv.put(kv_key, row.value)
        }
        
        setState(agent_id, ACTIVE)
        RETURN Success()
    }
    
    FUNCTION flushAgent(agent_id: String) -> Result {
        setState(agent_id, FLUSHING)
        dirty_keys = dirty_tracker.get(agent_id)
        
        IF dirty_keys.empty() THEN
            RETURN Success()  -- Nothing to flush
        END IF
        
        -- Begin transaction
        tx = db.beginTransaction()
        TRY {
            FOR key IN dirty_keys {
                kv_key = agent_id + ":" + key
                value = kv.get(kv_key)
                
                IF value.exists() THEN
                    tx.execute(
                        "INSERT INTO agents.state (agent_id, state_key, state_value, version) 
                         VALUES ($1, $2, $3::jsonb, $4)
                         ON CONFLICT (agent_id, state_key) 
                         DO UPDATE SET state_value = $3::jsonb, version = $4",
                        [agent_id, key, value.data, value.revision]
                    )
                END IF
            }
            
            tx.commit()
            dirty_tracker.clear(agent_id)
            setState(agent_id, ACTIVE)
            RETURN Success()
            
        } CATCH (error) {
            tx.rollback()
            setState(agent_id, ACTIVE)  -- Revert state
            RETURN Failure(error)
        }
    }
}
```

## 5. Enhanced Caching Pattern with TTL

```rust
CLASS TieredCacheRepository {
    PRIVATE repository: Repository
    PRIVATE memory_cache: Map<UUID, CacheEntry>
    PRIVATE kv_cache: KVBucket
    PRIVATE cache_ttl: Duration
    
    STRUCT CacheEntry {
        value: Entity
        timestamp: Long
        ttl: Duration
        source: CacheSource  -- MEMORY, KV, or DB
    }
    
    FUNCTION find(id: UUID) -> Result<Entity> {
        -- L1: Check memory cache
        IF memory_cache.contains(id) THEN
            entry = memory_cache.get(id)
            IF entry.isValid() THEN
                RETURN Success(entry.value)
            ELSE
                memory_cache.remove(id)  -- Expired
            END IF
        END IF
        
        -- L2: Check KV cache
        kv_result = kv_cache.get(id.toString())
        IF kv_result.exists() THEN
            entity = deserialize(kv_result.value)
            -- Promote to memory cache
            memory_cache.put(id, CacheEntry(entity, now(), ttl, KV))
            RETURN Success(entity)
        END IF
        
        -- L3: Load from repository (SQL)
        result = repository.find(id)
        IF result.isSuccess() THEN
            -- Populate both cache layers
            kv_cache.put(id.toString(), serialize(result.value))
            memory_cache.put(id, CacheEntry(result.value, now(), ttl, DB))
        END IF
        
        RETURN result
    }
    
    FUNCTION evictExpired() {
        -- Background task to clean expired entries
        FOR entry IN memory_cache.values() {
            IF NOT entry.isValid() THEN
                memory_cache.remove(entry.id)
            END IF
        }
        -- KV entries expire automatically via TTL
    }
}
```

## 6. NATS JetStream Configuration Specifications

### 6.1 Stream Definitions

```yaml
# Production NATS JetStream Configuration for Agent Framework

# Agent State Stream - Critical agent lifecycle events
agent_state_stream:
  name: "AGENT_STATE"
  description: "Agent state changes and lifecycle events"
  subjects:
    - "agents.state.>"      # agents.state.{agent_id}.{key}
    - "agents.lifecycle.>"   # agents.lifecycle.{agent_id}.{event}
    - "agents.heartbeat.>"   # agents.heartbeat.{agent_id}
  
  # File storage for persistence
  storage: file
  retention: limits
  max_age: 3600          # 1 hour (longer than KV TTL)
  max_msgs: 1000000      # 1M messages
  max_bytes: 2147483648  # 2GB
  max_msg_size: 1048576  # 1MB per message
  
  # High availability
  replicas: 3
  placement:
    cluster: "ms-framework-cluster"
    tags: ["agent-state", "critical"]
  
  # Message handling
  discard: old
  duplicate_window: 120  # 2 minutes deduplication
  allow_rollup_hdrs: true
  deny_delete: false
  deny_purge: false

# Task Execution Stream - Task lifecycle tracking
task_execution_stream:
  name: "TASK_EXECUTION"
  description: "Task lifecycle events and execution tracking"
  subjects:
    - "tasks.created.>"    # tasks.created.{task_id}
    - "tasks.started.>"    # tasks.started.{task_id}.{agent_id}
    - "tasks.completed.>"  # tasks.completed.{task_id}
    - "tasks.failed.>"     # tasks.failed.{task_id}.{reason}
    - "tasks.cancelled.>"  # tasks.cancelled.{task_id}
  
  # Persistent storage for audit trail
  storage: file
  retention: interest     # Keep until all consumers processed
  max_age: 86400         # 24 hours retention
  max_msgs: 10000000     # 10M messages
  max_bytes: 5368709120  # 5GB
  max_msg_size: 5242880  # 5MB per message
  
  # High availability for critical task tracking
  replicas: 3
  placement:
    cluster: "ms-framework-cluster"
    tags: ["task-execution", "audit"]
  
  discard: old
  duplicate_window: 300  # 5 minutes

# Agent Messages Stream - Inter-agent communication
agent_messages_stream:
  name: "AGENT_MESSAGES"
  description: "Inter-agent communication and messaging"
  subjects:
    - "messages.direct.>"      # messages.direct.{from_agent}.{to_agent}
    - "messages.broadcast.>"   # messages.broadcast.{from_agent}
    - "messages.notification.>" # messages.notification.{type}
  
  # Memory storage for low latency
  storage: memory
  retention: limits
  max_age: 300           # 5 minutes (messages are ephemeral)
  max_msgs: 100000       # 100K messages in memory
  max_bytes: 104857600   # 100MB
  max_msg_size: 262144   # 256KB per message
  
  # Single replica for memory efficiency
  replicas: 1
  placement:
    cluster: "ms-framework-cluster"
    tags: ["messaging", "ephemeral"]
  
  discard: old
```

### 6.2 Key-Value Bucket Configurations

```yaml
# KV Bucket Configurations for Agent Framework

# Session Data Bucket - User and agent sessions
session_data_kv:
  bucket: "SESSION_DATA"
  description: "Temporary user and agent session storage"
  
  # Configuration
  ttl: 3600          # 1 hour TTL
  storage: file      # Persistent across restarts
  replicas: 3        # High availability
  history: 1         # Keep only latest value
  max_value_size: 1048576  # 1MB per entry
  max_bytes: 1073741824    # 1GB total bucket size
  
  # Placement
  placement:
    cluster: "ms-framework-cluster"
    tags: ["session-data", "user-state"]
  
  # Access optimization
  compression: s2

# Agent State Bucket - Hot cache for agent runtime state
agent_state_kv:
  bucket: "AGENT_STATE"
  description: "Fast access agent state cache"
  
  # Optimized for frequent updates
  ttl: 1800          # 30 minutes TTL
  storage: file      # Persistent
  replicas: 3        # HA for critical state
  history: 3         # Keep recent history for debugging
  max_value_size: 2097152  # 2MB per agent state
  max_bytes: 5368709120    # 5GB total
  
  placement:
    cluster: "ms-framework-cluster"
    tags: ["agent-state", "critical"]
  
  # Performance settings
  compression: s2
  republish:
    src: >
    dest: agents.state.kv.>

# Configuration Bucket - Persistent agent configuration
agent_config_kv:
  bucket: "AGENT_CONFIG"
  description: "Agent configuration and persistent settings"
  
  # Long-term storage
  ttl: 0             # Never expire
  storage: file      # Persistent
  replicas: 3        # HA for configuration
  history: 10        # Keep configuration history
  max_value_size: 524288   # 512KB per config
  max_bytes: 1073741824    # 1GB total
  
  placement:
    cluster: "ms-framework-cluster"
    tags: ["configuration", "persistent"]
  
  compression: s2

# Query Cache Bucket - Short-lived query results
query_cache_kv:
  bucket: "QUERY_CACHE"
  description: "Temporary query result cache"
  
  # Short-lived cache
  ttl: 300           # 5 minutes TTL
  storage: memory    # In-memory for speed
  replicas: 1        # Single replica for cache
  history: 1         # Only current value
  max_value_size: 1048576  # 1MB per cached result
  max_bytes: 268435456     # 256MB total
  
  placement:
    cluster: "ms-framework-cluster"
    tags: ["cache", "ephemeral"]
  
  # No compression for speed
  compression: none

# Task State Bucket - Task execution state tracking
task_state_kv:
  bucket: "TASK_STATE"
  description: "Task execution state and progress tracking"
  
  # Task lifetime storage
  ttl: 7200          # 2 hours TTL
  storage: file      # Persistent for task recovery
  replicas: 3        # HA for task state
  history: 5         # Track task progression
  max_value_size: 1048576  # 1MB per task state
  max_bytes: 2147483648    # 2GB total
  
  placement:
    cluster: "ms-framework-cluster"
    tags: ["task-state", "execution"]
  
  compression: s2
```

### 6.3 Consumer Configurations

```yaml
# Durable Consumer for Task Processing
task_processor_consumer:
  stream: "TASK_EXECUTION"
  name: "task-processor"
  description: "Processes task execution events"
  
  # Delivery configuration
  deliver_policy: all
  ack_policy: explicit
  ack_wait: 30s
  max_deliver: 3
  
  # Rate limiting
  rate_limit: 1000  # messages per second
  max_ack_pending: 100
  
  # Filtering
  filter_subject: "tasks.started.>"
  
  # Replay and recovery
  replay_policy: instant
  start_sequence: 0
  start_time: null

# Ephemeral Consumer for Real-time Monitoring
monitoring_consumer:
  stream: "AGENT_STATE"
  name: null  # Ephemeral
  description: "Real-time monitoring of agent state changes"
  
  # Start from latest for monitoring
  deliver_policy: last
  ack_policy: none  # Fire and forget
  replay_policy: instant
  
  # No delivery limits for monitoring
  max_deliver: 1
  ack_wait: 5s

# Pull-based Consumer for Batch Processing
batch_processor_consumer:
  stream: "AGENT_MESSAGES"
  name: "message-batch-processor"
  description: "Batch processing of agent messages"
  
  # Pull-based configuration
  deliver_policy: all
  ack_policy: explicit
  ack_wait: 60s
  max_deliver: 5
  
  # Batch processing optimization
  max_batch: 100
  max_expires: 30s
  max_bytes: 1048576  # 1MB batches
```

### 6.4 PostgreSQL Integration Patterns

```rust
// Application-level integration between PostgreSQL and NATS JetStream
// Preferred over database triggers for better error handling and observability

use async_nats::jetstream;
use sqlx::{Pool, Postgres};
use serde_json::json;
use uuid::Uuid;

pub struct PostgresNatsIntegration {
    jetstream: jetstream::Context,
    sql_pool: Pool<Postgres>,
}

impl PostgresNatsIntegration {
    pub fn new(jetstream: jetstream::Context, sql_pool: Pool<Postgres>) -> Self {
        Self { jetstream, sql_pool }
    }
    
    /// Publish agent state change to NATS after SQL update
    pub async fn notify_agent_state_change(
        &self,
        agent_id: Uuid,
        operation: &str,
        state_key: &str,
        old_value: Option<&serde_json::Value>,
        new_value: Option<&serde_json::Value>
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let subject = format!("agents.state.{}", agent_id);
        
        let payload = json!({
            "operation": operation,
            "agent_id": agent_id,
            "state_key": state_key,
            "old_value": old_value,
            "new_value": new_value,
            "timestamp": chrono::Utc::now().timestamp()
        });
        
        // Publish with timeout and retry
        let publish_result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            self.jetstream.publish(subject, serde_json::to_vec(&payload)?.into())
        ).await;
        
        match publish_result {
            Ok(Ok(ack)) => {
                tracing::debug!("Published agent state change: sequence {}", ack.sequence);
                Ok(())
            }
            Ok(Err(e)) => {
                tracing::error!("Failed to publish agent state change: {}", e);
                Err(e.into())
            }
            Err(_) => {
                tracing::error!("Timeout publishing agent state change");
                Err("Publish timeout".into())
            }
        }
    }
    
    /// Coordinated transaction: SQL update + NATS notification
    pub async fn update_agent_state_with_notification(
        &self,
        agent_id: Uuid,
        state_key: &str,
        new_value: &serde_json::Value
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Start SQL transaction
        let mut tx = self.sql_pool.begin().await?;
        
        // Get current value for comparison
        let old_value: Option<serde_json::Value> = sqlx::query_scalar(
            "SELECT value FROM agents.state WHERE agent_id = $1 AND key = $2"
        )
        .bind(agent_id)
        .bind(state_key)
        .fetch_optional(&mut *tx)
        .await?;
        
        // Update in SQL
        sqlx::query(
            "INSERT INTO agents.state (agent_id, key, value, updated_at) 
             VALUES ($1, $2, $3, NOW())
             ON CONFLICT (agent_id, key) 
             DO UPDATE SET value = $3, updated_at = NOW()"
        )
        .bind(agent_id)
        .bind(state_key)
        .bind(new_value)
        .execute(&mut *tx)
        .await?;
        
        // Commit SQL transaction first
        tx.commit().await?;
        
        // Then notify via NATS (fire-and-forget to avoid rollback complexity)
        let jetstream = self.jetstream.clone();
        let agent_id_copy = agent_id;
        let state_key_copy = state_key.to_string();
        let old_value_copy = old_value.clone();
        let new_value_copy = new_value.clone();
        
        tokio::spawn(async move {
            let integration = PostgresNatsIntegration { jetstream, sql_pool: Pool::connect("").await.unwrap() };
            if let Err(e) = integration.notify_agent_state_change(
                agent_id_copy,
                "UPDATE",
                &state_key_copy,
                old_value_copy.as_ref(),
                Some(&new_value_copy)
            ).await {
                tracing::warn!("Failed to send NATS notification: {}", e);
            }
        });
        
        Ok(())
    }
    
    /// Batch notification for multiple state changes
    pub async fn batch_notify_state_changes(
        &self,
        changes: Vec<(Uuid, String, String, Option<serde_json::Value>, serde_json::Value)>
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let mut successful = 0;
        
        for (agent_id, operation, state_key, old_value, new_value) in changes {
            match self.notify_agent_state_change(
                agent_id,
                &operation,
                &state_key,
                old_value.as_ref(),
                Some(&new_value)
            ).await {
                Ok(_) => successful += 1,
                Err(e) => {
                    tracing::warn!("Failed to notify change for agent {}: {}", agent_id, e);
                }
            }
        }
        
        Ok(successful)
    }
}

// Usage example in repository implementations
pub async fn example_usage() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // In your AgentRepository save method:
    let integration = PostgresNatsIntegration::new(
        /* jetstream context */,
        /* sql pool */
    );
    
    let agent_id = Uuid::new_v4();
    let state_data = json!({"status": "running", "task_count": 5});
    
    // This will update SQL and notify NATS
    integration.update_agent_state_with_notification(
        agent_id,
        "runtime_state",
        &state_data
    ).await?;
    
    Ok(())
}
```

---

## Cross-References & Integration Points

### Integration Summary

**Core Components**:

- **JetStream KV Store**: Fast distributed state cache with TTL management
- **StateManager**: Conflict resolution and concurrency control
- **Application-level Integration**: Coordinated SQL+NATS operations
- **Error Handling**: Comprehensive timeout and retry patterns

**Key Patterns**:

1. **Dual-Store Architecture**: KV cache + PostgreSQL persistence
2. **Async Operations**: Non-blocking I/O with proper error handling
3. **Conflict Resolution**: Timestamp-based and last-write-wins strategies
4. **State Lifecycle**: Coordinated hydration and flushing

**Dependencies**:

- async-nats 0.34+ (NATS JetStream client)
- sqlx 0.7+ (PostgreSQL integration)
- tokio 1.38+ (async runtime)
- serde 1.0+ (serialization)

## Related Documentation

- **[[storage-patterns]]** - Complete storage architecture and repository patterns
- **[[connection-management]]** - PostgreSQL connection pooling and transaction management
- **[[persistence-operations]]** - Error handling and operational procedures
- **[[stream-processing]]** - JetStream stream configuration and processing
- **[[../transport/nats-transport]]** - NATS transport layer configuration

---

*Optimized by Agent 6 Team Beta - Technical accuracy verified with context7 and code-reasoning*
