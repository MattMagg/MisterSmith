Work in `/Users/macmain/MisterSmith`.

This is spec-only work. Do not implement code, do not generate tasks, and do not widen into repo
workflow execution. Your job is to create a new SpecKit feature specification for the next major
product slice.

Use these skills in this exact order:

1. [$mister-smith-frontier-mandate](/Users/macmain/MisterSmith/.codex/skills/mister-smith-frontier-mandate/SKILL.md)
2. [$speckit-specify](/Users/macmain/MisterSmith/.agents/skills/speckit-specify/SKILL.md)

Before any spec work, use Smith MCP first for the frontier-legitimacy check:

- `evaluate_issue_legitimacy`
- `classify_follow_up_work`

Treat that as a legitimacy and scope check, not as a blocker if the work is judged legitimate but
still triage-stage.

Read repo authority in this order before you run the SpecKit flow:

1. `AGENTS.md`
2. `docs/current-state.md`
3. `docs/direction.md`
4. `docs/plans/2026-04-05-session-first-user-shell-pre-speckit-primer.md`
5. `docs/plans/2026-04-05-mister-smith-operational-cli-proposal.md`
6. `specs/029-session-first-user-shell/spec.md`
7. `specs/030-session-first-cli-shell/spec.md`
8. `.specify/init-options.json`
9. `.specify/templates/spec-template.md`

Then run the full `speckit-specify` workflow in the correct order.

Required execution order:

1. Check for `.specify/extensions.yml` and handle any `before_specify` hooks exactly as the
   skill requires.
2. Read `.specify/init-options.json`.
   The repo uses sequential numbering unless the file says otherwise.
3. Generate a concise short name for the new feature.
4. Run `.specify/scripts/bash/create-new-feature.sh` exactly once with `--json` and
   `--short-name`.
   Do not pass `--number`.
5. Use the JSON output from that script as the source of truth for branch name, feature dir, and
   spec path.
6. Load `.specify/templates/spec-template.md`.
7. Write the full spec using the template structure and the `speckit-specify` quality rules.
8. Create `FEATURE_DIR/checklists/requirements.md`.
9. Validate the spec against the checklist and iterate up to 3 times.
10. If clarification is still required, keep it to at most 3 high-impact questions and present
    them together.
11. Report completion with branch name, spec path, checklist status, and readiness for the next
    phase.
12. After completion, check for any `after_specify` hooks and handle them exactly as the skill
    requires.

Feature description to feed into `speckit-specify`:

Create a new frontier product spec for making Mister Smith fully competitive with the best
chat-first coding agents, while preserving Mister Smith's own standard-setting identity instead of
copying the market.

The spec must start from current repo truth:

- today the session shell exists, but the product still feels workflow-first
- a user message still reads as “submit work” more than “stay in a live coding-agent conversation”
- durable sessions, resume flows, recent-session browsing, and in-session controls already exist
- the new work must build on those foundations instead of ignoring them

The desired end state is a product that feels like a true multi-turn chat-first coding agent:

- open the shell and immediately feel like you are in a live coding-agent conversation
- keep talking inside one session naturally
- make the assistant feel live and interactive rather than like a detached job launcher
- keep model, permissions, config, status, and MCP controls inside the session
- preserve durable session identity, resume, continuity, runtime truth, and supervised autonomy
- keep workflow, runtime, proof, and admin machinery as supporting substrate rather than the main
  user experience

Frontier mandate requirements:

- do not frame the target as “match Claude Code or Codex”
- benchmark chat-first coding agents and then exceed them
- prefer designs with long-term leverage in coordination, execution, supervision, memory,
  streaming, routing, reliability, observability, state, and distributed behavior
- preserve strong execution boundaries and honest proof claims
- stay product-side and do not collapse Linear, Symphony, Ralph, SpecKit, or repo workflow into
  the shipped product

Scope rules for the new spec:

- if the full ambition is too large for one honest spec, define the highest-leverage bounded slice
  that is clearly on the critical path to that end state
- make the end-state ambition explicit, but keep the actual spec bounded and testable
- build from the session-first shell work already explored in packets 029 and 030
- do not blindly reuse their scope if a larger chat-first product slice needs a different bounded
  framing

Non-goals:

- do not implement code
- do not write tasks or plans beyond what `speckit-specify` itself requires
- do not widen into repo workflow automation
- do not reduce the feature to just minor CLI polish
- do not propose generic imitation of existing agent tools without Mister Smith-specific leverage

When you finish, report only:

- legitimacy result
- chosen short name
- created branch name
- spec file path
- checklist file path
- whether clarifications are still needed
