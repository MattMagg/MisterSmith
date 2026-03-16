-- Phase 10: Durable conversation sessions and ordered session turns

DO $$ BEGIN
    CREATE TYPE session_status_type AS ENUM (
        'active', 'ended'
    );
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS tasks.sessions (
    session_id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    coordinator_agent_id       UUID NOT NULL,
    status                     session_status_type NOT NULL DEFAULT 'active',
    provider_kind              VARCHAR(100) NOT NULL,
    model_id                   VARCHAR(100) NOT NULL,
    active_workflow_id         UUID REFERENCES tasks.records(task_id) ON DELETE SET NULL,
    last_completed_workflow_id UUID REFERENCES tasks.records(task_id) ON DELETE SET NULL,
    turn_count                 INTEGER NOT NULL DEFAULT 0,
    retained_context           JSONB NOT NULL DEFAULT '{}',
    created_at                 TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at                 TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at                   TIMESTAMPTZ,
    CONSTRAINT valid_turn_count CHECK (turn_count >= 0),
    CONSTRAINT valid_session_end_state CHECK (
        (status = 'active' AND ended_at IS NULL)
        OR (status = 'ended' AND ended_at IS NOT NULL)
    )
);

CREATE TABLE IF NOT EXISTS tasks.session_turns (
    turn_id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id      UUID NOT NULL REFERENCES tasks.sessions(session_id) ON DELETE CASCADE,
    turn_index      INTEGER NOT NULL,
    workflow_id     UUID NOT NULL UNIQUE REFERENCES tasks.records(task_id) ON DELETE RESTRICT,
    user_message    TEXT NOT NULL,
    result_summary  JSONB,
    status          task_status_type NOT NULL DEFAULT 'queued',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at    TIMESTAMPTZ,
    CONSTRAINT valid_turn_index CHECK (turn_index > 0),
    CONSTRAINT unique_session_turn_index UNIQUE (session_id, turn_index)
);

CREATE INDEX IF NOT EXISTS idx_sessions_active_workflow
    ON tasks.sessions (active_workflow_id)
    WHERE active_workflow_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_sessions_status_updated
    ON tasks.sessions (status, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_session_turns_session_created
    ON tasks.session_turns (session_id, created_at ASC);

CREATE INDEX IF NOT EXISTS idx_session_turns_workflow_status
    ON tasks.session_turns (workflow_id, status);
