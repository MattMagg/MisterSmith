-- Phase 6: Initial schema — schemas, domain types, core tables
-- Mister Smith Persistence Layer

-- Enable UUID generation
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Schemas
CREATE SCHEMA IF NOT EXISTS agents;
CREATE SCHEMA IF NOT EXISTS tasks;
CREATE SCHEMA IF NOT EXISTS messages;

-- Domain types
DO $$ BEGIN
    CREATE TYPE agent_status_type AS ENUM (
        'initializing', 'active', 'idle', 'suspended', 'terminated', 'error'
    );
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE task_status_type AS ENUM (
        'pending', 'queued', 'running', 'paused', 'completed', 'failed', 'cancelled'
    );
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

-- Agent Registry
CREATE TABLE IF NOT EXISTS agents.registry (
    agent_id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_type      VARCHAR(50) NOT NULL,
    agent_name      VARCHAR(255) NOT NULL,
    status          agent_status_type NOT NULL DEFAULT 'initializing',
    capabilities    JSONB NOT NULL DEFAULT '{}',
    configuration   JSONB NOT NULL DEFAULT '{}',
    metadata        JSONB NOT NULL DEFAULT '{}',
    parent_agent_id UUID REFERENCES agents.registry(agent_id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_heartbeat  TIMESTAMPTZ,
    CONSTRAINT no_self_parent CHECK (agent_id != parent_agent_id),
    CONSTRAINT valid_agent_name CHECK (LENGTH(agent_name) > 0),
    CONSTRAINT unique_agent_name UNIQUE (agent_name)
);

-- Agent State (partitioned by hash on agent_id)
CREATE TABLE IF NOT EXISTS agents.state (
    agent_id    UUID NOT NULL REFERENCES agents.registry(agent_id) ON DELETE CASCADE,
    state_key   VARCHAR(255) NOT NULL,
    state_value JSONB NOT NULL,
    version     BIGINT NOT NULL DEFAULT 1,
    checksum    VARCHAR(64),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ,
    PRIMARY KEY (agent_id, state_key),
    CONSTRAINT positive_version CHECK (version > 0),
    CONSTRAINT valid_expiry CHECK (expires_at IS NULL OR expires_at > created_at)
) PARTITION BY HASH (agent_id);

-- Create 8 hash partitions for agent state
CREATE TABLE IF NOT EXISTS agents.state_p0 PARTITION OF agents.state FOR VALUES WITH (MODULUS 8, REMAINDER 0);
CREATE TABLE IF NOT EXISTS agents.state_p1 PARTITION OF agents.state FOR VALUES WITH (MODULUS 8, REMAINDER 1);
CREATE TABLE IF NOT EXISTS agents.state_p2 PARTITION OF agents.state FOR VALUES WITH (MODULUS 8, REMAINDER 2);
CREATE TABLE IF NOT EXISTS agents.state_p3 PARTITION OF agents.state FOR VALUES WITH (MODULUS 8, REMAINDER 3);
CREATE TABLE IF NOT EXISTS agents.state_p4 PARTITION OF agents.state FOR VALUES WITH (MODULUS 8, REMAINDER 4);
CREATE TABLE IF NOT EXISTS agents.state_p5 PARTITION OF agents.state FOR VALUES WITH (MODULUS 8, REMAINDER 5);
CREATE TABLE IF NOT EXISTS agents.state_p6 PARTITION OF agents.state FOR VALUES WITH (MODULUS 8, REMAINDER 6);
CREATE TABLE IF NOT EXISTS agents.state_p7 PARTITION OF agents.state FOR VALUES WITH (MODULUS 8, REMAINDER 7);

-- Agent Checkpoints
CREATE TABLE IF NOT EXISTS agents.checkpoints (
    agent_id        UUID NOT NULL REFERENCES agents.registry(agent_id) ON DELETE CASCADE,
    checkpoint_id   UUID NOT NULL DEFAULT gen_random_uuid(),
    state_snapshot  JSONB NOT NULL,
    kv_revision     BIGINT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (agent_id, checkpoint_id)
);

-- Task Records
CREATE TABLE IF NOT EXISTS tasks.records (
    task_id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_type       VARCHAR(50) NOT NULL,
    agent_id        UUID REFERENCES agents.registry(agent_id) ON DELETE SET NULL,
    payload         JSONB NOT NULL,
    result          JSONB,
    metadata        JSONB NOT NULL DEFAULT '{}',
    status          task_status_type NOT NULL DEFAULT 'pending',
    priority        INTEGER NOT NULL DEFAULT 2,
    correlation_id  UUID,
    parent_task_id  UUID REFERENCES tasks.records(task_id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    expires_at      TIMESTAMPTZ,
    CONSTRAINT valid_priority CHECK (priority >= 0 AND priority <= 4)
);

-- Message Records (partitioned by range on created_at)
CREATE TABLE IF NOT EXISTS messages.records (
    id                UUID NOT NULL DEFAULT gen_random_uuid(),
    from_agent_id     UUID,
    to_agent_id       UUID,
    message_type      VARCHAR(50) NOT NULL,
    subject           TEXT,
    content           JSONB NOT NULL,
    priority          INTEGER NOT NULL DEFAULT 2,
    status            VARCHAR(20) NOT NULL DEFAULT 'pending',
    correlation_id    UUID,
    parent_message_id UUID,
    retry_count       INTEGER NOT NULL DEFAULT 0,
    max_retries       INTEGER NOT NULL DEFAULT 3,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sent_at           TIMESTAMPTZ,
    delivered_at      TIMESTAMPTZ,
    processed_at      TIMESTAMPTZ,
    expires_at        TIMESTAMPTZ,
    error_message     TEXT,
    PRIMARY KEY (id, created_at),
    CONSTRAINT valid_msg_priority CHECK (priority >= 0 AND priority <= 4),
    CONSTRAINT valid_msg_status CHECK (status IN ('pending', 'sent', 'delivered', 'processed', 'failed', 'expired', 'cancelled'))
) PARTITION BY RANGE (created_at);

-- Configuration schema
CREATE SCHEMA IF NOT EXISTS config;

-- Configurations
CREATE TABLE IF NOT EXISTS config.configurations (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key         TEXT NOT NULL,
    value       JSONB NOT NULL,
    environment VARCHAR(20) NOT NULL DEFAULT 'production',
    agent_id    UUID REFERENCES agents.registry(agent_id) ON DELETE CASCADE,
    version     INTEGER NOT NULL DEFAULT 1,
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_environment CHECK (environment IN ('development', 'staging', 'production', 'testing'))
);

-- Partial unique indexes for NULL-safe uniqueness on configurations
CREATE UNIQUE INDEX IF NOT EXISTS idx_config_key_env_agent
    ON config.configurations (key, environment, agent_id)
    WHERE agent_id IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_config_key_env_global
    ON config.configurations (key, environment)
    WHERE agent_id IS NULL;
