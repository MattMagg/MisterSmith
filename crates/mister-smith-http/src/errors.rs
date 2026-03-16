//! HTTP error types with Axum `IntoResponse` integration.
//!
//! [`HttpError`] variants map to HTTP status codes and produce consistent
//! JSON error responses with request ID tracking.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use thiserror::Error;
use uuid::Uuid;

/// HTTP transport error type.
#[derive(Debug, Error)]
pub enum HttpError {
    /// Resource not found (404).
    #[error("Not found: {0}")]
    NotFound(String),
    /// Invalid request (400).
    #[error("Bad request: {0}")]
    BadRequest(String),
    /// Conflict with the current resource state (409).
    #[error("{message}")]
    Conflict {
        /// Stable error code surfaced to API clients.
        code: String,
        /// Human-readable error message.
        message: String,
        /// Additional context fields to flatten into the JSON response body.
        context: BTreeMap<String, Value>,
    },
    /// Rate limit exceeded (429).
    #[error("Rate limit exceeded")]
    RateLimited,
    /// Internal server error (500).
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// JSON error response body.
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
    request_id: String,
    #[serde(flatten)]
    context: BTreeMap<String, Value>,
}

impl HttpError {
    /// Returns the HTTP status code for this error.
    fn status_code(&self) -> StatusCode {
        match self {
            HttpError::NotFound(_) => StatusCode::NOT_FOUND,
            HttpError::BadRequest(_) => StatusCode::BAD_REQUEST,
            HttpError::Conflict { .. } => StatusCode::CONFLICT,
            HttpError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            HttpError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Returns the error code string for the JSON response.
    fn error_code(&self) -> String {
        match self {
            HttpError::NotFound(_) => "not_found".to_string(),
            HttpError::BadRequest(_) => "bad_request".to_string(),
            HttpError::Conflict { code, .. } => code.clone(),
            HttpError::RateLimited => "rate_limited".to_string(),
            HttpError::InternalError(_) => "internal_error".to_string(),
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let message = self.to_string();
        let context = match &self {
            HttpError::Conflict { context, .. } => context.clone(),
            _ => BTreeMap::new(),
        };
        let body = ErrorResponse {
            error: self.error_code(),
            message,
            request_id: Uuid::new_v4().to_string(),
            context,
        };
        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_status_code() {
        let err = HttpError::NotFound("agent abc".to_string());
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(err.error_code(), "not_found");
    }

    #[test]
    fn bad_request_status_code() {
        let err = HttpError::BadRequest("missing field".to_string());
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(err.error_code(), "bad_request");
    }

    #[test]
    fn conflict_status_code() {
        let err = HttpError::Conflict {
            code: "session_busy".to_string(),
            message: "session is busy".to_string(),
            context: BTreeMap::new(),
        };
        assert_eq!(err.status_code(), StatusCode::CONFLICT);
        assert_eq!(err.error_code(), "session_busy");
    }

    #[test]
    fn rate_limited_status_code() {
        let err = HttpError::RateLimited;
        assert_eq!(err.status_code(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(err.error_code(), "rate_limited");
    }

    #[test]
    fn internal_error_status_code() {
        let err = HttpError::InternalError("unexpected".to_string());
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.error_code(), "internal_error");
    }

    #[test]
    fn error_response_json_format() {
        let body = ErrorResponse {
            error: "not_found".to_string(),
            message: "Not found: agent abc".to_string(),
            request_id: "test-id".to_string(),
            context: BTreeMap::new(),
        };
        let json = serde_json::to_value(&body).unwrap();
        assert_eq!(json["error"], "not_found");
        assert_eq!(json["message"], "Not found: agent abc");
        assert_eq!(json["request_id"], "test-id");
    }

    #[test]
    fn error_display() {
        let err = HttpError::NotFound("agent xyz".to_string());
        assert_eq!(err.to_string(), "Not found: agent xyz");
    }
}
