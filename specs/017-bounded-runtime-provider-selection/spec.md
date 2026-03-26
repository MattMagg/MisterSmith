# Feature Specification: Bounded Runtime Provider Selection

**Feature Branch**: `017-bounded-runtime-provider-selection`
**Created**: 2026-03-26
**Status**: Implemented
**Input**: `docs/current-state.md`,
`docs/plans/2026-03-21-post-packet-016-development-checkpoint.md`,
`docs/plans/2026-03-26-bounded-runtime-provider-selection.md`, and current app/config code in
`crates/mister-smith-config/`, `crates/mister-smith-app/`, and `crates/mister-smith-llm/`

## Current Truth & Scope

Current repo truth on `main` already includes:

- a provider-neutral LLM contract and `ModelRouter`
- runtime proof on `openai_chatgpt` / `gpt-5.4`
- deterministic `MockProvider`
- app-binary support for `openai_chatgpt` and `claude_subscription`

The remaining bounded gap is narrower than a router program:

- the runtime-backed task path still hardcodes `openai_chatgpt` and `gpt-5.4` instead of reading a
  typed runtime selection from framework configuration

This packet therefore freezes one bounded epic:

1. add typed runtime `llm` selection to framework configuration
2. let the runtime-backed task path honor that selection for the provider set the current binary
   actually ships
3. preserve the current default path, supervision, provenance, and operator-visible metadata

This is not a budget-control packet, not a JetStream KV control-loop packet, not a multi-provider
fan-out packet, not an external-agent packet, and not a queue-staging packet.

## User Scenarios & Testing

### User Story 1 - Keep today's live path as the default (Priority: P1)

An operator starts the runtime with no new LLM config and gets the same live default path as
today.

**Independent Test**: boot configuration defaults and confirm runtime selection resolves to
`openai_chatgpt` with `gpt-5.4`.

**Acceptance Scenarios**:

1. **Given** no explicit `llm` config, **When** the framework config loads, **Then** the runtime
   selection defaults to `openai_chatgpt` and `gpt-5.4`.
2. **Given** existing task/session/autonomy metadata flows, **When** the default runtime path is
   used, **Then** provider/model metadata remains present and unchanged in meaning.

### User Story 2 - Select a shipped non-default provider (Priority: P1)

An operator chooses another provider that the current binary actually supports and the runtime
boots on that provider without ad hoc env parsing in execution code.

**Independent Test**: apply config/env overlay for `claude_subscription` or `mock` and confirm the
runtime task service resolves that selection.

**Acceptance Scenarios**:

1. **Given** `MISTER_SMITH_LLM__PROVIDER_KIND=mock` and a non-default model id, **When** the
   runtime task service boots, **Then** it uses the mock provider and surfaces the configured model
   id in runtime metadata.
2. **Given** `claude_subscription` credentials are present, **When** the provider kind is set to
   `claude_subscription`, **Then** boot uses that provider and preserves supervision/provenance
   behavior.

### User Story 3 - Fail explicitly on unsupported binary/provider combinations (Priority: P2)

An operator asks the current binary for a provider it does not ship and gets a clear bounded
failure instead of silent fallback.

**Independent Test**: request `openai` or `anthropic` through config and confirm boot returns a
clear unsupported-provider error.

**Acceptance Scenarios**:

1. **Given** the current app binary lacks `openai` and `anthropic` provider features, **When** one
   of those kinds is selected, **Then** runtime bootstrap fails with a message that the binary does
   not support that provider.
2. **Given** unsupported-provider selection fails early, **When** the operator retries with a
   shipped provider kind, **Then** the runtime can boot normally.

### Edge Cases

- config omits `model_id`
- env overlay overrides a file-provided provider kind
- `mock` runs should not require external authentication
- `claude_subscription` must fail clearly when credentials are missing or expired
- unsupported provider kinds must not silently coerce to the default

## Requirements

### Functional Requirements

- **FR-001**: System MUST add a typed `llm` configuration section to `FrameworkConfig`.
- **FR-002**: System MUST default that selection to `openai_chatgpt` with `gpt-5.4`.
- **FR-003**: System MUST honor `MISTER_SMITH_LLM__PROVIDER_KIND` and
  `MISTER_SMITH_LLM__MODEL_ID` env overlays.
- **FR-004**: System MUST keep current runtime supervision/provenance behavior intact.
- **FR-005**: System MUST support runtime selection only for providers the current app binary ships:
  `openai_chatgpt`, `claude_subscription`, and `mock`.
- **FR-006**: System MUST fail explicitly when configuration selects an unsupported provider kind
  for the current binary.
- **FR-007**: System MUST preserve provider/model metadata on task, session, and autonomy surfaces.
- **FR-008**: System MUST NOT widen into multi-provider fan-out or new routing policies.
- **FR-009**: System MUST NOT widen into budget-control, JetStream KV, or external-agent follow-on
  work.

### Key Entities

- **LlmConfig**: typed framework configuration for runtime provider/model selection
- **RuntimeLlmSelection**: resolved provider/model choice the runtime task path uses at boot
- **UnsupportedRuntimeProvider**: explicit early failure for provider kinds the current binary does
  not ship

## Success Criteria

- **SC-001**: default boot path remains `openai_chatgpt` / `gpt-5.4`
- **SC-002**: `mock` and `claude_subscription` can be selected through the normal config path
- **SC-003**: task/session/autonomy metadata show the selected provider/model rather than fixed
  constants
- **SC-004**: unsupported provider kinds fail explicitly with actionable messaging
- **SC-005**: the packet stays bounded to config plus runtime selection and does not widen into a
  router-control program
