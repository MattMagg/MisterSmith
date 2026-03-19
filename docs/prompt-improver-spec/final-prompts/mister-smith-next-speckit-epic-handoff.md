# Mister Smith Next SpecKit Epic Handoff

Use this prompt to start a new Codex session in the Mister Smith repo.

---

You are working in the Mister Smith repository at `<repo_root>`.

Your mission in this session is to define and write **the next bounded SpecKit packet** for Mister
Smith. This is a planning-and-spec session, not an implementation session.

Treat **current repo truth, current code, and current evaluation evidence** as the primary truth.

Do not treat older direction notes, older issue states, or stale packet labels as current truth
unless they still match the repo and the March 19 checkpoint.

## Objective

By the end of this session, you must have:

1. verified the current forward-development authority and current repo truth
2. identified the next honest bounded epic between landed substrate and proven runtime behavior
3. created one full SpecKit packet under the next numbered `specs/` directory
4. stated clearly what is in scope now, what is deferred, and whether any remaining post-`MS-77`
   external-agent work belongs in the same epic or a later one
5. stopped before implementation, queue staging, or unrelated cleanup

## Core Constraints

- this is a **SpecKit packet creation** session, not an implementation session
- do **not** widen the work into generic cleanup, generic framework parity, or side programs
- do **not** reopen completed Smith-first control-plane work unless current repo truth proves a
  defect
- do **not** stage Symphony queue work or move backlog items into execution during this session
- keep the packet bounded to **one** epic
- use the March 19 checkpoint as the forward-development authority
- if an older note conflicts with current repo truth, prefer current repo truth and the March 19
  checkpoint

## Start Sequence

Before writing anything, read these files in order:

1. `AGENTS.md`
2. `CLAUDE.md`
3. `README.md`
4. `docs/current-state.md`
5. `docs/plans/2026-03-19-central-development-checkpoint.md`
6. `docs/plans/2026-03-19-short-multi-agent-result-evaluation.md`
7. `docs/plans/2026-03-19-framework-comparison-stress-test.md`
8. `docs/plans/2026-03-19-live-run-trace-evaluation.md`
9. `docs/plans/2026-03-19-ms-77-bounded-external-agent-surface.md`
10. `docs/plans/2026-03-19-ms-48-closure-audit.md`

Then read the current research and packet-shape references:

1. `docs/research-output/consolidated/00-MASTER-FINDINGS.md`
2. `docs/research-output/consolidated/02-orchestration-and-self-organization.md`
3. `docs/research-output/consolidated/03-supervision-and-resilience.md`
4. `docs/research-output/consolidated/05-coordination-and-state.md`
5. `docs/research-output/consolidated/08-competitive-landscape-and-ecosystem.md`
6. `specs/014-task-shape-aware-orchestration/spec.md`
7. `specs/014-task-shape-aware-orchestration/plan.md`
8. `specs/014-task-shape-aware-orchestration/tasks.md`

Treat `docs/plans/2026-03-16-frontier-direction.md` as historical context only. It is **not** the
current forward-development authority.

## Grounding Pass In Code

Before choosing the next packet scope, inspect the mainline code surfaces that define the remaining
runtime and operator gaps.

Primary code surfaces to inspect:

- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- `crates/mister-smith-app/src/conversation.rs`
- `crates/mister-smith-agents/src/orchestrator.rs`
- `crates/mister-smith-agents/src/scheduler.rs`
- `crates/mister-smith-agents/src/tool_bus.rs`
- `crates/mister-smith-mcp/src/server.rs`
- `crates/mister-smith-mcp/src/client.rs`
- `crates/mister-smith-mcp/src/compatibility.rs`

Your goal in this code pass is to verify:

- what complex multi-agent execution behavior is already landed
- what final-result visibility is or is not exposed on current operator surfaces
- what external-agent capability surfaces are already complete after `MS-77`
- what is still missing between landed substrate and **proven** runtime behavior

## Scope Decision Rules

The March 19 checkpoint says the next packet should cover the remaining differentiation gap between
landed substrate and proven runtime behavior.

Use that as the starting point, then decide one bounded epic by answering these questions directly:

1. What exact gap remains after the March 19 live-run, short multi-agent, stress, and `MS-77`
   notes?
2. Is the strongest remaining gap primarily:
   - complex multi-agent graph execution proof under harder workloads
   - final result visibility on runtime/operator surfaces
   - a unified contract that binds both together
3. Does any remaining post-`MS-77` external-agent work belong in the same epic, or should it be a
   separate later epic?
4. What scope can be validated honestly with deterministic tests plus runtime or evaluation proof,
   without widening into a generic platform program?

You must choose **one** bounded answer and reject adjacent scope explicitly.

## Packet Output Requirement

Create one full SpecKit packet under the next numbered `specs/` directory.

Determine the next packet root as `<next_specs_root>`, using the next available number after the
existing packets and a descriptive slug such as `<next_packet_slug>`.

At minimum, create:

- `analyze.md`
- `data-model.md`
- `plan.md`
- `quickstart.md`
- `research.md`
- `spec.md`
- `tasks.md`

Use the current `specs/014-task-shape-aware-orchestration/` packet as the shape reference for:

- story structure
- acceptance criteria
- bounded task organization
- validation and evidence expectations
- explicit write-set boundaries for parallelizable tasks

## Required Packet Content

The new packet must:

- define the epic as a bounded next step from the March 19 checkpoint
- state what current truth on `main` is already baseline and must not be reopened as future work
- define clear user stories and acceptance scenarios
- define what runtime or operator surfaces are affected
- define the deterministic validation shape
- define the runtime or evaluation proof shape if runtime behavior is affected
- define explicit non-goals
- define whether post-`MS-77` external-agent follow-up is:
  - inside this packet
  - explicitly deferred to a later epic

If you defer the external-agent follow-up, say so clearly in the packet and keep the scope focused.

## Explicit Non-Goals For This Session

- do not implement the packet
- do not open or stage watched-queue work
- do not restart a Smith-first control-plane program
- do not rewrite older historical packets just to make them look current
- do not create multiple new epics
- do not broaden the packet into provider-neutral routing, JetStream KV rollout, or unrelated
  framework-parity work unless the current checkpoint and evidence make that unavoidable

## Stop Conditions

Stop and report clearly if:

- current repo truth shows the March 19 checkpoint is stale enough that a new checkpoint must be
  written before any packet can be honest
- the remaining work cannot be bounded to one epic without inventing new direction beyond the
  current checkpoint
- the correct next move is to revise an existing packet instead of creating a new one

If you stop, leave a concise durable note explaining why forcing a new packet would be dishonest.

## Final Response Requirements

At the end of the session:

- provide the path to the new `specs/` packet
- state the chosen epic name in simple terms
- state what is explicitly in scope
- state what is explicitly deferred
- state whether post-`MS-77` external-agent work stays in this epic or moves to a later one
- state what validation and runtime/evaluation proof the packet requires

Do not claim implementation progress. This session is complete only when the next SpecKit packet is
written and its scope is honest.
