//! Error conversion helpers for the persistence layer.
//!
//! Re-exports `PersistenceError` from core and provides conversion functions
//! from external library errors (sqlx, async-nats KV) to the domain error type.
//! Free functions are used instead of `From` impls due to the orphan rule.

pub use mister_smith_core::PersistenceError;

/// Convert a sqlx error into a `PersistenceError`.
///
/// Maps sqlx-specific error variants to the appropriate persistence domain error.
#[cfg(feature = "sqlx")]
pub fn from_sqlx_error(err: sqlx::Error) -> PersistenceError {
    match err {
        sqlx::Error::RowNotFound => PersistenceError::NotFound("Row not found".to_string()),
        sqlx::Error::Database(ref db_err) => {
            let code = db_err.code().unwrap_or_default();
            // PostgreSQL error codes: 23505 = unique_violation
            if code == "23505" {
                PersistenceError::DuplicateKey(db_err.message().to_string())
            } else {
                PersistenceError::DatabaseFailed(err.to_string())
            }
        }
        sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed => {
            PersistenceError::ConnectionFailed(err.to_string())
        }
        sqlx::Error::Io(_) => PersistenceError::ConnectionFailed(err.to_string()),
        _ => PersistenceError::DatabaseFailed(err.to_string()),
    }
}

/// Stub for non-sqlx builds.
#[cfg(not(feature = "sqlx"))]
pub fn from_sqlx_error(_msg: String) -> PersistenceError {
    PersistenceError::DatabaseFailed("sqlx feature not enabled".to_string())
}

/// Convert an async-nats KV error into a `PersistenceError`.
///
/// Handles common KV operation failure modes: put, entry, update, delete, watch.
pub fn from_kv_error(err: impl std::fmt::Display) -> PersistenceError {
    let msg = err.to_string();
    if msg.contains("wrong last sequence") || msg.contains("revision") {
        PersistenceError::VersionConflict {
            key: String::new(),
            expected: 0,
            actual: 0,
        }
    } else if msg.contains("timeout") || msg.contains("connection") {
        PersistenceError::ConnectionFailed(msg)
    } else if msg.contains("not found") || msg.contains("no message found") {
        PersistenceError::NotFound(msg)
    } else {
        PersistenceError::DatabaseFailed(msg)
    }
}

/// Convert a KV update error with revision context into a `PersistenceError`.
pub fn from_kv_version_error(key: &str, expected: u64, _err: impl std::fmt::Display) -> PersistenceError {
    PersistenceError::VersionConflict {
        key: key.to_string(),
        expected,
        actual: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // from_kv_error tests
    // -----------------------------------------------------------------------

    #[test]
    fn kv_error_wrong_last_sequence_maps_to_version_conflict() {
        let err = from_kv_error("wrong last sequence: 5");
        assert!(matches!(err, PersistenceError::VersionConflict { .. }));
    }

    #[test]
    fn kv_error_revision_maps_to_version_conflict() {
        let err = from_kv_error("revision mismatch");
        assert!(matches!(err, PersistenceError::VersionConflict { .. }));
    }

    #[test]
    fn kv_error_timeout_maps_to_connection_failed() {
        let err = from_kv_error("operation timeout");
        assert!(matches!(err, PersistenceError::ConnectionFailed(_)));
    }

    #[test]
    fn kv_error_connection_maps_to_connection_failed() {
        let err = from_kv_error("connection refused");
        assert!(matches!(err, PersistenceError::ConnectionFailed(_)));
    }

    #[test]
    fn kv_error_not_found_maps_to_not_found() {
        let err = from_kv_error("key not found");
        assert!(matches!(err, PersistenceError::NotFound(_)));
    }

    #[test]
    fn kv_error_no_message_found_maps_to_not_found() {
        let err = from_kv_error("no message found");
        assert!(matches!(err, PersistenceError::NotFound(_)));
    }

    #[test]
    fn kv_error_unknown_maps_to_database_failed() {
        let err = from_kv_error("some unknown KV error");
        assert!(matches!(err, PersistenceError::DatabaseFailed(_)));
    }

    // -----------------------------------------------------------------------
    // from_kv_version_error tests
    // -----------------------------------------------------------------------

    #[test]
    fn kv_version_error_preserves_key_and_expected() {
        let err = from_kv_version_error("agent:state_key", 42, "wrong last sequence");
        match err {
            PersistenceError::VersionConflict {
                key,
                expected,
                actual,
            } => {
                assert_eq!(key, "agent:state_key");
                assert_eq!(expected, 42);
                assert_eq!(actual, 0);
            }
            other => panic!("Expected VersionConflict, got: {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // from_sqlx_error tests
    // -----------------------------------------------------------------------

    #[cfg(feature = "sqlx")]
    #[test]
    fn sqlx_row_not_found_maps_to_not_found() {
        let err = from_sqlx_error(sqlx::Error::RowNotFound);
        assert!(matches!(err, PersistenceError::NotFound(_)));
    }

    #[cfg(feature = "sqlx")]
    #[test]
    fn sqlx_pool_timed_out_maps_to_connection_failed() {
        let err = from_sqlx_error(sqlx::Error::PoolTimedOut);
        assert!(matches!(err, PersistenceError::ConnectionFailed(_)));
    }

    #[cfg(feature = "sqlx")]
    #[test]
    fn sqlx_pool_closed_maps_to_connection_failed() {
        let err = from_sqlx_error(sqlx::Error::PoolClosed);
        assert!(matches!(err, PersistenceError::ConnectionFailed(_)));
    }
}
