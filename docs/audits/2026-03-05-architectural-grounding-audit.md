# Mister Smith: Architectural Grounding Audit Report

**Date**: 2026-03-05
**Scope**: Phase 3 onward, with Phase 7 as focal point
**Branch**: `008-agent-system` (PR #110 open, under review)
**Author**: Claude Code (automated audit at user request)

---

## Executive Summary

This report examines whether the Mister Smith implementation from Phase 3 onward is properly grounded in the canonical architecture documents (`spec/`), the roadmap (`ROADMAP.md`), and the phase-specific implementation specs (`specs/`). The audit was triggered by confusion over Phase 7 naming, spec directory numbering, and the discovery that the Phase 7 implementation spec contains zero references to architecture documents.

**Determination**: This moment is a **checkpoint, not a crisis**. The project is not at a roadblock. The core infrastructure (Phases 1-7) is architecturally sound at the implementation level. What has degraded is *traceability* — the documentation chain from architecture specs through implementation specs to code. This creates real risk for the project's next stages but does not invalidate work completed so far.

**Root cause**: The SpecKit pipeline (`/speckit.specify` -> `/speckit.plan` -> `/speckit.implement`) progressively detached from the canonical architecture documents starting at Phase 4. This is a *process gap*, not an *implementation gap*. The code itself generally implements the architecture correctly because the coding agent had access to the full codebase (including `spec/` files) during implementation, even though the SpecKit specs didn't formally reference them.

---

## 1. The Three Document Layers

Understanding Mister Smith requires distinguishing three document layers that should form a traceable chain:

| Layer | Location | Purpose | Created By |
|-------|----------|---------|-----------|
| **Architecture specs** | `spec/` (65+ files, 8 domains) | Define the system contract: types, patterns, interfaces, message schemas | Manual specification work |
| **Roadmap** | `ROADMAP.md` | Phase ordering, gate criteria, dependency graph, `spec/` references | Manual planning |
| **Implementation specs** | `specs/001-*` through `specs/007-*` | Per-phase feature specs, plans, tasks for SpecKit-driven implementation | SpecKit pipeline (AI-assisted) |

The intended flow: Architecture specs define *what* → Roadmap defines *when* → Implementation specs define *how* → Code implements.

---

## 2. Grounding Assessment by Phase

### Phase 1: Foundation — STRONG grounding

- `specs/001-phase1-foundation/spec.md` contains explicit `rg` validation commands referencing `spec/core-architecture/type-definitions.md`, `spec/data-management/`, `spec/testing/test-schemas.md`, `spec/transport/nats-transport.md`
- Implementation directly validates types against canonical sources
- **Classification**: Aligned implementation

### Phase 2: Runtime & Async — STRONG grounding

- `specs/002-phase2-runtime-async/spec.md` defines 5 acceptance scenarios with `rg` validation commands referencing canonical docs
- Plan lists primary dependencies from `spec/core-architecture/` and `spec/operations/`
- **Classification**: Aligned implementation

### Phase 3: Actor & Supervision — ADEQUATE grounding

- `specs/003-phase3-actor-supervision/spec.md` line 6 explicitly lists 5 architecture sources:
  `spec/core-architecture/async-patterns.md, supervision-trees.md, supervision-and-events.md, type-definitions.md, component-architecture.md`
- Line 141 references `spec/core-architecture/type-definitions.md` for canonical types
- Code implements OneForOne/OneForAll/RestForOne restart strategies matching `spec/core-architecture/supervision-tree-specifications.md`
- Actor trait uses associated types (Message, State, Error, Response) consistent with architecture vision
- **Grounding quality**: Explicit in header, one body reference, implicit in code
- **Classification**: Aligned implementation with weakening documentation

### Phase 4: Transport & Messaging — WEAK grounding

- `specs/004-phase4-transport-messaging/spec.md` line 6 Input field says: `"Phase 4: Transport & Messaging — NATS transport, HTTP/gRPC endpoints, message envelope and serialization"` — **no architecture doc references**
- Single reference to `spec/mcp_integration_analysis.md` in a clarification Q&A (line 214)
- No references to `spec/transport/nats-transport.md`, `spec/transport/transport-layer-specifications.md`, or `spec/data-management/message-schemas.md`
- **However**: MessageEnvelope implementation includes correlation IDs, priority, schema version, source/target agent IDs — all patterns from the architecture docs. The code is correct; the spec just doesn't cite its sources.
- **Classification**: Documentation/naming confusion — code is grounded, spec is not

### Phase 5: Security — ABSENT grounding in spec

- `specs/005-phase5-security/spec.md` contains **zero** references to `spec/security/` documents
- Input field is a bare description string, not architecture doc references
- Plan.md mentions `spec/security/` only in Constitution Check table (metadata, not body)
- **However**: Implementation correctly produces JWT, RBAC, TLS/mTLS, audit logging consistent with `spec/security/` architecture
- **Classification**: Documentation/naming confusion — same pattern as Phase 4

### Phase 6: Persistence & State — ABSENT grounding in spec

- `specs/006-phase6-persistence-state/spec.md` contains **zero** references to `spec/data-management/` persistence docs
- Input field is a bare description string
- Implementation correctly produces dual-store (PostgreSQL + JetStream KV) pattern from architecture
- **Classification**: Documentation/naming confusion — same pattern as Phases 4-5

### Phase 7: Agent System — ABSENT grounding in spec, PARTIAL implementation alignment

- `specs/007-phase7-agent-system/spec.md` contains **zero** references to any `spec/` document
- Not referenced: `spec/data-management/agent-orchestration.md` (the 3400-line primary agent spec)
- Not referenced: `spec/data-management/agent-lifecycle.md` (2400-line lifecycle spec)
- Not referenced: `spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md` (15 agent domains)
- Not referenced: `spec/data-management/message-schemas.md` Section 5 (LLM backend hook schemas)
- Agent role implementations are **functional but thin** — they process messages with real logic (Supervisor tracks children, Worker executes tasks, etc.) but lack the depth described in architecture docs
- Gate 7 test validates the end-to-end orchestration flow as specified in ROADMAP
- **Classification**: Documentation gap (spec), plus implementation that is correct-but-shallow relative to architecture vision

---

## 3. The Roadmap as Hidden Bridge

A critical finding: **the ROADMAP itself is well-grounded**. Every phase in `ROADMAP.md` explicitly references `spec/` architecture documents:

| Phase | Architecture References in ROADMAP |
|-------|-----------------------------------|
| Phase 3 | `spec/core-architecture/async-patterns.md`, `supervision-tree-specifications.md`, `type-definitions.md` |
| Phase 4 | `spec/transport/nats-transport.md`, `spec/transport/transport-layer-specifications.md`, `spec/data-management/message-schemas.md` |
| Phase 5 | `spec/security/authentication-authorization.md`, `spec/security/security-patterns.md`, `spec/security/tls-specifications.md` |
| Phase 6 | `spec/data-management/persistence-operations.md`, `spec/data-management/data-integration-patterns.md`, `spec/data-management/jetstream-kv.md` |
| Phase 7 | `spec/data-management/agent-lifecycle.md`, `agent-orchestration.md`, `agent-communication.md`, `spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md` |

The ROADMAP bridges architecture → phases. The SpecKit pipeline was supposed to bridge ROADMAP → implementation. It did so for Phases 1-2 (where the Input field cited architecture docs). From Phase 3 onward, the SpecKit Input field shifted from architecture doc references to bare description strings. The bridge was lost in the *SpecKit generation step*, not in the architecture or the code.

---

## 4. Classification of Issues

### Issue 1: SpecKit Input Degradation (Phases 4-7)
- **Type**: Process gap / documentation confusion
- **Severity**: Medium — affects traceability, not correctness
- **Evidence**: Phase 1-3 specs cite architecture docs in Input field; Phases 4-7 use bare description strings
- **Impact**: Future developers cannot trace implementation decisions back to architecture without reading the ROADMAP as an intermediary
- **Recommendation**: Document clearly; fix forward in Phase 8+

### Issue 2: Phase 7 Agent Roles Are Thin
- **Type**: Intentional scope boundary
- **Severity**: Low for framework layer, High for competitive positioning
- **Evidence**: Planner stores goal strings, Critic increments counters, Worker echoes input. Architecture docs describe richer behavior involving LLM integration, prompt templates, evaluation criteria
- **However**: The architecture docs describe *application-level behavior* that depends on LLM backends. The framework layer correctly provides the Actor-based containers and pluggable traits (TaskDecomposer, ResultAggregator) where this behavior would be injected. Making roles "thin" at the framework layer is architecturally appropriate for a model-agnostic library.
- **Classification**: Justified evolution — framework provides extension points; application layer fills them

### Issue 3: LLM Integration Designed but Not Roadmapped
- **Type**: Roadmap/spec gap
- **Severity**: High for project trajectory
- **Evidence**: `spec/data-management/message-schemas.md` Section 5 defines full JSON schemas for LLM hook events/responses. `spec/data-management/agent-orchestration.md` Section 10.4 defines `LlmTaskOutputParser` and parallel execution patterns. `spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md` defines Neural/AI Operations domain. None of this is in the ROADMAP as a numbered phase.
- **Classification**: Roadmap omission — the architecture vision includes LLM integration, but the implementation roadmap stops before reaching it

### Issue 4: Spec Directory Numbering Mismatch
- **Type**: Naming/documentation confusion
- **Severity**: Low (now fixed)
- **Evidence**: `specs/008-agent-system/` was Phase 7 in the roadmap. Renamed to `specs/007-phase7-agent-system/` during this session.
- **Classification**: Naming confusion — resolved

### Issue 5: Obsolete Claude CLI Integration Files
- **Type**: Documentation debt
- **Severity**: Low
- **Evidence**: 5 files (~2200 lines) in `spec/core-architecture/` and `spec/research/` marked "OBSOLETE — FOR ARCHIVAL ONLY" but still present
- **Classification**: Neutral divergence — marked obsolete, should be archived

### Issue 6: `spec/` vs `specs/` Confusion
- **Type**: Naming/documentation confusion
- **Severity**: Medium — actively confused the project owner
- **Evidence**: Two directories with near-identical names serving different purposes. `spec/` = architecture. `specs/` = implementation. No documentation explains the distinction.
- **Classification**: Documentation confusion — needs explicit explanation in README or CLAUDE.md

---

## 5. What This Means for Mister Smith's Trajectory

### Strengths Confirmed

1. **The architecture is coherent**. The `spec/` documents define a well-thought-out system with clear type definitions, message contracts, supervision patterns, and extension points. This is a genuine differentiator — most competing frameworks (CrewAI, LangChain) lack this level of architectural rigor.

2. **The implementation is technically sound**. 950+ tests pass. Clippy is clean. The actor system, supervision trees, transport layer, security, persistence, and agent orchestration all work. Gate 7 validates the end-to-end flow.

3. **The ROADMAP bridges architecture to implementation**. It correctly references `spec/` documents for every phase and defines clear gate criteria.

4. **The pluggable trait design is correct for model-agnostic goals**. `TaskDecomposer`, `ResultAggregator`, and the Actor trait's associated types provide clean extension points for any LLM backend without coupling the framework.

### Gaps to Address

1. **No application layer exists or is planned**. The framework provides orchestration infrastructure. There is no crate, binary, or roadmap phase that wires it to actual LLM backends. The `spec/` documents describe LLM integration patterns extensively, but no phase implements them.

2. **Traceability is broken from Phase 4 onward**. The SpecKit specs don't reference their architecture sources. This makes it harder to verify alignment, onboard contributors, or audit changes.

3. **The `spec/` documents contain forward-looking design that is unimplemented and unscheduled**. Specifically: LLM hook schemas, parallel LLM task coordination, Neural/AI Operations domain, TrainableAgent trait. This creates confusion about what's "done" vs "designed" vs "aspirational."

---

## 6. Competitive Context

Mister Smith's architecture targets the same space as OpenAI Agents SDK, Google ADK, CrewAI, LangChain/LangGraph, and Claude Agent SDK. Its differentiation (Rust performance, NATS messaging, OTP-style supervision, strong multi-agent orchestration) is real but currently unrealized at the application layer.

The framework layer (Phases 1-7) provides infrastructure these competitors lack:
- **Supervision trees with restart strategies** — no Python framework has this
- **NATS/JetStream durable messaging** — genuine advantage over in-process message passing
- **Actor model with bounded mailboxes** — backpressure and isolation guarantees
- **Type-safe agent roles** — Rust's type system enforces contracts at compile time

What competitors provide that Mister Smith does not:
- **A runnable system** — every competitor ships something you can `pip install` and run
- **Model provider integrations** — Claude, GPT, Gemini, etc.
- **Tool calling** — structured function calling with model APIs
- **Higher-level abstractions** — workflows, chains, graphs, crews

The critical path to competitive viability is an application layer that turns the framework into something users can run.

---

## 7. Next-Steps Posture

### Correct Now

1. **Document the `spec/` vs `specs/` distinction** — Add a clear note to `CLAUDE.md` and `README.md` explaining that `spec/` contains architecture specifications and `specs/` contains SpecKit implementation artifacts. This was the primary source of confusion in this session.

2. **Archive obsolete Claude CLI files** — Move the 5 files marked "OBSOLETE" to `archive/claude-cli-research/`. They add noise and confusion.

### Document Clearly But Don't Change Yet

3. **Note the SpecKit grounding gap** — Record in project memory that Phases 4-7 implementation specs lack architecture doc references. The code is correct; the docs need backfilling if/when someone revisits these specs.

4. **Note the LLM integration gap** — Record that `spec/` contains extensive LLM backend integration designs (hook schemas, parallel execution, Neural/AI Ops domain) that are not in the ROADMAP and not implemented.

### Defer for Later Planning

5. **Application layer design** — The brainstorming session (interrupted by this audit) should resume. An application layer connecting the framework to real LLM backends is the critical path to Mister Smith being usable. This needs proper spec work, not ad-hoc implementation.

6. **Phase 8 scope review** — Phase 8 (Operations) produces a "main binary entry point" per the ROADMAP. Evaluate whether LLM provider integration should be folded into Phase 8 alongside process management, or whether it warrants a new Phase 9.

### Requires Explicit Roadmap Update

7. **Add an LLM integration phase to the ROADMAP** — The architecture docs describe it. The ROADMAP does not schedule it. This is the highest-impact gap. It needs a formal phase with gate criteria, not an afterthought.

---

## 8. Summary Determination

| Question | Answer |
|----------|--------|
| Is this a major shift? | No — the architecture is sound, the implementation is correct |
| Is this a roadblock? | No — but it's a planning checkpoint |
| What caused the confusion? | Naming (`spec/` vs `specs/`, `008` vs Phase 7), plus degrading traceability in SpecKit pipeline |
| Is there implementation drift? | Minimal — code follows architecture; documentation doesn't cite it |
| Is Phase 3 where drift started? | Partially — Phase 3 spec still cited sources but plan didn't; Phase 4 is where citations disappeared from specs |
| What's the real risk? | Not having an application layer, not the traceability gap |
| What should happen next? | Finish the brainstorm for the application layer; update the ROADMAP; fix documentation forward |

---

*This report serves as project memory and planning context. It is not an implementation plan.*
