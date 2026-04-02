# Remote And Delegated Execution Transfer Ideas

## 1. Turn Loop With Mid-Turn Notification Drain

**OpenClaude feature**

One long-lived query loop keeps turn state alive and drains queued task or worker notifications
into the next model turn instead of treating each turn as a simple request wrapper.

**OpenClaude evidence**

- `src/query.ts`

**Why it matters**

This is a strong foundation for a future real coordinator or subagent runtime. It gives the system
one place to merge tool results, worker completion, and retry logic.

**Mister Smith fit**

`Conditional fit next`

Useful for a later stronger coordinator runtime, but it should extend packet-023 run-trace truth
instead of bypassing it.

**How to translate into Mister Smith**

- add a bounded runtime inbox for worker or subordinate events
- feed those events into:
  - task inspect
  - autonomy status
  - run-trace summaries
- keep proof-boundary wording conservative until grounded outputs exist

## 2. Stable Child-Agent Identity With Continue-In-Place Messaging

**OpenClaude feature**

Spawned workers keep a stable identity and can receive later follow-up messages instead of being
respawned from scratch.

**OpenClaude evidence**

- `src/tools/SendMessageTool/SendMessageTool.ts`
- `src/tasks/LocalAgentTask/LocalAgentTask.tsx`
- `src/tasks/InProcessTeammateTask/InProcessTeammateTask.tsx`

**Why it matters**

This avoids repeated rediscovery work, lowers cost, and makes repair loops cleaner.

**Mister Smith fit**

`Conditional fit next`

This is one of the best later-stage ideas if Smith grows a first-class child-agent runtime.

**How to translate into Mister Smith**

- give each child agent a stable identifier tied to `workflow_id` and, where relevant, `session_id`
- support:
  - inbox append
  - resume
  - stop
  - inspect
- project child continuity into operator-visible runtime truth

## 3. Subagent Context Isolation With Shared Root Channels

**OpenClaude feature**

Per-agent mutable state is cloned by default, while only a small set of root channels stay shared
for lifecycle, task registration, and cancellation.

**OpenClaude evidence**

- `src/utils/forkedAgent.ts`
- `src/tools/AgentTool/runAgent.ts`

**Why it matters**

Nested agents are safer when scratch state is private but lifecycle ownership stays centralized.

**Mister Smith fit**

`Conditional fit next`

This fits packet-024 style least-privilege thinking and is a good shape for later coordinator work.

**How to translate into Mister Smith**

- keep child-agent scratch state private
- share only root-owned channels such as:
  - task registration
  - cancellation
  - runtime-truth projection
  - capability enforcement

## 4. Unified Subordinate Task Runtime

**OpenClaude feature**

Local shell work, local agents, teammates, remote agents, and other long-lived work all use one
typed task model with stable IDs, status, output files, and notification envelopes.

**OpenClaude evidence**

- `src/Task.ts`
- `src/tasks.ts`
- `src/tasks/LocalAgentTask/LocalAgentTask.tsx`

**Why it matters**

This gives operators one mental model for subordinate work and makes resume and inspection much
easier.

**Mister Smith fit**

`Conditional fit next`

Smith already has task, session, and autonomy surfaces. The useful next step is a subordinate
execution-unit layer under one workflow, not a parallel unrelated task system.

**How to translate into Mister Smith**

- add one bounded subordinate execution-unit view for:
  - planner branches
  - verifier passes
  - delegated tool bundles
  - later child agents
- project child status into packet-023 runtime truth

## 5. Secret-Minimized Remote Bridge

**OpenClaude feature**

Remote child sessions get a tight environment allowlist, explicit control traffic, permission
bridges, and remote-session message adaptation instead of inheriting the whole local process
environment.

**OpenClaude evidence**

- `src/bridge/sessionRunner.ts`
- `src/remote/RemoteSessionManager.ts`
- `src/remote/remotePermissionBridge.ts`
- `src/bridge/remoteBridgeCore.ts`
- `src/remote/sdkMessageAdapter.ts`

**Why it matters**

If Smith grows remote workers or hosted child executors, secret-minimized bridging is a strong
default pattern.

**Mister Smith fit**

`Later or do-not-copy`

This is very useful reference material, but it belongs to a later remote-executor lane, not the
current default local runtime path.

**How to translate into Mister Smith**

- keep worker environments on an allowlist
- move permission and interrupt traffic over an explicit control protocol
- avoid no-intercept proxy mistakes
- persist enough sidecar metadata to resume safely

## 6. Specialist Child Roles With Enforced Tool Subsets

**OpenClaude feature**

Roles like explore, plan, and verification are not just prompts. They also get enforced tool
subsets.

**OpenClaude evidence**

- `src/tools/AgentTool/built-in/exploreAgent.ts`
- `src/tools/AgentTool/built-in/planAgent.ts`
- `src/tools/AgentTool/built-in/verificationAgent.ts`
- `src/tools/AgentTool/agentToolUtils.ts`

**Why it matters**

Bounded child roles are easier to supervise, safer to authorize, and easier to explain to
operators.

**Mister Smith fit**

`Conditional fit next`

This fits later coordinator work well and lines up with packet-024 permission discipline.

**How to translate into Mister Smith**

- define a small child-role taxonomy:
  - explorer
  - planner
  - verifier
  - maybe repairer
- bind each role to hard tool subsets instead of prompt-only expectations

## 7. Cross-Worker Permission Mailbox

**OpenClaude feature**

Workers raise structured approval requests to a leader instead of receiving broader default
permissions.

**OpenClaude evidence**

- `src/utils/swarm/permissionSync.ts`

**Why it matters**

This is a practical approval pattern for delegated actions that preserves least privilege.

**Mister Smith fit**

`Conditional fit next`

This lines up well with packet `024` and could become a future operator or parent-agent approval
channel.

**How to translate into Mister Smith**

- keep baseline capability grants narrow
- add an approval mailbox on top of:
  - delegation chains
  - auth-callout style checks
  - ToolBus boundary enforcement

## 8. Surface-Specific Command Gating

**OpenClaude feature**

Remote or reduced-authority surfaces only get a safe subset of commands, and some background
command results re-enter the main loop as hidden prompts instead of direct execution.

**OpenClaude evidence**

- `src/commands.ts`
- `src/utils/processUserInput/processUserInput.ts`
- `src/utils/processUserInput/processSlashCommand.tsx`

**Why it matters**

This is a strong model for keeping remote, HTTP, CLI, and console surfaces honest about what they
are allowed to do.

**Mister Smith fit**

`High fit now`

Even before a larger coordinator runtime exists, Smith can benefit from surface-specific action
allowlists across CLI, HTTP, and operator-console initiated actions.

**How to translate into Mister Smith**

- define surface-specific action policy for:
  - CLI
  - HTTP operator endpoints
  - operator console
  - future delegated or remote clients
- reject out-of-scope actions explicitly and surface the reason
