# 2026-03-27 MS-110 Ambiguous Prompt Evidence Freeze

## Summary

This note executes Milestone 1 from `MS-110`: run a bounded live matrix of ambiguous prompts on
current `main` to determine whether the runtime still over-shapes non-explicit work into
unnecessary branching.

Result:

- all three ambiguous prompts formed a graph and completed
- all three stayed `Sequential`
- all three kept `parallelism_width = 1` and `branch_count = 1`
- no residual forced parallelism or synthesized merge step was observed

Current recommendation:

- do not open a new implementation slice yet
- keep `MS-110` as a dormant planning/backlog item unless new live evidence shows over-shaping

## Baseline

This evidence freeze starts from the March 27 landed baseline in:

- `docs/plans/2026-03-27-runtime-planning-simplification.md`

That earlier pass already proved:

- simple prompts stay sequential
- explicit parallel prompts still produce a legal coordinator merge
- non-memo prompts preserve requested output shape
- packet-020-style repair probes no longer fail on the old `join` topology trap

The remaining question for `MS-110` was narrower:

- do ambiguous prompts that could plausibly drift into multi-step or multi-branch work still stay
  bounded and sequential by default?

## Deterministic Validation

The narrow deterministic preflight for this evidence freeze passed:

```bash
cargo test -p mister-smith-app
git diff --check
```

These checks support the live evaluation but do not replace runtime proof.

## Live Runs

All three runs used:

- provider: `openai_chatgpt`
- model: `gpt-5.4`
- ingress: `POST /api/v1/tasks`
- autonomy surface: `GET /api/v1/autonomy/status/{workflow_id}`
- harness: `python3 scripts/live_runtime_proof_smoke.py --profile baseline --scenario non_memo --task-description ...`

### Case 1: Trust-First Question

Prompt:

```text
Inspect the live runtime and answer one question: when task status and autonomy status disagree
on emphasis, which surface should an operator trust first and why? Ground the answer only in
directly observed evidence. Use the smallest workflow that can finish the task. Keep the answer
under 140 words.
```

Artifact directory:

- `docs/plans/artifacts/2026-03-27-ms-110-ambiguous-prompt-evidence-freeze/trust-first/20260327T184924Z/`

Observed result:

- `topology_kind = Sequential`
- `parallelism_width = 1`
- `branch_count = 1`
- `node_count = 3`
- `proof_outcome = graph_formed_and_completed`
- normalized plan stayed on one branch:
  - inspect runtime surfaces
  - compare authority signals
  - write constrained answer

### Case 2: Three-Axis Compare

Prompt:

```text
Inspect the live runtime and compare provider/model selection, execution graph shape, and
operator-facing result provenance in one concise answer grounded only in directly observed
evidence. Use whatever workflow is actually necessary and keep the answer under 180 words.
```

Artifact directory:

- `docs/plans/artifacts/2026-03-27-ms-110-ambiguous-prompt-evidence-freeze/three-axis-compare/20260327T185027Z/`

Observed result:

- `topology_kind = Sequential`
- `parallelism_width = 1`
- `branch_count = 1`
- `node_count = 2`
- `proof_outcome = graph_formed_and_completed`
- normalized plan collapsed to a two-step chain:
  - inspect live runtime once
  - compose one evidence-only answer

### Case 3: Readiness Versus Result

Prompt:

```text
Inspect the live runtime and explain whether bootstrap/readiness evidence and terminal result
evidence can be summarized as one bounded operator answer without splitting the work. Ground the
answer only in directly observed runtime evidence and keep it under 160 words.
```

Artifact directory:

- `docs/plans/artifacts/2026-03-27-ms-110-ambiguous-prompt-evidence-freeze/readiness-vs-result/20260327T185140Z/`

Observed result:

- `topology_kind = Sequential`
- `parallelism_width = 1`
- `branch_count = 1`
- `node_count = 2`
- `proof_outcome = graph_formed_and_completed`
- normalized plan again stayed as one bounded chain:
  - inspect runtime evidence
  - compose one bounded operator answer

## Evaluation Result

Milestone 1 stop condition is met.

On current `main`, the supported live path did not show residual over-shaping for these ambiguous
prompts. The current smallest-workflow baseline appears stable:

- no prompt drifted into parallel branches
- no prompt synthesized a merge/coordinator step
- the planner/runtime consistently kept the work on one branch and reduced two of the prompts to
  two-step chains

That means there is not yet evidence strong enough to justify a new code slice in planner policy,
runtime normalization, or topology selection.

## Remaining Limits

- this was a bounded three-prompt evidence freeze, not an exhaustive search over all ambiguous task
  phrasings
- explicit parallel prompts still remain a separate positive-control case and were not rerun here
  because the question in `MS-110` was residual over-shaping on non-explicit prompts
- `MS-110` should remain open only as a dormant planning lane unless future live evidence shows
  renewed drift

## Cleanup

- each run used the existing smoke harness and left a replayable artifact bundle in a dedicated
  sublane
- no code changes were required for this evidence freeze
- the next honest action is tracker-only: update `MS-110` to record that Milestone 1 completed and
  no immediate implementation slice is justified
