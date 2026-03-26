# Research Notes: Bounded Runtime Provider Selection

## Current binary support

- `mister-smith-app` enables `mister-smith-llm` features `openai-chatgpt` and
  `claude-subscription`
- `MockProvider` is always available in `mister-smith-llm`
- `openai` and `anthropic` provider implementations exist in the repo but are not enabled in the
  current app binary

## Current gap

- the runtime task path uses a provider-neutral router type but hardcodes one provider and one
  model in `crates/mister-smith-app/src/execution.rs`
- `docs/current-state.md` still correctly records that limitation on the shipped path

## Bounded conclusion

The next legitimate slice is to make the shipped runtime path configurable across the providers the
binary already contains, while keeping defaults and supervision unchanged.
