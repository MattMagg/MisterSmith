# Task Checklist — Mister Smith R9 Deep-Research Prompt Suite

## Steps 1-3: Planning and Initial Draft

- [x] Step 1: normalized the user request into an additive `R9` deep-research prompt suite
- [x] Step 1a: identified existing prompt families and the relevant repo baseline under
  `docs/research-output/`
- [x] Step 2: mapped the five production prompts, README, baseline docs, and overlap boundaries
- [x] Step 2a: locked the final production home to `docs/research-prompts/R9/`
- [x] Step 3: created one temporary draft prompt per production prompt under
  `docs/prompt-improver-spec/final-prompts/`

## Steps 4-6: Critique and Finalization

- [x] Step 4: critiqued the draft suite for overlap, stale baseline reuse, and weak frontier
  gating
- [x] Step 5: strengthened the prompts with explicit baseline-doc lists, frontier classification,
  implementation vectors, and production-vs-research separation
- [x] Step 6: finalized the production prompt set under `docs/research-prompts/R9/`
- [x] Step 6a: removed all temporary `r9-*-draft.md` files from
  `docs/prompt-improver-spec/final-prompts/`
- [x] Validation: run targeted markdownlint on touched files
- [x] Validation: run `git diff --check`
- [x] Validation: verify no `r9-*-draft.md` files remain
