# Mister Smith — Review Triage & Consolidation

You are triaging code review findings from multiple independent review versions run against the same review scope in the Mister Smith multi-agent orchestration framework. Your job is to verify each finding, deduplicate across versions, and produce a single prioritized task list.

<review_versions>
Version 1:

## Findings (Phase 9 implementation focus)
### 1) Budget reservation is never released on failed completions, and reconciliation failures are silently ignored
- **Category**: Bug
- **Severity**: critical
- **Where**: `crates/mister-smith-llm/src/router.rs:260-265, 293-298, 303-311`
- **What**:
  - A reservation is created before request dispatch:
```rust
    Some(enforcer.reserve(root, estimated).await?)
```
  - On success, reconciliation errors are dropped:
```rust
    let _ = enforcer.reconcile(reservation, response.usage.total_tokens).await;
```
  - On provider error path, there is no reconciliation/release at all.
- **Why it matters**: Budget accounting can drift upward permanently (token "leak"), eventually producing false `BudgetExhausted` and violating the reserve/reconcile invariant from FR-022.
> **Suggested task**: Make budget accounting symmetric and fail-safe in ModelRouter
---
### 2) Cascade routing bypasses budget enforcement entirely
- **Category**: Spec violation
- **Severity**: major
- **Where**: `crates/mister-smith-llm/src/router.rs:251-255`
- **What**:
```rust
  if let RoutingPolicy::Cascade(ref policy) = self.routing_policy {
      let policy = policy.clone();
      return self.route_cascade(request, &policy).await;
  }
```
  This early return occurs **before** budget reserve/reconcile logic.
- **Why it matters**: FR-022 requires reserve-before-send/reconcile-after-completion budget behavior for routing; cascade requests currently skip budget controls.
> **Suggested task**: Apply budget reserve/reconcile flow to cascade routing path
---
### 3) UI text coalescing runs unconditionally, not "under backpressure"
- **Category**: Bug
- **Severity**: major
- **Where**: `crates/mister-smith-llm/src/dual_stream.rs:159-172`
- **What**: For `TextDelta`, code always appends to `pending_text` and only flushes at threshold/finish:
```rust
  self.coalesce_count += 1;
  ...
  if self.coalesce_count >= self.config.max_coalesce_count {
      self.flush_pending_text().await;
  }
```
  No fast-path attempts immediate send when channel has capacity.
- **Why it matters**: This changes runtime behavior from streaming deltas to buffered bursts even when there is no pressure, conflicting with spec intent ("coalesce under backpressure") and increasing perceived latency.
> **Suggested task**: Implement opportunistic immediate UI text delivery with backpressure-triggered coalescing
---
### 4) Anthropic streaming tool-call chunks can be correlated to the wrong call id
- **Category**: Bug
- **Severity**: major
- **Where**: `crates/mister-smith-llm/src/providers/anthropic.rs:389-398, 412-423`
- **What**:
  - `ToolUseStart` uses provider `content_block.id` as `call_id`.
  - `input_json_delta` maps `call_id` from numeric `index` to synthetic `tool-{i}`. These two IDs can differ, so deltas may not attach to the started tool call.
- **Why it matters**: Breaks tool call assembly/semantic correctness in streaming pipelines.
> **Suggested task**: Use consistent Anthropic tool-call identity across start and input deltas
---
### 5) `tasks.md` marks multiple test tasks complete, but referenced test files do not exist
- **Category**: Convention violation (docs/implementation consistency)
- **Severity**: minor
- **Where**: `specs/009-phase9-llm-provider-integration/tasks.md:138-147, 205-206`
- **What**: Tasks claim completed coverage in specific files (`tests/model_event_tests.rs`, `tests/dual_stream_tests.rs`, `mister-smith-transport/tests/...`, `agents/tests/role_tests.rs`) that are absent in repository layout; coverage appears embedded elsewhere instead.
- **Why it matters**: Misleading completion tracking makes audit/review harder and risks hidden test gaps despite "all tasks complete".
> **Suggested task**: Reconcile Phase 9 task checklist paths with actual test locations
---
### Checks run
✅ `pwd; rg --files | rg '^(AGENTS\\.md|CLAUDE\\.md|ROADMAP\\.md|specs/009-phase9-llm-provider-integration/|crates/mister-smith-(llm|agents|transport|core)/|Cargo.toml$)'`
✅ `sed -n '1,220p' CLAUDE.md; echo '---'; sed -n '1,260p' ROADMAP.md; echo '---'; sed -n '1,260p' specs/009-phase9-llm-provider-integration/spec.md`
✅ `git log --oneline --reverse 075813a^..9e27ce0; echo '---'; git diff --name-only 075813a^..9e27ce0`
✅ `git diff --name-only 075813a^..2f782ce`
✅ `rg --files -g '**/AGENTS.md'`
✅ `nl -ba ...` / `rg -n ...` static inspections over targeted files in `mister-smith-llm`, `mister-smith-agents`, `mister-smith-core`, `mister-smith-transport`, and `specs/009-phase9-llm-provider-integration`

Version 2:

## Findings (Phase 9 implementation review)
### 1) Spec violation / missing implementation (major): `RoutingHint` contract is not implemented end-to-end
- **Where**:
  - `crates/mister-smith-llm/src/types.rs:7-28` (no `routing_hint` field on `CompletionRequest`)
  - `crates/mister-smith-llm/src/router.rs:179` (`select_provider(&self, _hint: Option<&RoutingHint>)` ignores hint)
  - `crates/mister-smith-llm/src/router.rs:267` (`select_provider(None)` always called)
- **What**: The code declares `RoutingHint` type, but request types and routing path never carry/use it.
- **Why it matters**: This breaks the Phase 9 contract that callers can request tier/cost/capability preferences via `CompletionRequest.routing_hint` and have router consume/strip it.
> **Suggested task**: Implement `CompletionRequest.routing_hint` and enforce it in provider selection
---
### 2) Bug (major): budget reservation is leaked on failed completion paths
- **Where**: `crates/mister-smith-llm/src/router.rs:259-312`
- **What**: Budget is reserved before provider selection/call (`reserve(...)`), but reconciliation only runs on success (`Ok(response)`); on `Err(err)` branch no release occurs.
- **Why it matters**: Failed requests permanently consume token budget, causing false `BudgetExhausted` and routing degradation over time.
> **Suggested task**: Reconcile or release budget reservations for failed/aborted requests
---
### 3) Spec violation (major): cascade routing bypasses budget enforcement
- **Where**:
  - `crates/mister-smith-llm/src/router.rs:252-255` — delegates to `route_cascade` before budget logic
  - `crates/mister-smith-llm/src/router.rs:320-412` — `route_cascade()` has no reserve/reconcile calls
- **What**: Non-cascade path applies budget checks; cascade path does not.
- **Why it matters**: Violates "router selects provider based on policy, health, and budget constraints" and allows unrestricted spend under `RoutingPolicy::Cascade`.
> **Suggested task**: Apply budget reserve/reconcile semantics in `route_cascade`
---
### 4) Bug (major): `DualStreamActor` loses tool name on completion events
- **Where**: `crates/mister-smith-llm/src/dual_stream.rs:116-120`
- **What**: `ModelEvent::ToolCallCompleted` is emitted with `name: String::new()` even though start event has the real name.
- **Why it matters**: Breaks lossless semantic stream guarantees for tool-call boundaries; downstream execution/audit cannot identify which tool completed.
> **Suggested task**: Preserve tool name through ToolUseStart → ToolCallCompleted assembly
---
### 5) Bug (major): Anthropic streaming emits inconsistent tool call IDs across chunks
- **Where**:
  - Start: `crates/mister-smith-llm/src/providers/anthropic.rs:412-423` — uses `content_block.id`
  - Input delta: `crates/mister-smith-llm/src/providers/anthropic.rs:389-399` — synthesizes `tool-{index}`
- **What**: `ToolUseStart` and `ToolUseInput` for the same tool use can have different `call_id` values.
- **Why it matters**: Tool-call assembly in stream actor cannot correlate deltas with starts; semantic loss/corruption of tool call data.
> **Suggested task**: Use a single stable Anthropic tool call identifier for start and delta events
---
### 6) Missing implementation / spec violation (major): Agent LLM bridge does not consume `ModelEvent` streams and hides failures
- **Where**:
  - `crates/mister-smith-agents/src/roles/planner.rs:117-150`
  - `crates/mister-smith-agents/src/roles/critic.rs:119-166`
  - `crates/mister-smith-agents/src/roles/executor.rs:126-164`
- **What**: Roles call only `route_completion` (non-stream), and if LLM call fails they silently fall through to deterministic stub output.
- **Why it matters**: Contract requires ModelEvent-driven bridge behavior (semantic stream correctness), and fallback-on-error masks routing/provider problems.
> **Suggested task**: Implement ModelEvent-based role bridge and stop silently swallowing LLM failures
---
### 7) Spec violation (minor): "all tasks complete" is inconsistent with repository state (Anthropic test tracking)
- **Where**:
  - `specs/009-phase9-llm-provider-integration/tasks.md` marks Anthropic test task complete (`T011`)
  - `crates/mister-smith-llm/tests/integration.rs:1-2` only includes OpenAI tests
  - `crates/mister-smith-llm/tests/integration/` has no `anthropic_tests.rs`
- **What**: Task ledger claims completion of Anthropic integration coverage, but corresponding test file/module is missing.
- **Why it matters**: Reduces trust in completion tracking and leaves provider parity under-validated.
> **Suggested task**: Align Phase 9 task ledger with actual Anthropic test coverage
---
### Checks performed (static only)
- `pwd && rg --files -g 'AGENTS.md'`
- `git log --oneline --no-decorate 075813a^..9e27ce0`
- `git diff --name-only 075813a^..9e27ce0`
- `rg --files crates/mister-smith-llm/tests | sort`
- `rg -n "ModelEvent::RoutingDecision|RoutingDecision \\{" crates/mister-smith-llm/src crates/mister-smith-agents/src`
- `rg -n "to_tool_definitions|execute_tool_call|invoke\\(" crates/mister-smith-agents/src/tool_bus.rs`
- `rg -n "dual-stream|ModelEvent|fallback|must|router|PlanGoal|Critic|Executor|tool" specs/009-phase9-llm-provider-integration/...`
⚠️ No runtime/test commands were executed per the read-only QA constraints in this task.

Version 3:
## Review Findings (075813a..2f782ce primary scope)
### 1) Cascade routing bypasses budget reserve/reconcile entirely
**Type:** Bug + Spec violation  
**Severity:** critical  
**Location:** `crates/mister-smith-llm/src/router.rs:251-255`, `257-265`, `292-299`, `320-412`
`route_completion()` returns early for `RoutingPolicy::Cascade`:
```rust
if let RoutingPolicy::Cascade(ref policy) = self.routing_policy {
    let policy = policy.clone();
    return self.route_cascade(request, &policy).await;
}
```
Budget reserve/reconcile logic exists only in the non-cascade path (`reserve` at lines 260-263, reconcile at 293-299).
> **Suggested task**: Apply budget reserve/reconcile to cascade routing path
---
### 2) `CompletionRequest` is missing `routing_hint`, and router ignores hints
**Type:** Missing implementation + Spec violation  
**Severity:** major  
**Location:** `crates/mister-smith-llm/src/types.rs:7-28`, `crates/mister-smith-llm/src/router.rs:179`, `202-217`
Contract says `CompletionRequest` gains `routing_hint: Option<RoutingHint>`, but struct has no such field. Router selection signature exists but discards `_hint`.
```rust
async fn select_provider(&self, _hint: Option<&RoutingHint>) -> Result<usize, LlmError>
```
> **Suggested task**: Implement request-level routing hints and use them in provider selection
---
### 3) Cascade policy tiers are not actually used to choose providers
**Type:** Bug + Spec violation  
**Severity:** major  
**Location:** `crates/mister-smith-llm/src/router.rs:317-355`
Code says "attempts providers in registration order (cheapest first)" and indexes directly into `providers[tier_idx]`. `CascadeTier.provider_config` is not used to look up providers.
**Impact**: configured cascade tiers can drift from runtime provider order and route incorrectly (wrong model for the tier).
> **Suggested task**: Bind cascade tiers to concrete providers instead of relying on registration index
---
### 4) Required routing observability event (`ModelEvent::RoutingDecision`) is never emitted
**Type:** Missing implementation  
**Severity:** major  
**Location:** `crates/mister-smith-llm/src/model_event.rs:88-92`, `crates/mister-smith-llm/src/router.rs` (no emit site)
`ModelEvent::RoutingDecision` exists, and tasks/spec require per-attempt cascade routing decision logging, but router returns `RoutingDecision` struct without ever emitting a `ModelEvent::RoutingDecision`.
> **Suggested task**: Emit ModelEvent::RoutingDecision for each cascade attempt
---
### 5) `ToolBus::execute_tool_call()` erases ToolBus error semantics by converting all invoke errors into `Ok(ToolResult::failure)`
**Type:** Spec violation  
**Severity:** major  
**Location:** `crates/mister-smith-agents/src/tool_bus.rs:487-517`
Current behavior:
```rust
Err(err) => Ok(ToolResult::failure(call.call_id.clone(), err.to_string()))
```
Contract requires preserving ToolBus semantics for permission denied / timeout / not found / unavailable separately from execution-level failures.
> **Suggested task**: Preserve typed ToolBus execution errors in execute_tool_call
---
### 6) Dual-stream tool-call completion drops tool name (`name: String::new()`)
**Type:** Bug  
**Severity:** major  
**Location:** `crates/mister-smith-llm/src/dual_stream.rs:44`, `95-97`, `110-120`
`active_tool_calls` stores only accumulated input (`HashMap<String, String>`). On stop, `ToolCallCompleted` is emitted with empty name:
```rust
name: String::new(), // Name was in the Start event
```
This loses semantic completeness for downstream consumers and weakens "lossless tool-call boundaries" contract.
> **Suggested task**: Retain tool name during tool-call assembly in DualStreamActor
---
### 7) Task checklist reports completed work that is not present in the implementation surface
**Type:** Convention violation + Missing implementation tracking  
**Severity:** minor  
**Location:** `specs/009-phase9-llm-provider-integration/tasks.md:202-205`, `142-143`, `108-111`
Examples marked complete but missing in reviewed scope:
- T020 claims `crates/mister-smith-agents/src/orchestrator.rs` orchestrator updates; was not modified in the implementation commits.
- T021 claims `tests/role_tests.rs`; file does not exist in `crates/mister-smith-agents/tests/`.
- T046 claims `crates/mister-smith-llm/tests/dual_stream_tests.rs`; file does not exist.
- T011 claims Anthropic integration test file; missing from `crates/mister-smith-llm/tests/integration/`.
> **Suggested task**: Reconcile Phase 9 tasks checklist with actual implementation artifacts
---
### Checks Performed
- `cd /workspace/MisterSmith && rg --files -g 'AGENTS.md' -g 'CLAUDE.md' -g 'ROADMAP.md'`
- `cd /workspace/MisterSmith && sed -n '1,220p' AGENTS.md && sed -n '1,220p' CLAUDE.md && sed -n '1,260p' ROADMAP.md && sed -n '1,260p' specs/...`
- `cd /workspace/MisterSmith && git log --oneline --no-decorate --reverse 075813a^..9e27ce0 && git diff --name-only ...`
- ✅ inspections across targeted Rust/spec files (`budget.rs`, `router.rs`, `dual_stream.rs`, `tool_bus.rs`, role files, `types.rs`, `health.rs`, contracts/spec/tasks) via `nl -ba ...` / `sed`
No code execution, tests, installs, or file modifications were performed (per read-only QA scope).

Version 4:
## Primary Findings (075813a..2f782ce)
### 1) Budget enforcement is skipped entirely for cascade routing
- **Type**: Bug / Spec violation
- **Severity**: major
- **Location**: `crates/mister-smith-llm/src/router.rs:251-255`, `320-412`
- **What**: `route_completion()` returns early into `route_cascade()` before any reserve/reconcile logic:
```rust
  if let RoutingPolicy::Cascade(ref policy) = self.routing_policy {
      let policy = policy.clone();
      return self.route_cascade(request, &policy).await;
  }
```
  Budget reservation/reconciliation exists only in the non-cascade path (`257-299`).
- **Why it matters**: Spec requires router-level budget checks as part of data-plane routing, including reserve-before-send and reconcile-after-completion for all paths.
> **Suggested task**: Apply budget reserve/reconcile in cascade routing path
---
### 2) Failed completions leak reserved budget; reconciliation errors are silently dropped
- **Type**: Bug
- **Severity**: major
- **Location**: `crates/mister-smith-llm/src/router.rs:293-299`, `303-311`; `crates/mister-smith-llm/src/budget.rs:177-196`
- **What**:
  - On success, reconcile errors are ignored:
```rust
    let _ = enforcer.reconcile(reservation, response.usage.total_tokens).await;
```
  - On provider error, no reconcile/release is attempted before returning `Err(err)`.
- **Why it matters**: Over time, failed/errored requests can permanently consume reserved tokens, causing premature `BudgetExhausted`.
> **Suggested task**: Guarantee reservation cleanup for both success and error paths
---
### 3) `CompletionRequest.routing_hint` from contract is missing, and router ignores hints
- **Type**: Missing implementation / Spec violation
- **Severity**: major
- **Location**: `crates/mister-smith-llm/src/types.rs:7-28`, `crates/mister-smith-llm/src/router.rs:179`
- **What**:
  - `CompletionRequest` has no `routing_hint` field.
  - `select_provider` explicitly discards hints:
```rust
    async fn select_provider(&self, _hint: Option<&RoutingHint>) -> Result<usize, LlmError>
```
- **Why it matters**: Contract requires caller-provided routing hints and router consumption/stripping. This is marked complete but not implemented.
> **Suggested task**: Implement routing_hint end-to-end on CompletionRequest and ModelRouter
---
### 4) Anthropic streaming tool-call assembly can break due to inconsistent call IDs
- **Type**: Bug
- **Severity**: major
- **Location**: `crates/mister-smith-llm/src/providers/anthropic.rs:412-423`, `387-399`
- **What**:
  - Tool start uses provider `content_block.id`:
```rust
    delta: ChunkDelta::ToolUseStart { call_id, name }
```
  - Tool input delta fabricates `tool-{index}`:
```rust
    let call_id = event.get("index").and_then(|v| v.as_u64()).map(|i| format!("tool-{i}"))
```
- **Why it matters**: `DualStreamActor` assembles tool inputs keyed by `call_id`; mismatched IDs split start/input events and can produce malformed tool calls.
> **Suggested task**: Use stable Anthropic tool_use IDs across start and input deltas
---
### 5) Anthropic stream may omit terminal stop chunk
- **Type**: Spec violation / Missing implementation
- **Severity**: major
- **Location**: `crates/mister-smith-llm/src/providers/anthropic.rs:428-448`
- **What**:
  - Stop chunk is emitted only when `message_delta.delta.stop_reason` exists.
  - `message_stop` branch contains only comments and emits nothing.
- **Why it matters**: Contract requires stream termination semantics; consumers may wait indefinitely for terminal state if `message_delta` arrives without a stop reason.
> **Suggested task**: Guarantee terminal stop emission on Anthropic stream completion
---
### 6) Planner/Critic/Executor silently fall back to stub behavior when LLM call fails
- **Type**: Convention violation / Spec risk
- **Severity**: major
- **Location**:
  - `crates/mister-smith-agents/src/roles/planner.rs:145-150`
  - `crates/mister-smith-agents/src/roles/critic.rs:162-166`
  - `crates/mister-smith-agents/src/roles/executor.rs:160-164`
- **What**: In all three roles, router errors are swallowed and execution falls back to deterministic stub output.
- **Why it matters**: This masks provider/routing failures and can produce false-positive Gate 9 behavior instead of surfacing real errors.
> **Suggested task**: Stop masking LLM failures in role handlers when router is configured
---
## Secondary Consistency Note (docs/tasks vs implementation)
### 7) Tasks claim complete items that are not implemented in code
- **Type**: Convention violation
- **Severity**: minor
- **Location**: `specs/009-phase9-llm-provider-integration/tasks.md:89-105, 240-245` vs code paths above
- **What**: Task list marks routing/budget/cascade observability items as complete, but key behavior is still missing or broken.
- **Why it matters**: Inaccurate completion state undermines handoff reliability and review confidence.
> **Suggested task**: Reconcile Phase 9 task checklist with actual implementation status
---
### Checks Performed (static inspection only)
✅ `cd /workspace/MisterSmith && rg --files | rg 'AGENTS.md|CLAUDE.md|ROADMAP.md|specs/009-phase9-llm-provider-integration/...'`
✅ `cd /workspace/MisterSmith && sed -n '1,220p' CLAUDE.md && sed -n '1,240p' ROADMAP.md && sed -n '1,260p' specs/...`
✅ `cd /workspace/MisterSmith && git show --name-only --pretty=format: 075813a..2f782ce | rg '\\S'`
✅ `cd /workspace/MisterSmith && nl -ba crates/mister-smith-llm/src/router.rs | sed -n '1,520p'`
✅ `cd /workspace/MisterSmith && nl -ba crates/mister-smith-llm/src/budget.rs | sed -n '1,260p'`
✅ `cd /workspace/MisterSmith && nl -ba crates/mister-smith-llm/src/providers/anthropic.rs | sed -n '1,4...'`
✅ `cd /workspace/MisterSmith && nl -ba crates/mister-smith-agents/src/tool_bus.rs | sed -n '240,560p'`
✅ `cd /workspace/MisterSmith && nl -ba crates/mister-smith-agents/src/roles/planner.rs | sed -n '90,220p'`
✅ `cd /workspace/MisterSmith && nl -ba crates/mister-smith-transport/src/envelope.rs | sed -n '1,360p'`
✅ `cd /workspace/MisterSmith && nl -ba crates/mister-smith-llm/tests/budget_tests.rs | sed -n '1,260p'`
✅ `cd /workspace/MisterSmith && rg -n "routing_hint|ModelEvent::RoutingDecision|reserve-before-send|reconcile..." specs/ crates/`

</review_versions>

<original_prompt>
# Mister Smith — Post-Implementation Review

You are reviewing code in the Mister Smith multi-agent orchestration framework — a Rust + NATS + supervision tree system built across 19 workspace crates. Your job is to find what's wrong, missing, incomplete, or suboptimal. You are not here to confirm that things work.

<review_scope>
⏺ Commits 075813a through 9e27ce0 on main (6 commits total) comprising the complete Phase 9 LLM Provider Integration:                                                         
                                                                                                                                                                              
  075813a feat: add llm provider foundation and OpenAI auth flows                                                                                                          
  fe951e4 feat: add Claude subscription provider with OAuth credential auth                                                                                                   
  2f782ce feat(llm): complete Phase 9 LLM Provider Integration
  8f22d7e docs: add Phase 9 research corpus and discovery prompts                                                                                                             
  0b6e618 docs: add Phase 9.1 security hardening spec set                                                                                                                  
  9e27ce0 chore: add agent workflows, prompt-improver skill, and spec tooling

  Primary review target is 075813a..2f782ce (the 3 implementation commits). The doc/tooling commits (8f22d7e, 0b6e618, 9e27ce0) are secondary — review only for consistency
  with the implementation.

  Crates modified:
  - mister-smith-llm — new modules: router.rs, health.rs, budget.rs, dual_stream.rs, model_event.rs, providers/anthropic.rs; modified: lib.rs, providers/mod.rs
  - mister-smith-agents — modified: roles/planner.rs, roles/critic.rs, roles/executor.rs, tool_bus.rs, errors.rs, Cargo.toml; new test files: tests/gate9_tests.rs,
  tests/tool_bus_llm_tests.rs
  - mister-smith-transport — modified: envelope.rs, lib.rs
  - mister-smith-core — modified: error.rs

  specs/009-phase9-llm-provider-integration/spec.md — primary governing spec

  Supporting contracts:
  - specs/009-phase9-llm-provider-integration/contracts/model-provider.md
  - specs/009-phase9-llm-provider-integration/contracts/agent-llm-bridge.md
  - specs/009-phase9-llm-provider-integration/contracts/tool-calling-bridge.md
  - specs/009-phase9-llm-provider-integration/data-model.md
  - specs/009-phase9-llm-provider-integration/tasks.md (all 44 tasks marked complete — verify)

  Implementation was performed across multiple agent sessions with handoffs. This increases risk of:
  - Inconsistent patterns between modules written by different sessions
  - Partial implementations where one session assumed another would finish something
  - Convention drift (early modules may not match conventions established in later modules)

  Known trade-offs made during implementation:
  - BudgetStore uses InMemoryBudgetStore only — JetStream KV CAS backing was spec'd but deferred
  - AnthropicProvider streaming uses string-based SSE parsing rather than a typed SSE library
  - DualStreamActor text coalescing had a capacity-based early-flush bug that was patched — verify the fix is correct and complete
  - Circuit breaker tests required error_rate_threshold: 1.1 to isolate consecutive-failure testing from error-rate triggering — this may indicate the two triggers interact
  in unexpected ways
  - #[allow(clippy::too_many_arguments)] was added to 4 pre-existing Phase 7 functions in tool_bus.rs — legitimate suppression, not a Phase 9 concern
  - Cascade routing ConfidenceSignal scoring is heuristic (response length + stop reason) — verify the heuristic is reasonable

  Areas of specific concern:
  - Budget CAS reserve/reconcile: is the pattern actually atomic or is there a TOCTOU window?
  - route_cascade() attempts providers in registration order as a proxy for "cheapest first" — is this assumption documented and enforced?
  - Agent role integration (planner/critic/executor): three files with the same pattern written by the same agent — check for copy-paste bugs where role-specific logic should
   differ
  - ToolBus execute_tool_call() delegates to invoke() — verify it passes through ALL security/audit/timeout boundaries, not just some
</review_scope>

---

## Phase 1: Orient

Read and internalize the project context before reviewing any implementation code:

1. **`CLAUDE.md`** — workspace structure, crate dependency tree, technology stack, conventions
2. **`ROADMAP.md`** — 9-phase build order, dependency flow, gate criteria
3. The **governing spec** for the review scope — from `<governing_spec>`, or locate it from `specs/` (implementation specs) and `spec/` (architecture specs)

From these, establish: which crates are affected, what the spec says the implementation should do, what cross-crate boundaries are involved, and what conventions the surrounding code follows.

---

## Phase 2: Scope

Identify everything touched by the review scope and its immediate dependency surface.

- **Files and modules**: Every source file modified or added within the review scope
- **Dependency path**: If a type was added/changed in `mister-smith-core`, trace every crate that imports it
- **Feature gate matrix**: All `#[cfg(feature = "...")]` code paths — both enabled and disabled
- **Public API surface**: New exports, changed signatures, new trait implementations, new re-exports
- **Cargo.toml changes**: New dependencies, version constraints, feature flag definitions, optional dependency declarations
- **Tests**: Unit tests (`mod tests`), integration tests (`tests/`), cross-crate tests (`mister-smith-integration-tests`)

Follow the dependency path to its natural boundary but don't expand scope infinitely — if you discover issues in adjacent code that's outside the review scope, note them as observations rather than primary findings.

Build a complete inventory. If you are uncertain whether a file is relevant, read it.

---

## Phase 3: Analyze

Read every file identified in Phase 2. Annotate findings with `file:line` references as you go — do not defer citation to synthesis.

The dimensions below are analytical lenses, not a checklist. Apply the ones relevant to the review scope. Err on the side of checking rather than skipping.

### Correctness & Spec Compliance
Does the implementation match the governing spec? Check bidirectionally:
- **Spec → Code**: Are all spec requirements implemented? What's missing?
- **Code → Spec**: Is there code that isn't in the spec? Unauthorized additions, scope creep, speculative implementations?
- Logic errors, incorrect state transitions, broken invariants, off-by-one errors
- State machine correctness: transition logic, terminal states, recovery paths
- Concurrent code: TOCTOU races, deadlocks, lock ordering violations, unsound `unsafe`

### Contract & API Compliance
- Trait implementations: do they satisfy the full behavioral contract, not just compile?
- Trait object safety where `dyn Trait` is used
- `From`/`Into` semantic preservation, `Default` consistency with documented behavior
- Public API boundaries: no internal types leaking, re-exports clean and intentional
- Feature flag composition across the dependency tree

### Error Handling
- Are error variants used correctly across crate boundaries?
- Error swallowing: `unwrap()`, silent `Ok(())` on failure paths, information-losing conversions
- `From` conversions preserving sufficient debug context
- Orphan rule compliance for foreign-type error conversions (free functions vs `From` impl)

### Backward Compatibility & Serialization
- Serialization changes: `#[serde(default)]` on new `Option<T>` fields, `#[non_exhaustive]` where appropriate
- Existing consumers compile without modification
- Enum variants are additive (no renames or removals)
- MessagePack and JSON round-trip preservation

### Concurrency & Async
- `Arc`/`Mutex`/`RwLock`/atomics appropriate to use case (`std::sync` vs `parking_lot` vs `tokio::sync`)
- No locks held across `.await` points
- Channel capacities justified (unbounded channels flagged)
- `Send + Sync + 'static` bounds correct on types crossing task boundaries

### Completeness & Optimization
- Feature-complete against spec, or pieces missing?
- Performance: unnecessary allocations, algorithmic complexity, redundant clones
- Dead code, unused imports, vestigial scaffolding
- Meaningful simplification opportunities (not cosmetic)

---

## Phase 4: Synthesize

Produce your findings. Every finding must include:

- **The specific file and line** (`crates/mister-smith-foo/src/bar.rs:42`)
- **What you found** — quote the relevant code
- **Why it matters** — the impact: correctness, safety, performance, maintainability
- **Severity**: critical (breaks correctness or safety), major (significant issue), minor (should fix), nit (style/preference)

Categorize findings as:
- **Bug**: produces incorrect behavior
- **Spec violation**: works but doesn't match the spec
- **Missing implementation**: spec requirement not implemented
- **Optimization**: works but meaningfully improvable
- **Convention violation**: works but doesn't match project patterns

---

## Framework Conventions (delta from CLAUDE.md)

CLAUDE.md documents the full project structure and tech stack. These are the implementation-level conventions not captured there that inform review judgment:

- **Error placement**: Domain errors (`SecurityError`, `PersistenceError`, `LlmError`) are defined in `mister-smith-core/src/error.rs` and re-exported by domain crates — not defined in the domain crate itself
- **Orphan rule workaround**: Foreign-type error conversions use `from_X_error()` free functions, not `From` trait impls
- **Config extension**: `RuntimeConfigExt` trait pattern — domain crates extend the base config without modifying `mister-smith-config`
- **Sync primitives by use case**: `std::sync` for non-async paths, `parking_lot::RwLock` for hot-path reads, `DashMap` for concurrent KV, `ArcSwap` for hot-reload, `AtomicU8` for lock-free probes
- **Serialization**: Dual JSON + MessagePack support via `serde`. Enums use `#[serde(rename_all = "snake_case")]`. `MessageEnvelope` is `#[non_exhaustive]`.
- **Feature-gated tests**: Test files use `#![cfg(feature = "...")]` at crate level. Env-gated tests use `#[ignore]` + runtime env var checks.

---

## Secondary: Automated Validation

CI and automated tooling handle these. Run only to verify a specific concern from your analysis.

- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `cargo build --workspace` (feature matrix)

</original_prompt>

---

## Phase 1: Inventory

Read every finding across all versions. For each finding, record:
- Version and issue number (e.g., 1.3 = Version 1, issue 3)
- The claimed severity and category
- The specific file:line cited
- A one-line summary of the claim

Do not evaluate yet — just catalog.

---

## Phase 2: Verify

For each finding, read the actual source code at the cited location and determine:

### Legitimacy
- **Is the code actually there?** Does the file:line reference match what the reviewer claims? Reviewers hallucinate line numbers and code quotes — verify.
- **Is the analysis correct?** Does the code actually behave the way the reviewer says it does? Trace the logic yourself.
- **Is the severity accurate?** A reviewer may call something "critical" that is actually minor, or miss that a "minor" issue is actually a correctness bug.

### Intentionality
- **Was this deliberate?** Check `<additional_context>` for known trade-offs. Check git commit messages, code comments, and spec documents for design decisions that explain the behavior.
- **Is the spec actually violated?** Read the governing spec yourself — don't trust the reviewer's characterization of what the spec says.
- **Is this deferred work, not missing work?** Some gaps are intentional scope boundaries, not bugs.

Mark each finding as:
- **Valid** — legitimate issue, should be fixed
- **Valid but intentional** — real gap but documented/deliberate trade-off; note why
- **Invalid** — wrong analysis, hallucinated code, or mischaracterized behavior
- **Downgrade/Upgrade** — correct finding but wrong severity; note the corrected severity

---

## Phase 3: Deduplicate

Group findings that describe the same underlying issue across versions. For each duplicate cluster:

- List all version.issue references (e.g., 1.2, 3.5, 4.1)
- Identify which version's report is strongest based on:
  - **Accuracy**: Does it cite the correct lines and quote actual code?
  - **Completeness**: Does it explain the full impact, not just the symptom?
  - **Actionability**: Does the suggested fix actually address the root cause?
  - **Context**: Does it reference the spec or design decision that's violated?
- Select the best version for each duplicate cluster

Also identify **gaps** — issues found by only one version that other versions missed. These are often the most interesting findings (either highly insightful or false positives — verify carefully).

---

## Phase 4: Prioritize

Produce the final ordered task list. Order by:

1. **Critical bugs** — correctness or safety issues
2. **Spec violations** — code doesn't match what the spec says it should do
3. **Missing implementation** — spec requirements not implemented
4. **Major issues** — significant but not correctness-breaking
5. **Minor issues and nits** — lowest priority

Within each priority band, order by blast radius (issues affecting more code paths or crates first).

---

## Output

Produce a single ordered task list. Each entry:

```
[priority]. [V.I] — [one-line description]
  Severity: [critical/major/minor/nit] (corrected if reviewer was wrong)
  Category: [bug/spec-violation/missing/optimization/convention]
  Location: [file:line]
  Also found in: [other V.I references, or "unique"]
  Why this version: [brief justification if duplicated — e.g., "V1 cites exact lines and root cause; V3 only describes symptom"]
  Notes: [any context on intentionality, trade-offs, or verification findings]
```

After the task list, include:

- **Rejected findings**: Issues marked invalid with brief explanation of why
- **Intentional gaps**: Issues marked valid-but-intentional with the justifying decision/trade-off
- **Cross-version observations**: Patterns in what versions found vs missed — useful for calibrating future review runs
