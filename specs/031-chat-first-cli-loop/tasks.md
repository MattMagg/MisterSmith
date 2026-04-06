# Tasks: Chat-First CLI Loop

**Input**: Design documents from `/specs/031-chat-first-cli-loop/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`,
`contracts/`, and `analyze.md`

**Tests**: Included. Use targeted Rust coverage for the CLI session loop, resumed continuity, and
truth-notice behavior. Keep deterministic checks separate from any later live-proof rerun.

**Organization**: Group tasks by blocking scope freeze first, then the three bounded user stories,
then final validation and evidence.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Use only when every blocking checkpoint in the current section is complete and the
  write set is disjoint from other active lanes
- **[Story]**: Use `US1`, `US2`, `US3` for story-bound work
- Include exact file paths in every implementation or documentation task

## Status Reconciliation

Capture the current repo truth this packet must preserve.

- packet `030` already owns recent-first startup, resume entry, and core in-session controls
- the current durable session model and retained history remain the one source of truth
- runtime-truth and proof-boundary claims must stay explicit when the loop becomes more
  conversation-shaped

---

## T1. Scope And Design Freeze (Blocking Prerequisites)

**Goal**: freeze the bounded packet before implementation lanes begin.

**CRITICAL**: no `[P]` lane may begin until this checkpoint is complete.

- [x] T001 [US1] Freeze packet scope in `specs/031-chat-first-cli-loop/spec.md` and
      `specs/031-chat-first-cli-loop/plan.md`
- [x] T002 [US1] Record packet-owned loop entities and invariants in
      `specs/031-chat-first-cli-loop/data-model.md` and
      `specs/031-chat-first-cli-loop/contracts/cli-session-loop-contract.md`
- [x] T003 [US1] Confirm deferrals, proof boundaries, and validation posture in
      `specs/031-chat-first-cli-loop/quickstart.md` and
      `specs/031-chat-first-cli-loop/analyze.md`

**Checkpoint**: the packet is frozen around one bounded CLI session-loop gap and one honest
validation story.

---

## User Story 1 - Stay Inside One Live Conversation (Priority: P1)

**Goal**: keep the user inside one active CLI session loop while turns are accepted, running,
completed, failed, or blocked.

**Independent Test**: open an active session, send multiple follow-up turns, and confirm the loop
surfaces inline turn-state without relying on detached inspection output.

### Tests For User Story 1

- [ ] T004 [P] [US1] Add loop-state rendering coverage in
      `crates/mister-smith-app/src/conversation.rs`
- [ ] T005 [P] [US1] Add active CLI session-loop interaction coverage in
      `crates/mister-smith-app/src/main.rs`

### Implementation For User Story 1

- [ ] T006 [P] [US1] Replace detached turn-accepted-plus-inspect flow in
      `crates/mister-smith-app/src/main.rs`
- [ ] T007 [US1] Add inline current-turn state and conversation-loop rendering helpers in
      `crates/mister-smith-app/src/conversation.rs`

**Checkpoint**: follow-up turns read as one live CLI conversation instead of submit plus inspect.

---

## User Story 2 - Resume Retained Work Back Into The Loop (Priority: P1)

**Goal**: make resumed sessions reopen as usable live conversation context with retained history
and stored controls.

**Independent Test**: resume the most recent session and a selected prior session and confirm each
one lands back in a usable live loop with retained context still visible.

### Tests For User Story 2

- [ ] T008 [P] [US2] Add resumed-session continuity and retained-context coverage in
      `crates/mister-smith-app/src/conversation.rs`
- [ ] T009 [P] [US2] Add session-view contract coverage for resumed and degraded states in
      `crates/mister-smith-http/src/server.rs`

### Implementation For User Story 2

- [ ] T010 [P] [US2] Extend resumed-session view mapping and loop entry behavior in
      `crates/mister-smith-app/src/conversation.rs` and
      `crates/mister-smith-app/src/main.rs`
- [ ] T011 [US2] Preserve runtime-unavailable, busy-session, and ended-session truth inside the
      resumed loop in `crates/mister-smith-app/src/conversation.rs` and
      `crates/mister-smith-http/src/server.rs`

**Checkpoint**: resumed work feels like continuation of one session, not reopening a static
archive.

---

## User Story 3 - Steer And Supervise The Session In Place (Priority: P1)

**Goal**: keep steering controls and truth notices inside the live loop while preserving honest
runtime and proof-boundary wording.

**Independent Test**: adjust in-session controls during an active session and confirm busy,
degraded, or proof-limited states remain visible without leaving the loop.

### Tests For User Story 3

- [ ] T012 [P] [US3] Add in-session steering and truth-notice coverage in
      `crates/mister-smith-app/src/conversation.rs`
- [ ] T013 [P] [US3] Add slash-command loop coverage for active-session steering in
      `crates/mister-smith-app/src/main.rs`

### Implementation For User Story 3

- [ ] T014 [P] [US3] Keep steering commands inside the live loop in
      `crates/mister-smith-app/src/main.rs`
- [ ] T015 [US3] Preserve support notices, proof-boundary wording, and retained-control posture in
      `crates/mister-smith-app/src/conversation.rs` and
      `crates/mister-smith-http/src/server.rs`

**Checkpoint**: the live loop keeps steering and truth visible without collapsing into an
admin-first workflow.

---

## Final Validation And Evidence

- [ ] T016 Run `cargo test -p mister-smith-app`
- [ ] T017 Run `cargo test -p mister-smith-http`
- [ ] T018 Run `cargo build --workspace`
- [ ] T019 Run
      `SPECIFY_FEATURE=031-chat-first-cli-loop ./.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks`
- [ ] T020 Run
      `npx markdownlint-cli2 "specs/031-chat-first-cli-loop/**/*.md" --config .markdownlint.json`
- [ ] T021 Run `git diff --check`

## Parallel Directive

`[P]` means a task may run in parallel only when:

- every blocking checkpoint task in the current section is already complete
- its write set is disjoint from every other active lane

Shared-write choke points for this packet:

- `crates/mister-smith-app/src/main.rs`
- `crates/mister-smith-app/src/conversation.rs`
- `crates/mister-smith-http/src/server.rs`

Allowed concurrent lanes after the blocking freeze:

- CLI loop lane: `T004`, `T005`, `T006`, `T014`
- session rendering lane: `T007`, `T008`, `T012`, `T015`
- session-view contract lane: `T009`, `T011`

Serial merge points:

- `T010` because it joins resume behavior across `main.rs` and `conversation.rs`
- final validation tasks `T016` through `T021`
