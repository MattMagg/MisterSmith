use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use mister_smith_core::LlmError;
use serde::{Deserialize, Serialize};

/// Budget enforcement behavior.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum BudgetPolicy {
    /// Reject requests when budget is exhausted.
    #[default]
    HardCap,
    /// Downgrade to cheaper model when budget is low.
    SoftCap,
    /// Route to progressively cheaper models as budget depletes.
    Conditioned,
}

/// Hierarchical budget entry stored in JetStream KV.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetNode {
    pub key: String,
    pub limit_tokens: u64,
    pub used_tokens: u64,
    pub period: String,
    pub policy: BudgetPolicy,
    /// Revision for CAS -- used by JetStream KV to prevent concurrent overwrite.
    #[serde(default)]
    pub revision: u64,
}

impl BudgetNode {
    pub fn remaining(&self) -> i64 {
        self.limit_tokens as i64 - self.used_tokens as i64
    }

    pub fn is_exhausted(&self) -> bool {
        self.used_tokens >= self.limit_tokens
    }

    /// Utilization percentage (0.0 to 1.0+).
    pub fn utilization(&self) -> f64 {
        if self.limit_tokens == 0 {
            return 1.0;
        }
        self.used_tokens as f64 / self.limit_tokens as f64
    }
}

/// Reservation handle returned from a successful reserve operation.
#[derive(Debug, Clone)]
pub struct BudgetReservation {
    pub budget_key: String,
    pub estimated_tokens: u64,
    pub revision: u64,
}

/// Budget store abstraction. Production uses JetStream KV CAS; tests use in-memory.
#[async_trait]
pub trait BudgetStore: Send + Sync {
    /// Get the current budget node.
    async fn get(&self, key: &str) -> Result<Option<BudgetNode>, LlmError>;

    /// Atomic compare-and-swap update. Returns the new revision on success.
    async fn cas_update(&self, node: &BudgetNode, expected_revision: u64)
        -> Result<u64, LlmError>;
}

/// In-memory budget store for testing and local development.
pub struct InMemoryBudgetStore {
    nodes: Mutex<HashMap<String, BudgetNode>>,
}

impl InMemoryBudgetStore {
    pub fn new() -> Self {
        Self {
            nodes: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert(&self, node: BudgetNode) {
        let mut nodes = self.nodes.lock().unwrap();
        nodes.insert(node.key.clone(), node);
    }
}

impl Default for InMemoryBudgetStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BudgetStore for InMemoryBudgetStore {
    async fn get(&self, key: &str) -> Result<Option<BudgetNode>, LlmError> {
        let nodes = self.nodes.lock().unwrap();
        Ok(nodes.get(key).cloned())
    }

    async fn cas_update(
        &self,
        node: &BudgetNode,
        expected_revision: u64,
    ) -> Result<u64, LlmError> {
        let mut nodes = self.nodes.lock().unwrap();
        if let Some(existing) = nodes.get(&node.key) {
            if existing.revision != expected_revision {
                return Err(LlmError::InvalidRequest(format!(
                    "CAS conflict on budget key '{}': expected revision {}, actual {}",
                    node.key, expected_revision, existing.revision
                )));
            }
        }
        let new_revision = expected_revision + 1;
        let mut updated = node.clone();
        updated.revision = new_revision;
        nodes.insert(node.key.clone(), updated);
        Ok(new_revision)
    }
}

/// Budget enforcer that implements the reserve-before-send / reconcile-after-completion pattern.
pub struct BudgetEnforcer {
    store: Box<dyn BudgetStore>,
}

impl BudgetEnforcer {
    const RECONCILE_MAX_RETRIES: usize = 3;

    pub fn new(store: Box<dyn BudgetStore>) -> Self {
        Self { store }
    }

    /// Reserve estimated tokens before sending a request.
    /// Returns a reservation handle for later reconciliation.
    pub async fn reserve(
        &self,
        budget_key: &str,
        estimated_tokens: u64,
    ) -> Result<BudgetReservation, LlmError> {
        let node = self.store.get(budget_key).await?.ok_or_else(|| {
            LlmError::InvalidRequest(format!("Budget key '{}' not found", budget_key))
        })?;

        match node.policy {
            BudgetPolicy::HardCap => {
                if node.used_tokens + estimated_tokens > node.limit_tokens {
                    return Err(LlmError::BudgetExhausted {
                        message: format!(
                            "Hard cap: {} used + {} estimated > {} limit",
                            node.used_tokens, estimated_tokens, node.limit_tokens
                        ),
                        budget_key: budget_key.to_string(),
                    });
                }
            }
            BudgetPolicy::SoftCap | BudgetPolicy::Conditioned => {
                // SoftCap and Conditioned allow the request but signal degradation
                // The caller (ModelRouter) handles downgrade logic
            }
        }

        let mut updated = node.clone();
        updated.used_tokens += estimated_tokens;

        let new_revision = self.store.cas_update(&updated, node.revision).await?;

        Ok(BudgetReservation {
            budget_key: budget_key.to_string(),
            estimated_tokens,
            revision: new_revision,
        })
    }

    /// Reconcile actual usage after completion. Adjusts the budget if actual differs from estimated.
    pub async fn reconcile(
        &self,
        reservation: &BudgetReservation,
        actual_tokens: u64,
    ) -> Result<(), LlmError> {
        for attempt in 0..=Self::RECONCILE_MAX_RETRIES {
            let node = self.store.get(&reservation.budget_key).await?.ok_or_else(|| {
                LlmError::InvalidRequest(format!(
                    "Budget key '{}' not found during reconciliation",
                    reservation.budget_key
                ))
            })?;

            let adjustment = actual_tokens as i64 - reservation.estimated_tokens as i64;
            let new_used = (node.used_tokens as i64 + adjustment).max(0) as u64;

            let mut updated = node.clone();
            updated.used_tokens = new_used;

            match self.store.cas_update(&updated, node.revision).await {
                Ok(_) => return Ok(()),
                Err(err)
                    if Self::is_cas_conflict(&err) && attempt < Self::RECONCILE_MAX_RETRIES =>
                {
                    continue;
                }
                Err(err) => return Err(err),
            }
        }

        unreachable!("reconcile loop exits via return")
    }

    fn is_cas_conflict(err: &LlmError) -> bool {
        matches!(err, LlmError::InvalidRequest(message) if message.contains("CAS conflict"))
    }

    /// Get the current budget state for a key.
    pub async fn get_budget(&self, key: &str) -> Result<Option<BudgetNode>, LlmError> {
        self.store.get(key).await
    }

    /// Resolve the budget key hierarchy. Returns keys from most specific to least.
    /// E.g., for "budget/org1/team-alpha/user-42", returns:
    /// ["budget/org1/team-alpha/user-42", "budget/org1/team-alpha", "budget/org1", "budget"]
    pub fn resolve_hierarchy(key: &str) -> Vec<String> {
        let parts: Vec<&str> = key.split('/').collect();
        let mut keys = Vec::new();
        for i in (1..=parts.len()).rev() {
            keys.push(parts[..i].join("/"));
        }
        keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn test_store() -> InMemoryBudgetStore {
        let store = InMemoryBudgetStore::new();
        store.insert(BudgetNode {
            key: "budget/org1/team-alpha".to_string(),
            limit_tokens: 10000,
            used_tokens: 0,
            period: "2026-03-daily".to_string(),
            policy: BudgetPolicy::HardCap,
            revision: 1,
        });
        store
    }

    #[tokio::test]
    async fn reserve_and_reconcile_round_trip() {
        let store = test_store();
        let enforcer = BudgetEnforcer::new(Box::new(store));

        let reservation = enforcer
            .reserve("budget/org1/team-alpha", 500)
            .await
            .unwrap();
        assert_eq!(reservation.estimated_tokens, 500);

        // Budget should show 500 used
        let node = enforcer
            .get_budget("budget/org1/team-alpha")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(node.used_tokens, 500);

        // Reconcile with actual usage (300 tokens used instead of 500 estimated)
        enforcer.reconcile(&reservation, 300).await.unwrap();

        let node = enforcer
            .get_budget("budget/org1/team-alpha")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(node.used_tokens, 300);
    }

    #[tokio::test]
    async fn hard_cap_rejects_when_exhausted() {
        let store = test_store();
        let enforcer = BudgetEnforcer::new(Box::new(store));

        // Reserve most of the budget
        enforcer
            .reserve("budget/org1/team-alpha", 9500)
            .await
            .unwrap();

        // This should fail - would exceed hard cap
        let result = enforcer.reserve("budget/org1/team-alpha", 1000).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LlmError::BudgetExhausted { .. }
        ));
    }

    #[tokio::test]
    async fn soft_cap_allows_overage() {
        let store = InMemoryBudgetStore::new();
        store.insert(BudgetNode {
            key: "budget/soft".to_string(),
            limit_tokens: 1000,
            used_tokens: 900,
            period: "test".to_string(),
            policy: BudgetPolicy::SoftCap,
            revision: 1,
        });
        let enforcer = BudgetEnforcer::new(Box::new(store));

        // SoftCap allows the request even though it would exceed
        let result = enforcer.reserve("budget/soft", 500).await;
        assert!(result.is_ok());
    }

    struct ConflictOnceStore {
        inner: InMemoryBudgetStore,
        cas_attempts: AtomicUsize,
    }

    #[async_trait]
    impl BudgetStore for ConflictOnceStore {
        async fn get(&self, key: &str) -> Result<Option<BudgetNode>, LlmError> {
            self.inner.get(key).await
        }

        async fn cas_update(
            &self,
            node: &BudgetNode,
            expected_revision: u64,
        ) -> Result<u64, LlmError> {
            let attempt = self.cas_attempts.fetch_add(1, Ordering::SeqCst);
            if attempt == 1 {
                // First reconcile CAS update fails once to simulate contention.
                return Err(LlmError::InvalidRequest(
                    "CAS conflict on budget key 'budget/org1/team-alpha': expected revision 2, actual 3"
                        .to_string(),
                ));
            }

            self.inner.cas_update(node, expected_revision).await
        }
    }

    #[tokio::test]
    async fn reconcile_retries_transient_cas_conflicts() {
        let store = InMemoryBudgetStore::new();
        store.insert(BudgetNode {
            key: "budget/org1/team-alpha".to_string(),
            limit_tokens: 10000,
            used_tokens: 0,
            period: "2026-03-daily".to_string(),
            policy: BudgetPolicy::HardCap,
            revision: 1,
        });

        let store = ConflictOnceStore {
            inner: store,
            cas_attempts: AtomicUsize::new(0),
        };
        let enforcer = BudgetEnforcer::new(Box::new(store));

        let reservation = enforcer
            .reserve("budget/org1/team-alpha", 500)
            .await
            .unwrap();
        enforcer.reconcile(&reservation, 300).await.unwrap();

        let node = enforcer
            .get_budget("budget/org1/team-alpha")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(node.used_tokens, 300);
    }

    #[test]
    fn budget_node_remaining_and_utilization() {
        let node = BudgetNode {
            key: "test".to_string(),
            limit_tokens: 10000,
            used_tokens: 7500,
            period: "test".to_string(),
            policy: BudgetPolicy::HardCap,
            revision: 1,
        };
        assert_eq!(node.remaining(), 2500);
        assert!((node.utilization() - 0.75).abs() < f64::EPSILON);
        assert!(!node.is_exhausted());
    }

    #[test]
    fn hierarchy_resolution() {
        let keys = BudgetEnforcer::resolve_hierarchy("budget/org1/team-alpha/user-42");
        assert_eq!(
            keys,
            vec![
                "budget/org1/team-alpha/user-42",
                "budget/org1/team-alpha",
                "budget/org1",
                "budget",
            ]
        );
    }
}
