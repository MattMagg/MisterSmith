# Contract: Session Surface

**Spec**: [../spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md)

## Design Goal

Add the smallest operator-visible session contract on top of the current runtime-backed one-shot
workflow path:

- create
- continue
- inspect
- end

Deep autonomy inspection remains workflow-scoped. The session contract only adds the linkage needed
to map a conversation session to the workflow roots created by each turn.

## HTTP Contract

### 1. Create Session

`POST /api/v1/sessions`

Request:

```json
{
  "message": "Summarize the runtime proof and tell me what changed since the last turn.",
  "priority": "high"
}
```

Accepted response:

```json
{
  "session_id": "11111111-1111-1111-1111-111111111111",
  "workflow_id": "22222222-2222-2222-2222-222222222222",
  "coordinator_agent_id": "33333333-3333-3333-3333-333333333333",
  "turn_index": 1,
  "status": "queued"
}
```

Behavior:

- creates the durable session envelope
- creates the first root workflow turn
- returns the stable session coordinator identity

### 2. Continue Session

`POST /api/v1/sessions/{session_id}/turns`

Request:

```json
{
  "message": "Now turn that into a short operator checklist."
}
```

Accepted response:

```json
{
  "session_id": "11111111-1111-1111-1111-111111111111",
  "workflow_id": "44444444-4444-4444-4444-444444444444",
  "coordinator_agent_id": "33333333-3333-3333-3333-333333333333",
  "turn_index": 2,
  "status": "queued"
}
```

Behavior:

- reuses the existing `session_id`
- reuses the existing `coordinator_agent_id`
- creates one new root `workflow_id`

Busy-session conflict:

```json
{
  "error": "session_busy",
  "session_id": "11111111-1111-1111-1111-111111111111",
  "active_workflow_id": "22222222-2222-2222-2222-222222222222"
}
```

Ended-session conflict:

```json
{
  "error": "session_ended",
  "session_id": "11111111-1111-1111-1111-111111111111"
}
```

### 3. Inspect Session

`GET /api/v1/sessions/{session_id}`

Success response:

```json
{
  "session_id": "11111111-1111-1111-1111-111111111111",
  "status": "active",
  "coordinator_agent_id": "33333333-3333-3333-3333-333333333333",
  "provider_kind": "openai_chatgpt",
  "model_id": "gpt-5.4",
  "active_workflow_id": null,
  "last_completed_workflow_id": "44444444-4444-4444-4444-444444444444",
  "turn_count": 2,
  "turns": [
    {
      "turn_index": 1,
      "workflow_id": "22222222-2222-2222-2222-222222222222",
      "status": "completed",
      "user_message": "Summarize the runtime proof and tell me what changed since the last turn."
    },
    {
      "turn_index": 2,
      "workflow_id": "44444444-4444-4444-4444-444444444444",
      "status": "completed",
      "user_message": "Now turn that into a short operator checklist."
    }
  ],
  "ended_at": null
}
```

Behavior:

- returns session lifecycle state and the stable same-agent identifier
- returns ordered turn summaries
- returns the workflow IDs required for deeper task or autonomy inspection

### 4. End Session

`POST /api/v1/sessions/{session_id}/end`

Success response:

```json
{
  "session_id": "11111111-1111-1111-1111-111111111111",
  "status": "ended",
  "ended_at": "2026-03-16T20:15:00Z"
}
```

Behavior:

- logically closes the session
- preserves history for later inspection
- does not delete rows

Active-session conflict:

```json
{
  "error": "session_busy",
  "session_id": "11111111-1111-1111-1111-111111111111",
  "active_workflow_id": "44444444-4444-4444-4444-444444444444"
}
```

## CLI Contract

### Create

```bash
mister-smith conversation start --message "Summarize the runtime proof."
```

Expected operator output:

- `session_id`
- `workflow_id`
- `coordinator_agent_id`
- `turn_index`
- current accepted status

### Continue

```bash
mister-smith conversation continue \
  --session-id 11111111-1111-1111-1111-111111111111 \
  --message "Now make it shorter."
```

Expected operator output:

- same `session_id`
- same `coordinator_agent_id`
- new `workflow_id`
- incremented `turn_index`

### Inspect

```bash
mister-smith conversation inspect \
  --session-id 11111111-1111-1111-1111-111111111111
```

Expected operator output:

- session lifecycle state
- coordinator agent identifier
- active and last workflow linkage
- ordered turn summaries

### End

```bash
mister-smith conversation end \
  --session-id 11111111-1111-1111-1111-111111111111
```

Expected operator output:

- ended state
- end timestamp

## Relationship To Existing Surfaces

The following existing surfaces remain valid:

- `POST /api/v1/tasks` for one-shot submission
- `GET /api/v1/tasks/{task_id}` where root `task_id == workflow_id`
- `GET /api/v1/autonomy/status/{workflow_id}`
- `mister-smith autonomy status --workflow-id <id>`

## Minimal Autonomy Change

Keep workflow autonomy as the deep inspection surface and add these optional linkage fields to the
workflow status contract:

- `session_id`
- `session_turn_index`
- `coordinator_agent_id`

That lets an operator move between:

- session inspect -> active or last `workflow_id`
- workflow autonomy inspect -> owning `session_id` and turn index
