//! DataRouter — type-based storage layer selection.
//!
//! Maps each [`DataType`] to its optimal [`StorageLayer`] based on access
//! patterns, durability requirements, and TTL from the data model.

use std::time::Duration;

/// Categories of data managed by the persistence layer.
///
/// Each variant maps to a specific storage strategy via [`DataRouter`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataType {
    /// User/agent session state — ephemeral, fast access.
    SessionData,
    /// Hot agent runtime state — fast access with durable backup.
    AgentState,
    /// Agent registry records — durable, authoritative.
    AgentRegistry,
    /// Task lifecycle records — durable, queryable.
    TaskRecord,
    /// Inter-agent message history — durable, time-partitioned.
    MessageRecord,
    /// Immutable audit trail — durable, append-only.
    AuditLog,
    /// Expensive query result caching — ephemeral.
    QueryCache,
    /// System and per-agent configuration — durable.
    Configuration,
    /// Point-in-time state snapshots — durable.
    Checkpoint,
}

/// Where data is stored and how reads/writes are routed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageLayer {
    /// Only stored in JetStream KV. Lost when TTL expires.
    KvOnly,
    /// Only stored in PostgreSQL.
    SqlOnly,
    /// KV is the primary store; SQL is the durable backup.
    /// Writes go to KV first, then async-flushed to SQL.
    /// Reads try KV first, fall back to SQL on miss.
    KvPrimary,
}

/// Routes data types to their optimal storage layer.
///
/// Uses a zero-cost static dispatch — no runtime configuration needed.
/// The routing rules are derived from the data model's storage patterns.
pub struct DataRouter;

impl DataRouter {
    /// Determine which storage layer handles a given data type.
    pub fn select_storage(data_type: DataType) -> StorageLayer {
        match data_type {
            DataType::SessionData => StorageLayer::KvOnly,
            DataType::AgentState => StorageLayer::KvPrimary,
            DataType::AgentRegistry => StorageLayer::SqlOnly,
            DataType::TaskRecord => StorageLayer::SqlOnly,
            DataType::MessageRecord => StorageLayer::SqlOnly,
            DataType::AuditLog => StorageLayer::SqlOnly,
            DataType::QueryCache => StorageLayer::KvOnly,
            DataType::Configuration => StorageLayer::SqlOnly,
            DataType::Checkpoint => StorageLayer::SqlOnly,
        }
    }

    /// Get the TTL for a data type in the KV layer.
    ///
    /// Returns `None` for data types that are SQL-only (no KV involvement).
    /// TTL values come from the KV bucket configuration in the data model.
    pub fn get_ttl(data_type: DataType) -> Option<Duration> {
        match data_type {
            DataType::SessionData => Some(Duration::from_secs(3600)), // 60 min
            DataType::AgentState => Some(Duration::from_secs(1800)),  // 30 min
            DataType::QueryCache => Some(Duration::from_secs(300)),   // 5 min
            // SQL-only types have no KV TTL
            DataType::AgentRegistry
            | DataType::TaskRecord
            | DataType::MessageRecord
            | DataType::AuditLog
            | DataType::Configuration
            | DataType::Checkpoint => None,
        }
    }

    /// Check if a data type uses the KV layer at all.
    pub fn uses_kv(data_type: DataType) -> bool {
        matches!(
            Self::select_storage(data_type),
            StorageLayer::KvOnly | StorageLayer::KvPrimary
        )
    }

    /// Check if a data type uses the SQL layer at all.
    pub fn uses_sql(data_type: DataType) -> bool {
        matches!(
            Self::select_storage(data_type),
            StorageLayer::SqlOnly | StorageLayer::KvPrimary
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_data_routes_to_kv_only() {
        assert_eq!(
            DataRouter::select_storage(DataType::SessionData),
            StorageLayer::KvOnly
        );
    }

    #[test]
    fn agent_state_routes_to_kv_primary() {
        assert_eq!(
            DataRouter::select_storage(DataType::AgentState),
            StorageLayer::KvPrimary
        );
    }

    #[test]
    fn agent_registry_routes_to_sql_only() {
        assert_eq!(
            DataRouter::select_storage(DataType::AgentRegistry),
            StorageLayer::SqlOnly
        );
    }

    #[test]
    fn task_record_routes_to_sql_only() {
        assert_eq!(
            DataRouter::select_storage(DataType::TaskRecord),
            StorageLayer::SqlOnly
        );
    }

    #[test]
    fn query_cache_routes_to_kv_only() {
        assert_eq!(
            DataRouter::select_storage(DataType::QueryCache),
            StorageLayer::KvOnly
        );
    }

    #[test]
    fn kv_types_have_ttl() {
        assert!(DataRouter::get_ttl(DataType::SessionData).is_some());
        assert!(DataRouter::get_ttl(DataType::AgentState).is_some());
        assert!(DataRouter::get_ttl(DataType::QueryCache).is_some());
    }

    #[test]
    fn sql_only_types_have_no_ttl() {
        assert!(DataRouter::get_ttl(DataType::AgentRegistry).is_none());
        assert!(DataRouter::get_ttl(DataType::TaskRecord).is_none());
        assert!(DataRouter::get_ttl(DataType::MessageRecord).is_none());
        assert!(DataRouter::get_ttl(DataType::AuditLog).is_none());
        assert!(DataRouter::get_ttl(DataType::Configuration).is_none());
        assert!(DataRouter::get_ttl(DataType::Checkpoint).is_none());
    }

    #[test]
    fn ttl_values_match_data_model() {
        assert_eq!(
            DataRouter::get_ttl(DataType::SessionData),
            Some(Duration::from_secs(3600))
        );
        assert_eq!(
            DataRouter::get_ttl(DataType::AgentState),
            Some(Duration::from_secs(1800))
        );
        assert_eq!(
            DataRouter::get_ttl(DataType::QueryCache),
            Some(Duration::from_secs(300))
        );
    }

    #[test]
    fn uses_kv_correct() {
        assert!(DataRouter::uses_kv(DataType::SessionData));
        assert!(DataRouter::uses_kv(DataType::AgentState));
        assert!(DataRouter::uses_kv(DataType::QueryCache));
        assert!(!DataRouter::uses_kv(DataType::AgentRegistry));
        assert!(!DataRouter::uses_kv(DataType::TaskRecord));
    }

    #[test]
    fn uses_sql_correct() {
        assert!(DataRouter::uses_sql(DataType::AgentRegistry));
        assert!(DataRouter::uses_sql(DataType::TaskRecord));
        assert!(DataRouter::uses_sql(DataType::AgentState)); // KvPrimary uses both
        assert!(!DataRouter::uses_sql(DataType::SessionData)); // KvOnly
        assert!(!DataRouter::uses_sql(DataType::QueryCache)); // KvOnly
    }
}
