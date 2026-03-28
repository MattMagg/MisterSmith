# Walkthrough — Mister Smith Post-Research Analysis Brief Prompt

## Original Prompt Summary

The user wanted a reusable prompt-improver deliverable for a very specific follow-on workflow:
after an external research report has already been produced, bring that report into the Mister
Smith repo and have a local agent analyze it deeply against Mister Smith's existing or proposed
architecture.

The report path `/Users/macmain/Downloads/deep-research-report.md` was only an example of the kind
of artifact that will be received. The prompt was not supposed to execute against that file during
this run.

## Final Prompt Location

`docs/prompt-improver-spec/final-prompts/mister-smith-post-research-analysis-brief.md`

## Key Improvements Made

- shifted the task from vague "analyze deeply" language to an explicit **post-research local
  analysis** workflow
- made imported research reports the **primary evidence** and repo-local context a **baseline for
  transfer analysis**, not a reason to re-run research or audit repo state
- added support for **one or more reports** so the prompt can consolidate overlapping findings
- strengthened the prompt from summary-oriented to **decision-brief-oriented**
- added explicit handling for:
  - novelty versus existing or proposed architecture
  - implement now / prototype / monitor / not worth pursuing posture
  - further-research needs before implementation
  - separation of imported evidence from repo-local inference
- added anti-patterns and a final verification checklist so the receiving agent does not drift into
  generic summarization

## Before / After Highlights

### Before

- the draft said to "compare those findings to Mister Smith's architecture and research baseline"
  but did not define how repo-local context should be used
- the output shape was short and risked producing a linear summary
- the draft did not explicitly block accidental new web research

### After

- the final prompt explicitly says this is **not a new research run**
- repo-local context is explicitly limited to judging novelty, fit, leverage, and transferability
- the output is framed as a **decision-grade brief**
- evaluation lenses, anti-patterns, and a verification checklist make the prompt more reliable in
  real use

## How To Use The Final Prompt

1. Open a new local Codex session in `/Users/macmain/MisterSmith`.
2. Provide one or more completed research reports inside `<research_reports>`.
3. Optionally provide:
   - `<analysis_goal>` for a narrower question
   - `<architecture_context>` for specific local design docs
   - `<existing_research_context>` for prior repo-local synthesis
   - `<decision_horizon>` if near-term versus later-stage separation matters
4. Let the receiving agent produce the brief without starting a new research pass.

## Cleanup Performed

- created the draft prompt under `docs/prompt-improver-spec/final-prompts/`
- created the production prompt in the same directory
- removed the draft file after finalization
