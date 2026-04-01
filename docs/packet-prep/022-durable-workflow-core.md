# Packet 022: Durable Workflow Core

## Packet Name

Durable workflow core  
Sharper framing: durable workflow semantics, effect boundaries, and lifecycle control

## Why This Packet Exists

The repo now has durable-looking pieces: JetStream streams, KV-backed checkpoints, session continuity,
repair lineage, and workflow IDs. What it does not have yet is one explicit durable workflow contract
for:

- event-history semantics
- replay-safe state transitions
- idempotent activity boundaries
- pause, cancel, terminate, and resume behavior
- history compaction and version-safe replay

Without that seam, later orchestration work will keep leaning on partial durability instead of a
clear substrate contract.

## Why This Stage Is Correct

`docs/direction.md` puts durable workflow semantics first in the current `Now` band. That matches
the March 28 durable-workflow brief: the next real substrate gain is not another new frontier trick,
it is getting long-running workflow semantics honest and replay-safe before autonomy expands.

This packet belongs before:

- real coordinator-subagent runtime work
- interoperability expansion
- stronger coordination models

## Repo Truth Status

- Packet outcome today: `planned-only`
- Foundation truth status: `landed-not-default`
- Live-default today:
  - accepted workflow execution on the supported runtime path
  - stable `workflow_id`, `session_id`, and `coordinator_agent_id` continuity
  - runtime-owned repair lineage and checkpoint references on supported result surfaces
- Landed but not yet one frozen durable-workflow contract:
  - branch checkpoint capture, resume metadata, and KV-backed branch history
  - HybridStateManager read/write helpers for branch checkpoint and resume history
- Missing for this packet:
  - one repo-native event-history model
  - one durable lifecycle state machine
  - one effect/outbox discipline for external side effects

## Current Repo Grounding

### Live on the default runtime path now

- runtime-backed workflow execution on the default path
- bounded same-agent sessions with `session_id` and `coordinator_agent_id`
- repair lineage, last-stable-checkpoint references, and runtime-owned provenance surfaces

### Landed in repo but not yet the default durable contract

- branch checkpoint and branch resume KV keys in `mister-smith-persistence`
- repository-backed branch checkpoint persistence with SQL-authoritative metadata plus KV cache
- JetStream-backed budget and workflow state surfaces
- workflow IDs, task IDs, session IDs, and branch checkpoints exist, but there is no single
  event-history contract for durable replay
- the March 19 restart/resume proof note shows recovery seams exist, but it is evidence of one
  bounded runtime slice, not the durable-workflow contract this packet still needs to freeze

### Missing pieces

- explicit durable workflow state machine
- effect commit discipline for external side effects
- outbox/inbox bridging
- lifecycle verbs and semantics across task, session, and autonomy surfaces
- history compaction, continue-as-new, and replay-regression gates

### High-Signal Repo Anchors

- `crates/mister-smith-agents/src/branch_checkpoint.rs`
  - `BranchCheckpointStore`
  - `RepositoryBranchCheckpointStore`
  - `BranchResumeMetadata`
  - `resume_branch_with_delegation`
  - `reassign_branch_with_delegation`
  - This is the clearest current branch-durability seam.
- `crates/mister-smith-agents/src/execution_graph.rs`
  - `BranchCheckpoint`
  - `ExecutionGraph.checkpoint_lineage`
  - This is the current in-memory shape the packet would need to make durable and replay-safe.
- `crates/mister-smith-agents/src/orchestrator.rs`
  - `record_branch_checkpoint`
  - `resume_branch_with_delegation`
  - `reassign_branch_with_delegation`
  - This is the runtime bridge from orchestration decisions into checkpoint capture and recovery.
- `crates/mister-smith-persistence/src/kv/state.rs`
  - `branch_checkpoint_key`
  - `branch_resume_history_key`
  - `StateManager::update`
  - This is the current CAS-backed state-transition primitive.
- `crates/mister-smith-persistence/src/hybrid/manager.rs`
  - `HybridStateManager::write_branch_checkpoint`
  - `HybridStateManager::read_branch_checkpoint`
  - `HybridStateManager::write_branch_resume_history`
  - This is the current SQL-plus-KV durability bridge.
- `crates/mister-smith-app/src/conversation.rs`
  - `ConversationRuntimeService`
  - `SessionRecord`
  - `SessionTurnRecord`
  - This is the current session-lifecycle continuity surface that packet `022` must not break.
- `docs/plans/2026-03-19-session-restart-resume-live-proof.md`
  - session restart and resumed lineage preserved on the supported `openai_chatgpt` / `gpt-5.4`
    path
  - artifact lane under
    `docs/plans/artifacts/2026-03-19-session-restart-resume-live-proof/`
  - This is the strongest live proof that session continuity already survives restart/resume even
    though durable workflow semantics are not yet frozen.
- `crates/mister-smith-persistence/src/repository/task.rs`
  - `TaskRepository::save_branch_checkpoint`
  - `TaskRepository::save_branch_resume_history`
  - `branch_recovery_metadata_tracks_latest_checkpoint_per_branch`
  - `branch_resume_history_filters_by_branch_and_preserves_order`
  - These are the strongest current SQL-authoritative checkpoint metadata seams and tests.
- `crates/mister-smith-agents/tests/gate10_tests.rs`
  - `gate10_delegated_resume_reconstructs_checkpoint_provenance`
  - `gate10_delegated_resume_preserves_existing_failure_context`
  - `gate10_rejected_delegated_resume_surfaces_operator_reason`
  - These are the strongest runtime-facing delegation-and-resume proof anchors for current branch
    recovery behavior.
- `docs/research-output/analysis/2026-03-28-durable-workflows-transfer-brief.md`
  - This is the research note that already argues for event history, idempotent effects, and
    lifecycle verbs as the next substrate seam.

## Official Docs / Primary Sources

These sources set the semantic bar for durable workflows. They are not proof that Mister Smith
already ships a Temporal-style workflow engine today.

### Direct Substrate Source

- [NATS JetStream model deep dive](https://docs.nats.io/using-nats/developer/develop_jetstream/model_deep_dive)  
  Why it matters: clarifies what JetStream does provide, and what broker-level semantics do not automatically guarantee.

### Durable-Workflow Semantic Comparators

- [Temporal Workflow Execution](https://docs.temporal.io/workflow-execution)  
  Why it matters: strong comparator reference for event-history workflow semantics and replay.
- [Temporal Continue-As-New](https://docs.temporal.io/workflow-execution/continue-as-new)  
  Why it matters: comparator reference for bounding long-running workflow history instead of
  letting replay cost grow forever.
- [Temporal Activities](https://docs.temporal.io/activities)  
  Why it matters: clearest comparator reference for idempotent activity boundaries and side
  effects.
- [Azure Durable Functions overview](https://learn.microsoft.com/azure/azure-functions/durable/durable-functions-overview)  
  Why it matters: secondary comparator reference for orchestrator replay, event sourcing, and
  lifecycle operations in a production workflow engine.
- [Azure Durable Functions code constraints](https://learn.microsoft.com/azure/azure-functions/durable/durable-functions-code-constraints)  
  Why it matters: secondary comparator reference for deterministic-orchestrator constraints that
  matter for any replay-based engine.

## Research Findings That Matter

- The March 28 durable-workflow transfer brief says the strongest transfer is event history as the
  semantic source of truth, not copying Temporal's product shape.
- The same brief is explicit that broker deduplication is not enough; effectively-once outcomes
  still need idempotent activities and durable intent/effect bridging.
- Lifecycle operations should be first-class runtime semantics, not ad hoc side behavior.
- History compaction and version-safe replay are mandatory if Mister Smith broadens durable
  workflows.

## Best-Practice Guidance

- Treat workflow event history as the canonical durable source of truth.
- Treat side effects as activity boundaries with explicit idempotency keys.
- Separate "exactly-once state transition" claims from "effectively-once outcome" claims.
- Add explicit lifecycle semantics for pause, resume, cancel, terminate, and reset/rewind.
- Design history compaction up front. Do not let replay cost grow without a bound.
- Add replay-regression fixtures before widening the durable surface.
- Let packet `022` own durable semantics and lifecycle verbs. Later packets like `023` should
  project those semantics, not redefine them.

## Likely Architecture Shape

- one workflow-history stream or equivalent append-only event log per workflow
- one effect/outbox seam for side-effect intent and completion
- workflow state machine projections for task, session, and autonomy views
- lifecycle commands that act on durable workflow state, not only transient process state
- compaction strategy that keeps old proof artifacts while allowing active execution to continue

## Risks / Constraints / Non-Goals

- Do not turn this into a full Temporal clone.
- Do not claim JetStream deduplication alone solves exactly-once outcomes.
- Do not widen into CRIU-style snapshots or opaque process images.
- Do not mix this packet with generic coordinator-runtime or federation work.
- Do not break the current happy path while introducing stronger durability semantics.

## Open Questions Before Spec Writing

- What is the repo-native event-history record shape?
- Where should the outbox/inbox boundary live across PostgreSQL and JetStream?
- What is the exact lifecycle vocabulary exposed on task, session, and autonomy surfaces?
- What is the first compaction mechanism: rollup event, snapshot record, KV pointer, or hybrid?
- How should version-safe replay be tested in CI?

## Fixed Constraints Before Spec Writing

- Keep packet `022` about event-history semantics, lifecycle verbs, and effect boundaries. Do not
  absorb coordinator-runtime, interop, or strong-coordination scope here.
- Preserve current session continuity and restart/resume behavior instead of redefining it from
  scratch.
- Separate durable state-transition guarantees from effect correctness. Do not let JetStream dedup
  stand in for effectively-once outcomes.
- Treat Temporal and Durable Functions as semantic comparators, not as the target product shape.

## Recommended Inputs For Future SpecKit Packet

Read these in order: repo routers -> durable-workflow brief -> current proof notes -> code seams ->
official comparators.

- `docs/direction.md`
- `docs/current-state.md`
- `docs/research-output/analysis/2026-03-28-durable-workflows-transfer-brief.md`
- `docs/plans/2026-03-19-session-restart-resume-live-proof.md`
  - use as the current restart/resume proof note, not as a durable-workflow contract by itself
- `crates/mister-smith-agents/src/branch_checkpoint.rs`
  - start from `BranchCheckpointStore`, `RepositoryBranchCheckpointStore`,
    `resume_branch_with_delegation`, and `reassign_branch_with_delegation`
- `crates/mister-smith-agents/src/execution_graph.rs`
  - start from `BranchCheckpoint` and `ExecutionGraph.checkpoint_lineage`
- `crates/mister-smith-agents/src/orchestrator.rs`
  - start from `record_branch_checkpoint`, `resume_branch_with_delegation`, and
    `reassign_branch_with_delegation`
- `crates/mister-smith-persistence/src/kv/state.rs`
  - start from `StateManager::update`, `branch_checkpoint_key`, and
    `branch_resume_history_key`
- `crates/mister-smith-persistence/src/hybrid/manager.rs`
  - start from branch checkpoint and resume-history read/write helpers
- `crates/mister-smith-persistence/src/repository/task.rs`
  - start from `TaskRepository::save_branch_checkpoint`,
    `TaskRepository::save_branch_resume_history`, and the branch-recovery metadata tests
- `crates/mister-smith-agents/tests/gate10_tests.rs`
  - start from the delegated resume and operator-visibility tests before widening lifecycle claims
- `crates/mister-smith-app/src/conversation.rs`
  - start from `ConversationRuntimeService` and persisted session-turn continuity
- only after the repo seams above are clear, re-confirm the official docs and primary sources
  linked earlier
