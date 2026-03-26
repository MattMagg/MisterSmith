# Data Model: Budget-Backed Runtime Routing Control Loop

## New typed runtime config

### `RuntimeRoutingProfile`

- `policy`: one bounded runtime routing policy, initially `cascade`
- `budget_root`: canonical budget key root used by the runtime task path
- `tiers`: ordered list of shipped provider tiers for runtime registration and fallback

### `RuntimeProviderTier`

- `label`: operator-visible tier label
- `provider_kind`: shipped provider kind for the tier
- `model_id`: model identifier for that provider
- `metadata`: optional provider tier metadata such as preferred tier name

## Runtime-resolved state

### `RuntimeBudgetState`

- budget key
- token limit
- used tokens
- budget policy
- CAS revision for reconciliation

### `RuntimeRoutingDecisionView`

- routing policy used for the step
- accepted tier label
- selected provider/model
- budget-aware checkpoints and fallback rationale

## Invariants

- omitting the routing profile preserves today's single-provider `openai_chatgpt` / `gpt-5.4`
  runtime path
- configured runtime tiers must use only providers the current binary actually ships
- budget reconciliation must remain CAS-safe and must not leak tokens on failed attempts
- operator-visible routing evidence must stay consistent with the actual accepted runtime tier
