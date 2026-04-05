# Automation Reports

Use this folder for recurring Codex-generated maintenance reports that should stay separate from
one-off plans in `docs/plans/`.

The tracking-and-status-doc sync workflow does not write a report here. That workflow updates the
repo-truth docs in place.

## Naming Convention

- `YYYY-MM-DD-dependency-and-sdk-drift.md`
- `YYYY-MM-DD-bug-scan.md`
- `YYYY-MM-DD-performance-audit.md`

Use the local repo date in `YYYY-MM-DD` format.

If the same workflow runs more than once on the same day, update the existing day's file instead of
creating duplicate variants unless the user explicitly asks for separate rerun artifacts.
