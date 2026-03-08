# Contract: Tool-Calling Bridge

## Overview

The tool-calling bridge connects provider-neutral LLM tool requests to the existing `ToolBus`. It is
the only supported path for model-requested tool execution in Phase 9.

## Source Map

| Source | Contract impact |
| ------ | --------------- |
| `spec/data-management/agent-orchestration.md` §10.4 | Grounds LLM coordination and subject-routing context without importing deferred parser work. |
| `spec/data-management/message-schemas.md` §5 | Confirms `llm.hooks.*` subjects remain out of scope for the tool bridge. |
| `spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md` §15 | Keeps Neural/AI Operations tool workflows out of this contract. |
| `spec/core-architecture/type-definitions.md` | Anchors shared Tool and Agent identifiers plus result and error conventions. |
| `spec/core-architecture/async-patterns.md` | Provides the primary ToolBus, permission, timeout, and agent-as-tool patterns this bridge extends. |
| `spec/core-architecture/coding-standards.md` | Requires explicit permission checks, timeout handling, audit posture, typed errors, and tests. |

## Export Contract

The ToolBus must expose currently registered tools as provider-neutral definitions:

```rust
impl ToolBus {
    pub fn to_tool_definitions(&self) -> Vec<ToolDefinition>;
}
```

Each exported definition must include:

- stable tool name
- human-readable description
- JSON Schema input contract

## Execution Contract

The ToolBus must execute a model-emitted tool request through the same boundary used for ordinary
tool invocation:

```rust
impl ToolBus {
    pub async fn execute_tool_call(
        &self,
        /* caller context */,
        call: &ToolCall,
    ) -> Result<ToolResult, AgentSystemError>;
}
```

The exact Rust signature may carry caller or audit context, but the behavior is fixed.

## Required Execution Flow

1. Resolve the requested tool from the existing ToolBus registry.
2. Perform the same permission checks required for ordinary tool execution.
3. Enforce the same timeout policy used by the ToolBus.
4. Dispatch to the existing backing implementation:
   - native agent-backed tool -> current agent invocation path
   - MCP-backed tool -> current MCP invocation path
5. Return a structured `ToolResult`.
6. Preserve metrics and audit behavior already required at the ToolBus boundary.

## Error Contract

The bridge must preserve current ToolBus semantics for:

- tool not found
- permission denied
- timeout
- execution failure
- tool unavailable

Provider adapters may convert the final result into provider-specific follow-up payloads internally,
but the bridge itself returns provider-neutral `ToolResult` and shared agent errors.

## Explicit Non-Scope

The bridge must not:

- introduce a second provider-specific tool registry
- bypass ToolBus permission checks
- bypass ToolBus timeout behavior
- bypass ToolBus audit or metrics boundaries
- rely on hook events or `llm.hooks.*` subjects

## Routing and Stream Integration

Tool-call events are classified as **lossless** in the backpressure policy matrix. They flow
through the semantic stream (JetStream) with guaranteed delivery. This means:

- `ModelEvent::ToolCallStart`, `ToolCallDelta`, and `ToolCallCompleted` must never be coalesced
  or dropped under backpressure
- Tool calls route through the `ModelRouter`'s data plane (NATS request-reply)
- The `ModelRouter` applies budget enforcement to tool-call-bearing requests the same way it
  handles completion requests

Tool-call serialization must support both OpenAI function-calling format (`tools` array with
`function` type) and Anthropic tool-use format (`tool_use` content blocks). The unified
`ToolDefinition` type abstracts this difference at the provider boundary.

## Validation Requirements

- unit tests for tool-definition export shape
- unit or integration tests for successful tool-call execution
- negative tests for permission denial and timeout behavior
- end-to-end Gate 9 coverage showing model -> ToolBus -> model round-trips
- verification that tool-call events are lossless (never coalesced/dropped) under backpressure
