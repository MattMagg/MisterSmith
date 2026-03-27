# Mister Smith MS-103 Packet-020 Live Evaluation Handoff

You are Codex in a fresh session working in:

- <repo_root>`/Users/macmain/MisterSmith`</repo_root>

Your mission in this session is to perform bounded **live runtime evaluations** for the completed
packet-020 parent issue:

- <linear_issue>`MS-103`</linear_issue>
- Title: `Packet 020: Verifier-gated adaptive orchestration`
- Current known state at handoff:
  - parent issue `MS-103` is `Done`
  - child slices `MS-104` through `MS-107` are landed and `Done`
  - packet `020` is landed on `main`
  - local repo is clean synced `main` at
    <starting_main_sha>`e360323b85183e83351d118f3d24e376d9f22a8b`</starting_main_sha>
  - no newer post-packet-020 bounded phase is frozen yet
- Packet source:
  <packet_source>`/Users/macmain/MisterSmith/specs/020-verifier-gated-adaptive-orchestration/`</packet_source>
- Suggested evidence note:
  <evidence_note_path>`/Users/macmain/MisterSmith/docs/plans/2026-03-27-packet-020-live-evaluation.md`</evidence_note_path>
- Suggested artifact root:
  <artifact_root>`/Users/macmain/MisterSmith/docs/plans/artifacts/packet-020-live-evaluation/`</artifact_root>
- Base URL when using HTTP surfaces:
  <base_url>`http://127.0.0.1:8080`</base_url>

If the session date differs from the handoff date, keep the artifact slug but update the date
prefix in `<evidence_note_path>` and `<artifact_root>` to match the actual evaluation date.

Before running anything, read:

1. `/Users/macmain/MisterSmith/AGENTS.md`
2. `/Users/macmain/MisterSmith/WORKFLOW.md`
3. `/Users/macmain/MisterSmith/docs/linear/LINEAR.md`
4. `/Users/macmain/MisterSmith/docs/current-state.md`
5. `/Users/macmain/MisterSmith/docs/ms_recent_context.md`
6. `/Users/macmain/MisterSmith/docs/plans/2026-03-26-verifier-gated-adaptive-orchestration.md`
7. `/Users/macmain/MisterSmith/docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md`
8. `/Users/macmain/MisterSmith/docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md`
9. `/Users/macmain/MisterSmith/specs/020-verifier-gated-adaptive-orchestration/spec.md`
10. `/Users/macmain/MisterSmith/specs/020-verifier-gated-adaptive-orchestration/quickstart.md`
11. `/Users/macmain/MisterSmith/specs/020-verifier-gated-adaptive-orchestration/tasks.md`

Then ground on the live runtime and packet-020 surfaces before choosing an evaluation procedure:

- `/Users/macmain/MisterSmith/crates/mister-smith-app/src/execution.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-app/src/autonomy.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-app/src/agent_inspection.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-core/src/autonomy.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-core/src/supervision.rs`
- `/Users/macmain/MisterSmith/scripts/live_runtime_proof_smoke.py`

Follow this control-plane posture:

1. fetch the current issue/control-plane state for `MS-103`
2. do **not** reopen `MS-103` or restage `MS-104` through `MS-107` just to run evaluation
3. execute directly in `/Users/macmain/MisterSmith`
4. do not create or use git worktrees
5. do not create a new implementation lane unless a real defect is found and durable follow-up is
   required later

## Objective

By the end of this session, you must:

1. verify the current packet-020 runtime path from code and docs
2. run at least one real live runtime-backed evaluation on the shipped baseline path
3. attempt one bounded packet-020-focused probe run if the current path supports it
4. capture durable evidence rather than terminal output only
5. state clearly what packet `020` proves live, what remains deterministic-only, and what follow-up
   gap exists if the live path does not expose the expected behavior

## Core Constraints

- do **real live runs**, not a mock-only or code-only audit
- keep proof boundaries explicit: observed live evidence versus inference from code
- do not use Linear or Symphony as the primary truth source for runtime behavior
- do not change provider/model selection silently
- do not mistake packet-019 baseline proof for packet-020 proof unless the transcript actually
  shows verifier or repair behavior
- do not widen into implementation, router changes, benchmark claims, or UI work
- leave one durable evidence note in the repo at `<evidence_note_path>`

## Evaluation-Only Boundary

This session is for evaluation, not implementation.

- do not patch code, change runtime semantics, or refresh packet docs just to make the run look
  better
- do not reopen `MS-103` or any child slice unless a real defect is found and a later follow-up
  issue is genuinely needed
- do not create a branch or PR unless you are explicitly asked to fix a discovered defect after the
  evaluation

## Live Evaluation Shape

Use the narrowest honest live procedure supported by current `main`.

At minimum, perform:

1. **one baseline live run** that confirms the current shipped runtime path still works
2. **one bounded packet-020 probe run** that can reveal verifier, clarification, retry, re-plan,
   or orchestration-quality provenance behavior if the current path supports it

You may use `scripts/live_runtime_proof_smoke.py` as the baseline entrypoint if it remains the
narrowest honest live path, but do not treat it as packet-020 proof by itself. If the baseline run
only re-proves the packet-019 path, you must either follow it with a packet-020-focused probe run
or say explicitly that packet `020` remains unproven live.

If the current code or runtime surfaces cannot honestly trigger packet-020 behavior without
unsupported changes, stop and record that boundary clearly instead of forcing a false positive.

## Run Selection Rule

When choosing the live procedure:

1. prefer an existing repo-native smoke harness, CLI path, or HTTP task surface over ad hoc
   evaluation code
2. prefer a task shape that produces real planner and executor behavior plus a plausible weak
   handoff or local-repair opportunity
3. use only current supported knobs, configs, and task wording; do not invent hidden flags or
   synthetic control paths
4. stop after the narrowest honest bounded probe rather than turning the session into an open-ended
   search for a packet-020 event

## Environment Verification

Before running anything, verify and record:

- current branch and worktree state
- local infrastructure required by the chosen live path
- NATS and JetStream availability
- PostgreSQL availability
- current provider auth surface
- current runtime base URL if HTTP surfaces are used
- whether the existing smoke harness or HTTP task path is the right entrypoint for packet-020
  evaluation

If any prerequisite is missing or broken, attempt bounded local recovery if it is reversible. Stop
if the blocker prevents an honest live run.

## Evidence Checklist

You must capture and cite the following where available:

- runtime startup command and environment assumptions
- readiness proof
- submitted task payload(s)
- accepted task or workflow identifier(s)
- task result payload(s)
- autonomy list or autonomy status output
- runtime log lines showing the actual execution path
- current provider and model used by each run
- routing policy and tier metadata when present
- packet-020-specific provenance fields when present, including verifier verdict, repair action,
  clarification count, checkpoint reference, last stable step, failure-context reference, and
  outcome summary

If a packet-020 field is absent, say whether that absence is expected, unexpected, or unclear from
current code.

## Evaluation Questions

Your final evaluation must answer:

1. Did the live run(s) use the current runtime path described by the code?
2. What exact provider/model path did each run use?
3. What did the baseline run prove?
4. Did any run prove packet-020-specific live behavior?
5. Which packet-020 fields or behaviors were directly observed, which were absent, and which were
   only inferred from code?
6. If packet-020 behavior did not appear, what prevented an honest proof?
7. What parts of packet `020` remain deterministic-only on current evidence?
8. Do the observed results match `docs/current-state.md` and the packet-020 closure note?
9. What is the narrowest honest next step if a proof gap or regression remains?

## Durable Artifact Requirement

Write one durable evidence note to `<evidence_note_path>` and store supporting artifacts under
`<artifact_root>`.

That note must include:

- objective
- date and environment used
- files read for grounding
- commands run
- task payloads and identifiers
- logs and operator evidence captured
- what packet `020` proved live
- what packet `020` did not prove live
- what remains deterministic-only
- blockers, mismatches, and recommended next step

## Do Not Claim

Do not claim any of the following unless you directly proved them in this session:

- a new benchmark gain or broader orchestration-quality claim
- broader provider proof beyond the actual run(s)
- packet-020 live proof if you only reproduced the packet-019 baseline
- clarification, retry, or re-plan proof without a transcript that actually shows it
- full production readiness

## Anti-Patterns

Avoid these failure modes:

- turning a baseline-only run into a packet-020 success claim
- reopening closed implementation work because a live probe was inconclusive
- adding one-off instrumentation or code changes just to manufacture a verifier event
- writing only a narrative summary without durable evidence files

## Stop Conditions

Stop and report clearly if:

- required local infrastructure cannot be brought up
- provider auth for the current shipped baseline is unavailable
- the runtime never becomes ready
- the chosen live path cannot submit a real runtime-backed task
- packet-020 behavior cannot be honestly exercised without code changes or unsupported flags

If you stop, leave a durable blocker note instead of a false success claim.

## Final Response Requirements

At the end of the session:

- provide a concise summary
- link to the durable evidence note
- list the live runs attempted
- state the exact provider/model used
- state what packet `020` proved live
- state which packet-020 fields and behaviors were observed, absent, or only inferred
- state what remains deterministic-only or unproven
- state blockers or concrete defects, if any
- state the narrowest honest next step
