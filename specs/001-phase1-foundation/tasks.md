# Tasks: Phase 1 Foundation Contracts

**Input**: Design documents from `/specs/001-phase1-foundation/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Contract and compile validation tasks are included because Gate 1 evidence is mandatory in the spec.

**Organization**: Tasks are grouped by user story so each story can be completed and validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: User story label (`US1`, `US2`, `US3`)
- Each task includes exact file paths

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish feature artifact skeleton and baseline context.

- [ ] T001 Confirm active feature paths with `./.specify/scripts/bash/check-prerequisites.sh --json --paths-only`
- [ ] T002 Verify canonical source references exist in
  `/Users/matthewmaggio/Mister-Smith/spec/core-architecture/` and
  `/Users/matthewmaggio/Mister-Smith/spec/operations/`, and verify `rg`, `cargo`, and `npx` are available locally
- [ ] T003 [P] Create/verify feature artifact paths under `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build shared contract artifacts and acceptance evidence definitions.

- [ ] T004 Author implementation summary and technical context in `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/plan.md`
- [ ] T005 [P] Consolidate decision log in `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/research.md`
- [ ] T006 [P] Define shared entities in `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/data-model.md`
- [ ] T007 [P] Define baseline contracts in `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/contracts/phase1-contract-baseline.md`
- [ ] T008 [P] Capture reproducible validation steps in `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/quickstart.md`
- [ ] T009 Update Codex agent context via `./.specify/scripts/bash/update-agent-context.sh codex`

**Checkpoint**: Shared feature artifacts complete and reusable by all user-story phases.

---

## Phase 3: User Story 1 - Canonical Type and Error Baseline (Priority: P1) 🎯 MVP

**Goal**: Ensure a single canonical core type and error baseline for Phase 1.1.

**Independent Test**: Run type presence and collision checks plus Gate 1 compile checks.

### Implementation for User Story 1

- [ ] T010 [US1] Map FR-001 to FR-005 in
  `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/spec.md`
  (`## Requirements`, `### Functional Requirements`, `### User Story 1`) and record evidence in
  `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/checklists/requirements.md`
  (CHK001, CHK002, CHK008, CHK012)
- [ ] T011 [P] [US1] Validate canonical type presence with `rg` against `/Users/matthewmaggio/Mister-Smith/spec/core-architecture/type-definitions.md` (`## Canonical Core Types (Phase 1.1)`)
- [ ] T012 [P] [US1] Validate `RestartPolicy` collision constraints with `rg` across
  `/Users/matthewmaggio/Mister-Smith/spec/core-architecture/` and
  `/Users/matthewmaggio/Mister-Smith/spec/data-management/` using `type-definitions.md`
  (`## Canonical Core Types (Phase 1.1)`) as source of truth
- [ ] T013 [US1] Record lifecycle-vs-availability naming checks from
  `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/spec.md` (`### User Story 1`, FR-004) into
  `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/checklists/requirements.md` (CHK008)
- [ ] T014 [US1] Execute `cargo build -p mister-smith-core` from `/Users/matthewmaggio/Mister-Smith`
- [ ] T015 [US1] Execute `cargo build -p mister-smith-config` from `/Users/matthewmaggio/Mister-Smith`

**Checkpoint**: Canonical type and error baseline is validated and independently demonstrable.

---

## Phase 4: User Story 2 - Stable Core Trait Contracts (Priority: P2)

**Goal**: Ensure stable trait contract signatures for Phase 1.2.

**Independent Test**: Verify trait signature consistency in canonical and integration references.

### Implementation for User Story 2

- [ ] T016 [US2] Map FR-006 and FR-007 in
  `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/spec.md` (`## Requirements`, `### User Story 2`)
  and record evidence in `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/checklists/requirements.md`
  (CHK003, CHK009, CHK012)
- [ ] T017 [P] [US2] Validate `Tool` trait signature consistency using `rg` on
  `/Users/matthewmaggio/Mister-Smith/spec/core-architecture/module-organization-type-system.md`
  (`## 2. Core Trait Hierarchy and Type System`) and
  `/Users/matthewmaggio/Mister-Smith/spec/core-architecture/system-integration.md`
  (`### 5.3 Shared Tool Registry Pattern`)
- [ ] T018 [P] [US2] Validate trait contract references for `Actor`, `Agent`, `Resource`, `Supervisor`,
  `Transport` in `/Users/matthewmaggio/Mister-Smith/spec/core-architecture/module-organization-type-system.md`
  (`### 2.1 Foundational Traits with Generic Constraints`, `### 2.2 Resource Management Traits`)
- [ ] T019 [US2] Confirm trait contract completeness in
  `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/contracts/phase1-contract-baseline.md`
  (`## 2. Core Trait Contracts`) and record checklist evidence (CHK003, CHK009)
- [ ] T020 [US2] Confirm implementation-free scope boundaries in
  `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/spec.md`
  (`## Scope`, `### Constitution Alignment Requirements`) and
  `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/plan.md` (`## Technical Context`) and record CHK015

**Checkpoint**: Trait contract baseline is validated and independently demonstrable.

---

## Phase 5: User Story 3 - Typed Configuration Contracts and Validation Rules (Priority: P3)

**Goal**: Ensure deterministic typed configuration contract domains for Phase 1.3.

**Independent Test**: Validate documented config domains, layering model, and explicit failure semantics.

### Implementation for User Story 3

- [ ] T021 [US3] Map FR-008 to FR-010 in
  `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/spec.md` (`## Requirements`, `### User Story 3`)
  and record evidence in `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/checklists/requirements.md`
  (CHK004, CHK012)
- [ ] T022 [P] [US3] Validate config-domain references in
  `/Users/matthewmaggio/Mister-Smith/spec/core-architecture/implementation-config.md`
  (`### 1.1 Core Agent Configuration`, `### 1.3 Configuration Validation System`) and
  `/Users/matthewmaggio/Mister-Smith/spec/operations/configuration-management.md`
  (`## 2. Configuration File Schemas`, `## 4. Configuration Validation Rules`)
- [ ] T023 [US3] Confirm deterministic layering text in
  `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/contracts/phase1-contract-baseline.md`
  (`## 3. Configuration Contracts`) against
  `/Users/matthewmaggio/Mister-Smith/spec/operations/configuration-management.md`
  (`### 6.1 Override Precedence Rules`) and record CHK006
- [ ] T024 [US3] Confirm explicit validation-failure semantics in
  `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/spec.md` (`### Edge Cases`) and
  `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/data-model.md`
  (`## Entity: ConfigurationContractSet`) and record CHK007, CHK013

**Checkpoint**: Configuration contract baseline is validated and independently demonstrable.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final traceability, consistency, and documentation quality checks.

- [ ] T025 [P] Validate spec markdown quality with `npx markdownlint-cli2 "specs/001-phase1-foundation/*.md" --config /Users/matthewmaggio/Mister-Smith/.markdownlint.json`
- [ ] T026 [P] Validate contract and checklist markdown quality with
  `npx markdownlint-cli2 "specs/001-phase1-foundation/contracts/*.md" "specs/001-phase1-foundation/checklists/*.md" --config /Users/matthewmaggio/Mister-Smith/.markdownlint.json`
- [ ] T027 Consolidate FR-to-scenario-to-command traceability in
  `/Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/spec.md`
  (`## Requirements`, `## User Scenarios & Testing`, `### Validation Command Set`, `## Success Criteria`)
  and confirm checklist coverage CHK001-CHK016
- [ ] T028 Run readiness gate check with `./.specify/scripts/bash/check-prerequisites.sh --json --include-tasks`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: Starts immediately.
- **Phase 2 (Foundational)**: Depends on Phase 1.
- **Phase 3 (US1)**: Depends on Phase 2.
- **Phase 4 (US2)**: Depends on Phase 2.
- **Phase 5 (US3)**: Depends on Phase 2.
- **Phase 6 (Polish)**: Depends on completion of Phases 3, 4, and 5.

### User Story Dependencies

- **US1**: Independent after Foundational; serves as MVP.
- **US2**: Independent after Foundational.
- **US3**: Independent after Foundational.

### Parallel Opportunities

- T003, T005, T006, T007, T008 can run in parallel after initial setup.
- US1 checks T011 and T012 can run in parallel.
- US2 checks T017 and T018 can run in parallel.
- Phase 6 lint checks T025 and T026 can run in parallel.

---

## Parallel Example: User Story 1

```bash
# Run canonical type checks in parallel:
Task: "T011 Validate canonical type presence with rg"
Task: "T012 Validate RestartPolicy collision constraints with rg"
```

---

## Implementation Strategy

### MVP First (US1)

1. Finish Setup + Foundational.
2. Complete US1 tasks T010-T015.
3. Validate Gate 1 compile and consistency evidence.

### Incremental Delivery

1. Deliver US1 evidence and baseline.
2. Add US2 trait consistency evidence.
3. Add US3 configuration-domain evidence.
4. Finish with cross-cutting polish checks.

### Team Parallelization

- One contributor can handle shared artifacts.
- Additional contributors can run US2 and US3 in parallel after foundational completion.

## Notes

- All tasks include explicit file paths or command targets.
- No `/speckit.implement` tasks are included.
- This tasks list is execution-ready for specification workflow completion.
