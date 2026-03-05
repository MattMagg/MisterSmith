//! HTTP error types with Axum `IntoResponse` integration.
//!
//! [`HttpError`] variants map to HTTP status codes and produce consistent
//! JSON error responses with request ID tracking.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
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
}

impl HttpError {
    /// Returns the HTTP status code for this error.
    fn status_code(&self) -> StatusCode {
        match self {
            HttpError::NotFound(_) => StatusCode::NOT_FOUND,
            HttpError::BadRequest(_) => StatusCode::BAD_REQUEST,
            HttpError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            HttpError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Returns the error code string for the JSON response.
    fn error_code(&self) -> &'static str {
        match self {
            HttpError::NotFound(_) => "not_found",
            HttpError::BadRequest(_) => "bad_request",
            HttpError::RateLimited => "rate_limited",
            HttpError::InternalError(_) => "internal_error",
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = ErrorResponse {
            error: self.error_code().to_string(),
            message: self.to_string(),
            request_id: Uuid::new_v4().to_string(),
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
