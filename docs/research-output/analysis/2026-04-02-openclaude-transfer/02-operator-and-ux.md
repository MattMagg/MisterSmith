# Operator And UX Transfer Ideas

## 1. Real Plan Mode

**OpenClaude feature**

Planning is a real session state, not just advice in the prompt. Entering plan mode changes what
the model is allowed to do and routes the user through dedicated approval UI.

**OpenClaude evidence**

- `src/commands/plan/plan.tsx`
- `src/tools/EnterPlanModeTool/EnterPlanModeTool.ts`
- `src/components/permissions/PermissionDialog.tsx`

**Why it matters**

This turns “plan first” into product behavior. It also matches the repo’s own insistence on
written plans before non-trivial work.

**Mister Smith fit**

`High fit now`

Mister Smith already has retained sessions, operator surfaces, and plan-first repo posture. A real
plan mode would tighten those together.

**How to translate into Mister Smith**

- add a `plan_only` or `planning_state` mode to:
  - CLI session handling in `crates/mister-smith-app/`
  - session endpoints under `/api/v1/sessions`
  - `apps/operator-console/`
- while active:
  - allow read/explore/planning actions
  - block normal execution actions
  - require explicit operator approval before switching to execution
- persist that state in retained session metadata

**Suggested validation**

- tests proving execution tools are blocked while plan mode is active
- console coverage for enter, approve, and exit flows

## 2. Live Work Cockpit

**OpenClaude feature**

Background agents, shell jobs, remote sessions, and other long-lived work appear in one live
control surface with stop, foreground, retry, and status actions.

**OpenClaude evidence**

- `src/components/tasks/BackgroundTasksDialog.tsx`
- `src/tools/AgentTool/AgentTool.tsx`
- `src/keybindings/defaultBindings.ts`

**Why it matters**

Once the system has more than one run in flight, raw logs stop being enough. Operators need a live
work surface.

**Mister Smith fit**

`High fit now`

The current operator console already has separate runs, sessions, agents, and health views. This
idea is mostly a consolidation and control improvement, not a product-boundary change.

**How to translate into Mister Smith**

- add one operator-console panel for live work across:
  - runs
  - retained sessions
  - subordinate work when Smith grows it
  - proof or diagnostics jobs
- support:
  - stop
  - foreground or inspect
  - retry
  - open output or evidence path

**Suggested validation**

- console interaction tests
- integration tests for action wiring against the existing runtime endpoints

## 3. Unified Command Palette Plus Conditional Helpers

**OpenClaude feature**

Built-in commands, local skills, project skills, and plugin commands all load into one command
surface. Some helpers only become visible when the current file or path context matches.

**OpenClaude evidence**

- `src/commands.ts`
- `src/skills/loadSkillsDir.ts`
- `src/utils/plugins/loadPluginCommands.ts`

**Why it matters**

This lowers operator friction. The right helper is available in the same surface instead of spread
across separate systems.

**Mister Smith fit**

`High fit now`

Mister Smith already has a CLI, operator console, Smith MCP, and repo-local command concepts. A
unified command surface fits the current posture well.

**How to translate into Mister Smith**

- create one Smith command palette that merges:
  - runtime verbs
  - operator actions
  - MCP-backed controls
  - packet-local repo helpers where appropriate
- optionally add path-triggered helpers for:
  - `specs/025-step-level-intelligence-v2/`
  - runtime proof notes
  - operator-console files

**Suggested validation**

- command discovery tests
- path-trigger tests
- no accidental exposure of unsafe commands on remote or reduced-authority surfaces

## 4. Durable Session Memory And Notes

**OpenClaude feature**

Sessions have manual memory controls, automatic memory extraction and consolidation, template
budgets, and shared memory sync patterns.

**OpenClaude evidence**

- `src/commands/memory/memory.tsx`
- `src/services/SessionMemory/sessionMemoryUtils.ts`
- `src/services/SessionMemory/prompts.ts`

**Why it matters**

Retained sessions are much more useful when they also keep structured operator notes and bounded
continuity summaries.

**Mister Smith fit**

`High fit now`

Mister Smith already exposes retained sessions in the operator console. The current gap is better
operator-visible continuity, not the existence of session storage itself.

**How to translate into Mister Smith**

- add session notes with a fixed structure:
  - mission
  - current state
  - key constraints
  - last verified evidence
  - next likely action
- later, add bounded automatic consolidation when a session crosses thresholds
- keep this operator-visible and separate from hidden conversation memory claims

**Suggested validation**

- note create/update/read tests
- size-budget tests if automatic consolidation is added

## 5. Capability Catalog With Trust Labels

**OpenClaude feature**

Optional capability sources have install, refresh, failure, and trust-state UI instead of staying
hidden in config.

**OpenClaude evidence**

- `src/services/plugins/PluginInstallationManager.ts`
- `src/commands/plugin/PluginSettings.tsx`
- `src/commands/plugin/PluginTrustWarning.tsx`

**Why it matters**

As optional capability surfaces grow, hidden configuration drift becomes an operator problem.

**Mister Smith fit**

`Conditional fit next`

Do not copy a plugin marketplace. The useful part is the operator-facing catalog: source, trust,
validation, and status.

**How to translate into Mister Smith**

- build a Smith capability catalog for:
  - MCP servers
  - tool packs
  - agent profiles
- show:
  - source label
  - trust state
  - auth state
  - last validation state
  - current availability

**Suggested validation**

- catalog state tests
- auth-expired and disabled-source coverage

## 6. IDE Bridge, Keybindings, And Voice

**OpenClaude feature**

OpenClaude has a VS Code extension, user-overridable keybindings, and voice-input support.

**OpenClaude evidence**

- `vscode-extension/openclaude-vscode/`
- `src/keybindings/`
- `src/commands/voice/voice.ts`
- `src/services/voice.ts`

**Why it matters**

These features improve operator ergonomics, but they are not the main architectural leverage.

**Mister Smith fit**

`Later or do-not-copy`

The most realistic near-term version is a minimal VS Code helper or more console shortcuts. Voice
input is a later desktop convenience feature, not a current platform priority.

**How to translate into Mister Smith**

- near-term:
  - add a small shortcut layer in the operator console and CLI
  - consider a minimal VS Code bridge that opens runtime status or session detail
- later:
  - desktop dictation for next session turn or operator note

**Suggested validation**

- shortcut tests
- extension smoke checks if a VS Code bridge is built
