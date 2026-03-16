-- Phase 6: Performance indexes
-- B-tree and GIN indexes for query optimization

-- Agent Registry indexes
CREATE INDEX IF NOT EXISTS idx_agents_registry_status ON agents.registry (status);
CREATE INDEX IF NOT EXISTS idx_agents_registry_type ON agents.registry (agent_type);
CREATE INDEX IF NOT EXISTS idx_agents_registry_parent ON agents.registry (parent_agent_id);
CREATE INDEX IF NOT EXISTS idx_agents_registry_heartbeat ON agents.registry (last_heartbeat);

-- Agent State indexes
CREATE INDEX IF NOT EXISTS idx_agents_state_updated ON agents.state (updated_at);
CREATE INDEX IF NOT EXISTS idx_agents_state_value ON agents.state USING GIN (state_value);

-- Agent Checkpoint indexes
CREATE INDEX IF NOT EXISTS idx_agents_checkpoints_latest ON agents.checkpoints (agent_id, created_at DESC);

-- Task Record indexes
CREATE INDEX IF NOT EXISTS idx_tasks_agent_status ON tasks.records (agent_id, status);
CREATE INDEX IF NOT EXISTS idx_tasks_correlation ON tasks.records (correlation_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status_priority ON tasks.records (status, priority);
CREATE INDEX IF NOT EXISTS idx_tasks_created ON tasks.records (created_at);

-- Configuration indexes
CREATE INDEX IF NOT EXISTS idx_config_agent ON config.configurations (agent_id);
CREATE INDEX IF NOT EXISTS idx_config_environment ON config.configurations (environment);
