use std::cmp::Ordering;
use std::collections::HashMap;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use mister_smith_core::{
    AgentId, AgentType, BudgetPolicy, ContextBudget, MemoryError, MemorySnapshotId, TaskId,
};

use super::consolidation::{build_summary, consolidate_fragments};
use super::fragment::{
    AccessPolicy, FragmentClass, FragmentFreshness, FragmentProvenance, MemoryFragment,
    SnapshotScope,
};
use super::snapshot::{MaterializedSnapshot, MemorySnapshot, ResumeSource};

/// In-process managed-memory coordinator used by Phase 10 context assembly.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManagedMemoryManager {
    fragments: HashMap<mister_smith_core::MemoryFragmentId, MemoryFragment>,
    snapshots: HashMap<MemorySnapshotId, MemorySnapshot>,
}

impl ManagedMemoryManager {
    /// Store or replace a fragment in the manager.
    pub fn record_fragment(&mut self, fragment: MemoryFragment) {
        self.fragments.insert(fragment.fragment_id, fragment);
    }

    /// Fetch a stored snapshot by identifier.
    pub fn snapshot(&self, snapshot_id: MemorySnapshotId) -> Option<MemorySnapshot> {
        self.snapshots.get(&snapshot_id).cloned()
    }

    /// Assemble a role-aware snapshot under the provided budget.
    pub async fn assemble_snapshot(
        &mut self,
        scope: SnapshotScope,
        role: AgentType,
        budget: ContextBudget,
    ) -> Result<MemorySnapshot, MemoryError> {
        let mut candidates = self.visible_fragments(&scope, role);
        let mut total_candidate_units = candidates.iter().map(|fragment| fragment.units).sum();

        if budget.policy == BudgetPolicy::Consolidate && total_candidate_units > budget.max_units {
            self.consolidate(scope.clone()).await?;
            candidates = self.visible_fragments(&scope, role);
            total_candidate_units = candidates.iter().map(|fragment| fragment.units).sum();
        }

        if budget.policy == BudgetPolicy::Reject && total_candidate_units > budget.max_units {
            return Err(MemoryError::BudgetExceeded {
                budget_id: Some(budget.budget_id),
                requested: total_candidate_units,
                max: budget.max_units,
            });
        }

        let snapshot = self.build_snapshot(scope, role, budget, candidates, total_candidate_units)?;
        self.snapshots.insert(snapshot.snapshot_id, snapshot.clone());
        Ok(snapshot)
    }

    /// Consolidate older fragments for a scope into a summary fragment.
    pub async fn consolidate(
        &mut self,
        scope: SnapshotScope,
    ) -> Result<Vec<MemoryFragment>, MemoryError> {
        let eligible = self
            .fragments
            .values()
            .filter(|fragment| {
                fragment.scope == scope
                    && matches!(
                        fragment.fragment_class,
                        FragmentClass::Episodic | FragmentClass::Summary | FragmentClass::Audit
                    )
            })
            .cloned()
            .collect::<Vec<_>>();

        let Some(summary) = consolidate_fragments(scope, &eligible)? else {
            return Ok(Vec::new());
        };

        for fragment in &eligible {
            self.fragments.remove(&fragment.fragment_id);
        }
        self.record_fragment(summary.clone());

        Ok(vec![summary])
    }

    /// Persist a checkpoint-ready snapshot for a branch.
    pub async fn checkpoint(
        &mut self,
        branch_id: mister_smith_core::ExecutionBranchId,
        role: AgentType,
        budget: ContextBudget,
    ) -> Result<MemorySnapshotId, MemoryError> {
        let scope = SnapshotScope::Branch(branch_id);
        let mut snapshot = self
            .assemble_snapshot(scope.clone(), role, budget.clone())
            .await?;

        let workflow_id = snapshot
            .fragment_ids
            .first()
            .and_then(|fragment_id| self.fragments.get(fragment_id))
            .map(|fragment| fragment.provenance.workflow_id)
            .unwrap_or_else(|| TaskId::from_uuid(*branch_id.as_ref()));

        let mut provenance = FragmentProvenance::new(
            workflow_id,
            Some(branch_id),
            AgentId::from_uuid(Uuid::nil()),
            AgentType::Memory,
            "managed_memory.checkpoint",
        );
        provenance.derived_from = snapshot.fragment_ids.clone();
        provenance.recorded_at = Utc::now();

        let checkpoint_fragment = MemoryFragment::new(
            scope.clone(),
            json!({
                "snapshot_id": snapshot.snapshot_id,
                "summary": snapshot.summary,
                "fragment_ids": snapshot.fragment_ids,
                "role": format!("{role:?}"),
                "delivered_units": snapshot.delivered_units,
            }),
            std::cmp::max(1, snapshot.delivered_units),
            FragmentClass::Checkpoint,
            provenance,
            FragmentFreshness::ttl(Utc::now(), chrono::Duration::hours(6)),
            AccessPolicy::for_roles(vec![role]).for_branch(branch_id),
        );
        let checkpoint_fragment_id = checkpoint_fragment.fragment_id;
        self.record_fragment(checkpoint_fragment);

        snapshot.checkpoint_fragment_id = Some(checkpoint_fragment_id);
        self.snapshots.insert(snapshot.snapshot_id, snapshot.clone());

        Ok(snapshot.snapshot_id)
    }

    /// Materialize a stored snapshot for resume or prompt assembly.
    pub fn materialize_snapshot(
        &self,
        snapshot_id: MemorySnapshotId,
    ) -> Result<MaterializedSnapshot, MemoryError> {
        let snapshot = self
            .snapshots
            .get(&snapshot_id)
            .cloned()
            .ok_or_else(|| MemoryError::SnapshotUnavailable {
                snapshot_id: Some(snapshot_id),
                message: "snapshot not found".to_string(),
            })?;

        if let Some(checkpoint_fragment_id) = snapshot.checkpoint_fragment_id {
            let checkpoint_fragment = self
                .fragments
                .get(&checkpoint_fragment_id)
                .cloned()
                .ok_or_else(|| MemoryError::SnapshotUnavailable {
                    snapshot_id: Some(snapshot_id),
                    message: "checkpoint fragment missing".to_string(),
                })?;

            return Ok(MaterializedSnapshot {
                snapshot,
                fragments: vec![checkpoint_fragment],
                resume_source: ResumeSource::Checkpoint,
            });
        }

        let mut fragments = Vec::with_capacity(snapshot.fragment_ids.len());
        for fragment_id in &snapshot.fragment_ids {
            let fragment = self
                .fragments
                .get(fragment_id)
                .cloned()
                .ok_or_else(|| MemoryError::SnapshotUnavailable {
                    snapshot_id: Some(snapshot_id),
                    message: format!("fragment {fragment_id} missing"),
                })?;
            fragments.push(fragment);
        }

        Ok(MaterializedSnapshot {
            snapshot,
            fragments,
            resume_source: ResumeSource::FragmentSelection,
        })
    }

    fn build_snapshot(
        &self,
        scope: SnapshotScope,
        role: AgentType,
        budget: ContextBudget,
        candidates: Vec<MemoryFragment>,
        total_candidate_units: u64,
    ) -> Result<MemorySnapshot, MemoryError> {
        let (selected, summary, delivered_units) = match budget.policy {
            BudgetPolicy::Reject => {
                let selected_units = candidates.iter().map(|fragment| fragment.units).sum();
                (candidates, None, selected_units)
            }
            BudgetPolicy::Evict | BudgetPolicy::Consolidate => {
                let (selected, selected_units) = select_under_budget(candidates, budget.max_units);
                (selected, None, selected_units)
            }
            BudgetPolicy::Summarize => summarize_under_budget(scope.clone(), candidates, budget.max_units),
        };

        Ok(MemorySnapshot {
            snapshot_id: MemorySnapshotId::new(),
            target_scope: scope,
            role,
            fragment_ids: selected.iter().map(|fragment| fragment.fragment_id).collect(),
            summary,
            created_at: Utc::now(),
            budget_id: budget.budget_id,
            total_candidate_units,
            delivered_units,
            checkpoint_fragment_id: None,
        })
    }

    fn visible_fragments(&self, scope: &SnapshotScope, role: AgentType) -> Vec<MemoryFragment> {
        let now = Utc::now();
        let mut fragments = self
            .fragments
            .values()
            .filter(|fragment| fragment.is_visible_to(role, scope, now))
            .cloned()
            .collect::<Vec<_>>();

        fragments.sort_by(compare_fragments);
        fragments
    }
}

fn compare_fragments(left: &MemoryFragment, right: &MemoryFragment) -> Ordering {
    priority_rank(right.fragment_class)
        .cmp(&priority_rank(left.fragment_class))
        .then_with(|| right.freshness.recorded_at.cmp(&left.freshness.recorded_at))
}

fn priority_rank(fragment_class: FragmentClass) -> u8 {
    match fragment_class {
        FragmentClass::Checkpoint => 5,
        FragmentClass::Working => 4,
        FragmentClass::Episodic => 3,
        FragmentClass::Summary => 2,
        FragmentClass::Audit => 1,
    }
}

fn select_under_budget(
    candidates: Vec<MemoryFragment>,
    max_units: u64,
) -> (Vec<MemoryFragment>, u64) {
    let mut selected = Vec::new();
    let mut selected_units = 0;

    for fragment in candidates {
        if selected_units + fragment.units <= max_units {
            selected_units += fragment.units;
            selected.push(fragment);
        }
    }

    (selected, selected_units)
}

fn summarize_under_budget(
    scope: SnapshotScope,
    candidates: Vec<MemoryFragment>,
    max_units: u64,
) -> (Vec<MemoryFragment>, Option<super::snapshot::MemorySummary>, u64) {
    let mut selected = Vec::new();
    let mut selected_units = 0;

    for fragment in candidates {
        if selected_units + fragment.units <= max_units {
            selected_units += fragment.units;
            selected.push(fragment);
        } else {
            let overflow = vec![fragment];
            return attach_summary(scope, selected, selected_units, overflow, max_units);
        }
    }

    (selected, None, selected_units)
}

fn attach_summary(
    scope: SnapshotScope,
    mut selected: Vec<MemoryFragment>,
    mut selected_units: u64,
    mut overflow: Vec<MemoryFragment>,
    max_units: u64,
) -> (Vec<MemoryFragment>, Option<super::snapshot::MemorySummary>, u64) {
    let mut remaining_capacity = max_units.saturating_sub(selected_units);
    if remaining_capacity == 0 && !selected.is_empty() {
        if let Some(last_fragment) = selected.pop() {
            selected_units = selected_units.saturating_sub(last_fragment.units);
            remaining_capacity = max_units.saturating_sub(selected_units);
            overflow.insert(0, last_fragment);
        }
    }

    let summary = build_summary(&scope, &overflow, remaining_capacity);
    let delivered_units = selected_units + summary.as_ref().map(|summary| summary.units).unwrap_or(0);

    (selected, summary, delivered_units)
}
