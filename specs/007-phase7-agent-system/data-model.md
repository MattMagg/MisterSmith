# Data Model: Phase 7 — Agent System

**Date**: 2026-03-05
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

## Entities

### AgentEntry

Registry entry for an active agent.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| agent_id | AgentId | PK, unique | Canonical UUID wrapper from mister-smith-core |
| agent_type | AgentType | Required, enum | Role enum (Supervisor, Worker, Coordinator, etc.) |
| state | AgentState | Required, enum | Current lifecycle state (Initializing, Running, Paused, Stopping, Terminated) |
| health | HealthLevel | Required, enum | Healthy, Degraded, Unhealthy, Critical |
| capabilities | Vec\<String\> | Optional | Capability tags for discovery queries |
| command_subject | String | Required, unique | NATS subject for sending commands to this agent |
| heartbeat_at | DateTime\<Utc\> | Updated on each heartbeat | Last heartbeat timestamp |
| started_at | DateTime\<Utc\> | Set on Running transition | When agent entered Running state |
| restart_count | u32 | Default 0 | Number of times this agent has been restarted |
| metadata | serde_json::Value | Optional | Arbitrary key-value metadata |
| supervisor_id | Option\<AgentId\> | FK → AgentEntry | Parent supervisor (None for root agents) |

**Uniqueness**: agent_id is globally unique. command_subject is unique per node.

**Lifecycle**: Created on agent spawn (Initializing), updated on state transitions, removed on Terminated after deregistration delay.

---

### Team

Ephemeral group of agents assembled by a Coordinator.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| team_id | Uuid | PK, unique | Team identifier |
| coordinator_id | AgentId | FK → AgentEntry | Coordinator that created this team |
| supervisor_id | AgentId | FK → AgentEntry | Team's shared supervisor |
| pattern | TeamPattern | Required, enum | SupervisorWorker, Pipeline, Consensus |
| task_id | TaskId | FK → TaskRecord | The task this team was assembled to execute |
| members | Vec\<AgentId\> | Non-empty | Agent IDs of team members |
| created_at | DateTime\<Utc\> | Set on creation | When the team was assembled |
| disbanded_at | Option\<DateTime\<Utc\>\> | Set on disband | When the team was disbanded |

**Lifecycle**: Created when Coordinator assembles a team, disbanded when the orchestrating task completes or is cancelled.

---

### TaskAssignment

A task submitted to the scheduling system.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| task_id | TaskId | PK, unique | Canonical UUID wrapper from mister-smith-core |
| task_type | String | Required | Task type for matching to capable agents |
| priority | u8 | Default 128, 0=highest | Processing priority |
| deadline | Option\<DateTime\<Utc\>\> | Optional | Absolute deadline for completion |
| input | serde_json::Value | Required | Task input payload |
| output | Option\<serde_json::Value\> | Set on completion | Task result payload |
| state | TaskState | Required, enum | Pending, Assigned, Running, Completed, Failed, TimedOut |
| assigned_to | Option\<AgentId\> | FK → AgentEntry | Agent executing this task |
| parent_task_id | Option\<TaskId\> | FK → TaskAssignment | Parent task (for decomposition) |
| team_id | Option\<Uuid\> | FK → Team | Team this task belongs to |
| message_id | Uuid | Unique | Deduplication key (maps to MessageEnvelope.message_id) |
| created_at | DateTime\<Utc\> | Set on creation | When the task was submitted |
| assigned_at | Option\<DateTime\<Utc\>\> | Set on assignment | When the task was assigned |
| completed_at | Option\<DateTime\<Utc\>\> | Set on completion | When the task finished |
| error_message | Option\<String\> | Set on failure | Error description if failed |

**State Machine**: Pending → Assigned → Running → (Completed | Failed | TimedOut)

**Decomposition**: A parent task can have multiple child tasks linked via parent_task_id. Result aggregation follows the dependency graph defined by parent-child relationships.

---

### ToolEntry

A tool registered in the ToolBus.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| name | String | Unique within namespace | Tool identifier |
| namespace | String | Required | Permission scope (e.g., "math", "search", "system") |
| description | String | Required | Human-readable description |
| input_schema | serde_json::Value | Required | JSON Schema for tool parameters |
| output_schema | serde_json::Value | Required | JSON Schema for tool result |
| agent_id | Option\<AgentId\> | FK → AgentEntry | Backing agent (None for MCP tools) |
| mcp_session | Option\<String\> | Set for MCP tools | MCP server session identifier |
| registered_at | DateTime\<Utc\> | Set on registration | When the tool was registered |
| timeout | Duration | Default 30s | Maximum invocation time |

**Uniqueness**: (namespace, name) is unique.

**Permission mapping**: Invocation requires `execute:tool:{namespace}`. Discovery requires `discover:tool:{namespace}`.

---

### AgentConfig

Configuration for a specific agent instance.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| agent_type | AgentType | Required | Which role this agent fulfills |
| restart_policy | RestartPolicy | Default OneForOne | Supervision restart strategy |
| heartbeat_interval | Duration | Default 5s | Time between heartbeat emissions |
| mailbox_capacity | usize | Default 1000 | Maximum messages in mailbox |
| priority_mailbox | bool | Default false | Enable priority-ordered processing |
| task_timeout | Duration | Default 60s | Default timeout for task execution |
| tool_permissions | Vec\<String\> | Default empty | Granted tool permission patterns |
| role_config | serde_json::Value | Optional | Role-specific configuration |

---

## Enums

### HealthLevel

```
Healthy     — All checks passing, heartbeat regular
Degraded    — Some checks failing or heartbeat irregular
Unhealthy   — Multiple checks failing
Critical    — Agent unresponsive or about to be terminated
```

### TeamPattern

```
SupervisorWorker — Fan-out task assignment, fan-in result aggregation
Pipeline         — Sequential handoff from one agent to the next
Consensus        — Parallel evaluation with voting/majority result
```

### TaskState

```
Pending    — Submitted, awaiting assignment
Assigned   — Assigned to an agent, awaiting execution start
Running    — Agent is actively executing
Completed  — Execution finished successfully
Failed     — Execution failed with error
TimedOut   — Deadline exceeded
Cancelled  — Cancelled by Coordinator or system
```

---

## Relationships

```
AgentEntry 1──* Team (as member)
AgentEntry 1──1 Team (as coordinator)
AgentEntry 1──1 Team (as supervisor)
AgentEntry 1──* TaskAssignment (as assignee)
AgentEntry 1──* ToolEntry (as backing agent)
AgentEntry *──1 AgentEntry (supervisor_id → parent)
Team 1──* TaskAssignment (team_id)
TaskAssignment *──1 TaskAssignment (parent_task_id → parent)
```

## State Transitions

### Agent Lifecycle

```
[spawn] → Initializing → Running ⇄ Paused
              ↓              ↓         ↓
           Error ←───── Stopping ← ───┘
              ↓              ↓
         [restart]     Terminated → [deregister]
```

### Task Lifecycle

```
[submit] → Pending → Assigned → Running → Completed
                         ↓          ↓
                      [timeout]   Failed
                         ↓
                      TimedOut
                         ↓
                    [retry/reassign/fail]
```
