# Quickstart: Chat-First CLI Loop

## Packet readiness validation

Use these checks to validate the packet bundle before implementation begins:

```bash
SPECIFY_FEATURE=031-chat-first-cli-loop \
  ./.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks
npx markdownlint-cli2 \
  "specs/031-chat-first-cli-loop/**/*.md" \
  --config .markdownlint.json
git diff --check
```

Implementation should not start until both packet checklists have zero incomplete items and the
blocking freeze tasks `T001` through `T003` are complete.

## Current authority inputs

Before implementation starts, re-read these sources in this order:

```bash
sed -n '1,280p' docs/current-state.md
sed -n '1,220p' docs/direction.md
sed -n '1,220p' specs/030-session-first-cli-shell/spec.md
sed -n '430,980p' crates/mister-smith-app/src/main.rs
sed -n '943,1765p' crates/mister-smith-app/src/conversation.rs
sed -n '240,380p' crates/mister-smith-http/src/server.rs
```

## Future implementation validation target

Once implementation starts, validate packet `031` with the narrowest honest checks for the
changed CLI session surfaces:

```bash
cargo test -p mister-smith-app
cargo test -p mister-smith-http
cargo build --workspace
git diff --check
```

This quickstart distinguishes packet-freeze readiness from implementation closure. Passing the
packet readiness gate does not claim that any product code or live-proof rerun has happened yet.

## Proof expectation

Packet `031` earns deterministic implementation proof when one bounded CLI scenario shows:

- open or resume into one active session loop
- send multiple follow-up turns without leaving that loop
- show inline accepted, active, completed, failed, or blocked turn state
- preserve retained context and stored controls on resume
- keep degraded or proof-limited states visible in user language

## Live-proof boundary

This packet bundle does not claim a fresh live runtime proof. Any future claim that the chat-first
CLI loop is live-proven on the supported `openai_chatgpt` / `gpt-5.4` baseline still requires a
later bounded rerun after implementation lands.
