//! Repository abstractions for entity persistence.
//!
//! The [`Repository`] trait defines a generic CRUD interface for entity
//! persistence. Concrete implementations route operations to the appropriate
//! storage backend (KV, SQL, or both) based on the data type.

pub mod agent;
pub mod audit;
pub mod message;
pub mod session;
pub mod task;

use async_trait::async_trait;
use uuid::Uuid;

use mister_smith_core::PersistenceError;

/// Generic repository for entity persistence.
///
/// Implementations route to KV, SQL, or both based on the data type.
/// All methods are async and return typed `PersistenceError` on failure.
#[async_trait]
pub trait Repository<T: Send + Sync>: Send + Sync {
    /// Persist an entity. Returns the saved entity (may include generated fields).
    async fn save(&self, entity: &T) -> Result<T, PersistenceError>;

    /// Find an entity by its primary identifier.
    async fn find(&self, id: &Uuid) -> Result<Option<T>, PersistenceError>;

    /// Update an existing entity. Returns the updated entity.
    async fn update(&self, entity: &T) -> Result<T, PersistenceError>;

    /// Delete an entity by its primary identifier.
    ///
    /// Returns `true` if the entity existed and was deleted, `false` if not found.
    async fn delete(&self, id: &Uuid) -> Result<bool, PersistenceError>;
}
