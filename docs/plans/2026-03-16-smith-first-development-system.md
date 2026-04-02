# Smith-First Mister Smith Development System

Date: March 16, 2026
Status: Historical

This note remains useful background for the Smith-first control-plane build-out, but it is no
longer the forward-development authority. Use `docs/current-state.md` for the current repo-wide
router, `specs/023-runtime-truth-and-run-trace/` and
`specs/024-agent-boundary-security-hardening/` for the latest landed packet authorities,
`specs/025-step-level-intelligence-v2/` for the next draft scaffold, and
`docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md` for the latest bounded live-proof
note.

## Objective

Make `smith` MCP the primary operational workflow for developing Mister Smith.

That means Smith becomes the default first hop for development work across:

- repo grounding and resumability
- Linear intake and work tracking
- Symphony queue and unattended execution
- GitHub PR, review, and merge flow
- Ralph-assisted loops
- SpecKit planning and task-pack execution
- validation, recovery, and continuation

This is not a prompt-improver exercise and not a docs-only cleanup pass. The handoff prompt under
`docs/prompt-improver-spec/final-prompts/` is background context for this mission. The mission is
to build the repo-backed workflow system itself.

## Current Authoritative Operating Model

### Source-of-record boundaries

- `WORKFLOW.md`: Symphony runtime contract and issue lifecycle
- `docs/linear/LINEAR.md`: Linear taxonomy, queue boundary, and project roles
- `docs/current-state.md`: current repo-wide router
- `specs/023-runtime-truth-and-run-trace/` and
  `specs/024-agent-boundary-security-hardening/`: latest landed packet authorities
- `specs/025-step-level-intelligence-v2/`: next draft scaffold
- `docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md`: latest bounded live-proof and
  closure note
- `docs/plans/2026-03-15-smith-mcp-comprehensive-workflows.md`: recovered Smith workflow design
- `docs/plans/2026-03-15-smith-mcp-workflow-forensics.md`: evidence for the workflow model
- `docs/plans/2026-03-16-frontier-direction.md`: historical frontier-direction note, useful as
  repo history but not the defining objective for this effort

### Live control-plane facts verified on March 16, 2026

- `smith` MCP is configured through `scripts/run-smith-mcp.sh`
- Symphony is configured through `scripts/run-symphony.sh`
- the watched Linear project slug is `320a0741920c`
- active dispatch states are `Todo`, `In Progress`, `Merging`, and `Rework`
- the watched queue currently has no active issues in active workflow states
- the repo is on `main`

## Implemented Smith Workflow Surface

The current compatibility layer now exposes these workflow families:

- route and state discovery:
  `route_workflow_request`, `get_control_plane_snapshot`, `get_issue_execution_snapshot`,
  `resolve_issue_lifecycle`
- Linear and workpad mutation:
  `save_linear_issue`, `save_issue_workpad`
- backlog and watched-queue control:
  `materialize_backlog_slices`, `plan_queue_stage`, `apply_queue_stage`
- Ralph and SpecKit glue:
  `prepare_ralph_packet`, `record_ralph_outcome`, `prepare_speckit_context`,
  `translate_speckit_tasks`

The supporting tools now share queue and lifecycle logic:

- `sync_linear_with_runtime` and `review_merge_dispatch_cycle` consume the same honest-staging and
  lifecycle signals used by the new workflow-family tools
- broad Smith-first development-system requests now route to `development_workflow` instead of
  falling straight into review or narrow Linear heuristics

### Prompt provenance and boundaries

- `docs/prompt-improver-spec/final-prompts/mister-smith-control-plane-centralization-handoff.md`
  is a valid execution brief for this effort
- `docs/prompt-improver-spec/implementation_plan.md` and `docs/prompt-improver-spec/task.md`
  currently describe a different prompt-improver job and must not be treated as provenance for this
  work

## Task-Type Taxonomy

Smith should own the router and workflow chain for each of these task classes:

| Task class | Smith responsibility | External source of record |
| --- | --- | --- |
| Bootstrap and readiness | audit, route, snapshot, safe adjustments | repo config, local tools |
| Intake and legitimacy | decide if work is real, in scope, and queue-worthy | repo mandate, Linear |
| Repo grounding | collect current contracts, plans, and prior attempts | repo files |
| Planning and workpad generation | create durable plan and execution ledger | repo notes, Linear comment |
| Research and external lookup | route to Rube or external sources when needed | external systems |
| Implementation and validation | route work, preserve scope, pick proof commands | repo code, tests |
| PR, review, merge, rework | reconcile GitHub, CI, review, and queue state | GitHub, Linear |
| Ralph-assisted execution | generate ephemeral prompt input and expected outputs | repo issue/workpad context |
| SpecKit planning | route into SpecKit scaffolds with repo-local policy | `.codex/commands`, `.specify/` |
| Queue staging and Symphony dispatch | stage only honest runnable slices | Linear, Symphony |
| Recovery and continuation | restore context and choose next safe step | repo notes, Linear, GitHub |

## Stale, Fragmented, Or Contradictory Surfaces

- `PROMPT.md`: normalized.
  The checked-in prompt is now explicitly ephemeral and requires regeneration from the active issue
  or workpad before every Ralph run.
- `README.md`: normalized.
  The repo entrypoint now documents Smith workflow families and the default Smith-first operator
  sequence.
- `docs/ms_recent_context.md`: normalized.
  The recovery note now points at both the Smith-first operating model and the currently
  implemented workflow-family surface.
- Smith routing heuristics in `crates/mister-smith-mcp/src/compatibility.rs`: fixed.
  Broad workflow-system requests now route through `development_workflow`, and dedicated
  `backlog_slicing` plus `issue_lifecycle` families exist for the new Smith control-plane surface.
- repo-local Smith skills: normalized.
  The high-signal skills now point at the current Smith-first note and the active workflow-family
  tools.
- prompt-improver artifacts under `docs/prompt-improver-spec/`: mixed.
  The final handoff prompt is relevant, but sibling artifacts belong to another prompt-improver
  run. Treat only the final handoff prompt as scope input.

## Target Smith-Centered Operating Model

Smith should become the entrypoint and control-plane brain for Mister Smith development work.

### What Smith should own

- request routing by task type
- control-plane snapshots and reconciliations
- legitimacy and backlog classification
- generation of durable execution context
- repo-local Ralph and SpecKit glue
- resumability and next-step selection
- workflow chaining across Linear, Symphony, GitHub, validation, and repo notes

### What Smith should not replace

- Linear as the durable tracker and queue source of record
- Symphony as the unattended execution engine for watched-queue issues
- GitHub as the branch, PR, and merge source of record
- Ralph as the loop runner
- SpecKit as the spec and task-pack scaffold

### Core operating rule

Any broad Mister Smith development request should begin at Smith. Smith then decides whether the
work stays in direct Smith flow, becomes Ralph-assisted, enters SpecKit, stages a Symphony slice,
or routes into GitHub and review reconciliation.

## Missing Smith Capabilities And Routing Gaps

### Current strengths

- readiness audit and repo/runtime snapshot
- dedicated `development_workflow`, `backlog_slicing`, and `issue_lifecycle` route families
- direct Linear issue and workpad mutation through Smith
- backlog slicing, honest queue planning, and watched-queue apply controls
- issue lifecycle resolution shared by execution snapshots, queue sync, and review dispatch
- Ralph packet preparation and outcome recording
- SpecKit routing support and task-pack translation into bounded backlog slices
- legitimacy and follow-up classification

### Current gaps

- no top-level autopilot tool yet for "status check and start the next honest workflow" as one
  atomic Smith action
- real watched-queue mutation, Symphony pickup, and end-to-end return-path proofs remain to be run
  for the new workflow-family surface
- Ralph outcome recording and SpecKit translation still need full live proof through the watched
  queue, not just deterministic and read-side validation

## Phased Implementation Slices

### Slice 1: Workflow archaeology and canonical plan

- land this note
- use it as the durable reference for follow-on work
- create a Linear master issue for the program

Validation:

- markdown lint for the new note
- repo checks that the main workflow entry surfaces can point here cleanly

### Slice 2: Contract normalization

- update high-signal docs and Smith skills so they point to one Smith-first workflow entry
- replace the static-prompt posture in `PROMPT.md` with the Smith-managed ephemeral prompt contract
- clarify external system boundaries without rewriting those systems

Validation:

- markdown lint on touched docs and skills
- targeted search confirming one consistent Smith-first entry sequence

### Slice 3: Smith capability alignment

- add a dedicated route family for broad development-system and workflow-architecture requests
- keep the existing tool names stable
- prefer route and chaining improvements over new tools unless a recurring uncovered operation
  remains after contract normalization

Validation:

- `cargo build -p mister-smith-mcp`
- `cargo test -p mister-smith-mcp`
- live Smith route checks for broad development-system requests

### Slice 4: Representative end-to-end proofs

- prove one docs/planning flow
- prove one code-change flow
- prove one PR or review flow
- prove one Ralph or SpecKit-assisted flow

Validation:

- live Smith calls and the narrowest honest proof for each selected flow

## Linear Tracking Model

Create one master issue in Linear representing this program of work.

Use:

- Team: `MisterSmith`
- Project: `MisterSmith Validated Backlog`
- State: `Backlog`
- Title: `Make Smith MCP the central development workflow for Mister Smith`
- Labels: `Improvement`, `Validated`, `crate:mcp`

Do not mark the master issue `Symphony Candidate` yet. The parent issue is too broad for unattended
execution. Create narrower child slices later once the workflow map and acceptance criteria are
stable.

## Stop Conditions

- stop before claiming Smith is the main development workflow if the repo entry surfaces still
  disagree
- stop before adding new Smith tools if routing and existing tool chaining can solve the problem
- stop before staging queue work until a child slice is genuinely execution-ready
