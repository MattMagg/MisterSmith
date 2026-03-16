# Quickstart: Multi-Turn Same-Agent Conversations

## Prerequisites

- active feature directory: `specs/013-multi-turn-same-agent-conversations/`
- existing real runtime-backed path from the March 16 runtime proof
- PostgreSQL available through `DATABASE_URL`
- NATS + JetStream available through `MISTER_SMITH_TRANSPORT__NATS_URL`
- provider authentication available for the chosen real runtime proof

## Required Validation Bundle

```bash
# 1. Narrow automated checks for the slice
cargo test -p mister-smith-persistence
cargo test -p mister-smith-events
cargo test -p mister-smith-http
cargo test -p mister-smith-app

# 2. Cross-crate compile safety
cargo build --workspace
```

## Real Runtime Proof Scenario

The feature is not complete until the operator can prove a real same-session, two-turn
conversation with the exact provider/model pair named in evidence.

### Scenario 1: Create A Session

1. Start the runtime with the same infrastructure contract used by the March 16 runtime proof.
2. Create a session with one initial user message.
3. Capture:
   - `session_id`
   - `workflow_id`
   - `coordinator_agent_id`
4. Verify the first turn reaches a terminal workflow state.

### Scenario 2: Continue The Same Session

1. Submit a second turn against the same `session_id`.
2. Capture the new `workflow_id`.
3. Verify:
   - `session_id` did not change
   - `coordinator_agent_id` did not change
   - the new `workflow_id` is different from the first one

### Scenario 3: Inspect Session And Workflow Linkage

1. Inspect the session over HTTP or CLI.
2. Verify the response shows:
   - the stable `coordinator_agent_id`
   - ordered turn summaries
   - the workflow IDs for both turns
3. Inspect workflow autonomy for at least one turn.
4. Verify the autonomy output includes session linkage back to the owning `session_id`.

### Scenario 4: End The Session

1. End the session after it is idle.
2. Verify the session becomes logically ended.
3. Attempt to continue the ended session.
4. Verify the runtime rejects the request without creating another workflow.

### Scenario 5: Restart And Resume

1. Stop the runtime after the first or second turn is complete.
2. Restart the runtime against the same PostgreSQL state.
3. Inspect the existing session.
4. Verify the session is still available and can be continued if it is idle and not ended.

## Example Operator Flow

```bash
# Start runtime
cargo run -q -p mister-smith-app -- run

# Create a session
curl -sS -X POST http://127.0.0.1:8080/api/v1/sessions \
  -H 'content-type: application/json' \
  -d '{"message":"Summarize the runtime proof."}'

# Inspect root workflow as a task
curl -sS http://127.0.0.1:8080/api/v1/tasks/<workflow_id>

# Continue the session
curl -sS -X POST http://127.0.0.1:8080/api/v1/sessions/<session_id>/turns \
  -H 'content-type: application/json' \
  -d '{"message":"Now turn that into a short checklist."}'

# Inspect the session
curl -sS http://127.0.0.1:8080/api/v1/sessions/<session_id>

# Inspect workflow autonomy
cargo run -q -p mister-smith-app -- autonomy status --workflow-id <workflow_id>

# End the session
curl -sS -X POST http://127.0.0.1:8080/api/v1/sessions/<session_id>/end
```

## Expected Proof Artifacts

- one stable `session_id`
- one stable `coordinator_agent_id`
- at least two distinct root `workflow_id` values
- session inspect output showing ordered turn history
- workflow autonomy output showing session linkage
- explicit provider/model attribution in captured evidence
