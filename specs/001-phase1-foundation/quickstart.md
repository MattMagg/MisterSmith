# Quickstart: Validate Phase 1 Foundation Contracts

## Prerequisites

- Run from repository root: `/Users/matthewmaggio/Mister-Smith`
- Active branch: `001-phase1-foundation`
- Rust toolchain with `cargo` available
- Node.js tooling with `npx` available
- `rg` (ripgrep) available

## 1. Contract Consistency Checks

```bash
rg -n "pub enum AgentState|pub enum AgentAvailability|pub enum MessagePriority|pub enum AgentType|pub enum RestartPolicy|pub enum RestartScope" spec/core-architecture/type-definitions.md
rg -n "pub struct RestartPolicy\\b|pub enum RestartPolicy\\b" spec/data-management spec/core-architecture
rg -n "pub trait Tool" spec/core-architecture/module-organization-type-system.md spec/core-architecture/system-integration.md
rg -n "MessagePriority" spec/testing/test-schemas.md spec/data-management/message-schemas.md spec/transport/nats-transport.md
```

Expected outcome:

- Canonical definitions are present in expected files.
- No unresolved active-reference naming collisions.

## 2. Gate 1 Compile Checks

```bash
cargo build -p mister-smith-core
cargo build -p mister-smith-config
```

Expected outcome:

- Both crates compile cleanly.

## 3. Feature Artifact Quality Checks

```bash
npx markdownlint-cli2 "specs/001-phase1-foundation/*.md" --config .markdownlint.json
npx markdownlint-cli2 "specs/001-phase1-foundation/contracts/*.md" --config .markdownlint.json
npx markdownlint-cli2 "specs/001-phase1-foundation/checklists/*.md" --config .markdownlint.json
```

Expected outcome:

- Zero markdown lint errors.

## 4. Interpretation Rules

- If any command fails, do not proceed to implementation planning for later phases.
- Resolve failures by aligning spec/plan/tasks to canonical Phase 1 contracts.
