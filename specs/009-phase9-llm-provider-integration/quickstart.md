# Quickstart: Phase 9 — LLM Provider Integration

## Prerequisites

- Phase 1-7 workspace crates present and building
- Active feature directory: `specs/009-phase9-llm-provider-integration/`
- Branch: `009-phase9-llm-provider-integration`
- For future real-provider validation:
  - `ANTHROPIC_API_KEY`
  - `OPENAI_API_KEY`
- For ChatGPT-subscription validation:
  - `codex` available on `PATH` with `app-server` support
- For future Gate 9 orchestration validation: local NATS and PostgreSQL services available if the
  implementation exercises existing ToolBus security or audit boundaries

## Source Map

| Source | Quickstart impact |
| ------ | ----------------- |
| `spec/data-management/agent-orchestration.md` §10.4 | Grounds the Gate 9 Planner-to-Orchestrator-to-Worker scenario and existing LLM coordination seams. |
| `spec/data-management/message-schemas.md` §5 | Confirms prep-mode validation does not include hook-event subjects or `llm.hooks.*` workflows. |
| `spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md` §15 | Keeps Neural/AI Operations flows out of this validation path. |
| `spec/core-architecture/type-definitions.md` | Anchors the provider-neutral types and feature-gated role boundaries shown in the usage sketch. |
| `spec/core-architecture/async-patterns.md` | Keeps `ToolBus::to_tool_definitions()` and `execute_tool_call()` inside the existing tool patterns. |
| `spec/core-architecture/coding-standards.md` | Keeps the validation tiers tied to testing, permission, timeout, audit, and typed-error expectations. |

## Validate The Planning Artifacts

These commands should pass immediately in prep mode:

```bash
./.specify/scripts/bash/check-prerequisites.sh --json
npx markdownlint-cli2 "specs/009-phase9-llm-provider-integration/**/*.md" \
  "specs/009-phase9-llm-provider-integration/*.md" \
  --config .markdownlint.json
```

Expected result:

- `AVAILABLE_DOCS` includes `research.md`, `data-model.md`, `contracts/`, and `quickstart.md`
- markdownlint returns `0 error(s)`

## Planned Build Flow After Implementation

Phase 9 implementation is expected to validate in this order:

```bash
# 1. Core provider-neutral contract and mock behavior
cargo test -p mister-smith-llm

# 2. Anthropic adapter when credentials are available
ANTHROPIC_API_KEY=... cargo test -p mister-smith-llm --features anthropic -- --ignored

# 3. OpenAI adapter when credentials are available
OPENAI_API_KEY=... cargo test -p mister-smith-llm --features openai -- --ignored

# 4. ChatGPT-backed OpenAI path through Codex app-server
cargo test -p mister-smith-llm --features openai-chatgpt
cargo test -p mister-smith-app

# 5. Agent bridge and Gate 9 orchestration path
cargo test -p mister-smith-agents --features llm gate9 -- --ignored --nocapture
```

The exact test names can be refined during implementation, but the validation tiers must remain:
mock contract tests, real-provider integration tests, Codex app-server auth or turn validation, and
Gate 9 orchestration validation.

## Usage Sketch

### Completion Through The Shared Interface

```rust
use mister_smith_llm::{CompletionRequest, ModelProvider, MockProvider};

let provider = MockProvider::default();

let response = provider
    .complete(CompletionRequest {
        messages: vec![],
        system: Some("decompose work into subtasks".into()),
        tools: None,
        temperature: None,
        max_tokens: Some(512),
        stop_sequences: None,
        metadata: serde_json::json!({}),
    })
    .await?;
```

### Tool Definitions From ToolBus

```rust
use mister_smith_agents::ToolBus;

let tool_definitions = tool_bus.to_tool_definitions();
```

### Tool Call Round-Trip

```rust
let tool_result = tool_bus.execute_tool_call(/* caller context */, &tool_call).await?;
```

The bridge contract is behavioral: tool execution must preserve the existing ToolBus permission,
timeout, and audit boundary even if the final method signature evolves during implementation.

### ChatGPT Subscription Login And Status

```bash
cargo run -p mister-smith-app -- auth openai-chatgpt login
cargo run -p mister-smith-app -- auth openai-chatgpt status
```

Expected behavior:

- `login` starts Codex app-server, requests `account/login/start` with `type = "chatgpt"`, opens
  the returned browser URL, prints the same URL for manual fallback, and waits for
  `account/login/completed` or a confirming `account/updated`.
- The login flow must correlate `account/login/completed` to the exact returned `loginId` and
  cancel the pending login with `account/login/cancel` if the browser flow times out before
  completion.
- `status` reports the current Codex app-server account state from `account/read` and whether
  Mister Smith can use the `openai_chatgpt` backend.
- `requiresOpenaiAuth = true` in `account/read` does not mean ChatGPT login failed; it only means
  the active provider requires OpenAI authentication. The ChatGPT path keys off the returned
  account type instead.
- If Codex is currently authenticated with an API key instead of a ChatGPT subscription, the status
  command must report that explicitly and direct the operator to run
  `mister-smith auth openai-chatgpt login`.
- If `account/read` returns `account = null` and `requiresOpenaiAuth = false`, the status command
  must report that the active Codex provider does not currently require OpenAI authentication
  instead of treating that state as a failed ChatGPT login.

If login is missing or expired, `OpenAiChatGptProvider` must return a typed authentication error
that directs the operator to run `mister-smith auth openai-chatgpt login`.

## Gate 9 Scenario

The roadmap Gate 9 validation for Phase 9 is:

1. A Planner receives a high-level task.
2. The Planner calls a real LLM through `ModelProvider`.
3. The response contains a structured subtask decomposition.
4. The Orchestrator assigns subtasks to Worker agents through the existing agent-system flow.
5. Tool calls, when requested by the model, round-trip through the ToolBus.
6. The same flow succeeds with Anthropic and the OpenAI API-key backend.
7. The ChatGPT-backed OpenAI path validates completion and streaming through Codex app-server and
   returns typed unsupported-capability errors for embeddings and tool calling without introducing a
   second auth stack into Mister Smith.
8. Codex app-server turns normalize the authoritative `item/completed` agent message, propagated
   token-usage updates, and any `model/rerouted` notification before the response leaves the
   provider boundary.

## Blocker Review Before `/speckit.tasks`

Before task generation, confirm that these remain visible as blockers or prerequisites rather than
Phase 9 scope:

- ToolBus permission and audit hardening
- Router balancing strategies
- Memory metadata and versioning
- Heartbeat receiver and failure detection
- Supervisor delegation to Phase 3 supervision
- Priority mailbox wiring
