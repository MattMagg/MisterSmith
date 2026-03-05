//! Error conversions for the security crate.
//!
//! Re-exports [`SecurityError`] from `mister-smith-core` and provides
//! conversion helpers for external library errors.

pub use mister_smith_core::SecurityError;

/// Convert a `jsonwebtoken::errors::Error` into a [`SecurityError`].
///
/// This is a free function rather than a `From` impl because the orphan rule
/// prevents implementing a foreign trait (`From`) for two foreign types.
#[cfg(feature = "jwt")]
pub fn from_jwt_error(err: jsonwebtoken::errors::Error) -> SecurityError {
    use jsonwebtoken::errors::ErrorKind;
    match err.kind() {
        ErrorKind::ExpiredSignature => SecurityError::TokenExpired,
        ErrorKind::InvalidSignature => SecurityError::InvalidSignature,
        ErrorKind::InvalidToken => SecurityError::InvalidToken(err.to_string()),
        _ => SecurityError::InvalidToken(err.to_string()),
    }
}
