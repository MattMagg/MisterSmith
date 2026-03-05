-- Phase 6: Audit log schema with time partitioning
-- Persists events from Phase 5 AuditLogger ring buffer

CREATE TABLE IF NOT EXISTS audit_log (
    id              UUID NOT NULL DEFAULT gen_random_uuid(),
    event_type      TEXT NOT NULL,
    agent_id        UUID REFERENCES agents.registry(agent_id) ON DELETE SET NULL,
    resource_type   TEXT,
    resource_id     UUID,
    action          TEXT NOT NULL,
    old_values      JSONB,
    new_values      JSONB,
    metadata        JSONB NOT NULL DEFAULT '{}',
    correlation_id  UUID,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- Create audit log partitions: current month + 3 months ahead
SELECT create_monthly_partition('public', 'audit_log', CURRENT_DATE);
SELECT create_monthly_partition('public', 'audit_log', CURRENT_DATE + INTERVAL '1 month');
SELECT create_monthly_partition('public', 'audit_log', CURRENT_DATE + INTERVAL '2 months');
SELECT create_monthly_partition('public', 'audit_log', CURRENT_DATE + INTERVAL '3 months');

-- Audit log indexes
CREATE INDEX IF NOT EXISTS idx_audit_agent_created ON audit_log (agent_id, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_event_type ON audit_log (event_type);
CREATE INDEX IF NOT EXISTS idx_audit_correlation ON audit_log (correlation_id);
