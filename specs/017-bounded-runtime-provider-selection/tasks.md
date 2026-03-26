# Tasks: Bounded Runtime Provider Selection

## T1. Freeze config and packet framing

- [x] Add packet `017` artifacts
- [x] Add repo planning note in `docs/plans/`
- [x] Register the slice in validated backlog tracking

## T2. Add typed framework config for runtime provider selection

- [x] Add `LlmConfig` to `mister-smith-config`
- [x] Add defaults and validation
- [x] Add env overlay support for provider kind and model id
- [x] Add config tests

## T3. Wire runtime selection into app bootstrap

- [x] Resolve runtime provider/model from `FrameworkConfig`
- [x] Build a provider for shipped provider kinds only
- [x] Preserve today's default
- [x] Fail explicitly on unsupported provider kinds

## T4. Preserve task/session/autonomy metadata continuity

- [x] Replace fixed provider/model constants in runtime metadata paths
- [x] Keep session/task/autonomy output provider/model fields accurate
- [x] Add targeted app tests

## T5. Refresh state-bearing docs and validate

- [x] Update `docs/current-state.md` if shipped truth changed
- [x] Run `cargo test -p mister-smith-config`
- [x] Run `cargo test -p mister-smith-app`
- [x] Run `cargo build --workspace`
