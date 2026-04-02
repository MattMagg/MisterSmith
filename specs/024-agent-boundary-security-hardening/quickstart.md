# Quickstart: Agent-Boundary Security Hardening

## Packet-ready workflow

Use this packet in two steps:

1. finish the packet-authority gate
2. run the bounded code hardening and deterministic validation

Phase 0 is complete only when `spec.md`, `plan.md`, `research.md`, `data-model.md`, `quickstart.md`,
`analyze.md`, `contracts/`, `tasks.md`, and `checklists/requirements.md` all match current
`main`, and the checklist is `16/16`.

## Targeted implementation checks

Use the narrowest honest checks for the code and docs touched by packet `024`:

```bash
npx markdownlint-cli2 "specs/024-agent-boundary-security-hardening/**/*.md" --config .markdownlint.json
cargo test -p mister-smith-security --test delegation_tests --test auth_callout_tests --test quarantine_tests --test sandbox_tests
cargo test -p mister-smith-agents --test tool_bus_tests --test quarantine_tests --test sandbox_tests
cargo test -p mister-smith-mcp
cargo test -p mister-smith-persistence
cargo build --workspace
git diff --check
scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync
```

## Proof expectation

Packet `024` is done when the checks above prove:

- discover and execute remain separate across ToolBus and MCP boundaries
- action-bound execution rejects mismatched, revoked, or descriptorless delegated authority before
  handler execution
- cross-boundary and shared-state payloads are deterministically validated before agent
  consumption
- sanitized and monitored suspicious outcomes always carry deterministic reasons
- auth-callout fallback stays at or below the current quarantined posture even when broader
  defaults are configured
- packet `016` continuity remains intact without inventing a workflow-backed live reject surface

## Proof boundary

This packet reports deterministic hardening only.

Do not claim:

- a new generic IAM program
- a new interoperability protocol claim
- a broader compliance or observability claim
- a new workflow-backed live reject surface unless the runtime actually grows one
- a new packet-022-owned live rerun or a broader runtime-truth packet
