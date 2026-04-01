# Tasks: Capability Discovery And Interoperability

**Input**: Design documents from `specs/027-capability-discovery-and-interoperability/`  
**Prerequisites**: `plan.md`, `spec.md`, plus packet-local `research.md`, `data-model.md`,
`quickstart.md`, and both contracts under `contracts/`

**Tests**: Future implementation must keep deterministic packet validation separate from runtime
proof. Run targeted crate tests for discovery and lifecycle mapping, then `cargo build --workspace`,
`git diff --check`, packet-local markdown linting, and final closure checks only after the
blocking refresh gate is complete.

**Organization**: Group tasks by blocking refresh work first, then bounded user stories, then final
validation and evidence.

This tasks file is a future implementation scaffold only. Do not begin code work until `T001`
through `T003` refresh packet `027` against the completed packet `022`, `023`, and `024` outputs.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Use only when every blocking checkpoint in the current section is already complete and
  the write set is disjoint from every other active lane
- **[Story]**: Use `US1`, `US2`, and `US3` for story-bound work
- Include exact file paths in every implementation or documentation task

## Status Reconciliation

Capture the current repo truth this packet must preserve.

- Local capability discovery already exists in
  `crates/mister-smith-agents/src/tool_bus.rs`.
- Bounded MCP discovery and delegated boundary enforcement already exist in
  `crates/mister-smith-mcp/src/client.rs`, `crates/mister-smith-mcp/src/server.rs`, and
  `crates/mister-smith-mcp/src/compatibility.rs`.
- Packet `016` continuity and provenance baselines already exist in
  `specs/016-external-agent-boundary-continuity-and-runtime-proof/spec.md` and
  `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`.
- Packets `022`, `023`, and `024` are the current upstream packet baseline and must be rechecked
  again before any later implementation work starts.

---

## T1. Scope And Design Freeze (Blocking Prerequisites)

**Goal**: refresh packet `027` against finished upstream packet contracts before any implementation
lane begins.

**CRITICAL**: no `[P]` lane may begin until this checkpoint is complete.

- [ ] T001 Refresh `specs/027-capability-discovery-and-interoperability/spec.md` and
  `specs/027-capability-discovery-and-interoperability/plan.md` against the completed packet
  `022`, `023`, and `024` outputs before implementation starts
- [ ] T002 Refresh `specs/027-capability-discovery-and-interoperability/research.md`,
  `specs/027-capability-discovery-and-interoperability/data-model.md`,
  `specs/027-capability-discovery-and-interoperability/contracts/capability-normalization-contract.md`,
  and
  `specs/027-capability-discovery-and-interoperability/contracts/a2a-lifecycle-mapping-contract.md`
  against the final upstream packet contracts
- [ ] T003 Re-run the scope and boundary checks in
  `specs/027-capability-discovery-and-interoperability/checklists/interop.md` and
  `specs/027-capability-discovery-and-interoperability/quickstart.md` so MCP `2025-11-25`, A2A
  `v0.3.0`, packet `016` continuity wording, and explicit deferrals stay frozen

**Checkpoint**: packet `027` is refreshed against the final upstream contracts and still freezes
one bounded interoperability scaffold only.

---

## User Story 1 - Normalize Capability Discovery (Priority: P1)

**Goal**: implement one shared descriptor model across local ToolBus, MCP, and one A2A discovery
input without turning discovery into execution permission.

**Independent Test**: targeted tests prove local ToolBus, MCP, and A2A inputs normalize into the
same descriptor shape while permission and trust remain separate fields.

### Tests For User Story 1

- [ ] T004 [P] [US1] Add descriptor normalization coverage in
  `crates/mister-smith-agents/tests/tool_bus_tests.rs`
- [ ] T005 [P] [US1] Add MCP and A2A normalization coverage in
  `crates/mister-smith-mcp/tests/capability_normalization_tests.rs`

### Implementation For User Story 1

- [ ] T006 [P] [US1] Add or extend normalized local capability projection in
  `crates/mister-smith-agents/src/tool_bus.rs`
- [ ] T007 [P] [US1] Add normalized remote capability parsing for MCP and A2A discovery inputs in
  `crates/mister-smith-mcp/src/client.rs`
- [ ] T008 [US1] Preserve discovery-versus-execution separation and explicit permission references
  in `crates/mister-smith-mcp/src/server.rs` and
  `crates/mister-smith-mcp/src/compatibility.rs`

**Checkpoint**: local ToolBus, MCP, and A2A discovery inputs share one normalized descriptor model
without granting execution authority by discovery alone.

---

## User Story 2 - Map One Remote Lifecycle Model (Priority: P1)

**Goal**: map one A2A `v0.3.0` task lifecycle into Mister Smith workflow, result, and autonomy
surfaces without widening into generic federation or live remote proof claims.

**Independent Test**: targeted tests prove the A2A discovery adapter and lifecycle bridge map
known states into Mister Smith workflow, result, and autonomy projections, with explicit handling
for unsupported or boundary-only states.

### Tests For User Story 2

- [ ] T009 [P] [US2] Add A2A discovery adapter coverage in
  `crates/mister-smith-mcp/tests/a2a_discovery_tests.rs`
- [ ] T010 [P] [US2] Add lifecycle projection coverage in
  `crates/mister-smith-app/tests/a2a_lifecycle_projection_tests.rs`

### A2A Discovery Mapping

- [ ] T011 [P] [US2] Implement the first A2A discovery adapter in
  `crates/mister-smith-mcp/src/a2a.rs`
- [ ] T012 [US2] Thread A2A discovery normalization through
  `crates/mister-smith-mcp/src/compatibility.rs`

### A2A Lifecycle Mapping

- [ ] T013 [P] [US2] Implement the A2A-to-workflow lifecycle binding in
  `crates/mister-smith-core/src/autonomy.rs`
- [ ] T014 [P] [US2] Project lifecycle events onto the event bus in
  `crates/mister-smith-events/src/bus.rs`
- [ ] T015 [US2] Surface mapped lifecycle state in operator-facing autonomy and result views in
  `crates/mister-smith-app/src/autonomy.rs`

**Checkpoint**: one A2A `v0.3.0` task lifecycle is mapped into Mister Smith workflow, result, and
autonomy surfaces with explicit unsupported-state handling and no generic federation claims.

---

## User Story 3 - Preserve Provenance And Scope Boundaries (Priority: P2)

**Goal**: keep operator-visible provenance, packet `016` continuity wording, and discovery-versus-
execute boundaries explicit while recording the first interop slice honestly.

**Independent Test**: packet-local docs and proof notes show where remote capability information
came from, which lifecycle binding applied, what remains deferred, and what packet `027` still
does not claim.

### Tests For User Story 3

- [ ] T016 [P] [US3] Add provenance and boundary regression coverage in
  `crates/mister-smith-core/tests/remote_capability_provenance_tests.rs`
- [ ] T017 [P] [US3] Add operator-surface provenance coverage in
  `crates/mister-smith-app/tests/remote_capability_status_tests.rs`

### Operator And Proof-Boundary Projection

- [ ] T018 [P] [US3] Add operator-visible remote capability provenance projection in
  `crates/mister-smith-core/src/autonomy.rs`
- [ ] T019 [US3] Capture the bounded interop proof note in
  `docs/plans/2026-04-01-packet-027-capability-discovery-and-interoperability.md`
- [ ] T020 [US3] Refresh packet-local boundary language in
  `specs/027-capability-discovery-and-interoperability/quickstart.md` and
  `specs/027-capability-discovery-and-interoperability/spec.md` so packet `016` remains
  continuity and provenance only

**Checkpoint**: operator-facing provenance is explicit, packet `016` stays narrow, and packet
`027` still reads as one bounded interoperability scaffold.

---

## Final Validation And Evidence

- [ ] T021 Run targeted Rust validation for the touched lifecycle and discovery crates with
  `cargo test -p mister-smith-agents`, `cargo test -p mister-smith-mcp`,
  `cargo test -p mister-smith-core`, `cargo test -p mister-smith-events`, and
  `cargo test -p mister-smith-app`
- [ ] T022 Run broader compatibility checks with `cargo build --workspace` and
  `npx markdownlint-cli2 "specs/027-capability-discovery-and-interoperability/**/*.md" --config .markdownlint.json`
- [ ] T023 Refresh the durable proof note and any packet-local validation evidence in
  `docs/plans/2026-04-01-packet-027-capability-discovery-and-interoperability.md`
- [ ] T024 Run `git diff --check`
- [ ] T025 Run `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync`

## Dependencies And Execution Order

- Complete `T001` through `T003` before starting any user-story implementation work.
- Complete User Story 1 before User Story 2 so the shared descriptor shape is frozen before A2A
  lifecycle projection depends on it.
- Complete User Story 2 before User Story 3 so provenance and operator-facing projection reflect
  the final lifecycle mapping.
- Run `T021` through `T025` only after the story checkpoints are complete.

## Parallel Directive

`[P]` means a task may run in parallel only when:

- every blocking checkpoint task in the current section is already complete
- its write set is disjoint from every other active lane

Shared-write choke points for this packet:

- `crates/mister-smith-core/src/autonomy.rs`
- `crates/mister-smith-mcp/src/compatibility.rs`
- `specs/027-capability-discovery-and-interoperability/spec.md`
- `docs/plans/2026-04-01-packet-027-capability-discovery-and-interoperability.md`

Allowed concurrent lanes after the blocking freeze:

- descriptor normalization lane: `T004`, `T006`, and `T007` across
  `crates/mister-smith-agents/src/tool_bus.rs`,
  `crates/mister-smith-agents/tests/tool_bus_tests.rs`,
  `crates/mister-smith-mcp/src/client.rs`, and
  `crates/mister-smith-mcp/tests/capability_normalization_tests.rs`
- A2A lifecycle lane: `T009`, `T010`, `T011`, `T013`, and `T014` across
  `crates/mister-smith-mcp/src/a2a.rs`,
  `crates/mister-smith-mcp/tests/a2a_discovery_tests.rs`,
  `crates/mister-smith-app/tests/a2a_lifecycle_projection_tests.rs`,
  `crates/mister-smith-core/src/autonomy.rs`, and
  `crates/mister-smith-events/src/bus.rs`

Serial merge points:

- `T008` and `T012` in `crates/mister-smith-mcp/src/compatibility.rs`
- `T015` and `T018` in `crates/mister-smith-core/src/autonomy.rs` and
  `crates/mister-smith-app/src/autonomy.rs`
- `T019`, `T020`, and `T023` in the packet-local docs and proof note
