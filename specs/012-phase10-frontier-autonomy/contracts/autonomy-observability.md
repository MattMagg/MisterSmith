# Contract: Autonomy Observability

## Overview

The Autonomy Observability contract defines the operator-facing state surface for Phase 10. It
exposes topology choice, branch health, checkpoint lineage, context pressure, routing rationale,
and intervention history as typed autonomy state rather than as raw logs.

## Source Map

| Source | Contract impact |
| ------ | --------------- |
| `docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md` | Human-coupled observability as architectural pillar |
| `spec/operations/observability-monitoring-framework.md` | Typed telemetry and operator presentation patterns |
| `docs/research-output/consolidated/03-supervision-and-resilience.md` | Intervention visibility requirements |

## Public API

```rust
pub struct AutonomyStatusView {
    pub graph: ExecutionGraphSummary,
    pub topology: TopologyPlanSummary,
    pub branches: Vec<BranchSummary>,
    pub memory_pressure: Vec<ContextPressureSummary>,
    pub interventions: Vec<InterventionRecord>,
    pub delegation_alerts: Vec<DelegationAlert>,
}

pub trait AutonomyStatusProvider: Send + Sync {
    async fn current_view(&self, workflow_id: TaskId) -> Result<AutonomyStatusView, StatusError>;
}
```

## Behavioral Requirements

1. The operator view MUST be reconstructable from typed autonomy state, not log scraping.
2. Topology choice MUST include rationale.
3. Branch health MUST include checkpoint lineage or recovery state when relevant.
4. Context pressure MUST be visible whenever budgets cause summarization, paging, or rejection.
5. Delegation or provenance failures MUST appear in the same autonomy status surface.

## Validation Requirements

- Operator can inspect topology and branch state for a running workflow.
- Intervention history is visible after a Guard action.
- Context-pressure event becomes visible after budget-triggered summarization.
- Delegation failure is visible without reading raw logs.
