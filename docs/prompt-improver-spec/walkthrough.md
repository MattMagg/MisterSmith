# Walkthrough — Mister Smith Post-Packet-020 Next Phase Spec Handoff Prompt

## Original Prompt Summary

The source request asked for a prompt, built through the prompt-improver workflow, that tells a
fresh agent what to do next after the March 27 packet-020 follow-up work.

The key challenge was not writing a generic "next phase" prompt. It was producing a forward
development handoff that:

- starts from current repo truth
- routes through the existing `docs/research-output/` corpus
- respects the frontier mandate instead of drifting toward framework imitation
- does not assume a next packet already exists
- keeps the receiving session on research/spec-building rather than implementation

## Key Improvements Made

- converted the vague "what's next?" request into a concrete fresh-session handoff for deciding
  whether the next honest deliverable is:
  - one bounded new SpecKit packet
  - or one checkpoint note explaining why freezing a packet would be premature
- anchored the prompt on the current March 27 authority stack:
  - `docs/current-state.md`
  - packet-020 closure note
  - March 27 runtime-planning simplification note
  - March 27 `MS-110` evidence-freeze note
- added the repo's research corpus to the required grounding pass:
  - `docs/research-output/ROUTING_MANIFEST.md`
  - `docs/research-output/consolidated/00-MASTER-FINDINGS.md`
  - the most relevant consolidated frontier documents
- added a frontier-legitimacy gate so the next agent must use Smith tooling before turning a
  speculative research direction into a new packet
- added explicit code-grounding surfaces so the receiving agent checks current runtime and operator
  gaps before freezing scope
- added anti-patterns that forbid reopening packet-020 work or slipping into implementation
- kept the prompt from pre-solving the next packet choice

## Before / After Comparison

### Scope framing

- Before: "write up a prompt for the next phase of development"
- After: a prompt that tells the receiving agent to verify whether a new bounded phase is actually
  ready to freeze

### Forward-development posture

- Before: ambiguous between research, spec writing, and implementation
- After: explicitly a research-and-spec session with a stop path when a new packet would be
  dishonest, plus a required frontier-legitimacy check before packet freeze

### Repo-truth grounding

- Before: could have reused older pre-packet prompts
- After: uses the current March 27 repo state, the existing research-output corpus, and recent
  evidence notes as the authority base

## How To Use The Improved Prompt

1. Start a fresh Codex session in `/Users/macmain/MisterSmith`.
2. Paste the final prompt from the file below.
3. Let the receiving agent determine whether current repo truth supports:
   - one new bounded SpecKit packet
   - or one checkpoint note instead
4. Expect the receiving agent to stop at checkpoint/triage if the research corpus and legitimacy
   checks do not yet justify a frontier-worthy bounded packet

## Final Prompt Location

- `docs/prompt-improver-spec/final-prompts/mister-smith-post-packet-020-next-phase-spec-handoff.md`
