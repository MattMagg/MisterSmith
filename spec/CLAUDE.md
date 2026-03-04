# Specifications

Framework specifications for Mister Smith. These documents define the contract for implementation — agent types, message formats, architecture patterns, and integration points.

## Structure

```
spec/
├── core-architecture/   # System design, async patterns, supervision trees, types (21 files)
├── data-management/     # Agent orchestration, message schemas, persistence (19 files)
├── transport/           # NATS, gRPC, HTTP transport layers (5 files)
├── security/            # Authentication, authorization, TLS, patterns (7 files)
├── operations/          # Deployment, monitoring, configuration, build scripts (7 + scripts/)
├── agent-domains/       # Consolidated agent type analysis (1 file)
├── testing/             # Test framework and schemas (2 files)
└── research/            # LLM CLI integration analysis (3 files, legacy — to be archived)
```

## Document Interconnections

**High-impact files** — changes to these cascade across the spec:

- `core-architecture/system-architecture.md` — foundation for all specs
- `core-architecture/type-definitions.md` — core types referenced everywhere
- `data-management/message-schemas.md` — message formats used across system
- `agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md` — agent type definitions

**Cross-reference patterns** — when modifying:

- Agent-related changes → check `agent-domains/` and `data-management/agent-*.md`
- Message changes → update `message-schemas.md`, `core-message-schemas.md`, `workflow-message-schemas.md`
- Architecture changes → verify impact on `integration-*.md` files
- Security changes → cross-reference with `transport/` specifications

## Quality Standards

- **Technical accuracy**: Specifications must be implementable with stated technologies
- **Consistency**: Terms, patterns, agent names, and version numbers uniform across files
- **Completeness**: Minimize placeholder or unresolved sections
- **Feasibility**: Patterns must follow Rust best practices for the stated dependency versions

### Terminology Consistency

- **Agent types**: Use exact names from `SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md`
- **Messages**: Match schemas in `message-schemas.md`
- **Async patterns**: Align with `async-patterns.md` and `tokio-runtime.md`
- **Versions**: Tokio 1.49, async-nats 0.46, Axum 0.8, Tonic 0.14 (see `core-architecture/dependency-specifications.md` for full matrix)

## Navigation

Start with `core-architecture/system-architecture.md` for the architecture overview, then `component-architecture.md` for structure, then `integration-patterns.md` for connections between components.

For agent specifications, start with `agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md`, then read the lifecycle and orchestration files in `data-management/`.
