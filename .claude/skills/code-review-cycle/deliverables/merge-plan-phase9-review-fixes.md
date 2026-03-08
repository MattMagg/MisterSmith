# Phase 9 Review Fixes — PR Merge Plan

## PR Inventory

| PR | Title | Finding | Files | Assessment |
|----|-------|---------|-------|------------|
| #116 | fix(spec): reconcile phase 9 task completion status | #12 | `tasks.md` | Correct |
| #117 | fix(anthropic): emit fallback stop chunk on message_stop | #11 | `anthropic.rs` | Correct |
| #118 | fix(llm): reconcile budget reservations on all completion paths | #1 | `router.rs`, `budget.rs`, tests | Correct |
| #119 | fix(llm): enforce and reconcile budget for cascade routing | #2 | `router.rs`, `budget.rs`, tests | Partially correct (see notes) |
| #120 | fix(llm): map Anthropic input_json_delta events to real tool call IDs | #3 | `anthropic.rs` | Correct |
| #121 | feat(llm): add routing_hint to CompletionRequest and honor hints | #4 | `types.rs`, `router.rs`, `lib.rs`, tests | Correct |
| #122 | fix(llm): preserve tool metadata across dual-stream tool assembly | #5 | `dual_stream.rs` | Correct |
| #123 | fix(agents): propagate ToolBus boundary errors for LLM tool calls | #6 | `tool_bus.rs`, tests | Correct |
| #124 | fix(llm): enforce cascade tier-to-provider routing order | #7 | `router.rs`, tests | Correct |
| #125 | fix: propagate LLM failures in planner/critic/executor | #10 | `planner.rs`, `critic.rs`, `executor.rs`, tests | Correct |
| #126 | fix: refine dual stream text delta backpressure routing | #9 | `dual_stream.rs` | Correct |
| #127 | feat(router): emit ModelEvent::RoutingDecision for cascade tiers | #8 | `router.rs`, tests | Correct |

## Conflict Map

### `router.rs` — 5-WAY CONFLICT (PRs #118, #119, #121, #124, #127)

| Function | #118 | #119 | #121 | #124 | #127 |
|----------|------|------|------|------|------|
| `route_completion()` | error-path budget fix | move budget before cascade | add hint to select_provider, strip hint | — | — |
| `route_cascade()` | — | add estimated_tokens param | hint filtering, tier reordering | rewrite to use attempt plan | add event emission |
| `select_provider()` | — | — | change signature (add params, filtering) | — | — |
| New types/methods | — | `BudgetReservationContext`, `reserve_budget()` | `provider_supports_capability()`, `provider_matches_hint()`, `provider_request()` | `CascadeAttempt`, `build_cascade_attempt_plan()` | `ModelEventSink`, `emit_routing_event()` |

**Cannot merge independently.** Every rebase after the first produces heavy conflicts in `route_cascade()`.

### `budget.rs` — 2-WAY CONFLICT (PRs #118, #119)

Both add CAS retry loop to `reconcile()`. Nearly identical implementations. Pick one.

### `anthropic.rs` — 2-WAY CONFLICT (PRs #117, #120)

- #120 rewrites the entire streaming loop into `parse_stream_event()` + `AnthropicStreamState`
- #117 adds `terminal_stop_emitted` tracking + fallback emission
- #120 does NOT fix the terminal stop issue (same comment-only `message_stop` branch)

**Must combine.** Merge #120's refactor first, then integrate #117's fallback into the new structure.

### `gate9_tests.rs` — 2-WAY CONFLICT (PRs #123, #125)

- #123 changes `tool_bus_round_trip_with_mock_provider` to use new `execute_tool_call_provider_result`
- #125 adds failing provider tests and formatting changes

**Compatible.** Merge #123 first, then rebase #125.

### `dual_stream.rs` — NO CONFLICT (PRs #122, #126)

- #122 modifies `convert_chunk()` and HashMap type
- #126 modifies `route_event()` and `flush_pending_text()`

Different functions — merge independently.

## Correctness Issues Found

### #119 — `BudgetReservationContext::reconcile()` silently drops errors

```rust
impl<'a> BudgetReservationContext<'a> {
    async fn reconcile(self, actual_tokens: u64) {
        let _ = self.enforcer.reconcile(&self.reservation, actual_tokens).await;
    }
}
```

This re-introduces the `let _` pattern that #118 specifically fixes. During the combine, this must be changed to propagate or log the error.

---

## Merge Strategy

### Phase 1: Zero-conflict independent merges

| Order | PR | Disposition | Rationale |
|-------|----|-------------|-----------|
| 1.1 | #116 | Merge as-is | docs only, no code conflicts |
| 1.2 | #122 | Merge as-is | `dual_stream.rs` `convert_chunk()` — no overlap with other PRs |
| 1.3 | #123 | Merge as-is | `tool_bus.rs` only + tests |
| 1.4 | #126 | Merge as-is | `dual_stream.rs` `route_event()` — different area from #122 |

**Verify after Phase 1:** `cargo build --workspace && cargo test --workspace`

### Phase 2: Role error propagation

| Order | PR | Disposition | Rationale |
|-------|----|-------------|-----------|
| 2.1 | #125 | Merge with rebase | `gate9_tests.rs` conflicts with #123 (Phase 1.3); rebase onto main |

Changes are in `planner.rs`, `critic.rs`, `executor.rs` (no other PR touches these) and `gate9_tests.rs` (needs rebase after #123).

**Verify after Phase 2:** `cargo test -p mister-smith-agents`

### Phase 3: Anthropic streaming (combine #120 + #117)

| Order | PR | Disposition | Rationale |
|-------|----|-------------|-----------|
| 3.1 | #120 | Primary branch | Major refactor — `parse_stream_event()` + `AnthropicStreamState` + ID mapping |
| 3.2 | #117 | Combine into #120 | Integrate `terminal_stop_emitted` into `AnthropicStreamState`, add fallback logic to `parse_stream_event()`'s `message_stop` branch |

Integration plan:
1. Check out #120's branch
2. Add `terminal_stop_emitted: bool` field to `AnthropicStreamState`
3. In `parse_stream_event()` `message_delta` branch: set `state.terminal_stop_emitted = true` when emitting stop chunk
4. In `message_stop` branch: emit `StreamChunk::stop(state.chunk_index, StopReason::Completed)` if `!state.terminal_stop_emitted`
5. Port #117's tests into #120's test module
6. Merge the combined branch, close #117 as superseded

**Verify after Phase 3:** `cargo test -p mister-smith-llm`

### Phase 4: Router megamerge (combine #118 + #119 + #121 + #124 + #127)

These 5 PRs all modify `router.rs` and cannot merge independently. Combine into a single branch.

| Order | PR | Disposition | Role in combination |
|-------|----|-------------|---------------------|
| 4.base | #121 | Primary branch | Largest structural change: `RoutingHint` on `CompletionRequest`, `select_provider` filtering, hint stripping |
| 4.layer | #118 | Combine into #121 | Budget error-path fix, reconcile error propagation (`let _` → `?`), CAS retry in `budget.rs` |
| 4.layer | #119 | Combine into #121 | `reserve_budget()` before cascade early-return, reconcile on cascade path. **Fix**: change `BudgetReservationContext::reconcile()` to propagate errors instead of `let _` |
| 4.layer | #124 | Combine into #121 | `build_cascade_attempt_plan()` — tier-to-provider binding by `model_id` + `provider_kind` |
| 4.layer | #127 | Combine into #121 | `ModelEventSink` trait + `emit_routing_event()` in cascade loop |

Integration order within the combined branch:
1. Start from #121 (types.rs changes, select_provider signature, hint filtering/stripping)
2. Apply #118's budget fixes (error-path reconcile, reconcile error propagation, CAS retry)
3. Apply #119's cascade budget logic (reserve before cascade, but with error propagation fixed)
4. Apply #124's cascade tier binding (route_cascade rewrite with attempt plan)
5. Apply #127's event emission (ModelEventSink, emit in cascade loop)
6. Reconcile all router_tests.rs additions (deduplicate test helpers: FailingProvider, SharedBudgetStore, RecordingSink)

**Budget.rs resolution**: Use #118's `CAS_RETRY_LIMIT` const approach and `FlakyCasStore` test. Drop #119's duplicate `RECONCILE_MAX_RETRIES` + `ConflictOnceStore`.

Close #118, #119, #124, #127 as superseded by the combined branch.

**Verify after Phase 4:** `cargo build --workspace && cargo test --workspace && cargo clippy --workspace -- -D warnings`

---

## Summary

| Phase | PRs | Action | PRs closed as superseded |
|-------|-----|--------|--------------------------|
| 1 | #116, #122, #123, #126 | Merge as-is (4 merges) | — |
| 2 | #125 | Merge with rebase | — |
| 3 | #120 + #117 | Combine, merge | #117 |
| 4 | #121 + #118 + #119 + #124 + #127 | Combine, merge | #118, #119, #124, #127 |

**Total: 12 PRs → 7 merges, 5 closures**
