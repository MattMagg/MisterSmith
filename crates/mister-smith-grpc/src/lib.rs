#![deny(missing_docs, unsafe_code)]

//! gRPC service layer for the Mister Smith multi-agent orchestration framework.
//!
//! This crate provides the gRPC transport layer implementing the service
//! definitions from the Phase 4 proto contracts:
//!
//! - **proto** — Protobuf message types matching `common.proto`, `agent_service.proto`,
//!   and `system_service.proto`. These are hand-defined prost `Message` structs that
//!   will be replaced by codegen types from `mister-smith-transport` once available.
//! - **config** — [`GrpcTransportConfig`] with bind address and max message size.
//! - **errors** — Bidirectional mapping between [`TransportError`] and [`tonic::Status`].
//! - **agent_service** — [`AgentServiceImpl`] with agent listing, task submission, and streaming.
//! - **system_service** — [`SystemServiceImpl`] with event streaming, config, and metrics.
//! - **health** — Standard `grpc.health.v1.Health` service via `tonic-health`.
//! - **server** — [`GrpcServer`] composing all services with graceful shutdown.

pub mod agent_service;
pub mod config;
pub mod errors;
pub mod health;
pub mod proto;
pub mod server;
pub mod system_service;

// Public re-exports for convenience.
pub use agent_service::AgentServiceImpl;
pub use config::GrpcTransportConfig;
pub use errors::TransportError;
pub use server::GrpcServer;
pub use system_service::SystemServiceImpl;
