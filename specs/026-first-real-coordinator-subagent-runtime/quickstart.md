# Quickstart: First Real Coordinator-Subagent Runtime

## Packet readiness validation

Use these checks to validate the implementation-ready packet bundle now:

```bash
SPECIFY_FEATURE=026-first-real-coordinator-subagent-runtime \
  ./.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks
npx markdownlint-cli2 \
  "specs/026-first-real-coordinator-subagent-runtime/**/*.md" \
  docs/current-state.md \
  docs/direction.md \
  docs/plans/2026-04-03-packet-026-implementation-ready.md \
  --config .markdownlint.json
git diff --check
```

## Current authority inputs

Before implementation starts, re-read these sources in this order:

```bash
sed -n '1,260p' docs/current-state.md
sed -n '1,220p' docs/direction.md
sed -n '1,260p' specs/022-durable-workflow-core/spec.md
sed -n '1,260p' specs/023-runtime-truth-and-run-trace/spec.md
sed -n '1,260p' specs/024-agent-boundary-security-hardening/spec.md
sed -n '1,260p' specs/025-step-level-intelligence-v2/spec.md
sed -n '1,220p' docs/2026-03-28-session-context-report.md
sed -n '1,220p' docs/plans/2026-03-27-runtime-planning-simplification.md
sed -n '100,240p' docs/research-output/analysis/2026-04-02-openclaude-transfer/04-priority-backlog.md
```

## Future implementation validation target

Once implementation starts, packet `026` should be validated with the narrowest honest checks for
the touched runtime and operator surfaces:

```bash
cargo test -p mister-smith-core
cargo test -p mister-smith-agents
cargo test -p mister-smith-events --test autonomy_event_tests
cargo test -p mister-smith-app --test autonomy_status_tests
npm --prefix apps/operator-console test
npm --prefix apps/operator-console run build
git diff --check
```

## Proof expectation

Packet `026` only earns deterministic implementation proof when one bounded scenario shows:

- visible coordinator-owned delegation
- visible subordinate inbox activity
- visible child state
- grounded delegated work evidence
- visible coordinator merge or recovery decisions
- explicit proof text on task result, autonomy status, and run detail

## Live-proof boundary

Packet `026` being implementation-ready does not mean a fresh live runtime proof already exists.
Any claim of real coordinator-subagent runtime on the supported live path still requires a later
bounded rerun after implementation lands.
