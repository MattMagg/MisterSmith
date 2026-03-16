# WinYear Frontier Direction

Date: March 16, 2026
Status: Active

## Objective

Define the post-recovery mainline direction for Mister Smith, update the repo's durable planning
surfaces to match what is now true on `main`, and identify the next frontier epics that should
feed Symphony once they are explicitly staged.

`WinYear` is the name for this next program of work: turn Mister Smith from a validated
multi-agent substrate into a clearly differentiated orchestration operating system.

## Scope

- current mainline truth after the March 16 recovery reconciliation
- the frontier mandate that should govern the next phase of development
- one recommended primary epic plus additional validated backlog epics
- future Symphony staging posture, including what can later be split across parallel agents

## Assumptions

- `main` is now the only durable development branch that matters
- the recovered runtime-backed task path and bounded session path are both landed on `main`
- Symphony's watched queue should remain empty until new work is explicitly staged
- future work should be judged by operating-system leverage, not by framework feature parity

## Constraints

- do not move new work into `Todo` during this refresh pass
- keep the watched project limited to explicitly staged runnable work
- prefer issue slices that can validate honestly with local runtime or targeted crate proof
- keep the frontier mandate intact: Mister Smith is an operating system, not a generic framework

## Non-Goals

- re-open landed Phase 10 or March 16 recovery work unless current validation shows a defect
- bulk-stage the backlog just to keep Symphony busy
- widen the next phase into generic hardening or undifferentiated SDK parity work

## Current Mainline State

- the runtime-backed task path is live on `main` through `mister-smith run`,
  `POST /api/v1/tasks`, and the autonomy inspection surfaces
- the first real provider-backed proof was completed locally on `openai_chatgpt` with `gpt-5.4`
- the first bounded same-agent session slice is live on `main` through the session HTTP and CLI
  surfaces
- the watched Symphony queue is currently empty, which is correct because no new runnable slice has
  been staged into `Todo`
- Linear now treats `WinYear` as the active strategic initiative, with `MS-43` and `MS-44` closed
  against the March 16 landed work and `MS-45` through `MS-48` holding the next backlog epics
- the repo and Linear still needed a narrative refresh because several canonical docs and backlog
  artifacts still described Mister Smith as a framework or still described March 16 work as future

## Frontier Direction

The next phase should optimize for one outcome:

> Mister Smith should outperform other agent systems by behaving like an adaptive orchestration
> operating system: it should compile task structure into execution topology, size teams to the
> work, preserve state across turns and restarts, make routing decisions at step granularity, and
> expose interoperable capability boundaries without surrendering supervised control.

That direction follows the accepted mandate in
`docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md`:

- topology and coordination must beat static teams
- state and memory must survive long-running operation
- routing must become budget-aware, capability-aware, and step-aware
- operator visibility and revocable authority remain mandatory runtime properties

## Recommended Main Epic

### Epic 1: Task-Shape-Aware Orchestration and Dynamic Team Sizing (`MS-45`)

This should be the next primary epic.

Why this is first:

- the research corpus says topology now matters more than model choice for system-level advantage
- the repo already has the Phase 10 execution-graph and topology substrate, so this epic extends a
  real control-plane seam instead of starting from zero
- dynamic team sizing is the clearest way to make Mister Smith feel like an operating system rather
  than a static agent runtime

Primary repo surfaces:

- `crates/mister-smith-agents/src/orchestrator.rs`
- `crates/mister-smith-agents/src/execution_graph.rs`
- `crates/mister-smith-agents/src/topology.rs`
- `crates/mister-smith-agents/src/team.rs`
- `crates/mister-smith-agents/src/scheduler.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- `crates/mister-smith-events/src/autonomy.rs`

Acceptance shape:

- planner output is classified by dependency shape before dispatch
- the runtime selects a topology and team size that match the task instead of defaulting to static
  role fan-out
- operator-visible autonomy surfaces show why the topology and team size were chosen
- validation demonstrates improvement on at least one task class where parallel structure matters

Parallelizable slices once this epic is staged:

1. **Task-shape classification and topology signals**
   - dependency-shape heuristics
   - task structure scoring
   - explicit topology rationale output
2. **Dynamic team sizing and lifecycle integration**
   - spawn fewer or more workers based on branch width and dependency depth
   - keep supervision and scheduler behavior coherent under variable team size
3. **Observability and evaluation harness**
   - expose topology and team-size decisions in autonomy status
   - add a repeatable benchmark harness for sequential versus parallel task classes

## Additional Validated Backlog Epics

### Epic 2: Session Restart-Resume and Distributed Operating State (`MS-46`)

Why it matters:

- the current session slice proves bounded same-agent turns, but the remaining frontier value is
  restart-safe continuation and stronger distributed operating-state recovery
- this is the most direct path from "session feature" to "operating-system state continuity"

Primary surfaces:

- `crates/mister-smith-app/src/conversation.rs`
- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-persistence/src/repository/session.rs`
- `crates/mister-smith-persistence/src/postgres/queries.rs`
- `crates/mister-smith-events/src/autonomy.rs`

Acceptance shape:

- stop and restart the runtime between accepted turns and continue the same idle session
- recover session-linked workflow state without manual repair
- expose restart and resume provenance through operator-visible status

### Epic 3: Step-Level Intelligence and Model Routing Control Loop (`MS-47`)

Why it matters:

- the current runtime can run a real provider-backed workflow, but it still lacks the
  step-granular control loop that would let the operating system adjust model choice, verification,
  and cost posture mid-flight
- this is how Mister Smith turns routing into a live operating-system primitive instead of a static
  config choice

Primary surfaces:

- `crates/mister-smith-llm/src/router.rs`
- `crates/mister-smith-llm/src/model_router.rs`
- `crates/mister-smith-agents/src/orchestrator.rs`
- `crates/mister-smith-agents/src/roles/planner.rs`
- `crates/mister-smith-agents/src/roles/critic.rs`

Acceptance shape:

- per-step verification or confidence scoring can trigger escalation, fallback, or downgrade
- routing decisions become visible in workflow state rather than being hidden inside provider calls
- validation shows reduced cost or improved reliability on at least one representative task bundle

### Epic 4: Capability Kernel and External Agent Interoperability (`MS-48`)

Why it matters:

- the frontier mandate requires revocable capability, not ambient trust
- the research corpus says A2A-style external interoperability is the long-term standard for
  agent-to-agent delegation
- an operating system should be able to expose and consume external agent capability boundaries
  without giving up local supervision or provenance

Primary surfaces:

- `crates/mister-smith-security/src/delegation.rs`
- `crates/mister-smith-mcp/`
- `crates/mister-smith-http/`
- `crates/mister-smith-agents/src/tool_bus.rs`
- `crates/mister-smith-app/src/autonomy.rs`

Acceptance shape:

- capability descriptions are discoverable and enforceable
- external delegation surfaces preserve provenance and local policy
- interoperability remains additive to the zero-trust substrate instead of bypassing it

## Recommended Staging Posture

- keep `MS-45`, `MS-46`, `MS-47`, and `MS-48` in `MisterSmith Validated Backlog` with state
  `Backlog`
- do not move them into `Todo` during this refresh pass
- when the next execution cycle starts, stage only one primary epic into the watched queue at a
  time
- if the primary epic is `Task-Shape-Aware Orchestration and Dynamic Team Sizing`, split it into
  the three bounded slices listed above so Symphony can later run multiple agents in parallel
  without creating overlapping write sets

## Decision Rule For Future Work

Before staging any new issue into the watched queue, ask:

1. Does it strengthen supervised autonomy?
2. Does it improve coordination, supervision, execution, memory, routing, observability, state, or
   distributed behavior enough to make Mister Smith more like an operating system and less like a
   framework clone?

If the answer to both questions is not clearly yes, the issue should stay out of `Todo`.

## Validation For This Note

- repo docs updated to point at this note as the current forward-direction artifact
- Linear strategic and backlog surfaces updated to match the same direction without staging new
  work
- historical recovery and runtime/session notes updated so they point forward here instead of
  implying March 16 work is still pending

## Stop Conditions

- stop before staging any new watched-queue issue during this pass
- stop if a proposed epic cannot be described with a bounded validation bundle
- stop if a proposed direction drifts into generic framework parity instead of operating-system
  leverage
