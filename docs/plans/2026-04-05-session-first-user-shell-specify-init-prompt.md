# Session-First User Shell Specify Init Prompt

Use this to start the SpecKit spec for the session-first Mister Smith user shell.

Source notes:

- `docs/plans/2026-04-05-session-first-user-shell-pre-speckit-primer.md`
- `docs/plans/2026-04-05-mister-smith-operational-cli-proposal.md`

```text
[$speckit-specify](/Users/macmain/MisterSmith/.agents/skills/speckit-specify/SKILL.md)

Create the spec for a session-first Mister Smith user shell.
Use docs/plans/2026-04-05-session-first-user-shell-pre-speckit-primer.md as the main source
and docs/plans/2026-04-05-mister-smith-operational-cli-proposal.md as supporting context.
Keep this product-side, not repo-workflow-side: the feature is one shared session system with
two front ends, terminal UI and desktop GUI.
Center the spec on opening the shell, starting or resuming sessions, browsing recent sessions,
steering a live session in place, and moving between CLI and GUI without losing session state.
Include shared session storage and app protocol, startup home behavior, recent-session and
resume flows, and in-session controls for model, permissions, config, status, and MCP.
Keep runtime, proof, auth, doctor, and MCP admin surfaces as support features, not the main
product path.
Do not let the spec drift into Linear, Symphony, Ralph, SpecKit workflow glue, or a generic
admin console.
```
