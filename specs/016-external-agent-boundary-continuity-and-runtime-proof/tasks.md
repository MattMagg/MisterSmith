# Tasks: External-Agent Boundary Continuity And Runtime Proof

**Input**: Design documents from
`/specs/016-external-agent-boundary-continuity-and-runtime-proof/`  
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`

**Tests**: Included. This packet requires deterministic delegated-ingress and rejection coverage,
workflow-level autonomy-status coverage, CLI parity checks, one accepted live ingress proof note,
and `cargo build --workspace`.

**Organization**: Tasks are grouped by blocking scope freezes first, then bounded lanes for task
ingress continuity, workflow-level projection, and proof.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel only when every blocking checkpoint in the current section is
  already landed and the write set is disjoint from every other active lane
- **[Story]**: Which user story the task advances (`US1` through `US3`)
- Include exact file paths in every task description

## Path Conventions

- **HTTP ingress**: `crates/mister-smith-http/src/`
- **App source**: `crates/mister-smith-app/src/`
- **App tests**: `crates/mister-smith-app/tests/`
- **Events**: `crates/mister-smith-events/src/`, `crates/mister-smith-events/tests/`
- **Evidence artifacts**: `docs/plans/`

## Status Reconciliation (2026-03-20)

- packet `015` and `MS-95` are complete on `main`
- `MS-77` already landed the bounded MCP discovery and enforcement surface
- raw delegated ingress context is already persisted as `external_delegation`
- metadata-only delegated context must not fabricate an accepted or rejected operator-visible
  boundary decision
- this task pack must close only the accepted delegated HTTP task-ingress continuity gap

---

## Subphase 16.1 - Scope And Decision-Surface Freeze (Blocking Prerequisites)

**Goal**: freeze the exact packet-016 gap before implementation lanes begin.

**⚠️ CRITICAL**: no `[P]` lane may begin until this checkpoint is complete.

This checkpoint was completed during packet creation on 2026-03-20 through the `MS-96`
pre-spec decision note and the landed packet artifact set.

- [x] T001 [US1] Freeze the packet around `POST /api/v1/tasks`,
  `GET /api/v1/autonomy/status/{workflow_id}`, and CLI parity in
  `specs/016-external-agent-boundary-continuity-and-runtime-proof/spec.md` and
  `specs/016-external-agent-boundary-continuity-and-runtime-proof/plan.md`.
- [x] T002 [US1] Freeze the data-model mapping between raw metadata `external_delegation`,
  preferred reuse of `external_capability_decisions`, and retained session continuity rules in
  `specs/016-external-agent-boundary-continuity-and-runtime-proof/data-model.md`.
- [x] T003 [US1] Decide whether the current operator-visible summary needs a backward-compatible
  discriminator; if not, record “no new contract doc required” in
  `specs/016-external-agent-boundary-continuity-and-runtime-proof/research.md`.

**Checkpoint**: the packet is frozen around one accepted delegated task-ingress path and one
workflow-level inspection path.

---

## User Story 1 - Persist Accepted Delegated HTTP Task-Ingress Continuity (Priority: P1)

**Goal**: preserve enough accepted-ingress continuity through workflow metadata for later
operator-visible projection.

**Independent Test**: submit one accepted delegated task request and verify the workflow record
retains the right continuity material without fabricating a decision from raw metadata alone.

### Tests For User Story 1

- [ ] T004 [P] [US1] Extend delegated task-ingress coverage in
  `crates/mister-smith-http/src/handlers.rs`.
- [ ] T005 [P] [US1] Extend metadata persistence and no-fabrication coverage in
  `crates/mister-smith-app/src/execution.rs`.

### Implementation For User Story 1

- [ ] T006 [P] [US1] Extend accepted task-ingress metadata continuity in
  `crates/mister-smith-app/src/execution.rs`.
- [ ] T007 [US1] Keep `POST /api/v1/tasks` delegated-ingress forwarding aligned with the bounded
  packet scope in `crates/mister-smith-http/src/handlers.rs`.

**Checkpoint**: accepted delegated task ingress leaves enough persisted workflow evidence for
workflow-level proof without weakening the no-fabrication rule.

---

## User Story 2 - Project Accepted Ingress Continuity On Workflow-Level Inspection (Priority: P1)

**Goal**: expose one first-class accepted boundary decision on workflow-level autonomy inspection
and CLI parity.

**Independent Test**: inspect one accepted delegated workflow through the active HTTP status route
and CLI and verify the same accepted decision is visible on both surfaces.

### Tests For User Story 2

- [ ] T008 [P] [US2] Extend workflow-level status rendering and CLI parity coverage in
  `crates/mister-smith-app/tests/autonomy_status_tests.rs`.
- [ ] T009 [P] [US2] Extend projection coverage in
  `crates/mister-smith-events/tests/autonomy_event_tests.rs`.

### Implementation For User Story 2

- [ ] T010 [P] [US2] Project accepted ingress continuity on the workflow-level autonomy status
  surface in `crates/mister-smith-app/src/autonomy.rs` and `crates/mister-smith-app/src/bootstrap.rs`.
- [ ] T011 [P] [US2] Reuse `external_capability_decisions` if possible, or add the smallest
  backward-compatible discriminator only if implementation proves it is necessary in
  `crates/mister-smith-events/src/autonomy.rs` and `crates/mister-smith-events/src/bus.rs`.
- [ ] T012 [US2] Preserve retained session continuity rules in
  `crates/mister-smith-app/src/conversation.rs` without widening the packet to delegated session
  ingress proof.

**Checkpoint**: accepted task-ingress continuity is visible on workflow-level autonomy status and
CLI parity without relabeling task or session views as autonomy-status surfaces.

---

## User Story 3 - Prove Accepted Live Ingress And Keep Rejection Proof Deterministic (Priority: P2)

**Goal**: capture one honest accepted live proof run and keep rejection proof bounded.

**Independent Test**: run one accepted delegated `POST /api/v1/tasks` request, capture the
returned `workflow_id`, inspect workflow-level status and CLI parity, and separately run
deterministic rejection tests.

### Tests For User Story 3

- [ ] T013 [P] [US3] Extend deterministic delegated rejection coverage in
  `crates/mister-smith-http/src/server.rs`.
- [ ] T014 [P] [US3] Extend accepted-ingress continuity coverage in
  `crates/mister-smith-app/tests/autonomy_status_tests.rs`.

### Implementation For User Story 3

- [ ] T015 [P] [US3] Capture the accepted live proof artifact in
  `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`.
- [ ] T016 [US3] Explicitly record whether a workflow-backed reject surface exists; if not, leave
  live rejection proof out of scope in
  `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`.

**Checkpoint**: one accepted live ingress proof exists, and rejection coverage remains honest and
deterministic.

---

## Final Validation And Evidence

- [ ] T017 Run targeted `cargo test` for touched HTTP, app, and event coverage
- [ ] T018 Run `cargo build --workspace`
- [ ] T019 Capture the durable proof artifact in
  `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`

## Parallel Symphony Directive

`[P]` means a task may run in parallel only when:

- every blocking checkpoint task in the current section is already landed
- its write set is disjoint from every other active lane

Shared-write choke points for this packet:

- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- `crates/mister-smith-events/src/autonomy.rs`
- `crates/mister-smith-events/src/bus.rs`
- the active packet-016 evaluation note

Only one implementation lane may own a choke-point file at a time.

Allowed concurrent lanes after `T001` through `T003`:

- delegated-ingress lane:
  `T004`, `T005`, `T006`, `T007`
- projection lane:
  `T008`, `T009`, `T010`, `T011`, `T012`
- proof lane:
  `T013`, `T014`, `T015`, `T016`

Serial merge points:

- `T006` remains serial because it reopens `crates/mister-smith-app/src/execution.rs`
- `T011` remains serial if it changes shared event summary typing
- `T015` and `T016` remain single-owner because the evaluation note is a choke point
