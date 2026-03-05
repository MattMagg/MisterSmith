-- Rollback: 00002_indexes
-- DROP all performance indexes

-- Configuration indexes
DROP INDEX IF EXISTS idx_config_environment;
DROP INDEX IF EXISTS idx_config_agent;

-- Task Record indexes
DROP INDEX IF EXISTS idx_tasks_created;
DROP INDEX IF EXISTS idx_tasks_status_priority;
DROP INDEX IF EXISTS idx_tasks_correlation;
DROP INDEX IF EXISTS idx_tasks_agent_status;

-- Agent Checkpoint indexes
DROP INDEX IF EXISTS idx_agents_checkpoints_latest;

-- Agent State indexes
DROP INDEX IF EXISTS idx_agents_state_value;
DROP INDEX IF EXISTS idx_agents_state_updated;

-- Agent Registry indexes
DROP INDEX IF EXISTS idx_agents_registry_heartbeat;
DROP INDEX IF EXISTS idx_agents_registry_parent;
DROP INDEX IF EXISTS idx_agents_registry_type;
DROP INDEX IF EXISTS idx_agents_registry_status;
