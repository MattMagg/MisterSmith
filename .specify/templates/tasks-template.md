---
description: "Task list template for Mister Smith packet execution"
---

# Tasks: [FEATURE NAME]

**Input**: Design documents from `/specs/[###-feature-name]/`
**Prerequisites**: `plan.md`, `spec.md`, plus any packet-local `research.md`, `data-model.md`,
`quickstart.md`, and `analyze.md`

**Tests**: List the exact targeted validation expected for this packet. Keep deterministic checks
and live-proof expectations separate.

**Organization**: Group tasks by blocking freeze work first, then bounded user stories or lanes,
then final validation and evidence.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Use only when every blocking checkpoint in the current section is already complete and
  the task write set is disjoint from every other active lane
- **[Story]**: Use `US1`, `US2`, `US3`, etc. for story-bound work
- Include exact file paths in every implementation or documentation task

## Status Reconciliation

Capture the current repo truth this packet must preserve.

- [landed baseline truth]
- [landed baseline truth]
- [explicit invariant that must not regress]

---

## T1. Scope And Design Freeze (Blocking Prerequisites)

**Goal**: freeze the bounded packet before implementation lanes begin.

**CRITICAL**: no `[P]` lane may begin until this checkpoint is complete.

- [ ] T001 [US1] Freeze packet scope in `specs/[###-feature-name]/spec.md` and
  `specs/[###-feature-name]/plan.md`
- [ ] T002 [US1] Record data model, assumptions, or contract boundaries in packet artifacts
- [ ] T003 [US1] Confirm explicit deferrals and validation boundaries in packet artifacts

**Checkpoint**: the packet is frozen around one bounded gap and one honest validation story.

---

## User Story 1 - [Title] (Priority: P1)

**Goal**: [bounded story goal]

**Independent Test**: [story-level validation]

### Tests For User Story 1

- [ ] T004 [P] [US1] Add or extend targeted coverage in [path]
- [ ] T005 [P] [US1] Add or extend targeted coverage in [path]

### Implementation For User Story 1

- [ ] T006 [P] [US1] Implement bounded change in [path]
- [ ] T007 [US1] Preserve related invariant or fallback behavior in [path]

**Checkpoint**: [what is true once US1 is complete]

---

## User Story 2 - [Title] (Priority: P1 or P2)

**Goal**: [bounded story goal]

**Independent Test**: [story-level validation]

### Tests For User Story 2

- [ ] T008 [P] [US2] Add or extend targeted coverage in [path]
- [ ] T009 [P] [US2] Add or extend targeted coverage in [path]

### Implementation For User Story 2

- [ ] T010 [P] [US2] Implement bounded change in [path]
- [ ] T011 [US2] Preserve related invariant or compatibility rule in [path]

**Checkpoint**: [what is true once US2 is complete]

---

## User Story 3 - [Title] (Priority: P2 or P3)

**Goal**: [bounded proof, inspection, or follow-on goal]

**Independent Test**: [story-level validation]

### Tests For User Story 3

- [ ] T012 [P] [US3] Add or extend targeted coverage in [path]
- [ ] T013 [P] [US3] Add or extend targeted coverage in [path]

### Implementation For User Story 3

- [ ] T014 [P] [US3] Capture durable evidence or guidance in [path]
- [ ] T015 [US3] Record explicit proof boundaries or deferred claims in [path]

**Checkpoint**: [what is true once US3 is complete]

---

## Final Validation And Evidence

- [ ] T016 Run targeted validation for the touched code and docs
- [ ] T017 Run any required broader compatibility build or lint command
- [ ] T018 Refresh the durable proof note, artifact index, or state-bearing docs in [path]
- [ ] T019 Run `git diff --check`
- [ ] T020 Run `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync`

## Parallel Directive

`[P]` means a task may run in parallel only when:

- every blocking checkpoint task in the current section is already complete
- its write set is disjoint from every other active lane

Shared-write choke points for this packet:

- [file or doc path]
- [file or doc path]

Allowed concurrent lanes after the blocking freeze:

- [lane name]: [task IDs and write set]
- [lane name]: [task IDs and write set]

Serial merge points:

- [task or file that must stay single-owner]
