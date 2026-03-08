---
name: code-review-cycle
description: Use when a completed phase, feature, or significant code change in Mister Smith needs post-implementation review — especially before merge, after multi-agent implementation sessions, or when dispatching parallel review versions to cloud agents.
---

# Code Review Cycle

Three-stage executable workflow for post-implementation review in the Mister Smith framework. You (the local agent) drive each stage — generating deliverables, performing analysis, and executing merges.

## Stages at a Glance

| Stage | What YOU Do | Deliverable |
|-------|-------------|-------------|
| **1: Review Dispatch** | Analyze current context, fill in prompt template variables, generate a ready-to-send prompt document | `deliverables/review-prompt-<scope>.md` — user copies to cloud agents |
| **2: Triage** | User pastes back 4 version results; you verify findings against source, deduplicate, prioritize | `deliverables/triage-report-<scope>.md` — ordered task list |
| **3: PR Merge** | After fix PRs are open, you inventory, plan merge strategy, and execute | `deliverables/merge-plan-<scope>.md` — merge plan before execution |

Deliverables are saved to `.claude/skills/code-review-cycle/deliverables/`.

---

## Stage 1: Generate Review Prompt for Cloud Agents

**You are NOT doing the review.** You are preparing a filled-in prompt document that the user will send to cloud coding agents (Codex, etc.) which run 4 independent versions.

### What to do:

1. **Determine the review scope** from conversation context:
   - What was just implemented? Which commits, which crates, which phase?
   - Run `git log --oneline` to identify the relevant commit range
   - Run `git diff <base>..HEAD --stat` to map all affected files

2. **Identify the governing spec:**
   - Which spec in `specs/` governs this implementation?
   - Which contract files in the spec's `contracts/` directory are relevant?

3. **Build the additional context:**
   - What trade-offs were made during implementation? (Check conversation history, commit messages, code comments)
   - Were there known deferred items?
   - Were multiple agent sessions involved? (Higher risk of convention drift)
   - Any areas of specific concern?

4. **Read the template** from `references/mister-smith-post-implementation-review.md`

5. **Generate the deliverable** — a complete markdown document with all `<review_scope>`, `<governing_spec>`, and `<additional_context>` variables filled in with the real content from steps 1-3. Write it to `deliverables/review-prompt-<scope>.md`.

6. **Tell the user** the document is ready and where to find it. They will copy it to the cloud agent platform and run 4 versions.

### Example deliverable filename:
- `deliverables/review-prompt-phase9-llm.md`
- `deliverables/review-prompt-pr-142.md`
- `deliverables/review-prompt-budget-refactor.md`

**STOP after Stage 1.** The user needs to send the prompt to cloud agents and wait for results. Stage 2 begins when they bring back the version results.

---

## Stage 2: Triage Version Results

**The user pastes 4 version results.** You verify, deduplicate, and prioritize.

### What to do:

1. **Read the triage template** from `references/mister-smith-review-triage.md`

2. **Execute the triage workflow directly** — you have codebase access, so you can:
   - Read every cited `file:line` to verify findings are real
   - Check commit messages and specs to determine intentionality
   - Trace logic to confirm or refute each reviewer's analysis

3. **Fill in the variables and run the analysis:**
   - `<review_versions>` — the 4 version results the user provided
   - `<review_scope>` — same scope from Stage 1
   - `<additional_context>` — same context from Stage 1 plus anything the user adds

4. **Produce the triage report** following the template's Phase 1-4 workflow (Inventory → Verify → Deduplicate → Prioritize). Save to `deliverables/triage-report-<scope>.md`.

5. **Present the ordered task list** to the user in `V.I` format. They will dispatch fix tasks to coding agents, which produce PRs.

**STOP after Stage 2.** The user dispatches fix tasks. Stage 3 begins when the fix PRs are open.

---

## Stage 3: PR Merge Strategy & Execution

**Fix PRs are open.** You plan and execute the merge.

### What to do:

1. **Read the merge template** from `references/mister-smith-pr-merge-strategy.md`

2. **Inventory open PRs:**
   ```bash
   gh pr list --state open --json number,title,headRefName
   ```

3. **Execute the merge workflow directly:**
   - Read every PR diff
   - Map file-level and semantic conflicts
   - Identify dependency ordering using the crate tree
   - Assign dispositions (merge/combine/edit/reorder/discard)
   - Sort into topological merge phases

4. **Save the merge plan** to `deliverables/merge-plan-<scope>.md` and present it to the user for approval before executing.

5. **After user approval**, execute the merge phases with verification between each phase.

---

## Invoking the Skill

The user can invoke any stage independently:

- **`/code-review-cycle`** — Start from Stage 1 (most common: full cycle)
- **`/code-review-cycle triage`** — Jump to Stage 2 (user already has version results)
- **`/code-review-cycle merge`** — Jump to Stage 3 (fix PRs are already open)

If no argument is given, start at Stage 1 and determine context from the conversation.
