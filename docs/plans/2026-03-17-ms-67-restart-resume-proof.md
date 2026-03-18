# MS-67 Restart-Resume Integration Proof

Date: March 17, 2026
Status: completed locally on `ms-67-restart-resume-proof`

## Objective

Add one honest runtime-backed proof for the bounded restart-resume session contract:

- accept a session turn through the real `mister-smith` HTTP surface
- stop the runtime before that turn finishes cleanly
- restart the runtime against the same Postgres and NATS state
- inspect the session until it is idle again
- continue the same session and verify the resumed lineage

This proof stays inside the existing session contract. It does not add UI, multi-user, or
concurrent-turn semantics.

## Repo Artifact

The automated proof now lives in:

- `crates/mister-smith-integration-tests/tests/conversation_restart_resume.rs`

It is intentionally `#[ignore]` and env-gated because it requires:

- local PostgreSQL on `127.0.0.1:5432`
- local NATS/JetStream on `127.0.0.1:4222`
- an authenticated ChatGPT session for the selected real runtime provider path
- a prebuilt `target/debug/mister-smith` binary

## Selected Runtime Path

This proof uses the already-selected Tier 2 provider/model from the recovered runtime proof:

- provider: `openai_chatgpt`
- model: `gpt-5.4`

That keeps the restart-resume evidence aligned with the currently landed real runtime path on
`main`.

## Smoke Procedure

1. Start the local infra with published host ports:
   - `docker compose -f deploy/docker-compose.yml up -d --force-recreate postgres nats`
2. Build the app binary used by the subprocess harness:
   - `cargo build -p mister-smith-app --bin mister-smith`
3. Run the ignored live proof:
   - `cargo test -p mister-smith-integration-tests live_restart_resume_http_roundtrip_recovers_idle_session_and_resumed_lineage -- --ignored --exact --nocapture`

Optional overrides:

- `MS67_TEST_ADMIN_DATABASE_URL` to point the harness at a different PostgreSQL admin database
- `MS67_TEST_NATS_URL` to point the harness at a different NATS address
- `MISTER_SMITH_APP_BINARY` to point the harness at a non-default built binary path

## What The Harness Proves

The test performs one bounded end-to-end sequence:

1. Create a fresh temporary PostgreSQL database for the proof run.
2. Start `target/debug/mister-smith run` against that database and local NATS.
3. `POST /api/v1/sessions` to accept turn 1 and capture:
   - `session_id`
   - `workflow_id`
   - `coordinator_agent_id`
4. Kill the first runtime immediately after acceptance so the first workflow is left for
   restart-time recovery.
5. Start a second `mister-smith run` process against the same database and NATS.
6. `GET /api/v1/sessions/{session_id}` until the runtime repairs the orphaned first turn and makes
   the session idle again.
7. `POST /api/v1/sessions/{session_id}/turns` to accept turn 2 on the same session.
8. `GET /api/v1/sessions/{session_id}` until turn 2 reaches a terminal state and exposes resumed
   lineage from the recovered first turn.
9. Kill the second runtime and drop the temporary database.

Assertions:

- the restarted runtime can inspect the same session without manual repair
- the repaired session becomes idle with `active_workflow_id = null`
- turn 1 exposes `resume_provenance.recovered_after_restart = true`
- turn 2 keeps the same `session_id` and `coordinator_agent_id`
- turn 2 has a distinct `workflow_id`
- turn 2 exposes:
  - `resume_provenance.resumed_after_restart = true`
  - `resume_provenance.resumed_from_turn_index = 1`
  - `resume_provenance.resumed_from_workflow_id = <turn-1 workflow>`

## Observed Result In This Session

Command:

- `cargo test -p mister-smith-integration-tests live_restart_resume_http_roundtrip_recovers_idle_session_and_resumed_lineage -- --ignored --exact --nocapture`

Observed result:

- test status: `ok`
- elapsed time: `152.25s`
- recovered session: `64c0f81a-2126-4147-b6ca-63a8ca2039ab`
- stable coordinator: `6403220e-163a-45d7-976f-ef0c60ee6577`
- recovered turn-1 workflow: `f410c10a-143b-4fcb-a10a-8d79f550cb10`
- resumed turn-2 workflow: `3a223a0c-949b-42d8-ace9-4e1095839505`

Observed session state during the run:

- after restart-driven inspect: session turn 1 was repaired to an idle failed turn and became the
  `last_completed_workflow_id`
- after continue: the same session and coordinator accepted turn 2 with a distinct workflow ID
- final inspect satisfied the resumed-lineage assertions and returned the session to an idle state

## Validation Boundary

What this note proves:

- one honest local restart-resume path for the current session HTTP/runtime contract
- durable evidence that the existing service can recover an interrupted accepted turn and continue
  the same session afterward

What this note does not prove:

- provider-neutral Tier 1 runtime proof
- multi-user or shared-session semantics
- concurrent queued turns
- force-end, branching, or any contract beyond the current bounded session slice
