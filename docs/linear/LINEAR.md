# Linear Protocol — Mister Smith

**Workspace**: Matt Agent Ops (`linear.app/agentic-ops`)
**Team**: MisterSmith (`MS`)
**Team URL**: `https://linear.app/agentic-ops/team/MS/all`

## Workspace Structure

### Hierarchy

```
Initiative (strategic goal)
└── Project (phase or workstream)
    └── Milestone (deliverable checkpoint)
        └── Issue (actionable work item)
```

### Initiatives

| Initiative | Status | Scope |
|-----------|--------|-------|
| Mister Smith Framework | Active | Top-level parent for all sub-initiatives |
| Framework Foundation & Stability | Completed | Phases 1-4 |
| Security & Data Layer | Completed | Phases 5-6 |
| Intelligence Layer | Active | Phases 7-9 |
| Production Hardening | Planned | Phase 9.1+ |
| Research & Innovation | Active | Ongoing research program |

### Projects

One project per implementation phase, plus workstream-specific projects (e.g., "Phase 9 Bug Fixes"). Each project is linked to its parent initiative and the MisterSmith team.

| Project | Initiative | State |
|---------|-----------|-------|
| Phase 1: Foundation | Foundation & Stability | Completed |
| Phase 2: Runtime & Async | Foundation & Stability | Completed |
| Phase 3: Actor System & Supervision | Foundation & Stability | Completed |
| Phase 4: Transport & Messaging | Foundation & Stability | Completed |
| Phase 5: Security | Security & Data Layer | Completed |
| Phase 6: Persistence & State | Security & Data Layer | Completed |
| Phase 7: Agent System | Intelligence Layer | Completed |
| Phase 8: Operations | Intelligence Layer | Completed |
| Phase 9: LLM Provider Integration | Intelligence Layer | Completed |
| Phase 9 Bug Fixes | Intelligence Layer | In Progress |
| Phase 9.1: Security Hardening | Production Hardening | Planned |

### Cycles

Two-week cycles aligned to implementation sprints:

- **Cycle 1** (Mar 9-23): Bug fixes + initial security hardening
- **Cycle 2** (Apr 6-20): Remaining security hardening + verification
- **Gap** (Mar 23 - Apr 6): Spillover buffer

New cycles should follow the same two-week cadence. Assign issues to cycles during sprint planning.

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

**Phase** (`#5E6AD2`): Phase 1: Foundation, Phase 2: Runtime, Phase 3: Actors, Phase 4: Transport, Phase 5: Security, Phase 6: Persistence, Phase 7: Agents, Phase 8: Operations, Phase 9: LLM, Phase 9.1: Security Hardening

**Crate** (`#26B5CE`): crate:core, crate:config, crate:runtime, crate:monitoring, crate:events, crate:async, crate:resources, crate:actor, crate:supervision, crate:transport, crate:nats, crate:http, crate:grpc, crate:mcp, crate:security, crate:persistence, crate:llm, crate:agents, crate:app, crate:integration-tests

**Source** (`#F2994A`): source:code-review, source:spec-validation, source:research, source:ci-cd

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
2. **Project**: The phase or workstream it belongs to
3. **Milestone**: The deliverable checkpoint within the project
4. **Priority**: 1-4 using Linear's built-in field
5. **Labels**: Type label + Phase label + Crate label + Source label (where applicable)
6. **Assignee**: The person responsible
7. **Cycle**: The sprint it's planned for
8. **Description**: Structured markdown with:
   - Context (what spec, finding, or requirement drives this)
   - Location (`file:line` references for bugs)
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

### Branch Naming

Linear auto-generates branch names from issues. The convention is:

```
matthewtmaggio/ms-<number>-<slug>
```

Include `MS-###` in commit messages and PR titles to link them to Linear issues.

## Statuses

| Status | Meaning |
|--------|---------|
| Backlog | Acknowledged but not scheduled |
| Todo | Scheduled for a cycle, ready to start |
| In Progress | Actively being worked on |
| In Review | Code written, PR open, awaiting review |
| Done | Merged and verified |
| Duplicate | Duplicate of another issue |
| Canceled | Will not be done |

### Status Transitions

```
Backlog → Todo (sprint planning)
Todo → In Progress (work begins)
In Progress → In Review (PR opened)
In Review → Done (PR merged)
```

With GitHub integration, branch creation moves to In Progress and PR merge moves to Done automatically.

## Documents

Linear documents are used for reference material linked to projects:

| Document | Project | Purpose |
|----------|---------|---------|
| Architecture Overview | Phase 1 | System architecture summary |
| Crate Dependency Map | Phase 1 | 20-crate workspace structure |
| Development Workflow | Phase 1 | Build, test, commit conventions |
| Phase 9.1 Security Hardening Spec | Phase 9.1 | Security hardening specification |
| Research Corpus Index | Phase 1 | Research program navigation |

Documents require a project link. For cross-cutting documents, link to Phase 1 (Foundation) as the general-purpose project.

## Status Updates

Post initiative-level status updates with health indicators:

| Health | Meaning |
|--------|---------|
| On Track | Progressing as planned |
| At Risk | Issues identified that may cause delays |
| Off Track | Behind schedule, needs intervention |

Include: current state, key blockers, plan for the next cycle.

## Integration Points

### GitHub Integration (manual setup required)

Settings > Integrations > GitHub > Connect `matthewmaggio/Mister-Smith`

- PR merge → issue moves to Done
- Branch creation → issue moves to In Progress
- Include `MS-###` in branch names and commit messages

### Claude Code Integration (manual setup required)

Settings > Preferences > Enable Claude Code

- "Work on issue" sends issue context to Claude Code
- Custom prompt should reference `CLAUDE.md` and relevant spec files

### Triage (manual setup required)

Team Settings > Triage > Enable

- New issues from integrations go to Triage first
- Review and assign during daily triage or sprint planning

## Working with Specs

Issues reference specs from the repository:

| Spec Location | Purpose |
|--------------|---------|
| `spec/` | Canonical architecture specifications (the system contract) |
| `specs/` | SpecKit implementation artifacts (build instructions) |
| `ROADMAP.md` | Phase descriptions and gate criteria |

When creating issues from spec violations or new features, always include the governing spec path in the issue description.

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
