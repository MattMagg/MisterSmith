# Runtime And Tooling Transfer Ideas

## 1. Provider Request Resolver

**OpenClaude feature**

One resolver decides the outbound transport, model alias, reasoning setting, base URL, and
credential source before the rest of the runtime touches the provider call.

**OpenClaude evidence**

- `src/services/api/providerConfig.ts`
- `src/services/api/client.ts`
- `src/services/api/openaiShim.ts`
- `src/services/api/codexShim.ts`

**Why it matters**

This keeps provider churn out of the main agent loop. The runtime talks to one internal request
shape instead of branching everywhere on provider quirks.

**Mister Smith fit**

`High fit now`

Mister Smith already has provider selection and routing in `crates/mister-smith-llm/`, but the
repo direction is still to harden the current runtime path rather than widen ad hoc provider
branches.

**How to translate into Mister Smith**

- add one resolver in `crates/mister-smith-llm/` that turns config plus task intent into a
  canonical internal request and a target transport enum
- keep the transport enum explicit, for example:
  - `AnthropicMessages`
  - `OpenAIChatCompletions`
  - `OpenAIResponses`
  - `VendorCompatible`
- keep provider credential lookup and model alias logic inside the resolver, not inside the main
  workflow executor

**Suggested validation**

- targeted unit tests for model alias resolution and transport selection
- one provider-contract test per shipped provider path
- no change to current packet-019 live-proof claims unless a real rerun happens

## 2. Canonical Message And Tool Translation Layer

**OpenClaude feature**

One internal conversation shape gets translated into provider-specific payloads and streamed back
into one internal event shape, including tool calls, tool results, usage, and finish state.

**OpenClaude evidence**

- `src/services/api/openaiShim.ts`
- `src/services/api/codexShim.ts`
- `src/services/api/openaiShim.test.ts`
- `src/services/api/codexShim.test.ts`

**Why it matters**

Provider APIs change. Tool-call wire formats change. The runtime should not have those details
spread across orchestrator code.

**Mister Smith fit**

`High fit now`

This matches the current direction to harden routing, proof boundaries, and operator-visible truth
without widening architectural sprawl.

**How to translate into Mister Smith**

- define one Smith-native turn envelope in `crates/mister-smith-llm/` or a small companion module
  for:
  - text blocks
  - image or attachment references where supported
  - tool calls
  - tool results
  - usage and finish reasons
- keep all provider adapters behind that envelope
- feed packet-023 runtime-truth and run-trace views from the canonical envelope, not from
  provider-specific raw events

**Suggested validation**

- adapter round-trip tests for tool-call and tool-result conversion
- stream-event normalization tests
- regression tests for usage accounting and finish-reason mapping

## 3. Schema Sanitizer For Tool And MCP Compatibility

**OpenClaude feature**

Tool and MCP schemas go through a compatibility pass that strips unsupported fields, normalizes
enums and defaults, and fixes provider-specific schema problems before the model sees them.

**OpenClaude evidence**

- `src/utils/schemaSanitizer.ts`
- `src/services/api/codexShim.ts`
- `src/services/api/codexShim.test.ts`

**Why it matters**

Tool calling fails for small schema mismatches more often than for major logic bugs. This is a
hardening layer, not a nice-to-have.

**Mister Smith fit**

`High fit now`

Packet `024` already hardened capability boundaries. A schema-compat compiler stage is a natural
next hardening seam for `mister-smith-mcp` plus ToolBus.

**How to translate into Mister Smith**

- add a provider-compat schema pass between:
  - `crates/mister-smith-mcp/` tool discovery
  - `crates/mister-smith-agents/` or ToolBus dispatch
  - `crates/mister-smith-llm/` outbound provider calls
- track both:
  - original schema
  - sanitized provider-facing schema
- surface sanitizer changes in debug or operator inspection so failures stay explainable

**Suggested validation**

- unit tests for malformed enums, unsupported URI formats, and invalid defaults
- integration tests proving the same tool can be exposed to more than one provider format

## 4. Ordered Parallel Tool Execution

**OpenClaude feature**

The runtime marks tools as concurrency-safe or serial-only, runs safe batches in parallel, and
still emits results in a deterministic order.

**OpenClaude evidence**

- `src/services/tools/toolOrchestration.ts`
- `src/services/tools/StreamingToolExecutor.ts`

**Why it matters**

This is a strong pattern for speeding up tool-heavy steps without making result order or operator
explanations chaotic.

**Mister Smith fit**

`Conditional fit next`

Mister Smith already has a ToolBus-backed runtime path. This idea fits well, but it should land as
an explicit extension of the current step/runtime truth contract, not as an invisible execution
change.

**How to translate into Mister Smith**

- add one concurrency flag per ToolBus action class:
  - `serial`
  - `concurrency_safe`
- run safe actions in ordered parallel batches inside one workflow step
- preserve one final ordered event stream for packet-023 run-trace and autonomy status views

**Suggested validation**

- deterministic ordering tests with mixed serial and safe tools
- cancellation tests
- proof that packet-023 runtime-truth summaries stay stable after parallelization

## 5. Provider-Aware Search And Fetch Brokers

**OpenClaude feature**

Search and fetch are brokered across provider-native search, Codex search, Firecrawl, and raw HTTP
fetch, with permission rules, redirect handling, and second-pass content extraction.

**OpenClaude evidence**

- `src/tools/WebSearchTool/WebSearchTool.ts`
- `src/tools/WebFetchTool/WebFetchTool.ts`
- `src/tools/WebFetchTool/utils.ts`

**Why it matters**

“Web access” is not one thing. Search, scrape, and authenticated retrieval have different cost,
policy, and reliability profiles.

**Mister Smith fit**

`Conditional fit next`

Useful if Smith broadens web-capability surfaces, but the bigger lesson is architectural: treat
search and fetch as policy-brokered capabilities, not one hardcoded backend.

**How to translate into Mister Smith**

- model two capability classes:
  - `search broker`
  - `fetch broker`
- let policy choose backend by:
  - auth requirement
  - source type
  - JS-rendering need
  - cost or quota posture
- keep boundary decisions visible in packet-024 style capability reporting

**Suggested validation**

- backend-selection tests
- redirect and domain-policy tests
- operator-visible provenance showing which broker path was used

## 6. Long-Lived MCP Lifecycle Management

**OpenClaude feature**

OpenClaude reconnects MCP servers, reacts to capability-list changes, inserts auth placeholder
tools, and avoids re-sending the entire MCP instruction block every turn.

**OpenClaude evidence**

- `src/services/mcp/client.ts`
- `src/services/mcp/useManageMCPConnections.ts`
- `src/services/mcp/InProcessTransport.ts`
- `src/utils/mcpInstructionsDelta.ts`
- `src/tools/McpAuthTool/McpAuthTool.ts`
- `src/tools/ListMcpResourcesTool/ListMcpResourcesTool.ts`
- `src/tools/ReadMcpResourceTool/ReadMcpResourceTool.ts`

**Why it matters**

This is the difference between “MCP exists” and “MCP stays healthy in a real long-running session.”

**Mister Smith fit**

`High fit now`

Mister Smith already ships `mister-smith-mcp` and packet `024` boundary work. Lifecycle
reconciliation, large-result offload, and incremental instruction deltas are a natural hardening
next step.

**How to translate into Mister Smith**

- keep live server-state reconciliation in `crates/mister-smith-mcp/`
- add explicit auth-needed capability placeholders instead of opaque failure states
- offload large MCP results outside prompt context and project them as references in runtime truth
- avoid rebuilding the whole capability prompt surface every turn when only a small delta changed

**Suggested validation**

- reconnect and capability-list-changed tests
- large-result offload tests
- auth-expiry and reauth placeholder tests
