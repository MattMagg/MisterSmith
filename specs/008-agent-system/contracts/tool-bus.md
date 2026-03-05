# Contract: Tool Bus

## Overview

The ToolBus is the central registry and invocation proxy for all tools in the system. It provides a uniform interface for both native agent-backed tools and MCP-compatible external tools.

## Registration

### Register Native Tool

```
ToolBus::register(name, namespace, agent_ref, schema, capabilities) -> Result<(), ToolBusError>
```

- `name` + `namespace` must be unique
- `agent_ref` is an `ActorRef` to the backing agent
- Emits `ToolRegistered` event on EventBus

### Register MCP Tool

```
ToolBus::register_mcp(name, namespace, mcp_session, schema, capabilities) -> Result<(), ToolBusError>
```

- Same uniqueness constraint
- `mcp_session` identifies the MCP server connection
- Emits `ToolRegistered` event

### Deregister

```
ToolBus::deregister(namespace, name) -> Result<(), ToolBusError>
```

- Removes tool from registry
- In-flight invocations are not cancelled (they complete or timeout)
- Emits `ToolDeregistered` event

## Discovery

```
ToolBus::discover(principal, filter) -> Result<Vec<ToolInfo>, ToolBusError>
```

- `principal` is the calling agent's identity (AgentId + JWT claims)
- Only returns tools where `principal` has `discover:tool:{namespace}` permission
- `filter` supports: by namespace, by capability tag, by name pattern
- Returns `ToolInfo` (name, namespace, description, schema) — no internal refs

## Invocation

```
ToolBus::invoke(principal, namespace, name, params, timeout) -> Result<Value, ToolBusError>
```

### Invocation Flow

1. **Permission check**: PolicyEngine evaluates `execute:tool:{namespace}` for `principal`
2. **Tool lookup**: Find ToolEntry by (namespace, name)
3. **Dispatch**:
   - Native tool → `ActorRef::ask(ToolInvocation { params })` with timeout
   - MCP tool → MCP client `call_tool(name, params)` with timeout
4. **Audit**: Log `ToolInvoked` event via AuditLogger (principal, tool, params hash, result status, latency)
5. **Metrics**: Record invocation count, latency histogram, error rate per tool
6. **Return**: Result value or error

### Error Cases

| Error | Trigger | Caller Sees |
|-------|---------|-------------|
| `ToolNotFound` | No tool with (namespace, name) | Error with tool identifier |
| `PermissionDenied` | PolicyEngine rejects | Error with required permission |
| `InvocationTimeout` | Timeout exceeded | Error with elapsed time |
| `InvocationFailed` | Tool returned error | Error with tool's error message |
| `ToolUnavailable` | Backing agent not Running | Error with agent state |

## MCP Bridge Contract

- MCP tools are registered by the `mister-smith-mcp` crate's tool discovery
- Each MCP server session registers its tools with the ToolBus on connection
- MCP tool deregistration happens on session disconnect
- MCP tool schemas are mapped from MCP `Tool` definitions to `ToolSchema`
- MCP invocation uses `mister-smith-mcp` client to call the remote tool

## Concurrency

- ToolBus is `Send + Sync` (uses `DashMap` for concurrent registry access)
- Multiple concurrent invocations to the same tool are allowed (tool agent handles its own concurrency)
- Tool registration/deregistration is lock-free via `DashMap`
