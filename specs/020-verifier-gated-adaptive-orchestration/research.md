# Research Notes: Verifier-Gated Adaptive Orchestration

## Current workflow truth

- the current runtime path already ships supervised planner and executor lifecycles plus ToolBus
  execution and autonomy/task provenance
- packet `019` closed the provider and budget control-loop gap, but it did not change workflow
  quality control between intermediate steps
- the current shipped path still lacks a first-class verifier gate, clarification contract, and
  checkpoint-based repair directive

## Research signals that matter here

### Step-level verification and planner/executor/verifier decomposition

- `docs/research-output/research/discovery-sweep-R4.md` identifies PRMs as the missing step-level
  verification seam and highlights the planner-executor-verifier decomposition from AgentFlow with
  Flow-GRPO
- `docs/research-output/research/targeted-step-level-intelligence-R6.md` recommends strict
  orchestrator-owned step evaluation, local rollback from the last good checkpoint, and a hard
  boundary between generation and verification

### Clarification and contextual repair

- `docs/research-output/research/targeted-supervision-fault-tolerance-R4.md` highlights
  AgentAsk-style clarification, structured reflection before retry, and COCO contextual rollback
- `docs/research-prompts/R8/06-predictive-supervision.md` treats clarification modules and failure
  context propagation as already justified baseline architecture, but not yet fully wired into the
  live runtime

### Adaptive workflow refinement

- `docs/research-output/research/discovery-sweep-R7b.md` points at adaptive workflow refinement
  and edge-level error mitigation as the next orchestration frontier for reliability

### Explicitly deferred frontier paths

- `docs/research-output/research/targeted-stigmergy-swarm-coordination-R4.md` shows workflow
  evolution and self-improving archives as promising later benchmark programs, but that is broader
  than this packet
- token-stream PRMs, speculative decoding, and learned routing remain relevant follow-ons, but
  this packet begins at explicit workflow-step boundaries

## Bounded conclusion

The next legitimate gap is not another provider or budget slice. It is verifier-gated adaptive
orchestration on the shipped workflow path: catch bad intermediate steps, clarify weak handoffs,
repair locally from the last stable checkpoint, and make the repair history visible.
