# Contract: Run Trace And Proof Boundary Surface

**Spec**: [../spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md)

## Design goal

Freeze one shared packet-023 contract for honest runtime truth and run traces so task, session,
autonomy, and operator surfaces can say exactly what a run proved and what it did not prove.

Packet `023` does **not** define durable lifecycle or event-history semantics. Packet `022`
remains the owner of those surfaces. Packet `023` also does **not** redefine packet `021`
predictive supervision.

## Canonical mapping

The contract for packet `023` is:

- one `runtime_truth` block per run surface
- one run-trace root per `workflow_id`
- one run-trace taxonomy covering graph, branch, node, tool boundary, handoff, repair, retry,
  fan-out, join, and supervision relationships
- one proof-boundary view that distinguishes:
  - orchestration-substrate completion
  - placeholder or simulated step completion
  - grounded tool execution
  - grounded task proof
- one consistent projection story for task, session, autonomy, and operator run-detail surfaces

## Canonical wording for current placeholder-boundary runs

When a run completes through the current placeholder step boundary, the rendered truth story must be
able to say:

- `workflow graph executed successfully`
- `semantic completion not yet proven`
- `grounded tool execution: none/minimal`
- `result is orchestration proof, not substantive task proof`

These phrases are frozen as packet-023-owned wording for the current first slice.

## Placeholder-step limit

The current `WorkflowStepTool` behavior is a contract constraint:

- it echoes the incoming payload
- it marks `status=completed`
- it marks `execution_boundary=tool_bus`
- it sets `tool_name=workflow.execute_step`

Until the runtime behavior changes and a fresh proof artifact says otherwise, a result that passes
through this boundary must not be described as grounded task proof.

## Proof-status contract

Current proof-status mapping for this packet is:

- packet `019`: bounded supported-path live proof exists
- packet `020`: bounded supported-path live proof exists
- packet `021`: landed on `main` and deterministically validated for predictive-supervision
  evidence, but no new live rerun claim
- packet `022`: landed on `main` and deterministically validated for durable workflow core, but no
  new live rerun claim
- packet `023`: deterministic validation only unless a fresh live rerun is explicitly captured

The contract must preserve this split until newer repo truth explicitly changes it.

## External tracing guidance

OpenTelemetry traces, context propagation, and W3C Trace Context may guide:

- trace-root naming
- relationship vocabulary
- propagation concepts

They may not be used to claim that Mister Smith already emits a complete span tree, complete span
links, or a production tracing export surface today.

## Surface expectations

The same bounded truth story should be renderable from:

- task result and inspect surfaces
- session retained-result and summary surfaces
- autonomy status surfaces
- operator run-detail surfaces

Packet `021` predictive supervision remains a separate adjacent surface and must not be collapsed
into packet `023` runtime truth.
