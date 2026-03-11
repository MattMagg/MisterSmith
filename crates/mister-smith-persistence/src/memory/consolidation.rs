use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use mister_smith_core::{AgentId, AgentType, MemoryError};

use super::fragment::{
    AccessPolicy, FragmentClass, FragmentFreshness, FragmentProvenance, MemoryFragment,
    SnapshotScope,
};
use super::snapshot::MemorySummary;

/// Build an inline summary over `fragments` constrained to `units`.
pub fn build_summary(
    scope: &SnapshotScope,
    fragments: &[MemoryFragment],
    units: u64,
) -> Option<MemorySummary> {
    if fragments.is_empty() || units == 0 {
        return None;
    }

    Some(MemorySummary {
        derived_from: fragments
            .iter()
            .map(|fragment| fragment.fragment_id)
            .collect(),
        content: json!({
            "scope": format!("{scope:?}"),
            "fragment_count": fragments.len(),
            "fragment_classes": fragments.iter().map(|fragment| format!("{:?}", fragment.fragment_class)).collect::<Vec<_>>(),
            "source_roles": fragments.iter().map(|fragment| format!("{:?}", fragment.provenance.source_role)).collect::<Vec<_>>(),
        }),
        units,
    })
}

/// Consolidate a scope's older fragments into a single summary fragment.
pub fn consolidate_fragments(
    scope: SnapshotScope,
    fragments: &[MemoryFragment],
) -> Result<Option<MemoryFragment>, MemoryError> {
    if fragments.is_empty() {
        return Ok(None);
    }

    let mut ordered_fragments = fragments.to_vec();
    ordered_fragments.sort_by(|left, right| {
        left.freshness
            .recorded_at
            .cmp(&right.freshness.recorded_at)
            .then_with(|| {
                left.provenance
                    .recorded_at
                    .cmp(&right.provenance.recorded_at)
            })
            .then_with(|| left.fragment_id.as_ref().cmp(right.fragment_id.as_ref()))
    });

    let workflow_id = ordered_fragments
        .first()
        .map(|fragment| fragment.provenance.workflow_id)
        .unwrap_or_default();
    let branch_id = ordered_fragments
        .first()
        .and_then(|fragment| fragment.provenance.branch_id);
    let allowed_roles = ordered_fragments
        .iter()
        .fold(Vec::new(), |mut roles, fragment| {
            for role in &fragment.access_policy.allowed_roles {
                if !roles.contains(role) {
                    roles.push(*role);
                }
            }
            roles
        });
    let derived_from = ordered_fragments
        .iter()
        .map(|fragment| fragment.fragment_id)
        .collect::<Vec<_>>();
    let total_units = ordered_fragments
        .iter()
        .map(|fragment| fragment.units)
        .sum::<u64>();
    let summary = build_summary(
        &scope,
        &ordered_fragments,
        std::cmp::max(1, total_units / 2),
    )
    .ok_or_else(|| MemoryError::SnapshotUnavailable {
        snapshot_id: None,
        message: "unable to build consolidation summary".to_string(),
    })?;

    let mut provenance = FragmentProvenance::new(
        workflow_id,
        branch_id,
        AgentId::from_uuid(Uuid::nil()),
        AgentType::Memory,
        "managed_memory.consolidation",
    );
    provenance.derived_from = derived_from;
    provenance.recorded_at = Utc::now();

    let mut access_policy = AccessPolicy::for_roles(allowed_roles);
    if let Some(branch_id) = branch_id {
        access_policy = access_policy.for_branch(branch_id);
    }

    Ok(Some(MemoryFragment::new(
        scope,
        summary.content,
        summary.units,
        FragmentClass::Summary,
        provenance,
        FragmentFreshness::ttl(Utc::now(), chrono::Duration::hours(6)),
        access_policy,
    )))
}
