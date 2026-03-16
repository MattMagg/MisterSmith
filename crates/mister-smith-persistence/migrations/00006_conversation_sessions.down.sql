DROP INDEX IF EXISTS tasks.idx_session_turns_workflow_status;
DROP INDEX IF EXISTS tasks.idx_session_turns_session_created;
DROP INDEX IF EXISTS tasks.idx_sessions_status_updated;
DROP INDEX IF EXISTS tasks.idx_sessions_active_workflow;

DROP TABLE IF EXISTS tasks.session_turns;
DROP TABLE IF EXISTS tasks.sessions;

DROP TYPE IF EXISTS session_status_type;
