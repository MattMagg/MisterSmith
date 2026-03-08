---
description: Review, triage, and merge a large batch of open PRs in dependency order
---

# Bulk PR Merge Workflow

Systematically review, group, resolve conflicts, and merge a large set of open PRs into `main` while maintaining codebase stability.

---

## Phase 0 — Inventory & Grouping

1. **List all open PRs:**
   ```bash
   // turbo
   gh pr list --state open --limit 100 --json number,title,headRefName,labels,baseRefName
   ```

2. **For each PR, fetch metadata & diff:**
   ```bash
   // turbo
   gh pr view <NUMBER> --json title,body,files,additions,deletions,headRefName
   gh pr diff <NUMBER>
   ```

3. **Group PRs by subsystem / crate / feature area.** Create a grouping artifact with:
   - Group name (e.g., "Concurrency & Soundness", "Actor System", "Transport")
   - PR numbers and titles in each group
   - Brief one-line rationale for each grouping

4. **If delegating to parallel agents**, produce a self-contained review prompt per group that includes:
   - The PR numbers, the `/pr-review` workflow reference
   - Instructions to write the review report to a specific artifact path

---

## Phase 1 — Individual PR Review

For **each PR** (or each group if running in parallel), follow the PR review workflow (`/pr-review`). At minimum, capture per-PR:

| Field | Description |
|-------|-------------|
| **Verdict** | ✅ Merge as-is / ⚠️ Merge with fixes / ❌ Needs rework / 🗑️ Discard |
| **Blocking issues** | Concrete bugs, compile errors, logic flaws |
| **Suggestions** | Non-blocking improvements |
| **Cross-PR conflicts** | Files this PR touches that other PRs also touch |
| **Dependencies** | Which PRs must land first |

---

## Phase 2 — Synthesis & Merge Plan

1. **Collect all review reports** (yours + any parallel agent reports).

2. **Build a dependency graph:**
   - For each PR, note which other PRs it textually or semantically conflicts with (same files / same functions).
   - Identify prerequisite PRs (e.g., PR #80 depends on #76's `Arc<AtomicUsize>`).

3. **Sort PRs into merge phases** using topological order:
   - **Phase N**: PRs with no unmerged dependencies → merge these first
   - **Phase N+1**: PRs that depend on Phase N PRs → rebase after Phase N merges
   - Continue until all PRs are assigned

4. **Within each phase**, order PRs so that:
   - Conflict-free PRs merge first (fast merges warm up the baseline)
   - PRs requiring code fixes merge after their fixes are applied
   - PRs that should be combined are noted (e.g., two PRs touching the same health abstraction)

5. **Create an implementation plan artifact** with:
   - Phase breakdown with PR numbers, titles, and merge order
   - Per-PR action: "merge as-is", "rebase then merge", "fix X then merge", "combine with #Y", "discard"
   - Verification gates between phases

6. **Get user approval** on the plan before executing.

---

## Phase 3 — Execute Merges (Per Phase)

Repeat the following block for each phase in order:

### 3a. Update local main
```bash
// turbo
git fetch origin main && git checkout main && git reset --hard origin/main
```

### 3b. For each PR in the phase (in order):

**Simple merge (no conflicts expected):**
```bash
gh pr merge <NUMBER> --squash --delete-branch --admin
```

**Rebase required (conflicts expected):**
```bash
# Checkout the PR branch
gh pr checkout <NUMBER>

# Rebase onto current main
git rebase origin/main

# If conflicts arise:
# 1. View conflict markers: grep -rn '<<<<<<' <conflicted_files>
# 2. View the conflicted regions in context
# 3. Resolve by combining both sides' intent (don't just pick one)
# 4. Stage resolved files: git add <file>
# 5. Continue rebase: git rebase --continue

# Force push the rebased branch
git push origin <BRANCH> --force

# Merge
gh pr merge <NUMBER> --squash --delete-branch --admin
```

**Fix required before merge:**
```bash
# Checkout the PR branch
gh pr checkout <NUMBER>

# Apply the fix (edit files as needed)
# Stage and amend the commit
git add <files>
git commit --amend --no-edit

# Force push
git push origin <BRANCH> --force

# Merge
gh pr merge <NUMBER> --squash --delete-branch --admin
```

**Combine two PRs:**
```bash
# Checkout the first PR
gh pr checkout <NUMBER_A>

# Cherry-pick or rebase the second PR's changes on top
git cherry-pick <COMMIT_FROM_PR_B>

# Resolve any conflicts, force push PR A's branch
# Merge PR A, then close PR B as superseded
gh pr merge <NUMBER_A> --squash --delete-branch --admin
gh pr close <NUMBER_B> --comment "Superseded by #<NUMBER_A>" --delete-branch
```

### 3c. Post-phase verification

After each phase completes:

```bash
// turbo
git fetch origin main && git checkout main && git reset --hard origin/main
```

```bash
# Run the full test suite
cargo test --workspace 2>&1
```

- If tests fail, **fix immediately** on main (or revert the last merge) before proceeding.
- If tests pass, update the task tracker and move to the next phase.

---

## Phase 4 — Final Verification

After all phases are merged:

1. **Full test suite:**
   ```bash
   cargo test --workspace 2>&1
   ```

2. **Clippy lint check:**
   ```bash
   // turbo
   cargo clippy --workspace -- -D warnings 2>&1
   ```

3. **Verify no open PRs remain** (or only intentionally deferred ones):
   ```bash
   // turbo
   gh pr list --state open --json number,title
   ```

4. **Create a walkthrough artifact** summarizing:
   - Total PRs merged, by phase
   - Conflicts resolved and how
   - PRs that were discarded or combined
   - Any remaining follow-up work

---

## Key Principles

- **Never merge a PR without rebasing first** if main has moved since the PR was created.
- **Always run `cargo test --workspace`** (or project-equivalent) between phases — never batch-merge blind.
- **Use `--admin` flag** to bypass CI gates only when the PR is otherwise approved and tests pass locally.
- **Fix forward, don't revert** — if a merge introduces a compile error, fix it on main immediately rather than reverting (unless the fix is non-trivial).
- **Track everything** in the task artifact — mark each PR as ✅ merged, current phase progress, and any blockers.

## Common Conflict Resolution Patterns

| Pattern | Resolution |
|---------|------------|
| Both PRs add imports | Combine all imports |
| Both PRs modify the same function | Read both diffs carefully; combine the intent of both changes |
| PR changes a type that another PR uses | Apply the type change first, then adapt the usage PR |
| Two PRs add the same abstraction differently | Merge the better one, close the other as superseded |
| Lock/concurrency changes conflict | Always take the safer (more correct) version, usually the one that narrows the critical section |
