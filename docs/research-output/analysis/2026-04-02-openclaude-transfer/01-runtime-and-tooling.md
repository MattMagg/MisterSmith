# Runtime And Tooling Transfer Ideas

## 1. Schema Sanitizer For Tool And MCP Compatibility

**Verdict**

`KEEP with update`

**Source files**

- `src/utils/schemaSanitizer.ts`
- `src/services/api/openaiShim.ts`
- `src/services/api/codexShim.ts`

**What it is**

OpenClaude runs tool schemas through a compatibility pass before exposing them to model providers.
It strips unsupported keywords, cleans up `type`, and removes enum or const values that no longer
match the sanitized schema.

**Why it is useful for Mister Smith**

This fits Smith's existing packet-024 boundary work. It is not a provider-nicety feature. It is a
hardening seam that reduces brittle tool-call failures across ToolBus, MCP, and future remote
descriptor normalization.

**Concrete adaptation path**

- add a provider-facing schema compiler stage between capability discovery and model exposure
- keep both versions:
  - original schema for operator truth and debugging
  - sanitized schema for provider-facing execution
- connect it to packet `027` capability normalization instead of treating it as isolated LLM glue

**Risk and compatibility caveats**

- do not silently hide semantic loss
- the sanitizer must emit operator-visible provenance when it drops fields
- Smith should keep this boundary generic and not lock it to OpenAI-specific quirks only

## 2. Ordered Parallel Tool Execution With Deterministic Emission

**Verdict**

`KEEP with update`

**Source files**

- `src/services/tools/toolOrchestration.ts`
- `src/services/tools/StreamingToolExecutor.ts`

**What it is**

OpenClaude marks tool calls as concurrency-safe or not, runs safe batches in parallel, preserves
output order, and cancels sibling work when needed. The important detail is not just concurrency.
It is deterministic result emission plus explicit interrupt and sibling-abort behavior.

**Why it is useful for Mister Smith**

This is one of the cleanest runtime ideas to adapt into packet `026`. Smith already has ToolBus,
runtime-truth, and proof-boundary surfaces. Parallel work only helps if it stays explainable and
operator-visible.

**Concrete adaptation path**

- extend Smith's subordinate runtime with explicit tool-batch policy:
  - `serial`
  - `concurrency_safe`
- require deterministic event ordering on the run-trace side even when execution is parallel
- propagate sibling cancellation as first-class runtime truth instead of leaving it buried in logs

**Risk and compatibility caveats**

- do not let parallel execution bypass packet-023 truth surfaces
- do not make parallelism the default for work that does not justify it
- cancellation semantics must be explicit before this becomes a default runtime behavior

## 3. Long-Lived MCP Lifecycle Reconciliation

**Verdict**

`KEEP with update`

**Source files**

- `src/services/mcp/client.ts`
- `src/services/mcp/useManageMCPConnections.ts`
- `src/utils/mcpInstructionsDelta.ts`
- `src/tools/McpAuthTool/McpAuthTool.ts`

**What it is**

OpenClaude treats MCP as a changing runtime surface, not a one-time prompt dump. It handles
reconnects, `listChanged` refresh, auth-needed state, instruction deltas, and large-result offload.

**Why it is useful for Mister Smith**

This is a direct fit for packet `027`. Smith already has bounded MCP discovery and packet-024
capability enforcement. The next leverage is lifecycle health, trust-state clarity, and prompt-size
discipline.

**Concrete adaptation path**

- track connected, pending, and needs-auth MCP state in Smith capability truth
- refresh capability lists from server notifications instead of requiring full rediscovery
- project changed instructions and changed capabilities as deltas, not full prompt rebuilds
- offload very large MCP results into referenced artifacts and keep operator-visible provenance

**Risk and compatibility caveats**

- delta refresh must not hide capability removal or auth expiry
- persisted large results need durable references and cleanup rules
- operator views must show what changed, not just the latest snapshot

## 4. Remote Tool Placeholder And Unknown-Tool Permission Bridge

**Verdict**

`KEEP with update`

**Source files**

- `src/remote/remotePermissionBridge.ts`
- `src/remote/RemoteSessionManager.ts`

**What it is**

When a remote session asks for permission on a tool the local client does not know about,
OpenClaude creates a synthetic assistant message and a minimal tool stub so the operator can still
approve or deny the request in a structured way.

**Why it is useful for Mister Smith**

This is one of the best missed findings from the first pass. It is a protocol-boundary feature, not
a UI trick. Smith packet `027` will eventually have to deal with remote or normalized capabilities
that the local surface does not execute directly.

**Concrete adaptation path**

- define a Smith-side placeholder capability for remote-only or unknown tools
- let the operator see:
  - source capability name
  - raw input preview
  - trust and auth state
  - approval options
- keep approval separate from local execution support

**Risk and compatibility caveats**

- a placeholder must never imply local executability
- the operator must see that this is a foreign capability and what protocol source it came from
- this belongs to packet `027` protocol-boundary language, not packet `026` runtime semantics

## 5. Provider Request Resolver And Canonical Translation Layer

**Verdict**

`SPLIT or DEFER`

**Source files**

- `src/services/api/providerConfig.ts`
- `src/services/api/openaiShim.ts`
- `src/services/api/codexShim.ts`

**What it is**

OpenClaude centralizes provider request resolution and message or tool translation behind a smaller
internal shape.

**Why it is useful for Mister Smith**

This is still a reasonable runtime-plumbing idea, but it is not one of the strongest packet `026`
or `027` transfers. The first pass over-weighted it because it is easy to notice. Smith's current
frontier leverage is stronger runtime coordination and protocol boundary truth, not more provider
API breadth.

**Concrete adaptation path**

- treat this as later runtime hardening work inside `crates/mister-smith-llm/`
- if it lands, keep it narrow:
  - canonical internal envelopes
  - explicit transport selection
  - provider-specific code isolated from orchestrator logic

**Risk and compatibility caveats**

- do not widen packet `026` or `027` around provider churn
- do not mistake transport normalization for interoperability architecture

## 6. Provider-Aware Search And Fetch Brokers

**Verdict**

`REMOVE as misfit`

**Source files**

- `src/tools/WebSearchTool/WebSearchTool.ts`
- `src/tools/WebFetchTool/WebFetchTool.ts`

**What it is**

OpenClaude brokers web search and fetch across multiple backends.

**Why it is not a strong Smith transfer**

This is interesting product plumbing, but it is not a durable Mister Smith OS advantage right now.
It pushes the transfer set back toward framework-feature parity instead of coordination runtime,
capability boundary, or operator proof.

**Keep only the narrow lesson**

- if Smith broadens web surfaces later, keep search and fetch policy-distinct
- do not pull this into the current `026` or `027` packet shape
