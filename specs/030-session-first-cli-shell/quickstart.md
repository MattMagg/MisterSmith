# Quickstart: Session-First CLI Shell

## Packet Readiness Validation

Run these checks to confirm the CLI-only packet artifacts are present and internally consistent:

```bash
SPECIFY_FEATURE=030-session-first-cli-shell ./.specify/scripts/bash/check-prerequisites.sh --json --paths-only
SPECIFY_FEATURE=030-session-first-cli-shell ./.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks
npx markdownlint-cli2 "specs/030-session-first-cli-shell/**/*.md" --config .markdownlint.json
git diff --check
```

## Current Authority Inputs

Review these before implementation starts:

- `docs/current-state.md`
- `docs/plans/2026-04-05-session-first-user-shell-pre-speckit-primer.md`
- `docs/plans/2026-04-05-mister-smith-operational-cli-proposal.md`
- `crates/mister-smith-app/src/main.rs`
- `crates/mister-smith-app/src/conversation.rs`
- `crates/mister-smith-http/src/server.rs`

## Future Implementation Validation Target

When implementation begins, the narrowest meaningful validation target for this packet is:

```bash
cargo test -p mister-smith-app
cargo test -p mister-smith-http
cargo build --workspace
SPECIFY_FEATURE=030-session-first-cli-shell ./.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks
npx markdownlint-cli2 "specs/030-session-first-cli-shell/**/*.md" --config .markdownlint.json
git diff --check
scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync
```

## Proof Boundary

- This packet is about CLI shell posture, retained-session flows, and in-session steering.
- Deterministic CLI validation is required before any broader runtime-proof claim.
- Broader live runtime proof remains a separate boundary from this CLI-shell packet unless later
  implementation explicitly lands and validates such work.
