# Data Model: Phase 6 — Persistence & State

**Date**: 2026-03-05
**Branch**: `006-phase6-persistence-state`

## Entity Relationship Overview

```
┌──────────────┐     1:N     ┌──────────────┐
│ AgentRegistry │────────────►│  AgentState   │
│              │             │ (partitioned) │
│  agent_id PK │◄──┐        └──────────────┘
│  agent_type  │   │
│  status      │   │  1:N   ┌──────────────┐
│  parent_id FK│───┘ ┌─────►│  Checkpoint   │
│  capabilities│   │ │      └──────────────┘
│  config      │───┘ │
└──────────────┘     │
       │             │
       │ 1:N         │
       ▼             │
┌──────────────┐     │      ┌──────────────┐
│  TaskRecord   │     │      │   AuditLog    │
│              │     │      │ (partitioned) │
│  task_id PK  │     │      │  event_type   │
│  agent_id FK │     │      │  agent_id FK  │
│  status      │     │      │  action       │
│  priority    │     │      │  old/new vals  │
│  correlation │     │      └──────────────┘
└──────────────┘     │
       │             │
       │ corr_id     │
       ▼             │
┌──────────────┐     │      ┌──────────────┐
│MessageRecord  │     │      │Configuration  │
│ (partitioned) │     │      │  key          │
│  from_agent  │     │      │  value JSONB  │
│  to_agent    │     │      │  environment  │
│  priority    │     │      │  agent_id FK  │
│  correlation │     │      └──────────────┘
└──────────────┘     │
                     │
                     │      ┌──────────────┐
                     └──────│  KV Buckets   │
                            │ (JetStream)  │
                            │  SESSION_DATA│
                            │  AGENT_STATE │
                            │  QUERY_CACHE │
                            └──────────────┘
```

## Entities

### AgentRegistry

Persistent record of all known agents in the framework.

| Field | Type | Constraints | Notes |
|-------|------|-------------|-------|
| agent_id | UUID | PK, NOT NULL | gen_random_uuid() default |
| agent_type | VARCHAR(50) | NOT NULL, CHECK | Matches core AgentType enum |
| agent_name | VARCHAR(255) | NOT NULL, UNIQUE | Human-readable identifier |
| status | agent_status_type | NOT NULL, DEFAULT 'initializing' | Enum: initializing, active, idle, suspended, terminated, error |
| capabilities | JSONB | DEFAULT '{}' | Flexible capability registry |
| configuration | JSONB | DEFAULT '{}' | Agent-specific config |
| metadata | JSONB | DEFAULT '{}' | Validated via check function |
| parent_agent_id | UUID | FK → agents.registry, NULLABLE | Self-referential for supervision tree |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | |
| last_heartbeat | TIMESTAMPTZ | NULLABLE | Last health check timestamp |

**Constraints**: `no_self_parent CHECK (agent_id != parent_agent_id)`, `valid_agent_name CHECK (LENGTH(agent_name) > 0)`

**Indexes**: B-tree on status, B-tree on agent_type, B-tree on parent_agent_id

### AgentState

Key-value state storage per agent, versioned and optionally expiring.

| Field | Type | Constraints | Notes |
|-------|------|-------------|-------|
| agent_id | UUID | PK (composite), FK → registry | |
| state_key | VARCHAR(255) | PK (composite), NOT NULL | |
| state_value | JSONB | NOT NULL | Flexible state data |
| version | BIGINT | NOT NULL, DEFAULT 1, CHECK > 0 | Optimistic concurrency |
| checksum | VARCHAR(64) | NULLABLE | SHA-256 integrity hash |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | |
| expires_at | TIMESTAMPTZ | NULLABLE, CHECK > created_at | Optional TTL |

**Partitioning**: HASH by agent_id (8 partitions for load distribution)

**Indexes**: GIN on state_value, B-tree on (agent_id, state_key), B-tree on updated_at

### AgentCheckpoint

Point-in-time snapshot of an agent's full state for recovery.

| Field | Type | Constraints | Notes |
|-------|------|-------------|-------|
| agent_id | UUID | PK (composite), FK → registry | |
| checkpoint_id | UUID | PK (composite), DEFAULT gen_random_uuid() | |
| state_snapshot | JSONB | NOT NULL | Complete state at checkpoint time |
| kv_revision | BIGINT | NULLABLE | KV store revision for sync tracking |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | |

**Indexes**: B-tree on (agent_id, created_at DESC) for latest-checkpoint lookup

### TaskRecord

Task lifecycle tracking with full metadata.

| Field | Type | Constraints | Notes |
|-------|------|-------------|-------|
| task_id | UUID | PK | |
| task_type | VARCHAR(50) | NOT NULL | |
| agent_id | UUID | FK → registry, NULLABLE | Assigned agent |
| payload | JSONB | NOT NULL | Task input data |
| result | JSONB | NULLABLE | Task output data |
| metadata | JSONB | DEFAULT '{}' | Extensible metadata |
| status | task_status_type | NOT NULL, DEFAULT 'pending' | Enum: pending, queued, running, paused, completed, failed, cancelled |
| priority | INTEGER | NOT NULL, DEFAULT 2, CHECK 0-4 | Matches MessagePriority: 0=Critical..4=Bulk |
| correlation_id | UUID | NULLABLE | Links related tasks |
| parent_task_id | UUID | FK → self, NULLABLE | Task decomposition |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | |
| started_at | TIMESTAMPTZ | NULLABLE | |
| completed_at | TIMESTAMPTZ | NULLABLE | |
| expires_at | TIMESTAMPTZ | NULLABLE | Optional deadline |

**Indexes**: B-tree on (agent_id, status), B-tree on correlation_id, B-tree on (status, priority), B-tree on created_at

### MessageRecord

Inter-agent message history, time-partitioned for volume management.

| Field | Type | Constraints | Notes |
|-------|------|-------------|-------|
| id | UUID | PK | |
| from_agent_id | UUID | FK → registry, NULLABLE | Sender |
| to_agent_id | UUID | FK → registry, NULLABLE | Receiver |
| message_type | VARCHAR(50) | NOT NULL | |
| subject | TEXT | NULLABLE | NATS subject |
| content | JSONB | NOT NULL | Message payload |
| priority | INTEGER | NOT NULL, DEFAULT 2, CHECK 0-4 | Matches MessagePriority |
| status | VARCHAR(20) | NOT NULL, DEFAULT 'pending' | pending/sent/delivered/processed/failed/expired |
| correlation_id | UUID | NULLABLE | Conversation threading |
| parent_message_id | UUID | FK → self, NULLABLE | Reply chain |
| retry_count | INTEGER | NOT NULL, DEFAULT 0 | |
| max_retries | INTEGER | NOT NULL, DEFAULT 3 | |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | Partition key |
| sent_at | TIMESTAMPTZ | NULLABLE | |
| delivered_at | TIMESTAMPTZ | NULLABLE | |
| processed_at | TIMESTAMPTZ | NULLABLE | |
| expires_at | TIMESTAMPTZ | NULLABLE | |
| error_message | TEXT | NULLABLE | |

**Partitioning**: RANGE by created_at (monthly)

**Indexes**: B-tree on (from_agent_id, created_at), B-tree on (to_agent_id, created_at), B-tree on correlation_id, B-tree on (status, created_at)

### Configuration

System and per-agent configuration with environment scoping.

| Field | Type | Constraints | Notes |
|-------|------|-------------|-------|
| id | UUID | PK | |
| key | TEXT | NOT NULL | Config key |
| value | JSONB | NOT NULL | Config value |
| environment | VARCHAR(20) | NOT NULL, DEFAULT 'production' | development/staging/production/testing |
| agent_id | UUID | FK → registry, NULLABLE | NULL = system-wide |
| version | INTEGER | NOT NULL, DEFAULT 1 | |
| description | TEXT | NULLABLE | |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | |
| updated_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | |

**Constraints**: UNIQUE(key, environment, agent_id)

### AuditLogEntry

Immutable audit trail, time-partitioned. Persists events from Phase 5 AuditLogger.

| Field | Type | Constraints | Notes |
|-------|------|-------------|-------|
| id | UUID | PK | |
| event_type | TEXT | NOT NULL | |
| agent_id | UUID | FK → registry, NULLABLE | |
| resource_type | TEXT | NULLABLE | |
| resource_id | UUID | NULLABLE | |
| action | TEXT | NOT NULL | |
| old_values | JSONB | NULLABLE | Before state |
| new_values | JSONB | NULLABLE | After state |
| metadata | JSONB | DEFAULT '{}' | |
| correlation_id | UUID | NULLABLE | |
| created_at | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | Partition key |

**Partitioning**: RANGE by created_at (monthly)

**Indexes**: B-tree on (agent_id, created_at), B-tree on event_type, B-tree on correlation_id

### KV Buckets (JetStream)

Three tiered buckets with different TTL and replication settings.

| Bucket | TTL | Replicas | History | Purpose |
|--------|-----|----------|---------|---------|
| SESSION_DATA | 60 min | 3 | 1 | User/agent session state |
| AGENT_STATE | 30 min | 3 | 1 | Hot agent runtime state |
| QUERY_CACHE | 5 min | 1 | 1 | Expensive query results |

**Key format**: `{agent_id}:{state_key}` for agent state; `session:{session_id}` for sessions; `cache:{query_hash}` for cache.

**Conflict resolution**: Configurable per-bucket — default is `Timestamp` (newest wins).

## State Transitions

### Task Status Lifecycle

```
pending → queued → running → completed
                          → failed
                          → cancelled
                → paused → running (resume)
                         → cancelled
```

### Agent Status Lifecycle

```
initializing → active → idle → active
                     → suspended → active (resume)
                     → terminated
                     → error → active (recovery)
                             → terminated
```

### Persistence State Lifecycle (HybridStateManager internal)

```
COLD → HYDRATING → ACTIVE → FLUSHING → ACTIVE
                         → EXPIRED (TTL)
```

## Validation Rules

- Agent names must be non-empty and unique across the registry.
- Task priority must be 0-4, matching the `MessagePriority` enum in core.
- Message `expires_at` must be after `created_at` when set.
- Agent state `version` must be positive and monotonically increasing per key.
- State `expires_at` must be after `created_at` when set.
- KV keys must follow the `{scope}:{identifier}` naming convention.
- Audit log entries are append-only — no updates or deletes.
