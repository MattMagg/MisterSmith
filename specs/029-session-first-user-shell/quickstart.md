# Quickstart: Session-First User Shell

## Packet readiness validation

Use these checks to validate the current packet artifacts before implementation starts:

```bash
SPECIFY_FEATURE=029-session-first-user-shell \
  ./.specify/scripts/bash/check-prerequisites.sh --json --paths-only
npx markdownlint-cli2 \
  "specs/029-session-first-user-shell/**/*.md" \
  --config .markdownlint.json
git diff --check
```

## Current authority inputs

Before implementation work begins, re-read these sources in this order:

```bash
sed -n '1,260p' docs/current-state.md
sed -n '1,260p' docs/plans/2026-04-05-session-first-user-shell-pre-speckit-primer.md
sed -n '1,260p' docs/plans/2026-04-05-mister-smith-operational-cli-proposal.md
sed -n '1,240p' specs/029-session-first-user-shell/spec.md
sed -n '1,260p' specs/029-session-first-user-shell/plan.md
sed -n '1,240p' specs/029-session-first-user-shell/contracts/session-shell-contract.md
sed -n '1,240p' specs/029-session-first-user-shell/contracts/shared-session-protocol-contract.md
```

## Future implementation validation target

Once implementation begins, validate the packet with the narrowest honest checks for the touched
session and desktop surfaces:

```bash
cargo test -p mister-smith-app
cargo test -p mister-smith-http
npm --prefix apps/operator-console test
npm --prefix apps/operator-console run build
git diff --check
```

Add narrower packet-specific tests once the task list freezes the exact write set.

## Proof expectation

This packet earns deterministic implementation proof when one bounded scenario shows:

- `mister-smith` opens into a recent-first home instead of a runtime-first entry
- recent sessions and resume-last are visible at startup
- a live session can be steered in place for model, permissions, config, status, and MCP
- a retained session can move between CLI and GUI without losing the same session identity and
  retained history
- degraded support state remains visible without taking over the main product path

## Live-proof boundary

This packet is about product-shell posture and shared session continuity. It does not by itself
create a broader live runtime-proof claim beyond the current bounded runtime-proof baseline already
documented in the repo.
