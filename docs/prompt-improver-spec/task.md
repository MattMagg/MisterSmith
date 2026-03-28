# Task Checklist — Mister Smith Post-Research Analysis Brief Prompt

- [x] Step 1: identified the source prompt intent and normalized the example report-path input
- [x] Step 2: documented deployment context, variables, flow, structural goals, and preserved
  constraints in `implementation_plan.md`
- [x] Step 3: wrote the initial draft prompt to
  `docs/prompt-improver-spec/final-prompts/mister-smith-post-research-analysis-brief-draft.md`
- [x] Step 4: critiqued the draft and recorded revision work in `implementation_plan.md`
- [x] Step 5: applied revisions and saved the final prompt to
  `docs/prompt-improver-spec/final-prompts/mister-smith-post-research-analysis-brief.md`
- [x] Step 6: wrote `walkthrough.md`, removed the draft file, and validated touched markdown files

## Validation

- [x] `npx markdownlint-cli2 "docs/prompt-improver-spec/*.md" "docs/prompt-improver-spec/final-prompts/mister-smith-post-research-analysis-brief.md" --config .markdownlint.json`
- [x] `git diff --check`
