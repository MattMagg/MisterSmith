# Tasks: Phase 2 Runtime and Async Infrastructure Contracts

**Input**: Design documents from `/specs/002-phase2-runtime-async/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Evidence tasks are included as contract-consistency checks and lint validation.

**Organization**: Tasks are grouped by user story so each story is independently completable and reviewable.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: User story label (`US1`, `US2`, `US3`)
- Include exact file paths in each task description

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Initialize Phase 2 feature workspace and references.

- [x] T001 Confirm active feature paths with `./.specify/scripts/bash/check-prerequisites.sh --json --paths-only`
- [x] T002 Verify Phase 2 canonical anchors exist in
  `/Users/matthewmaggio/Mister-Smith/spec/core-architecture/`,
  `/Users/matthewmaggio/Mister-Smith/spec/data-management/`, and
  `/Users/matthewmaggio/Mister-Smith/spec/operations/`; confirm Phase 1 Gate 1 evidence in
  `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/spec.md` (`## Success Criteria`) and
  `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/quickstart.md`
- [x] T003 [P] Create/verify artifact layout under `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build shared contract-planning artifacts needed by all stories.

- [x] T004 Author Phase 2 implementation summary and technical context in `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/plan.md`
- [x] T005 [P] Consolidate planning decisions in `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/research.md`
- [x] T006 [P] Define shared contract entities in `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/data-model.md`
- [x] T007 [P] Define baseline runtime/async contracts in `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/contracts/phase2-runtime-async-contracts.md`
- [x] T008 [P] Capture reproducible validation workflow in `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/quickstart.md`
- [x] T009 Update Codex agent context via `./.specify/scripts/bash/update-agent-context.sh codex`

**Checkpoint**: Shared design artifacts are ready for story-specific evidence tasks.

---

## Phase 3: User Story 1 - Runtime Lifecycle Baseline (Priority: P1) 🎯 MVP

**Goal**: Establish explicit runtime lifecycle and shutdown contracts.

**Independent Test**: Runtime lifecycle contract checks are reproducible and mapped to requirements.

### Implementation for User Story 1

- [x] T010 [US1] Map FR-001, FR-002, and FR-008 in
  `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/spec.md` (`## Requirements`, `### User Story 1`)
  and record checklist evidence in
  `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/checklists/requirements.md`
  (CHK001, CHK008, CHK012)
- [x] T011 [P] [US1] Validate runtime lifecycle references with `rg` on
  `/Users/matthewmaggio/Mister-Smith/spec/core-architecture/tokio-runtime.md`
  (`### 1.2 Runtime Lifecycle Management`) and
  `/Users/matthewmaggio/Mister-Smith/spec/core-architecture/runtime-and-errors.md`
  (`### Runtime Lifecycle Management`)
- [x] T012 [P] [US1] Validate graceful-shutdown terminology across
  `/Users/matthewmaggio/Mister-Smith/spec/core-architecture/tokio-runtime.md`
  (`### 1.2 Runtime Lifecycle Management`) and
  `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/{spec.md,contracts/phase2-runtime-async-contracts.md}`
  (`## 1. Runtime Lifecycle Contracts`)
- [x] T013 [US1] Confirm implementation-free runtime boundaries in
  `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/spec.md` (`## Scope`, `## Clarifications`) and
  `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/plan.md` (`## Technical Context`)
  and record CHK005
- [x] T014 [US1] Confirm US1 acceptance scenarios map to validation commands in
  `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/spec.md`
  (`## User Scenarios & Testing`, `### Validation Command Set`) and record CHK011, CHK012

**Checkpoint**: Runtime lifecycle baseline is independently validated.

---

## Phase 4: User Story 2 - Monitoring and Event Contract Baseline (Priority: P2)

**Goal**: Establish consistent health, metrics, and event contracts.

**Independent Test**: Monitoring and event cross-reference checks are reproducible with no unresolved terminology drift.

### Implementation for User Story 2

- [x] T015 [US2] Map FR-003, FR-004, and FR-005 in
  `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/spec.md` (`## Requirements`, `### User Story 2`)
  and record checklist evidence in
  `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/checklists/requirements.md`
  (CHK002, CHK009, CHK012)
- [x] T016 [P] [US2] Validate health and metrics references with `rg` on
  `/Users/matthewmaggio/Mister-Smith/spec/core-architecture/monitoring-and-health.md`
  (`## Health Check System`, `## Metrics Collection`) and
  `/Users/matthewmaggio/Mister-Smith/spec/operations/observability-monitoring-framework.md`
  (`### 4. Metrics Collection Patterns`, `### 15.4 Health Check Endpoints`)
- [x] T017 [P] [US2] Validate event-bus references with `rg` on
  `/Users/matthewmaggio/Mister-Smith/spec/core-architecture/supervision-and-events.md`
  (`## Event System Implementation`, `### Event Bus Architecture`) and
  `/Users/matthewmaggio/Mister-Smith/spec/core-architecture/monitoring-and-health.md`
- [x] T018 [US2] Confirm active-vs-legacy terminology policy in
  `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/spec.md` (`## Clarifications`, FR-005) and
  `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/contracts/phase2-runtime-async-contracts.md`
  (`## 2. Monitoring and Event Contracts`) and record CHK010
- [x] T019 [US2] Confirm US2 acceptance scenarios map to validation commands in
  `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/spec.md`
  (`## User Scenarios & Testing`, `### Validation Command Set`) and record CHK011, CHK012

**Checkpoint**: Monitoring/event baseline is independently validated.

---

## Phase 5: User Story 3 - Async Utility and Resource Management Baseline (Priority: P3)

**Goal**: Establish reusable async and resource lifecycle contracts.

**Independent Test**: Async utility and resource checks are reproducible and edge-case coverage is explicit.

### Implementation for User Story 3

- [x] T020 [US3] Map FR-006, FR-007, and FR-010 in
  `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/spec.md` (`## Requirements`, `### User Story 3`)
  and record checklist evidence in
  `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/checklists/requirements.md`
  (CHK003, CHK004, CHK012)
- [x] T021 [P] [US3] Validate async utility references with `rg` on
  `/Users/matthewmaggio/Mister-Smith/spec/core-architecture/async-patterns.md` (`## Task Management Framework`) and
  `/Users/matthewmaggio/Mister-Smith/spec/core-architecture/module-organization-type-system.md`
  (`## 2. Core Trait Hierarchy and Type System`)
- [x] T022 [P] [US3] Validate resource lifecycle references with `rg` on
  `/Users/matthewmaggio/Mister-Smith/spec/data-management/connection-management.md`
  (`### 5.1 Enterprise Connection Pool Architecture`, `### 5.4 Distributed Transaction Coordination`) and
  `/Users/matthewmaggio/Mister-Smith/spec/core-architecture/component-architecture.md`
  (`## Resource Management`)
- [x] T023 [US3] Confirm bounded-resource and backpressure semantics in
  `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/spec.md`
  (`### Constitution Alignment Requirements`, `### Edge Cases`) and
  `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/contracts/phase2-runtime-async-contracts.md`
  (`## 3. Async Utility Contracts`, `## 4. Resource Lifecycle Contracts`) and record CHK006, CHK016
- [x] T024 [US3] Confirm overload/degradation/resource-exhaustion edge cases in
  `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/spec.md` (`### Edge Cases`)
  and record CHK013, CHK014, CHK015

**Checkpoint**: Async/resource baseline is independently validated.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final traceability, consistency, and quality checks.

- [x] T025 [P] Validate spec markdown quality with `npx markdownlint-cli2 "specs/002-phase2-runtime-async/*.md" --config /Users/matthewmaggio/Mister-Smith/.markdownlint.json`
- [x] T026 [P] Validate contract and checklist markdown quality with
  `npx markdownlint-cli2 "specs/002-phase2-runtime-async/contracts/*.md" "specs/002-phase2-runtime-async/checklists/*.md" --config /Users/matthewmaggio/Mister-Smith/.markdownlint.json`
- [x] T027 Consolidate FR-to-scenario-to-command traceability for FR-001..FR-012 in
  `/Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/spec.md`
  (`## Requirements`, `## User Scenarios & Testing`, `### Validation Command Set`, `## Success Criteria`)
  and confirm checklist coverage CHK001-CHK017
- [x] T028 Validate feature readiness with `./.specify/scripts/bash/check-prerequisites.sh --json --include-tasks`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: Starts immediately.
- **Phase 2 (Foundational)**: Depends on Phase 1.
- **Phase 3 (US1)**: Depends on Phase 2.
- **Phase 4 (US2)**: Depends on Phase 2.
- **Phase 5 (US3)**: Depends on Phase 2.
- **Phase 6 (Polish)**: Depends on Phases 3, 4, and 5.

### User Story Dependencies

- **US1**: Independent after Foundational; recommended MVP.
- **US2**: Independent after Foundational.
- **US3**: Independent after Foundational.

### Parallel Opportunities

- T003, T005, T006, T007, and T008 can run in parallel after setup.
- US1 checks T011 and T012 can run in parallel.
- US2 checks T016 and T017 can run in parallel.
- US3 checks T021 and T022 can run in parallel.
- Phase 6 lint tasks T025 and T026 can run in parallel.

---

## Parallel Example: User Story 2

```bash
# Run observability checks in parallel:
Task: "T016 Validate health and metrics contract references"
Task: "T017 Validate event bus contract references"
```

---

## Implementation Strategy

### MVP First (US1)

1. Complete Setup + Foundational.
2. Complete US1 tasks T010-T014.
3. Validate runtime lifecycle baseline evidence.

### Incremental Delivery

1. Deliver US1 runtime lifecycle baseline.
2. Add US2 monitoring/event baseline.
3. Add US3 async/resource baseline.
4. Finalize with cross-cutting quality checks.

### Parallel Team Strategy

- Shared artifacts handled first.
- US2 and US3 can proceed in parallel once foundational tasks complete.

## Notes

- All tasks are execution-ready and reference exact file paths.
- `/speckit.implement` is intentionally excluded per user instruction.
