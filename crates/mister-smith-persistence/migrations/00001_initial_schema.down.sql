-- Rollback: 00001_initial_schema
-- DROP tables, types, schemas in reverse dependency order

DROP TABLE IF EXISTS config.configurations;
DROP SCHEMA IF EXISTS config CASCADE;
DROP TABLE IF EXISTS messages.records;
DROP TABLE IF EXISTS tasks.records;
DROP TABLE IF EXISTS agents.checkpoints;
DROP TABLE IF EXISTS agents.state_p0;
DROP TABLE IF EXISTS agents.state_p1;
DROP TABLE IF EXISTS agents.state_p2;
DROP TABLE IF EXISTS agents.state_p3;
DROP TABLE IF EXISTS agents.state_p4;
DROP TABLE IF EXISTS agents.state_p5;
DROP TABLE IF EXISTS agents.state_p6;
DROP TABLE IF EXISTS agents.state_p7;
DROP TABLE IF EXISTS agents.state;
DROP TABLE IF EXISTS agents.registry;

DROP TYPE IF EXISTS task_status_type;
DROP TYPE IF EXISTS agent_status_type;

DROP SCHEMA IF EXISTS messages CASCADE;
DROP SCHEMA IF EXISTS tasks CASCADE;
DROP SCHEMA IF EXISTS agents CASCADE;

DROP EXTENSION IF EXISTS pgcrypto;
