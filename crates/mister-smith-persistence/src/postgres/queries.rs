//! Prepared SQL query helpers for entity CRUD operations.
//!
//! Provides standalone `async fn` helpers for agent registry, agent state,
//! and checkpoint tables. All functions use runtime `sqlx::query()` /
//! `sqlx::query_as()` — **not** compile-time macros — so no `DATABASE_URL`
//! is required at build time.
//!
//! The `status` column in `agents.registry` is a PostgreSQL custom enum
//! (`agent_status_type`). Queries cast to/from `TEXT` so Rust can work with
//! plain `String` values.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{from_sqlx_error, PersistenceError};

// ---------------------------------------------------------------------------
// Row types
// ---------------------------------------------------------------------------

/// Row type for agent registry queries.
///
/// Maps 1:1 to the `agents.registry` table. The `status` field is read as
/// `TEXT` (cast from the `agent_status_type` enum) so no custom sqlx type is
/// needed.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AgentRecord {
    pub agent_id: Uuid,
    pub agent_type: String,
    pub agent_name: String,
    pub status: String,
    pub capabilities: serde_json::Value,
    pub configuration: serde_json::Value,
    pub metadata: serde_json::Value,
    pub parent_agent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_heartbeat: Option<DateTime<Utc>>,
}

/// Row type for agent state queries.
///
/// Maps 1:1 to the `agents.state` table (hash-partitioned by `agent_id`).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StateRow {
    pub agent_id: Uuid,
    pub state_key: String,
    pub state_value: serde_json::Value,
    pub version: i64,
    pub checksum: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Row type for checkpoint queries.
///
/// Maps 1:1 to the `agents.checkpoints` table.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CheckpointRow {
    pub agent_id: Uuid,
    pub checkpoint_id: Uuid,
    pub state_snapshot: serde_json::Value,
    pub kv_revision: Option<i64>,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Agent Registry CRUD
// ---------------------------------------------------------------------------

/// Insert a new agent into `agents.registry`.
///
/// The caller supplies all user-controlled fields. Timestamps (`created_at`,
/// `updated_at`) are set to `NOW()` by the database. Returns the fully
/// populated row via `INSERT ... RETURNING`.
pub async fn insert_agent(
    pool: &PgPool,
    record: &AgentRecord,
) -> Result<AgentRecord, PersistenceError> {
    sqlx::query_as::<_, AgentRecord>(
        r#"
        INSERT INTO agents.registry (
            agent_id, agent_type, agent_name, status,
            capabilities, configuration, metadata,
            parent_agent_id, created_at, updated_at, last_heartbeat
        )
        VALUES (
            $1, $2, $3, $4::agent_status_type,
            $5, $6, $7,
            $8, NOW(), NOW(), $9
        )
        RETURNING
            agent_id, agent_type, agent_name, status::TEXT AS status,
            capabilities, configuration, metadata,
            parent_agent_id, created_at, updated_at, last_heartbeat
        "#,
    )
    .bind(record.agent_id)
    .bind(&record.agent_type)
    .bind(&record.agent_name)
    .bind(&record.status)
    .bind(&record.capabilities)
    .bind(&record.configuration)
    .bind(&record.metadata)
    .bind(record.parent_agent_id)
    .bind(record.last_heartbeat)
    .fetch_one(pool)
    .await
    .map_err(from_sqlx_error)
}

/// Find an agent by primary key.
///
/// Returns `None` when the agent does not exist (rather than an error).
pub async fn find_agent(
    pool: &PgPool,
    agent_id: Uuid,
) -> Result<Option<AgentRecord>, PersistenceError> {
    sqlx::query_as::<_, AgentRecord>(
        r#"
        SELECT
            agent_id, agent_type, agent_name, status::TEXT AS status,
            capabilities, configuration, metadata,
            parent_agent_id, created_at, updated_at, last_heartbeat
        FROM agents.registry
        WHERE agent_id = $1
        "#,
    )
    .bind(agent_id)
    .fetch_optional(pool)
    .await
    .map_err(from_sqlx_error)
}

/// Update an agent's status and touch `updated_at`.
///
/// The `status` string must be a valid `agent_status_type` variant
/// (`initializing`, `active`, `idle`, `suspended`, `terminated`, `error`).
/// A Postgres cast error will surface as `PersistenceError::DatabaseFailed`
/// if an invalid value is supplied.
pub async fn update_agent_status(
    pool: &PgPool,
    agent_id: Uuid,
    status: &str,
) -> Result<(), PersistenceError> {
    let result = sqlx::query(
        r#"
        UPDATE agents.registry
        SET status = $2::agent_status_type,
            updated_at = NOW()
        WHERE agent_id = $1
        "#,
    )
    .bind(agent_id)
    .bind(status)
    .execute(pool)
    .await
    .map_err(from_sqlx_error)?;

    if result.rows_affected() == 0 {
        return Err(PersistenceError::NotFound(format!(
            "agent {agent_id} not found"
        )));
    }
    Ok(())
}

/// Find all agents with a given `agent_type`.
pub async fn find_agents_by_type(
    pool: &PgPool,
    agent_type: &str,
) -> Result<Vec<AgentRecord>, PersistenceError> {
    sqlx::query_as::<_, AgentRecord>(
        r#"
        SELECT
            agent_id, agent_type, agent_name, status::TEXT AS status,
            capabilities, configuration, metadata,
            parent_agent_id, created_at, updated_at, last_heartbeat
        FROM agents.registry
        WHERE agent_type = $1
        ORDER BY created_at
        "#,
    )
    .bind(agent_type)
    .fetch_all(pool)
    .await
    .map_err(from_sqlx_error)
}

/// Find all agents with a given status.
///
/// The `status` string is cast to `agent_status_type` for the comparison.
pub async fn find_agents_by_status(
    pool: &PgPool,
    status: &str,
) -> Result<Vec<AgentRecord>, PersistenceError> {
    sqlx::query_as::<_, AgentRecord>(
        r#"
        SELECT
            agent_id, agent_type, agent_name, status::TEXT AS status,
            capabilities, configuration, metadata,
            parent_agent_id, created_at, updated_at, last_heartbeat
        FROM agents.registry
        WHERE status = $1::agent_status_type
        ORDER BY created_at
        "#,
    )
    .bind(status)
    .fetch_all(pool)
    .await
    .map_err(from_sqlx_error)
}

// ---------------------------------------------------------------------------
// Agent State CRUD
// ---------------------------------------------------------------------------

/// Upsert a state key-value pair for an agent.
///
/// Performs `INSERT ... ON CONFLICT (agent_id, state_key) DO UPDATE`, which
/// atomically increments the `version` column on conflict. Returns the new
/// `version` value.
pub async fn upsert_state(
    pool: &PgPool,
    agent_id: Uuid,
    key: &str,
    value: serde_json::Value,
    checksum: Option<&str>,
) -> Result<i64, PersistenceError> {
    let row: (i64,) = sqlx::query_as(
        r#"
        INSERT INTO agents.state (agent_id, state_key, state_value, checksum, created_at, updated_at)
        VALUES ($1, $2, $3, $4, NOW(), NOW())
        ON CONFLICT (agent_id, state_key) DO UPDATE
        SET state_value = EXCLUDED.state_value,
            checksum    = EXCLUDED.checksum,
            version     = agents.state.version + 1,
            updated_at  = NOW()
        RETURNING version
        "#,
    )
    .bind(agent_id)
    .bind(key)
    .bind(&value)
    .bind(checksum)
    .fetch_one(pool)
    .await
    .map_err(from_sqlx_error)?;

    Ok(row.0)
}

/// Retrieve a single state entry by composite primary key.
///
/// Returns `None` when the key does not exist for the given agent.
pub async fn get_state(
    pool: &PgPool,
    agent_id: Uuid,
    key: &str,
) -> Result<Option<StateRow>, PersistenceError> {
    sqlx::query_as::<_, StateRow>(
        r#"
        SELECT agent_id, state_key, state_value, version, checksum,
               created_at, updated_at, expires_at
        FROM agents.state
        WHERE agent_id = $1 AND state_key = $2
        "#,
    )
    .bind(agent_id)
    .bind(key)
    .fetch_optional(pool)
    .await
    .map_err(from_sqlx_error)
}

/// Retrieve all state entries for a given agent.
pub async fn get_all_state(
    pool: &PgPool,
    agent_id: Uuid,
) -> Result<Vec<StateRow>, PersistenceError> {
    sqlx::query_as::<_, StateRow>(
        r#"
        SELECT agent_id, state_key, state_value, version, checksum,
               created_at, updated_at, expires_at
        FROM agents.state
        WHERE agent_id = $1
        ORDER BY state_key
        "#,
    )
    .bind(agent_id)
    .fetch_all(pool)
    .await
    .map_err(from_sqlx_error)
}

/// Delete a single state entry. Returns `true` if the row existed.
pub async fn delete_state(
    pool: &PgPool,
    agent_id: Uuid,
    key: &str,
) -> Result<bool, PersistenceError> {
    let result = sqlx::query(
        r#"
        DELETE FROM agents.state
        WHERE agent_id = $1 AND state_key = $2
        "#,
    )
    .bind(agent_id)
    .bind(key)
    .execute(pool)
    .await
    .map_err(from_sqlx_error)?;

    Ok(result.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// Task row type
// ---------------------------------------------------------------------------

/// Row type for task record queries.
///
/// Maps 1:1 to the `tasks.records` table. The `status` field is read as
/// `TEXT` (cast from the `task_status_type` enum) so no custom sqlx type is
/// needed.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TaskRecord {
    pub task_id: Uuid,
    pub task_type: String,
    pub agent_id: Option<Uuid>,
    pub payload: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub metadata: serde_json::Value,
    pub status: String,
    pub priority: i32,
    pub correlation_id: Option<Uuid>,
    pub parent_task_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Message row type
// ---------------------------------------------------------------------------

/// Row type for message record queries.
///
/// Maps 1:1 to the `messages.records` table (range-partitioned by `created_at`).
/// The `status` column is `VARCHAR(20)`, not a custom enum, so no casting is
/// needed.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MessageRecord {
    pub id: Uuid,
    pub from_agent_id: Option<Uuid>,
    pub to_agent_id: Option<Uuid>,
    pub message_type: String,
    pub subject: Option<String>,
    pub content: serde_json::Value,
    pub priority: i32,
    pub status: String,
    pub correlation_id: Option<Uuid>,
    pub parent_message_id: Option<Uuid>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub processed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    /// Envelope message_id for idempotent deduplication. Set from
    /// `MessageEnvelope.message_id` to prevent duplicate processing
    /// under at-least-once delivery.
    pub message_id: Option<Uuid>,
}

// ---------------------------------------------------------------------------
// Checkpoint helpers
// ---------------------------------------------------------------------------

/// Insert a new checkpoint snapshot for an agent.
///
/// The `checkpoint_id` is generated by the database (`gen_random_uuid()`).
/// Returns the generated `checkpoint_id`.
pub async fn insert_checkpoint(
    pool: &PgPool,
    agent_id: Uuid,
    snapshot: serde_json::Value,
    kv_revision: Option<i64>,
) -> Result<Uuid, PersistenceError> {
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO agents.checkpoints (agent_id, state_snapshot, kv_revision, created_at)
        VALUES ($1, $2, $3, NOW())
        RETURNING checkpoint_id
        "#,
    )
    .bind(agent_id)
    .bind(&snapshot)
    .bind(kv_revision)
    .fetch_one(pool)
    .await
    .map_err(from_sqlx_error)?;

    Ok(row.0)
}

/// Retrieve the most recent checkpoint for an agent.
///
/// Returns `None` if the agent has no checkpoints.
pub async fn get_latest_checkpoint(
    pool: &PgPool,
    agent_id: Uuid,
) -> Result<Option<CheckpointRow>, PersistenceError> {
    sqlx::query_as::<_, CheckpointRow>(
        r#"
        SELECT agent_id, checkpoint_id, state_snapshot, kv_revision, created_at
        FROM agents.checkpoints
        WHERE agent_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(agent_id)
    .fetch_optional(pool)
    .await
    .map_err(from_sqlx_error)
}

// ---------------------------------------------------------------------------
// Task CRUD
// ---------------------------------------------------------------------------

/// Insert a new task into `tasks.records`.
///
/// The caller supplies all user-controlled fields. Timestamps (`created_at`)
/// defaults to `NOW()` if not provided by the database. Returns the fully
/// populated row via `INSERT ... RETURNING`.
pub async fn insert_task(
    pool: &PgPool,
    record: &TaskRecord,
) -> Result<TaskRecord, PersistenceError> {
    sqlx::query_as::<_, TaskRecord>(
        r#"
        INSERT INTO tasks.records (
            task_id, task_type, agent_id, payload, result,
            metadata, status, priority, correlation_id,
            parent_task_id, created_at, started_at,
            completed_at, expires_at
        )
        VALUES (
            $1, $2, $3, $4, $5,
            $6, $7::task_status_type, $8, $9,
            $10, $11, $12,
            $13, $14
        )
        RETURNING
            task_id, task_type, agent_id, payload, result,
            metadata, status::TEXT AS status, priority, correlation_id,
            parent_task_id, created_at, started_at,
            completed_at, expires_at
        "#,
    )
    .bind(record.task_id)
    .bind(&record.task_type)
    .bind(record.agent_id)
    .bind(&record.payload)
    .bind(&record.result)
    .bind(&record.metadata)
    .bind(&record.status)
    .bind(record.priority)
    .bind(record.correlation_id)
    .bind(record.parent_task_id)
    .bind(record.created_at)
    .bind(record.started_at)
    .bind(record.completed_at)
    .bind(record.expires_at)
    .fetch_one(pool)
    .await
    .map_err(from_sqlx_error)
}

/// Find a task by primary key.
///
/// Returns `None` when the task does not exist.
pub async fn find_task(
    pool: &PgPool,
    task_id: Uuid,
) -> Result<Option<TaskRecord>, PersistenceError> {
    sqlx::query_as::<_, TaskRecord>(
        r#"
        SELECT
            task_id, task_type, agent_id, payload, result,
            metadata, status::TEXT AS status, priority, correlation_id,
            parent_task_id, created_at, started_at,
            completed_at, expires_at
        FROM tasks.records
        WHERE task_id = $1
        "#,
    )
    .bind(task_id)
    .fetch_optional(pool)
    .await
    .map_err(from_sqlx_error)
}

/// Update a task's status.
///
/// The `status` string must be a valid `task_status_type` variant
/// (`pending`, `queued`, `running`, `paused`, `completed`, `failed`, `cancelled`).
pub async fn update_task_status(
    pool: &PgPool,
    task_id: Uuid,
    status: &str,
) -> Result<(), PersistenceError> {
    let result = sqlx::query(
        r#"
        UPDATE tasks.records
        SET status = $2::task_status_type
        WHERE task_id = $1
        "#,
    )
    .bind(task_id)
    .bind(status)
    .execute(pool)
    .await
    .map_err(from_sqlx_error)?;

    if result.rows_affected() == 0 {
        return Err(PersistenceError::NotFound(format!(
            "task {task_id} not found"
        )));
    }
    Ok(())
}

/// Update a task's metadata document and return the updated row.
pub async fn update_task_metadata(
    pool: &PgPool,
    task_id: Uuid,
    metadata: serde_json::Value,
) -> Result<TaskRecord, PersistenceError> {
    sqlx::query_as::<_, TaskRecord>(
        r#"
        UPDATE tasks.records
        SET metadata = $2
        WHERE task_id = $1
        RETURNING
            task_id, task_type, agent_id, payload, result,
            metadata, status::TEXT AS status, priority, correlation_id,
            parent_task_id, created_at, started_at,
            completed_at, expires_at
        "#,
    )
    .bind(task_id)
    .bind(&metadata)
    .fetch_one(pool)
    .await
    .map_err(from_sqlx_error)
}

/// Find all tasks assigned to a given agent.
pub async fn find_tasks_by_agent(
    pool: &PgPool,
    agent_id: Uuid,
) -> Result<Vec<TaskRecord>, PersistenceError> {
    sqlx::query_as::<_, TaskRecord>(
        r#"
        SELECT
            task_id, task_type, agent_id, payload, result,
            metadata, status::TEXT AS status, priority, correlation_id,
            parent_task_id, created_at, started_at,
            completed_at, expires_at
        FROM tasks.records
        WHERE agent_id = $1
        ORDER BY created_at
        "#,
    )
    .bind(agent_id)
    .fetch_all(pool)
    .await
    .map_err(from_sqlx_error)
}

/// Find tasks within a time range (by `created_at`).
pub async fn find_tasks_by_time_range(
    pool: &PgPool,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<TaskRecord>, PersistenceError> {
    sqlx::query_as::<_, TaskRecord>(
        r#"
        SELECT
            task_id, task_type, agent_id, payload, result,
            metadata, status::TEXT AS status, priority, correlation_id,
            parent_task_id, created_at, started_at,
            completed_at, expires_at
        FROM tasks.records
        WHERE created_at >= $1 AND created_at < $2
        ORDER BY created_at
        "#,
    )
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
    .map_err(from_sqlx_error)
}

/// Find tasks by correlation ID.
pub async fn find_tasks_by_correlation(
    pool: &PgPool,
    correlation_id: Uuid,
) -> Result<Vec<TaskRecord>, PersistenceError> {
    sqlx::query_as::<_, TaskRecord>(
        r#"
        SELECT
            task_id, task_type, agent_id, payload, result,
            metadata, status::TEXT AS status, priority, correlation_id,
            parent_task_id, created_at, started_at,
            completed_at, expires_at
        FROM tasks.records
        WHERE correlation_id = $1
        ORDER BY created_at
        "#,
    )
    .bind(correlation_id)
    .fetch_all(pool)
    .await
    .map_err(from_sqlx_error)
}

/// Find tasks by status and minimum priority.
pub async fn find_tasks_by_status_and_priority(
    pool: &PgPool,
    status: &str,
    min_priority: i32,
) -> Result<Vec<TaskRecord>, PersistenceError> {
    sqlx::query_as::<_, TaskRecord>(
        r#"
        SELECT
            task_id, task_type, agent_id, payload, result,
            metadata, status::TEXT AS status, priority, correlation_id,
            parent_task_id, created_at, started_at,
            completed_at, expires_at
        FROM tasks.records
        WHERE status = $1::task_status_type AND priority >= $2
        ORDER BY priority DESC, created_at
        "#,
    )
    .bind(status)
    .bind(min_priority)
    .fetch_all(pool)
    .await
    .map_err(from_sqlx_error)
}

// ---------------------------------------------------------------------------
// Message CRUD
// ---------------------------------------------------------------------------

/// Insert a new message into `messages.records`.
///
/// Returns the fully populated row via `INSERT ... RETURNING`.
/// Insert a message record with idempotency support.
///
/// When `message_id` is set, duplicate inserts are rejected via the
/// `idx_messages_dedup` unique index (`ON CONFLICT DO NOTHING`). The
/// returned record is either the newly inserted row or the existing
/// row that matched on `message_id`.
pub async fn insert_message(
    pool: &PgPool,
    record: &MessageRecord,
) -> Result<MessageRecord, PersistenceError> {
    // If message_id is set, use idempotent insert with conflict detection.
    if let Some(msg_id) = record.message_id {
        // Try to insert; if a duplicate message_id exists, return the existing row.
        let maybe_inserted = sqlx::query_as::<_, MessageRecord>(
            r#"
            INSERT INTO messages.records (
                id, from_agent_id, to_agent_id, message_type, subject,
                content, priority, status, correlation_id, parent_message_id,
                retry_count, max_retries, created_at, sent_at,
                delivered_at, processed_at, expires_at, error_message, message_id
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9, $10,
                $11, $12, $13, $14,
                $15, $16, $17, $18, $19
            )
            ON CONFLICT (message_id) WHERE message_id IS NOT NULL DO NOTHING
            RETURNING
                id, from_agent_id, to_agent_id, message_type, subject,
                content, priority, status, correlation_id, parent_message_id,
                retry_count, max_retries, created_at, sent_at,
                delivered_at, processed_at, expires_at, error_message, message_id
            "#,
        )
        .bind(record.id)
        .bind(record.from_agent_id)
        .bind(record.to_agent_id)
        .bind(&record.message_type)
        .bind(&record.subject)
        .bind(&record.content)
        .bind(record.priority)
        .bind(&record.status)
        .bind(record.correlation_id)
        .bind(record.parent_message_id)
        .bind(record.retry_count)
        .bind(record.max_retries)
        .bind(record.created_at)
        .bind(record.sent_at)
        .bind(record.delivered_at)
        .bind(record.processed_at)
        .bind(record.expires_at)
        .bind(&record.error_message)
        .bind(msg_id)
        .fetch_optional(pool)
        .await
        .map_err(from_sqlx_error)?;

        // If DO NOTHING fired, fetch the existing record by message_id.
        match maybe_inserted {
            Some(row) => Ok(row),
            None => sqlx::query_as::<_, MessageRecord>(
                r#"
                    SELECT
                        id, from_agent_id, to_agent_id, message_type, subject,
                        content, priority, status, correlation_id, parent_message_id,
                        retry_count, max_retries, created_at, sent_at,
                        delivered_at, processed_at, expires_at, error_message, message_id
                    FROM messages.records
                    WHERE message_id = $1
                    LIMIT 1
                    "#,
            )
            .bind(msg_id)
            .fetch_one(pool)
            .await
            .map_err(from_sqlx_error),
        }
    } else {
        // No message_id — standard insert without dedup.
        sqlx::query_as::<_, MessageRecord>(
            r#"
            INSERT INTO messages.records (
                id, from_agent_id, to_agent_id, message_type, subject,
                content, priority, status, correlation_id, parent_message_id,
                retry_count, max_retries, created_at, sent_at,
                delivered_at, processed_at, expires_at, error_message, message_id
            )
            VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9, $10,
                $11, $12, $13, $14,
                $15, $16, $17, $18, $19
            )
            RETURNING
                id, from_agent_id, to_agent_id, message_type, subject,
                content, priority, status, correlation_id, parent_message_id,
                retry_count, max_retries, created_at, sent_at,
                delivered_at, processed_at, expires_at, error_message, message_id
            "#,
        )
        .bind(record.id)
        .bind(record.from_agent_id)
        .bind(record.to_agent_id)
        .bind(&record.message_type)
        .bind(&record.subject)
        .bind(&record.content)
        .bind(record.priority)
        .bind(&record.status)
        .bind(record.correlation_id)
        .bind(record.parent_message_id)
        .bind(record.retry_count)
        .bind(record.max_retries)
        .bind(record.created_at)
        .bind(record.sent_at)
        .bind(record.delivered_at)
        .bind(record.processed_at)
        .bind(record.expires_at)
        .bind(&record.error_message)
        .bind(record.message_id)
        .fetch_one(pool)
        .await
        .map_err(from_sqlx_error)
    }
}

/// Find a message by primary key.
///
/// Returns `None` when the message does not exist.
pub async fn find_message(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<MessageRecord>, PersistenceError> {
    sqlx::query_as::<_, MessageRecord>(
        r#"
        SELECT
            id, from_agent_id, to_agent_id, message_type, subject,
            content, priority, status, correlation_id, parent_message_id,
            retry_count, max_retries, created_at, sent_at,
            delivered_at, processed_at, expires_at, error_message, message_id
        FROM messages.records
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(from_sqlx_error)
}

/// Update a message's status.
///
/// The `status` column is `VARCHAR(20)`, so no enum casting is needed.
pub async fn update_message_status(
    pool: &PgPool,
    id: Uuid,
    status: &str,
) -> Result<(), PersistenceError> {
    let result = sqlx::query(
        r#"
        UPDATE messages.records
        SET status = $2
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(status)
    .execute(pool)
    .await
    .map_err(from_sqlx_error)?;

    if result.rows_affected() == 0 {
        return Err(PersistenceError::NotFound(format!(
            "message {id} not found"
        )));
    }
    Ok(())
}

/// Find messages sent by a given agent within a time range.
pub async fn find_messages_by_sender(
    pool: &PgPool,
    agent_id: Uuid,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<MessageRecord>, PersistenceError> {
    sqlx::query_as::<_, MessageRecord>(
        r#"
        SELECT
            id, from_agent_id, to_agent_id, message_type, subject,
            content, priority, status, correlation_id, parent_message_id,
            retry_count, max_retries, created_at, sent_at,
            delivered_at, processed_at, expires_at, error_message, message_id
        FROM messages.records
        WHERE from_agent_id = $1 AND created_at >= $2 AND created_at < $3
        ORDER BY created_at
        "#,
    )
    .bind(agent_id)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
    .map_err(from_sqlx_error)
}

/// Find messages addressed to a given agent.
pub async fn find_messages_by_receiver(
    pool: &PgPool,
    agent_id: Uuid,
) -> Result<Vec<MessageRecord>, PersistenceError> {
    sqlx::query_as::<_, MessageRecord>(
        r#"
        SELECT
            id, from_agent_id, to_agent_id, message_type, subject,
            content, priority, status, correlation_id, parent_message_id,
            retry_count, max_retries, created_at, sent_at,
            delivered_at, processed_at, expires_at, error_message, message_id
        FROM messages.records
        WHERE to_agent_id = $1
        ORDER BY created_at
        "#,
    )
    .bind(agent_id)
    .fetch_all(pool)
    .await
    .map_err(from_sqlx_error)
}

/// Find messages by correlation ID.
pub async fn find_messages_by_correlation(
    pool: &PgPool,
    correlation_id: Uuid,
) -> Result<Vec<MessageRecord>, PersistenceError> {
    sqlx::query_as::<_, MessageRecord>(
        r#"
        SELECT
            id, from_agent_id, to_agent_id, message_type, subject,
            content, priority, status, correlation_id, parent_message_id,
            retry_count, max_retries, created_at, sent_at,
            delivered_at, processed_at, expires_at, error_message, message_id
        FROM messages.records
        WHERE correlation_id = $1
        ORDER BY created_at
        "#,
    )
    .bind(correlation_id)
    .fetch_all(pool)
    .await
    .map_err(from_sqlx_error)
}

/// Find messages within a time range (by `created_at`).
pub async fn find_messages_by_time_range(
    pool: &PgPool,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<MessageRecord>, PersistenceError> {
    sqlx::query_as::<_, MessageRecord>(
        r#"
        SELECT
            id, from_agent_id, to_agent_id, message_type, subject,
            content, priority, status, correlation_id, parent_message_id,
            retry_count, max_retries, created_at, sent_at,
            delivered_at, processed_at, expires_at, error_message, message_id
        FROM messages.records
        WHERE created_at >= $1 AND created_at < $2
        ORDER BY created_at
        "#,
    )
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
    .map_err(from_sqlx_error)
}

// ---------------------------------------------------------------------------
// Audit log row type
// ---------------------------------------------------------------------------

/// Row type for audit log queries.
///
/// Maps 1:1 to the `audit_log` table (range-partitioned by `created_at`).
/// Persists events from Phase 5 AuditLogger. Entries are append-only.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditEntry {
    pub id: Uuid,
    pub event_type: String,
    pub agent_id: Option<Uuid>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub action: String,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub metadata: serde_json::Value,
    pub correlation_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

struct AuditBatchInsert {
    ids: Vec<Uuid>,
    event_types: Vec<String>,
    agent_ids: Vec<Option<Uuid>>,
    resource_types: Vec<Option<String>>,
    resource_ids: Vec<Option<Uuid>>,
    actions: Vec<String>,
    old_values: Vec<Option<serde_json::Value>>,
    new_values: Vec<Option<serde_json::Value>>,
    metadata: Vec<serde_json::Value>,
    correlation_ids: Vec<Option<Uuid>>,
    created_ats: Vec<DateTime<Utc>>,
}

impl AuditBatchInsert {
    const SQL: &str = r#"
        INSERT INTO audit_log (
            id, event_type, agent_id, resource_type, resource_id,
            action, old_values, new_values, metadata,
            correlation_id, created_at
        )
        SELECT *
        FROM UNNEST(
            $1::uuid[],
            $2::text[],
            $3::uuid[],
            $4::text[],
            $5::uuid[],
            $6::text[],
            $7::jsonb[],
            $8::jsonb[],
            $9::jsonb[],
            $10::uuid[],
            $11::timestamptz[]
        )
    "#;

    fn from_entries(entries: &[AuditEntry]) -> Self {
        Self {
            ids: entries.iter().map(|entry| entry.id).collect(),
            event_types: entries
                .iter()
                .map(|entry| entry.event_type.clone())
                .collect(),
            agent_ids: entries.iter().map(|entry| entry.agent_id).collect(),
            resource_types: entries
                .iter()
                .map(|entry| entry.resource_type.clone())
                .collect(),
            resource_ids: entries.iter().map(|entry| entry.resource_id).collect(),
            actions: entries.iter().map(|entry| entry.action.clone()).collect(),
            old_values: entries
                .iter()
                .map(|entry| entry.old_values.clone())
                .collect(),
            new_values: entries
                .iter()
                .map(|entry| entry.new_values.clone())
                .collect(),
            metadata: entries.iter().map(|entry| entry.metadata.clone()).collect(),
            correlation_ids: entries.iter().map(|entry| entry.correlation_id).collect(),
            created_ats: entries.iter().map(|entry| entry.created_at).collect(),
        }
    }
}

// ---------------------------------------------------------------------------
// Audit log CRUD
// ---------------------------------------------------------------------------

/// Append an audit entry (insert-only).
pub async fn insert_audit_entry(pool: &PgPool, entry: &AuditEntry) -> Result<(), PersistenceError> {
    sqlx::query(
        r#"
        INSERT INTO audit_log (
            id, event_type, agent_id, resource_type, resource_id,
            action, old_values, new_values, metadata,
            correlation_id, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(entry.id)
    .bind(&entry.event_type)
    .bind(entry.agent_id)
    .bind(&entry.resource_type)
    .bind(entry.resource_id)
    .bind(&entry.action)
    .bind(&entry.old_values)
    .bind(&entry.new_values)
    .bind(&entry.metadata)
    .bind(entry.correlation_id)
    .bind(entry.created_at)
    .execute(pool)
    .await
    .map_err(from_sqlx_error)?;

    Ok(())
}

/// Batch insert audit entries in a single transaction.
///
/// Returns the number of entries inserted. Rolls back on any failure.
pub async fn insert_audit_batch(
    pool: &PgPool,
    entries: &[AuditEntry],
) -> Result<usize, PersistenceError> {
    if entries.is_empty() {
        return Ok(0);
    }

    let mut tx = pool.begin().await.map_err(from_sqlx_error)?;
    let batch = AuditBatchInsert::from_entries(entries);
    let count = sqlx::query(AuditBatchInsert::SQL)
        .bind(&batch.ids[..])
        .bind(&batch.event_types[..])
        .bind(&batch.agent_ids[..])
        .bind(&batch.resource_types[..])
        .bind(&batch.resource_ids[..])
        .bind(&batch.actions[..])
        .bind(&batch.old_values[..])
        .bind(&batch.new_values[..])
        .bind(&batch.metadata[..])
        .bind(&batch.correlation_ids[..])
        .bind(&batch.created_ats[..])
        .execute(&mut *tx)
        .await
        .map_err(from_sqlx_error)?
        .rows_affected() as usize;

    tx.commit().await.map_err(from_sqlx_error)?;
    Ok(count)
}

/// Find audit entries by agent within a time range.
pub async fn find_audit_by_agent(
    pool: &PgPool,
    agent_id: Uuid,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<AuditEntry>, PersistenceError> {
    sqlx::query_as::<_, AuditEntry>(
        r#"
        SELECT
            id, event_type, agent_id, resource_type, resource_id,
            action, old_values, new_values, metadata,
            correlation_id, created_at
        FROM audit_log
        WHERE agent_id = $1 AND created_at >= $2 AND created_at < $3
        ORDER BY created_at
        "#,
    )
    .bind(agent_id)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
    .map_err(from_sqlx_error)
}

// ---------------------------------------------------------------------------
// Configuration row type
// ---------------------------------------------------------------------------

/// Row type for configuration queries.
///
/// Maps 1:1 to the `configurations` table. Environment-scoped settings
/// with optional agent-level granularity.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ConfigRecord {
    pub id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub environment: String,
    pub agent_id: Option<Uuid>,
    pub version: i32,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Configuration CRUD
// ---------------------------------------------------------------------------

/// Upsert a configuration entry.
///
/// Uses `INSERT ... ON CONFLICT` with partial unique indexes to atomically
/// create or update the config value. Increments the version on conflict.
/// Returns the new version.
pub async fn upsert_config(
    pool: &PgPool,
    key: &str,
    value: serde_json::Value,
    environment: &str,
    agent_id: Option<Uuid>,
    description: Option<&str>,
) -> Result<i32, PersistenceError> {
    let id = Uuid::new_v4();

    let row: (i32,) = if let Some(aid) = agent_id {
        sqlx::query_as(
            r#"
            INSERT INTO config.configurations (id, key, value, environment, agent_id, description, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (key, environment, agent_id) WHERE agent_id IS NOT NULL DO UPDATE
            SET value       = EXCLUDED.value,
                description = COALESCE(EXCLUDED.description, config.configurations.description),
                version     = config.configurations.version + 1,
                updated_at  = NOW()
            RETURNING version
            "#,
        )
        .bind(id)
        .bind(key)
        .bind(&value)
        .bind(environment)
        .bind(aid)
        .bind(description)
        .fetch_one(pool)
        .await
        .map_err(from_sqlx_error)?
    } else {
        sqlx::query_as(
            r#"
            INSERT INTO config.configurations (id, key, value, environment, description, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            ON CONFLICT (key, environment) WHERE agent_id IS NULL DO UPDATE
            SET value       = EXCLUDED.value,
                description = COALESCE(EXCLUDED.description, config.configurations.description),
                version     = config.configurations.version + 1,
                updated_at  = NOW()
            RETURNING version
            "#,
        )
        .bind(id)
        .bind(key)
        .bind(&value)
        .bind(environment)
        .bind(description)
        .fetch_one(pool)
        .await
        .map_err(from_sqlx_error)?
    };

    Ok(row.0)
}

/// Get a specific configuration entry by key and environment.
pub async fn get_config(
    pool: &PgPool,
    key: &str,
    environment: &str,
    agent_id: Option<Uuid>,
) -> Result<Option<ConfigRecord>, PersistenceError> {
    let query = if agent_id.is_some() {
        r#"
        SELECT id, key, value, environment, agent_id, version, description, created_at, updated_at
        FROM config.configurations
        WHERE key = $1 AND environment = $2 AND agent_id = $3
        "#
    } else {
        r#"
        SELECT id, key, value, environment, agent_id, version, description, created_at, updated_at
        FROM config.configurations
        WHERE key = $1 AND environment = $2 AND agent_id IS NULL
        "#
    };

    let result = if let Some(aid) = agent_id {
        sqlx::query_as::<_, ConfigRecord>(query)
            .bind(key)
            .bind(environment)
            .bind(aid)
            .fetch_optional(pool)
            .await
    } else {
        sqlx::query_as::<_, ConfigRecord>(query)
            .bind(key)
            .bind(environment)
            .fetch_optional(pool)
            .await
    };

    result.map_err(from_sqlx_error)
}

/// Get all configuration entries for a given environment.
pub async fn get_config_by_environment(
    pool: &PgPool,
    environment: &str,
) -> Result<Vec<ConfigRecord>, PersistenceError> {
    sqlx::query_as::<_, ConfigRecord>(
        r#"
        SELECT id, key, value, environment, agent_id, version, description, created_at, updated_at
        FROM config.configurations
        WHERE environment = $1
        ORDER BY key
        "#,
    )
    .bind(environment)
    .fetch_all(pool)
    .await
    .map_err(from_sqlx_error)
}

/// Get the history of a configuration key (all versions across environments).
///
/// Returns entries ordered by version descending (newest first).
pub async fn get_config_history(
    pool: &PgPool,
    key: &str,
) -> Result<Vec<ConfigRecord>, PersistenceError> {
    sqlx::query_as::<_, ConfigRecord>(
        r#"
        SELECT id, key, value, environment, agent_id, version, description, created_at, updated_at
        FROM config.configurations
        WHERE key = $1
        ORDER BY version DESC, updated_at DESC
        "#,
    )
    .bind(key)
    .fetch_all(pool)
    .await
    .map_err(from_sqlx_error)
}

// ---------------------------------------------------------------------------
// Transaction helper
// ---------------------------------------------------------------------------

/// Begin a new database transaction.
///
/// Callers can use the returned transaction handle with raw query helpers
/// for multi-operation atomicity. Call `.commit()` when done.
pub async fn begin_transaction(
    pool: &PgPool,
) -> Result<sqlx::Transaction<'_, sqlx::Postgres>, PersistenceError> {
    pool.begin().await.map_err(from_sqlx_error)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to build a sample AgentRecord for tests.
    fn sample_agent() -> AgentRecord {
        AgentRecord {
            agent_id: Uuid::new_v4(),
            agent_type: "orchestrator".to_string(),
            agent_name: "test-agent-1".to_string(),
            status: "initializing".to_string(),
            capabilities: serde_json::json!({"tools": ["search", "summarize"]}),
            configuration: serde_json::json!({"model": "gpt-4"}),
            metadata: serde_json::json!({}),
            parent_agent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_heartbeat: None,
        }
    }

    /// Helper to build a sample StateRow for tests.
    fn sample_state() -> StateRow {
        StateRow {
            agent_id: Uuid::new_v4(),
            state_key: "conversation.context".to_string(),
            state_value: serde_json::json!({"messages": []}),
            version: 1,
            checksum: Some("abc123def456".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        }
    }

    /// Helper to build a sample CheckpointRow for tests.
    fn sample_checkpoint() -> CheckpointRow {
        CheckpointRow {
            agent_id: Uuid::new_v4(),
            checkpoint_id: Uuid::new_v4(),
            state_snapshot: serde_json::json!({"keys": {"a": 1, "b": 2}}),
            kv_revision: Some(42),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn agent_record_construction_and_serialization() {
        let record = sample_agent();
        assert_eq!(record.agent_type, "orchestrator");
        assert_eq!(record.status, "initializing");
        assert!(record.parent_agent_id.is_none());
        assert!(record.last_heartbeat.is_none());

        // Round-trip through JSON
        let json = serde_json::to_string(&record).expect("serialize AgentRecord");
        let deserialized: AgentRecord =
            serde_json::from_str(&json).expect("deserialize AgentRecord");
        assert_eq!(deserialized.agent_id, record.agent_id);
        assert_eq!(deserialized.agent_name, record.agent_name);
        assert_eq!(deserialized.capabilities, record.capabilities);
    }

    #[test]
    fn agent_record_with_parent() {
        let parent_id = Uuid::new_v4();
        let mut record = sample_agent();
        record.parent_agent_id = Some(parent_id);

        let json = serde_json::to_string(&record).expect("serialize");
        let deserialized: AgentRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.parent_agent_id, Some(parent_id));
    }

    #[test]
    fn state_row_construction_and_serialization() {
        let state = sample_state();
        assert_eq!(state.state_key, "conversation.context");
        assert_eq!(state.version, 1);
        assert!(state.checksum.is_some());
        assert!(state.expires_at.is_none());

        // Round-trip through JSON
        let json = serde_json::to_string(&state).expect("serialize StateRow");
        let deserialized: StateRow = serde_json::from_str(&json).expect("deserialize StateRow");
        assert_eq!(deserialized.agent_id, state.agent_id);
        assert_eq!(deserialized.state_key, state.state_key);
        assert_eq!(deserialized.state_value, state.state_value);
        assert_eq!(deserialized.version, state.version);
    }

    #[test]
    fn state_row_without_checksum() {
        let mut state = sample_state();
        state.checksum = None;

        let json = serde_json::to_string(&state).expect("serialize");
        let deserialized: StateRow = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.checksum.is_none());
    }

    #[test]
    fn state_row_with_expiry() {
        let mut state = sample_state();
        state.expires_at = Some(Utc::now() + chrono::Duration::hours(1));

        let json = serde_json::to_string(&state).expect("serialize");
        let deserialized: StateRow = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.expires_at.is_some());
    }

    #[test]
    fn checkpoint_row_construction_and_serialization() {
        let checkpoint = sample_checkpoint();
        assert!(checkpoint.kv_revision.is_some());
        assert_eq!(checkpoint.kv_revision, Some(42));

        // Round-trip through JSON
        let json = serde_json::to_string(&checkpoint).expect("serialize CheckpointRow");
        let deserialized: CheckpointRow =
            serde_json::from_str(&json).expect("deserialize CheckpointRow");
        assert_eq!(deserialized.agent_id, checkpoint.agent_id);
        assert_eq!(deserialized.checkpoint_id, checkpoint.checkpoint_id);
        assert_eq!(deserialized.state_snapshot, checkpoint.state_snapshot);
        assert_eq!(deserialized.kv_revision, checkpoint.kv_revision);
    }

    #[test]
    fn checkpoint_row_without_kv_revision() {
        let mut checkpoint = sample_checkpoint();
        checkpoint.kv_revision = None;

        let json = serde_json::to_string(&checkpoint).expect("serialize");
        let deserialized: CheckpointRow = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.kv_revision.is_none());
    }

    // -------------------------------------------------------------------
    // TaskRecord tests (T039)
    // -------------------------------------------------------------------

    /// Helper to build a sample TaskRecord for tests.
    fn sample_task() -> TaskRecord {
        TaskRecord {
            task_id: Uuid::new_v4(),
            task_type: "research".to_string(),
            agent_id: Some(Uuid::new_v4()),
            payload: serde_json::json!({"query": "test"}),
            result: None,
            metadata: serde_json::json!({}),
            status: "pending".to_string(),
            priority: 2,
            correlation_id: Some(Uuid::new_v4()),
            parent_task_id: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            expires_at: None,
        }
    }

    #[test]
    fn task_record_construction_and_serialization() {
        let record = sample_task();
        assert_eq!(record.task_type, "research");
        assert_eq!(record.status, "pending");
        assert_eq!(record.priority, 2);
        assert!(record.agent_id.is_some());
        assert!(record.result.is_none());
        assert!(record.started_at.is_none());

        // Round-trip through JSON
        let json = serde_json::to_string(&record).expect("serialize TaskRecord");
        let deserialized: TaskRecord = serde_json::from_str(&json).expect("deserialize TaskRecord");
        assert_eq!(deserialized.task_id, record.task_id);
        assert_eq!(deserialized.task_type, record.task_type);
        assert_eq!(deserialized.payload, record.payload);
        assert_eq!(deserialized.priority, record.priority);
        assert_eq!(deserialized.correlation_id, record.correlation_id);
    }

    #[test]
    fn task_record_with_result_and_timestamps() {
        let mut record = sample_task();
        record.result = Some(serde_json::json!({"answer": "done"}));
        record.started_at = Some(Utc::now());
        record.completed_at = Some(Utc::now());
        record.status = "completed".to_string();

        let json = serde_json::to_string(&record).expect("serialize");
        let deserialized: TaskRecord = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.result.is_some());
        assert!(deserialized.started_at.is_some());
        assert!(deserialized.completed_at.is_some());
        assert_eq!(deserialized.status, "completed");
    }

    #[test]
    fn task_record_with_parent_task() {
        let parent_id = Uuid::new_v4();
        let mut record = sample_task();
        record.parent_task_id = Some(parent_id);

        let json = serde_json::to_string(&record).expect("serialize");
        let deserialized: TaskRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.parent_task_id, Some(parent_id));
    }

    #[test]
    fn task_record_priority_range() {
        // Valid priorities: 0-4
        for priority in 0..=4 {
            let mut record = sample_task();
            record.priority = priority;
            let json = serde_json::to_string(&record).expect("serialize");
            let deserialized: TaskRecord = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(deserialized.priority, priority);
        }
    }

    // -------------------------------------------------------------------
    // MessageRecord tests (T039)
    // -------------------------------------------------------------------

    /// Helper to build a sample MessageRecord for tests.
    fn sample_message() -> MessageRecord {
        MessageRecord {
            id: Uuid::new_v4(),
            from_agent_id: Some(Uuid::new_v4()),
            to_agent_id: Some(Uuid::new_v4()),
            message_type: "request".to_string(),
            subject: Some("task.execute".to_string()),
            content: serde_json::json!({"action": "run"}),
            priority: 1,
            status: "pending".to_string(),
            correlation_id: Some(Uuid::new_v4()),
            parent_message_id: None,
            retry_count: 0,
            max_retries: 3,
            created_at: Utc::now(),
            sent_at: None,
            delivered_at: None,
            processed_at: None,
            expires_at: None,
            error_message: None,
            message_id: None,
        }
    }

    #[test]
    fn message_record_construction_and_serialization() {
        let record = sample_message();
        assert_eq!(record.message_type, "request");
        assert_eq!(record.status, "pending");
        assert_eq!(record.priority, 1);
        assert_eq!(record.retry_count, 0);
        assert_eq!(record.max_retries, 3);
        assert!(record.from_agent_id.is_some());
        assert!(record.to_agent_id.is_some());
        assert!(record.error_message.is_none());

        // Round-trip through JSON
        let json = serde_json::to_string(&record).expect("serialize MessageRecord");
        let deserialized: MessageRecord =
            serde_json::from_str(&json).expect("deserialize MessageRecord");
        assert_eq!(deserialized.id, record.id);
        assert_eq!(deserialized.message_type, record.message_type);
        assert_eq!(deserialized.content, record.content);
        assert_eq!(deserialized.correlation_id, record.correlation_id);
    }

    #[test]
    fn message_record_with_error() {
        let mut record = sample_message();
        record.status = "failed".to_string();
        record.error_message = Some("timeout after 30s".to_string());
        record.retry_count = 3;

        let json = serde_json::to_string(&record).expect("serialize");
        let deserialized: MessageRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.status, "failed");
        assert_eq!(
            deserialized.error_message,
            Some("timeout after 30s".to_string())
        );
        assert_eq!(deserialized.retry_count, 3);
    }

    #[test]
    fn message_record_with_timestamps() {
        let mut record = sample_message();
        record.sent_at = Some(Utc::now());
        record.delivered_at = Some(Utc::now());
        record.processed_at = Some(Utc::now());
        record.status = "processed".to_string();

        let json = serde_json::to_string(&record).expect("serialize");
        let deserialized: MessageRecord = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.sent_at.is_some());
        assert!(deserialized.delivered_at.is_some());
        assert!(deserialized.processed_at.is_some());
    }

    #[test]
    fn message_record_with_parent() {
        let parent_id = Uuid::new_v4();
        let mut record = sample_message();
        record.parent_message_id = Some(parent_id);

        let json = serde_json::to_string(&record).expect("serialize");
        let deserialized: MessageRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.parent_message_id, Some(parent_id));
    }

    #[test]
    fn message_record_priority_range() {
        // Valid priorities: 0-4
        for priority in 0..=4 {
            let mut record = sample_message();
            record.priority = priority;
            let json = serde_json::to_string(&record).expect("serialize");
            let deserialized: MessageRecord = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(deserialized.priority, priority);
        }
    }

    // -------------------------------------------------------------------
    // AuditEntry tests (T041)
    // -------------------------------------------------------------------

    fn sample_audit_entry() -> AuditEntry {
        AuditEntry {
            id: Uuid::new_v4(),
            event_type: "authentication".to_string(),
            agent_id: Some(Uuid::new_v4()),
            resource_type: Some("security".to_string()),
            resource_id: None,
            action: "login".to_string(),
            old_values: None,
            new_values: Some(serde_json::json!({"outcome": "success"})),
            metadata: serde_json::json!({"source_ip": "127.0.0.1"}),
            correlation_id: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn audit_entry_construction_and_serialization() {
        let entry = sample_audit_entry();
        assert_eq!(entry.event_type, "authentication");
        assert_eq!(entry.action, "login");
        assert!(entry.agent_id.is_some());
        assert!(entry.old_values.is_none());
        assert!(entry.new_values.is_some());

        let json = serde_json::to_string(&entry).expect("serialize AuditEntry");
        let deserialized: AuditEntry = serde_json::from_str(&json).expect("deserialize AuditEntry");
        assert_eq!(deserialized.id, entry.id);
        assert_eq!(deserialized.event_type, entry.event_type);
        assert_eq!(deserialized.action, entry.action);
        assert_eq!(deserialized.metadata, entry.metadata);
    }

    #[test]
    fn audit_entry_with_all_optional_fields() {
        let mut entry = sample_audit_entry();
        entry.resource_id = Some(Uuid::new_v4());
        entry.old_values = Some(serde_json::json!({"status": "active"}));
        entry.correlation_id = Some(Uuid::new_v4());

        let json = serde_json::to_string(&entry).expect("serialize");
        let deserialized: AuditEntry = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.resource_id.is_some());
        assert!(deserialized.old_values.is_some());
        assert!(deserialized.correlation_id.is_some());
    }

    #[test]
    fn audit_batch_insert_uses_single_unnest_statement() {
        let entries = vec![sample_audit_entry(), sample_audit_entry()];

        let batch = AuditBatchInsert::from_entries(&entries);
        let normalized_sql = AuditBatchInsert::SQL
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        assert!(normalized_sql.contains("FROM UNNEST("));
        assert_eq!(batch.ids.len(), entries.len());
        assert_eq!(batch.event_types.len(), entries.len());
        assert_eq!(batch.agent_ids.len(), entries.len());
        assert_eq!(batch.resource_types.len(), entries.len());
        assert_eq!(batch.resource_ids.len(), entries.len());
        assert_eq!(batch.actions.len(), entries.len());
        assert_eq!(batch.old_values.len(), entries.len());
        assert_eq!(batch.new_values.len(), entries.len());
        assert_eq!(batch.metadata.len(), entries.len());
        assert_eq!(batch.correlation_ids.len(), entries.len());
        assert_eq!(batch.created_ats.len(), entries.len());
        assert_eq!(batch.ids[0], entries[0].id);
        assert_eq!(batch.event_types[1], entries[1].event_type);
    }

    // -------------------------------------------------------------------
    // ConfigRecord tests (T042)
    // -------------------------------------------------------------------

    fn sample_config() -> ConfigRecord {
        ConfigRecord {
            id: Uuid::new_v4(),
            key: "persistence.flush_interval".to_string(),
            value: serde_json::json!(30),
            environment: "production".to_string(),
            agent_id: None,
            version: 1,
            description: Some("Flush interval in seconds".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn config_record_construction_and_serialization() {
        let config = sample_config();
        assert_eq!(config.key, "persistence.flush_interval");
        assert_eq!(config.environment, "production");
        assert_eq!(config.version, 1);
        assert!(config.agent_id.is_none());

        let json = serde_json::to_string(&config).expect("serialize ConfigRecord");
        let deserialized: ConfigRecord =
            serde_json::from_str(&json).expect("deserialize ConfigRecord");
        assert_eq!(deserialized.id, config.id);
        assert_eq!(deserialized.key, config.key);
        assert_eq!(deserialized.value, config.value);
        assert_eq!(deserialized.version, config.version);
    }

    #[test]
    fn config_record_with_agent_scope() {
        let mut config = sample_config();
        config.agent_id = Some(Uuid::new_v4());
        config.environment = "staging".to_string();

        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: ConfigRecord = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.agent_id.is_some());
        assert_eq!(deserialized.environment, "staging");
    }
}
