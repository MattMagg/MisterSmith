# Quickstart: Runtime Truth And Run Trace

## Scaffold note

This is a scaffold quickstart for later packet work.

- It is meant to save time later.
- It is not a signal to implement immediately.
- It must be revised once upstream packet work is complete enough to revalidate packet `023`.

## Before implementation

Before any later implementation starts:

```bash
sed -n '1,220p' docs/direction.md
sed -n '1,260p' docs/current-state.md
sed -n '1,260p' docs/packet-prep/023-runtime-truth-and-run-trace.md
```

Then confirm:

- packet `022` is complete enough to rely on for lifecycle and history ownership
- the packet `019` / `020` live-proof baseline versus packet `021` deterministic-only split is
  still current
- the placeholder `workflow.execute_step` truth gap is still described correctly by this scaffold

If any of that changed, rerun:

```bash
./.specify/scripts/bash/check-prerequisites.sh --json --paths-only
./.specify/scripts/bash/setup-plan.sh --json
```

Then refresh the packet by rerunning `/speckit.clarify`, `/speckit.plan`, `/speckit.tasks`, and
`/speckit.analyze`.

## Future deterministic validation

When packet `023` is later revised for implementation, start with the narrowest honest checks for
the touched truth and projection seams:

```bash
cargo test -p mister-smith-core
cargo test -p mister-smith-events --test autonomy_event_tests
cargo test -p mister-smith-app --test autonomy_status_tests
cargo test -p mister-smith-app workflow_step_tool_marks_payload_as_tool_bus_completed
python3 -m unittest scripts.tests.test_live_runtime_proof_smoke
git diff --check
```

## Proof expectation

Later implementation work for packet `023` should earn proof by making it easy to tell, in one
inspection pass:

- whether the run only proved orchestration-substrate completion
- whether the current step boundary remained placeholder or simulated
- whether grounded tool execution exists and what evidence anchors it
- whether packet `019` and `020` live-proof claims stay separate from packet `021`
  deterministic-only proof for newer surfaces

## Live-proof boundary

This scaffold does not create a new live-proof claim.

If packet `023` later earns a live runtime proof, that proof must stay bounded to the exact
provider path and artifact set that were actually exercised and must not imply that:

- packet `022` semantics were already finalized if they were not
- the repo already emits a complete span model
- placeholder step completion became grounded task proof without the runtime actually changing
