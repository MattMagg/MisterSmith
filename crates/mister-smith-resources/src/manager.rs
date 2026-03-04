//! ResourceManager for heterogeneous pool management.
//!
//! [`ResourceManager`] stores named pools of arbitrary types behind
//! `Arc<dyn Any + Send + Sync>`, allowing a single manager to hold
//! `ConnectionPool<Postgres>`, `ConnectionPool<Redis>`, etc. simultaneously.

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use tracing::{debug, trace};

// ---------------------------------------------------------------------------
// ResourceManager
// ---------------------------------------------------------------------------

/// Central registry for heterogeneous resource pools.
///
/// Pools are stored as `Arc<dyn Any + Send + Sync>` and keyed by name.
/// Callers downcast to the concrete pool type via [`get_pool`](ResourceManager::get_pool).
///
/// # Thread Safety
///
/// All operations acquire a `RwLock` — reads are concurrent, writes are exclusive.
pub struct ResourceManager {
    pools: RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>,
}

impl ResourceManager {
    /// Create an empty resource manager.
    pub fn new() -> Self {
        Self {
            pools: RwLock::new(HashMap::new()),
        }
    }

    /// Register a pool under the given name.
    ///
    /// If a pool with the same name already exists it is replaced and the
    /// previous value is dropped.
    pub fn register_pool<P: Any + Send + Sync>(&self, name: &str, pool: P) {
        let mut pools = self.pools.write().expect("pools RwLock poisoned");
        pools.insert(name.to_string(), Arc::new(pool));
        debug!(name, "pool registered");
    }

    /// Look up a pool by name and downcast to the concrete type `P`.
    ///
    /// Returns `None` if the name is not found or the stored type does not
    /// match `P`.
    pub fn get_pool<P: Any + Send + Sync>(&self, name: &str) -> Option<Arc<P>> {
        let pools = self.pools.read().expect("pools RwLock poisoned");
        pools.get(name).and_then(|arc| {
            // Clone the Arc, then attempt to downcast the inner Any to P.
            let arc_any: Arc<dyn Any + Send + Sync> = Arc::clone(arc);
            arc_any.downcast::<P>().ok()
        })
    }

    /// Remove a pool by name, returning the type-erased `Arc` if it existed.
    pub fn remove_pool(&self, name: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        let mut pools = self.pools.write().expect("pools RwLock poisoned");
        let removed = pools.remove(name);
        if removed.is_some() {
            debug!(name, "pool removed");
        }
        removed
    }

    /// Return the names of all registered pools.
    pub fn pool_names(&self) -> Vec<String> {
        let pools = self.pools.read().expect("pools RwLock poisoned");
        pools.keys().cloned().collect()
    }

    /// Return the number of registered pools.
    pub fn pool_count(&self) -> usize {
        let pools = self.pools.read().expect("pools RwLock poisoned");
        let count = pools.len();
        trace!(count, "pool count queried");
        count
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct FakePool {
        name: String,
    }

    #[derive(Debug)]
    struct OtherPool {
        count: usize,
    }

    #[test]
    fn register_and_retrieve() {
        let mgr = ResourceManager::new();
        mgr.register_pool("db", FakePool { name: "postgres".into() });

        let pool = mgr.get_pool::<FakePool>("db").unwrap();
        assert_eq!(pool.name, "postgres");
    }

    #[test]
    fn get_nonexistent_returns_none() {
        let mgr = ResourceManager::new();
        assert!(mgr.get_pool::<FakePool>("nope").is_none());
    }

    #[test]
    fn wrong_type_returns_none() {
        let mgr = ResourceManager::new();
        mgr.register_pool("db", FakePool { name: "pg".into() });
        assert!(mgr.get_pool::<OtherPool>("db").is_none());
    }

    #[test]
    fn remove_pool() {
        let mgr = ResourceManager::new();
        mgr.register_pool("cache", OtherPool { count: 42 });
        assert_eq!(mgr.pool_count(), 1);

        let removed = mgr.remove_pool("cache");
        assert!(removed.is_some());
        assert_eq!(mgr.pool_count(), 0);
    }

    #[test]
    fn remove_nonexistent() {
        let mgr = ResourceManager::new();
        assert!(mgr.remove_pool("ghost").is_none());
    }

    #[test]
    fn pool_names() {
        let mgr = ResourceManager::new();
        mgr.register_pool("a", FakePool { name: "a".into() });
        mgr.register_pool("b", OtherPool { count: 1 });

        let mut names = mgr.pool_names();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn pool_count() {
        let mgr = ResourceManager::new();
        assert_eq!(mgr.pool_count(), 0);

        mgr.register_pool("x", FakePool { name: "x".into() });
        assert_eq!(mgr.pool_count(), 1);

        mgr.register_pool("y", FakePool { name: "y".into() });
        assert_eq!(mgr.pool_count(), 2);
    }

    #[test]
    fn replace_existing_pool() {
        let mgr = ResourceManager::new();
        mgr.register_pool("db", FakePool { name: "old".into() });
        mgr.register_pool("db", FakePool { name: "new".into() });

        assert_eq!(mgr.pool_count(), 1);
        let pool = mgr.get_pool::<FakePool>("db").unwrap();
        assert_eq!(pool.name, "new");
    }

    #[test]
    fn default_impl() {
        let mgr = ResourceManager::default();
        assert_eq!(mgr.pool_count(), 0);
    }
}
