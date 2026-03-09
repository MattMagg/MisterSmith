#![deny(missing_docs, unsafe_code)]

//! Security layer for the Mister Smith framework.
//!
//! Provides JWT authentication, RBAC authorization, TLS/mTLS certificate
//! management, transport security middleware, and audit logging.
//!
//! All subsystems are independently toggleable via feature flags and runtime
//! configuration. A master `enabled` switch disables all subsystems when off.
//!
//! # Feature Flags
//!
//! - **`jwt`** — JWT token generation, validation, refresh, and revocation
//! - **`rbac`** — Role-based access control with hierarchical roles
//! - **`tls`** — TLS 1.3 / mTLS certificate management with rustls
//! - **`audit`** — Tamper-evident audit logging with hash chaining
//!
//! State validation (`state_validator` module) is always compiled and not
//! feature-gated — it is required at every persistence-to-agent boundary.

pub mod config;
mod error;
pub mod state_validator;

#[cfg(feature = "jwt")]
pub mod jwt;

#[cfg(feature = "rbac")]
pub mod rbac;

#[cfg(feature = "tls")]
pub mod tls;

pub mod middleware;

#[cfg(feature = "audit")]
pub mod audit;

// Re-export SecurityError from core for convenience.
pub use mister_smith_core::SecurityError;

// Re-export config types.
pub use config::{AuditConfig, JwtConfig, RbacConfig, TlsConfig};
pub use state_validator::{
    JsonSchemaStateValidator, StateValidator, TaintLabel, ValidatedState, ValidationError,
};
