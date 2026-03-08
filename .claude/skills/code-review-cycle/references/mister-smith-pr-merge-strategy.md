# Mister Smith — PR Merge Strategy

You are planning and executing the merge of multiple pull requests into `main` for the Mister Smith multi-agent orchestration framework. These PRs originated from a code review cycle — each PR addresses one or more findings from a post-implementation review. The PRs were created by independent coding agents that did not coordinate with each other, so conflicts, overlaps, redundancies, and divergent approaches to the same problem are expected.

Your job is to analyze every PR, determine the optimal merge strategy, and execute it. "Optimal" means: `main` ends up with every legitimate fix applied, no regressions, no redundant changes, and a clean history. The path to get there is your call.

<open_prs>
<!-- List of open PR numbers from the code review fix cycle, or use: gh pr list --state open -->
</open_prs>

<review_triage>
<!-- Optional: the triage report that generated the task list these PRs address. Helps map PRs back to original findings. -->
</review_triage>

<additional_context>
<!-- Optional: known conflicts, PRs that should be prioritized or discarded, constraints on merge order -->
</additional_context>

---

## Phase 1: Inventory

For every open PR:

1. **Read the full diff** — not the summary, the actual code changes. Understand what each PR modifies at the function/struct/trait level.
2. **Map affected files** — which crates, modules, and specific functions each PR touches.
3. **Identify the original finding** — trace each PR back to the code review finding it addresses (from `<review_triage>` if provided, or from the PR description).
4. **Assess fix correctness** — does the PR actually fix the finding? A PR that claims to fix a budget leak but only adds a comment is not a fix. Read the code.

Build a complete map: PR number → files touched → functions modified → original finding → fix assessment.

---

## Phase 2: Conflict & Dependency Analysis

### File-Level Conflicts
Identify every case where two or more PRs modify the same file. For each overlap:
- Do they touch the same functions/structs, or different parts of the same file?
- Are the changes compatible (additive), conflicting (contradictory), or redundant (same fix, different approach)?

### Semantic Conflicts
Identify cases where PRs don't touch the same files but make semantically incompatible changes:
- PR A changes a type signature in `mister-smith-core`; PR B uses the old signature in `mister-smith-agents`
- PR A changes error handling behavior that PR B's tests depend on
- Two PRs add the same new function/struct with different implementations

### Dependency Ordering
Map merge prerequisites using the crate dependency tree:
- Changes to `mister-smith-core` must land before changes to downstream crates that depend on them
- Changes to trait definitions must land before changes to trait implementations
- Changes to `Cargo.toml` (new dependencies, feature flags) must land before code that uses them

### Redundancy & Supersession
Identify PRs that should NOT both merge:
- Two PRs that fix the same finding differently — pick the better one, close the other
- A PR whose changes are a strict subset of another PR
- A PR that fixes something a different PR's changes make obsolete

---

## Phase 3: Merge Strategy

Based on Phase 2 analysis, assign each PR one of these dispositions:

| Disposition | When | Action |
|-------------|------|--------|
| **Merge as-is** | Fix is correct, no conflicts, clean rebase | Rebase onto current `main`, merge |
| **Merge with edits** | Fix is mostly correct but needs adjustment | Check out branch, apply edits, force-push, merge |
| **Combine with PR #N** | Two PRs touch same area or fix related issues | Cherry-pick or rebase one onto the other, merge the combined branch, close the other |
| **Reorder** | PR depends on another PR landing first | Defer until prerequisite merges |
| **Discard** | Fix is wrong, redundant, or superseded | Close with explanation |

### Merge Phases

Sort PRs into ordered phases using topological order from the dependency analysis:

- **Phase 1**: PRs with no dependencies on other open PRs — merge these first
- **Phase 2**: PRs that depend on Phase 1 PRs — rebase after Phase 1 completes
- Continue until all PRs are assigned

Within each phase, order by:
1. Core crate changes first (`mister-smith-core`, `mister-smith-config`)
2. Domain crate changes next (in dependency order)
3. Test-only changes last

### Present the Plan

Before executing, present the full merge plan:
- Phase breakdown with PR numbers, dispositions, and ordering rationale
- Which PRs are being combined and why
- Which PRs are being discarded and why
- Which PRs need edits and what the edits are
- Expected conflict points and resolution approach

---

## Phase 4: Execute

For each phase, in order:

### Before each PR merge
- Rebase the PR branch onto current `main` (main moves after each merge)
- If conflicts arise, resolve them — combine both sides' intent, don't just pick one
- If the PR needs edits (from Phase 3), apply them on the branch before merging

### After each phase
Verify `main` is stable. If a merge broke something, fix it immediately before proceeding — don't batch broken merges.

### Combining PRs
When combining: check out the primary PR branch, cherry-pick or rebase the secondary PR's commits onto it, resolve conflicts, force-push the primary branch, merge it, close the secondary PR as superseded.

### After all phases
Confirm the final state: all intended fixes applied, no open PRs that should have been merged, workspace compiles and tests pass.

---

## Conflict Resolution — Mister Smith Patterns

These are common conflict patterns in this codebase and how to resolve them:

| Pattern | Resolution |
|---------|------------|
| Both PRs add variants to the same enum in `mister-smith-core` | Combine all variants; check `#[non_exhaustive]` is preserved |
| Both PRs modify the same function in a domain crate | Read both diffs; combine intent; if approaches conflict, take the one that better matches the governing spec |
| Both PRs add `#[cfg(feature = "...")]` blocks to the same module | Verify feature gates don't overlap or shadow each other |
| Both PRs modify `Cargo.toml` dependencies | Combine dependency lists; verify version compatibility; single `Cargo.lock` resolution |
| Type change in core + usage in downstream crate | Always land the type change first; adapt downstream usage in the next phase |
| Lock/concurrency changes conflict | Take the safer version — narrower critical section, fewer `unwrap()` calls, correct `Send + Sync` bounds |
| Both PRs add the same helper/utility | Keep the more complete one; close the other as superseded |
| Both PRs modify `MessageEnvelope` or transport types | Extra care — verify backward compat (`#[serde(default)]`, `Option<T>`) is preserved after combining |

---

## Secondary: Verification Commands

Run between phases only as needed — CI handles this for merged PRs.

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

Useful for conflict resolution:
```bash
gh pr list --state open --json number,title,headRefName
gh pr view <NUMBER> --json files --jq '.files[].path'
gh pr diff <NUMBER>
gh pr checkout <NUMBER>
gh pr merge <NUMBER> --squash --delete-branch
gh pr close <NUMBER> --comment "Superseded by #N" --delete-branch
```
