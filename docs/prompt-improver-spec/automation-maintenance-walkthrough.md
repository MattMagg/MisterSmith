# Walkthrough — Mister Smith Recurring Maintenance Prompts

## Original Prompt Set Summary

The source material described four recurring workflows:

1. keep the repo's tracking and status docs accurate
2. detect dependency and SDK drift
3. scan recent commits for likely bugs
4. audit performance regressions

The main improvement need was not more content. It was stronger separation of concerns, better
Mister Smith grounding, clearer issue-routing rules, and explicit output paths for recurring docs.

## Key Improvements

- broadened the first prompt from `AGENTS.md` only to the full repo tracking/status doc set
- removed optional-input XML sections because these prompts are meant to run unattended as fixed
  automations
- made the first prompt future-proof by replacing hardcoded packet references with dynamic packet
  and proof discovery rules
- replaced generic repo-source lists with exact Mister Smith files and runtime-proof surfaces
- kept all four prompts separate because they use different evidence standards and likely different
  cadences
- grounded the prompts in Mister Smith's current router and workflow contract docs instead of vague
  "search the repo" language
- added a dedicated recurring-report folder:
  `docs/automation-reports/`
- added a dated filename convention for report-writing workflows
- added explicit GitHub and Linear issue-routing rules for bugs, regressions, and proposed code
  changes
- added direct-to-`main` instructions for documentation-only edits

## Before And After

### Before

- brief task statements
- limited repo specificity
- no shared naming convention for generated docs
- no concrete issue metadata instructions

### After

- four standalone prompts with clear boundaries
- one repo-truth sync prompt that targets all eight tracking/status docs
- repo-specific grounding order and source list
- dated report output paths
- GitHub issue-template guidance plus Linear project, state, priority, and label guidance
- explicit commit and validation rules for documentation-only runs

## Final Prompt Locations

- `docs/prompt-improver-spec/final-prompts/mister-smith-tracking-and-status-doc-sync.md`
- `docs/prompt-improver-spec/final-prompts/mister-smith-dependency-and-sdk-drift.md`
- `docs/prompt-improver-spec/final-prompts/mister-smith-bug-scan.md`
- `docs/prompt-improver-spec/final-prompts/mister-smith-performance-audit.md`
