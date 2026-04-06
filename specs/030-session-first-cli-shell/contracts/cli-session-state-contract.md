# Contract: CLI Session State

**Spec**: [../spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md)

## Design goal

Freeze one bounded retained-session contract for the CLI shell so startup, resume, browse, and
live-session steering all consume the same durable session truth.

## Canonical shared objects

The CLI shell for this packet is built from:

- `Session`
- `SessionSummaryRow`
- `SessionTranscriptProjection`
- `SessionControlState`
- `StartupHomeView`
- `SupportStatusNotice`

No second CLI-only history store may claim to represent the same retained work.

## Startup snapshot contract

The CLI must be able to consume one startup snapshot shaped like:

```json
{
  "recent_sessions": [
    {
      "session_id": "11111111-1111-1111-1111-111111111111",
      "title": "refine cli shell packet",
      "status": "active",
      "last_preview": "next step is task generation",
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

- the CLI renders the same recent-session ordering and warning posture from the durable session
  source of truth
- resume-last and browse-reopen both resolve against the same retained session identities
- degraded support state remains visible without suppressing recent-session history

## Session continuity contract

When a session is opened or resumed from the CLI:

- the same `session_id` is used
- the same retained transcript or summary history is shown
- the same provider and model attribution is shown
- the same active or last-work linkage is preserved

Behavior:

- quick resume and browse-reopen both continue the same retained session rather than creating a
  second history record
- the CLI does not present conflicting session state when a selected session is already busy

## Live control contract

The core in-session controls operate on one `SessionControlState`.

Expected behavior:

- control changes in the CLI act on the same live session the user is already in
- model, permissions, config, status, and MCP posture stay part of the live CLI session rather
  than a separate primary workflow
- degraded support state may limit an action honestly, but it does not erase the session identity
  or retained history

## Degraded-state contract

If runtime or support state is degraded:

- recent sessions remain visible whenever history is available
- start-new remains visible unless it is truly blocked
- the CLI explains why a live action cannot proceed
- the recovery path points to the appropriate support surface without replacing the main shell

## Explicit exclusions

This CLI session-state contract does not define:

- GUI parity or cross-surface continuity
- repo-workflow metadata
- tracker or queue integrations
- plugin marketplace behavior
- a broad runtime redesign outside the existing session and support seams
