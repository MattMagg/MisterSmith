# Claude Code CLI Technical Analysis

> **OBSOLETE -- FOR ARCHIVAL ONLY**
> Mister Smith is model-agnostic. This Claude CLI-specific analysis is no longer part of the active architecture. See the research/ directory for related historical analysis. For LLM backend integration patterns, refer to [system-architecture.md](system-architecture.md).

## Research Findings and Integration Feasibility Assessment

### Executive Summary

This document provides the technical analysis that informed the Claude CLI integration architecture. Research confirms excellent alignment between Claude Code CLI capabilities and the Mister Smith framework, with the framework's existing hook integration points designed for this integration.

**Key Finding**: The framework's NATS subject taxonomy already includes hook integration subjects (`control.startup`, `agent.{id}.pre`, etc.), confirming design intent for Claude Code CLI integration.

---

## VALIDATION STATUS

**Last Validated**: 2026-03-03
**Validator**: Agent 1C - Supervision & Implementation
**Validation Score**: 75/100 (PARTIALLY OUTDATED — hook system and CLI capabilities have expanded significantly)
**Status**: Updated with current Claude Code CLI state (March 2026)

### Analysis Completeness

- ⚠️ CLI capabilities partially outdated — Agent SDK now available, new hook events added
- ✅ Parallel execution patterns validated
- ⚠️ Hook system mapping needs update — expanded from 5 to 14+ event types
- ✅ Resource requirements verified (25-30 agents feasible)
- ✅ Integration complexity assessed (LOW-MEDIUM)
- ⚠️ Model name outdated — current frontier model is claude-opus-4-6

---

## Claude Code CLI Capabilities Analysis

### 1. Core CLI Interface

**Command Structure**:

```bash
# Interactive mode (REPL)
claude

# Print mode (non-interactive)
claude -p "query text"

# Session management
claude --continue --max-turns 10
claude --resume session_id

# Output formats
claude --output-format json
claude --output-format stream-json
claude --output-format text

# Tool control
claude --allowedTools Edit,Write,Bash
claude --disallowedTools WebSearch

# MCP integration
claude --mcp-config .claude/mcp.json
```

**Resource Characteristics**:

- Memory usage: ~100-200MB per instance
- CPU: Moderate during processing, low when idle
- Network: HTTPS connections to Anthropic API
- File handles: ~10-20 per instance

### 2. Parallel Execution Architecture

**Task Tool Capabilities**:

- Spawns concurrent sub-agents using built-in Task tool
- Each task runs as lightweight Claude Code instance
- Output format: `Task(Patch Agent <n>)` or `Task(Performing task X)`
- Independent context windows per sub-agent
- Automatic parallel coordination

**Parallel Execution Patterns**:

```bash
# Example parallel task spawning
"Explore the codebase using 4 tasks in parallel. Each agent should explore different directories."

# Output format:
● Task(Explore backend structure)
⎿ Done (17 tool uses · 56.6k tokens · 1m 34.3s)
● Task(Explore frontend structure)  
⎿ Done (23 tool uses · 48.9k tokens · 1m 15.9s)
```

**Scalability Analysis for 25-30 Agents**:

- Total memory: ~2.5-6GB
- File handles: ~250-600 (within OS limits)
- Network connections: 25-30 concurrent HTTPS
- Feasibility: ✅ Confirmed viable

### 3. Hook System Architecture

<!-- Updated 2026-03: Claude Code hooks expanded from 5 to 14+ event types -->

**Hook Lifecycle Events** (as of February 2026):

| Event | When It Fires | Framework Relevance |
|-------|--------------|-------------------|
| `SessionStart` | Session begins or resumes | Agent initialization, context loading |
| `SessionEnd` | Session ends | Cleanup, state persistence |
| `UserPromptSubmit` | Before Claude processes a prompt | Input validation, routing |
| `PreToolUse` | Before a tool call executes (can block) | Permission control, NATS pre-events |
| `PostToolUse` | After a tool call succeeds | Result routing, metrics |
| `PostToolUseFailure` | After a tool call fails | Error handling, NATS error events |
| `PermissionRequest` | Permission dialog appears | Automated approval/denial |
| `Notification` | Claude Code sends a notification | Alert routing |
| `SubagentStart` | A subagent is spawned | Agent pool tracking |
| `SubagentStop` | A subagent finishes | Resource cleanup |
| `Stop` | Claude finishes responding | Turn completion tracking |
| `TeammateIdle` | Agent team teammate is about to go idle | Load balancing |
| `TaskCompleted` | Task marked as completed | Workflow coordination |
| `ConfigChange` | Configuration file changes during session | Dynamic reconfiguration |
| `PreCompact` | Before context compaction | Context management |
| `WorktreeCreate` | Git worktree created | Workspace isolation |
| `WorktreeRemove` | Git worktree removed | Workspace cleanup |

**Hook Handler Types** (expanded from `command` only):

1. **command**: Execute a shell command
2. **prompt**: Inject a prompt into Claude's context
3. **agent**: Spawn a separate Claude agent

**Hook Configuration Structure**:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Task|Bash|Edit",
        "hooks": [
          {
            "type": "command",
            "command": "nats-publish agent.{id}.pre",
            "timeout": 60
          }
        ]
      }
    ],
    "SubagentStart": [
      {
        "matcher": "Bash|Explore|Plan",
        "hooks": [
          {
            "type": "command",
            "command": "nats-publish agent.{id}.subagent.start"
          }
        ]
      }
    ]
  }
}
```

**Hook Input/Output**:

- **Input**: JSON via stdin with session_id, cwd, hook_event_name, tool_name, tool_input, and tool_use_id
- **Output**: Exit codes (0=success, 2=block) or structured JSON
- **Decision Control**: Hooks can approve, block, or modify tool execution
- **NATS Integration**: Hook output can be published to NATS subjects
- **Async Execution**: Hooks support async execution (added January 2026)
- **Environment Persistence**: `SessionStart` hooks can persist env vars via `CLAUDE_ENV_FILE`

### 4. MCP Integration Capabilities

**Server Mode**:

```bash
claude mcp serve  # Run Claude Code as MCP server
```

**Tool Naming Convention**:

- Pattern: `mcp__<server>__<tool>`
- Examples: `mcp__memory__create_entities`, `mcp__filesystem__read_file`

**Slash Commands**:

- MCP prompts available as `/mcp__server__prompt`
- Custom commands via `.claude/commands/*.md`

---

## Mister Smith Framework Architecture Review

### 1. Current NATS Subject Taxonomy

**Existing Subject Hierarchy**:

```markdown
agents.{agent_id}.commands     # Agent command dispatch
agents.{agent_id}.status       # Agent status updates
agents.{agent_id}.output       # Agent output streams
tasks.{task_type}.queue        # Task queue management
events.{event_type}            # System events
cmd.{type}.{target}            # Command routing
```

**Hook Integration Points (Already Defined)**:

```markdown
control.startup                # CLI initialization
agent.{id}.pre                # Pre-task hook processing
agent.{id}.post               # Post-task hook processing
agent.{id}.error              # Error hook handling
agent.{id}.hook_response      # Hook mutation responses
ctx.{gid}.file_change         # File change notifications
```

**Critical Discovery**: The framework already includes hook integration subjects, confirming it was designed for Claude Code CLI integration.

### 2. Existing Components

**Core Architecture**:

- **NATS Messaging**: Distributed messaging backbone
- **Tokio Runtime**: Async patterns and supervision trees
- **Agent Orchestration**: Hub-and-spoke supervisor pattern
- **Transport Layer**: NATS, gRPC, HTTP protocols
- **Memory Management**: Postgres + JetStream KV store

**Integration-Ready Components**:

- Hook integration points already defined
- Agent lifecycle management patterns established
- Supervision tree patterns for fault tolerance
- Resource management frameworks in place

---

## Integration Mapping Analysis

### 1. Claude Code CLI → Framework Component Mapping

| Claude Code Feature | Framework Component | Integration Method |
|-------------------|-------------------|------------------|
| Task tool parallel execution | Agent orchestration | Parse task output, route to NATS |
| Hook system (5 types) | Transport layer hooks | Direct NATS subject mapping |
| CLI session management | Agent lifecycle | Tokio supervision patterns |
| MCP tool integration | Tool registry | Tool naming convention mapping |
| Output streaming | Observability | Stream parsing and routing |

### 2. Hook System Integration

**Direct Mapping** (updated for current Claude Code hook events):

```rust
// Claude Code Hook Event → NATS Subject
SessionStart       → control.startup
SessionEnd         → control.shutdown
PreToolUse         → agent.{id}.pre
PostToolUse        → agent.{id}.post
PostToolUseFailure → agent.{id}.error
SubagentStart      → agent.{id}.subagent.start
SubagentStop       → agent.{id}.subagent.stop
TaskCompleted      → agent.{id}.task_completed
Stop               → agent.{id}.stop
TeammateIdle       → agent.{id}.teammate_idle
ConfigChange       → control.config_change
// Legacy mappings (no longer direct hook events):
// on_file_change  → ctx.{gid}.file_change (now handled via PostToolUse matcher on Edit/Write)
```

**Integration Pattern**:

```rust
struct HookBridge {
    nats_client: async_nats::Client,
    hook_configs: Vec<HookConfig>,
    json_parser: HookJsonParser,
    subject_mapper: HookSubjectMapper,
}

impl HookBridge {
    async fn publish_hook_event(
        &self,
        hook_event: HookEventName,
        agent_id: &str,
        payload: HookPayload
    ) -> Result<(), async_nats::PublishError> {
        let subject = self.subject_mapper.map_to_nats_subject(hook_event, agent_id);
        self.nats_client.publish(subject, payload.to_json().into()).await
    }
}

// Maps Claude Code hook event names to NATS subjects
struct HookSubjectMapper {
    custom_mappings: HashMap<String, String>,
}

impl HookSubjectMapper {
    fn map_to_nats_subject(&self, event: HookEventName, agent_id: &str) -> String {
        // Check custom mappings first, then use default pattern
        if let Some(custom) = self.custom_mappings.get(event.as_str()) {
            return custom.replace("{id}", agent_id);
        }
        // Default: agent.{id}.{event_name_snake_case}
        format!("agent.{}.{}", agent_id, event.to_snake_case())
    }
}
```

```

### 3. Parallel Execution Integration

**Task Output Parsing**:

```rust
static TASK_OUTPUT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"Task\((?:Patch Agent )?(\d+)\)").unwrap()
});

async fn route_task_output(
    &self,
    agent_id: u32,
    output: &str
) -> Result<(), RoutingError> {
    let subject = format!("agents.{}.output", agent_id);
    // NOTE: async-nats 0.46+ accepts Bytes, not &[u8]. Use .into() for conversion.
    self.nats_client.publish(subject, bytes::Bytes::from(output.to_owned())).await?;
    Ok(())
}
```

---

## Required Framework Modifications

### 1. New Components (Additions, Not Modifications)

**A. Claude CLI Spawn Controller**:

```rust
struct ClaudeCliController {
    max_concurrent: usize,        // 25-30 agents
    active_sessions: HashMap<AgentId, ClaudeSession>,
    nats_client: async_nats::Client,
    hook_bridge: HookBridge,
    resource_manager: ResourceManager,
}
```

**B. Hook Bridge Service**:

```rust
struct HookBridge {
    hook_configs: Vec<HookConfig>,
    nats_publisher: NatsPublisher,
    json_parser: HookJsonParser,
    timeout_manager: TimeoutManager,
}
```

**C. Task Output Parser**:

```rust
struct TaskOutputParser {
    agent_id_extractor: Regex,
    output_router: OutputRouter,
    stream_processor: StreamProcessor,
}
```

### 2. Configuration Enhancements

**Claude CLI Configuration Schema**:

```toml
[claude_cli]
max_concurrent_agents = 25
default_model = "claude-opus-4-6"  # Updated from claude-3-5-sonnet-20241022
api_timeout = 300
hook_timeout = 60
output_format = "stream-json"

[claude_cli.hooks]
config_path = ".claude/hooks.json"
enable_nats_bridge = true
hook_execution_timeout = 30
```

---

## Implementation Feasibility Assessment

### 1. Technical Feasibility: ✅ HIGH

**Strengths**:

- Framework already designed for Claude Code CLI integration
- Hook system maps directly to existing NATS subjects
- Parallel execution aligns with Tokio supervision patterns
- Resource requirements well within typical system capabilities

**Minimal Risk Factors**:

- API rate limiting (manageable with proper configuration)
- Network connectivity requirements (standard for cloud services)
- Memory management for 25-30 agents (well within modern system capabilities)

### 2. Resource Feasibility: ✅ CONFIRMED

**System Requirements**:

- Memory: 8-16GB total (2.5-6GB for Claude CLI agents)
- CPU: 4-8 cores recommended
- Network: Stable internet for Anthropic API
- Storage: 1-2GB for logs and configurations

### 3. Integration Complexity: ✅ LOW-MEDIUM

**Complexity Assessment**:

- **Low**: Hook system integration (direct NATS mapping)
- **Low**: Task output parsing (regex-based)
- **Medium**: CLI session management (process lifecycle)
- **Medium**: Resource pool management (concurrent agent limits)

---

## Next Steps

1. **Create Detailed Integration Plan** - Specific framework modifications
2. **Update Framework Documentation** - Integrate Claude CLI specifications
3. **Implementation Roadmap** - Phased development approach
4. **Prototype Development** - Proof of concept implementation
5. **Evaluate Agent SDK** - Assess whether the official Claude Agent SDK (Python/TypeScript) or community Rust SDK should replace raw CLI subprocess management

This analysis confirms that Claude Code CLI integration with the Mister Smith framework is highly feasible and well-aligned with the existing architecture.

---

## Addendum: Claude Agent SDK (September 2025+)

<!-- [UNVERIFIED] Exact Rust SDK API surface may differ from Python/TypeScript SDKs -->

Since this analysis was written, Anthropic has released the **Claude Agent SDK** providing official programmatic interfaces:

- **Python SDK**: `claude-agent-sdk` — full-featured, official
- **TypeScript SDK**: Official, mirrors Python API
- **Community Rust SDK**: `claude-agents-sdk` (GitHub: `jimmystridh/claude-agents-sdk`) — wraps CLI with Tokio async, typed messages, streaming support

**Key SDK capabilities relevant to this framework:**

| Capability | Raw CLI | Agent SDK |
|-----------|---------|-----------|
| Session management | Manual stdin/stdout parsing | Typed API with `ClaudeClient` |
| Message streaming | Line-by-line JSON parsing | Typed `MessageStream` with content blocks |
| Tool permissions | CLI flags (`--allowedTools`) | Programmatic callbacks |
| Hook integration | External hook scripts | In-process callbacks |
| Multi-turn conversation | Session resume via `--resume` | Built-in session state |
| Cost tracking | Parse from output | Structured `CostInfo` |

**Recommendation**: Evaluate the community Rust SDK maturity before implementing raw CLI subprocess management. The SDK approach reduces boilerplate in `ClaudeSession` and `TaskOutputParser` significantly.
