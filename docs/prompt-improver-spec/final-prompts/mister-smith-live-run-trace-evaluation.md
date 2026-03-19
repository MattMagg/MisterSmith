# Mister Smith Live Run Trace Evaluation

Use this prompt to start a new Codex session in the Mister Smith repo.

---

You are working in the Mister Smith repository at `<repo_root>`.

Your mission in this session is to perform **one real live run of Mister Smith**, trace the run
thoroughly, and evaluate what that run actually proves about the current system.

Treat **current code and observed runtime behavior** as the primary truth.

Do not treat old docs, old issue states, or prior summaries as truth unless they still match the
current code and the current run.

## Objective

By the end of this session, you must have:

1. verified the current live runtime path from code
2. executed one real runtime-backed Mister Smith run
3. captured durable evidence from the run, not just a verbal summary
4. traced the run through the key runtime and operator surfaces
5. stated clearly what the run proves, what it does not prove, and what mismatches remain

## Core Constraints

- do a **real live run**, not a mock-only or code-only audit
- keep proof boundaries explicit: live evidence versus inference from code
- do not use Linear or Symphony as the primary truth source for runtime behavior
- do not silently change the provider path
- if the current code still uses a fixed provider/model path, name it explicitly
- do not claim provider-neutral or `MockProvider` proof unless you actually run that path
- leave one durable evidence note in the repo at `<evidence_note_path>`

## Start Sequence

Before running anything, read these files in order:

1. `AGENTS.md`
2. `CLAUDE.md`
3. `README.md`
4. `docs/current-state.md`
5. `docs/plans/2026-03-18-ms-76-runtime-wiring.md`
6. `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md`

Then verify the current live runtime path directly in code. Do not skip this.

Primary code surfaces to inspect:

- `crates/mister-smith-app/src/main.rs`
- `crates/mister-smith-app/src/bootstrap.rs`
- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/conversation.rs`
- `crates/mister-smith-agents/src/agent.rs`
- `crates/mister-smith-agents/src/roles/executor.rs`
- `crates/mister-smith-agents/src/tool_bus.rs`

Your goal in this code pass is to verify:

- what command and HTTP surfaces are actually live
- what provider/model path the current runtime uses
- whether the live path is supervised
- whether step execution crosses a ToolBus boundary
- what runtime/result fields you should expect to see during and after the run

## Live Run Scope

The primary run should use the one-shot task path, not the development workflow queue.

Use a real task submission through the current runtime-backed surface. The task itself should be
non-trivial enough to produce planner, worker, and join behavior, but not so large that the run
becomes hard to trace cleanly.

Use this as the baseline task unless current code or docs prove a better minimal live task:

`<live_task_description>`

If the current code or environment requires a different task shape to produce an honest trace,
adjust it and explain why.

## Environment Verification

Before starting the run, verify and record:

- current branch and worktree state
- local infrastructure required by the current path
- NATS and JetStream availability
- PostgreSQL availability
- current provider auth surface
- current base URL, using `<base_url>` if needed

If any of these are missing or broken, do not guess. Record the blocker exactly, attempt bounded
recovery if it is local and reversible, and stop if the blocker prevents an honest live run.

## Run Procedure

Perform the run in this order:

1. verify current repo and runtime truth from code
2. start or confirm required local infrastructure
3. start the Mister Smith runtime
4. verify readiness and capture startup evidence
5. submit one real task through the current runtime-backed path
6. track the resulting workflow or task to terminal state
7. inspect autonomy state and any related operator surfaces
8. compare the observed run with the current code path and current docs

Keep the runtime attached or otherwise preserve enough log evidence to trace what happened.

## Evidence Checklist

You must capture and cite the following where available:

- runtime startup command and environment assumptions
- readiness proof
- submitted task payload
- accepted task or workflow identifier
- task result payload
- autonomy list or autonomy status output
- runtime log lines showing the actual execution path
- the current provider and model used by the run
- the current execution-boundary markers and lifecycle markers if present

Specifically check for current runtime markers described in
`docs/plans/2026-03-18-ms-76-runtime-wiring.md`, including where available:

- `runtime_execution_mode.planner_lifecycle`
- `runtime_execution_mode.executor_lifecycle`
- `runtime_execution_mode.workflow_runner`
- `runtime_execution_mode.execution_boundary`
- per-step execution boundary markers
- ToolBus evidence such as `workflow.execute_step`
- topology, worker count, branch state, and routing history surfaced to operators

If a marker is missing, say whether that is:

- expected because the current code does not expose it there
- unexpected and likely a regression
- unclear and requires follow-up

## Evaluation Questions

Your final evaluation must answer these questions directly:

1. Did the run use the current live runtime path described by the code?
2. What exact provider/model path did it use?
3. What parts of the run were clearly live and runtime-backed?
4. What parts were inferred from code rather than directly proved by the run?
5. Do the observed results match `docs/current-state.md`?
6. Do the observed results match `docs/plans/2026-03-18-ms-76-runtime-wiring.md`?
7. What mismatches, shortcuts, or remaining gaps did you find?
8. What is the narrowest honest next step if the run reveals a gap?

## Durable Artifact Requirement

Write one durable evidence note to `<evidence_note_path>`.

That note must include:

- objective
- date and environment used
- files read for grounding
- commands run
- identifiers produced by the live run
- logs and operator evidence captured
- what was proved
- what was not proved
- mismatches and open questions
- recommended next step

Do not leave the result as terminal output only.

## Do Not Claim

Do not claim any of the following unless you directly proved them in this session:

- provider-neutral runtime proof
- `MockProvider` runtime proof
- JetStream KV budget or distributed control-loop proof
- external-agent interoperability proof
- full production readiness

## Use Of Development Workflow Tools

Linear, Symphony, Ralph, and related development workflow surfaces are secondary here.

You may consult them only if they help explain development-state context, but they are not proof of
the product runtime path. Do not let them replace code inspection or live evidence.

## Stop Conditions

Stop and report clearly if:

- local infrastructure required for an honest live run cannot be brought up
- provider auth for the current code path is unavailable
- the runtime never becomes ready
- task submission is not actually runtime-backed on the current path
- the run cannot reach a terminal state without broad unrelated changes

If you stop, leave a durable blocker note instead of a false success claim.

## Final Response Requirements

At the end of the session:

- provide a concise summary
- link to the durable evidence note
- state the exact provider/model used
- state what the run proved
- state what remains unproven
- state the narrowest next step, if any
