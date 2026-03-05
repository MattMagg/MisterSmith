//! HTTP API and WebSocket transport for the Mister Smith multi-agent orchestration framework.
//!
//! This crate provides:
//!
//! - **config** — [`HttpTransportConfig`] with bind address, WebSocket, and rate limiting settings.
//! - **errors** — [`HttpError`] with Axum `IntoResponse` producing consistent JSON error responses.
//! - **middleware** — Request ID tracking, per-IP rate limiting, and security hooks.
//! - **handlers** — REST endpoint handlers for health, agents, tasks, and config.
//! - **routes** — Route composition under `/api/v1`.
//! - **websocket** — WebSocket event streaming with filtering and keepalive.
//! - **server** — Server lifecycle with [`AppState`], graceful shutdown, and router composition.

pub mod config;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod server;
pub mod websocket;

// Re-export key types at crate root for convenience.
pub use config::HttpTransportConfig;
pub use errors::HttpError;
pub use server::{start, AppState};
pub use websocket::WsEvent;
