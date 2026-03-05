-- Rollback: 00003_partitions
-- DROP message partition indexes, partitions, and helper functions

-- Message partition indexes (on the parent — inherited by partitions)
DROP INDEX IF EXISTS idx_messages_status;
DROP INDEX IF EXISTS idx_messages_correlation;
DROP INDEX IF EXISTS idx_messages_to_agent;
DROP INDEX IF EXISTS idx_messages_from_agent;

-- Drop helper functions
DROP FUNCTION IF EXISTS check_partition_coverage(TEXT, TEXT);
DROP FUNCTION IF EXISTS create_monthly_partition(TEXT, TEXT, DATE);
