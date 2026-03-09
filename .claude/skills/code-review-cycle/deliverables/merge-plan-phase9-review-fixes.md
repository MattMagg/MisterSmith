# Merge Plan: Phase 9 Bug Fixes — PRs #129-#132 (MS-2 through MS-5)

**Date**: 2026-03-09
**Reviewer**: Claude Code Agent Team (`ms-pr-review`)
**Status**: All 4 PRs APPROVED — no code changes required

---

## Review Summary

| PR | Issue | Title | Verdict | Agent Changes |
|----|-------|-------|---------|---------------|
| #129 | MS-3 | fix(llm): resume UI text delivery after backpressure | APPROVED | None |
| #130 | MS-4 | fix(llm): stabilize Anthropic streaming tool-call IDs | APPROVED | None |
| #131 | MS-5 | fix(llm): finish routing hint handling and clear fmt gate | APPROVED | None |
| #132 | MS-2 | fix(llm): enforce cascade budgets per attempt | APPROVED | None |

### Review Findings

- **PR #129 (MS-3)**: `route_text_delta()` correctly implements try-first-then-buffer. `pending_text` accumulates only while channel is full; next `TextDelta` with capacity delivers coalesced content immediately. 4 regression tests cover all scenarios.

- **PR #130 (MS-4)**: `tool_call_ids_by_content_index: HashMap<u64, String>` replaces broken `.keys().nth()` iteration-order lookup. `call_id` stored at `content_block_start`, looked up by index for `input_json_delta`, consumed at `content_block_stop`. Regression test exercises sequential multi-tool blocks.

- **PR #131 (MS-5)**: `RoutingHint.preferred_tier` wired through completion, streaming, and cascade paths. `prioritize_preferred_tier()` uses stable `Vec::partition()`. Hint stripped before provider dispatch via `provider_request()`. 6 tests cover hint-guided selection, streaming, cascade, capabilities, and cost filtering.

- **PR #132 (MS-2)**: Per-tier `reserve_budget()` inside cascade loop (line 532). `reconcile(actual)` after successful tier before escalation decision (line 549). `reconcile(0)` on failed tier (line 606). No `release()` needed — reconcile-with-zero correctly nets out. 4 regression tests cover accepted, escalated, multi-tier accounting, and hard-cap rejection.

---

## Conflict Analysis

### Shared Baseline (all 4 PRs)

All 4 PRs branch from the same commit on `main` and share an identical formatting/infrastructure baseline across 11 files:

- `critic.rs`, `executor.rs`, `planner.rs` (rustfmt)
- `gate9_tests.rs`, `tool_bus_llm_tests.rs` (minor test fixes)
- `budget.rs`, `budget_tests.rs` (rustfmt)
- `claude_credentials.rs` (`#[cfg(target_os = "macos")]` gate)
- `model_event.rs`, `anthropic.rs`, `envelope.rs` (rustfmt)

The first merged PR lands these changes. Subsequent rebases resolve these trivially (accept main).

### Divergent Files

| File | PR #129 | PR #130 | PR #131 | PR #132 | Conflict Risk |
|------|---------|---------|---------|---------|--------------|
| `router.rs` | +11/-7 | +11/-7 | **+78/-29** | +26/-22 | **HIGH** — #131 and #132 modify different functions |
| `router_tests.rs` | +66/-36 | +66/-36 | **+181/-43** | **+267/-39** | **MEDIUM** — tests are additive, non-overlapping |
| `dual_stream.rs` | **+174/-96** | +107/-71 | +107/-71 | +107/-71 | LOW — #129 unique; others are baseline |
| `claude_subscription.rs` | +53/-42 | **+157/-65** | +53/-42 | +53/-42 | LOW — #130 unique; others are baseline |
| `claude-code-review.yml` | — | +6/-0 | — | — | NONE — #130 only |

### Dependency Ordering

No PR depends on another — they fix independent bugs in independent code paths. The shared baseline creates file-level conflicts but no semantic dependencies.

---

## Merge Strategy

### Method: Squash Merge (`--squash --delete-branch`)

Each PR has 3 commits (fix + macOS gate + rustfmt baseline). Squash into a single commit per PR for clean history.

### Phase 1: PR #130 (MS-4) — Anthropic tool-call IDs

**Disposition**: Merge as-is (no rebase needed — branches from main)

**Rationale**: Most isolated unique change (`claude_subscription.rs`), plus the only PR with CI workflow fix. Lands the shared formatting baseline for all subsequent PRs.

```bash
gh pr merge 130 --squash --delete-branch
```

**Verify**: `cargo build --workspace && cargo test -p mister-smith-llm`

### Phase 2: PR #129 (MS-3) — UI backpressure resume

**Disposition**: Rebase onto main, resolve baseline conflicts, merge

**Expected conflicts**: 11 common baseline files (accept main), `dual_stream.rs` (accept PR — unique fix), `claude_subscription.rs` (accept main — #130's version), `router.rs`/`router_tests.rs` (accept main for baseline, keep PR's minimal additions)

```bash
gh pr merge 129 --squash --delete-branch
```

**Verify**: `cargo build --workspace && cargo test -p mister-smith-llm`

### Phase 3: PR #132 (MS-2) — Cascade budget enforcement

**Disposition**: Rebase onto main, resolve conflicts, merge

**Expected conflicts**: Common baseline (accept main), `router.rs` (manual merge — PR #132's cascade budget changes are in different functions than those already landed), `router_tests.rs` (combine — tests are additive, non-overlapping)

```bash
gh pr merge 132 --squash --delete-branch
```

**Verify**: `cargo build --workspace && cargo test -p mister-smith-llm`

### Phase 4: PR #131 (MS-5) — RoutingHint end-to-end

**Disposition**: Rebase onto main, resolve conflicts, merge

**Expected conflicts**: Common baseline (accept main), `router.rs` (manual merge — largest change, touches `select_provider_from_entries` and adds `prioritize_preferred_tier()`), `router_tests.rs` (combine — tests are additive)

```bash
gh pr merge 131 --squash --delete-branch
```

**Verify**: `cargo build --workspace && cargo test -p mister-smith-llm`

### Post-Merge Final Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test -p mister-smith-llm
cargo test -p mister-smith-agents
cargo build --workspace
```

---

## Linear Status Transitions

| Issue | Current | After Merge | Mechanism |
|-------|---------|-------------|-----------|
| MS-2 | Human Review | Done | Auto via GitHub integration on PR #132 merge |
| MS-3 | Human Review | Done | Auto via GitHub integration on PR #129 merge |
| MS-4 | Human Review | Done | Auto via GitHub integration on PR #130 merge |
| MS-5 | Human Review | Done | Auto via GitHub integration on PR #131 merge |
| MS-15 | In Progress | Done | Close manually — fixed by PR #130's CI advisory change |

---

## Risk Assessment

**Overall risk**: LOW — all fixes are correct, well-tested, and touch independent code paths.

The main risk is conflict resolution during rebasing phases 2-4. The conflicts are on the shared formatting baseline (trivial) plus `router.rs`/`router_tests.rs` where each PR's unique changes touch different functions.

**Rollback plan**: If a merge introduces test failures, revert the squash commit and investigate before continuing.
