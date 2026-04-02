# Quickstart: Runtime Truth And Run Trace

## Packet posture

Packet `023` is implementation-ready and deterministic-only by default.

It does not create a new live runtime-proof claim unless a real rerun is explicitly added and
executed.

## Read before coding

```bash
cat AGENTS.md
cat docs/current-state.md
cat docs/direction.md
cat docs/plans/2026-03-29-packet-021-supervision-evidence-proof-boundary.md
cat specs/022-durable-workflow-core/spec.md
cat specs/023-runtime-truth-and-run-trace/spec.md
cat specs/023-runtime-truth-and-run-trace/plan.md
cat specs/023-runtime-truth-and-run-trace/tasks.md
```

## First implementation pass

1. revise any remaining packet-023 truth drift in the packet docs
2. add the new packet-owned `runtime_truth` block in `mister-smith-core`
3. synthesize that block in orchestrator and event-bus state
4. project it through task, session, autonomy, and operator surfaces
5. keep packet `021` predictive supervision separate
6. keep packet `022` lifecycle and history ownership separate

## Guardrails

- do not widen `MessageEnvelope`
- do not fold runtime truth into `supervision_evidence`
- do not claim grounded task proof when the strongest evidence is still the placeholder
  `workflow.execute_step` tool-bus boundary
- do not claim fresh live proof unless a real rerun was actually executed
- do not widen into generic observability, coordinator-runtime, or interoperability work

## Deterministic validation

Run the narrowest honest checks for the touched surfaces:

```bash
cargo test -p mister-smith-core
cargo test -p mister-smith-agents
cargo test -p mister-smith-events --test autonomy_event_tests
cargo test -p mister-smith-app --test autonomy_status_tests
cargo test -p mister-smith-app workflow_step_tool_marks_payload_as_tool_bus_completed
npm --prefix apps/operator-console test
npm --prefix apps/operator-console run build
python3 -m unittest scripts.tests.test_live_runtime_proof_smoke
git diff --check
npx markdownlint-cli2 "specs/023-runtime-truth-and-run-trace/**/*.md" --config .markdownlint.json
```

## Proof expectation

After packet `023`, a reviewer should be able to inspect any supported run surface and tell:

- whether the run only proved orchestration-substrate completion
- whether the strongest evidence was still placeholder or simulated
- whether grounded tool execution exists and what evidence anchors it
- whether predictive supervision is present as a separate packet-021 projection
