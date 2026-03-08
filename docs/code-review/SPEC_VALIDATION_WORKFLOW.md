# Spec Validation Workflow

Systematic re-evaluation of the Mister Smith framework specifications using Claude Code agent teams in parallel batches, with Rube MCP as the external reference layer.

## Goal

Validate every spec file against current library APIs, Rust best practices, and internal consistency. For each file: confirm accuracy, flag outdated content, fill gaps, and improve where needed. Produce a concrete diff or patch per file, not just a report.

## External Reference Tools (via Rube MCP)

Each agent has access to these through Rube:

| Tool | Purpose | When to Use |
|------|---------|-------------|
| **Context7** (`RESOLVE_LIBRARY_ID` → `QUERY_DOCS`) | Current library docs and code examples | Verify API signatures, check version-specific behavior, confirm patterns |
| **Tavily** (web search) | Current best practices, changelogs, migration guides | Check for breaking changes between spec versions and current versions, find idiomatic patterns |
| **GitHub** | Source code of actual crate implementations | Verify struct definitions, trait implementations, actual API surface when Context7 is insufficient |

### Key Libraries to Validate Against

| Library | Spec Version | Current | Context7 ID | Priority |
|---------|-------------|---------|-------------|----------|
| async-nats | 0.34 | 0.46.0 | Resolve: `async-nats` | Critical — 12 version gap |
| Tokio | 1.38 | Check current | Resolve: `tokio` | High — runtime foundation |
| Axum | 0.8 | Check current | Resolve: `axum` | Medium |
| Tonic | 0.11 | Check current | Resolve: `tonic` | Medium |
| sqlx / tokio-postgres | unspecified | Check current | Resolve as needed | Medium |
| NATS server | unspecified | 2.12.4 (local Docker) | N/A | High — JetStream API changes |

## Agent Instructions (shared across all batches)

Every agent receives this as its system prompt prefix:

```
You are validating Mister Smith framework specifications against current
Rust ecosystem state. For each file assigned to you:

1. READ the spec file completely
2. IDENTIFY every external dependency reference (crate versions, API calls,
   struct/trait names, configuration formats)
3. VERIFY against current sources using this priority order:
   a. Context7 — resolve library ID first, then query for specific APIs
   b. Tavily web search — for changelogs, migration guides, breaking changes
   c. GitHub source — for struct definitions and trait signatures when needed
4. ASSESS internal consistency:
   - Cross-reference against high-impact files (system-architecture.md,
     type-definitions.md, message-schemas.md)
   - Check terminology matches SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md
   - Verify message formats align with core-message-schemas.md
5. APPLY changes directly to the spec files:
   - Fix outdated API references, version numbers, code examples
   - Add missing details where gaps are found
   - Remove or correct inaccurate content
   - For changes you're uncertain about, add an inline HTML comment:
     <!-- [UNVERIFIED] description of what needs human review -->
6. PRODUCE a summary per file after editing:
   - STATUS: validated (no changes) | updated | needs-review
   - CHANGES MADE: list of edits applied
   - UNVERIFIED: anything marked for human review

Do NOT generate vague recommendations. Make the edits. If you can't
verify something, make your best edit and mark it [UNVERIFIED] with
the verification path you attempted.
```

## Batch Execution

### Batch 0: Version Discovery (prerequisite, single agent)

**Purpose**: Resolve current versions for all referenced crates before domain agents start. This avoids 20+ agents making redundant Context7/Tavily calls for the same version lookups.

**Agent task**:
```
Using Rube MCP tools, resolve current stable versions for:
- tokio, async-nats, axum, tonic, hyper, tower
- sqlx, tokio-postgres, deadpool
- serde, serde_json
- tracing, tracing-subscriber, metrics
- jsonwebtoken, rustls, ring, rcgen
- nats-server (Docker image)
- Any other crates referenced in spec/core-architecture/dependency-specifications.md

For each crate:
1. RUBE_SEARCH_TOOLS → Context7 → RESOLVE_LIBRARY_ID → QUERY_DOCS
   "What is the latest stable version and key API changes since [spec_version]?"
2. Tavily: "[crate_name] changelog breaking changes [spec_version] to latest"

Output a VERSION_REFERENCE.md table:
| Crate | Spec Version | Current Version | Breaking Changes | Migration Notes |
```

**Output**: `VERSION_REFERENCE.md` — consumed by all subsequent batches.

---

### Batch 1: Core Architecture (3 parallel agents)

Must complete before Batches 2-4, since all domains reference core architecture.

**Agent 1A — System Foundation** (7 files):
| File | Validation Focus |
|------|-----------------|
| `system-architecture.md` | Tokio runtime config, worker thread model, async patterns vs current Tokio API |
| `component-architecture.md` | Component hierarchy, trait definitions, ownership model |
| `tokio-runtime.md` | Runtime builder API, task spawning, shutdown — verify against current Tokio |
| `async-patterns.md` | Select!, join!, spawn patterns — verify idioms are current |
| `async-patterns-detailed.md` | Advanced patterns — cancellation safety, structured concurrency |
| `runtime-and-errors.md` | Error types, thiserror/anyhow usage, Result patterns |
| `dependency-specifications.md` | All crate versions — primary target for VERSION_REFERENCE.md updates |

**Agent 1B — Integration & Types** (8 files):
| File | Validation Focus |
|------|-----------------|
| `type-definitions.md` | Core types — verify Rust type system usage, trait bounds |
| `module-organization-type-system.md` | Module tree, visibility, re-exports |
| `integration-patterns.md` | Inter-component communication patterns |
| `integration-contracts.md` | Trait-based contracts between subsystems |
| `integration-implementation.md` | Concrete implementations of contracts |
| `system-integration.md` | End-to-end integration architecture |
| `coding-standards.md` | Rust 2021 edition standards, clippy lints, formatting |
| `implementation-config.md` | Configuration patterns — verify against current config crate ecosystem |

**Agent 1C — Supervision & Implementation** (6 files):
| File | Validation Focus |
|------|-----------------|
| `supervision-trees.md` | Core supervision model — verify Erlang/OTP parallels are accurate for Rust |
| `supervision-and-events.md` | Event-driven supervision, restart strategies |
| `monitoring-and-health.md` | Health check patterns, liveness/readiness probes |
| `implementation-guidelines.md` | Development workflow, testing approach |
| `claude-cli-integration.md` | Claude Code integration patterns — verify against current Claude API |
| `claude-code-cli-technical-analysis.md` | Technical feasibility of CLI integration |

---

### Batch 2: Data & Messaging (2 parallel agents)

Depends on: Batch 1 complete (types and message formats validated first).

**Agent 2A — Agent System** (8 files):
| File | Validation Focus |
|------|-----------------|
| `agent-orchestration.md` | Orchestration patterns, team composition, task decomposition |
| `agent-lifecycle.md` | State machine, spawn/terminate, restart policies |
| `agent-operations.md` | Runtime agent operations, commands, status reporting |
| `agent-communication.md` | Inter-agent messaging patterns |
| `agent-integration.md` | Agent ↔ external system integration |
| `connection-management.md` | Connection pooling, reconnection, health monitoring |
| `cross-reference-index.md` | Validate all cross-references still resolve after Batch 1 changes |
| `data-integration-patterns.md` | Data flow between agents and storage |

**Agent 2B — Messages & Persistence** (11 files):
| File | Validation Focus |
|------|-----------------|
| `message-schemas.md` | Core message types — verify serde derive patterns are current |
| `core-message-schemas.md` | System-level messages — check for duplicates with above |
| `system-message-schemas.md` | Control plane messages |
| `workflow-message-schemas.md` | Workflow-specific messages |
| `message-framework.md` | Message routing, serialization, validation patterns |
| `data-persistence.md` | Persistence strategy — verify sqlx/tokio-postgres patterns |
| `database-schemas.md` | SQL schemas, migrations — verify PostgreSQL compatibility |
| `postgresql-implementation.md` | Connection management, query patterns — verify against current sqlx API |
| `jetstream-kv.md` | NATS JetStream KV — **critical**: verify against async-nats 0.46 JetStream API |
| `persistence-operations.md` | CRUD patterns, transactions |
| `storage-patterns.md` | Caching, write-ahead, event sourcing patterns |

---

### Batch 3: Transport & Security (2 parallel agents)

Depends on: Batch 1 complete.

**Agent 3A — Transport Layer** (5 files):
| File | Validation Focus |
|------|-----------------|
| `transport-layer-specifications.md` | Transport abstraction — verify trait design |
| `transport-core.md` | Core transport types and patterns |
| `nats-transport.md` | **Critical**: async-nats 0.34 → 0.46 migration. Verify Client, Subscriber, JetStream APIs |
| `grpc-transport.md` | Tonic patterns — verify against current tonic API |
| `http-transport.md` | Axum handlers, routing, middleware — verify against current axum API |

**Agent 3B — Security** (7 files):
| File | Validation Focus |
|------|-----------------|
| `security-framework.md` | Overall security architecture |
| `authentication-specifications.md` | Auth flows, token formats |
| `authentication-implementation.md` | JWT implementation — verify jsonwebtoken crate API |
| `authorization-specifications.md` | RBAC/ABAC model |
| `authorization-implementation.md` | Permission checking, middleware patterns |
| `security-integration.md` | Security ↔ transport integration, mTLS |
| `security-patterns.md` | Threat model, defense patterns — verify rustls/ring APIs |

---

### Batch 4: Operations, Testing & Agent Domains (2 parallel agents)

Depends on: Batch 1 complete.

**Agent 4A — Operations** (7 files):
| File | Validation Focus |
|------|-----------------|
| `deployment-architecture-specifications.md` | Kubernetes deployment — verify against current k8s API versions |
| `configuration-management.md` | Config loading, environment variables, secrets |
| `configuration-deployment-specifications.md` | Deployment configs, helm charts |
| `observability-monitoring-framework.md` | tracing/metrics crates — verify against current APIs |
| `process-management-specifications.md` | Process lifecycle, signals, graceful shutdown |
| `build-specifications.md` | Cargo build, cross-compilation, Docker |
| `PROCESS_MANAGEMENT_COMPLETION_SUMMARY.md` | Review for accuracy after other ops files updated |

**Agent 4B — Testing, Agent Domains & Research** (6 files):
| File | Validation Focus |
|------|-----------------|
| `testing-framework.md` | Test patterns — verify tokio::test, mockall, proptest APIs |
| `test-schemas.md` | Test data schemas |
| `SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md` | 9 agent types — verify completeness and internal consistency |
| `research/claude-code-cli-implementation-roadmap.md` | Claude integration feasibility — verify against current Claude API/SDK |
| `research/claude-code-cli-integration-plan.md` | Integration plan accuracy |
| `research/claude-code-cli-integration-summary.md` | Summary accuracy |

---

### Batch 5: Cross-Cutting Validation (single agent, final pass)

Depends on: All previous batches complete and changes applied.

**Agent 5 — Consistency & Integration**:
```
After all domain agents have completed their updates:

1. TERMINOLOGY SCAN
   - Extract all agent type names → verify uniform across all files
   - Extract all message type names → verify schemas match usage
   - Extract all crate versions → verify VERSION_REFERENCE.md applied consistently

2. CROSS-REFERENCE AUDIT
   - For every markdown link: verify target file exists and section anchor resolves
   - For every "see also" reference: verify bidirectional
   - Flag orphan files (referenced nowhere)

3. INTEGRATION POINT VALIDATION
   - Trace a message from agent spawn → communication → persistence → response
   - Verify the data types are consistent at each boundary
   - Verify error handling is specified at each boundary

4. GAP ANALYSIS
   - What components are specified but have no integration path?
   - What integration paths reference unspecified components?
   - What error conditions are unhandled?

Output: VALIDATION_REPORT.md with:
- Per-file status summary table
- Critical issues (blocks implementation)
- High-priority issues (should fix before implementation)
- Low-priority issues (nice to have)
- Updated readiness score (comparable to prior 82/100)
```

## Execution in Claude Code

### Running a Batch

Each agent is a Claude Code subagent launched with the `Agent` tool. Within each batch, agents run in parallel.

```
For each agent in the batch:
  Agent(
    subagent_type: "general-purpose",
    prompt: [agent system prompt] + [VERSION_REFERENCE.md content] + [file assignments],
    description: "Validate [domain] specs"
  )
```

### Rube MCP Usage Pattern per Agent

Each agent follows this lookup sequence for every external dependency it encounters:

```
1. RUBE_SEARCH_TOOLS → find Context7 tools (session from Batch 0)
2. CONTEXT7_MCP_RESOLVE_LIBRARY_ID(libraryName: "async-nats", query: "NATS Rust client")
3. CONTEXT7_MCP_QUERY_DOCS(libraryId: "/nats-io/nats.rs", query: "JetStream KV store API")
4. If Context7 insufficient → Tavily search: "async-nats 0.46 JetStream API changes"
5. If still insufficient → GitHub: check actual source at nats-io/nats.rs
```

### Applying Changes

Agents edit spec files directly in the working tree — each agent has non-overlapping file assignments so there are no conflicts. After each batch completes:
1. Review diffs
2. Stage and commit: `fix(spec): [domain] — update [what changed] per validation`
3. Run next batch (which sees updated files)

### Session Management

- Use a single Rube session ID across all agents in a batch to share connection state
- Pass `VERSION_REFERENCE.md` content to every agent to avoid redundant lookups

## Priority Order

If time/context is limited, validate in this order:

1. **Transport layer** — async-nats version gap is the largest risk
2. **Core architecture** — foundation everything else builds on
3. **Data management** — message schemas and persistence patterns
4. **Security** — JWT/TLS crate APIs
5. **Operations** — k8s and observability
6. **Testing/Research** — lowest risk of drift

## Success Criteria

- Every spec file has a status: `validated` | `updated` | `needs-review`
- All crate versions reconciled with current releases
- Zero broken cross-references
- All pseudocode examples use current API signatures
- Updated readiness score produced
- Changes committed in atomic, reviewable commits per batch
