# Ralph Task Prompt

Mode: prep

Goal:
Build the full SpecKit artifact set for Phase 9 — LLM Provider Integration under
`./specs/009-phase9-llm-provider-integration/`, ending at `/speckit.analyze`
without starting code implementation.

Context:
- Use the repo-native Speckit flow.
- Use `./spec/` as canonical architecture guidance.
- Use `./specs/` as the active SpecKit artifact directory.
- Ralph is only the loop runner here; do not substitute a Ralph-defined workflow
  for the existing SpecKit command chain.
- The workflow must be grounded in these repo sources:
  - `ROADMAP.md` Phase 9 (`LLM Provider Integration`)
  - `docs/2026-03-05-architectural-grounding-audit.md`
  - `docs/2026-03-05-implementation-deviation-report.md`
  - `docs/plans/2026-03-05-llm-provider-integration-design.md`
- Treat `specs/007-phase7-agent-system/spec.md`, `plan.md`, and `tasks.md` as the
  current baseline for agent-system capabilities and dependencies.
- Do not reuse the stale `specs/008-agent-system/` path for this work.
- Resulting SpecKit artifacts must include explicit architecture citations to:
  - `spec/data-management/agent-orchestration.md` §10.4
  - `spec/data-management/message-schemas.md` §5
  - `spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md` §15
  - `spec/core-architecture/type-definitions.md`
  - `spec/core-architecture/async-patterns.md` (agent-as-tool and ToolBus patterns)
  - `spec/core-architecture/coding-standards.md` (tool permissions, testing, errors)
- Use `README.md`, `ROADMAP.md`, `VALIDATION_REPORT.md`, `AGENTS.md`, and `CLAUDE.md`
  as supporting repo context.

Workflow Requirements:
- Required Speckit order:
  `/speckit.constitution -> /speckit.specify -> /speckit.clarify -> /speckit.plan -> /speckit.tasks -> /speckit.analyze -> /speckit.implement`
- `/speckit.checklist` is optional support for requirements quality; it does not
  replace `/speckit.analyze`.
- Never skip `/speckit.analyze` before `/speckit.implement`.
- Start with `/speckit.constitution`, but only amend the constitution if this
  Phase 9 work reveals a real governance gap. Do not invent constitution changes.
- Stop after `/speckit.analyze`; do not begin implementation or code changes.

Phase 9 Scope:
- In scope:
  - New `mister-smith-llm` crate
  - `ModelProvider` trait
  - Unified completion, streaming, embedding, and tool-calling types
  - `MockProvider`
  - `AnthropicProvider`
  - `OpenAiProvider`
  - `mister-smith-agents` `llm` feature
  - Agent–LLM bridge for Planner, Critic, and Executor roles
  - `ToolBus::to_tool_definitions()` and `ToolBus::execute_tool_call()`
  - Gate 9 criteria from the roadmap and approved design
- Out of scope and explicitly deferred:
  - Hook event system / `llm.hooks.*` subjects
  - `LlmTaskOutputParser` regex routing
  - Neural/AI Operations domain
  - Prompt-engineering framework
  - RAG pipeline
  - Guardrails / safety layer
  - Non-MVP providers beyond Anthropic and OpenAI

Phase 7.5 Dependencies And Blockers:
- Keep these visible as prerequisites, dependencies, or blockers.
- Do not fold them into main Phase 9 scope:
  - Security integration for agent messaging, tool permissions, and audit logging
  - Router balancing (`round-robin`, `least-loaded`)
  - Memory metadata, timestamps, versions, and access counts
  - Heartbeat receiver and failure detection
  - Supervisor delegation to the Phase 3 supervision system
  - Priority mailbox wiring

Artifact Expectations:
- `spec.md` must include Gate 9 success criteria:
  Planner calls a real LLM, gets structured subtask decomposition, the
  Orchestrator assigns subtasks to Workers, and the same flow works with both
  Anthropic and OpenAI.
- `plan.md` must preserve the approved Phase 9 subphases `9.1` through `9.5`.
- `tasks.md` must keep Phase 7.5 hardening visible without redefining it as the
  main implementation scope for Phase 9.
- Missing architecture citations are defects, not optional polish.

Analyze Requirements:
- `/speckit.analyze` must explicitly check:
  - Phase 9 scope matches the roadmap and approved design document
  - Deferred work has not been silently absorbed into the feature scope
  - Phase 7.5 hardening remains prerequisite/dependency work rather than merged
    Phase 9 implementation scope
  - Missing architecture references or traceability gaps are reported as blockers

Mode Semantics:
- `prep`: stop after `/speckit.analyze`
- `full`: continue through `/speckit.implement`
- `implement`: use existing SpecKit artifacts; validate/analyze first if needed

Definition of Done:
- `prep`: `./specs/009-phase9-llm-provider-integration/` exists with current
  `spec.md`, `plan.md`, and `tasks.md`, the artifacts are grounded in the cited
  architecture documents, and `/speckit.analyze` has been run with blockers
  surfaced or readiness cleared.
- `full`: `prep` is satisfied and implementation is completed with verification.
- `implement`: the target implementation task is completed without violating the
  current spec/plan/tasks/analyze state.
