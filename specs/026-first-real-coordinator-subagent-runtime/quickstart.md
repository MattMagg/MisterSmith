# Quickstart: First Real Coordinator-Subagent Runtime

## Current scaffold validation

Use these checks to validate the scaffold artifact set now:

```bash
./.specify/scripts/bash/check-prerequisites.sh --json
npx markdownlint-cli2 "specs/026-first-real-coordinator-subagent-runtime/**/*.md" --config .markdownlint.json
git diff --check
```

## Required pre-implementation refresh pass

Before implementation starts, rerun this refresh sequence:

```bash
sed -n '1,260p' docs/current-state.md
sed -n '1,220p' docs/direction.md
sed -n '1,260p' docs/packet-prep/022-durable-workflow-core.md
sed -n '1,260p' docs/packet-prep/023-runtime-truth-and-run-trace.md
sed -n '1,260p' docs/packet-prep/024-agent-boundary-security-hardening.md
sed -n '1,260p' docs/packet-prep/025-step-level-intelligence-v2.md
./.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks
```

Then revise:

- `spec.md`
- `plan.md`
- `tasks.md`
- `analyze.md`
- `contracts/coordinator-subagent-runtime-contract.md`

## Future implementation validation target

Once the revision gate is complete and implementation actually begins, the packet should be
validated with the narrowest honest checks for the touched runtime and operator surfaces:

```bash
cargo test -p mister-smith-core
cargo test -p mister-smith-agents
cargo test -p mister-smith-events
cargo test -p mister-smith-app
npm --prefix apps/operator-console test
npm --prefix apps/operator-console run build
git diff --check
```

## Proof expectation

Packet `026` only earns implementation proof when one bounded run shows:

- visible coordinator-owned delegation
- visible subagent state
- grounded delegated work evidence
- visible coordinator merge or recovery decisions
- explicit proof text on task result, autonomy status, and run detail

## Live-proof boundary

This scaffold does not claim implementation proof or live runtime proof. Those claims stay blocked
until the pre-implementation refresh pass is complete and the later implementation work is
actually validated.
