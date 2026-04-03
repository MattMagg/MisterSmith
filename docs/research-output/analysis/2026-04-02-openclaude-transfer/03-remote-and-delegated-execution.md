# Remote And Delegated Execution Transfer Ideas

## 1. Turn Loop With Scoped Mid-Turn Notification Drain

**Verdict**

`KEEP with update`

**Source files**

- `src/query.ts`
- `src/tasks/LocalAgentTask/LocalAgentTask.tsx`

**What it is**

OpenClaude keeps a long-lived turn loop alive, drains queued subordinate notifications into the
same turn, and scopes those drains by agent identity so the coordinator and children only consume
their own event streams.

**Why it is useful for Mister Smith**

This is one of the best packet `026` inputs. It gives Smith a clean model for subordinate-runtime
event intake without pretending each child run is a separate unrelated conversation.

**Concrete adaptation path**

- add a bounded subordinate-runtime inbox under the parent workflow
- scope event intake by parent and child runtime identity
- project the same events to:
  - task inspect
  - autonomy status
  - operator run detail

**Risk and compatibility caveats**

- keep prompt and command streams separate from subordinate notifications
- do not let mid-turn intake bypass packet-023 truth and proof-boundary wording

## 2. Stable Child Identity With Continue-In-Place Messaging

**Verdict**

`KEEP with update`

**Source files**

- `src/tasks/LocalAgentTask/LocalAgentTask.tsx`
- `src/tools/SendMessageTool/SendMessageTool.ts`
- `src/tasks/InProcessTeammateTask/InProcessTeammateTask.tsx`

**What it is**

OpenClaude gives child work units stable IDs and lets later turns append follow-up messages instead
of recreating the worker from scratch.

**Why it is useful for Mister Smith**

This is a real coordination-runtime advantage. It reduces rediscovery cost and makes recovery loops
inspectable. It belongs in packet `026` more than most of the generic UI findings.

**Concrete adaptation path**

- add stable delegated-work identity tied to `workflow_id`
- allow bounded follow-up actions:
  - append clarification
  - resume
  - stop
  - inspect
- keep all of that visible on runtime-truth surfaces

**Risk and compatibility caveats**

- do not confuse child identity with unlimited transcript carry-forward
- child continuity must stay bounded by explicit runtime records and evidence refs

## 3. Subagent Context Isolation With Shared Root Channels

**Verdict**

`KEEP as-is`

**Source files**

- `src/utils/forkedAgent.ts`
- `src/tools/AgentTool/runAgent.ts`

**What it is**

OpenClaude clones mutable agent state by default and shares only a small set of root-owned channels
for lifecycle, cancellation, and limited coordination.

**Why it is useful for Mister Smith**

This fits packet-024 least-privilege posture cleanly. It is exactly the sort of coordination
primitive Smith should adapt instead of copying broad framework defaults.

**Concrete adaptation path**

- make delegated runtime state private by default
- share only root-owned channels for:
  - registration
  - cancellation
  - operator truth projection
  - capability enforcement

**Risk and compatibility caveats**

- do not share mutable scratch context casually
- merged outputs should be explicit coordinator decisions, not side effects from shared memory

## 4. Unified Subordinate Execution-Unit Model

**Verdict**

`KEEP with update`

**Source files**

- `src/Task.ts`
- `src/tasks.ts`
- `src/tasks/LocalAgentTask/LocalAgentTask.tsx`

**What it is**

OpenClaude models shell jobs, local agents, remote agents, teammates, and other long-lived work as
typed tasks with stable IDs, status, output files, and notification envelopes.

**Why it is useful for Mister Smith**

Smith already has strong workflow and autonomy surfaces. The useful transfer is not "copy their task
system." It is to give packet `026` a bounded subordinate execution-unit layer under the existing
workflow identity.

**Concrete adaptation path**

- define one child execution-unit record under the parent workflow
- keep typed status, evidence refs, and output pointers
- reuse packet-022 lifecycle and packet-023 truth vocabulary where possible

**Risk and compatibility caveats**

- do not fork a second top-level lifecycle model beside current workflow truth
- preserve clear parent-child ownership

## 5. Specialist Child Roles With Enforced Tool Subsets

**Verdict**

`KEEP with update`

**Source files**

- `src/tools/AgentTool/built-in/exploreAgent.ts`
- `src/tools/AgentTool/built-in/planAgent.ts`
- `src/tools/AgentTool/built-in/verificationAgent.ts`
- `src/tools/AgentTool/agentToolUtils.ts`
- `src/utils/forkedAgent.ts`

**What it is**

OpenClaude pairs child-role prompts with enforced tool subsets and permission shaping.

**Why it is useful for Mister Smith**

This is a better packet `026` transfer than prompt-only specialist agents. It makes delegated roles
inspectable and safer.

**Concrete adaptation path**

- define a small Smith child-role set:
  - explorer
  - planner
  - verifier
  - repairer only if needed later
- bind roles to explicit tool or capability subsets
- keep role grant logic outside prompt text

**Risk and compatibility caveats**

- OpenClaude injects some allowed tools by mutating permission context; Smith should use explicit
  policy records instead of hidden context mutation
- role count should stay small

## 6. Cross-Worker Permission Mailbox

**Verdict**

`KEEP with update`

**Source files**

- `src/utils/swarm/permissionSync.ts`

**What it is**

OpenClaude lets workers raise structured permission requests to a leader and receive structured
responses back instead of holding broad standing permissions.

**Why it is useful for Mister Smith**

This is a strong later part of the packet `026` and `027` seam because it keeps least privilege
alive across delegation chains.

**Concrete adaptation path**

- add a coordinator-owned approval mailbox for delegated execution
- record:
  - requested action
  - source child
  - approval outcome
  - modified input if any
- project approval history into operator-visible proof

**Risk and compatibility caveats**

- keep this additive on top of packet-024 action-bound enforcement
- do not let mailbox approval become a vague "allow everything" override

## 7. Surface-Specific Command Gating

**Verdict**

`KEEP with update`

**Source files**

- `src/utils/processUserInput/processUserInput.ts`
- `src/utils/processUserInput/processSlashCommand.tsx`

**What it is**

OpenClaude does not expose the same command surface everywhere. Remote-control surfaces only get a
safe subset, and some background results re-enter the main loop as hidden prompts rather than raw
direct execution.

**Why it is useful for Mister Smith**

This is a strong packet `027` operator-safety and protocol-boundary pattern. It is also useful now
for CLI, HTTP, cockpit, and later remote surfaces.

**Concrete adaptation path**

- define explicit action allowlists by surface:
  - CLI
  - HTTP API
  - operator cockpit
  - later remote protocol bridges
- keep hidden or synthetic re-entry events clearly tagged as system-generated

**Risk and compatibility caveats**

- do not let hidden re-entry blur proof boundaries
- surface policy must be operator-visible and testable

## 8. Secret-Minimized Remote Bridge

**Verdict**

`SPLIT or DEFER`

**Source files**

- `src/bridge/sessionRunner.ts`
- `src/remote/RemoteSessionManager.ts`
- `src/remote/remotePermissionBridge.ts`

**What it is**

OpenClaude launches remote or bridged child sessions with a strict environment allowlist, a
session-scoped access token, and an explicit control path for permission and interrupt traffic.

**Why it is useful for Mister Smith**

This is strong reference material, but it is later than packet `026`'s local runtime and later than
packet `027`'s first discovery or lifecycle mapping slice.

**Concrete adaptation path**

- keep this for a later remote-executor packet
- preserve the key rules:
  - explicit env allowlist
  - session-scoped access token
  - control protocol for permission and interrupt traffic

**Risk and compatibility caveats**

- do not pull remote bridge complexity into the current local default runtime path
- this should wait until Smith has a stronger local coordinator-subagent proof baseline
