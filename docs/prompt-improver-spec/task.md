# Task Checklist — Mister Smith Pre-Packet Rube Tavily Research Handoff

- [x] Step 1: identified the source prompt intent and normalized the embedded examples from the
  user request
- [x] Step 2: documented deployment context, variables, flow, and preserved constraints in
  `docs/prompt-improver-spec/implementation_plan.md`
- [x] Step 3: wrote the initial draft prompt to
  `docs/prompt-improver-spec/final-prompts/mister-smith-pre-packet-rube-tavily-research-handoff-draft.md`
- [x] Step 4: critiqued the draft and recorded revision work in
  `docs/prompt-improver-spec/implementation_plan.md`
- [x] Step 5: applied revisions and saved the final prompt to
  `docs/prompt-improver-spec/final-prompts/mister-smith-pre-packet-rube-tavily-research-handoff.md`
- [x] Step 6: wrote `docs/prompt-improver-spec/walkthrough.md`, removed the draft file, and
  validated the touched prompt-improver files

## Validation

- [x] `npx markdownlint-cli2 "docs/prompt-improver-spec/*.md" "docs/prompt-improver-spec/final-prompts/mister-smith-pre-packet-rube-tavily-research-handoff.md" --config .markdownlint.json`
- [x] `git diff --check`
