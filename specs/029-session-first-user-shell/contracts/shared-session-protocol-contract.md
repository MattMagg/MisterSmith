# Contract: Shared Session Protocol

**Spec**: [../spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md)

## Design goal

Freeze one bounded shared-session contract for CLI and GUI so both front ends consume the same
retained session truth and the same live-session control model.

## Canonical shared objects

The shared protocol for this packet is built from:

- `Session`
- `SessionSummaryCard`
- `SessionTranscriptProjection`
- `SessionControlState`
- `StartupHomeView`
- `SupportStatusNotice`

No second GUI-only session model may claim to represent the same retained work.

## Startup snapshot contract

Both front ends must be able to consume one startup snapshot shaped like:

```json
{
  "recent_sessions": [
    {
      "session_id": "11111111-1111-1111-1111-111111111111",
      "title": "refine session shell packet",
      "status": "active",
      "last_preview": "next step is plan generation",
      "provider_kind": "openai_chatgpt",
      "model_id": "gpt-5.4",
      "updated_at": "2026-04-05T12:00:00Z"
    }
  ],
  "resume_last_target": {
    "session_id": "11111111-1111-1111-1111-111111111111"
  },
  "new_session_target": {
    "action": "start_session"
  },
  "startup_warnings": [
    {
      "notice_kind": "runtime_unavailable",
      "severity": "warning",
      "summary": "Recent sessions are available, but live work cannot continue until runtime recovers."
    }
  ],
  "config_target": {
    "action": "open_config"
  }
}
```

Behavior:

- both front ends render the same recent-session ordering and warning posture from the same source
- resume-last and browse-reopen both resolve against the same retained session identities
- degraded support state remains visible without suppressing recent-session history

## Session continuity contract

When a session is opened from either front end:

- the same `session_id` is used
- the same retained transcript or summary history is shown
- the same provider and model attribution is shown
- the same active or last-work linkage is preserved

Behavior:

- opening a session in the GUI after starting it in the CLI continues the same retained session
- opening a session in the CLI after viewing it in the GUI continues the same retained session
- neither front end may create a duplicate local copy that drifts from the canonical session truth

## Live control contract

The core in-session controls operate on one shared `SessionControlState`.

Expected behavior:

- control changes in one front end are reflected as changes to the same session control state
- model, permissions, config, status, and MCP posture stay part of the live session rather than a
  separate primary workflow
- degraded support state may limit an action honestly, but it does not erase the shared session
  identity or history

## Degraded-state contract

If runtime or support state is degraded:

- recent sessions remain visible whenever history is available
- start-new remains visible unless it is truly blocked
- the product explains why a live action cannot proceed
- the recovery path points to the appropriate support surface without replacing the main shell

## Explicit exclusions

This shared-session contract does not define:

- repo-workflow metadata
- tracker or queue integrations
- plugin marketplace behavior
- a second admin-console-only state model
- a broad runtime redesign outside the existing session and support seams
