-- Down-migration for 00005: Remove message idempotency support
DROP INDEX IF EXISTS messages.idx_messages_correlation;
DROP INDEX IF EXISTS messages.idx_messages_dedup;
ALTER TABLE messages.records DROP COLUMN IF EXISTS message_id;
