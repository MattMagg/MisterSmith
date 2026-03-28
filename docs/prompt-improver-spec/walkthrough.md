# Walkthrough — Mister Smith R9 Deep-Research Prompt Suite

## Original Prompt Summary

The source request asked for a fresh deep-research refresh in this repository, aimed at recent
advancements in workflow, orchestration, coordination, and real-time inter-agent communication
that are not already covered by the prior research corpus. The user explicitly wanted a series of
markdown prompt documents, produced through the prompt-improver workflow and filtered through the
Mister Smith frontier mandate.

The key design challenge was not whether to make more prompts. The repo already had substantial
`R8` deep-research prompts. The real problem was to turn the documented corpus gaps into a new
`R9` suite without duplicating the existing prompt batch or drifting into generic market scanning.

## Key Improvements Made

- converted a broad "catch me up" request into a concrete `R9` suite with five prompt owners and
  one suite README
- grounded every prompt in repo authority:
  - `docs/current-state.md`
  - `docs/research-output/ROUTING_MANIFEST.md`
  - prompt-specific consolidated research docs
- rebuilt the "already known" sections from `docs/research-output/*` instead of copying `R8`
  baseline text
- separated transport-level real-time communication from collaborative communication policy so
  the suite does not collapse transport, protocol safety, and cognitive alignment into one prompt
- added shared output requirements across the suite:
  - frontier classification
  - Mister Smith implementation vector
  - production-validated vs research-only separation
  - thin-results reporting
  - contradictions to current assumptions
- kept the frontier-first, anti-market-copying stance visible in each production prompt

## Before / After Comparison

### Scope shape

- Before: one broad request that could have turned into a loose set of overlapping prompts
- After: five production prompts with explicit overlap boundaries plus a README that explains the
  run order

### Baseline authority

- Before: existing `R8` prompts could have been reused too literally
- After: the suite treats `docs/research-output/*` as the hard "already discovered" boundary and
  uses `R8` only as structural reference

### Output rigor

- Before: a research agent could return a generic update or a market-comparison memo
- After: every prompt requires frontier taxonomy, implementation vectors, thin-results honesty,
  and contradiction reporting

## How To Use The Improved Prompt Suite

1. Start with `docs/research-prompts/R9/README.md` to choose a run order.
2. Run one prompt at a time with a deep-research agent that has live web access.
3. Treat the prompt text as production-ready; the temporary drafts under
   `docs/prompt-improver-spec/final-prompts/` are intentionally removed after finalization.
4. Feed any resulting findings back into `docs/research-output/` rather than modifying the prompt
   suite unless the baseline itself changes.

## Final Prompt Locations

- `docs/research-prompts/R9/README.md`
- `docs/research-prompts/R9/01-workflow-engines-compensation-and-resume.md`
- `docs/research-prompts/R9/02-dynamic-orchestration-and-topology-control.md`
- `docs/research-prompts/R9/03-coordination-protocols-shared-state-and-dynamic-verification.md`
- `docs/research-prompts/R9/04-real-time-inter-agent-communication-and-transport.md`
- `docs/research-prompts/R9/05-collaborative-communication-handoffs-and-cognitive-alignment.md`
