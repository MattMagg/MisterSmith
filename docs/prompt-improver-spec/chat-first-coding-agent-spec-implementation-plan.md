# Chat-First Coding-Agent Spec Implementation Plan

## Step 1: Source Inventory

### A. Source Truth

- Core objective: create a prompt for a new Codex session that produces a new SpecKit feature spec
  for making Mister Smith fully competitive with the best chat-first coding agents.
- Intended receiving agent: a new Codex session working inside `/Users/macmain/MisterSmith`.
- Deployment context: spec-only frontier planning in the Mister Smith repo, not implementation.
- Hard constraints:
  - use [$mister-smith-frontier-mandate](/Users/macmain/MisterSmith/.codex/skills/mister-smith-frontier-mandate/SKILL.md)
    before staging the feature
  - use the full
    [$speckit-specify](/Users/macmain/MisterSmith/.agents/skills/speckit-specify/SKILL.md)
    workflow in the correct order
  - do not implement code
  - keep the work product-side, not repo-workflow-side
- Required output contract: a prompt that can be pasted into a new Codex session and executed as a
  spec-generation request.
- Persona or tone: direct, explicit, repo-grounded, frontier-minded, and anti-imitation.
- Edge cases:
  - the goal is broad, so the new session must honestly bound the spec if one packet cannot carry
    the full ambition
  - the new session must distinguish currently landed session-first shell work from the new
    chat-first target state

### B. Example Audit

- No formal examples were provided.
- Repo-grounded reference materials exist:
  - `docs/direction.md`
  - `docs/current-state.md`
  - `docs/plans/2026-04-05-session-first-user-shell-pre-speckit-primer.md`
  - `docs/plans/2026-04-05-mister-smith-operational-cli-proposal.md`
  - `specs/029-session-first-user-shell/spec.md`
  - `specs/030-session-first-cli-shell/spec.md`

### C. Issue Inventory

- `"fully competitive with the best chat-first coding agents"`:
  - failure type: ambiguity
  - risk: the request is too broad unless the next session is told to bound the spec honestly.
- `"with consideration for $mister-smith-frontier-mandate"`:
  - failure type: missing definition
  - risk: the next session may reference the skill loosely instead of using Smith legitimacy tools
    first.
- `"using the full $speckit-specify workflow in the correct order"`:
  - failure type: weak handoff
  - risk: the next session may skip the actual SpecKit branch/script/template/checklist flow.

### D. Acceptance Rubric

- Outcome: the final prompt reliably produces a real SpecKit spec session.
- Fidelity: the frontier mandate, repo authority, and spec-only boundary are preserved.
- Clarity: the next session knows the exact reading order and execution order.
- Efficiency: the prompt stays compact and does not pre-write the spec.

### E. Ambiguity Ledger

- Should the new spec cover CLI only or shared CLI plus GUI:
  - non-blocking
  - resolved by assumption: let the next session decide the smallest honest bounded slice while
    keeping the full end-state explicit.
- Should the new spec reuse packet numbering assumptions:
  - non-blocking
  - resolved by assumption: the next session must let SpecKit create the next feature branch and
    not hardcode packet numbering.

### F. Variable / Interface Plan

- Variables needing explicit demarcation:
  - repo root
  - authority files to read
  - skills to use in order
  - feature description to feed into `speckit-specify`
- Contract format:
  - one paste-ready markdown prompt with ordered instructions
- Stage interfaces:
  - frontier legitimacy judgment first
  - repo authority read second
  - full `speckit-specify` flow third
  - final report limited to branch, spec path, checklist, and clarification state

### G. Prompt Filename

- `mister-smith-chat-first-coding-agent-spec.md`

## Step 2: Rewrite Plan

- Preserve:
  - the frontier ambition
  - the request for full `speckit-specify`
  - the desire for a new-session prompt rather than downstream execution in this session
- Tighten:
  - exact skill order
  - exact repo reading order
  - exact SpecKit flow order
  - the rule that the next session should create a bounded spec instead of trying to spec the
    entire universe
- Structure:
  - context
  - mandatory order of operations
  - feature description
  - constraints and non-goals
  - required report format
- Add:
  - explicit repo authority list
  - explicit `.specify/init-options.json` note that numbering is sequential
  - explicit requirement to use the create-new-feature script exactly once
- Remove:
  - vague wording about “consideration” with no execution behavior behind it

## Step 4: Critique & Revision Plan

### A. Must-Fix Issues

- The source ask does not tell the next session what to read first.
  - revision: add a precise repo-grounded reading order.
- The source ask does not force Smith legitimacy tools before spec work.
  - revision: require `evaluate_issue_legitimacy` and `classify_follow_up_work` first.
- The source ask does not protect against over-broad scope.
  - revision: tell the next session to define the smallest honest frontier slice if the full goal
    is too large for one spec.

### B. Expansion Needed

- Exact SpecKit command order.
- Clear feature description and non-goals.

### C. Compression Needed

- Avoid a long market comparison essay inside the prompt.
- Avoid implementation tactics.

### D. Constraint Preservation Check

- Preserve the frontier ambition: yes.
- Preserve the spec-only boundary: yes.
- Preserve the full `speckit-specify` workflow requirement: yes.
- Avoid doing the downstream spec work here: yes.
