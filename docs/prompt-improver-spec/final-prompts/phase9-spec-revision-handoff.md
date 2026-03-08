# Phase 9 Spec Revision — Handoff Prompt

You are a senior systems architect revising and supplementing specifications for the Mister Smith framework. You are NOT implementing code. You are producing spec artifacts that a coding agent will later execute using the SpecKit workflow.

---

## Mission

Two deliverables:

1. **Revise** the existing Phase 9 (LLM Provider Integration) spec set to incorporate three research findings
2. **Create** a new Phase 9.1 (Security Hardening) spec set based on crate audit findings

Both are informed by a completed 7-round research phase (2,000+ papers, 9 consolidated synthesis documents) and a 5-crate audit comparing existing implementation against those findings.

**You are editing and creating spec documents only. You are NOT writing Rust code.**

---

## Orientation — Read These First

### Project Context
- `CLAUDE.md` (project root) — full project overview, crate graph, tech stack, conventions
- `ROADMAP.md` — 9-phase build roadmap with gate criteria
- `.claude/projects/-Users-matthewmaggio-Mister-Smith/memory/MEMORY.md` — implementation history per phase, key patterns

### Canonical Architecture Specs
- `spec/core-architecture/type-definitions.md` — core types referenced everywhere
- `spec/data-management/message-schemas.md` — message formats (MessageEnvelope contract)
- `spec/data-management/agent-orchestration.md` — agent orchestration design; **explicitly flags MessageEnvelope security as a "CRITICAL GAP"**

### Research Corpus
The research corpus lives at `docs/research-output/` with navigation at `docs/research-output/CLAUDE.md`. **Start with the consolidated synthesis — these are the authoritative documents:**

| Document | Relevance |
|----------|-----------|
| `docs/research-output/consolidated/00-MASTER-FINDINGS.md` | **Read first.** Top 20 findings ranked by impact. Phase 9 applies #8, #9, #13. |
| `docs/research-output/consolidated/01-model-routing-and-cost-optimization.md` | Two-plane router, SLM-default, PRMs, budget enforcement — core Phase 9 |
| `docs/research-output/consolidated/06-streaming-architecture.md` | Dual-stream, backpressure, PrefillShare — Phase 9 streaming contract |
| `docs/research-output/consolidated/04-security-and-trust.md` | AgentSandbox, Auth Callouts, infectious jailbreaks — Phase 9.1 |
| `docs/RESEARCH_CHECKPOINT.md` | Confidence tiers, evidence gaps, what's not pursued |

You do not need to read every research file. The consolidated docs synthesize all 7 rounds. Read source files in `docs/research-output/research/` only if you need deeper evidence for a specific finding.

### Check for Existing Partial Implementation
Recent commits suggest some Phase 9 foundation work may already exist:
- `fe951e4` — Claude subscription provider with OAuth credential auth
- `075813a` — LLM provider foundation and OpenAI auth flows

**Before revising the spec, investigate what code already exists.** Search the codebase for any LLM-related modules, traits, or types. The spec should account for work already done — do not re-specify what's already built, and do not contradict existing implementation decisions unless they conflict with the research findings.

---

## Existing Phase 9 Spec Set

Lives at `specs/009-phase9-llm-provider-integration/`:

```
spec.md              — Feature specification (main document to revise)
plan.md              — Implementation plan
tasks.md             — Task breakdown
data-model.md        — Data model definitions
quickstart.md        — Getting started guide
research.md          — Research notes (PRE-DATES the 7-round research phase — update references)
analyze.md           — Cross-artifact analysis
contracts/
  agent-llm-bridge.md
  tool-calling-bridge.md
  model-provider.md
checklists/
  phase-7-5-readiness.md
```

**This spec was written BEFORE the research was completed.** It needs revision to incorporate three specific research findings.

---

## Research Findings to Incorporate (Phase 9)

### Finding #8 — Two-Plane Router (MUST incorporate)
- Separate microsecond data plane (NATS request-reply, ~50us) from control plane (JetStream KV watches)
- Budget enforcement via JetStream KV CAS (compare-and-swap)
- **Current gap**: Single unified transport handles both data and control with no architectural distinction
- Source: `consolidated/01-model-routing-and-cost-optimization.md`
- **Spec impact**: Add `MessagePlane` enum to `data-model.md`. Add plane-aware routing to `spec.md`. Update `contracts/model-provider.md` for routing metadata.

### Finding #9 — SLM-Default / LLM-Fallback Economics (MUST incorporate)
- Default routing policy: start with cheapest model capable of structured output, escalate on rejection
- 1-12B models with guided decoding match or exceed large models at 10-100x lower cost
- Source: `consolidated/01-model-routing-and-cost-optimization.md`
- **Spec impact**: Add routing policy types to `data-model.md`. Add SLM-default as the default routing strategy in `spec.md`.

### Finding #13 — Dual-Stream Formalization (MUST incorporate)
- Lossless semantic stream + best-effort UI stream running in parallel
- **Current gap**: Application must choose durable (JetStream) OR best-effort (pub/sub), cannot run both
- Source: `consolidated/06-streaming-architecture.md`
- **Spec impact**: Add `StreamClass` enum to `data-model.md`. Formalize dual-stream contract in `spec.md` and `contracts/agent-llm-bridge.md`.

### MessageEnvelope Additions (Phase 9 owns these fields)
The `MessageEnvelope` struct at `crates/mister-smith-transport/src/envelope.rs:24-64` needs two new fields for Phase 9:
- `plane: MessagePlane` — control vs data plane classification (Finding #8)
- `stream_class: StreamClass` — semantic vs UI stream classification (Finding #13)

*(Security fields — `signature`, `nonce`, `capability_token` — belong to Phase 9.1, not Phase 9.)*

---

## Crate Audit Findings — Phase 9.1 Security Hardening

Five crate audits compared existing implementation against research findings. The security-relevant findings below define the scope for a new Phase 9.1 spec.

### Transport — MessageEnvelope Security Fields (Phase 9.1 owns these)
Missing fields for message authentication:
- `signature: Option<String>` — HMAC-SHA256 for per-message authentication
- `nonce: Option<String>` — replay attack prevention
- `capability_token: Option<String>` — fine-grained capability delegation

### Security — Inter-Agent Message Authentication is Absent
- `SecureTransport<T>` at `crates/mister-smith-security/src/middleware/nats_mw.rs` enforces RBAC at subject level only (publish/subscribe permissions)
- No per-message signing or verification — agents can forge messages to peers if they have NATS publish permissions
- Research finding: 97% ASR in inter-agent hijacking attacks (COLM 2025)
- `AgentClaims.delegation_chain: Vec<String>` exists at `crates/mister-smith-security/src/jwt/claims.rs:46` but is never validated or propagated — either wire it up or remove the dead field
- No persistent/ephemeral agent separation (AgentSandbox pattern — reduces ASR from 58.8% to 4.34%)
- No NATS Auth Callout service for dynamic capability scoping
- **CVE-2025-30215** (JetStream Admin API flaw): NATS server version not pinned in deploy artifacts — pin to >= v2.11.1

### Persistence — No Data Sanitization
- `AgentRepository::get_state()` at `crates/mister-smith-persistence/src/repository/agent.rs:82-89` returns raw `Option<Value>` directly to callers
- No schema validation, bounds checking, or type validation before agent consumption
- Research finding: "Never pass raw JetStream KV/PostgreSQL retrievals into agent context without sanitization"

### Known Limitations (Documented, Not Addressed in Phase 9 or 9.1)
These are real gaps but belong to later phases:
- **HybridStateManager LWW-only conflict resolution** — CRDTs are Phase 13
- **Supervision is purely reactive** — predictive supervision is Phase 12
- **Agent composition is static** — dynamic topology is Phase 11

---

## Deliverables

### Deliverable 1: Revised Phase 9 Spec Set

Update the existing files at `specs/009-phase9-llm-provider-integration/` in place:

- **spec.md**: Incorporate two-plane router architecture, SLM-default routing, dual-stream contract, MessageEnvelope additions (`plane`, `stream_class`)
- **plan.md**: Update implementation plan to reflect revised architecture
- **tasks.md**: Regenerate task breakdown after spec revision
- **data-model.md**: Add `MessagePlane`, `StreamClass` enums, routing policy types, budget types
- **contracts/model-provider.md**: Ensure ModelProvider trait supports routing metadata (model tier, cost constraints)
- **contracts/tool-calling-bridge.md**: Ensure LLM function calling serialization covers OpenAI + Anthropic formats
- **contracts/agent-llm-bridge.md**: Add dual-stream handling, budget enforcement interface
- **research.md**: Update to reference the consolidated synthesis docs as the authoritative research source (replacing stale pre-research notes)
- **analyze.md**: Re-run cross-artifact analysis after all revisions

### Deliverable 2: New Phase 9.1 Spec Set

Create a new spec set at `specs/011-phase9.1-security-hardening/` using the same SpecKit artifact structure as existing phases:

```
specs/011-phase9.1-security-hardening/
  spec.md
  plan.md
  tasks.md
  data-model.md
  quickstart.md
  research.md          — Reference consolidated/04-security-and-trust.md
  contracts/
    message-signer.md  — MessageSigner trait, HMAC-SHA256, signature/nonce on MessageEnvelope
    auth-callout.md    — NATS Auth Callout service, dynamic per-request capability scoping
    state-validator.md — Sanitization layer between persistence retrieval and agent consumption
    agent-sandbox.md   — Persistent/ephemeral agent separation, I/O firewall
  checklists/
    requirements.md
```

This spec must address:
- **Inter-agent message authentication**: MessageSigner trait, HMAC-SHA256 signing/verification, signature + nonce fields
- **NATS Auth Callout service**: Dynamic per-request capability scoping based on behavioral trust
- **Data quarantine**: StateValidator trait — sanitization between persistence retrieval and agent consumption
- **AgentSandbox architecture**: Persistent/ephemeral agent separation, I/O firewall between agent boundaries
- **Infectious jailbreak defense**: Quarantine actors for cross-boundary data, COWPOX-inspired edge monitoring
- **CVE-2025-30215 mitigation**: NATS server version pinning >= v2.11.1, ACL audit for wildcard `>` and `$JS.>` permissions
- **`AgentClaims.delegation_chain`**: Either validate and propagate across agent boundaries, or remove the dead field

---

## SpecKit Workflow

These are slash commands available in Claude Code that invoke the SpecKit skill pipeline. Use them to produce spec artifacts.

**Execution order: Phase 9 first, then Phase 9.1.** Phase 9.1 references transport structural changes (MessageEnvelope fields) specified in Phase 9.

### For Phase 9 (revision of existing spec):
1. **`/specify`** — Revise the existing `spec.md` to incorporate findings #8, #9, #13
2. **`/clarify`** — Identify underspecified areas, encode answers back into spec
3. **`/plan`** — Regenerate `plan.md` for revised architecture
4. **`/tasks`** — Regenerate `tasks.md` with dependency-ordered task breakdown
5. **`/analyze`** — Cross-artifact consistency check across spec.md, plan.md, tasks.md

### For Phase 9.1 (new spec from scratch):
1. **`/specify`** — Create `spec.md` from the security audit findings above
2. **`/clarify`** — Identify underspecified areas
3. **`/plan`** — Generate `plan.md`
4. **`/tasks`** — Generate `tasks.md`
5. **`/analyze`** — Cross-artifact consistency check

---

## Scope Boundaries

### In Scope — Phase 9 Spec Revision
- Two-plane router architecture (Finding #8)
- SLM-default routing policy (Finding #9)
- Dual-stream formalization (Finding #13)
- ModelProvider trait (complete/stream/embed/capabilities)
- MockProvider (always available), Anthropic + OpenAI providers (feature-gated)
- Tool-calling bridge (ToolBus <> LLM function calling)
- Agent-LLM bridge (`llm` feature flag, `AgentRuntime::with_model()`)
- MessageEnvelope additions: `plane`, `stream_class`
- Budget enforcement via JetStream KV CAS

### In Scope — Phase 9.1 Spec Creation
- Inter-agent message authentication (MessageSigner, signature/nonce)
- NATS Auth Callout service
- Data quarantine / StateValidator
- AgentSandbox persistent/ephemeral separation
- Infectious jailbreak defense (quarantine actors)
- CVE-2025-30215 mitigation
- AgentClaims.delegation_chain resolution

### Explicitly Out of Scope (Phase 10+)
Do NOT include these in Phase 9 or 9.1 specs:

| Finding | Phase | Why Deferred |
|---------|-------|-------------|
| Step-level intelligence / PRMs (#2) | 10 | Depends on Phase 9 streaming infrastructure |
| Dynamic topology / MaAS (#1) | 11 | Depends on Phase 9 agent-LLM bridge |
| VCV discovery / HNSW (#14) | 11 | Depends on Phase 9 capability model |
| Predictive supervision (#4) | 12 | Independent of LLM integration |
| MAST failure taxonomy (#15) | 12 | Independent of LLM integration |
| CRDT coordination (#3) | 13 | Independent of LLM integration |
| MPST session types (#5) | 13 | Independent of LLM integration |
| Game-theoretic mechanism design (#20) | 14 | Research-stage concept |

### Explicitly NOT Changing (Correct As-Is)
- Supervision crate — architecturally correct, enhancements are Phase 12
- Agent composition model — static teams fine for Phase 9, dynamic topology is Phase 11
- Agent registry — string-match capabilities adequate for Phase 9, VCVs are Phase 11
- HybridStateManager conflict resolution — LWW adequate for Phase 9, CRDTs are Phase 13

---

## Established Patterns to Follow

When specifying new types and traits, follow patterns established in Phases 1-8:

| Pattern | Example | Apply To |
|---------|---------|----------|
| Error types in core | `SecurityError` in `mister-smith-core`, re-exported from domain crate | `LlmError` in core |
| Config in config crate | `PersistenceConfig` in `mister-smith-config` | `LlmConfig` in config crate |
| Feature flags for optional deps | `jwt`, `rbac`, `tls`, `audit` in security | `anthropic`, `openai` for providers |
| Orphan rule workaround | `from_jwt_error()` free function | Foreign type conversions in LLM crate |
| Forward-compatible enums | `#[non_exhaustive]` + `#[serde(other)]` | All new public enums (`MessagePlane`, `StreamClass`, etc.) |
| Cross-phase bridges | `HeartbeatBridge`, `SupervisionRecorder`, `SecurityBridge` | Agent-LLM bridge |
| Env-gated integration tests | `#[ignore]` + require `DATABASE_URL` / `NATS_URL` | `OPENAI_API_KEY`, `ANTHROPIC_API_KEY` |

---

## Governing Principles

- **"No Rust implementation exists" is never a valid reason to dismiss or defer.** The team builds Rust implementations. 19 crates, 983 tests, 8 phases built in 2 days with agent assistance. Dismiss only when mathematically inferior or strategically wrong.
- **Frontier-first**: Build the framework others will copy. Capabilities absent from all competing frameworks are highest priority.
- **Model-agnostic**: Works with ANY LLM. Provider-specific code goes behind feature flags. No Claude-specific or OpenAI-specific architecture.
- **Evidence-grounded**: Cite the relevant consolidated research doc and finding number when making architectural decisions.

---

## Completion Checklist

The spec revision is complete when:

- [ ] Existing Phase 9 `spec.md` incorporates findings #8, #9, #13 with research citations
- [ ] Existing Phase 9 `research.md` updated to reference consolidated synthesis docs
- [ ] Phase 9 `data-model.md` includes `MessagePlane`, `StreamClass`, routing policy, and budget types
- [ ] Phase 9 `plan.md` and `tasks.md` regenerated via `/plan` and `/tasks`
- [ ] Phase 9 contracts updated (model-provider, tool-calling-bridge, agent-llm-bridge)
- [ ] Phase 9 `/analyze` passes with no critical inconsistencies
- [ ] Phase 9.1 spec set created at `specs/011-phase9.1-security-hardening/`
- [ ] Phase 9.1 covers all 7 security audit findings (message auth, Auth Callouts, sanitization, AgentSandbox, jailbreak defense, CVE mitigation, delegation chain)
- [ ] Phase 9.1 `/analyze` passes with no critical inconsistencies
- [ ] No Phase 10+ items leaked into either spec
- [ ] Existing partial implementation (commits `fe951e4`, `075813a`) accounted for — spec doesn't contradict what's already built
