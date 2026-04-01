# Tasks: Selective Strong Coordination

**Input**: Design documents from `/specs/028-selective-strong-coordination/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`,
`contracts/`

**Tests**: Included. The first phase validates the scaffold packet itself. Later phases describe
the future implementation work that remains blocked until revalidation passes.

**Organization**: Tasks are grouped by bounded checkpoints so the pre-implementation revalidation
gate lands first, then later work can reuse the taxonomy, decision rule, and primitive without
widening packet scope.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel only when every blocking checkpoint in the current section is
  already complete and the write set is disjoint
- **[Story]**: Which user story the task advances (`US1` through `US3`)
- Include exact file paths in descriptions

## Phase 1: Pre-Implementation Revalidation (Blocking Gate)

**Purpose**: Reconfirm that the scaffold still matches current repo truth before any code task
starts.

**⚠️ CRITICAL**: No later task may begin until this phase is complete.

- [ ] T001 Re-read current truth and sequencing sources in `docs/direction.md` and `docs/current-state.md`
- [ ] T002 Re-check packet dependency seams in `docs/current-state.md`,
      `specs/023-runtime-truth-and-run-trace/spec.md`,
      `docs/research-output/analysis/2026-03-28-coordination-state-protocol-transfer-brief.md`,
      and `docs/research-output/consolidated/05-coordination-and-state.md`
- [ ] T003 Refresh `specs/028-selective-strong-coordination/spec.md` if truth-status,
      dependency-gate, or non-goal wording drifted
- [ ] T004 Refresh `specs/028-selective-strong-coordination/plan.md`,
      `specs/028-selective-strong-coordination/research.md`, and
      `specs/028-selective-strong-coordination/data-model.md` if the scaffold boundary changed
- [ ] T005 Re-run consistency review for `specs/028-selective-strong-coordination/analyze.md`
      after the revalidation pass

**Checkpoint**: Packet `028` is revalidated against the then-current state of earlier packet work.

## Phase 2: Foundational Freeze (Blocking Prerequisites)

**Purpose**: Lock the packet-owned taxonomy, contract, and examples before any code-facing work.

**⚠️ CRITICAL**: No user story work may begin until this phase is complete.

- [ ] T006 Freeze the packet-owned taxonomy in
      `specs/028-selective-strong-coordination/spec.md`,
      `specs/028-selective-strong-coordination/data-model.md`, and
      `specs/028-selective-strong-coordination/contracts/selective-strong-coordination-contract.md`
- [ ] T007 [P] Add representative state examples and coordination rationale in
      `specs/028-selective-strong-coordination/research.md`
- [ ] T008 [P] Refresh the packet-quality checks in
      `specs/028-selective-strong-coordination/checklists/requirements.md` and
      `specs/028-selective-strong-coordination/checklists/coordination.md`

**Checkpoint**: The packet-owned taxonomy and contract are frozen before code work begins.

## Phase 3: User Story 1 - Classify Shared State By Coordination Need (Priority: P1) 🎯 MVP

**Goal**: Add a canonical shared-state taxonomy to the future runtime seams.

**Independent Test**: Representative persistence and transport state surfaces can be mapped to one
taxonomy class without overlap.

### Tests for User Story 1

- [ ] T009 [P] [US1] Add classification coverage in
      `crates/mister-smith-persistence/tests/hybrid_tests.rs` and
      `crates/mister-smith-integration-tests/tests/transport_e2e.rs`

### Implementation for User Story 1

- [ ] T010 [P] [US1] Add shared taxonomy types in
      `crates/mister-smith-core/src/autonomy.rs` and `crates/mister-smith-core/src/lib.rs`
- [ ] T011 [P] [US1] Map current strict-state routing seams to the taxonomy in
      `crates/mister-smith-persistence/src/hybrid/router.rs`
- [ ] T012 [US1] Map current durable effect seams to the taxonomy in
      `crates/mister-smith-transport/src/durable.rs` and
      `crates/mister-smith-transport/src/envelope.rs`

**Checkpoint**: Future implementation has one canonical taxonomy for current repo-owned state.

## Phase 4: User Story 2 - Choose Strong Coordination Only When An Invariant Requires It (Priority: P1)

**Goal**: Make the invariant-driven coordination rule executable against current strict-state seams.

**Independent Test**: Representative invariant cases produce the expected coordination posture
without forcing strict coordination everywhere.

### Tests for User Story 2

- [ ] T013 [P] [US2] Add invariant-decision coverage in
      `crates/mister-smith-persistence/tests/kv_tests.rs`

### Implementation for User Story 2

- [ ] T014 [P] [US2] Add invariant metadata and decision-rule helpers in
      `crates/mister-smith-core/src/autonomy.rs`
- [ ] T015 [P] [US2] Wire coordinated-versus-convergent selection into
      `crates/mister-smith-persistence/src/kv/state.rs`
- [ ] T016 [US2] Preserve effect-path exclusions in
      `crates/mister-smith-transport/src/durable.rs` and
      `crates/mister-smith-transport/src/subject.rs`

**Checkpoint**: Future implementation can choose stronger coordination by invariant need, not by
default habit.

## Phase 5: User Story 3 - Reuse One Strong-Coordination Primitive Without Widening Packet Scope (Priority: P2)

**Goal**: Introduce `InvariantCell` as the only reusable primitive in the first slice.

**Independent Test**: `InvariantCell` can be explained, tested, and bounded without adding more
packet-owned primitives or turning on protocol safety work.

### Tests for User Story 3

- [ ] T017 [P] [US3] Add `InvariantCell` contract coverage in
      `crates/mister-smith-persistence/tests/kv_tests.rs`
- [ ] T018 [P] [US3] Add protocol-seam gate coverage in
      `specs/028-selective-strong-coordination/analyze.md` and
      `specs/028-selective-strong-coordination/contracts/selective-strong-coordination-contract.md`

### Implementation for User Story 3

- [ ] T019 [P] [US3] Add `InvariantCell` value objects in
      `crates/mister-smith-core/src/autonomy.rs` and `crates/mister-smith-core/src/lib.rs`
- [ ] T020 [P] [US3] Implement `InvariantCell` CAS behavior in
      `crates/mister-smith-persistence/src/kv/state.rs`
- [ ] T021 [US3] Keep protocol safety deferred unless the packet `027` seam check passes by
      refreshing `specs/028-selective-strong-coordination/spec.md`,
      `specs/028-selective-strong-coordination/plan.md`, and
      `specs/028-selective-strong-coordination/contracts/selective-strong-coordination-contract.md`

**Checkpoint**: Future implementation gets one reusable primitive without widening packet scope.

## Phase 6: Polish & Cross-Cutting Concerns

- [ ] T022 [P] Refresh packet-proof wording in
      `specs/028-selective-strong-coordination/spec.md`,
      `specs/028-selective-strong-coordination/quickstart.md`, and
      `specs/028-selective-strong-coordination/analyze.md`
- [ ] T023 [P] Run `git diff --check`
- [ ] T024 [P] Run `npx markdownlint-cli2 \"specs/028-selective-strong-coordination/**/*.md\" --config .markdownlint.json`
- [ ] T025 [P] Re-run packet analysis and save the result in
      `specs/028-selective-strong-coordination/analyze.md`

## Dependencies & Execution Order

### Phase Dependencies

- **Pre-Implementation Revalidation (Phase 1)**: No dependencies, starts first and blocks
  everything else
- **Foundational Freeze (Phase 2)**: Depends on Phase 1 completion
- **User Stories (Phase 3+)**: Depend on Phase 2 completion
- **Polish (Phase 6)**: Depends on all desired user story work being complete

### User Story Dependencies

- **User Story 1 (P1)**: Starts after Phase 2 and has no dependency on later stories
- **User Story 2 (P1)**: Starts after Phase 2 and reuses the taxonomy from User Story 1
- **User Story 3 (P2)**: Starts after Phase 2 and depends on the decision rule from User Story 2

### Parallel Opportunities

- Phase 2 tasks marked `[P]` can run in parallel after T006
- User Story 1 tasks T010 and T011 can run in parallel
- User Story 2 tasks T014 and T015 can run in parallel
- User Story 3 tasks T017, T018, and T019 can run in parallel when the shared write sets are
  disjoint

## Implementation Strategy

### Scaffold First

1. complete Phase 1 revalidation
2. complete Phase 2 foundational freeze
3. stop and confirm the packet still belongs in later-gated scope

### MVP First

1. complete User Story 1 after the revalidation gate passes
2. validate that the taxonomy alone is still coherent
3. continue to the decision rule and `InvariantCell` only if scope remains stable

## Notes

- No code task should start before the pre-implementation revalidation phase is complete.
- This task pack is scaffold-owned. It may need revision when upstream packet work settles.
