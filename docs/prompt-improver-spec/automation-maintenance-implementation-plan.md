# Implementation Plan — Mister Smith Recurring Maintenance Prompts

## Step 1: Example Identification

### Source Prompt Set

Improve four recurring Codex workflow prompts for Mister Smith:

1. update `AGENTS.md`
2. detect dependency and SDK drift
3. scan recent commits for likely bugs
4. audit performance regressions

Constraints carried forward from the user request:

- do not merge prompts unless the split stays honest
- use the prompt-improver workflow style
- tailor the prompts to Mister Smith repo truth
- do not describe them as automations inside the prompts
- for documentation work, instruct direct commits to `main`
- for bugs, issues, or proposed code changes, create both GitHub and Linear issues with strong
  metadata
- for document-writing workflows, define a dated naming convention and folder

### Normalized Input To Ideal Output Pairs

- Input: "Update AGENTS.md with newly discovered workflows and commands"
  Ideal output: a repo-truth sync prompt that prioritizes current state and router docs, keeps
  edits minimal, and updates `AGENTS.md` only when grounded
- Input: "Dependency and SDK drift"
  Ideal output: a drift-audit prompt that proves drift from manifests and lockfiles, writes a
  dated report, and opens issues only when drift is real
- Input: "Bug scan"
  Ideal output: a recent-change correctness prompt that relies on concrete commit, diff, test, and
  CI evidence and opens issues for credible bugs
- Input: "Performance audit"
  Ideal output: a performance-regression prompt that relies on measurements when available, writes
  a dated report, and opens issues only for grounded regressions or bounded follow-up work

### What The Examples Demonstrate

- the four prompts have different evidence standards and should not be collapsed into one vague
  maintenance sweep
- Mister Smith repo truth must come from current router and workflow docs before broader lore
- dated reports belong in a dedicated docs folder, not mixed into packet plans
- issue creation must be repo-native and metadata-rich rather than generic

## Step 2: Planning Analysis

### Intent Summary

Create four reusable prompts for recurring Mister Smith maintenance work without blurring repo-doc
sync, drift analysis, bug triage, and performance work into one overbroad task.

### Deployment Summary

- prompt artifact plan:
  `docs/prompt-improver-spec/automation-maintenance-implementation-plan.md`
- task checklist:
  `docs/prompt-improver-spec/automation-maintenance-task.md`
- walkthrough:
  `docs/prompt-improver-spec/automation-maintenance-walkthrough.md`
- final prompts:
  - `docs/prompt-improver-spec/final-prompts/mister-smith-agents-doc-sync.md`
  - `docs/prompt-improver-spec/final-prompts/mister-smith-dependency-and-sdk-drift.md`
  - `docs/prompt-improver-spec/final-prompts/mister-smith-bug-scan.md`
  - `docs/prompt-improver-spec/final-prompts/mister-smith-performance-audit.md`
- report folder:
  `docs/automation-reports/`

### Split Decision

Keep four prompts.

Reason:

- `AGENTS.md` sync is a documentation truth-maintenance task
- dependency drift is version-alignment analysis
- bug scan is correctness triage from concrete repo signals
- performance audit depends on measurement evidence and different acceptance rules

Combining any of these would weaken the grounding rules and make scheduling less useful.

### Repo Grounding Notes

Prompts should explicitly route through these Mister Smith sources where relevant:

- `docs/current-state.md`
- `docs/direction.md`
- `WORKFLOW.md`
- `docs/linear/LINEAR.md`
- `README.md`
- `CLAUDE.md`
- `ROADMAP.md`
- recent relevant `docs/plans/*.md`
- current packet authorities under `specs/022-*` through `specs/026-*` when packet/router truth
  matters
- `.codex/README.md`
- `.codex/agents/README.md`
- `.codex/commands/*.md` and `.codex/prompts/*.md` when command behavior is the claim being
  documented
- relevant Mister Smith workflow skills when they establish current repo usage

### Output Format

Markdown prompt files with XML-tagged optional inputs and explicit workflow sections.

### Variable Plan

| Prompt | Variables |
| ------ | --------- |
| AGENTS sync | `<report_date>`, `<focus_note>`, `<extra_paths>` |
| Dependency drift | `<report_date>`, `<lookback_window>`, `<scope_note>` |
| Bug scan | `<report_date>`, `<lookback_window>`, `<focus_note>` |
| Performance audit | `<report_date>`, `<lookback_window>`, `<focus_note>` |

### Constraint Preservation Checklist

- [x] Keep four prompts separate unless a merge is clearly better
- [x] Preserve repo-grounded evidence rules
- [x] Preserve the user's direct-to-main instruction for documentation-only work
- [x] Preserve the requirement to create both GitHub and Linear issues for real bugs or proposed
  code changes
- [x] Add a dated report-folder convention for document-writing prompts
