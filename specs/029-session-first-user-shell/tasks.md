# Tasks: Session-First User Shell

**Input**: Design documents from `/specs/029-session-first-user-shell/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`, and
`contracts/`

**Tests**: Included. Use targeted Rust and desktop-app coverage for shell entry, recent-session
home, resume behavior, shared session continuity, and live-session controls. Keep broader runtime
proof claims separate.

**Organization**: Tasks are grouped by blocking contract freeze first, then by the three product
stories, then final validation and evidence.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel only after all blocking tasks in the current section are complete
  and the write set is disjoint
- **[Story]**: Which user story the task advances (`US1` through `US3`)
- Every task includes exact file paths

## Status Reconciliation

Capture the current repo truth this packet must preserve.

- the current repo already has durable retained sessions with stable session identity and retained
  turn history
- `mister-smith` currently still feels runtime-first at the default entry
- the desktop app already exposes session and runtime state, but it is not yet the product shell
- one shared retained-session source of truth must remain canonical across CLI and GUI

---

## T1. Scope And Design Freeze (Blocking Prerequisites)

**Goal**: freeze the bounded product shell packet before implementation lanes begin.

**CRITICAL**: no `[P]` lane may begin until this checkpoint is complete.

- [ ] T001 Freeze packet scope in `specs/029-session-first-user-shell/spec.md` and
      `specs/029-session-first-user-shell/plan.md`
- [ ] T002 Record data model and shared-session invariants in
      `specs/029-session-first-user-shell/data-model.md`
- [ ] T003 Freeze shell entry and shared-session behavior in
      `specs/029-session-first-user-shell/contracts/session-shell-contract.md` and
      `specs/029-session-first-user-shell/contracts/shared-session-protocol-contract.md`
- [ ] T004 Confirm explicit deferrals, proof boundaries, and validation guidance in
      `specs/029-session-first-user-shell/research.md` and
      `specs/029-session-first-user-shell/quickstart.md`

**Checkpoint**: the packet is frozen around one shared session-first shell and one honest
validation story.

---

## User Story 1 - Open The Shell And Start Work (Priority: P1)

**Goal**: make `mister-smith` open into a recent-first shell home instead of a runtime-first
default entry.

**Independent Test**: launching `mister-smith` with no arguments shows a recent-first home with
recent sessions, start-new, resume-last, warnings, and config in both front ends.

### Tests For User Story 1

- [ ] T005 [P] [US1] Add or extend default-entry and startup-home CLI coverage in
      `crates/mister-smith-app/tests/session_shell_entry_tests.rs`
- [ ] T006 [P] [US1] Add startup-home desktop coverage in
      `apps/operator-console/src/App.test.tsx`

### Implementation For User Story 1

- [ ] T007 [P] [US1] Rework the default `mister-smith` entry behavior in
      `crates/mister-smith-app/src/main.rs`
- [ ] T008 [P] [US1] Add startup-home session snapshot helpers in
      `crates/mister-smith-app/src/conversation.rs` and
      `crates/mister-smith-http/src/server.rs`
- [ ] T009 [US1] Render the recent-first home and startup warnings in
      `apps/operator-console/src/App.tsx`, `apps/operator-console/src/services.ts`, and
      `apps/operator-console/src/types.ts`

**Checkpoint**: the default shell entry is recent-first and no longer teaches runtime-first
navigation.

---

## User Story 2 - Resume And Browse Recent Sessions (Priority: P1)

**Goal**: make resume-last, resume-by-session, and recent-session browsing first-class across CLI
and GUI while preserving one shared retained-session model.

**Independent Test**: a user can reopen the most recent session directly, browse and reopen a
specific retained session, and see the same session summary data in both front ends.

### Tests For User Story 2

- [ ] T010 [P] [US2] Add CLI or HTTP resume-flow coverage in
      `crates/mister-smith-app/tests/session_shell_resume_tests.rs`
- [ ] T011 [P] [US2] Add recent-session browse and reopen desktop coverage in
      `apps/operator-console/src/App.test.tsx`

### Implementation For User Story 2

- [ ] T012 [P] [US2] Add resume-last, resume-by-id, and recent-session browse support in
      `crates/mister-smith-app/src/main.rs` and `crates/mister-smith-app/src/conversation.rs`
- [ ] T013 [P] [US2] Extend shared recent-session and session-detail payloads in
      `crates/mister-smith-http/src/server.rs`
- [ ] T014 [US2] Reuse the same recent-session and session-detail model in
      `apps/operator-console/src/services.ts`, `apps/operator-console/src/types.ts`, and
      `apps/operator-console/src/App.tsx`

**Checkpoint**: resume-last, resume-by-id, and recent-session browsing are distinct but consistent
product behaviors across both front ends.

---

## User Story 3 - Steer A Live Session Across CLI And GUI (Priority: P1)

**Goal**: keep model, permissions, config, status, and MCP controls inside the live session and
preserve one shared live-session state when users move between CLI and GUI.

**Independent Test**: a session started in one front end can be opened in the other, the same
session identity and retained history are preserved, and the same core control set is available
in-session.

### Tests For User Story 3

- [ ] T015 [P] [US3] Add live-session control coverage in
      `crates/mister-smith-app/tests/session_shell_control_tests.rs`
- [ ] T016 [P] [US3] Add GUI control-parity and cross-surface continuity coverage in
      `apps/operator-console/src/App.test.tsx`

### Implementation For User Story 3

- [ ] T017 [P] [US3] Add or extend in-session control handling for model, permissions, config,
      status, and MCP in `crates/mister-smith-app/src/main.rs` and
      `crates/mister-smith-app/src/conversation.rs`
- [ ] T018 [P] [US3] Extend shared session-control and degraded-state payloads in
      `crates/mister-smith-http/src/server.rs`
- [ ] T019 [US3] Add GUI in-session control parity and same-session continuity handling in
      `apps/operator-console/src/App.tsx`, `apps/operator-console/src/services.ts`, and
      `apps/operator-console/src/types.ts`

**Checkpoint**: users can steer live sessions in place and move between CLI and GUI without
losing the shared session story.

---

## Final Validation And Evidence

- [ ] T020 Run `cargo test -p mister-smith-app`
- [ ] T021 Run `cargo test -p mister-smith-http`
- [ ] T022 Run `npm --prefix apps/operator-console test`
- [ ] T023 Run `npm --prefix apps/operator-console run build`
- [ ] T024 Run
      `SPECIFY_FEATURE=029-session-first-user-shell ./.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks`
- [ ] T025 Run
      `npx markdownlint-cli2 "specs/029-session-first-user-shell/**/*.md" --config .markdownlint.json`
- [ ] T026 Run `git diff --check`
- [ ] T027 Refresh a durable packet proof note under `docs/plans/` and any state-bearing docs
      only when the implementation actually lands
- [ ] T028 Run `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync`

## Parallel Directive

`[P]` means a task may run in parallel only when:

- every blocking checkpoint task in the current section is complete
- its write set is disjoint from every other active lane

Shared-write choke points for this packet:

- `crates/mister-smith-app/src/main.rs`
- `crates/mister-smith-app/src/conversation.rs`
- `crates/mister-smith-http/src/server.rs`
- `apps/operator-console/src/App.tsx`

Allowed concurrent lanes after the blocking freeze:

- CLI entry lane: `T005`, `T007`, and packet-owned CLI shell files
- retained-session lane: `T010`, `T012`, `T013`, and shared session payload files
- desktop shell lane: `T006`, `T011`, `T016`, `T019`, and desktop app files

Serial merge points:

- `T008` and `T013` before the desktop lane consumes new shared payloads
- `T017` and `T018` before the final cross-surface continuity checks
