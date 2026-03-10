# Contract: Memory Manager

## Overview

The Memory Manager contract defines the managed-memory layer above existing JetStream KV and
PostgreSQL storage. It is responsible for role-aware context assembly, paging, consolidation,
summaries, and checkpoint-ready snapshots.

## Source Map

| Source | Contract impact |
| ------ | --------------- |
| `docs/research-output/consolidated/07-memory-and-context.md` | Tiered memory, summaries, snapshots, role-aware routing |
| `spec/data-management/agent-orchestration.md` | Context management boundary |
| `spec/data-management/message-schemas.md` | Checkpoint and workflow state coordination |

## Public API

```rust
pub trait MemoryManager: Send + Sync {
    async fn assemble_snapshot(
        &self,
        scope: SnapshotScope,
        role: AgentType,
        budget: ContextBudget,
    ) -> Result<MemorySnapshot, MemoryError>;

    async fn consolidate(
        &self,
        scope: SnapshotScope,
    ) -> Result<Vec<MemoryFragment>, MemoryError>;

    async fn checkpoint(
        &self,
        branch_id: ExecutionBranchId,
    ) -> Result<MemorySnapshotId, MemoryError>;
}
```

## Memory Classes

```text
Working     — current active context
Episodic    — recent branch/workflow memory
Summary     — compacted older context
Checkpoint  — recovery-ready reconstruction state
Audit       — provenance / forensic context
```

## Behavioral Requirements

1. Delivered context MUST respect the provided `ContextBudget`.
2. Role-aware assembly MUST filter context by role and branch relevance.
3. Consolidation MUST run asynchronously and MUST NOT block active execution.
4. Every persisted fragment MUST retain provenance, freshness, access policy, and version metadata.
5. A `MemorySnapshot` MUST support checkpoint resume without replaying the entire raw history.

## Validation Requirements

- Oversized candidate context is summarized, paged, or rejected per policy.
- Different roles on the same workflow receive different assembled context sets when relevant.
- Consolidation preserves provenance metadata.
- Checkpoint snapshot is reconstructable after simulated branch failure.
