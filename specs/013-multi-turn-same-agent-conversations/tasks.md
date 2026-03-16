# Tasks: Multi-Turn Same-Agent Conversations

**Input**: Design documents from `/specs/013-multi-turn-same-agent-conversations/`
**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `quickstart.md`,
`contracts/`

**Tests**: Required. The slice needs targeted persistence, events, HTTP, and app tests plus a real
runtime smoke proof of one session with two turns.

**Organization**: Tasks are grouped by one bounded implementation slice so the create/continue
conversation path lands first, then inspect, then end.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel when files do not overlap
- **[Story]**: Maps to the user story in `spec.md`
- Include exact file paths in every task description

## Path Conventions

- **Core shared IDs**: `crates/mister-smith-core/src/`
- **Persistence**: `crates/mister-smith-persistence/`
- **Events/autonomy**: `crates/mister-smith-events/`
- **HTTP transport**: `crates/mister-smith-http/src/`
- **Runtime and CLI**: `crates/mister-smith-app/src/`

## Foundational Tasks (Blocking Prerequisites)

**Goal**: establish stable session identifiers, explicit persistence, and workflow linkage before
operator surfaces are added.

- [ ] T001 Create shared `SessionId` and `SessionStatus` definitions in
  `crates/mister-smith-core/src/ids.rs`, `crates/mister-smith-core/src/enums.rs`, and
  `crates/mister-smith-core/src/lib.rs`.
- [ ] T002 [P] Add session persistence schema in
  `crates/mister-smith-persistence/migrations/00006_conversation_sessions.sql`.
- [ ] T003 [P] Add session and turn query helpers in
  `crates/mister-smith-persistence/src/postgres/queries.rs` and expose a repository facade in
  `crates/mister-smith-persistence/src/repository/session.rs`.
- [ ] T004 Add persistence coverage for create, append, inspect, busy-session conflict, and
  logical end in `crates/mister-smith-persistence/tests/session_repository_tests.rs`.
- [ ] T005 Add optional session linkage fields to workflow autonomy status in
  `crates/mister-smith-events/src/autonomy.rs` and preserve them in
  `crates/mister-smith-events/src/bus.rs`.

**Checkpoint**: the codebase has one stable session identifier model, durable session storage, and
workflow-autonomy linkage fields.

---

## User Story 1 - Create And Continue One Conversation (Priority: P1) 🎯 MVP

**Goal**: let an operator create a session and submit a follow-up turn that reuses the same
coordinator identity while creating a new root workflow.

**Independent Test**: create one session, wait for completion, continue the same session, and
verify same `session_id` plus same `coordinator_agent_id` with a distinct second `workflow_id`.

### Tests For User Story 1

- [ ] T006 [P] [US1] Add runtime/session integration tests in
  `crates/mister-smith-app/tests/conversation_runtime_tests.rs`.
- [ ] T007 [P] [US1] Add HTTP handler tests for session create and continue in
  `crates/mister-smith-http/tests/session_http_tests.rs`.

### Implementation For User Story 1

- [ ] T008 [US1] Create `crates/mister-smith-app/src/conversation.rs` with a session-aware service
  that wraps `RuntimeTaskService` and materializes retained session context.
- [ ] T009 [US1] Extend `crates/mister-smith-app/src/execution.rs` so root workflow metadata
  records `session_id`, `turn_index`, and the stable `coordinator_agent_id`.
- [ ] T010 [US1] Extend `crates/mister-smith-http/src/server.rs`,
  `crates/mister-smith-http/src/handlers.rs`, and `crates/mister-smith-http/src/routes.rs` with
  `POST /api/v1/sessions` and `POST /api/v1/sessions/{session_id}/turns`.
- [ ] T011 [US1] Extend `crates/mister-smith-app/src/bootstrap.rs` to wire the session service into
  the HTTP app state and runtime bootstrap.
- [ ] T012 [US1] Add `mister-smith conversation start` and
  `mister-smith conversation continue` in `crates/mister-smith-app/src/main.rs`.

**Checkpoint**: operators can create and continue one honest same-agent conversation through the
real runtime path.

---

## User Story 2 - Inspect Session Lineage And Current State (Priority: P1)

**Goal**: let an operator inspect the session state, ordered turns, and workflow/autonomy linkage.

**Independent Test**: inspect a two-turn session and verify ordered turn summaries, active or last
workflow linkage, and workflow autonomy session linkage.

### Tests For User Story 2

- [ ] T013 [P] [US2] Extend `crates/mister-smith-events/tests/autonomy_event_tests.rs` with
  session-linkage assertions.
- [ ] T014 [P] [US2] Add session inspect HTTP coverage in
  `crates/mister-smith-http/tests/session_http_tests.rs`.

### Implementation For User Story 2

- [ ] T015 [US2] Add session inspect query and mapping logic in
  `crates/mister-smith-persistence/src/repository/session.rs`.
- [ ] T016 [US2] Extend `crates/mister-smith-app/src/autonomy.rs` to render session linkage from
  workflow autonomy views and expose session inspect helpers.
- [ ] T017 [US2] Extend `crates/mister-smith-http/src/handlers.rs` and
  `crates/mister-smith-http/src/routes.rs` with `GET /api/v1/sessions/{session_id}`.
- [ ] T018 [US2] Add `mister-smith conversation inspect` in
  `crates/mister-smith-app/src/main.rs`.

**Checkpoint**: session inspect and workflow autonomy linkage are operator-visible without raw
database queries.

---

## User Story 3 - End A Session Cleanly (Priority: P2)

**Goal**: let an operator end an idle session while preserving history and rejecting later turns.

**Independent Test**: end an idle session, inspect it again, then verify a new turn is rejected.

### Tests For User Story 3

- [ ] T019 [P] [US3] Add end-session coverage in
  `crates/mister-smith-persistence/tests/session_repository_tests.rs` and
  `crates/mister-smith-http/tests/session_http_tests.rs`.

### Implementation For User Story 3

- [ ] T020 [US3] Add logical end/update support in
  `crates/mister-smith-persistence/src/repository/session.rs`.
- [ ] T021 [US3] Extend `crates/mister-smith-http/src/handlers.rs` and
  `crates/mister-smith-http/src/routes.rs` with `POST /api/v1/sessions/{session_id}/end`.
- [ ] T022 [US3] Add `mister-smith conversation end` in
  `crates/mister-smith-app/src/main.rs`.
- [ ] T023 [US3] Reject continue and end requests for busy or ended sessions inside
  `crates/mister-smith-app/src/conversation.rs`.

**Checkpoint**: ended sessions are preserved for inspection and cannot accept new turns.

---

## Final Validation And Evidence

- [ ] T024 Run `cargo test -p mister-smith-persistence`
- [ ] T025 Run `cargo test -p mister-smith-events`
- [ ] T026 Run `cargo test -p mister-smith-http`
- [ ] T027 Run `cargo test -p mister-smith-app`
- [ ] T028 Run `cargo build --workspace`
- [ ] T029 Capture a real runtime smoke proof in `docs/plans/2026-03-16-multi-turn-same-agent-conversations.md`
  showing one session, two turns, one stable `coordinator_agent_id`, and two distinct
  `workflow_id` values

## Explicitly Out Of Scope For This Slice

- shared sessions or multi-user collaboration
- queued concurrent turns inside one session
- force-end or force-cancel semantics
- worker-identity stability guarantees
- a new session-specific autonomy subsystem separate from workflow autonomy
