# Quickstart: Agent-Boundary Security Hardening

## Draft Status

This quickstart is part of draft packet scaffolding.

- packet `024` is being scaffolded before earlier packets are fully complete
- before implementation, refresh the packet against the then-current `docs/current-state.md`,
  `docs/direction.md`, and newly landed earlier packet artifacts

## Targeted deterministic validation

Run the narrowest honest checks for a future packet `024` implementation:

```bash
cargo test -p mister-smith-security
cargo test -p mister-smith-agents --test tool_bus_tests
cargo test -p mister-smith-agents --test quarantine_tests
cargo test -p mister-smith-mcp
cargo test -p mister-smith-persistence
cargo build --workspace
git diff --check
npx markdownlint-cli2 "specs/024-agent-boundary-security-hardening/**/*.md" --config .markdownlint.json
```

## Proof expectation

Packet `024` earns proof by showing:

- discover and execute remain separate across ToolBus and MCP boundaries
- delegated action mismatch, missing delegation, and revocation are rejected before handler
  execution
- cross-boundary and shared-state payloads are deterministically validated before agent
  consumption
- auth-callout fallback stays on the current minimal quarantined posture
- packet `016` continuity remains intact without inventing a workflow-backed live reject surface

## Live-proof boundary

This scaffolding pass does not create a new live runtime proof claim.

If a later implementation captures live proof, it must stay bounded to the actual runtime surface
used and must not imply:

- a new generic IAM program
- a new interoperability protocol claim
- a broader compliance or observability claim
- a new workflow-backed live reject surface unless the runtime actually grows one
