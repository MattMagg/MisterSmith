-- Migration 00005: Message-level idempotency
--
-- Adds a message_id column (from MessageEnvelope.message_id) to the messages
-- table with a unique index for deduplication. This enables idempotent message
-- processing under NATS JetStream at-least-once delivery.

-- Add message_id column (nullable for backward compatibility with existing rows)
ALTER TABLE messages.records ADD COLUMN IF NOT EXISTS message_id UUID;

-- Unique index for deduplication. Uses NULLS NOT DISTINCT to prevent multiple
-- NULL entries (though in practice all new inserts should set message_id).
-- Partial index: only enforces uniqueness where message_id IS NOT NULL.
CREATE UNIQUE INDEX IF NOT EXISTS idx_messages_dedup
    ON messages.records (message_id)
    WHERE message_id IS NOT NULL;

-- Index on correlation_id for workflow-level lookups (complements existing
-- tasks.records correlation_id index).
CREATE INDEX IF NOT EXISTS idx_messages_correlation
    ON messages.records (correlation_id)
    WHERE correlation_id IS NOT NULL;
