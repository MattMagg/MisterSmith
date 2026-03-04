# MCP Integration: Cross-Framework Analysis & Refined Proposal

## 1. Cross-Framework Research Summary

Analysis of 7 agent frameworks' MCP implementations — every pattern, decision, and trade-off relevant to Mister Smith.

### Comparative Matrix

| | LangChain | CrewAI | OpenAI SDK | Google ADK | Mastra | AG2 | agentic-rag-sdk |
|---|---|---|---|---|---|---|---|
| **Role** | Both | Both | Both | Both | Both | Both | Server only |
| **Pattern** | Adapter/wrapper | Adapter + DSL | Server classes | MCPToolset bridge | Client/Server split | Session manager | FastMCP decorator |
| **Auto-discovery** | `loadMcpTools` | `get_tools` | `listTools()` | `McpToolset.list_tools()` | `mcp.listTools()` | `register_function` | N/A (static) |
| **Transport** | stdio, streamable-HTTP | streamable-HTTP | stdio, streamable-HTTP | stdio, SSE/HTTP | stdio, HTTP | stdio, HTTP, WebSocket | stdio |
| **Tool caching** | No (stateless) | No | Yes (`cacheToolsList`) | Implicit | Yes | Implicit | N/A |
| **Tool filtering** | No | No | Yes (`toolFilter`) | Yes (`tool_filter`) | No | No | N/A |
| **Multi-server** | Yes (`MultiServerMCPClient`) | Single adapter | Multiple instances | Multiple `McpToolset` | Yes (named configs) | Yes (dynamic selection) | Single server |
| **Session model** | Stateless per-call | Lazy on-demand | Context manager | Session manager | Persistent | On-demand + cleanup | Persistent |
| **Auth model** | Scoped tokens, TLS | API key headers | toolFilter + headers | Auth headers | Token refresh hooks | Session tokens | Env vars |
| **Namespace isolation** | No | No | No | No | Yes (per-server) | No | N/A |

### Key Patterns Extracted

**Pattern 1: Universal Bidirectional Role**
Every framework acts as both MCP client (consuming external servers) and MCP server (exposing its own tools). This is not optional — it's the baseline expectation.

**Pattern 2: Bridge/Adapter Architecture**
All frameworks use a bridge layer that translates between their native tool system and MCP's JSON-RPC format. Nobody modifies their core tool abstractions — they wrap them.

**Pattern 3: Auto-Discovery via `listTools()`**
All client-side integrations call `tools/list` at connection time and auto-register discovered tools into their native tool system. This is the fundamental onboarding mechanism.

**Pattern 4: Transport Flexibility (stdio + HTTP)**
stdio is used for local subprocess servers. Streamable HTTP (replacing deprecated SSE) is used for remote servers. Both must be supported.

**Pattern 5: Tool Caching + Invalidation (OpenAI, Mastra)**
OpenAI SDK caches tool lists per-server with `invalidateToolsCache()`. This avoids redundant `tools/list` calls and is important for latency. MCP's `notifications/tools/list_changed` triggers cache invalidation.

**Pattern 6: Tool Filtering (OpenAI, Google ADK)**
Both provide `toolFilter` / `tool_filter` to limit which tools from an MCP server are exposed to agents. This is critical for security (don't expose `write_file` to read-only agents) and UX (reduce tool overload).

**Pattern 7: Multi-Server with Namespace Isolation (Mastra)**
Mastra's `MCPClient` connects to multiple named servers, and tools from each server are namespaced to avoid collisions. Example: `filesystem.read_file` vs `github.read_file`.

**Pattern 8: On-Demand Sessions with Cleanup (AG2, CrewAI)**
AG2's `MCPClientSessionManager` creates sessions on-demand and cleans up via async context managers. CrewAI's DSL establishes connections only when a tool is actually invoked (lazy). Both reduce resource waste.

**Anti-Pattern: LangChain's Stateless Per-Call Sessions**
LangChain's `MultiServerMCPClient` creates a fresh session per tool invocation. While simple, this is inefficient — MCP session setup has overhead (capability negotiation, `initialize`). LangGraph warns about memory leaks with long-lived connections, but the solution should be *managed sessions*, not *disposable ones*.

---

## 2. Resolved Design Questions

### Q1: Should agent internal state be exposed as MCP Resources?

**Answer: Yes, selectively.**

Google ADK and Mastra both expose internal state as MCP resources. The pattern is: agents' **knowledge bases, context windows, and configuration** are natural MCP Resources. Agent **runtime state** (mailbox depth, current task) belongs in monitoring, not MCP.

**MS Design:**
```
Resource types to expose:
├── Agent knowledge bases → resources/read (read-only access to agent memory)
├── Configuration snapshots → resources/read (current config as JSON)
├── Tool schemas → resources/read (complete tool catalog)
└── NOT: runtime state, mailbox contents, supervision status
```

The `ResourceRegistry` in the proposed `mister-smith-mcp` crate will register these as MCP resources with `resources/list` and `resources/read` handlers.

### Q2: How should tool permissions map to MCP auth?

**Answer: Two-layer model, borrowed from CrewAI enterprise + Mastra + MS permissions.**

Every framework handles this differently because MCP itself has no permission model. The best pattern is a **two-layer approach**:

| Layer | Responsibility | Mechanism |
|---|---|---|
| **Transport auth** (MCP layer) | Who can *connect* | OAuth2 / API key / mTLS on transport |
| **Tool-level auth** (MS layer) | Who can *call what* | `PermissionSystem` checks inside `McpHandler` |

**MS Design:**

```rust
// Conceptual flow for tool call authorization
impl McpHandler {
    async fn handle_call_tool_request(&self, params: CallToolRequestParams) -> Result<...> {
        // Layer 1: Transport auth already validated by middleware (mTLS/OAuth)
        let caller_identity = self.extract_identity(&params);
        
        // Layer 2: MS PermissionSystem check 
        let tool_id = self.resolve_tool_id(&params.name)?;
        self.permission_system.check(caller_identity, tool_id, Action::Execute)?;
        
        // Authorized — execute via ToolBus
        self.tool_bus.call(caller_identity, tool_id, params.arguments).await
    }
}
```

Additionally, adopt OpenAI's `toolFilter` pattern: when exposing tools outward, let MCP server config specify which tools are visible to external clients, separate from which ones the caller has *permission* to invoke.

```toml
[[mcp.servers]]
name = "public-api"
expose_tools = ["search", "status", "health"]  # Whitelist
# Even if ToolBus has 50 tools, only these 3 appear in tools/list
```

### Q3: Multi-server topology — per-team or global?

**Answer: Global server with config-driven filtering, borrowing from Mastra's namespace isolation.**

Running separate MCP servers per agent team adds operational overhead. Instead: one MCP server with **namespace-scoped tool views**, applying Mastra's isolation pattern.

**MS Design:**

```toml
[[mcp.servers]]
name = "agent-platform"
transport = "streamable-http"
bind = "0.0.0.0:3001"

# Namespace-scoped views — different clients see different tool subsets
[[mcp.servers.views]]
namespace = "analytics"
include_agents = ["data-analyst", "reporter"]
expose_resources = true

[[mcp.servers.views]]
namespace = "devops"
include_agents = ["deployer", "monitor"]
expose_resources = false
```

When an MCP client connects with namespace `analytics`, `tools/list` returns only tools from agents `data-analyst` and `reporter`. This gives the isolation of per-team servers with the simplicity of a single server.

### Q4: Should MCP messages bridge over NATS?

**Answer: Yes — this is a novel differentiator unique to MS.**

No other framework does this. NATS is already MS's transport backbone. Bridging MCP over NATS enables:

- **Distributed MCP**: An MCP server on node A can execute tools registered on node B via NATS routing
- **MCP event bus**: `notifications/tools/list_changed` propagated to all nodes via NATS pub/sub
- **Federated tool discovery**: `tools/list` aggregates tools from all NATS-connected nodes

**MS Design:**

```
MCP Client → HTTP → McpServer (Node A)
                        ↓ tools/call("agent-b.analyze")
                    NATS Message Bus
                        ↓ routed to Node B
                    ToolBus (Node B)
                        ↓ execute
                    Agent B → result
                        ↑ NATS return
                    McpServer (Node A) → HTTP response → MCP Client
```

This is implemented by making `McpHandler` resolve tool ownership: local tools call `ToolBus.call()` directly; remote tools publish a NATS request to the owning node.

### Q5: Should client-side (consuming external MCPs) come before server-side?

**Answer: Yes — client-first, borrowing from AG2's on-demand session model.**

Practical rationale:
1. Consuming external MCP servers (filesystem, database, API) provides **immediate value** — agents gain access to the real world
2. Serving MS tools via MCP requires agents and tools to exist first — that's further down the ROADMAP
3. Client-side is lower risk — no public-facing surface to secure

**Phase order adjustment:**
- **Phase A**: MCP Client (connect to external servers, import tools into ToolBus)
- **Phase B**: MCP Server (expose MS tools to external MCP clients)
- **Phase C**: Full protocol (resources, prompts, notifications)
- **Phase D**: Advanced (NATS bridge, hot-reload, sampling)

---

## 3. Refined Architecture (Incorporating Cross-Framework Best Practices)

### 3.1 Core Components (Updated)

```
mister-smith-mcp/src/
├── lib.rs              # Public API
├── client/
│   ├── mod.rs          # McpClient — connects to external MCP servers
│   ├── session.rs      # McpSessionManager (AG2-inspired on-demand sessions)
│   ├── external_tool.rs # ExternalMcpTool (wraps remote tool as MS Tool)
│   └── discovery.rs    # Auto-discovery + cache + invalidation
├── server/
│   ├── mod.rs          # McpServerActor — supervised MCP server
│   ├── handler.rs      # McpHandler (ServerHandler impl)
│   ├── views.rs        # Namespace-scoped tool views (Mastra-inspired)
│   └── filter.rs       # Tool filtering (OpenAI-inspired toolFilter)
├── bridge.rs           # MS Tool ↔ MCP Tool schema translation
├── registry.rs         # McpRegistry — manages all servers and clients
├── config.rs           # TOML-driven McpConfig
├── permissions.rs      # McpPermissionBridge (two-layer auth)
├── resources.rs        # MCP Resource adapter
├── prompts.rs          # MCP Prompt adapter
├── nats_bridge.rs      # NATS ↔ MCP bridge (distributed tools)
└── transport.rs        # Transport abstraction (stdio/HTTP/SSE)
```

### 3.2 Adopted Patterns by Source

| Pattern | Source Framework | MS Component |
|---|---|---|
| Auto-discovery + cache | OpenAI SDK | `discovery.rs` — cache `tools/list`, invalidate on `list_changed` |
| Tool filtering | OpenAI SDK, Google ADK | `filter.rs` — whitelist/regex tool filter on server output |
| Namespace isolation | Mastra | `views.rs` — per-namespace tool views on single server |
| On-demand sessions | AG2 | `session.rs` — lazy connect, async cleanup, reconnect on failure |
| Two-layer auth | CrewAI enterprise + MS | `permissions.rs` — transport auth + ToolBus permission checks |
| Bridge/adapter wrapper | All frameworks | `bridge.rs` — `ToolSchema` ↔ MCP `inputSchema` translation |
| Supervised lifecycle | Unique to MS | `McpServerActor` — actor in supervision tree with restart policies |
| NATS distributed tools | Unique to MS | `nats_bridge.rs` — federate tool calls over NATS |
| Lazy connection | CrewAI DSL | Clients connect on first tool call, not at registration |

### 3.3 Configuration (Final)

```toml
[mcp]
enabled = true

# --- CLIENT SIDE: Connect to external MCP servers ---
[[mcp.clients]]
name = "filesystem"
transport = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/data"]
auto_register_tools = true
lazy_connect = true              # Connect on first tool call (CrewAI pattern)
cache_tools_list = true          # Cache tools/list response (OpenAI pattern)
tool_filter = ["read_*"]         # Only import read tools (ADK pattern)
allowed_agents = ["researcher"]  # MS permission scoping
timeout_ms = 30000

[[mcp.clients]]
name = "postgres"
transport = "streamable-http"
url = "http://localhost:3002/mcp"
auto_register_tools = true
cache_tools_list = true
auth.type = "bearer"
auth.token_env = "POSTGRES_MCP_TOKEN"

# --- SERVER SIDE: Expose MS tools to external MCP clients ---
[[mcp.servers]]
name = "agent-platform"
transport = "streamable-http"
bind = "0.0.0.0:3001"
auth.type = "mtls"               # Use MS security infrastructure
auth.ca_cert = "/etc/ms/ca.pem"
expose_resources = true
expose_prompts = false

[[mcp.servers.views]]
namespace = "analytics"
include_agents = ["data-analyst", "reporter"]
tool_filter = ["analyze_*", "report_*"]

[[mcp.servers.views]]
namespace = "ops"
include_agents = ["deployer"]

# --- DISTRIBUTED (Phase D) ---
[mcp.nats_bridge]
enabled = false                  # Enable when multi-node
subject_prefix = "ms.mcp"       # NATS subject prefix for MCP routing
```

### 3.4 Implementation Phases (Revised Priority)

#### Phase A: MCP Client (highest immediate value)
1. Add `rust-mcp-sdk` dependency (client + stdio features)
2. Implement `McpSessionManager` — on-demand sessions with reconnect
3. Implement `ExternalMcpTool` — wraps remote MCP tool as MS `Tool`
4. Implement auto-discovery — `tools/list` → register in `ToolBus`
5. Implement tool caching with `list_changed` invalidation
6. Add `[mcp.clients]` config section
7. `tool_filter` + `allowed_agents` permission gating

#### Phase B: MCP Server
1. Add `rust-mcp-sdk` (server + streamable-http features)
2. Implement `McpHandler` — bridge `ServerHandler` → `ToolBus`
3. Implement `McpServerActor` — supervised actor with restart policies
4. Implement tool filtering + namespace views
5. Add `[mcp.servers]` config section
6. Two-layer auth (`permissions.rs`)

#### Phase C: Full Protocol
1. Resources adapter (`resources/list`, `resources/read`)
2. Prompts adapter (`prompts/list`, `prompts/get`)
3. `notifications/tools/list_changed` via EventBus
4. SSE transport (legacy compatibility)

#### Phase D: MS Differentiators
1. NATS MCP bridge (federated tool discovery + remote execution)
2. Hot-reload via `ConfigurationManager` watcher
3. Sampling support (server → client LLM calls)
4. Runtime management API on `SystemCore`

---

## 4. What Makes This Better Than Any Single Framework

| Advantage | Why |
|---|---|
| **Supervised MCP actors** | No other framework runs MCP connections under a supervision tree. MS gets automatic restarts, health checks, and graceful shutdown for free |
| **NATS-federated tools** | No other framework can discover and call tools across distributed nodes. This is a unique MS capability |
| **Namespace-scoped views** | Mastra has namespace isolation but not as a configurable server view. MS makes this declarative in TOML |
| **Two-layer auth** | Most frameworks do transport-level auth OR tool-level auth. MS does both, with the inner layer using its existing `PermissionSystem` |
| **On-demand + cached** | Combines AG2's lazy sessions with OpenAI's tool caching — connect late, cache aggressively, invalidate on `list_changed` |
| **Actor-based tool wrapping** | `AgentTool` already wraps actors as tools. MCP integration reuses this — an MCP tool call goes Actor → Tool → ToolBus → MCP, with no new abstractions |
