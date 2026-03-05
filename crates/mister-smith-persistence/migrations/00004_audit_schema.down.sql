-- Rollback: 00004_audit_schema
-- DROP audit log indexes, table, and partitions

-- Audit log indexes
DROP INDEX IF EXISTS idx_audit_correlation;
DROP INDEX IF EXISTS idx_audit_event_type;
DROP INDEX IF EXISTS idx_audit_agent_created;

-- Drop audit_log table (CASCADE drops partitions)
DROP TABLE IF EXISTS audit_log CASCADE;
