# Tasks: Complex Multi-Agent Proof and Unified Result Surfaces

**Input**: Design documents from
`/specs/015-complex-multi-agent-proof-and-unified-result-surfaces/`  
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`,
`contracts/`

**Tests**: Included. This packet requires targeted agents, events, and app tests, a durable proof
artifact, and workspace compile verification.

**Organization**: Tasks are grouped by blocking contract checkpoints first, then bounded execution
lanes for runtime proof, result projection, and evaluation.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel only when every blocking checkpoint in the current section is
  already landed and the write set is disjoint from every other active lane
- **[Story]**: Which user story the task advances (`US1` through `US4`)
- Include exact file paths in every task description

## Path Conventions

- **Shared types**: `crates/mister-smith-core/src/`, `crates/mister-smith-core/tests/`
- **Events**: `crates/mister-smith-events/src/`, `crates/mister-smith-events/tests/`
- **App source**: `crates/mister-smith-app/src/`
- **App tests**: `crates/mister-smith-app/tests/`
- **Agents source**: `crates/mister-smith-agents/src/`
- **Agents tests**: `crates/mister-smith-agents/tests/`
- **Evidence artifacts**: `docs/plans/`

## Status Reconciliation (2026-03-19)

- the March 19 checkpoint is the governing direction for this packet
- supervised planner and executor live path, `tool_bus` execution boundary, topology and routing
  visibility, bounded MCP capability discovery and enforcement, and existing result plumbing are
  already landed on `main`
- this task pack must unify and project that baseline rather than reopen it

---

## Subphase 15.1 - Shared Result Contract Freeze (Blocking Prerequisites)

**Goal**: define one stable result contract and one stable proof-outcome taxonomy before any
parallel lane begins.

**⚠️ CRITICAL**: no `[P]` lane may begin until this checkpoint is complete.

- [ ] T001 [US2] Add shared result and proof-outcome value objects in
  `crates/mister-smith-core/src/autonomy.rs` and re-export them from
  `crates/mister-smith-core/src/lib.rs`.
- [ ] T002 [US2] Extend typed operator-facing result summaries in
  `crates/mister-smith-events/src/autonomy.rs`.
- [ ] T003 [US2] Extend result preview and proof-outcome aggregation in
  `crates/mister-smith-events/src/bus.rs`.
- [ ] T004 [US2] Align the future implementation contract with
  `specs/015-complex-multi-agent-proof-and-unified-result-surfaces/contracts/result-surface-contract.md`.
- [ ] T005 [US2] Add shared contract coverage in
  `crates/mister-smith-core/tests/trait_compilation_tests.rs` and
  `crates/mister-smith-events/tests/autonomy_event_tests.rs`.

**Checkpoint**: the relationship between `task.result`, metadata `final_result`, nested
`aggregated_result`, session `assistant_result`, and operator preview/provenance is frozen once.

---

## Subphase 15.2 - Proof Outcome Matrix Freeze (Blocking Prerequisites)

**Goal**: define the runtime proof matrix before changing the proof path.

**⚠️ CRITICAL**: runtime work must not start until the proof classes are frozen.

- [ ] T006 [US4] Freeze proof outcome classification in
  `crates/mister-smith-core/src/autonomy.rs` and
  `crates/mister-smith-events/src/autonomy.rs`.
- [ ] T007 [US4] Add proof outcome aggregation coverage in
  `crates/mister-smith-events/tests/autonomy_event_tests.rs`.
- [ ] T008 [US4] Capture the expected success, collapse, and failure-visible matrix in
  `docs/plans/2026-03-19-complex-multi-agent-proof-and-unified-result-surfaces-evaluation.md`.

**Checkpoint**: the packet now has one frozen taxonomy:
`graph_formed_and_completed`, `collapsed_to_sequential`, `failed_before_graph`.

---

## User Story 1 - Harder Workload Graph Proof On The Default Path (Priority: P1)

**Goal**: prove harder workloads honestly on the live path when the planner can support them.

**Independent Test**: run representative harder workloads and verify success, collapse, and
failure-visible cases are distinguishable from stored runtime evidence.

### Tests For User Story 1

- [ ] T009 [P] [US1] Add harder-workload proof fixtures in
  `crates/mister-smith-agents/tests/step_routing_benchmark_tests.rs` and
  `crates/mister-smith-agents/tests/team_sizing_benchmark_tests.rs`.
- [ ] T010 [P] [US1] Extend default-path guard coverage in
  `crates/mister-smith-agents/tests/gate10_tests.rs`.

### Implementation For User Story 1

- [ ] T011 [P] [US1] Extend runtime proof-path assembly in
  `crates/mister-smith-app/src/execution.rs`.
- [ ] T012 [P] [US1] Extend graph and proof-outcome assembly in
  `crates/mister-smith-agents/src/orchestrator.rs`.
- [ ] T013 [US1] Preserve one-shot task runtime compatibility while exposing harder-workload proof
  state in `crates/mister-smith-app/src/execution.rs`.

**Checkpoint**: the default path can prove a harder workload outcome honestly without widening into
an unrelated runtime program.

---

## User Story 2 - Unified Result Contract Across Task And Session Views (Priority: P1)

**Goal**: make task and session result surfaces depend on the same canonical result contract.

**Independent Test**: inspect one completed workflow through task and session surfaces and verify
they map back to the same canonical result object.

### Tests For User Story 2

- [ ] T014 [P] [US2] Extend task and operator result rendering coverage in
  `crates/mister-smith-app/tests/autonomy_status_tests.rs`.
- [ ] T015 [P] [US2] Extend session retained-result coverage in
  `crates/mister-smith-app/src/conversation.rs`.

### Implementation For User Story 2

- [ ] T016 [P] [US2] Project the canonical task-facing result envelope in
  `crates/mister-smith-app/src/execution.rs`.
- [ ] T017 [P] [US2] Project retained session result views in
  `crates/mister-smith-app/src/conversation.rs`.
- [ ] T018 [US2] Keep task and session result mapping aligned with
  `specs/015-complex-multi-agent-proof-and-unified-result-surfaces/contracts/result-surface-contract.md`.

**Checkpoint**: task and session surfaces expose one result contract rather than competing result
shapes.

---

## User Story 3 - Operator Preview And Provenance (Priority: P2)

**Goal**: expose enough bounded result preview and provenance to verify behavior without dumping
full payloads.

**Independent Test**: inspect a completed workflow through autonomy status and verify the rendered
preview and provenance map back to the canonical result object.

### Tests For User Story 3

- [ ] T019 [P] [US3] Extend result preview and provenance event coverage in
  `crates/mister-smith-events/tests/autonomy_event_tests.rs`.
- [ ] T020 [P] [US3] Extend operator rendering coverage in
  `crates/mister-smith-app/tests/autonomy_status_tests.rs`.

### Implementation For User Story 3

- [ ] T021 [P] [US3] Extend operator-facing result summaries in
  `crates/mister-smith-events/src/autonomy.rs` and
  `crates/mister-smith-events/src/bus.rs`.
- [ ] T022 [P] [US3] Render bounded result preview and provenance in
  `crates/mister-smith-app/src/autonomy.rs`.

**Checkpoint**: operator inspection can verify result behavior without reconstructing it from raw
logs or full payload dumps.

---

## User Story 4 - Visible Collapse And Failure-Visible Classification (Priority: P2)

**Goal**: keep success, collapse, and failure-visible boundaries inspectable and classifiable.

**Independent Test**: run or replay the proof matrix and verify each case lands in one explicit
proof outcome class.

### Tests For User Story 4

- [ ] T023 [P] [US4] Extend proof-outcome matrix coverage in
  `crates/mister-smith-agents/tests/step_routing_benchmark_tests.rs`.
- [ ] T024 [P] [US4] Extend proof-outcome rendering coverage in
  `crates/mister-smith-app/tests/autonomy_status_tests.rs`.

### Implementation For User Story 4

- [ ] T025 [P] [US4] Persist proof outcome classification through the runtime result envelope in
  `crates/mister-smith-app/src/execution.rs`.
- [ ] T026 [P] [US4] Preserve proof outcome visibility in operator status and retained session
  views in `crates/mister-smith-app/src/autonomy.rs` and
  `crates/mister-smith-app/src/conversation.rs`.

**Checkpoint**: proof review can distinguish successful graph execution, visible collapse, and
failure before graph formation from stored evidence.

---

## Final Validation And Evidence

- [ ] T027 Run `cargo test -p mister-smith-agents`
- [ ] T028 Run `cargo test -p mister-smith-events`
- [ ] T029 Run `cargo test -p mister-smith-app`
- [ ] T030 Run `cargo build --workspace`
- [ ] T031 Capture the durable proof artifact in
  `docs/plans/2026-03-19-complex-multi-agent-proof-and-unified-result-surfaces-evaluation.md`
- [ ] T032 Run an MCP non-regression check only if the touched result surfaces intersect the
  bounded post-`MS-77` capability path

## Parallel Symphony Directive

`[P]` means a task may run in parallel only when:

- every blocking checkpoint task in the current section is already landed
- its write set is disjoint from every other active lane

Shared-write choke points for this packet:

- `crates/mister-smith-core/src/autonomy.rs`
- `crates/mister-smith-core/src/lib.rs`
- `crates/mister-smith-events/src/autonomy.rs`
- `crates/mister-smith-events/src/bus.rs`
- `crates/mister-smith-app/src/execution.rs`
- `specs/015-complex-multi-agent-proof-and-unified-result-surfaces/contracts/result-surface-contract.md`
- the active `docs/plans/2026-03-19-complex-multi-agent-proof-and-unified-result-surfaces-evaluation.md`

Only one Symphony run may own a choke-point file at a time.

Allowed concurrent lanes after `T001` through `T008`:

- runtime proof-path lane:
  `T009`, `T010`, `T011`, `T012`
- result projection lane:
  `T014`, `T015`, `T016`, `T017`, `T019`, `T020`, `T021`, `T022`
- classification and evidence lane:
  `T023`, `T024`, `T031`

Serial merge points:

- `T013` must remain serial because it reopens `crates/mister-smith-app/src/execution.rs`
- `T018` must remain serial because the contract file is single-owner
- `T025` must remain serial because it reopens the canonical runtime result envelope
- `T026` must remain serial because it spans app autonomy and conversation projections
- `T032` must remain serial because the external capability check is a single bounded decision

## Implementation Strategy

### MVP For Remaining Work

1. complete `T001` through `T008` to freeze the result contract and proof taxonomy
2. complete `T009` through `T013` for harder-workload proof on the default path
3. complete `T014` through `T026` for consistent result projection and outcome visibility
4. capture durable evidence and run cross-crate safety checks

### Incremental Delivery

1. preserve March 19 baseline truth
2. freeze the shared result contract
3. freeze the proof-outcome taxonomy
4. run the harder-workload proof lane
5. run the result projection lane
6. capture durable evidence and bounded non-regression checks

## Explicitly Out Of Scope For This Packet

- reopening provider-neutral routing work
- reopening KV or budget-control work
- broadening into external-agent interoperability beyond a bounded non-regression check
- adding a new operator subsystem just for result inspection
