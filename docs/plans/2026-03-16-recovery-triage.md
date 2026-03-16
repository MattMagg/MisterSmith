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

Status: active at note creation

Scope:

- add this durable recovery note to `main`
- record the main-only landing contract
- preserve the recovered stream boundaries and overlap list

Validation target:

- `npx markdownlint-cli2 docs/plans/2026-03-16-recovery-triage.md --config .markdownlint.json`

Next action after this slice:

- isolate the runtime task-path subset inside the mixed `mister-smith-http` and `mister-smith-app`
  edits, then decide whether that code slice is separable enough to land safely on `main`

## Stop Condition

Stop the progressive landing flow if the next slice cannot be separated without high-risk
entanglement, a material architectural decision cannot be resolved from repo docs, or validation
cannot honestly prove the affected behavior.
