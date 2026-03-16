# Mister Smith Control-Plane Centralization Handoff

Use this prompt to start the next high-capability agent session.

---

You are working in the Mister Smith repository at `/Users/macmain/MisterSmith`.

Your mission in this session is to **plan and then execute** a repository-wide effort to improve,
integrate, and centralize Mister Smith workflow surfaces so that **Smith MCP becomes the primary
operating framework for Codex work in this repository**, including seamless workflow integration
with Ralph orchestrator and SpecKit.

Do not treat this as a narrow docs cleanup. Treat it as a control-plane consolidation effort across
repo guidance, planning artifacts, workflow definitions, Symphony integration, Linear conventions,
GitHub operating flow, and Smith MCP behavior.

<integration_boundary>
Ralph orchestrator and SpecKit are part of the workflow environment that Smith MCP must work with.

Your goal is to make Smith MCP workflows use, cooperate with, and build on Ralph and SpecKit where
appropriate.

Your goal is **not** to make Ralph or SpecKit the rewrite target.

Preserve the upstream boundary:

- do not treat Ralph itself as a repo-local orchestration surface to replace
- do not treat SpecKit itself as a repo-local planning system to rewrite
- do identify how Smith MCP workflows should invoke, complement, route into, or prepare work for
  Ralph and SpecKit
- do centralize the repo-local workflow glue, prompts, plans, wrappers, and operational contracts
  that make Smith MCP interoperate with Ralph and SpecKit cleanly
</integration_boundary>

<frontier_mandate>
Before doing anything else, read and apply
`/Users/macmain/MisterSmith/.agents/workflows/mister-smith-mandate.md`.

Operate with the repository's frontier mandate:

- benchmark conventional agent-framework patterns, but do not copy them by default
- reuse what is already correct
- where the choice affects coordination, execution, supervision, memory, streaming, routing,
  reliability, observability, state, or distributed behavior, prefer the higher-leverage
  architecture over the more familiar one
- do not settle for incremental imitation when a better operating model is available

</frontier_mandate>

<core_objective>
By the end of this session, you should have:

1. built a deep, repo-grounded understanding of how Mister Smith currently operates and how it
   evolved
2. consolidated the main workflow, planning, control-plane, and operational surfaces into one
   coherent model
3. identified stale, duplicated, fragmented, or contradictory workflow surfaces
4. produced a durable master plan for making Smith MCP the center of gravity for Codex work here
5. executed the highest-leverage reversible slice or slices that move the repository toward that
   target operating model, unless a blocker prevents safe execution

</core_objective>

<do_not_short_circuit>
Do not jump straight to implementation.

You must first build evidence, reconcile conflicting sources, and leave durable planning artifacts
before major edits.

Do not claim a workflow surface is current just because it exists.
Treat historical plans, prompt files, and prior handoffs as evidence, not truth.
</do_not_short_circuit>

## Control-Plane-First Start Sequence

1. Read these repo contracts first:
   - `AGENTS.md`
   - `CLAUDE.md`
   - `README.md`
   - `ROADMAP.md`
   - `WORKFLOW.md`
   - `docs/linear/LINEAR.md`
2. Use Smith MCP before raw fallbacks:
   - `route_workflow_request`
   - `get_control_plane_snapshot`
   - `get_symphony_checkout_snapshot`
   - `audit_workflow_readiness`
   - `sync_linear_with_runtime` when queue state or watched-project truth is relevant
3. Treat Rube MCP as the gateway for external research or app state when you need sources outside
   the repo.
4. Create or update one durable plan note under `docs/plans/` before major edits. That note must
   carry:
   - objective
   - current known state
   - verified facts versus inference
   - contradictions and open questions
   - execution slices
   - validation evidence
5. Explicitly account for the repo's Ralph and SpecKit posture from `WORKFLOW.md`:
   - Ralph is a loop runner, not a replacement for SpecKit or repo-native workflow contracts
   - if a workflow uses Ralph, `PROMPT.md` must be rewritten from the active issue or workpad
     context before `ralph run`
   - Smith MCP should orchestrate workflows that cooperate with Ralph and SpecKit without rewriting
     those upstream systems

## Current Anchors To Verify, Not Assume

These were the last locally observed anchors and must be re-verified at session start:

- `WORKFLOW.md` currently points Symphony at Linear project slug `320a0741920c`.
- Active workflow states currently include `Todo`, `In Progress`, `Merging`, and `Rework`.
- Smith MCP is launched through `scripts/run-smith-mcp.sh`.
- Symphony is launched through `scripts/run-symphony.sh`.
- The local Symphony checkout is expected at `/Users/macmain/symphony`.
- The Symphony workspace root is expected at `~/.local/share/symphony-workspaces`.
- A recent Smith control-plane snapshot reported the watched project had historical issues but no
  active issues in active workflow states.

Treat these as hypotheses to confirm through live repo and Smith MCP evidence before you use them in
planning or implementation.

## In-Scope Surfaces

Treat the following as in scope, but not exhaustive.

### Repo contracts and orientation

- `AGENTS.md`
- `CLAUDE.md`
- `README.md`
- `ROADMAP.md`
- `WORKFLOW.md`
- `docs/linear/LINEAR.md`
- `.agents/workflows/mister-smith-mandate.md`

### Smith MCP, Symphony, and launch surfaces

- `scripts/run-smith-mcp.sh`
- `scripts/run-symphony.sh`
- `crates/mister-smith-mcp/`
- `mistersmith-api.json`
- the local Symphony checkout at `/Users/macmain/symphony`

### Workflow and control-plane planning artifacts

- `docs/plans/2026-03-14-smith-mcp-rebuild.md`
- `docs/plans/2026-03-15-smith-mcp-workflow-forensics.md`
- `docs/plans/2026-03-15-smith-mcp-comprehensive-workflows.md`
- `docs/plans/2026-03-15-mister-smith-state-audit-and-recovery.md`
- `docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md`
- `docs/plans/2026-03-15-symphony-throughput-ramp.md`
- `docs/plans/2026-03-16-recovery-triage.md`
- `plans/IMPLEMENTATION_PLANNING_TRACKER.md`
- `docs/ms_recent_context.md`
- `docs/RESEARCH_CHECKPOINT.md`

### Skills, workflows, and agent operating surfaces

- `.codex/skills/mister-smith-control-plane-router/SKILL.md`
- `.codex/skills/symphony-linear-mister-smith/SKILL.md`
- `.codex/skills/symphony-mister-smith-review-dispatch/SKILL.md`
- `.codex/skills/commit/SKILL.md`
- `.codex/skills/pull/SKILL.md`
- `.codex/skills/push/SKILL.md`
- `.codex/skills/land/SKILL.md`

### Ralph and SpecKit workflow surfaces

- `ralph.yml`
- `PROMPT.md`
- `.specify/`
- `.codex/commands/`
- `.codex/prompts/speckit.*.md`
- `docs/plans/2026-03-15-phase10-spec-kit-refresh-and-audit.md`
- `docs/plans/2026-03-15-ms-35-phase10-gate-and-speckit-refresh.md`
- `docs/plans/2026-03-16-multi-turn-same-agent-conversations.md`

### Architecture and implementation surfaces likely to matter

- `spec/`
- `specs/`
- `crates/mister-smith-app/`
- `crates/mister-smith-agents/`
- `crates/mister-smith-runtime/`
- `crates/mister-smith-mcp/`
- `crates/mister-smith-llm/`
- `crates/mister-smith-events/`
- `crates/mister-smith-persistence/`

### GitHub and CI workflow surfaces

- `.github/workflows/`
- current open PRs
- branch and merge conventions described in repo docs and skills

## Parallel Research Lanes

After local grounding and the first control-plane snapshot, use parallel agents where helpful.
Keep each lane bounded, evidence-first, and non-overlapping.

Every lane must return:

- verified current facts
- historical artifacts that still matter
- stale or contradictory surfaces
- recommended consolidation targets
- open questions

### Lane A: Repository history and evolution

Focus:

- major development phases
- changes in operating model
- prior attempts to centralize workflows
- recovery and rebuild moments

Primary evidence:

- `git log`
- `docs/plans/`
- `docs/ms_recent_context.md`
- `plans/IMPLEMENTATION_PLANNING_TRACKER.md`

### Lane B: Workflow contracts and planning surfaces

Focus:

- current authoritative workflow surfaces
- overlapping instructions
- stale or duplicated guidance
- where repo contracts and historical plans disagree

Primary evidence:

- `AGENTS.md`
- `WORKFLOW.md`
- `docs/linear/LINEAR.md`
- `CLAUDE.md`
- `README.md`
- `ROADMAP.md`
- relevant `docs/plans/`

### Lane C: Smith MCP implementation and behavior

Focus:

- actual Smith MCP capability surface
- compatibility versus documentation
- gaps between `smith` tooling and the desired control-plane role
- whether Smith MCP currently behaves like a workflow control plane or a bag of admin tools

Primary evidence:

- `crates/mister-smith-mcp/`
- `scripts/run-smith-mcp.sh`
- `mistersmith-api.json`
- live Smith MCP tool outputs

### Lane D: Symphony and Linear integration

Focus:

- watched project model
- dispatch boundaries
- state transitions
- workspace lifecycle
- queue truth versus documentation

Primary evidence:

- `WORKFLOW.md`
- `docs/linear/LINEAR.md`
- `scripts/run-symphony.sh`
- live Smith control-plane snapshots
- Symphony checkout reality

### Lane E: GitHub, PR, and merge operating flow

Focus:

- branch and PR lifecycle
- automation versus actual merge behavior
- review gates
- where GitHub flow is documented, duplicated, or implicit

Primary evidence:

- `.github/workflows/`
- `.codex/skills/push/SKILL.md`
- `.codex/skills/land/SKILL.md`
- current open PR state
- repo workflow docs

### Lane F: Agent prompts, skills, Ralph, SpecKit, and operational memory surfaces

Focus:

- where important operational knowledge is embedded in prompts, plans, skills, and prior handoffs
- which surfaces are current operating truth versus historical scaffolding
- what should move into Smith MCP-centered workflows or consolidated repo docs
- how Smith MCP should interoperate with Ralph and SpecKit without taking ownership of their
  upstream implementations

Primary evidence:

- `.agents/workflows/`
- `.codex/skills/`
- `ralph.yml`
- `PROMPT.md`
- `.specify/`
- `.codex/commands/`
- `.codex/prompts/speckit.*.md`
- `docs/prompt-improver-spec/`
- `docs/plans/`
- `docs/ms_recent_context.md`

## Required Analysis

Your investigation must explicitly answer these questions:

1. What is the current authoritative operating model for Mister Smith across repo, Smith MCP,
   Symphony, Linear, and GitHub?
2. What has already been built, learned, planned, repaired, or partially attempted?
3. Which workflow surfaces are duplicated, stale, fragmented, contradictory, or over-specialized?
4. Which important operational truths live in too many places or in the wrong places?
5. What should be centralized into Smith MCP behavior, Smith MCP-facing docs, or repo contracts?
6. What should remain external to Smith MCP because another system is the true source of record?
7. What is the minimum viable path to make Smith MCP the primary orchestration layer for:
   - in-session orchestration
   - chained workflow execution
   - Linear coordination
   - Symphony workflow integration
   - GitHub and PR operations
   - long-lived operational context for developing Mister Smith
8. How should Smith MCP workflows integrate with Ralph orchestrator and SpecKit so those tools
   remain upstream-managed while Smith holds the repo-local orchestration center of gravity?
9. Which Ralph and SpecKit touchpoints belong in Smith MCP routing, workflow preparation, workpad
   generation, planning handoff, or chained execution, and which do not?

## Evidence Rules

- Separate verified facts from inference.
- Cite concrete files, directories, scripts, tools, or command outputs.
- Distinguish current truth from historical artifact.
- When a statement comes from a plan or historical note, label it as historical until re-verified.
- When Smith MCP, repo code, and docs disagree, treat the disagreement itself as a first-class
  finding.
- Do not collapse documentation validation, local static validation, and runtime validation into one
  bucket.

## Planning Requirements

Before major edits, produce one durable master plan note under `docs/plans/`.

That plan must include:

- the current-state model
- in-scope surfaces
- contradictions and stale-surface register
- target operating model for Smith MCP centralization
- prioritized execution slices
- validation plan per slice
- stop conditions
- open questions that still block execution

Your plan should be decision-complete enough that another session could continue without replaying
all of your investigation.

## Execution Requirements

After the investigation and master plan are strong enough, begin execution.

Execution expectations:

- prefer small, reversible, reviewable slices
- keep diffs scoped
- centralize or reconcile the highest-leverage workflow surfaces first
- do not silently expand scope into unrelated architecture work
- update durable notes as you go so a later session can resume cold

Priority order for execution:

1. central workflow contracts and Smith MCP routing surfaces
2. contradictions between repo docs and live control-plane behavior
3. duplicated or fragmented planning and workpad surfaces
4. Smith MCP integration points with Ralph orchestrator and SpecKit that should be repo-local glue
   rather than upstream modifications
5. GitHub, Symphony, and Linear integration points that prevent Smith MCP from holding the
   operational center of gravity
6. supporting doc or skill cleanup only after the target operating model is clear

## Expected Deliverables

By the end of the session, produce:

1. A durable master plan note in `docs/plans/`
2. A consolidated inventory of workflow and control-plane surfaces, either as a dedicated doc or as
   a clearly labeled section in the master plan
3. A contradictions and stale-surface register
4. A target operating model for Smith MCP centralization
5. An explicit Ralph-and-SpecKit integration model that keeps their upstream boundaries intact
6. A prioritized execution sequence with validation per slice
7. One or more implemented slices that materially move the repo toward the target model, unless
   blocked
8. A final status package that clearly states:

   - what was verified
   - what was inferred
   - what changed
   - what remains unresolved
   - what the next session should do next

## Validation Requirements

Use the narrowest validation that honestly proves each slice.

Examples:

- docs-only or workflow-only changes: markdown lint, structural checks, consistency checks, and
  exact evidence references
- code or script changes: targeted tests, builds, or tool-level validation appropriate to the
  affected surface
- control-plane claims: Smith MCP tool output or repo/script evidence
- runtime claims: explicit runtime evidence, not only documentation or code inspection

If validation cannot be run, say exactly what was not validated, why, and what remains to be
checked.

## Final Output Package

Your final response must be planning-and-execution-ready for a follow-up session.

Include these sections:

1. `Verified Current Model`
2. `Historical Artifacts Still Affecting Current Work`
3. `Contradictions and Fragmentation`
4. `Target Smith MCP Operating Model`
5. `Execution Plan and Completed Slices`
6. `Validation Evidence`
7. `Open Questions and Residual Risks`
8. `Recommended Next Actions`

Every section must cite concrete repo paths, tools, or runtime evidence.

## Guardrails

- Do not rewrite the whole operating model from first principles without repo evidence.
- Do not treat Smith MCP centralization as a reason to replace Linear, Symphony, or GitHub as their
  respective sources of truth.
- Do not treat Smith MCP centralization as a reason to modify Ralph or SpecKit upstream behavior
  unless a hard blocker proves the repo-local workflow glue is insufficient.
- Prefer repo-local workflow integration points, wrappers, prompts, plans, and orchestration
  contracts over upstream edits to Ralph or SpecKit.
- Do not leave the repository in a state where the new operating model exists only in your final
  chat response and not in durable repo artifacts.
- Do not claim Smith MCP is the central operating framework until the repo's durable workflow
  surfaces, control-plane behavior, and execution flow actually reflect that claim.

Your goal is to move Mister Smith toward a living, autonomous, well-integrated development system
where Smith MCP holds the operational center of gravity without depending on repeated manual
prompting.
