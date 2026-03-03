---
title: storage-patterns
type: data-management
permalink: ms-framework-docs/data-management/storage-patterns
tags:
- '#data-management #storage-architecture #repository-patterns #foundation'
---

## Storage Architecture & Repository Patterns

## Foundation Storage Patterns Guide

**Related Documentation**:

- [[connection-management]] - Connection pooling and transaction coordination
- [[persistence-operations]] - Error handling and operational procedures  
- [[stream-processing]] - JetStream patterns and stream management
- [[jetstream-kv]] - JetStream Key-Value storage configuration

## Overview

Storage architecture patterns for distributed agent state management using:

- **PostgreSQL 15 + SQLx 0.7**: Long-term persistence and complex queries
- **NATS JetStream KV**: Fast distributed state cache with TTL
- **Dual-store pattern**: KV for hot data, SQL for durability
- **Async patterns**: Non-blocking operations with proper error handling

## 1. Basic Storage Architecture

### 1.1 Storage Pattern Overview

```rust
use async_nats::jetstream;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub enum StorageLayer {
    MemoryCache,     // In-process cache
    DistributedKv,   // JetStream KV (short-term)
    RelationalDb,    // PostgreSQL (long-term)
    VectorStore,     // Optional: for semantic search
}

#[async_trait::async_trait]
pub trait DataStorage {
    async fn store(&self, key: &str, value: &[u8]) -> Result<(), StorageError>;
    async fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError>;
    async fn remove(&self, key: &str) -> Result<(), StorageError>;
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("NATS error: {0}")]
    Nats(#[from] async_nats::Error),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Timeout after {duration:?}")]
    Timeout { duration: std::time::Duration },
}
```

### 1.2 Data Categories & Two-Tier Architecture

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    SessionData,     // Temporary user sessions (KV)
    AgentState,      // Agent runtime state (KV → SQL)
    TaskInfo,        // Task metadata (SQL)
    MessageLog,      // Communication history (SQL)
    KnowledgeBase,   // Long-term facts (Vector + SQL)
}

pub struct DataRouter {
    ttl_config: std::collections::HashMap<DataType, std::time::Duration>,
}

impl DataRouter {
    pub fn new() -> Self {
        let mut ttl_config = std::collections::HashMap::new();
        ttl_config.insert(DataType::SessionData, std::time::Duration::from_secs(3600)); // 1 hour
        ttl_config.insert(DataType::AgentState, std::time::Duration::from_secs(1800));  // 30 min
        
        Self { ttl_config }
    }
    
    pub fn select_storage(&self, data_type: &DataType) -> StorageLayer {
        match data_type {
            DataType::SessionData => StorageLayer::DistributedKv,  // Fast, TTL-based
            DataType::AgentState => StorageLayer::DistributedKv,   // Primary, flushed to SQL
            DataType::TaskInfo => StorageLayer::RelationalDb,
            DataType::MessageLog => StorageLayer::RelationalDb,
            DataType::KnowledgeBase => StorageLayer::VectorStore,  // With SQL metadata
        }
    }
    
    pub fn get_ttl(&self, data_type: &DataType) -> Option<std::time::Duration> {
        self.ttl_config.get(data_type).copied()
    }
}
```

### 1.3 Hybrid Storage Pattern

```rust
use async_nats::jetstream::kv::Store;
use sqlx::{Pool, Postgres, Row};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use serde::{Serialize, Deserialize};
use std::time::Duration;

#[derive(Clone)]
pub struct HybridStateManager {
    kv_store: Store,
    sql_pool: Pool<Postgres>,
    flush_threshold: usize,
    dirty_keys: Arc<Mutex<HashSet<String>>>,
    ttl: Duration,
}

impl HybridStateManager {
    pub fn new(kv_store: Store, sql_pool: Pool<Postgres>) -> Self {
        Self {
            kv_store,
            sql_pool,
            flush_threshold: 50,
            dirty_keys: Arc::new(Mutex::new(HashSet::new())),
            ttl: Duration::from_secs(1800), // 30 minutes
        }
    }
    
    pub async fn write_state(&self, key: &str, value: &[u8]) -> Result<(), StorageError> {
        // Write to fast KV first with TTL
        self.kv_store.put(key, value.into()).await
            .map_err(StorageError::Nats)?;
        
        // Track dirty key
        {
            let mut dirty = self.dirty_keys.lock().await;
            dirty.insert(key.to_string());
            
            // Trigger flush if threshold reached
            if dirty.len() >= self.flush_threshold {
                let keys: Vec<String> = dirty.drain().collect();
                drop(dirty); // Release lock before async operation
                
                let kv_store = self.kv_store.clone();
                let sql_pool = self.sql_pool.clone();
                
                // Spawn background flush task
                tokio::spawn(async move {
                    if let Err(e) = Self::flush_to_sql(kv_store, sql_pool, keys).await {
                        tracing::error!("Failed to flush to SQL: {}", e);
                    }
                });
            }
        }
        
        Ok(())
    }
    
    pub async fn read_state(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        // Try fast KV first
        match self.kv_store.get(key).await {
            Ok(Some(entry)) => return Ok(Some(entry.value)),
            Ok(None) => {}, // Not in KV, try SQL
            Err(e) => {
                tracing::warn!("KV read failed, falling back to SQL: {}", e);
            }
        }
        
        // Fallback to SQL (lazy hydration)
        let row = sqlx::query("SELECT value FROM agent_state WHERE key = $1")
            .bind(key)
            .fetch_optional(&self.sql_pool)
            .await
            .map_err(StorageError::Database)?;
            
        if let Some(row) = row {
            let value: Vec<u8> = row.get("value");
            
            // Populate KV for next access (fire-and-forget)
            let kv_store = self.kv_store.clone();
            let key = key.to_string();
            let value_clone = value.clone();
            tokio::spawn(async move {
                if let Err(e) = kv_store.put(&key, value_clone.into()).await {
                    tracing::warn!("Failed to populate KV cache: {}", e);
                }
            });
            
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
    
    async fn flush_to_sql(
        kv_store: Store,
        sql_pool: Pool<Postgres>,
        keys: Vec<String>,
    ) -> Result<(), StorageError> {
        let mut tx = sql_pool.begin().await.map_err(StorageError::Database)?;
        
        for key in keys {
            if let Ok(Some(entry)) = kv_store.get(&key).await {
                sqlx::query(
                    "INSERT INTO agent_state (key, value, version, updated_at) 
                     VALUES ($1, $2, $3, NOW())
                     ON CONFLICT (key) 
                     DO UPDATE SET value = $2, version = $3, updated_at = NOW()"
                )
                .bind(&key)
                .bind(&entry.value)
                .bind(entry.revision as i64)
                .execute(&mut *tx)
                .await
                .map_err(StorageError::Database)?;
            }
        }
        
        tx.commit().await.map_err(StorageError::Database)?;
        Ok(())
    }
}
```

## 2. PostgreSQL Schema Patterns

### 2.1 Basic Schema Design with JSONB

```sql
-- Core schema organization
CREATE SCHEMA IF NOT EXISTS agents;
CREATE SCHEMA IF NOT EXISTS tasks;
CREATE SCHEMA IF NOT EXISTS messages;

-- Enhanced agent state with JSONB
CREATE TABLE agents.state (
    agent_id UUID NOT NULL,
    key TEXT NOT NULL,
    value JSONB NOT NULL,
    version BIGINT DEFAULT 1,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (agent_id, key)
);

-- Optimized indexing for JSONB queries
CREATE INDEX idx_state_value_gin ON agents.state USING gin(value);
CREATE INDEX idx_state_agent_key ON agents.state(agent_id, key);
CREATE INDEX idx_state_updated ON agents.state(updated_at);
CREATE INDEX idx_state_version ON agents.state(version);

-- Task tracking with comprehensive metadata
CREATE TABLE tasks.queue (
    task_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_type VARCHAR(50) NOT NULL,
    payload JSONB NOT NULL,
    metadata JSONB DEFAULT '{}',
    status VARCHAR(20) DEFAULT 'pending' 
        CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    priority INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3
);

-- Indexes for task processing efficiency
CREATE INDEX idx_tasks_status_priority ON tasks.queue(status, priority DESC);
CREATE INDEX idx_tasks_created ON tasks.queue(created_at);
CREATE INDEX idx_tasks_expires ON tasks.queue(expires_at) WHERE expires_at IS NOT NULL;
```

### 2.2 State Hydration Support

```sql
-- Agent checkpoint table for recovery
CREATE TABLE agents.checkpoints (
    agent_id UUID NOT NULL,
    checkpoint_id UUID DEFAULT gen_random_uuid(),
    state_snapshot JSONB NOT NULL,
    kv_revision BIGINT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_complete BOOLEAN DEFAULT false,
    PRIMARY KEY (agent_id, checkpoint_id)
);

CREATE INDEX idx_checkpoints_agent_created ON agents.checkpoints(agent_id, created_at DESC);
CREATE INDEX idx_checkpoints_complete ON agents.checkpoints(agent_id, is_complete, created_at DESC);

-- Optimized hydration query for agent startup
CREATE OR REPLACE FUNCTION hydrate_agent_state(p_agent_id UUID) 
RETURNS TABLE(key TEXT, value JSONB, version BIGINT, updated_at TIMESTAMP WITH TIME ZONE) 
AS $$
BEGIN
    RETURN QUERY
    SELECT s.key, s.value, s.version, s.updated_at
    FROM agents.state s
    WHERE s.agent_id = p_agent_id
    ORDER BY s.updated_at DESC;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to clean up old checkpoints
CREATE OR REPLACE FUNCTION cleanup_old_checkpoints(p_agent_id UUID, p_keep_count INTEGER DEFAULT 5)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    WITH ranked_checkpoints AS (
        SELECT checkpoint_id, 
               ROW_NUMBER() OVER (ORDER BY created_at DESC) as rn
        FROM agents.checkpoints 
        WHERE agent_id = p_agent_id
    )
    DELETE FROM agents.checkpoints 
    WHERE checkpoint_id IN (
        SELECT checkpoint_id 
        FROM ranked_checkpoints 
        WHERE rn > p_keep_count
    );
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;
```

## 4. Common Patterns

### 4.1 Repository Pattern with Dual Store

```rust
use async_trait::async_trait;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::time::Duration;

#[async_trait]
pub trait Repository<T> {
    async fn save(&self, entity: &T) -> Result<T, StorageError>;
    async fn find(&self, id: Uuid) -> Result<Option<T>, StorageError>;
    async fn update(&self, entity: &T) -> Result<T, StorageError>;
    async fn delete(&self, id: Uuid) -> Result<(), StorageError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: Uuid,
    pub agent_type: String,
    pub status: AgentStatus,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Inactive,
    Starting,
    Running,
    Stopping,
    Error,
}

pub struct AgentRepository {
    sql_pool: Pool<Postgres>,
    kv_store: Store,
    ttl: Duration,
}

impl AgentRepository {
    pub fn new(sql_pool: Pool<Postgres>, kv_store: Store) -> Self {
        Self {
            sql_pool,
            kv_store,
            ttl: Duration::from_secs(1800), // 30 minutes
        }
    }
    
    fn kv_key(id: Uuid) -> String {
        format!("agent:{}", id)
    }
}

#[async_trait]
impl Repository<Agent> for AgentRepository {
    async fn save(&self, agent: &Agent) -> Result<Agent, StorageError> {
        // Serialize agent for KV storage
        let serialized = serde_json::to_vec(agent)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        
        // Save to KV for immediate access
        let kv_key = Self::kv_key(agent.id);
        self.kv_store.put(&kv_key, serialized.into()).await
            .map_err(StorageError::Nats)?;
        
        // Async write to SQL (with timeout)
        let sql_pool = self.sql_pool.clone();
        let agent_clone = agent.clone();
        
        tokio::spawn(async move {
            let result = tokio::time::timeout(
                Duration::from_secs(30),
                sqlx::query(
                    "INSERT INTO agents.registry 
                     (agent_id, agent_type, status, metadata, created_at, updated_at) 
                     VALUES ($1, $2, $3, $4, $5, $6)
                     ON CONFLICT (agent_id) 
                     DO UPDATE SET 
                        agent_type = $2, 
                        status = $3, 
                        metadata = $4, 
                        updated_at = $6"
                )
                .bind(agent_clone.id)
                .bind(&agent_clone.agent_type)
                .bind(serde_json::to_string(&agent_clone.status).unwrap())
                .bind(&agent_clone.metadata)
                .bind(agent_clone.created_at)
                .bind(agent_clone.updated_at)
                .execute(&sql_pool)
            ).await;
            
            if let Err(e) = result {
                tracing::error!("Failed to persist agent {} to SQL: {}", agent_clone.id, e);
            }
        });
        
        Ok(agent.clone())
    }
    
    async fn find(&self, id: Uuid) -> Result<Option<Agent>, StorageError> {
        let kv_key = Self::kv_key(id);
        
        // Try KV first (fast path)
        match self.kv_store.get(&kv_key).await {
            Ok(Some(entry)) => {
                match serde_json::from_slice::<Agent>(&entry.value) {
                    Ok(agent) => return Ok(Some(agent)),
                    Err(e) => {
                        tracing::warn!("Failed to deserialize agent from KV: {}", e);
                        // Continue to SQL fallback
                    }
                }
            }
            Ok(None) => {}, // Not in KV, try SQL
            Err(e) => {
                tracing::warn!("KV lookup failed for agent {}: {}", id, e);
                // Continue to SQL fallback
            }
        }
        
        // Fallback to SQL and hydrate KV
        let row = sqlx::query(
            "SELECT agent_id, agent_type, status, metadata, created_at, updated_at 
             FROM agents.registry WHERE agent_id = $1"
        )
        .bind(id)
        .fetch_optional(&self.sql_pool)
        .await
        .map_err(StorageError::Database)?;
        
        if let Some(row) = row {
            let agent = Agent {
                id: row.get("agent_id"),
                agent_type: row.get("agent_type"),
                status: serde_json::from_str(row.get::<&str, _>("status"))
                    .map_err(|e| StorageError::Serialization(e.to_string()))?,
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            
            // Populate KV for next access (fire-and-forget)
            let kv_store = self.kv_store.clone();
            let kv_key_clone = kv_key.clone();
            let agent_clone = agent.clone();
            tokio::spawn(async move {
                if let Ok(serialized) = serde_json::to_vec(&agent_clone) {
                    if let Err(e) = kv_store.put(&kv_key_clone, serialized.into()).await {
                        tracing::warn!("Failed to populate KV cache for agent {}: {}", agent_clone.id, e);
                    }
                }
            });
            
            Ok(Some(agent))
        } else {
            Ok(None)
        }
    }
    
    async fn update(&self, agent: &Agent) -> Result<Agent, StorageError> {
        // Update is same as save for this implementation
        self.save(agent).await
    }
    
    async fn delete(&self, id: Uuid) -> Result<(), StorageError> {
        let kv_key = Self::kv_key(id);
        
        // Remove from KV
        if let Err(e) = self.kv_store.delete(&kv_key).await {
            tracing::warn!("Failed to delete agent {} from KV: {}", id, e);
        }
        
        // Remove from SQL
        sqlx::query("DELETE FROM agents.registry WHERE agent_id = $1")
            .bind(id)
            .execute(&self.sql_pool)
            .await
            .map_err(StorageError::Database)?;
        
        Ok(())
    }
}
```

### 4.2 State Lifecycle Management

```rust
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum LifecycleState {
    Cold,       // No state loaded
    Hydrating,  // Loading from SQL
    Active,     // In KV, operational
    Flushing,   // Writing to SQL
    Expired,    // TTL exceeded
}

pub struct StateLifecycleManager {
    kv_store: Store,
    sql_pool: Pool<Postgres>,
    dirty_tracker: Arc<Mutex<HashMap<Uuid, HashSet<String>>>>,
    agent_states: Arc<RwLock<HashMap<Uuid, LifecycleState>>>,
}

impl StateLifecycleManager {
    pub fn new(kv_store: Store, sql_pool: Pool<Postgres>) -> Self {
        Self {
            kv_store,
            sql_pool,
            dirty_tracker: Arc::new(Mutex::new(HashMap::new())),
            agent_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    async fn set_state(&self, agent_id: Uuid, state: LifecycleState) {
        let mut states = self.agent_states.write().await;
        states.insert(agent_id, state);
    }
    
    pub async fn get_state(&self, agent_id: Uuid) -> LifecycleState {
        let states = self.agent_states.read().await;
        states.get(&agent_id).cloned().unwrap_or(LifecycleState::Cold)
    }
    
    pub async fn hydrate_agent(&self, agent_id: Uuid) -> Result<(), StorageError> {
        self.set_state(agent_id, LifecycleState::Hydrating).await;
        
        // Load from SQL with timeout
        let rows = tokio::time::timeout(
            Duration::from_secs(30),
            sqlx::query("SELECT key, value, version FROM agents.state WHERE agent_id = $1")
                .bind(agent_id)
                .fetch_all(&self.sql_pool)
        )
        .await
        .map_err(|_| StorageError::Timeout { duration: Duration::from_secs(30) })?
        .map_err(StorageError::Database)?;
        
        // Batch load into KV with error handling
        let mut successful_loads = 0;
        let total_rows = rows.len();
        
        for row in rows {
            let key: String = row.get("key");
            let value: Vec<u8> = row.get("value");
            let kv_key = format!("{}:{}", agent_id, key);
            
            match self.kv_store.put(&kv_key, value.into()).await {
                Ok(_) => successful_loads += 1,
                Err(e) => {
                    tracing::warn!("Failed to hydrate key {} for agent {}: {}", key, agent_id, e);
                }
            }
        }
        
        // Log hydration results
        if successful_loads == total_rows {
            tracing::info!("Successfully hydrated {} keys for agent {}", successful_loads, agent_id);
        } else {
            tracing::warn!(
                "Partial hydration for agent {}: {}/{} keys loaded", 
                agent_id, successful_loads, total_rows
            );
        }
        
        self.set_state(agent_id, LifecycleState::Active).await;
        Ok(())
    }
    
    pub async fn flush_agent(&self, agent_id: Uuid) -> Result<(), StorageError> {
        self.set_state(agent_id, LifecycleState::Flushing).await;
        
        // Get and clear dirty keys
        let dirty_keys = {
            let mut tracker = self.dirty_tracker.lock().await;
            tracker.remove(&agent_id).unwrap_or_default()
        };
        
        if dirty_keys.is_empty() {
            self.set_state(agent_id, LifecycleState::Active).await;
            return Ok(());
        }
        
        // Begin transaction with timeout
        let mut tx = tokio::time::timeout(
            Duration::from_secs(60),
            self.sql_pool.begin()
        )
        .await
        .map_err(|_| StorageError::Timeout { duration: Duration::from_secs(60) })?
        .map_err(StorageError::Database)?;
        
        let mut flushed_count = 0;
        
        for key in &dirty_keys {
            let kv_key = format!("{}:{}", agent_id, key);
            
            match self.kv_store.get(&kv_key).await {
                Ok(Some(entry)) => {
                    match sqlx::query(
                        "INSERT INTO agents.state (agent_id, key, value, version, updated_at) 
                         VALUES ($1, $2, $3, $4, NOW())
                         ON CONFLICT (agent_id, key) 
                         DO UPDATE SET value = $3, version = $4, updated_at = NOW()"
                    )
                    .bind(agent_id)
                    .bind(key)
                    .bind(&entry.value)
                    .bind(entry.revision as i64)
                    .execute(&mut *tx)
                    .await
                    {
                        Ok(_) => flushed_count += 1,
                        Err(e) => {
                            tracing::error!("Failed to flush key {} for agent {}: {}", key, agent_id, e);
                            // Continue with other keys rather than failing entire transaction
                        }
                    }
                }
                Ok(None) => {
                    tracing::warn!("Key {} not found in KV for agent {} during flush", key, agent_id);
                }
                Err(e) => {
                    tracing::error!("Failed to read key {} from KV for agent {}: {}", key, agent_id, e);
                }
            }
        }
        
        // Commit transaction
        match tx.commit().await {
            Ok(_) => {
                tracing::info!("Flushed {}/{} keys for agent {}", flushed_count, dirty_keys.len(), agent_id);
                self.set_state(agent_id, LifecycleState::Active).await;
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to commit flush transaction for agent {}: {}", agent_id, e);
                // Restore dirty keys since flush failed
                {
                    let mut tracker = self.dirty_tracker.lock().await;
                    tracker.insert(agent_id, dirty_keys);
                }
                self.set_state(agent_id, LifecycleState::Active).await;
                Err(StorageError::Database(e))
            }
        }
    }
    
    pub async fn mark_dirty(&self, agent_id: Uuid, key: String) {
        let mut tracker = self.dirty_tracker.lock().await;
        tracker.entry(agent_id).or_insert_with(HashSet::new).insert(key);
    }
    
    pub async fn cleanup_expired_agents(&self) -> Result<usize, StorageError> {
        // Get agents that haven't been active recently
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(1);
        
        let expired_agents = sqlx::query_scalar::<_, Uuid>(
            "SELECT DISTINCT agent_id FROM agents.state 
             WHERE updated_at < $1"
        )
        .bind(cutoff)
        .fetch_all(&self.sql_pool)
        .await
        .map_err(StorageError::Database)?;
        
        let mut cleaned_count = 0;
        
        for agent_id in expired_agents {
            // Remove from tracking
            {
                let mut tracker = self.dirty_tracker.lock().await;
                tracker.remove(&agent_id);
            }
            {
                let mut states = self.agent_states.write().await;
                states.insert(agent_id, LifecycleState::Expired);
            }
            
            cleaned_count += 1;
        }
        
        Ok(cleaned_count)
    }
}
```

### 4.3 Enhanced Caching Pattern with TTL

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

## Summary

This document establishes the foundation storage patterns for the Mister Smith AI Agent Framework:

1. **Storage Architecture**: Multi-tiered approach with memory cache, distributed KV, and PostgreSQL
2. **Schema Patterns**: JSONB-based flexible schemas with proper indexing strategies
3. **Repository Patterns**: Dual-store repositories with automatic synchronization
4. **Lifecycle Management**: State hydration and flushing with TTL support
5. **Caching Strategies**: Three-tier caching with automatic promotion and eviction

These patterns provide the building blocks for scalable, performant data persistence while maintaining consistency and durability guarantees.

### Implementation Workflow

For complete implementation of these patterns:

1. **Foundation (This Document)**: Establish storage architecture and repository patterns
2. **Infrastructure [[connection-management]]**: Configure connection pools and transaction coordination
3. **Operations [[persistence-operations]]**: Implement error handling, monitoring, and maintenance procedures

> **Next Steps**: After implementing these storage patterns, proceed to [[connection-management]] for infrastructure setup, then [[persistence-operations]] for operational procedures.

## Related Documentation

### Core Data Management Trilogy

- **[[connection-management]]** - Connection pooling and transaction coordination (complements storage with infrastructure)
- **[[persistence-operations]]** - Error handling, monitoring, and migrations (complements storage with operations)

### Extended Framework

- [[stream-processing]] - JetStream KV patterns and stream management
- [[schema-definitions]] - Complete PostgreSQL schema specifications
- [[data-management/CLAUDE]] - Complete data management navigation

### Integration Points

- [[../core-architecture/integration-implementation]] - Integration testing and validation patterns

---
*Agent 10 - Framework Modularization Operation - Phase 1, Group 1B*
