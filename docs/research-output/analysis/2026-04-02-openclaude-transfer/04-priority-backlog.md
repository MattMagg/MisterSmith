# Frontier Second-Pass Review

## New Findings

### 1. Scoped mid-turn subordinate event intake is a real packet-026 input

- Source files:
  - `src/query.ts`
  - `src/tasks/LocalAgentTask/LocalAgentTask.tsx`
- What it is:
  - OpenClaude drains queued subordinate notifications into the same turn and scopes them by
    agent identity.
- Why Smith should care:
  - This gives packet `026` a concrete way to make delegated work visible without faking a new
    conversation per child.
- Adaptation path:
  - add a bounded subordinate-runtime inbox under the parent workflow and project it into
    packet-023 truth surfaces.
- Caveat:
  - this must stay subordinate-event intake only, not a hidden prompt side channel.

### 2. Deterministic parallel tool batches matter more than "parallelism"

- Source files:
  - `src/services/tools/toolOrchestration.ts`
  - `src/services/tools/StreamingToolExecutor.ts`
- What it is:
  - OpenClaude batches concurrency-safe tool calls, preserves result order, and propagates sibling
    cancellation and interrupt behavior explicitly.
- Why Smith should care:
  - This is the strongest OpenClaude execution-runtime pattern that still fits Smith's frontier
    mandate.
- Adaptation path:
  - make packet `026` define ordered parallel subordinate execution with explicit cancellation
    truth.
- Caveat:
  - parallel work should stay opt-in and bounded by the smallest-workflow rule.

### 3. MCP lifecycle health is bigger than simple tool discovery

- Source files:
  - `src/services/mcp/client.ts`
  - `src/services/mcp/useManageMCPConnections.ts`
  - `src/utils/mcpInstructionsDelta.ts`
- What it is:
  - OpenClaude handles reconnects, `listChanged`, auth-needed state, delta instruction updates,
    and large-result persistence.
- Why Smith should care:
  - This is directly useful for packet `027` because Smith already has bounded MCP discovery and
    packet-024 execution boundaries.
- Adaptation path:
  - add lifecycle, auth, and large-result handling to Smith capability truth rather than widening
    protocol count first.
- Caveat:
  - discovery refresh must not blur trust or execution state.

### 4. Unknown remote tools need a protocol-boundary placeholder

- Source files:
  - `src/remote/remotePermissionBridge.ts`
  - `src/remote/RemoteSessionManager.ts`
- What it is:
  - OpenClaude can present a remote permission request even when the local client does not
    implement that tool.
- Why Smith should care:
  - This is a missing piece in the first-pass analysis and a good packet `027` boundary rule.
- Adaptation path:
  - add a placeholder capability type for remote-only tools with source, auth, and approval
    metadata.
- Caveat:
  - placeholder does not equal executable.

### 5. Surface-specific command gates are stronger than a generic command palette

- Source files:
  - `src/utils/processUserInput/processUserInput.ts`
  - `src/utils/processUserInput/processSlashCommand.tsx`
- What it is:
  - OpenClaude gives different command powers to different surfaces and explicitly re-routes some
    results back into the main loop.
- Why Smith should care:
  - This is operator safety and protocol-boundary leverage, not UI polish.
- Adaptation path:
  - define explicit surface allowlists across CLI, HTTP, cockpit, and later remote surfaces.
- Caveat:
  - synthetic re-entry messages need visible provenance.

## Revisit Verdict On Old Findings

| Existing idea | Verdict | Why |
| --- | --- | --- |
| Provider request resolver | `SPLIT or DEFER` | real plumbing, but not top frontier leverage for packet `026` or `027` |
| Canonical message and tool translation | `SPLIT or DEFER` | useful later for runtime hardening, but not the strongest transfer in this packet range |
| Schema sanitizer | `KEEP with update` | strong packet `027` hardening seam |
| Ordered parallel tool execution | `KEEP with update` | keep it, but capture deterministic emission and cancellation rules |
| Provider-aware search and fetch brokers | `REMOVE as misfit` | too close to framework parity |
| Long-lived MCP lifecycle management | `KEEP with update` | stronger and more packet-relevant than the first pass stated |
| Real plan mode | `KEEP with update` | useful supporting safety feature, but not a main frontier differentiator |
| Live work cockpit | `KEEP with update` | good operator surface for packet `026` child runtime visibility |
| Unified command palette plus helpers | `REMOVE as misfit` | UX convenience, not current Smith leverage |
| Durable session memory and notes | `KEEP with update` | keep only the visible continuity-note part |
| Capability catalog with trust labels | `KEEP with update` | packet `027` operator clarity, not plugin UX |
| IDE bridge, keybindings, and voice | `REMOVE as misfit` | convenience features, not packet work |
| Turn loop with mid-turn notification drain | `KEEP with update` | one of the best packet `026` inputs |
| Stable child-agent identity | `KEEP with update` | strong coordination-runtime fit |
| Subagent context isolation with shared root channels | `KEEP as-is` | already a strong fit |
| Unified subordinate task runtime | `KEEP with update` | good if kept under existing workflow truth |
| Secret-minimized remote bridge | `SPLIT or DEFER` | good later remote-executor reference |
| Specialist child roles with enforced tool subsets | `KEEP with update` | strong packet `026` safety input |
| Cross-worker permission mailbox | `KEEP with update` | strong, but later in the packet chain than baseline child runtime truth |
| Surface-specific command gating | `KEEP with update` | good packet `027` boundary rule |

## 026/027 Recommended Edits

### Packet 026: First Real Coordinator-Subagent Runtime

#### 1. Add a bounded subordinate-runtime inbox and event-drain rule

- Why:
  - `frontier leverage`
  - `implementation correctness`
- Edit paths:
  - `/Users/macmain/MisterSmith/specs/026-first-real-coordinator-subagent-runtime/spec.md`
  - `/Users/macmain/MisterSmith/specs/026-first-real-coordinator-subagent-runtime/research.md`
  - `/Users/macmain/MisterSmith/specs/026-first-real-coordinator-subagent-runtime/contracts/coordinator-subagent-runtime-contract.md`
- Add:
  - a coordinator-owned event intake path for child completions, clarification requests, and
    subordinate notifications
  - scoped delivery rules so each child only receives its own event stream

#### 2. Clarify that delegated work units need stable child identity and follow-up actions

- Why:
  - `frontier leverage`
  - `operator clarity`
- Edit paths:
  - `/Users/macmain/MisterSmith/specs/026-first-real-coordinator-subagent-runtime/spec.md`
  - `/Users/macmain/MisterSmith/specs/026-first-real-coordinator-subagent-runtime/data-model.md`
  - `/Users/macmain/MisterSmith/specs/026-first-real-coordinator-subagent-runtime/contracts/coordinator-subagent-runtime-contract.md`
- Add:
  - stable delegated-work identity under the parent workflow
  - allowed follow-up actions:
    - clarify
    - resume
    - stop
    - inspect

#### 3. Add subagent context-isolation and shared-root-channel rules

- Why:
  - `protocol boundary`
  - `implementation correctness`
- Edit paths:
  - `/Users/macmain/MisterSmith/specs/026-first-real-coordinator-subagent-runtime/spec.md`
  - `/Users/macmain/MisterSmith/specs/026-first-real-coordinator-subagent-runtime/research.md`
  - `/Users/macmain/MisterSmith/specs/026-first-real-coordinator-subagent-runtime/contracts/coordinator-subagent-runtime-contract.md`
- Add:
  - child scratch state is private by default
  - only root-owned channels may be shared:
    - registration
    - cancellation
    - runtime-truth projection
    - capability enforcement

#### 4. Add deterministic ordered parallel tool batches with sibling-abort semantics

- Why:
  - `frontier leverage`
  - `implementation correctness`
- Edit paths:
  - `/Users/macmain/MisterSmith/specs/026-first-real-coordinator-subagent-runtime/spec.md`
  - `/Users/macmain/MisterSmith/specs/026-first-real-coordinator-subagent-runtime/plan.md`
  - `/Users/macmain/MisterSmith/specs/026-first-real-coordinator-subagent-runtime/research.md`
- Add:
  - concurrency-safe versus serial subordinate execution
  - deterministic result ordering
  - explicit sibling-cancel and user-interrupt outcome projection

#### 5. Add role-bounded child execution instead of prompt-only child specialization

- Why:
  - `protocol boundary`
  - `operator clarity`
- Edit paths:
  - `/Users/macmain/MisterSmith/specs/026-first-real-coordinator-subagent-runtime/spec.md`
  - `/Users/macmain/MisterSmith/specs/026-first-real-coordinator-subagent-runtime/data-model.md`
- Add:
  - child role definitions with explicit tool or capability subsets
  - start with a small set:
    - explorer
    - planner
    - verifier

### Packet 027: Capability Discovery And Interoperability

#### 6. Expand the normalized capability descriptor to carry trust, auth, and execution-state separation

- Why:
  - `protocol boundary`
  - `operator clarity`
- Edit paths:
  - `/Users/macmain/MisterSmith/specs/027-capability-discovery-and-interoperability/spec.md`
  - `/Users/macmain/MisterSmith/specs/027-capability-discovery-and-interoperability/data-model.md`
  - `/Users/macmain/MisterSmith/specs/027-capability-discovery-and-interoperability/contracts/capability-normalization-contract.md`
- Add:
  - discovery metadata
  - trust state
  - auth state
  - execution-availability reference
  - remote-placeholder capability for tools the local surface cannot execute directly

#### 7. Add lifecycle health rules for MCP capability surfaces

- Why:
  - `implementation correctness`
  - `operator clarity`
- Edit paths:
  - `/Users/macmain/MisterSmith/specs/027-capability-discovery-and-interoperability/spec.md`
  - `/Users/macmain/MisterSmith/specs/027-capability-discovery-and-interoperability/research.md`
  - `/Users/macmain/MisterSmith/specs/027-capability-discovery-and-interoperability/plan.md`
  - `/Users/macmain/MisterSmith/specs/027-capability-discovery-and-interoperability/contracts/capability-normalization-contract.md`
- Add:
  - reconnect and refresh expectations
  - `listChanged`-style capability delta handling
  - auth-needed placeholder state
  - large-result offload with durable reference and provenance

## Deferment List

- secret-minimized remote worker bridge with explicit env allowlist and session access token
- plugin-style capability marketplace
- command-palette parity work
- IDE bridge and voice support
- provider-specific search and fetch broker expansion
- broad provider resolver work as a packet `026` or `027` deliverable

These should wait because they either belong to a later remote-executor packet or they pull Smith
away from its stronger coordination-runtime and capability-boundary leverage.

## Risks And Sequencing

### Why this should be in Smith now

- packet `026` needs better real subordinate-runtime truth, and OpenClaude has useful patterns for
  event intake, child identity, and bounded role execution
- packet `027` needs stronger capability-boundary clarity, and OpenClaude has useful patterns for
  lifecycle refresh, auth state, placeholder capabilities, and surface gating
- these changes harden already-landed Smith foundations instead of widening product scope

### Why some ideas should wait

- remote bridge isolation is strong, but Smith should prove a stronger local coordinator-subagent
  runtime before taking on remote child execution complexity
- provider and search-broker work is real, but it is not the best frontier move while packet
  `026` and `027` are still scaffold-level
- plugin and UX-shell features are easy to copy and easy to regret

### Recommended sequence

1. refresh packet `026` around subordinate-runtime truth, child identity, and bounded role rules
2. refresh packet `027` around normalized capability state, MCP lifecycle health, and
   discovery-versus-execute clarity
3. leave remote bridge and broader runtime-transport work for a later packet after `026` has real
   proof
