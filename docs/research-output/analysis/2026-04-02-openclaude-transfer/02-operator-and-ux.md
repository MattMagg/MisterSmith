# Operator And UX Transfer Ideas

## 1. Live Work Cockpit For Subordinate Runtime State

**Verdict**

`KEEP with update`

**Source files**

- `src/components/tasks/BackgroundTasksDialog.tsx`
- `src/tasks/LocalAgentTask/LocalAgentTask.tsx`
- `src/Task.ts`

**What it is**

OpenClaude gives one live surface for long-lived work units such as local agents, remote agents,
shell jobs, and workflow tasks.

**Why it is useful for Mister Smith**

This is a direct operator-clarity match for packet `026`. If Smith exposes real subordinate runtime
state, operators need one place to inspect and act on it. The useful idea is not the exact UI. It
is the unified work-unit visibility.

**Concrete adaptation path**

- extend `/Users/macmain/MisterSmith/apps/operator-console/` with one subordinate-runtime panel
- project bounded child work under the parent workflow:
  - delegated agent work
  - verifier passes
  - repair loops
  - diagnostics runs
- keep actions narrow:
  - inspect
  - cancel
  - retry where valid
  - open evidence

**Risk and compatibility caveats**

- do not build a second task system beside packet-022 workflow truth
- child work must stay clearly attached to parent workflow identity

## 2. Real Plan Mode As Permission State

**Verdict**

`KEEP with update`

**Source files**

- `src/tools/EnterPlanModeTool/EnterPlanModeTool.ts`
- `src/commands/plan/plan.tsx`

**What it is**

OpenClaude turns plan mode into a real permission state. The model can enter it, actions are
restricted while active, and session state changes when the mode flips.

**Why it is useful for Mister Smith**

This is execution-safety infrastructure, not just UX. It is still lower leverage than packet `026`
and `027` core coordination work, but it is a reasonable supporting idea for operator clarity and
safer session control.

**Concrete adaptation path**

- if Smith takes this on, keep it small:
  - explicit planning state on the session
  - read-only or planning-safe action subset while active
  - explicit transition back to execution

**Risk and compatibility caveats**

- do not let plan mode consume packet `026` or `027` scope
- this should reinforce existing planning posture, not become a large UI project

## 3. Session Notes And Visible Continuity Memory

**Verdict**

`KEEP with update`

**Source files**

- `src/services/SessionMemory/sessionMemory.ts`
- `src/services/SessionMemory/sessionMemoryUtils.ts`
- `src/services/SessionMemory/prompts.ts`

**What it is**

OpenClaude maintains durable session memory and updates it during long-lived use.

**Why it is useful for Mister Smith**

The useful transfer is not hidden automatic memory. It is operator-visible continuity notes. Smith
already has session surfaces. The near-term leverage is explicit resumability, not an opaque memory
subsystem.

**Concrete adaptation path**

- keep visible session notes with fixed fields:
  - mission
  - current state
  - constraints
  - last evidence
  - next likely step
- only later consider bounded automatic summarization into the same visible structure

**Risk and compatibility caveats**

- do not create hidden continuity claims that operators cannot inspect
- do not let automatic memory drift from packet-023 proof wording

## 4. Capability Catalog With Trust, Auth, And Validation State

**Verdict**

`KEEP with update`

**Source files**

- `src/services/plugins/PluginInstallationManager.ts`
- `src/commands/plugin/PluginSettings.tsx`
- `src/commands/plugin/PluginTrustWarning.tsx`
- `src/services/mcp/useManageMCPConnections.ts`

**What it is**

OpenClaude exposes optional capability sources with install, trust, failure, and refresh state.

**Why it is useful for Mister Smith**

The first-pass writeup was too close to plugin UX. The real Smith fit is packet `027` operator
clarity: show where a capability came from, whether it is trusted, whether it is authenticated, and
whether it is actually executable.

**Concrete adaptation path**

- add one operator capability catalog view for:
  - local ToolBus capabilities
  - MCP capabilities
  - later remote protocol descriptors
- show:
  - source
  - trust state
  - auth state
  - validation state
  - execution availability

**Risk and compatibility caveats**

- do not build a plugin marketplace
- keep discovery metadata separate from execution permission

## 5. Unified Command Palette And Conditional Helpers

**Verdict**

`REMOVE as misfit`

**Source files**

- `src/commands.ts`
- `src/skills/loadSkillsDir.ts`
- `src/utils/plugins/loadPluginCommands.ts`

**Why it is not a strong Smith transfer**

This is useful UX polish, but it is not frontier leverage for Smith right now. It is easy for this
kind of feature to pull the roadmap toward framework shell parity instead of coordination runtime
and capability boundary work.

## 6. IDE Bridge, Keybindings, And Voice

**Verdict**

`REMOVE as misfit`

**Source files**

- `vscode-extension/openclaude-vscode/`
- `src/keybindings/`
- `src/commands/voice/voice.ts`

**Why it is not a strong Smith transfer**

These are convenience features. They are not where Smith should spend packet `026` or `027` energy.
