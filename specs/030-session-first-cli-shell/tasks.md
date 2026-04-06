# Tasks: Session-First CLI Shell

**Input**: Design documents from `/specs/030-session-first-cli-shell/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`, and
`contracts/`

**Tests**: Included. Use targeted Rust coverage for shell entry, recent-session home, resume
behavior, and live-session controls. Keep broader runtime proof claims separate.

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
- support commands already exist, but they are not yet shaped around a session-first CLI
- one durable retained-session source of truth must remain canonical

---

## T1. Scope And Design Freeze (Blocking Prerequisites)

**Goal**: freeze the bounded CLI shell packet before implementation lanes begin.

**CRITICAL**: no `[P]` lane may begin until this checkpoint is complete.

- [ ] T001 Freeze packet scope in `specs/030-session-first-cli-shell/spec.md` and
      `specs/030-session-first-cli-shell/plan.md`
- [ ] T002 Record data model and CLI session invariants in
      `specs/030-session-first-cli-shell/data-model.md`
- [ ] T003 Freeze CLI shell entry and CLI session behavior in
      `specs/030-session-first-cli-shell/contracts/cli-session-shell-contract.md` and
      `specs/030-session-first-cli-shell/contracts/cli-session-state-contract.md`
- [ ] T004 Confirm explicit deferrals, proof boundaries, and validation guidance in
      `specs/030-session-first-cli-shell/research.md` and
      `specs/030-session-first-cli-shell/quickstart.md`

**Checkpoint**: the packet is frozen around one session-first CLI shell and one honest validation
story.

---

## User Story 1 - Open The CLI And Start Work (Priority: P1)

**Goal**: make `mister-smith` open into a recent-first CLI shell home instead of a runtime-first
default entry.

**Independent Test**: launching `mister-smith` with no arguments shows a recent-first CLI home
with recent sessions, start-new, resume-last, warnings, and config.

### Tests For User Story 1

- [ ] T005 [P] [US1] Add or extend default-entry and startup-home CLI coverage in
      `crates/mister-smith-app/tests/session_cli_shell_entry_tests.rs`

### Implementation For User Story 1

- [ ] T006 [P] [US1] Rework the default `mister-smith` entry behavior in
      `crates/mister-smith-app/src/main.rs`
- [ ] T007 [P] [US1] Add startup-home session snapshot helpers in
      `crates/mister-smith-app/src/conversation.rs` and
      `crates/mister-smith-http/src/server.rs`
- [ ] T008 [US1] Add CLI startup-home rendering and warning presentation in
      `crates/mister-smith-app/src/main.rs`

**Checkpoint**: the default CLI entry is recent-first and no longer teaches runtime-first
navigation.

---

## User Story 2 - Resume And Browse Recent CLI Sessions (Priority: P1)

**Goal**: make resume-last, resume-by-session, and recent-session browsing first-class in the CLI
while preserving one durable retained-session model.

**Independent Test**: a user can reopen the most recent session directly, browse and reopen a
specific retained session, and see the same session summary data across the CLI resume flows.

### Tests For User Story 2

- [ ] T009 [P] [US2] Add CLI resume-flow coverage in
      `crates/mister-smith-app/tests/session_cli_shell_resume_tests.rs`

### Implementation For User Story 2

- [ ] T010 [P] [US2] Add resume-last, resume-by-id, and recent-session browse support in
      `crates/mister-smith-app/src/main.rs` and `crates/mister-smith-app/src/conversation.rs`
- [ ] T011 [P] [US2] Extend recent-session and session-detail payloads for the CLI shell in
      `crates/mister-smith-http/src/server.rs`
- [ ] T012 [US2] Add CLI session-picker and browse output behavior in
      `crates/mister-smith-app/src/main.rs`

**Checkpoint**: resume-last, resume-by-id, and recent-session browsing are distinct but consistent
CLI behaviors.

---

## User Story 3 - Steer A Live CLI Session In Place (Priority: P1)

**Goal**: keep model, permissions, config, status, and MCP controls inside the live CLI session
while preserving one retained session story.

**Independent Test**: a session can stay live in the CLI while the user changes core controls,
and the same session identity and retained history remain intact.

### Tests For User Story 3

- [ ] T013 [P] [US3] Add live-session control coverage in
      `crates/mister-smith-app/tests/session_cli_shell_control_tests.rs`

### Implementation For User Story 3

- [ ] T014 [P] [US3] Add or extend in-session control handling for model, permissions, config,
      status, and MCP in `crates/mister-smith-app/src/main.rs` and
      `crates/mister-smith-app/src/conversation.rs`
- [ ] T015 [P] [US3] Extend session-control and degraded-state payloads used by the CLI shell in
      `crates/mister-smith-http/src/server.rs`
- [ ] T016 [US3] Add CLI command help and live-session status output for the core control set in
      `crates/mister-smith-app/src/main.rs`

**Checkpoint**: users can steer live sessions in place and stay inside the CLI shell.

---

## Final Validation And Evidence

- [ ] T017 Run `cargo test -p mister-smith-app`
- [ ] T018 Run `cargo test -p mister-smith-http`
- [ ] T019 Run `cargo build --workspace`
- [ ] T020 Run
      `SPECIFY_FEATURE=030-session-first-cli-shell ./.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks`
- [ ] T021 Run
      `npx markdownlint-cli2 "specs/030-session-first-cli-shell/**/*.md" --config .markdownlint.json`
- [ ] T022 Run `git diff --check`
- [ ] T023 Refresh a durable packet proof note under `docs/plans/` and any state-bearing docs
      only when the implementation actually lands
- [ ] T024 Run `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync`

## Parallel Directive

`[P]` means a task may run in parallel only when:

- every blocking checkpoint task in the current section is complete
- its write set is disjoint from every other active lane

Shared-write choke points for this packet:

- `crates/mister-smith-app/src/main.rs`
- `crates/mister-smith-app/src/conversation.rs`
- `crates/mister-smith-http/src/server.rs`

Allowed concurrent lanes after the blocking freeze:

- CLI entry lane: `T005`, `T006`, and packet-owned CLI shell files
- retained-session lane: `T009`, `T010`, `T011`, and shared session payload files

Serial merge points:

- `T007` and `T011` before the CLI shell consumes new shared payloads
- `T014` and `T015` before the final live-session control checks
