-- Phase 6: Time-based partitioning for high-volume tables
-- Creates monthly partitions for messages and audit_log

-- Helper function: create monthly partition for a given table
CREATE OR REPLACE FUNCTION create_monthly_partition(
    schema_name TEXT,
    table_name TEXT,
    partition_date DATE
) RETURNS TEXT AS $$
DECLARE
    partition_name TEXT;
    start_date DATE;
    end_date DATE;
BEGIN
    start_date := DATE_TRUNC('month', partition_date)::DATE;
    end_date := (start_date + INTERVAL '1 month')::DATE;
    partition_name := schema_name || '.' || table_name || '_y' ||
                      TO_CHAR(start_date, 'YYYY') || 'm' ||
                      TO_CHAR(start_date, 'MM');

    EXECUTE FORMAT(
        'CREATE TABLE IF NOT EXISTS %I.%I PARTITION OF %I.%I FOR VALUES FROM (%L) TO (%L)',
        schema_name,
        table_name || '_y' || TO_CHAR(start_date, 'YYYY') || 'm' || TO_CHAR(start_date, 'MM'),
        schema_name,
        table_name,
        start_date,
        end_date
    );

    RETURN partition_name;
END;
$$ LANGUAGE plpgsql;

-- Helper function: check partition coverage for a table
CREATE OR REPLACE FUNCTION check_partition_coverage(
    schema_name TEXT,
    table_name TEXT
) RETURNS TABLE(partition_name TEXT, range_start TEXT, range_end TEXT) AS $$
BEGIN
    RETURN QUERY
    SELECT
        c.relname::TEXT AS partition_name,
        pg_get_expr(c.relpartbound, c.oid)::TEXT AS range_start,
        ''::TEXT AS range_end
    FROM pg_class c
    JOIN pg_inherits i ON c.oid = i.inhrelid
    JOIN pg_class parent ON i.inhparent = parent.oid
    JOIN pg_namespace n ON parent.relnamespace = n.oid
    WHERE n.nspname = schema_name
      AND parent.relname = table_name
    ORDER BY c.relname;
END;
$$ LANGUAGE plpgsql;

-- Create message partitions: current month + 3 months ahead
SELECT create_monthly_partition('messages', 'records', CURRENT_DATE);
SELECT create_monthly_partition('messages', 'records', (CURRENT_DATE + INTERVAL '1 month')::DATE);
SELECT create_monthly_partition('messages', 'records', (CURRENT_DATE + INTERVAL '2 months')::DATE);
SELECT create_monthly_partition('messages', 'records', (CURRENT_DATE + INTERVAL '3 months')::DATE);

-- Message partition indexes (must be created on each partition)
-- These will be automatically created on new partitions via partition-level index inheritance
CREATE INDEX IF NOT EXISTS idx_messages_from_agent ON messages.records (from_agent_id, created_at);
CREATE INDEX IF NOT EXISTS idx_messages_to_agent ON messages.records (to_agent_id, created_at);
CREATE INDEX IF NOT EXISTS idx_messages_correlation ON messages.records (correlation_id);
CREATE INDEX IF NOT EXISTS idx_messages_status ON messages.records (status, created_at);
