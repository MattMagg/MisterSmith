# Recovery Triage

Date: March 16, 2026

## Objective

Preserve the recovered pre-cleanup workspace non-destructively and reconcile its intended work
back onto `main` in small, validated slices.

This note is the durable resume point for the recovery effort. It supersedes the recovery branch's
earlier "split into dedicated branches" idea: the current operator contract is to land work
directly on `main` and use `codex/recovery-20260316` only as a source branch for inspection and
selective porting.

## Control Boundaries

- Stay on `main` while landing slices.
- Treat `codex/recovery-20260316` as the primary recovered source of truth.
- Do not merge the recovery branch wholesale.
- Do not create new branches, worktrees, or stashes to organize the port.
- Keep `main` coherent, validated, and clean after every landed slice.

## Current State At Start

- clean baseline branch: `main`
- baseline commit at session start: `4918735`
- recovery source branch: `codex/recovery-20260316`
- recovery source commit: `8dc18c45e5bea6cd96bc460ddebbe338c76827ca`
- external backup retained only as a fallback artifact:
  `/Users/macmain/MisterSmith-cleanup-backups/cleanup-20260316-083808/`

Recovered workspace content is one mixed snapshot with two primary streams plus a smaller overlap
zone.

## Stream A: Real Runtime Path

Primary files:

- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/bootstrap.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- `crates/mister-smith-agents/src/orchestrator.rs`
- `crates/mister-smith-llm/src/app_server.rs`
- `crates/mister-smith-llm/src/providers/openai_chatgpt.rs`
- `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md`

Recovered intent:

- real provider-backed workflow execution on `openai_chatgpt` / `gpt-5.4`
- runtime-backed `POST /api/v1/tasks` task submission path
- runtime-backed autonomy inspection that remains useful during real execution
- March 16 runtime proof evidence recorded in the recovery snapshot

## Stream B: Spec 013 Session Surface

Primary files:

- `crates/mister-smith-app/src/conversation.rs`
- `crates/mister-smith-http/src/server.rs`
- `crates/mister-smith-http/src/handlers.rs`
- `crates/mister-smith-http/src/routes.rs`
- `crates/mister-smith-app/src/main.rs`
- `crates/mister-smith-core/src/ids.rs`
- `crates/mister-smith-core/src/enums.rs`
- `crates/mister-smith-core/src/lib.rs`
- `crates/mister-smith-persistence/src/postgres/queries.rs`
- `crates/mister-smith-persistence/src/repository/session.rs`
- `crates/mister-smith-persistence/migrations/00006_conversation_sessions.sql`
- `crates/mister-smith-persistence/migrations/00006_conversation_sessions.down.sql`
- `docs/plans/2026-03-16-multi-turn-same-agent-conversations.md`
- `specs/013-multi-turn-same-agent-conversations/`

Recovered intent:

- durable session lifecycle
- create, continue, inspect, and end session HTTP and CLI surfaces
- stable session identifiers and retained coordinator identity across turns
- SpecKit packet and implementation notes for multi-turn same-agent conversations

## Shared Overlap

Files with mixed concerns that should be split last unless a current slice truly requires them:

- `README.md`
- `crates/mister-smith-app/Cargo.toml`
- `crates/mister-smith-http/Cargo.toml`
- `crates/mister-smith-events/src/autonomy.rs`
- `crates/mister-smith-events/src/bus.rs`
- `crates/mister-smith-events/tests/autonomy_event_tests.rs`
- `crates/mister-smith-http/src/errors.rs`
- `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- `crates/mister-smith-persistence/src/lib.rs`
- `crates/mister-smith-persistence/src/repository/mod.rs`
- `Cargo.lock`

## Landing Order

1. Land this triage note on `main` so the recovery state is resumable without hidden context.
2. Carve the smallest task-only runtime subset out of the mixed runtime branch and land it with
   honest compile and test proof.
3. Land the runtime proof docs once the corresponding runtime behavior is true on `main`.
4. Land the Spec 013 session docs and code once the session slice can validate honestly on top of
   the runtime-backed task path.
5. Reconcile shared overlap files only after the two main streams are either landed or reduced to
   clear residual deltas.

## Slice Log

### Slice 1: Recovery Triage On Main

Status: landed on `main`

Scope:

- add this durable recovery note to `main`
- record the main-only landing contract
- preserve the recovered stream boundaries and overlap list

Validation target:

- `npx markdownlint-cli2 docs/plans/2026-03-16-recovery-triage.md --config .markdownlint.json`

Validation result:

- passed on `main` before commit `9a1a0bd925ba0b4d2441a74bf3c98c4f5d0e44d7`

Next action after this slice:

- isolate the runtime task-path subset inside the mixed `mister-smith-http` and `mister-smith-app`
  edits, then decide whether that code slice is separable enough to land safely on `main`

### Slice 2: Task-Only Runtime Path

Status: landed on `main`

Scope:

- restore a real runtime-backed `POST /api/v1/tasks` path without pulling in the Spec 013 session
  surface
- restore operator-visible autonomy inspection for runtime-backed task execution
- carry the minimum `mister-smith-llm` and persistence migration fixes needed for a live local run
  against PostgreSQL and NATS
- defer conversation/session identifiers, session HTTP endpoints, and migration `00006` to the
  later session slice

Files in scope:

- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/bootstrap.rs`
- `crates/mister-smith-app/src/main.rs`
- `crates/mister-smith-app/Cargo.toml`
- `crates/mister-smith-http/src/server.rs`
- `crates/mister-smith-http/src/handlers.rs`
- `crates/mister-smith-http/src/routes.rs`
- `crates/mister-smith-agents/src/orchestrator.rs`
- `crates/mister-smith-llm/src/app_server.rs`
- `crates/mister-smith-llm/src/providers/openai_chatgpt.rs`
- `crates/mister-smith-persistence/src/postgres/migrations.rs`
- `crates/mister-smith-persistence/migrations/00002_indexes.sql`
- `crates/mister-smith-persistence/migrations/00003_partitions.sql`
- `crates/mister-smith-persistence/migrations/00004_audit_schema.sql`
- `crates/mister-smith-persistence/migrations/00005_message_idempotency.sql`
- `Cargo.lock`

Validation:

- `cargo fmt --all`
- `cargo build -p mister-smith-http -p mister-smith-llm -p mister-smith-app`
- `cargo test -p mister-smith-app`
- `cargo test -p mister-smith-http`
- `cargo test -p mister-smith-llm --test app_server_tests --test openai_provider_tests`
- `cargo run -q -p mister-smith-app -- auth openai-chatgpt status`
- `env DATABASE_URL='postgres://mistersmith:mistersmith_dev@127.0.0.1:5433/mistersmith_runtime_slice3' MISTER_SMITH_TRANSPORT__NATS_URL='nats://127.0.0.1:4223' cargo run -q -p mister-smith-app -- run`
- `curl -sS -X POST http://127.0.0.1:8080/api/v1/tasks -H 'content-type: application/json'`
  `-d '{"description":"Create a concise runtime readiness brief by splitting the work into two`
  `parallel tracks: one worker analyzes bootstrap and infrastructure startup, one worker analyzes`
  `HTTP task submission and autonomy visibility, then synthesize the findings into one final`
  `answer.","priority":"high"}'`
- `cargo run -q -p mister-smith-app -- autonomy list --base-url http://127.0.0.1:8080`
- `cargo run -q -p mister-smith-app -- autonomy status --workflow-id cc209240-cfc6-4232-b159-b8de21b2a55e --base-url http://127.0.0.1:8080`

Validation result:

- all compile and targeted crate tests passed
- ChatGPT auth proof succeeded for `openai_chatgpt`
- runtime startup on a fresh local database succeeded after carrying the recovered migration fixes
- live task submission returned workflow `cc209240-cfc6-4232-b159-b8de21b2a55e`
- autonomy list/status reflected the live workflow and later terminal completion on `gpt-5.4`
- advisory `vet` attempts were inconclusive because one run lacked `OPENAI_API_KEY` and the
  agentic retry stalled without producing findings

Notes:

- the first live startup attempt against the existing `mistersmith` database failed honestly because
  migration `00006` from the session stream had already been applied there; using a fresh database
  kept this slice bounded and avoided smuggling session migration state into the runtime slice
- `mister-smith-persistence` required the recovered `migration_table_exists()` guard and SQL fixes in
  migrations `00002` through `00005` before a cold-start database could boot cleanly
- the runtime proof for this slice used the explicit provider/model pair named in the recovered
  workstream: `openai_chatgpt` with `gpt-5.4`

Remaining recovery scope after this slice:

- Spec 013 session lifecycle code and migration `00006`
- session/spec docs in `docs/plans/2026-03-16-multi-turn-same-agent-conversations.md` and
  `specs/013-multi-turn-same-agent-conversations/`
- shared overlap files still tied to the unreconciled session stream

### Slice 3: Spec 013 Docs And Packet

Status: landed on `main`

Scope:

- import the recovered SpecKit packet under `specs/013-multi-turn-same-agent-conversations/`
- import the companion plan note
- correct the companion note so it distinguishes recovered implementation evidence from code that
  is already ported onto `main`

Validation:

- `npx markdownlint-cli2 docs/plans/2026-03-16-multi-turn-same-agent-conversations.md 'specs/013-multi-turn-same-agent-conversations/**/*.md' --config .markdownlint.json`

Validation result:

- passed on `main`

Next action after this slice:

- port the bounded session code path from `codex/recovery-20260316` onto `main`, then validate the
  session crate/test bundle before attempting any live two-turn proof

### Slice 4: Bounded Session Implementation

Status: landed on `main`

Scope:

- add `SessionId` and `SessionStatus` to `mister-smith-core`
- add migration `00006_conversation_sessions.sql`, session queries, and a session repository in
  `mister-smith-persistence`
- add `ConversationRuntimeService`, session-aware runtime submission metadata, and CLI session
  commands in `mister-smith-app`
- add HTTP session contracts plus create, continue, inspect, and end routes in
  `mister-smith-http`
- enrich autonomy status with optional `session_id`, `turn_index`, and
  `coordinator_agent_id`

Validation:

- `cargo fmt --all`
- `cargo test -p mister-smith-persistence -p mister-smith-events -p mister-smith-http -p mister-smith-app`
- `cargo build --workspace`
- `env DATABASE_URL='postgres://mistersmith:mistersmith_dev@127.0.0.1:5433/mistersmith_session_slice1' MISTER_SMITH_TRANSPORT__NATS_URL='nats://127.0.0.1:4223' cargo run -q -p mister-smith-app -- run`
- `curl -sS -X POST http://127.0.0.1:8080/api/v1/sessions -H 'content-type: application/json'`
  `-d '{"message":"Summarize the runtime session slice in three bullets and keep enough retained`
  `context to turn it into a checklist on the next turn.","priority":"high"}'`
- `curl -sS -X POST http://127.0.0.1:8080/api/v1/sessions/ffe062e9-81ca-44b0-937e-12ea855d7a66/turns`
  `-H 'content-type: application/json'`
  `-d '{"message":"Turn the summary into a checklist.","priority":"high"}'`
- `curl -sS -w '\n%{http_code}\n' -X POST`
  `http://127.0.0.1:8080/api/v1/sessions/ffe062e9-81ca-44b0-937e-12ea855d7a66/turns`
  `-H 'content-type: application/json'`
  `-d '{"message":"A third turn should be rejected while turn two is still active."}'`
- `curl -sS http://127.0.0.1:8080/api/v1/sessions/ffe062e9-81ca-44b0-937e-12ea855d7a66`
- `cargo run -q -p mister-smith-app -- autonomy status --workflow-id 2f424586-88eb-4e0d-96c5-19e6186b3bed --base-url http://127.0.0.1:8080`
- `curl -sS -X POST http://127.0.0.1:8080/api/v1/sessions -H 'content-type: application/json'`
  `-d '{"message":"Reply with exactly READY.","priority":"high"}'`
- `curl -sS -X POST http://127.0.0.1:8080/api/v1/sessions/e5edd025-b59d-4b2d-9c26-21c938290917/end`
- `curl -sS -w '\n%{http_code}\n' -X POST`
  `http://127.0.0.1:8080/api/v1/sessions/e5edd025-b59d-4b2d-9c26-21c938290917/turns`
  `-H 'content-type: application/json'`
  `-d '{"message":"This should be rejected because the session already ended."}'`

Validation result:

- targeted crate tests passed and `cargo build --workspace` passed
- live provider-backed proof on `openai_chatgpt` / `gpt-5.4` accepted two turns on session
  `ffe062e9-81ca-44b0-937e-12ea855d7a66` with stable coordinator
  `4938b447-bee1-4888-92c9-c5e162a1f5f7`
- the two accepted turns used distinct root workflows
  `8340e91f-dcf6-4f22-9e82-83fd174b5619` and `2f424586-88eb-4e0d-96c5-19e6186b3bed`
- autonomy status for the second workflow rendered session linkage with session ID, turn index,
  and coordinator ID
- a third turn against the active session returned HTTP `409` with `session_busy`
- a separate idle session ended cleanly and a later continue returned HTTP `409` with
  `session_ended`

Remaining recovery scope after this slice:

- overlap documentation review only: `README.md`,
  `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md`,
  `docs/plans/2026-03-16-multi-turn-same-agent-conversations.md`, and
  `docs/plans/2026-03-16-recovery-triage.md`
- no unreconciled session code or migration files remain after this slice

## Recovery Closure

The recovery goal is now complete in substance:

- the recovered runtime-backed task path is on `main`
- the recovered bounded session path is on `main`
- the remaining reconciliation work is documentation and forward-direction cleanup only

This note is now historical recovery evidence, not the primary forward-planning artifact. Use
`docs/plans/2026-03-16-winyear-frontier-direction.md` for the next backlog and staging decisions.

## Stop Condition

Stop the progressive landing flow if the next slice cannot be separated without high-risk
entanglement, a material architectural decision cannot be resolved from repo docs, or validation
cannot honestly prove the affected behavior.
