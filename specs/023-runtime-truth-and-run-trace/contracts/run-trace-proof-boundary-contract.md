# Contract: Run Trace And Proof Boundary Surface

**Spec**: [../spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md)

## Scaffold note

This contract is a scaffold freeze for later revision.

- It is meant to preserve packet `023` naming and proof-boundary rules now.
- It is not implementation-ready without a later revalidation pass.
- It must be revised once upstream packet work, especially packet `022`, is complete enough to
  depend on safely.

## Design goal

Freeze one shared contract for honest runtime truth and run traces so current and future result
surfaces can say exactly what a run proved and what it did not prove.

This packet does **not** define durable lifecycle or event-history semantics. Packet `022`
remains the owner of those surfaces.

## Canonical mapping

The contract for packet `023` is:

- one run trace root per `workflow_id`
- one trace taxonomy covering graph, branch, node, tool, handoff, repair, retry, fan-out, join,
  and supervision relationships
- one proof-boundary view that distinguishes:
  - orchestration-substrate completion
  - placeholder or simulated step completion
  - grounded tool execution
  - grounded task proof
- one consistent projection story for task, session, autonomy, and operator run-detail surfaces

No other packet in this sequence should redefine these naming and proof-boundary rules once packet
`023` is revised and implemented.

## Canonical wording for current placeholder-boundary runs

When a run completes through the current placeholder step boundary, the rendered truth story should
be able to say:

- `workflow graph executed successfully`
- `semantic completion not yet proven`
- `grounded tool execution: none/minimal`
- `result is orchestration proof, not substantive task proof`

These phrases are frozen as current conservative wording for this packet scaffold.

## Placeholder-step limit

The current `WorkflowStepTool` behavior is a contract constraint for this scaffold:

- it echoes the incoming payload
- it marks `status=completed`
- it marks `execution_boundary=tool_bus`
- it sets `tool_name=workflow.execute_step`

Until that runtime behavior changes and is revalidated, a result that passes through this boundary
must not be described as grounded task proof.

## Proof-status contract

Current proof-status mapping for this packet is:

- packet `019`: bounded supported-path live proof exists
- packet `020`: bounded supported-path live proof exists
- packet `021`: newer supervision-evidence surface is landed and deterministically validated, but
  this scaffold must not treat that as a fresh default-path live proof claim by itself

The contract must preserve this split until newer repo truth explicitly changes it.

## External tracing guidance

OpenTelemetry traces, context propagation, and W3C Trace Context may guide:

- trace-root naming
- relationship vocabulary
- propagation concepts

They may not be used to claim that Mister Smith already emits a complete span tree, complete span
links, or a production-grade tracing export surface today.

## Surface expectations

The same bounded truth story should be renderable from:

- task result and inspect surfaces
- session-result or session-summary surfaces
- autonomy status surfaces
- operator run-detail surfaces

This scaffold does not require a new dashboard or a UI redesign. It only requires one consistent
truth contract for existing and future projections.

## Revalidation gate

Before implementing this contract later:

1. reread `docs/direction.md`
2. reread `docs/current-state.md`
3. reread `docs/research-output/analysis/2026-03-28-dynamic-orchestration-transfer-brief.md`
4. confirm packet `022` is complete enough to anchor lifecycle and history ownership
5. rerun `/speckit.clarify`, `/speckit.plan`, `/speckit.tasks`, and `/speckit.analyze` if repo
   truth moved
