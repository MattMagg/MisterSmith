# Data Model: Bounded Runtime Provider Selection

## New typed configuration

### `LlmConfig`

- `provider_kind`: selected runtime provider kind
- `model_id`: selected runtime model identifier

## Runtime-resolved state

### `RuntimeLlmSelection`

- canonical provider kind string for operator-facing metadata
- model id
- selected provider instance for the current binary support set

## Invariants

- default selection remains `openai_chatgpt` / `gpt-5.4`
- task, session, and autonomy metadata expose the resolved provider/model pair
- unsupported provider kinds fail before the runtime accepts work
