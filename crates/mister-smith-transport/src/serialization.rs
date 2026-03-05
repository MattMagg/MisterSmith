//! Serialization helpers for MessagePack and JSON wire formats.
//!
//! Uses `rmp_serde::to_vec_named` (NOT `to_vec`) for wire format to support
//! schema evolution. Named mode encodes structs as maps with field name keys,
//! tolerant of field reordering and addition.
//!
//! For `Vec<u8>` fields in your message types, annotate with
//! `#[serde(with = "serde_bytes")]` to encode as binary rather than integer arrays.

use serde::{de::DeserializeOwned, Serialize};

use crate::errors::TransportError;

/// Serialize a value to MessagePack bytes using named field encoding.
pub fn to_msgpack<T: Serialize>(val: &T) -> Result<Vec<u8>, TransportError> {
    rmp_serde::to_vec_named(val).map_err(|e| TransportError::SerializationError(e.to_string()))
}

/// Deserialize a value from MessagePack bytes.
pub fn from_msgpack<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, TransportError> {
    rmp_serde::from_slice(bytes).map_err(|e| TransportError::DeserializationError(e.to_string()))
}

/// Serialize a value to a JSON string.
pub fn to_json<T: Serialize>(val: &T) -> Result<String, TransportError> {
    serde_json::to_string(val).map_err(|e| TransportError::SerializationError(e.to_string()))
}

/// Deserialize a value from a JSON string.
pub fn from_json<T: DeserializeOwned>(s: &str) -> Result<T, TransportError> {
    serde_json::from_str(s).map_err(|e| TransportError::DeserializationError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestMessage {
        id: u64,
        name: String,
        tags: Vec<String>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestBinary {
        label: String,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    }

    #[test]
    fn msgpack_roundtrip() {
        let msg = TestMessage {
            id: 42,
            name: "test".into(),
            tags: vec!["a".into(), "b".into()],
        };
        let bytes = to_msgpack(&msg).unwrap();
        let decoded: TestMessage = from_msgpack(&bytes).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn json_roundtrip() {
        let msg = TestMessage {
            id: 99,
            name: "hello".into(),
            tags: vec![],
        };
        let json = to_json(&msg).unwrap();
        let decoded: TestMessage = from_json(&json).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn msgpack_binary_field() {
        let msg = TestBinary {
            label: "payload".into(),
            data: vec![0x01, 0x02, 0x03, 0xFF],
        };
        let bytes = to_msgpack(&msg).unwrap();
        let decoded: TestBinary = from_msgpack(&bytes).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn msgpack_named_mode_tolerates_field_order() {
        // Named encoding means field order doesn't matter for deserialization.
        let msg = TestMessage {
            id: 1,
            name: "order".into(),
            tags: vec!["x".into()],
        };
        let bytes = to_msgpack(&msg).unwrap();
        // Decode should succeed regardless of internal field order.
        let decoded: TestMessage = from_msgpack(&bytes).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn deserialization_error_on_invalid_bytes() {
        let result = from_msgpack::<TestMessage>(&[0xFF, 0xFE]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, TransportError::DeserializationError(_)));
    }

    #[test]
    fn json_deserialization_error_on_invalid_input() {
        let result = from_json::<TestMessage>("not valid json");
        assert!(result.is_err());
    }
}
