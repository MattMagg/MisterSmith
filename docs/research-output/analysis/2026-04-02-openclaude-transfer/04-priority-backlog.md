# Priority Backlog For Mister Smith

## Recommended Order

| Priority | Idea | Fit | Why this order | Likely Smith surfaces |
| --- | --- | --- | --- | --- |
| 1 | Provider request resolver plus canonical translation layer | High fit now | Hardens the current runtime path without changing product claims | `crates/mister-smith-llm/`, `crates/mister-smith-config/` |
| 2 | Schema sanitizer for tool and MCP compatibility | High fit now | Reduces fragile tool-call failures and fits packet-024 hardening | `crates/mister-smith-mcp/`, `crates/mister-smith-agents/`, `crates/mister-smith-llm/` |
| 3 | MCP lifecycle reconciliation plus large-result offload | High fit now | Makes shipped MCP capability more durable in long sessions | `crates/mister-smith-mcp/`, `crates/mister-smith-app/`, `apps/operator-console/` |
| 4 | Real plan mode | High fit now | Matches repo planning posture and existing retained-session surfaces | `crates/mister-smith-app/`, `apps/operator-console/` |
| 5 | Live work cockpit | High fit now | Improves operator control over current runs and sessions | `apps/operator-console/`, `crates/mister-smith-app/` |
| 6 | Surface-specific action gating | High fit now | Strengthens safety across CLI, HTTP, and console surfaces | `crates/mister-smith-app/`, `crates/mister-smith-http/`, `apps/operator-console/` |
| 7 | Ordered parallel tool execution | Conditional fit next | Good leverage, but should extend the current runtime-truth contract carefully | `crates/mister-smith-agents/`, `crates/mister-smith-events/` |
| 8 | Stable child-agent identity and subordinate execution units | Conditional fit next | High upside for later coordinator runtime work | `crates/mister-smith-agents/`, `crates/mister-smith-events/`, `crates/mister-smith-persistence/` |
| 9 | Cross-worker approval mailbox | Conditional fit next | Strong later fit with packet-024 posture | `crates/mister-smith-security/`, `crates/mister-smith-agents/`, `crates/mister-smith-events/` |
| 10 | Resumable remote executors | Later | Valuable, but it belongs after stronger local coordinator truth exists | later packet beyond current default path |

## Suggested Near-Term Epics

## Epic A: Runtime Compatibility Hardening

**Contents**

- provider request resolver
- canonical message and tool translation
- schema sanitizer

**Why now**

This hardens the current runtime path without widening product claims or forcing a new runtime
shape.

**Success signal**

- fewer provider-specific branches in the main runtime loop
- provider adapters covered by targeted tests
- no regression in packet-019 through packet-024 proof boundaries

## Epic B: MCP Durability Hardening

**Contents**

- server-state reconciliation
- auth-needed placeholders
- large-result offload
- incremental capability instruction deltas

**Why now**

MCP is already part of current Smith truth. Durability matters more than adding one more server.

**Success signal**

- reconnects do not silently degrade the user experience
- large MCP outputs stay usable without prompt bloat
- auth failures are visible and recoverable

## Epic C: Operator Session Control

**Contents**

- real plan mode
- session notes
- live work cockpit
- surface-specific action gating

**Why now**

These extend current CLI, session, and operator-console surfaces that already exist on `main`.

**Success signal**

- operators can move a retained session from planning to execution explicitly
- the console can inspect and control live work from one place
- out-of-scope actions get rejected clearly per surface

## Ideas To Keep Later

- stable child-agent inbox and continue-in-place messaging
- subordinate execution units under one workflow
- remote resumable executors
- plugin-marketplace style discovery
- voice input

These ideas are useful, but they fit better after the current runtime path is harder, more
observable, and more honestly proven.
