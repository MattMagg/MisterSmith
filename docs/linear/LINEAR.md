# Linear Protocol — Mister Smith

**Workspace**: Matt Agent Ops (`linear.app/agentic-ops`)
**Team**: MisterSmith (`MS`)
**Team URL**: `https://linear.app/agentic-ops/team/MS/all`

This document now serves two roles:

- active Linear issue and labeling guidance for direct Codex execution
- historical reference for older Symphony automation design further below

For the active Smith MCP route, prefer `AGENTS.md`, `docs/current-state.md`, and
`docs/plans/2026-04-05-smith-mcp-direct-execution-overhaul.md`.

## Workspace Structure

### Hierarchy

```
Initiative (strategic goal)
└── Project (phase or workstream)
    └── Milestone (deliverable checkpoint)
        └── Issue (actionable work item)
```

### Initiatives

Use initiatives as the strategic layer, not as Symphony's dispatch boundary. Historical initiatives
remain useful for reporting and status updates, but the current repo-wide router is
`docs/current-state.md`. The latest landed packet authorities are
`specs/023-runtime-truth-and-run-trace/`,
`specs/024-agent-boundary-security-hardening/`, and
`specs/025-step-level-intelligence-v2/`. The next implementation-ready packet is
`specs/026-first-real-coordinator-subagent-runtime/`, and the last explicit bounded live-proof
note remains `docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md`.

### Current Strategy

- `MS-45`, `MS-46`, and `MS-47` are complete post-recovery operating-system epics; treat them as
  backlog history rather than current runnable direction.
- the packet-016 family (`MS-96` parent, `MS-97` through `MS-100` child slices) is complete on
  `main`; treat it as closed backlog history rather than current runnable direction
- packets `019` through `025` are complete on `main`; treat them as landed frontier history unless
  current repo truth shows a defect
- packet `026` is the next implementation-ready packet for direct execution when that slice is
  deliberately selected
- packets `027` and `028` remain later scaffolds and should stay in backlog until they are
  deliberately selected
- the historical `Smith MCP Development` project is archived and should not be reopened as a
  separate forward program
- the next active planning move is not an automatic refill; start from `docs/current-state.md`
  plus `docs/direction.md`, `specs/023-runtime-truth-and-run-trace/`,
  `specs/024-agent-boundary-security-hardening/`,
  `specs/025-step-level-intelligence-v2/`, and
  `specs/026-first-real-coordinator-subagent-runtime/`, confirm current repo truth, and only then
  create one fresh bounded packet slice
- `MisterSmith Validated Backlog` should hold only genuinely open frontier work in `Backlog` after
  fresh repo-grounded validation and explicit packet framing
- Do not move these issues into `Todo` just to keep automation busy. Select only the next bounded
  runnable slice.
- Keep `Todo` intentionally small and explicit. It should reflect real near-term execution, not a
  generic parking lot.
- the current repo-owned direction notes are `docs/current-state.md`, `docs/direction.md`,
  `specs/023-runtime-truth-and-run-trace/`, `specs/024-agent-boundary-security-hardening/`,
  `specs/025-step-level-intelligence-v2/`,
  `specs/026-first-real-coordinator-subagent-runtime/`, and
  `docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md`; packet-021 and packet-016
  notes remain useful historical support references

### Projects

Projects serve planning, reporting, and issue organization. Do not assume project placement alone
decides whether work is runnable. For the active Smith MCP path, runnable work is determined by
issue truth, workpad state, review state, and available packet context.

| Project | Role | State | Notes |
|---------|------|-------|-------|
| MisterSmith Execution Queue | Historical Automation Queue | In Progress | Legacy queue-oriented project retained for traceability; not the active Smith MCP boundary |
| MisterSmith Validated Backlog | Validated Backlog | Backlog | Curated repo-validated future work until it is deliberately selected for direct execution |
| MisterSmith Workspace Docs | Docs Hub | Backlog | Visible home for architecture, workflow, and Linear operating docs |
| Archived historical phases | Historical Phase | Completed | Completed phase and batch projects are archived by default and can be unarchived if needed |
| Smith MCP Development | Historical Control Plane | Archived | Historical Smith-first workflow build-out; do not reopen as a parallel product program |

### Project Role Labels

Project role is encoded with project labels, not additional issue labels:

- `Active Queue`: legacy automation queue context retained for reporting
- `Validated Backlog`: repo-validated work that should not start yet
- `Historical Phase`: completed phase or batch projects retained for context

### Current Direct-Execution Boundary

The active Smith MCP route no longer depends on `WORKFLOW.md`, a watched `project_slug`, or
queue-stage tools. Runnable work is determined by:

- the issue state and blockers
- the single `## Codex Workpad`
- PR and review state
- available packet docs and validation context

`Human Review` is still a real workflow state, but it is now resolved through direct Codex review
and merge work rather than queue dispatch.

Do not collapse all planning into a single giant `MisterSmith` project just to satisfy that current
runtime limitation. If project switching becomes the real bottleneck, prefer a dedicated execution
project such as `MisterSmith Queue` over flattening the entire workspace taxonomy.

## Smith MCP Control-Plane Mapping

Use Smith as the default workflow layer over this Linear model.

- route and state discovery:
  `route_workflow_request`, `get_control_plane_snapshot`, `get_issue_execution_snapshot`,
  `prepare_direct_execution`, `resolve_issue_lifecycle`
- issue and workpad mutation:
  `save_linear_issue`, `save_issue_workpad`
- backlog, packet prep, and review guidance:
  `materialize_backlog_slices`, `translate_speckit_tasks`, `prepare_speckit_context`,
  `review_merge_status`
- Ralph and SpecKit glue:
  `prepare_ralph_packet`, `record_ralph_outcome`

For frozen packet execution, keep the workflow hybrid: use Smith MCP to route, reconcile state,
and prepare the active slice, then run the repo-local `speckit.implement` surface before
implementation against `specs/`.

Use raw Linear fallback only for workspace administration or metadata operations that Smith does not
yet model.

### Cycles

Use cycles only for scheduled near-term work.

- Leave `Triage` and validated backlog items cycle-free until they are actually selected.
- Assign a cycle only when the issue is moving toward `Todo`.
- Do not bulk-assign the validated backlog to cycles just for visibility.

## Label System

### Type Labels (standalone)

| Label | Color | Use For |
|-------|-------|---------|
| Bug | `#EB5757` | Incorrect behavior |
| Feature | `#BB87FC` | New capability |
| Improvement | `#4EA7FC` | Enhancement to existing functionality |
| Chore | `#95A2B3` | Maintenance, cleanup, dependency updates |
| Research | `#F2994A` | Research tasks, exploration |
| Docs | `#6FCF97` | Documentation changes |
| Security | `#EB5757` | Security-related work |
| Performance | `#F2C94C` | Performance optimization |
| Spec Violation | `#E87461` | Implementation doesn't match specification |

### Group Labels (exclusive — only one per group per issue)

**Phase** (`#5E6AD2`): Phase 1: Foundation, Phase 2: Runtime, Phase 3: Actors,
Phase 4: Transport, Phase 5: Security, Phase 6: Persistence, Phase 7: Agents,
Phase 8: Operations, Phase 9: LLM, Phase 9.1: Security Hardening, Phase 10:
Frontier Autonomy

**Crate** (`#26B5CE`): crate:core, crate:config, crate:runtime, crate:monitoring, crate:events, crate:async, crate:resources, crate:actor, crate:supervision, crate:transport, crate:nats, crate:http, crate:grpc, crate:mcp, crate:security, crate:persistence, crate:llm, crate:agents, crate:app, crate:integration-tests

**Source** (`#F2994A`): source:code-review, source:spec-validation, source:research, source:ci-cd

### Operational Standalone Labels

- `Validated`: the work has been repo-validated and accepted as real

### Group Label Constraint

Linear group labels are **exclusive** — you can only apply one label from each group to an issue. For issues spanning multiple crates, apply the **primary** crate label and note secondary crates in the issue description (e.g., "Also touches: `crate:transport`").

### Priority

Use Linear's built-in priority field, not labels:

| Value | Name | Use For |
|-------|------|---------|
| 1 | Urgent | Critical bugs, security vulnerabilities, blocking issues |
| 2 | High | Major bugs, spec violations, important features |
| 3 | Normal | Standard work items |
| 4 | Low | Nice-to-haves, minor improvements |

## Issue Conventions

### Creating Issues

Every issue should include:

1. **Title**: Short, descriptive (under 70 characters)
2. **Project**:
   - leave raw intake in `Triage` minimally routed until validated
   - use `MisterSmith Validated Backlog` for validated future work not yet selected for execution
   - use an active working project only when the issue is deliberately in play now
3. **Milestone**: Only when the project actively uses milestones
4. **Priority**: 1-4 using Linear's built-in field
5. **Labels**:
   - one type label
   - one primary crate label
   - one source label when the source is known
   - one phase label when it adds routing value
   - `Validated` when the finding is repo-grounded
6. **Assignee**: Optional until real human ownership exists
7. **Cycle**: Only after the issue is actually scheduled; validated backlog items can remain cycle-free
8. **Description**: Structured markdown with:
   - Context (what spec, finding, or requirement drives this)
   - Location (`file:line` references for bugs)
   - Workflow expectations when a mandate, loop runner, or spec artifact is
     required for execution
   - Task checklist (for multi-step work)
   - Acceptance criteria

### Issue Description Template

```markdown
## [Category]: [One-line summary]

**Severity**: [Critical/Major/Minor]
**Spec**: [spec file reference or FR-### requirement]
**Tasks**: [task IDs if from a task file, e.g., S001-S007]

### Location

`crates/mister-smith-<crate>/src/<file>.rs:<lines>`

### What

[Description of the issue with code quotes if applicable]

### Impact

[Why this matters — correctness, safety, performance]

### Workflow Expectations

- [Required mandate, issue-local workflow, or loop-runner instructions]

### Task Checklist

- [ ] **S001**: [Description]
- [ ] **S002**: [Description]

### Acceptance Criteria

- [Criterion 1]
- [Criterion 2]
```

### Blocking Relationships

Use Linear's blocking feature for dependency chains. Document the dependency in the issue description as well:

```
**Blocked by**: [issue identifier] — [reason]
```

### Admission Rules

- `Triage`: raw suggestions, scanner output, Slack/Asks intake, or CI findings that are not yet repo-validated
- `Backlog`: validated but unscheduled
- `Todo`: unblocked, selected for near-term execution, and safe to start now
- An empty `Todo` list means nothing is runnable right now; it does not mean the state disappeared
- Do not move work into `Todo` simply to make it visible

### Branch Naming

Linear auto-generates branch names from issues. The convention is:

```
matthewtmaggio/ms-<number>-<slug>
```

Include `MS-###` in commit messages and PR titles to link them to Linear issues.
No execution should finish with local uncommitted or untracked changes. Leftovers must be reviewed,
landed on a branch/PR if valid, or explicitly dropped after verifying they are already landed or
stale.

## Statuses

| Status | Type | Meaning |
|--------|------|---------|
| Triage | triage | Raw intake not yet validated |
| Backlog | backlog | Validated but unscheduled |
| Todo | unstarted | Unblocked work selected for near-term execution |
| In Progress | started | Actively being worked on |
| In Review | started | Optional human-only review state |
| Human Review | started | Review checkpoint for direct Codex review or external reviewer input |
| Rework | started | Reviewer requested changes |
| Merging | started | Review is complete and the PR is being landed |
| Done | completed | Merged and verified |
| Duplicate | canceled | Duplicate of another issue |
| Canceled | canceled | Will not be done |

### Status Transitions (Manual)

```
Triage → Backlog (validated but not scheduled)
Backlog → Todo (explicitly selected for execution)
Todo → In Progress (work begins)
In Progress → In Review (human-only flow)
In Review → Done (PR merged)
```

With GitHub integration, branch creation should move to `In Progress`, PR open/review-requested
should move issues to `Human Review`, review-complete work should move to `Merging`, and merge to
`main` should move to `Done`, but only after the issue workspace has been reconciled back to a
clean local checkpoint.

## Documents

Linear documents are used for reference material linked to projects:

| Document | Project | Purpose |
|----------|---------|---------|
| Architecture Overview | MisterSmith Workspace Docs | System architecture summary |
| Crate Dependency Map | MisterSmith Workspace Docs | 20-crate workspace structure |
| Development Workflow | MisterSmith Workspace Docs | Build, test, commit conventions |
| Symphony Intake Template | MisterSmith Workspace Docs | Canonical issue intake and readiness template |
| Symphony Linear Feature Matrix | MisterSmith Workspace Docs | Business vs Enterprise feature decisions and adoption stance |
| Symphony Linear Operating Model | MisterSmith Workspace Docs | Current-state audit, target model, and manual follow-up checklist |
| Smith-First Development System | MisterSmith Workspace Docs | Canonical Smith-first development workflow model |
| Smith MCP Direct-Execution Overhaul | MisterSmith Workspace Docs | Current Smith workflow-family implementation status and active cleanup plan |
| Execution Queue Operating Rules | MisterSmith Execution Queue | Historical watched-queue routing and dispatch rules retained for legacy Symphony sessions |
| Validated Backlog Admission Rules | MisterSmith Validated Backlog | Project-specific gating rules for what belongs in curated backlog |
| Phase 9.1 Security Hardening Spec | Phase 9.1 | Security hardening specification |
| Research Corpus Index | MisterSmith Workspace Docs | Research program navigation |

Documents require a project link. For cross-cutting documents, link them to
`MisterSmith Workspace Docs` as the general-purpose project. Historical
phase-specific specs can stay attached to their archived phase projects.

## Templates

The `MisterSmith` team now has a small issue-template set focused on Symphony execution hygiene and
operator handoff. These are standard issue templates, not form templates, and there is currently no
default template for members or non-members.

| Template | Use |
|----------|-----|
| Symphony Execution-Ready Issue | Work that is truly safe to stage into the watched execution queue |
| Validated Backlog Item | Repo-validated work that is real but not yet scheduled |
| Human Review Handoff | PR-ready or review-ready work that needs a final review decision or merge action |
| Workflow / CI Issue | Repo workflow, automation, CI, tooling, or release-process failures |

Keep templates opinionated and small. They should enforce issue quality and routing clarity, not
become a second workflow engine hidden inside descriptions.

## Status Updates

Post initiative-level status updates with health indicators:

| Health | Meaning |
|--------|---------|
| On Track | Progressing as planned |
| At Risk | Issues identified that may cause delays |
| Off Track | Behind schedule, needs intervention |

Include: current state, key blockers, plan for the next cycle.

The historical execution queue project still has a native Linear reminder cadence configured:

- `MisterSmith Execution Queue`: weekly reminder, Mondays at 9:00 local time for the project lead

## Integration Points

### External Knowledge (via Rube MCP)

When agents need external documentation, research, or app connections, prefer Rube as the gateway.

- `Context7 MCP`: version-specific API and library docs
- `GitHub`: PR, branch, review, and CI state
- `Linear`: live issue, project, view, and document state
- `Mem0`: long-term memory when the task actually needs it
- `Parallel`: deeper multi-source research and structured synthesis
- `Tavily`: lighter search and targeted extraction

Preference:

- use `Parallel` for deeper or broader research
- use `Tavily` for quick verification or extraction from known pages
- for Linear product behavior, use official Linear docs and developer docs as the source base

### Slack and Asks

Slack is not part of Symphony's execution control plane today. For unattended agent work, Linear is
the source of truth and Slack is optional.

Use Slack only when it adds one of these concrete benefits:

- low-friction human intake through Slack threads or Asks
- synced human discussion on an issue that should remain visible outside Linear
- project or initiative status distribution for humans

Do not depend on Slack notifications to keep Symphony moving. The queue must remain correct in
Linear even if Slack is ignored.

### GitHub Integration

GitHub connection is still a workspace/integration setting, but the team-level status automation is
now aligned to the Symphony workflow:

Settings > Integrations > GitHub > Connect `matthewmaggio/Mister-Smith`

- Branch creation → issue moves to In Progress
- PR opened / review requested → issue moves to Human Review, not In Review
- PR becomes mergeable → issue moves to Merging
- PR merge to `main` → issue moves to Done
- Include `MS-###` in branch names and commit messages
- GitHub Actions are intentionally disabled in this repository; use local validation plus
  CodeRabbit and operator review instead of waiting for hosted checks
- When the operator has explicitly delegated authority in the active Codex session, the agent may
  perform the Human Review decision and advance the issue to `Merging` without waiting for another
  human to click approve.

### Claude Code Integration (manual setup required)

Settings > Preferences > Enable Claude Code

- "Work on issue" sends issue context to Claude Code
- Custom prompt should reference `CLAUDE.md` and relevant spec files

### Triage (manual setup required)

Team Settings > Triage > Enable

- New issues from integrations go to Triage first
- Review and route during daily triage or sprint planning
- Keep Triage as the only raw-intake state; do not let suggestions bypass it directly into backlog or queue

## Working with Specs

Issues reference specs from the repository:

| Spec Location | Purpose |
|--------------|---------|
| `spec/` | Canonical architecture specifications (the system contract) |
| `specs/` | SpecKit implementation artifacts (build instructions) |
| `ROADMAP.md` | Phase descriptions and gate criteria |

When creating issues from spec violations or new features, always include the governing spec path in the issue description.
When executing a frozen packet from `specs/`, do not treat the spec files alone as the full
workflow. Reconcile Smith control-plane state first, then execute the repo-local `speckit.implement`
path before code changes.

## Adding New Phases

When starting a new phase:

1. Create a new project linked to the appropriate initiative
2. Add a Phase label child (e.g., "Phase 10: ...")
3. Create milestones for each deliverable checkpoint
4. Create issues with task checklists from the phase's `tasks.md`
5. Assign issues to a cycle
6. Post an initiative status update

## Adding New Crates

When adding a new crate to the workspace:

1. Create a new Crate label child (e.g., "crate:new-crate")
2. Update the Crate Dependency Map document in Linear
3. Update `CLAUDE.md` workspace structure

## Historical Symphony Integration

[OpenAI Symphony](https://github.com/openai/symphony) orchestrates Codex agents
against Linear issues. It polls for `Todo` issues, spawns a Codex `app-server`
per issue, and manages the full lifecycle through the status state machine.

This section is historical reference only. The active Smith MCP route no longer depends on a
watched `project_slug` or queue-stage tools. Use it only when you are intentionally operating a
legacy Symphony queue session through `WORKFLOW.md`.

### State Machine

```
┌──────┐    ┌─────────────┐    ┌──────────────┐    ┌─────────┐    ┌──────┐
│ Todo │───▶│ In Progress │───▶│ Human Review │───▶│ Merging │───▶│ Done │
└──────┘    └─────────────┘    └──────────────┘    └─────────┘    └──────┘
                  ▲                    │
                  │              ┌─────▼────┐
                  └──────────────│  Rework  │
                                └──────────┘
```

- **Todo → In Progress**: Symphony picks up the issue, spawns a Codex agent
- **In Progress → Human Review**: Agent opens a PR and requests review
- **Human Review gate**: PR branch must already be pushed and the workspace must pass the clean
  closure check
- **Human Review → Merging**: Delegated agent review or external reviewer approval completes
- **Human Review → Rework**: Reviewer or delegated agent requests changes; agent restarts
- **Rework → In Progress**: Agent re-reads feedback, creates fresh plan
- **Merging → Done**: Agent lands the PR merge via the `land` skill and restores the issue
  workspace to a clean `origin/main` checkpoint

### Configuration

Symphony is configured via `WORKFLOW.md` in the repository root:

| Setting | Value |
|---------|-------|
| `tracker.api_key` | `$LINEAR_API_KEY` |
| `project_slug` | `320a0741920c` (`MisterSmith Execution Queue` Linear `slugId`) |
| `active_states` | Todo, In Progress, Merging, Rework |
| `terminal_states` | Done, Canceled, Duplicate |
| `polling.interval_ms` | 5000 |
| `agent.max_concurrent_agents` | 10 |
| `agent.max_turns` | 150 |

### Historical Queue Contract

- Current watched project: `MisterSmith Execution Queue`
- Current watched slugId: `320a0741920c`
- Validated future backlog: `MisterSmith Validated Backlog`
- Workspace docs hub: `MisterSmith Workspace Docs`
- Completed historical queue: `Phase 9.1: Security Hardening` (archived)
- Queue reminder cadence: weekly Monday 9:00 local time to the project lead
- `Todo` is a live dispatch queue, not a generic "next work" list
- If `Todo` looks empty, verify whether the runnable issue has already been claimed and moved to `In Progress`

### Historical Queue Design

The dedicated execution queue was the legacy dispatch boundary. Keep it small, runnable, and
deliberately staged when you are intentionally running that historical Symphony flow:

- `MisterSmith Execution Queue` is for active or immediately runnable work only
- `MisterSmith Validated Backlog` is for real repo-validated work that is not yet scheduled
- historical phase projects remain for reporting and traceability, not dispatch
- if Symphony concurrency is underutilized, fill the execution queue with additional unblocked
  validated issues instead of retargeting the runtime to a different phase project

### Required Credentials

| Credential | Purpose |
|------------|---------|
| `LINEAR_API_KEY` | Required. Loaded by `./scripts/run-symphony.sh` into the Symphony process environment for tracker access. |
| Codex auth | Required for `codex app-server` to start issue sessions. An existing Codex login is acceptable; this does not have to be `OPENAI_API_KEY`. |
| GitHub auth | Required when agents need to create PRs, fetch review state, or merge via `gh` CLI. |

### Running Symphony

```bash
./scripts/run-symphony.sh
```

`./scripts/run-symphony.sh` is the supported launcher for this repo. It loads this repository's `.env`,
verifies `LINEAR_API_KEY`, defaults `SYMPHONY_ROOT` to `$HOME/symphony`, runs the Elixir app from
`$SYMPHONY_ROOT/elixir`, and starts Symphony against this repo's `WORKFLOW.md`. The runtime workspace
root is `~/.local/share/symphony-workspaces` unless explicitly overridden in `WORKFLOW.md`.

Important: Symphony does not auto-read repo `.env` files. Keeping `LINEAR_API_KEY` in `.env` is not sufficient unless the launch path exports it into the Symphony process environment first.

### Codex Skills

Symphony agents use skills defined in `.codex/skills/`:

| Skill | Purpose |
|-------|---------|
| `commit` | Create conventional commits with rationale |
| `linear` | Raw Linear GraphQL operations |
| `pull` | Merge `origin/main` into current branch |
| `push` | Publish branch and create/update PR |
| `land` | Merge PR when issue reaches Merging |
