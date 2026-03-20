# Complex Multi-Agent Proof And Unified Result Surfaces Evaluation

Date: March 19, 2026
Updated: March 20, 2026
Status: Complete
Packet: `specs/015-complex-multi-agent-proof-and-unified-result-surfaces/`
Tasks: full packet closure baseline through `T001` to `T032`

## Objective

Freeze the packet-015 proof-outcome matrix and record the final cross-crate validation and bounded
MCP non-regression evidence required to close the packet.

## Landed Baseline

The final validation pass ran against `ad209f1` on top of the landed packet sequence:

- `e35b0da` `MS-87: Freeze shared result contract (#216)`
- `914a035` `MS-88: freeze proof outcome matrix (#218)`
- `787c0b4` `MS-89: harder-workload graph proof on the default path (#222)`
- `ab4a05c` `MS-90: unify result contract across task and session views (#221)`
- `f7aa2a8` `MS-91: extend operator preview and provenance (#220)`
- `517f8b8` `MS-92: extend proof outcome coverage (#219)`
- `ad209f1` `MS-93: persist proof outcome across runtime and views (#223)`

This means the packet closure lane validated the landed implementation baseline rather than
reopening packet scope with new feature work.

Supporting runtime-capture artifacts from the final packet-015 live evaluation are recorded under:

- `docs/plans/artifacts/2026-03-20-packet-015-live-runtime-evaluation/`

## Frozen Proof Matrix

Packet 015 remains frozen to these three proof-outcome classes:

- `graph_formed_and_completed`
  - use when a real graph formed and the workflow reached terminal completion
- `collapsed_to_sequential`
  - use when the workflow completed but the planner collapsed the workload to a trivial sequential
    path
- `failed_before_graph`
  - use when the run failed or aborted before a usable completed graph outcome existed

Boundary rules preserved by this packet:

1. one shared three-label taxonomy across task, session, and operator result surfaces
2. no fourth packet-level failure label
3. visible collapse remains distinct from both successful graph execution and pre-graph failure

## Final Validation Evidence

### `cargo test -p mister-smith-agents`

Status: passed on March 20, 2026

Relevant packet coverage observed in the passing run:

- harder-workload graph and topology coverage remained green in
  `step_routing_benchmark_tests.rs`, `team_sizing_benchmark_tests.rs`, and `topology_tests.rs`
- packet-015 result-preview and proof-outcome rendering stayed green in
  `gate10_tests.rs`
- bounded external capability decision handling stayed green in
  `tool_bus_tests.rs`

### `cargo test -p mister-smith-events`

Status: passed on March 20, 2026

Relevant packet coverage observed in the passing run:

- `proof_outcome_classification_freezes_the_three_packet_labels`
- `event_bus_aggregates_the_frozen_proof_outcome_matrix`
- `event_bus_assembles_operator_visible_autonomy_projection`
- `operator_result_preview_roundtrips_with_shared_contract_fields`

### `cargo test -p mister-smith-app`

Status: passed on March 20, 2026

Relevant packet coverage observed in the passing run:

- `terminal_result_views_preserve_proof_outcome_across_task_and_final_results`
- `retained_result_for_turn_uses_stored_projection_with_proof_outcome`
- `classify_proof_outcome_covers_success_collapse_and_failure_visible_matrix`
- `recover_persisted_autonomy_status_preserves_allowed_external_capability_decision_snapshot`
- `recover_persisted_autonomy_status_preserves_rejected_external_capability_decision_snapshot`

### `cargo build --workspace`

Status: passed on March 20, 2026

Result:

- the full Rust workspace still compiled cleanly after the landed packet sequence
- no cross-crate compatibility regression was exposed by the packet-015 result-surface changes

## Bounded MCP Non-Regression Decision

`T032` was required and executed.

Reason:

- the landed packet result surfaces now preserve allowed and rejected external capability decision
  snapshots in `crates/mister-smith-app/src/execution.rs`,
  `crates/mister-smith-app/src/conversation.rs`, and
  `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- those operator-visible result surfaces are downstream of the bounded post-`MS-77` MCP capability
  path, so final validation needed one explicit non-regression check

Executed evidence:

- a live `describe_external_capabilities` probe without a delegation envelope returned the expected
  boundary error: `delegation envelope required for MCP tool 'describe_external_capabilities'`
- `cargo test -p mister-smith-mcp` passed on March 20, 2026, including the bounded capability
  checks:
  - `describe_external_capabilities_requires_discover_delegation`
  - `describe_external_capabilities_returns_catalog_with_matching_discover_delegation`
  - `required_boundary_action_rejects_missing_delegation`
  - `required_boundary_action_accepts_matching_discover_delegation`

Validation boundary:

- this session did not carry a delegated `Discover` envelope, so it did not perform a successful
  live catalog read
- the expected live rejection plus the passing `mister-smith-mcp` test suite is sufficient bounded
  proof that the `MS-77` capability-discovery contract did not regress in this packet

## Result

Packet 015 now has:

- one frozen proof-outcome matrix
- one landed shared result-surface implementation baseline
- passing targeted validation for `mister-smith-agents`, `mister-smith-events`, and
  `mister-smith-app`
- a passing full-workspace build
- an explicit bounded MCP non-regression decision and proof record tied to the post-`MS-77`
  external capability path

No additional packet-scope implementation changes were required after the final validation pass.
