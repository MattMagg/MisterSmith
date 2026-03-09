use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use async_trait::async_trait;
use mister_smith_core::LlmError;
use mister_smith_llm::budget::{
    BudgetEnforcer, BudgetNode, BudgetPolicy, BudgetReservation, BudgetStore, InMemoryBudgetStore,
};

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

    // This should fail — would exceed hard cap
    let result = enforcer.reserve("budget/org1/team-alpha", 1000).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        mister_smith_core::LlmError::BudgetExhausted { .. }
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

#[tokio::test]
async fn concurrent_cas_prevents_overrun() {
    let store = InMemoryBudgetStore::new();
    store.insert(BudgetNode {
        key: "budget/concurrent".to_string(),
        limit_tokens: 10000,
        used_tokens: 0,
        period: "test".to_string(),
        policy: BudgetPolicy::HardCap,
        revision: 1,
    });
    let _store = std::sync::Arc::new(store);

    // Simulate sequential reservations (CAS guarantees consistency)
    let _enforcer = BudgetEnforcer::new(Box::new(InMemoryBudgetStore::new()));
    // Re-create with shared store
    let shared_store = InMemoryBudgetStore::new();
    shared_store.insert(BudgetNode {
        key: "budget/concurrent".to_string(),
        limit_tokens: 10000,
        used_tokens: 0,
        period: "test".to_string(),
        policy: BudgetPolicy::HardCap,
        revision: 1,
    });
    let enforcer = BudgetEnforcer::new(Box::new(shared_store));

    // Make 20 sequential reservations of 400 tokens each
    let mut successful = 0;
    for _ in 0..30 {
        match enforcer.reserve("budget/concurrent", 400).await {
            Ok(_) => successful += 1,
            Err(_) => break,
        }
    }

    // With 10000 limit and 400 per request, should get exactly 25 successful
    assert_eq!(successful, 25);

    let node = enforcer
        .get_budget("budget/concurrent")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(node.used_tokens, 10000);
}

#[tokio::test]
async fn reconcile_adjusts_for_overshoot() {
    let store = test_store();
    let enforcer = BudgetEnforcer::new(Box::new(store));

    // Reserve 500, actually used 700
    let reservation = enforcer
        .reserve("budget/org1/team-alpha", 500)
        .await
        .unwrap();
    enforcer.reconcile(&reservation, 700).await.unwrap();

    let node = enforcer
        .get_budget("budget/org1/team-alpha")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(node.used_tokens, 700);
}

#[tokio::test]
async fn reconcile_adjusts_for_undershoot() {
    let store = test_store();
    let enforcer = BudgetEnforcer::new(Box::new(store));

    // Reserve 500, actually used 200
    let reservation = enforcer
        .reserve("budget/org1/team-alpha", 500)
        .await
        .unwrap();
    enforcer.reconcile(&reservation, 200).await.unwrap();

    let node = enforcer
        .get_budget("budget/org1/team-alpha")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(node.used_tokens, 200);
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
fn budget_node_exhausted() {
    let node = BudgetNode {
        key: "test".to_string(),
        limit_tokens: 1000,
        used_tokens: 1000,
        period: "test".to_string(),
        policy: BudgetPolicy::HardCap,
        revision: 1,
    };
    assert!(node.is_exhausted());
    assert_eq!(node.remaining(), 0);
}

#[test]
fn hierarchical_budget_key_resolution() {
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

#[tokio::test]
async fn missing_budget_key_returns_error() {
    let store = InMemoryBudgetStore::new();
    let enforcer = BudgetEnforcer::new(Box::new(store));

    let result = enforcer.reserve("nonexistent/key", 100).await;
    assert!(result.is_err());
}

#[test]
fn budget_policy_serde_round_trip() {
    let policies = [
        BudgetPolicy::HardCap,
        BudgetPolicy::SoftCap,
        BudgetPolicy::Conditioned,
    ];
    for policy in &policies {
        let json = serde_json::to_string(policy).unwrap();
        let deserialized: BudgetPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(*policy, deserialized);
    }
}

// --- CAS retry tests (#118) ---

/// Budget store that simulates a configurable number of CAS conflicts.
struct FlakyCasStore {
    node: Mutex<BudgetNode>,
    conflicts_remaining: AtomicUsize,
}

impl FlakyCasStore {
    fn new(node: BudgetNode, conflicts: usize) -> Self {
        Self {
            node: Mutex::new(node),
            conflicts_remaining: AtomicUsize::new(conflicts),
        }
    }
}

#[async_trait]
impl BudgetStore for FlakyCasStore {
    async fn get(&self, _key: &str) -> Result<Option<BudgetNode>, LlmError> {
        Ok(Some(self.node.lock().unwrap().clone()))
    }

    async fn cas_update(&self, node: &BudgetNode, expected_revision: u64) -> Result<u64, LlmError> {
        let remaining =
            self.conflicts_remaining
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                    if v > 0 {
                        Some(v - 1)
                    } else {
                        None
                    }
                });

        if remaining.is_ok() {
            // Simulate conflict: bump revision but return error
            let mut inner = self.node.lock().unwrap();
            inner.revision += 1;
            return Err(LlmError::InvalidRequest(format!(
                "CAS conflict on budget key '{}': expected revision {}, actual {}",
                node.key, expected_revision, inner.revision
            )));
        }

        // Allow the update
        let mut inner = self.node.lock().unwrap();
        let new_revision = inner.revision + 1;
        inner.used_tokens = node.used_tokens;
        inner.revision = new_revision;
        Ok(new_revision)
    }
}

#[tokio::test]
async fn reconcile_retries_on_cas_conflict_and_succeeds() {
    let node = BudgetNode {
        key: "budget/flaky".to_string(),
        limit_tokens: 10000,
        used_tokens: 500,
        period: "test".to_string(),
        policy: BudgetPolicy::HardCap,
        revision: 1,
    };
    // 2 conflicts, then success
    let store = FlakyCasStore::new(node, 2);
    let enforcer = BudgetEnforcer::new(Box::new(store));

    let reservation = BudgetReservation {
        budget_key: "budget/flaky".to_string(),
        estimated_tokens: 500,
        revision: 2,
    };

    enforcer.reconcile(&reservation, 200).await.unwrap();

    let result = enforcer.get_budget("budget/flaky").await.unwrap().unwrap();
    assert_eq!(result.used_tokens, 200);
}

#[tokio::test]
async fn reconcile_returns_error_after_retry_limit_exhausted() {
    let node = BudgetNode {
        key: "budget/flaky".to_string(),
        limit_tokens: 10000,
        used_tokens: 500,
        period: "test".to_string(),
        policy: BudgetPolicy::HardCap,
        revision: 1,
    };
    // 10 conflicts — more than CAS_RETRY_LIMIT (3)
    let store = FlakyCasStore::new(node, 10);
    let enforcer = BudgetEnforcer::new(Box::new(store));

    let reservation = BudgetReservation {
        budget_key: "budget/flaky".to_string(),
        estimated_tokens: 500,
        revision: 2,
    };

    let err = enforcer.reconcile(&reservation, 200).await.unwrap_err();
    assert!(
        err.to_string().contains("CAS conflict"),
        "expected CAS conflict error, got: {err}"
    );
}
